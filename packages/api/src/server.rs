//! HTTP server that mirrors OpenCode's REST API
//! This provides the same endpoints as `opencode serve`

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, Sse},
    routing::{delete, get, patch, post},
    Json, Router,
};
use futures::stream::{self, Stream};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;

use crate::{
    agent::{AgentExecutor, AgentRegistry},
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
        .route("/session/:id", get(get_session))
        .route("/session/:id", delete(delete_session))
        .route("/session/:id", patch(update_session))
        // Message endpoints
        .route("/session/:id/message", get(list_messages))
        .route("/session/:id/message", post(send_message))
        // Test endpoint for tools
        .route("/test/tool/:name", post(test_tool))
        // TODO: Other endpoints
        // .route("/config", get(get_config))
        // .route("/config/providers", get(list_providers))
        // .route("/agent", get(list_agents))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Initialize storage and build router
pub async fn create_router_with_storage() -> Result<Router, String> {
    let session_store = Arc::new(SessionStore::new());

    // Initialize storage and load existing data
    session_store.init().await?;

    tracing::info!(
        "Storage initialized, loaded {} sessions",
        session_store.list(None).unwrap_or_default().len()
    );

    let agent_registry = Arc::new(AgentRegistry::new());
    let lock_manager = Arc::new(SessionLockManager::new());
    let provider_config = ProviderConfig::moonshot();

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
        .route("/session/:id", get(get_session))
        .route("/session/:id", delete(delete_session))
        .route("/session/:id", patch(update_session))
        .route("/session/:id/fork", post(fork_session))
        .route("/session/:id/abort", post(abort_session))
        .route("/session/:id/children", get(get_session_children))
        .route("/session/:id/todo", get(get_session_todo))
        // Message endpoints
        .route("/session/:id/message", get(list_messages))
        .route("/session/:id/message", post(send_message))
        .route("/session/:id/message/:message_id", get(get_message))
        // Streaming message endpoint
        .route("/session/:id/message/stream", post(send_message_stream))
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
        .route("/test/tool/:name", post(test_tool))
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

/// GET /session/:id - Get a session by ID
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

/// DELETE /session/:id - Delete a session
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

/// PATCH /session/:id - Update a session
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

/// GET /session/:id/message - List all messages in a session
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

/// POST /session/:id/message - Send a message to a session
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
    "default".to_string()
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
    let provider_config = ProviderConfig::moonshot();
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

/// POST /session/:id/message/stream - Send a message with streaming response
async fn send_message_stream(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Create a channel for streaming events
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // Spawn task to execute agent and stream events
    tokio::spawn(async move {
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
        let provider_config = ProviderConfig::moonshot();
        let provider = match crate::providers::ProviderClient::new(provider_config) {
            Ok(p) => p,
            Err(e) => {
                let _ = tx.send(Ok(Event::default()
                    .event("error")
                    .data(format!("{{\"error\":\"{}\"}}", e))));
                return;
            }
        };

        let executor = AgentExecutor::new(
            provider,
            state.tool_registry.clone(),
            state.session_store.clone(),
            state.agent_registry.clone(),
            state.lock_manager.clone(),
        );

        // Get working directory
        let working_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // Stream tool execution - for now just execute and send result
        match executor
            .execute_turn(&session_id, "build", &working_dir, parts)
            .await
        {
            Ok(assistant_message) => {
                // Stream each part
                for part in &assistant_message.parts {
                    let part_json = serde_json::to_string(&part).unwrap_or_default();
                    let _ = tx.send(Ok(Event::default().event("part").data(part_json)));
                }

                // Send completion event
                let message_json = serde_json::to_string(&assistant_message).unwrap_or_default();
                let _ = tx.send(Ok(Event::default()
                    .event("message.complete")
                    .data(message_json)));
            }
            Err(e) => {
                let _ = tx.send(Ok(Event::default()
                    .event("error")
                    .data(format!("{{\"error\":\"{}\"}}", e))));
            }
        }
    });

    // Convert receiver to stream
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive"),
    )
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

/// POST /test/tool/:name - Test a tool directly
async fn test_tool(
    State(state): State<AppState>,
    Path(tool_name): Path<String>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<crate::tools::ToolResult>, (StatusCode, String)> {
    // Create test context
    let ctx = crate::tools::ToolContext {
        session_id: "test-session".to_string(),
        message_id: "test-message".to_string(),
        agent: "test".to_string(),
        working_dir: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
    };

    state
        .tool_registry
        .execute(&tool_name, input, &ctx)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

/// POST /session/:id/fork - Fork a session at a specific message
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

/// POST /session/:id/abort - Abort a running session
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

/// GET /session/:id/children - Get child sessions
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

/// GET /session/:id/todo - Get todo list for session
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

/// GET /session/:id/message/:message_id - Get a specific message
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
