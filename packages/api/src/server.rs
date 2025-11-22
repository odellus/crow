//! HTTP server that mirrors OpenCode's REST API
//! This provides the same endpoints as `opencode serve`

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, Sse},
    routing::{delete, get, patch, post},
    Json, Router,
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;

use crate::{
    agent::{AgentExecutor, AgentRegistry},
    config::ConfigLoader,
    providers::ProviderConfig,
    session::{MessageWithParts, SessionLockManager, SessionStore},
    tools::ToolRegistry,
    types::{Message, MessageTime, Part, Session},
};

/// Server state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub session_store: Arc<SessionStore>,
    pub tool_registry: Arc<ToolRegistry>,
    pub agent_registry: Arc<AgentRegistry>,
    pub lock_manager: Arc<SessionLockManager>,
}

/// Build the Axum router with OpenCode-compatible endpoints
pub fn create_router() -> Router {
    let state = AppState {
        session_store: Arc::new(SessionStore::new()),
        tool_registry: Arc::new(ToolRegistry::new()),
        agent_registry: Arc::new(AgentRegistry::new()),
        lock_manager: Arc::new(SessionLockManager::new()),
    };

    Router::new()
        // Session endpoints
        .route("/session", get(list_sessions))
        .route("/session", post(create_session))
        .route("/session/{id}", get(get_session))
        .route("/session/{id}", delete(delete_session))
        .route("/session/{id}", patch(update_session))
        // Message endpoints
        .route("/session/{id}/message", get(list_messages))
        .route("/session/{id}/message", post(send_message))
        // Test endpoint for tools
        .route("/test/tool/{name}", post(test_tool))
        // TODO: Other endpoints
        // .route("/config", get(get_config))
        // .route("/config/providers", get(list_providers))
        // .route("/agent", get(list_agents))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Initialize storage and build router
pub async fn create_router_with_storage() -> Result<Router, String> {
    // Load configuration
    let config_loader = ConfigLoader::new();
    let config = config_loader.load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load config, using defaults: {}", e);
        crate::config::Config::default()
    });

    // Get provider config from loaded config
    let provider_config = get_provider_config_from_config(&config);

    tracing::info!(
        "Loaded config: model={:?}, provider={}",
        config.model,
        provider_config.name
    );

    let session_store = Arc::new(SessionStore::new());

    // Initialize storage and load existing data
    session_store.init().await?;

    tracing::info!(
        "Storage initialized, loaded {} sessions",
        session_store.list(None).unwrap_or_default().len()
    );

    let agent_registry = Arc::new(AgentRegistry::new());
    let lock_manager = Arc::new(SessionLockManager::new());

    // Create tool registry with dependencies for Task tool
    let tool_registry = ToolRegistry::new_with_deps(
        session_store.clone(),
        agent_registry.clone(),
        lock_manager.clone(),
        provider_config,
    )
    .await;

    let state = AppState {
        session_store,
        tool_registry,
        agent_registry,
        lock_manager,
    };

    Ok(Router::new()
        // Session endpoints
        .route("/session", get(list_sessions))
        .route("/session", post(create_session))
        .route("/session/{id}", get(get_session))
        .route("/session/{id}", delete(delete_session))
        .route("/session/{id}", patch(update_session))
        .route("/session/{id}/fork", post(fork_session))
        .route("/session/{id}/abort", post(abort_session))
        .route("/session/{id}/revert", post(revert_session))
        .route("/session/{id}/unrevert", post(unrevert_session))
        .route("/session/{id}/children", get(get_session_children))
        .route("/session/{id}/todo", get(get_session_todo))
        // Message endpoints
        .route("/session/{id}/message", get(list_messages))
        .route("/session/{id}/message", post(send_message))
        .route("/session/{id}/message/{message_id}", get(get_message))
        // Streaming message endpoint
        .route("/session/{id}/message/stream", post(send_message_stream))
        // Dual-agent endpoint
        .route("/session/dual", post(create_dual_session))
        // File endpoints
        .route("/file", get(list_files))
        .route("/file/content", get(read_file))
        // Config endpoints
        .route("/config", get(get_config))
        .route("/config/providers", get(list_providers))
        // Tool and agent endpoints
        .route("/experimental/tool/ids", get(list_tool_ids))
        .route("/experimental/tool", get(list_tools))
        .route("/agent", get(list_agents))
        // Test endpoint for tools
        .route("/test/tool/{name}", post(test_tool))
        // Global event stream
        .route("/event", get(event_stream))
        .layer(CorsLayer::permissive())
        .with_state(state))
}

/// GET /session - List all sessions
async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<Vec<Session>>, (StatusCode, String)> {
    state
        .session_store
        .list(None)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

/// POST /session - Create a new session
async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<Session>, (StatusCode, String)> {
    // Use directory from request, or default to server's cwd
    let directory = if let Some(dir) = req.directory {
        dir
    } else {
        std::env::current_dir()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .to_string_lossy()
            .to_string()
    };

    state
        .session_store
        .create(directory, req.parent_id, req.title)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

/// GET /session/{id} - Get a session by ID
async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Session>, (StatusCode, String)> {
    state
        .session_store
        .get(&id)
        .map(Json)
        .map_err(|_| (StatusCode::NOT_FOUND, format!("Session not found: {}", id)))
}

/// DELETE /session/{id} - Delete a session
async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .session_store
        .delete(&id)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|_| (StatusCode::NOT_FOUND, format!("Session not found: {}", id)))
}

/// PATCH /session/{id} - Update a session
#[derive(serde::Deserialize)]
struct UpdateSessionRequest {
    title: Option<String>,
}

async fn update_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSessionRequest>,
) -> Result<Json<Session>, (StatusCode, String)> {
    state
        .session_store
        .update(&id, req.title)
        .map(Json)
        .map_err(|_| (StatusCode::NOT_FOUND, format!("Session not found: {}", id)))
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateSessionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "parentID", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
}

// ============================================================================
// Message Endpoints
// ============================================================================

/// GET /session/{id}/message - List all messages in a session
async fn list_messages(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<MessageWithParts>>, (StatusCode, String)> {
    state
        .session_store
        .get_messages(&id)
        .map(Json)
        .map_err(|e| {
            if e.contains("not found") {
                (StatusCode::NOT_FOUND, e)
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e)
            }
        })
}

/// POST /session/{id}/message - Send a message to a session
/// Input format matches OpenCode API - client sends minimal data, server generates IDs
#[derive(serde::Deserialize)]
struct PartInput {
    #[serde(rename = "type")]
    part_type: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(serde::Deserialize)]
struct SendMessageRequest {
    #[serde(default = "default_agent")]
    agent: String,
    parts: Vec<PartInput>,
    #[serde(rename = "noReply", default)]
    no_reply: bool,
}

fn default_agent() -> String {
    "build".to_string()
}

async fn send_message(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<MessageWithParts>, (StatusCode, String)> {
    // Generate message ID
    let message_id = format!("msg-{}", uuid::Uuid::new_v4());

    // Get current time
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .as_millis() as u64;

    // Convert PartInput to Part objects with generated IDs
    let parts: Vec<Part> = req
        .parts
        .iter()
        .map(|input| {
            let part_id = format!("prt-{}", uuid::Uuid::new_v4());
            match input.part_type.as_str() {
                "text" => Part::Text {
                    id: part_id,
                    session_id: session_id.clone(),
                    message_id: message_id.clone(),
                    text: input.text.clone().unwrap_or_default(),
                },
                "thinking" => Part::Thinking {
                    id: part_id,
                    session_id: session_id.clone(),
                    message_id: message_id.clone(),
                    text: input.text.clone().unwrap_or_default(),
                },
                _ => Part::Text {
                    id: part_id,
                    session_id: session_id.clone(),
                    message_id: message_id.clone(),
                    text: input.text.clone().unwrap_or_default(),
                },
            }
        })
        .collect();

    // Create user message
    let user_message = MessageWithParts {
        info: Message::User {
            id: message_id.clone(),
            session_id: session_id.clone(),
            time: MessageTime {
                created: now,
                completed: None,
            },
            summary: None,
            metadata: None,
        },
        parts: parts.clone(),
    };

    // Store the user message
    state
        .session_store
        .add_message(&session_id, user_message.clone())
        .map_err(|e| {
            if e.contains("not found") {
                (StatusCode::NOT_FOUND, e)
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e)
            }
        })?;

    // If noReply is true, just return the user message (matching OpenCode behavior)
    if req.no_reply {
        return Ok(Json(user_message));
    }

    // Execute agent to get response
    // Load config to get default model
    let config_loader = ConfigLoader::new();
    let config = config_loader.load().unwrap_or_default();
    let (_provider_id, model_id) = config.get_default_model();

    let mut provider_config = ProviderConfig::moonshot();
    provider_config.default_model = model_id;

    let provider = crate::providers::ProviderClient::new(provider_config)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let executor = AgentExecutor::new(
        provider,
        state.tool_registry.clone(),
        state.session_store.clone(),
        state.agent_registry.clone(),
        state.lock_manager.clone(),
    );

    // Get working directory from session
    let session = state
        .session_store
        .get(&session_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let working_dir = std::path::PathBuf::from(&session.directory);

    let assistant_message = executor
        .execute_turn(&session_id, &req.agent, &working_dir, parts)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(assistant_message))
}

/// POST /session/{id}/message/stream - Send a message with streaming response
async fn send_message_stream(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Create a channel for streaming events
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // Spawn task to execute agent and stream events
    tokio::spawn(async move {
        // Acquire session lock for abort support
        let lock = match state.lock_manager.acquire(&session_id) {
            Ok(lock) => lock,
            Err(e) => {
                let _ = tx.send(Ok(Event::default()
                    .event("error")
                    .data(format!("{{\"error\":\"Failed to acquire lock: {}\"}}", e))));
                return;
            }
        };

        // Send start event
        let _ = tx.send(Ok(Event::default()
            .event("message.start")
            .data("{\"status\":\"starting\"}")));

        // Generate message ID
        let message_id = format!("msg-{}", uuid::Uuid::new_v4());

        // Get current time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Convert PartInput to Part objects with generated IDs
        let parts: Vec<Part> = req
            .parts
            .iter()
            .map(|input| {
                let part_id = format!("prt-{}", uuid::Uuid::new_v4());
                match input.part_type.as_str() {
                    "text" => Part::Text {
                        id: part_id,
                        session_id: session_id.clone(),
                        message_id: message_id.clone(),
                        text: input.text.clone().unwrap_or_default(),
                    },
                    "thinking" => Part::Thinking {
                        id: part_id,
                        session_id: session_id.clone(),
                        message_id: message_id.clone(),
                        text: input.text.clone().unwrap_or_default(),
                    },
                    _ => Part::Text {
                        id: part_id,
                        session_id: session_id.clone(),
                        message_id: message_id.clone(),
                        text: input.text.clone().unwrap_or_default(),
                    },
                }
            })
            .collect();

        // Create user message
        let user_message = MessageWithParts {
            info: Message::User {
                id: message_id.clone(),
                session_id: session_id.clone(),
                time: MessageTime {
                    created: now,
                    completed: None,
                },
                summary: None,
                metadata: None,
            },
            parts: parts.clone(),
        };

        // Store user message
        if let Err(e) = state.session_store.add_message(&session_id, user_message) {
            let _ = tx.send(Ok(Event::default()
                .event("error")
                .data(format!("{{\"error\":\"{}\"}}", e))));
            return;
        }

        // Execute agent
        // Load config to get default model
        let config_loader = ConfigLoader::new();
        let config = config_loader.load().unwrap_or_default();
        let (_provider_id, model_id) = config.get_default_model();

        let mut provider_config = ProviderConfig::moonshot();
        provider_config.default_model = model_id;

        let provider = match crate::providers::ProviderClient::new(provider_config) {
            Ok(p) => p,
            Err(e) => {
                let _ = tx.send(Ok(Event::default()
                    .event("error")
                    .data(format!("{{\"error\":\"{}\"}}", e))));
                return;
            }
        };

        let mut executor = AgentExecutor::new(
            provider,
            state.tool_registry.clone(),
            state.session_store.clone(),
            state.agent_registry.clone(),
            state.lock_manager.clone(),
        );

        // Link executor's cancellation token to the session lock
        executor.set_cancellation_token(lock.token());

        // Get working directory
        let working_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // Set up snapshot manager for tracking file changes
        let data_dir = crate::global::Global::data_dir();
        let snapshot_manager =
            crate::snapshot::SnapshotManager::new(&data_dir, &session_id, working_dir.clone());
        executor.set_snapshot_manager(snapshot_manager);

        // Clone session_id for use after executor consumes it
        let session_id_for_release = session_id.clone();

        // Create channel for streaming events
        let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();

        // Spawn executor in background
        let executor_handle = tokio::spawn(async move {
            executor
                .execute_turn_streaming(&session_id, "build", &working_dir, parts, event_tx)
                .await
        });

        // Forward events to SSE stream
        while let Some(event) = event_rx.recv().await {
            match event {
                crate::agent::ExecutionEvent::TextDelta { id, delta } => {
                    let delta_json = serde_json::json!({ "id": id, "delta": delta });
                    let _ = tx.send(Ok(Event::default()
                        .event("text.delta")
                        .data(delta_json.to_string())));
                }
                crate::agent::ExecutionEvent::Part(part) => {
                    let part_json = serde_json::to_string(&part).unwrap_or_default();
                    let _ = tx.send(Ok(Event::default().event("part").data(part_json)));
                }
                crate::agent::ExecutionEvent::Complete(msg) => {
                    let message_json = serde_json::to_string(&msg).unwrap_or_default();
                    let _ = tx.send(Ok(Event::default()
                        .event("message.complete")
                        .data(message_json)));
                }
                crate::agent::ExecutionEvent::Error(e) => {
                    let _ = tx.send(Ok(Event::default()
                        .event("error")
                        .data(format!("{{\"error\":\"{}\"}}", e))));
                }
            }
        }

        // Wait for executor to finish
        let _ = executor_handle.await;

        // Release the session lock
        state.lock_manager.release(&session_id_for_release);
    });

    // Convert receiver to stream
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

/// POST /session/{id}/revert - Revert session to a specific message
#[derive(serde::Deserialize)]
struct RevertRequest {
    #[serde(rename = "messageID")]
    message_id: String,
    #[serde(rename = "partID")]
    part_id: Option<String>,
}

async fn revert_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<RevertRequest>,
) -> Result<Json<Session>, (StatusCode, String)> {
    // Get the session
    let session = state
        .session_store
        .get(&session_id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    // Get all messages
    let messages = state
        .session_store
        .get_messages(&session_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Find all patches from messages after the target message
    let mut patches = vec![];
    let mut found_target = false;

    for msg in &messages {
        let msg_id = msg.info.id();
        if msg_id == req.message_id {
            found_target = true;
            continue;
        }
        if found_target {
            // Collect patches from this message
            for part in &msg.parts {
                if let Part::Patch { hash, files, .. } = part {
                    patches.push(crate::snapshot::Patch {
                        hash: hash.clone(),
                        files: files.iter().map(std::path::PathBuf::from).collect(),
                    });
                }
            }
        }
    }

    if !found_target {
        return Err((StatusCode::NOT_FOUND, "Message not found".to_string()));
    }

    // Create snapshot manager and revert
    let working_dir = std::path::PathBuf::from(&session.directory);
    let data_dir = crate::global::Global::data_dir();
    let snapshot_manager =
        crate::snapshot::SnapshotManager::new(&data_dir, &session_id, working_dir.clone());

    // Track current state before reverting (so we can unrevert)
    let current_snapshot = snapshot_manager.track().await.ok().flatten();

    // Revert files
    if !patches.is_empty() {
        snapshot_manager
            .revert(&patches)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    // Get diff for display
    let diff = if let Some(ref hash) = current_snapshot {
        snapshot_manager.diff(hash).await.ok()
    } else {
        None
    };

    // Update session with revert state
    let updated_session = state
        .session_store
        .update_with(&session_id, |s| {
            s.revert = Some(crate::types::SessionRevert {
                message_id: req.message_id.clone(),
                part_id: req.part_id.clone(),
                snapshot: current_snapshot.clone(),
                diff: diff.clone(),
            });
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // TODO: Delete messages after the target from storage

    Ok(Json(updated_session))
}

/// POST /session/{id}/unrevert - Undo a revert operation
async fn unrevert_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<Session>, (StatusCode, String)> {
    // Get the session
    let session = state
        .session_store
        .get(&session_id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    // Check if there's a revert to undo
    let revert = session
        .revert
        .as_ref()
        .ok_or((StatusCode::BAD_REQUEST, "No revert to undo".to_string()))?;

    // Restore to the snapshot before revert
    if let Some(ref snapshot) = revert.snapshot {
        let working_dir = std::path::PathBuf::from(&session.directory);
        let data_dir = crate::global::Global::data_dir();
        let snapshot_manager =
            crate::snapshot::SnapshotManager::new(&data_dir, &session_id, working_dir);

        snapshot_manager
            .restore(snapshot)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    // Clear revert state
    let updated_session = state
        .session_store
        .update_with(&session_id, |s| {
            s.revert = None;
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(updated_session))
}

/// POST /session/dual - Create and run a dual-agent session
#[derive(serde::Deserialize)]
struct CreateDualSessionRequest {
    task: String,
    directory: Option<String>,
}

#[derive(serde::Serialize)]
struct CreateDualSessionResponse {
    completed: bool,
    steps: usize,
    verdict: Option<String>,
    conversation_id: String,
    executor_session_id: String,
    discriminator_session_id: String,
}

async fn create_dual_session(
    State(state): State<AppState>,
    Json(req): Json<CreateDualSessionRequest>,
) -> Result<Json<CreateDualSessionResponse>, (StatusCode, String)> {
    // Use directory from request, or default to server's cwd
    let working_dir = if let Some(dir) = req.directory {
        std::path::PathBuf::from(dir)
    } else {
        std::env::current_dir().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    // Store both sessions first to get real session IDs
    let executor_session = state
        .session_store
        .create(
            working_dir.to_string_lossy().to_string(),
            None,
            Some(format!("Executor: {}", req.task)),
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let discriminator_session = state
        .session_store
        .create(
            working_dir.to_string_lossy().to_string(),
            None,
            Some(format!("Discriminator: {}", req.task)),
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Create shared conversation with the actual session IDs
    let mut shared_conversation = crate::agent::SharedConversation::new(
        req.task.clone(),
        executor_session.id.clone(),
        discriminator_session.id.clone(),
    );

    // Create runtime (it will create its own providers internally)
    let runtime = crate::agent::DualAgentRuntime::new(
        state.tool_registry.clone(),
        state.session_store.clone(),
        state.agent_registry.clone(),
        state.lock_manager.clone(),
    );

    // Run the dual-agent loop
    let result = runtime
        .run(&mut shared_conversation, &working_dir)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(CreateDualSessionResponse {
        completed: result.completed,
        steps: result.steps,
        verdict: result.verdict,
        conversation_id: result.conversation_id,
        executor_session_id: result.executor_session_id,
        discriminator_session_id: result.discriminator_session_id,
    }))
}

// ============================================================================
// Test Endpoints (for development)
// ============================================================================

/// POST /test/tool/{name} - Test a tool directly
async fn test_tool(
    State(state): State<AppState>,
    Path(tool_name): Path<String>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<crate::tools::ToolResult>, (StatusCode, String)> {
    // Create test context
    let ctx = crate::tools::ToolContext::new(
        "test-session".to_string(),
        "test-message".to_string(),
        "test".to_string(),
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
    );

    state
        .tool_registry
        .execute(&tool_name, input, &ctx)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

/// POST /session/{id}/fork - Fork a session at a specific message
#[derive(serde::Deserialize)]
struct ForkSessionRequest {
    #[serde(rename = "messageID")]
    message_id: Option<String>,
}

async fn fork_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<ForkSessionRequest>,
) -> Result<Json<Session>, (StatusCode, String)> {
    // Get the original session
    let original_session = state.session_store.get(&session_id).map_err(|e| {
        if e.contains("not found") {
            (StatusCode::NOT_FOUND, e)
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e)
        }
    })?;

    // Create a new session
    let new_session = state
        .session_store
        .create(
            original_session.directory.clone(),
            Some(session_id.clone()),
            Some(format!("Fork of {}", original_session.title)),
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Get messages from original session
    let messages = state
        .session_store
        .get_messages(&session_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Copy messages up to the specified message_id
    for msg in messages {
        if let Some(ref fork_msg_id) = req.message_id {
            if msg.info.id() >= fork_msg_id.as_str() {
                break;
            }
        }
        // Clone the message to the new session
        state
            .session_store
            .add_message(&new_session.id, msg)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    Ok(Json(new_session))
}

/// POST /session/{id}/abort - Abort a running session
async fn abort_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .lock_manager
        .abort(&session_id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    Ok(Json(serde_json::json!({
        "aborted": true,
        "session_id": session_id
    })))
}

/// GET /session/{id}/children - Get child sessions
async fn get_session_children(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<Vec<Session>>, (StatusCode, String)> {
    // List all sessions and filter by parent_id
    let all_sessions = state
        .session_store
        .list(None)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let children: Vec<Session> = all_sessions
        .into_iter()
        .filter(|s| s.parent_id.as_ref() == Some(&session_id))
        .collect();

    Ok(Json(children))
}

/// GET /session/{id}/todo - Get todo list for session
async fn get_session_todo(
    State(_state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // TODO: Integrate with TodoReadTool
    Ok(Json(serde_json::json!({
        "sessionID": session_id,
        "todos": []
    })))
}

/// GET /session/{id}/message/{message_id} - Get a specific message
async fn get_message(
    State(state): State<AppState>,
    Path((session_id, message_id)): Path<(String, String)>,
) -> Result<Json<MessageWithParts>, (StatusCode, String)> {
    let messages = state.session_store.get_messages(&session_id).map_err(|e| {
        if e.contains("not found") {
            (StatusCode::NOT_FOUND, e)
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e)
        }
    })?;

    messages
        .into_iter()
        .find(|m| m.info.id() == message_id)
        .map(Json)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Message {} not found", message_id),
            )
        })
}

/// GET /file - List files and directories
#[derive(serde::Deserialize)]
struct ListFilesQuery {
    path: Option<String>,
    pattern: Option<String>,
}

async fn list_files(
    axum::extract::Query(query): axum::extract::Query<ListFilesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let path = query.path.unwrap_or_else(|| ".".to_string());
    let dir_path = std::path::Path::new(&path);

    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(dir_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let metadata = entry
            .metadata()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let name = entry.file_name().to_string_lossy().to_string();

        // Apply pattern filter if specified
        if let Some(ref pattern) = query.pattern {
            if !name.contains(pattern) {
                continue;
            }
        }

        entries.push(serde_json::json!({
            "name": name,
            "path": entry.path().to_string_lossy(),
            "is_dir": metadata.is_dir(),
            "size": if metadata.is_file() { Some(metadata.len()) } else { None }
        }));
    }

    Ok(Json(serde_json::json!({
        "path": path,
        "entries": entries,
        "count": entries.len()
    })))
}

/// GET /file/content - Read file content
#[derive(serde::Deserialize)]
struct ReadFileQuery {
    path: String,
}

async fn read_file(
    axum::extract::Query(query): axum::extract::Query<ReadFileQuery>,
) -> Result<String, (StatusCode, String)> {
    std::fs::read_to_string(&query.path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// GET /config - Get configuration
async fn get_config() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    Ok(Json(serde_json::json!({
        "version": "0.1.0",
        "providers": ["moonshotai"],
        "agents": ["default"]
    })))
}

/// GET /config/providers - List all providers
async fn list_providers() -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    Ok(Json(vec![serde_json::json!({
        "id": "moonshotai",
        "name": "Moonshot AI",
        "models": ["kimi-k2-thinking", "kimi-k2-thinking-turbo", "moonshot-v1-128k", "moonshot-v1-32k"]
    })]))
}

/// GET /experimental/tool/ids - List all tool IDs
async fn list_tool_ids(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    Ok(Json(state.tool_registry.list_ids()))
}

/// GET /experimental/tool - List tools with schemas
#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct ListToolsQuery {
    provider: Option<String>,
    model: Option<String>,
}

async fn list_tools(
    State(state): State<AppState>,
    axum::extract::Query(_query): axum::extract::Query<ListToolsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    let tools = state.tool_registry.list_tools();
    Ok(Json(tools))
}

/// GET /event - Global SSE event stream
async fn event_stream() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = crate::bus::subscribe();

    let stream = async_stream::stream! {
        // Send connected event
        yield Ok(Event::default()
            .event("message")
            .data(serde_json::json!({
                "type": "server.connected",
                "properties": {}
            }).to_string()));

        // Stream all events
        loop {
            match rx.recv().await {
                Ok(event) => {
                    yield Ok(Event::default()
                        .event("message")
                        .data(serde_json::to_string(&event).unwrap_or_default()));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    // Client fell behind, skip missed events
                    tracing::warn!("Event stream lagged by {} events", n);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

/// GET /agent - List all agents
async fn list_agents(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    let agents = state.agent_registry.get_all().await;

    let agent_list: Vec<serde_json::Value> = agents
        .into_iter()
        .map(|agent| {
            serde_json::json!({
                "id": agent.name,
                "name": agent.name,
                "description": agent.description.unwrap_or_else(|| format!("{} agent", agent.name)),
                "mode": match agent.mode {
                    crate::agent::AgentMode::Primary => "primary",
                    crate::agent::AgentMode::Subagent => "subagent",
                    crate::agent::AgentMode::All => "all",
                },
                "builtIn": agent.built_in,
            })
        })
        .collect();

    Ok(Json(agent_list))
}

/// Convert loaded Config to ProviderConfig
fn get_provider_config_from_config(config: &crate::config::Config) -> ProviderConfig {
    // Get default model from config
    let (provider_id, model_id) = config.get_default_model();

    // Check if we have provider-specific config
    if let Some(providers) = &config.provider {
        if let Some(provider_cfg) = providers.get(&provider_id) {
            if let Some(options) = &provider_cfg.options {
                // Build custom provider config from loaded settings
                let base_url =
                    options
                        .base_url
                        .clone()
                        .unwrap_or_else(|| match provider_id.as_str() {
                            "moonshotai" => "https://api.moonshot.ai/v1".to_string(),
                            "openai" => "https://api.openai.com/v1".to_string(),
                            "anthropic" => "https://api.anthropic.com/v1".to_string(),
                            _ => "http://localhost:8080/v1".to_string(),
                        });

                let api_key_env = match provider_id.as_str() {
                    "moonshotai" => "MOONSHOT_API_KEY",
                    "openai" => "OPENAI_API_KEY",
                    "anthropic" => "ANTHROPIC_API_KEY",
                    _ => "API_KEY",
                };

                return ProviderConfig::custom(
                    provider_id.clone(),
                    base_url,
                    api_key_env.to_string(),
                    model_id,
                );
            }
        }
    }

    // Fall back to built-in provider configs
    match provider_id.as_str() {
        "openai" => {
            let mut cfg = ProviderConfig::openai();
            cfg.default_model = model_id;
            cfg
        }
        "moonshotai" | _ => {
            let mut cfg = ProviderConfig::moonshot();
            cfg.default_model = model_id;
            cfg
        }
    }
}
