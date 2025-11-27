//! Crow Tauri Application
//!
//! This crate provides the Tauri command layer that bridges the frontend
//! with the crow_core backend logic.

use crow_core::{
    session::{MessageWithParts, SessionLockManager, SessionStore},
    types::{Message, MessageTime, Part, Session},
    AgentExecutor, AgentRegistry, ProviderClient, ProviderConfig, ToolRegistry,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{ipc::Channel, State};

// ============================================================================
// Global State
// ============================================================================

/// Application state managed by Tauri
pub struct AppState {
    pub session_store: SessionStore,
    pub tool_registry: Arc<ToolRegistry>,
    pub agent_registry: Arc<AgentRegistry>,
    pub lock_manager: Arc<SessionLockManager>,
    pub working_dir: PathBuf,
}

impl AppState {
    pub async fn new(working_dir: PathBuf) -> Self {
        let session_store = Arc::new(SessionStore::new());
        if let Err(e) = session_store.init_sync() {
            eprintln!("Failed to initialize session storage: {}", e);
        }

        let agent_registry = Arc::new(AgentRegistry::new());
        let lock_manager = Arc::new(SessionLockManager::new());
        let provider_config = ProviderConfig::moonshot();

        // Use new_with_deps to get TaskTool for subagent spawning
        let tool_registry = ToolRegistry::new_with_deps(
            session_store.clone(),
            agent_registry.clone(),
            lock_manager.clone(),
            provider_config,
        )
        .await;

        Self {
            session_store: Arc::try_unwrap(session_store).unwrap_or_else(|arc| (*arc).clone()),
            tool_registry,
            agent_registry,
            lock_manager,
            working_dir,
        }
    }
}

// ============================================================================
// Session Commands
// ============================================================================

#[tauri::command]
async fn list_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    state.session_store.list(None)
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
    #[serde(rename = "parentID")]
    pub parent_id: Option<String>,
}

#[tauri::command]
async fn create_session(
    state: State<'_, AppState>,
    request: CreateSessionRequest,
) -> Result<Session, String> {
    let directory = state.working_dir.to_string_lossy().to_string();
    state
        .session_store
        .create(directory, request.parent_id, request.title)
}

#[tauri::command]
async fn get_session(state: State<'_, AppState>, session_id: String) -> Result<Session, String> {
    state.session_store.get(&session_id)
}

#[tauri::command]
async fn get_messages(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<MessageWithParts>, String> {
    state.session_store.get_messages(&session_id)
}

// ============================================================================
// Message Streaming Commands
// ============================================================================

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "part")]
    Part { part: Part },
    #[serde(rename = "text_delta")]
    TextDelta { part_id: String, delta: String },
    #[serde(rename = "thinking")]
    Thinking { text: String },
    #[serde(rename = "tool_start")]
    ToolStart { tool_name: String, tool_id: String },
    #[serde(rename = "tool_end")]
    ToolEnd {
        tool_id: String,
        success: bool,
        output: String,
    },
    #[serde(rename = "complete")]
    Complete { message_id: String },
    #[serde(rename = "error")]
    Error { message: String },
}

#[tauri::command]
async fn send_message(
    state: State<'_, AppState>,
    session_id: String,
    content: String,
    on_event: Channel<StreamEvent>,
) -> Result<String, String> {
    use crow_core::agent::ExecutionEvent;
    use tokio::sync::mpsc;

    let user_msg_id = format!("msg-user-{}", uuid::Uuid::new_v4());
    let user_parts = vec![Part::Text {
        id: format!("part-{}", uuid::Uuid::new_v4()),
        session_id: session_id.clone(),
        message_id: user_msg_id.clone(),
        text: content.clone(),
    }];

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let user_message = MessageWithParts {
        info: Message::User {
            id: user_msg_id.clone(),
            session_id: session_id.clone(),
            time: MessageTime {
                created: now,
                completed: Some(now),
            },
            summary: None,
            metadata: None,
        },
        parts: user_parts.clone(),
    };

    state
        .session_store
        .add_message(&session_id, user_message)
        .map_err(|e| format!("Failed to add user message: {}", e))?;

    let provider_config = ProviderConfig::moonshot();
    let provider = ProviderClient::new(provider_config)
        .map_err(|e| format!("Failed to create provider: {}", e))?;

    let executor = AgentExecutor::new(
        provider,
        state.tool_registry.clone(),
        Arc::new(state.session_store.clone()),
        state.agent_registry.clone(),
        state.lock_manager.clone(),
    );

    // Create channel for streaming events
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<ExecutionEvent>();

    // Spawn the executor in a separate task
    // Note: We pass empty user_parts since the message is already stored above.
    // The executor will read it from the session store via build_llm_context.
    let session_id_clone = session_id.clone();
    let working_dir = state.working_dir.clone();
    let executor_handle = tokio::spawn(async move {
        executor
            .execute_turn_streaming(
                &session_id_clone,
                "build",
                &working_dir,
                vec![], // Already stored above
                event_tx,
            )
            .await
    });

    // Track accumulated text for streaming deltas
    let mut text_buffers: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    // Forward events to the Tauri channel
    while let Some(event) = event_rx.recv().await {
        match event {
            ExecutionEvent::TextDelta { id, delta } => {
                // Accumulate text and send delta
                text_buffers.entry(id.clone()).or_default().push_str(&delta);
                let _ = on_event.send(StreamEvent::TextDelta { part_id: id, delta });
            }
            ExecutionEvent::Part(part) => {
                let _ = on_event.send(StreamEvent::Part { part });
            }
            ExecutionEvent::Complete(msg) => {
                let msg_id = match &msg.info {
                    Message::Assistant { id, .. } => id.clone(),
                    _ => String::new(),
                };
                let _ = on_event.send(StreamEvent::Complete { message_id: msg_id });
            }
            ExecutionEvent::Error(err) => {
                let _ = on_event.send(StreamEvent::Error { message: err });
            }
        }
    }

    // Wait for executor to complete
    let response = executor_handle
        .await
        .map_err(|e| format!("Executor task failed: {}", e))?
        .map_err(|e| format!("Agent execution failed: {}", e))?;

    let assistant_msg_id = match &response.info {
        Message::Assistant { id, .. } => id.clone(),
        _ => String::new(),
    };

    Ok(assistant_msg_id)
}

// ============================================================================
// File Commands
// ============================================================================

#[tauri::command]
async fn read_file(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let full_path = if std::path::Path::new(&path).is_relative() {
        state.working_dir.join(&path)
    } else {
        PathBuf::from(&path)
    };

    tokio::fs::read_to_string(&full_path)
        .await
        .map_err(|e| format!("Failed to read file '{}': {}", full_path.display(), e))
}

#[tauri::command]
async fn write_file(path: String, content: String) -> Result<(), String> {
    tokio::fs::write(&path, content)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))
}

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileListResponse {
    pub path: String,
    pub entries: Vec<FileEntry>,
    pub count: usize,
}

#[tauri::command]
async fn list_directory(
    state: State<'_, AppState>,
    path: String,
) -> Result<FileListResponse, String> {
    // Resolve relative paths against the working directory
    let full_path = if path == "." || path.is_empty() {
        state.working_dir.clone()
    } else if std::path::Path::new(&path).is_relative() {
        state.working_dir.join(&path)
    } else {
        PathBuf::from(&path)
    };

    let mut read_dir = tokio::fs::read_dir(&full_path)
        .await
        .map_err(|e| format!("Failed to read directory '{}': {}", full_path.display(), e))?;

    let mut entries = Vec::new();
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read entry: {}", e))?
    {
        if let Some(name) = entry.file_name().to_str() {
            let entry_path = entry.path();
            let metadata = entry.metadata().await.ok();
            let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = metadata.and_then(|m| if m.is_file() { Some(m.len()) } else { None });

            entries.push(FileEntry {
                name: name.to_string(),
                path: entry_path.to_string_lossy().to_string(),
                is_dir,
                size,
            });
        }
    }

    // Sort: directories first, then by name
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    let count = entries.len();
    Ok(FileListResponse {
        path,
        entries,
        count,
    })
}

// ============================================================================
// Shell Commands (for Terminal)
// ============================================================================

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ShellOutput {
    #[serde(rename = "stdout")]
    Stdout { data: String },
    #[serde(rename = "stderr")]
    Stderr { data: String },
    #[serde(rename = "exit")]
    Exit { code: i32 },
}

#[tauri::command]
async fn run_shell_command(
    command: String,
    cwd: String,
    on_output: Channel<ShellOutput>,
) -> Result<String, String> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    let shell_id = format!("shell-{}", uuid::Uuid::new_v4());

    // Spawn the command
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(&command)
        .current_dir(&cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn command: {}", e))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let on_output_stdout = on_output.clone();
    let on_output_stderr = on_output.clone();

    // Stream stdout
    if let Some(stdout) = stdout {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = on_output_stdout.send(ShellOutput::Stdout {
                    data: format!("{}\r\n", line),
                });
            }
        });
    }

    // Stream stderr
    if let Some(stderr) = stderr {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = on_output_stderr.send(ShellOutput::Stderr {
                    data: format!("{}\r\n", line),
                });
            }
        });
    }

    // Wait for exit and send exit code
    tokio::spawn(async move {
        match child.wait().await {
            Ok(status) => {
                let code = status.code().unwrap_or(-1);
                let _ = on_output.send(ShellOutput::Exit { code });
            }
            Err(_) => {
                let _ = on_output.send(ShellOutput::Exit { code: -1 });
            }
        }
    });

    Ok(shell_id)
}

// ============================================================================
// Config Commands
// ============================================================================

#[tauri::command]
async fn get_config() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "provider": "moonshot",
        "model": "moonshot-v1-8k"
    }))
}

// ============================================================================
// Application Entry Point
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Use PWD (the shell's working directory) if available, otherwise fall back to current_dir
    // --project-dir flag can override both
    let working_dir = std::env::args()
        .skip_while(|arg| arg != "--project-dir")
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var("PWD")
                .map(PathBuf::from)
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        });

    println!("Crow starting with working directory: {:?}", working_dir);

    // Create async runtime for initialization
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let app_state = rt.block_on(AppState::new(working_dir));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            create_session,
            get_session,
            get_messages,
            send_message,
            read_file,
            write_file,
            list_directory,
            run_shell_command,
            get_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
