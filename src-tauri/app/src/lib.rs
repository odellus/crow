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
    pub fn new(working_dir: PathBuf) -> Self {
        let session_store = SessionStore::new();
        if let Err(e) = session_store.init_sync() {
            eprintln!("Failed to initialize session storage: {}", e);
        }

        Self {
            session_store,
            tool_registry: Arc::new(ToolRegistry::new()),
            agent_registry: Arc::new(AgentRegistry::new()),
            lock_manager: Arc::new(SessionLockManager::new()),
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

    let response = executor
        .execute_turn(&session_id, "build", &state.working_dir, user_parts)
        .await
        .map_err(|e| format!("Agent execution failed: {}", e))?;

    for part in &response.parts {
        let _ = on_event.send(StreamEvent::Part { part: part.clone() });
    }

    let assistant_msg_id = match &response.info {
        Message::Assistant { id, .. } => id.clone(),
        _ => String::new(),
    };
    let _ = on_event.send(StreamEvent::Complete {
        message_id: assistant_msg_id.clone(),
    });

    Ok(assistant_msg_id)
}

// ============================================================================
// File Commands
// ============================================================================

#[tauri::command]
async fn read_file(path: String) -> Result<String, String> {
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
async fn write_file(path: String, content: String) -> Result<(), String> {
    tokio::fs::write(&path, content)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))
}

#[tauri::command]
async fn list_directory(path: String) -> Result<Vec<String>, String> {
    let mut entries = tokio::fs::read_dir(&path)
        .await
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    let mut files = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read entry: {}", e))?
    {
        if let Some(name) = entry.file_name().to_str() {
            files.push(name.to_string());
        }
    }

    Ok(files)
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
    let working_dir = std::env::args()
        .skip_while(|arg| arg != "--project-dir")
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    println!("Crow starting with working directory: {:?}", working_dir);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new(working_dir))
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            create_session,
            get_session,
            get_messages,
            send_message,
            read_file,
            write_file,
            list_directory,
            get_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
