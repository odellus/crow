//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;

pub mod types;
pub use types::*;

pub mod providers;
pub use providers::*;

pub mod session;
pub use session::*;

#[cfg(feature = "server")]
pub mod tools;

#[cfg(feature = "server")]
pub mod agent;

#[cfg(feature = "server")]
pub mod storage;

#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "server")]
pub use server::*;

#[cfg(feature = "server")]
use std::sync::OnceLock;

#[cfg(feature = "server")]
static SESSION_STORE: OnceLock<SessionStore> = OnceLock::new();

#[cfg(feature = "server")]
fn get_session_store() -> &'static SessionStore {
    SESSION_STORE.get_or_init(|| SessionStore::new())
}

/// Echo the user input on the server.
#[post("/api/echo")]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

// ============================================================================
// OpenCode-Compatible API Endpoints
// ============================================================================

/// List all sessions (GET /session)
#[post("/session")]
pub async fn list_sessions() -> Result<Vec<Session>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let store = get_session_store();
        store
            .list(None)
            .map_err(|e| ServerFnError::new(format!("Failed to list sessions: {}", e)))
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("Server feature not enabled"))
    }
}

/// Create a new session (POST /session)
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CreateSessionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "parentID", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

#[post("/session/create")]
pub async fn create_session(req: CreateSessionRequest) -> Result<Session, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let store = get_session_store();

        // Get current directory
        let directory = std::env::current_dir()
            .map_err(|e| ServerFnError::new(format!("Failed to get current directory: {}", e)))?
            .to_string_lossy()
            .to_string();

        store
            .create(directory, req.parent_id, req.title)
            .map_err(|e| ServerFnError::new(format!("Failed to create session: {}", e)))
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("Server feature not enabled"))
    }
}

/// Get a session by ID (GET /session/:id)
#[post("/session/get")]
pub async fn get_session(session_id: String) -> Result<Session, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let store = get_session_store();
        store
            .get(&session_id)
            .map_err(|e| ServerFnError::new(format!("Failed to get session: {}", e)))
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("Server feature not enabled"))
    }
}

/// Send a message and get a response from the agent with tool execution
#[post("/api/send_message")]
pub async fn send_message(content: String) -> Result<Vec<Part>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        use agent::AgentExecutor;
        use std::sync::Arc;
        use tools::ToolRegistry;

        // Get or create a session
        let store = get_session_store();

        // Try to get the first session, or create one
        let session = match store
            .list(None)
            .ok()
            .and_then(|sessions| sessions.first().cloned())
        {
            Some(s) => s,
            None => {
                // Create a new session
                let directory = std::env::current_dir()
                    .map_err(|e| {
                        ServerFnError::new(format!("Failed to get current directory: {}", e))
                    })?
                    .to_string_lossy()
                    .to_string();
                store
                    .create(directory, None, Some("Chat Session".to_string()))
                    .map_err(|e| ServerFnError::new(format!("Failed to create session: {}", e)))?
            }
        };

        let session_id = session.id;

        // Create user message parts
        let user_parts = vec![Part::Text {
            id: format!("part-user-{}", uuid::Uuid::new_v4()),
            session_id: session_id.clone(),
            message_id: format!("msg-user-{}", uuid::Uuid::new_v4()),
            text: content.clone(),
        }];

        // Store user message
        let user_message = MessageWithParts {
            info: Message::User {
                id: format!("msg-user-{}", uuid::Uuid::new_v4()),
                session_id: session_id.clone(),
                time: MessageTime {
                    created: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                    completed: None,
                },
                summary: None,
                metadata: None,
            },
            parts: user_parts.clone(),
        };

        store
            .add_message(&session_id, user_message)
            .map_err(|e| ServerFnError::new(format!("Failed to add user message: {}", e)))?;

        // Create agent executor
        let provider_config = ProviderConfig::moonshot();
        let provider = ProviderClient::new(provider_config).map_err(|e| ServerFnError::new(e))?;
        let tool_registry = Arc::new(ToolRegistry::new());
        let agent_registry = Arc::new(agent::AgentRegistry::new());
        let session_store = Arc::new(store.clone());

        let executor = AgentExecutor::new(provider, tool_registry, session_store, agent_registry);

        // Get working directory
        let working_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // Execute agent turn with build agent
        let response = executor
            .execute_turn(&session_id, "build", &working_dir, user_parts)
            .await
            .map_err(|e| ServerFnError::new(format!("Agent execution failed: {}", e)))?;

        Ok(response.parts)
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("Server feature not enabled"))
    }
}
