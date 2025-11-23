# Crow A2A Protocol Integration Plan

Expose crow as an A2A-compliant agent that can communicate with other A2A agents.

---

## A2A Protocol Overview

A2A (Agent-to-Agent) is a protocol for agents to communicate with each other. Key concepts:

- **Agent Card**: Self-describing manifest (capabilities, skills, security)
- **Task**: Core unit of work with state machine (submitted → working → completed/failed)
- **Message**: Communication unit with Parts (text, file, data)
- **Context**: Session/conversation grouping for tasks
- **Streaming**: SSE for real-time updates

---

## Mapping Crow to A2A

| Crow Concept | A2A Concept | Notes |
|--------------|-------------|-------|
| Session | Context | 1:1 mapping, use session ID as context ID |
| Message | Message | Similar structure, different part types |
| Agent turn | Task | Each agent response is a task |
| Streaming response | StreamResponse | SSE with status/artifact updates |
| Tool execution | Artifact | Tool outputs as artifacts |

---

## Required A2A Endpoints

### Core Methods (JSON-RPC over HTTP)

| Method | A2A | Crow Mapping |
|--------|-----|--------------|
| `message/send` | Send message, get task | `POST /session/:id/message` |
| `message/stream` | Send message, stream response | `POST /session/:id/message/stream` |
| `tasks/get` | Get task by ID | New endpoint needed |
| `tasks/cancel` | Cancel running task | `POST /session/:id/abort` |
| `tasks/list` | List tasks | New endpoint (list messages) |
| `tasks/subscribe` | Subscribe to task updates | SSE stream |

### Agent Card

| Endpoint | Purpose |
|----------|---------|
| `GET /.well-known/agent.json` | Agent Card discovery |
| `GET /agent/authenticatedExtendedCard` | Extended card with auth |

---

## Implementation Plan

### Phase 1: Agent Card (1-2 hours)

Create the agent card that describes crow's capabilities:

```rust
// src/a2a/agent_card.rs

pub fn create_agent_card() -> AgentCard {
    AgentCard {
        name: "Crow".to_string(),
        description: "Autonomous coding agent for software development tasks".to_string(),
        version: "0.1.0".to_string(),
        protocol_version: "1.0".to_string(),
        
        provider: AgentProvider {
            organization: "Crow".to_string(),
            url: "https://github.com/anthropics/crow".to_string(),
        },
        
        capabilities: AgentCapabilities {
            streaming: true,
            push_notifications: false,
            state_transition_history: true,
            extensions: vec![],
        },
        
        skills: vec![
            AgentSkill {
                id: "code".to_string(),
                name: "Code Generation & Editing".to_string(),
                description: "Write, edit, and refactor code across multiple languages".to_string(),
                tags: vec!["coding", "development", "refactoring"],
                examples: vec![
                    "Write a function to sort a list".to_string(),
                    "Refactor this code to use async/await".to_string(),
                    "Add error handling to this function".to_string(),
                ],
                input_modes: vec!["text/plain".to_string()],
                output_modes: vec!["text/plain".to_string(), "application/json".to_string()],
            },
            AgentSkill {
                id: "debug".to_string(),
                name: "Debugging".to_string(),
                description: "Find and fix bugs in code".to_string(),
                tags: vec!["debugging", "troubleshooting"],
                examples: vec![
                    "Why is this test failing?".to_string(),
                    "Find the bug in this function".to_string(),
                ],
                input_modes: vec!["text/plain".to_string()],
                output_modes: vec!["text/plain".to_string()],
            },
            AgentSkill {
                id: "explain".to_string(),
                name: "Code Explanation".to_string(),
                description: "Explain how code works".to_string(),
                tags: vec!["explanation", "documentation"],
                examples: vec![
                    "Explain this function".to_string(),
                    "What does this regex do?".to_string(),
                ],
                input_modes: vec!["text/plain".to_string()],
                output_modes: vec!["text/plain".to_string()],
            },
        ],
        
        default_input_modes: vec!["text/plain".to_string()],
        default_output_modes: vec!["text/plain".to_string(), "application/json".to_string()],
        
        supported_interfaces: vec![
            AgentInterface {
                url: "http://localhost:7070/a2a".to_string(),
                protocol_binding: "JSONRPC".to_string(),
            },
        ],
        
        security_schemes: HashMap::new(), // No auth for now
        security: vec![],
    }
}
```

**Endpoint:**
```rust
// GET /.well-known/agent.json
async fn get_agent_card() -> Json<AgentCard> {
    Json(create_agent_card())
}
```

---

### Phase 2: Type Definitions (2-3 hours)

Create Rust types matching A2A schema:

```rust
// src/a2a/types.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub message_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    pub role: Role,
    pub parts: Vec<Part>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    RoleUnspecified,
    RoleUser,
    RoleAgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FilePart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub media_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_with_bytes: Option<String>, // base64
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_with_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub context_id: String,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<Message>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<Artifact>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatus {
    pub state: TaskState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>, // ISO 8601
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskState {
    TaskStateUnspecified,
    TaskStateSubmitted,
    TaskStateWorking,
    TaskStateCompleted,
    TaskStateFailed,
    TaskStateCancelled,
    TaskStateInputRequired,
    TaskStateRejected,
    TaskStateAuthRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub artifact_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parts: Vec<Part>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

// Request/Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<SendMessageConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_output_modes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_length: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<Task>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_update: Option<TaskStatusUpdateEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_update: Option<TaskArtifactUpdateEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatusUpdateEvent {
    pub task_id: String,
    pub context_id: String,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#final: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskArtifactUpdateEvent {
    pub task_id: String,
    pub context_id: String,
    pub artifact: Artifact,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub append: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_chunk: Option<bool>,
}
```

---

### Phase 3: JSON-RPC Handler (3-4 hours)

A2A uses JSON-RPC 2.0. Create a handler:

```rust
// src/a2a/jsonrpc.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String, // "2.0"
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    pub id: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// Standard JSON-RPC error codes
pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;

// A2A specific error codes
pub const TASK_NOT_FOUND: i32 = -32001;
pub const TASK_CANCELLED: i32 = -32002;
```

---

### Phase 4: Core A2A Endpoints (4-6 hours)

#### message/send

```rust
async fn handle_message_send(
    state: &AppState,
    params: SendMessageRequest,
) -> Result<SendMessageResponse, JsonRpcError> {
    // 1. Get or create context (session)
    let context_id = match params.message.context_id {
        Some(id) => id,
        None => {
            // Create new session
            let session = state.session_store.create(
                std::env::current_dir()?.to_string_lossy().to_string(),
                None,
                None,
            )?;
            session.id
        }
    };
    
    // 2. Create task ID for this interaction
    let task_id = format!("task-{}", uuid::Uuid::new_v4());
    
    // 3. Convert A2A message to crow format
    let crow_parts = convert_a2a_parts_to_crow(&params.message.parts);
    
    // 4. Store user message
    let user_msg = create_crow_user_message(&context_id, &crow_parts);
    state.session_store.add_message(&context_id, user_msg)?;
    
    // 5. Execute agent
    let executor = create_executor(state);
    let working_dir = get_session_working_dir(state, &context_id)?;
    
    let result = executor
        .execute_turn(&context_id, "build", &working_dir, crow_parts)
        .await?;
    
    // 6. Convert crow response to A2A format
    let a2a_message = convert_crow_message_to_a2a(&result, &context_id, &task_id);
    let task = create_task(&task_id, &context_id, TaskState::TaskStateCompleted, &result);
    
    Ok(SendMessageResponse {
        message: Some(a2a_message),
        task: Some(task),
    })
}
```

#### message/stream

```rust
async fn handle_message_stream(
    state: AppState,
    params: SendMessageRequest,
    tx: mpsc::UnboundedSender<StreamResponse>,
) -> Result<(), JsonRpcError> {
    let context_id = params.message.context_id
        .unwrap_or_else(|| create_new_context(&state));
    let task_id = format!("task-{}", uuid::Uuid::new_v4());
    
    // Send initial status: SUBMITTED
    tx.send(StreamResponse {
        status_update: Some(TaskStatusUpdateEvent {
            task_id: task_id.clone(),
            context_id: context_id.clone(),
            status: TaskStatus {
                state: TaskState::TaskStateSubmitted,
                message: None,
                timestamp: Some(now_iso8601()),
            },
            r#final: Some(false),
        }),
        ..Default::default()
    })?;
    
    // Send status: WORKING
    tx.send(StreamResponse {
        status_update: Some(TaskStatusUpdateEvent {
            task_id: task_id.clone(),
            context_id: context_id.clone(),
            status: TaskStatus {
                state: TaskState::TaskStateWorking,
                message: None,
                timestamp: Some(now_iso8601()),
            },
            r#final: Some(false),
        }),
        ..Default::default()
    })?;
    
    // Execute with streaming
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    
    tokio::spawn(async move {
        let executor = create_executor(&state);
        executor.execute_turn_streaming(
            &context_id, "build", &working_dir, crow_parts, event_tx
        ).await
    });
    
    // Forward crow events as A2A events
    let mut accumulated_text = String::new();
    let mut artifacts = Vec::new();
    
    while let Some(event) = event_rx.recv().await {
        match event {
            ExecutionEvent::TextDelta { delta, .. } => {
                accumulated_text.push_str(&delta);
                
                // Send as message part update
                tx.send(StreamResponse {
                    message: Some(Message {
                        message_id: format!("msg-{}", uuid::Uuid::new_v4()),
                        context_id: Some(context_id.clone()),
                        task_id: Some(task_id.clone()),
                        role: Role::RoleAgent,
                        parts: vec![Part {
                            text: Some(accumulated_text.clone()),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                    ..Default::default()
                })?;
            }
            
            ExecutionEvent::Part(part) => {
                if let crate::types::Part::Tool { tool, state: tool_state, .. } = part {
                    // Tool execution → Artifact
                    if let crate::types::ToolState::Completed { output, .. } = tool_state {
                        let artifact = Artifact {
                            artifact_id: format!("art-{}", uuid::Uuid::new_v4()),
                            name: tool.clone(),
                            description: Some(format!("Output from {} tool", tool)),
                            parts: vec![Part {
                                text: Some(output),
                                ..Default::default()
                            }],
                            metadata: None,
                        };
                        
                        artifacts.push(artifact.clone());
                        
                        tx.send(StreamResponse {
                            artifact_update: Some(TaskArtifactUpdateEvent {
                                task_id: task_id.clone(),
                                context_id: context_id.clone(),
                                artifact,
                                append: Some(false),
                                last_chunk: Some(true),
                            }),
                            ..Default::default()
                        })?;
                    }
                }
            }
            
            ExecutionEvent::Complete(msg) => {
                // Send final status: COMPLETED
                tx.send(StreamResponse {
                    status_update: Some(TaskStatusUpdateEvent {
                        task_id: task_id.clone(),
                        context_id: context_id.clone(),
                        status: TaskStatus {
                            state: TaskState::TaskStateCompleted,
                            message: Some(convert_crow_message_to_a2a(&msg, &context_id, &task_id)),
                            timestamp: Some(now_iso8601()),
                        },
                        r#final: Some(true),
                    }),
                    task: Some(Task {
                        id: task_id.clone(),
                        context_id: context_id.clone(),
                        status: TaskStatus {
                            state: TaskState::TaskStateCompleted,
                            ..Default::default()
                        },
                        artifacts: Some(artifacts.clone()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })?;
            }
            
            ExecutionEvent::Error(e) => {
                tx.send(StreamResponse {
                    status_update: Some(TaskStatusUpdateEvent {
                        task_id: task_id.clone(),
                        context_id: context_id.clone(),
                        status: TaskStatus {
                            state: TaskState::TaskStateFailed,
                            message: Some(Message {
                                message_id: format!("msg-{}", uuid::Uuid::new_v4()),
                                role: Role::RoleAgent,
                                parts: vec![Part {
                                    text: Some(e),
                                    ..Default::default()
                                }],
                                ..Default::default()
                            }),
                            timestamp: Some(now_iso8601()),
                        },
                        r#final: Some(true),
                    }),
                    ..Default::default()
                })?;
            }
        }
    }
    
    Ok(())
}
```

#### tasks/get, tasks/cancel, tasks/list

```rust
async fn handle_tasks_get(
    state: &AppState,
    params: GetTaskRequest,
) -> Result<Task, JsonRpcError> {
    // Parse task name: "tasks/{task_id}"
    let task_id = parse_task_name(&params.name)?;
    
    // Look up task in our store (need to add task storage)
    let task = state.task_store.get(&task_id)?;
    
    Ok(task)
}

async fn handle_tasks_cancel(
    state: &AppState,
    params: CancelTaskRequest,
) -> Result<Task, JsonRpcError> {
    let task_id = parse_task_name(&params.name)?;
    let task = state.task_store.get(&task_id)?;
    
    // Abort the session
    state.lock_manager.abort(&task.context_id)?;
    
    // Update task status
    let mut task = task;
    task.status.state = TaskState::TaskStateCancelled;
    state.task_store.update(&task)?;
    
    Ok(task)
}

async fn handle_tasks_list(
    state: &AppState,
    params: ListTasksRequest,
) -> Result<ListTasksResponse, JsonRpcError> {
    let tasks = if let Some(context_id) = params.context_id {
        state.task_store.list_by_context(&context_id)?
    } else {
        state.task_store.list_all()?
    };
    
    Ok(ListTasksResponse {
        tasks,
        total_size: tasks.len() as i32,
        page_size: params.page_size.unwrap_or(50),
        next_page_token: String::new(),
    })
}
```

---

### Phase 5: Router Setup (1-2 hours)

```rust
// src/a2a/router.rs

pub fn create_a2a_router(state: AppState) -> Router {
    Router::new()
        // Agent Card discovery
        .route("/.well-known/agent.json", get(get_agent_card))
        
        // JSON-RPC endpoint
        .route("/a2a", post(handle_jsonrpc))
        
        // SSE streaming endpoint
        .route("/a2a/stream", post(handle_jsonrpc_stream))
        
        .with_state(state)
}

async fn handle_jsonrpc(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    let result = match request.method.as_str() {
        "message/send" => {
            let params: SendMessageRequest = serde_json::from_value(request.params.unwrap())?;
            handle_message_send(&state, params).await
        }
        "tasks/get" => {
            let params: GetTaskRequest = serde_json::from_value(request.params.unwrap())?;
            handle_tasks_get(&state, params).await
        }
        "tasks/cancel" => {
            let params: CancelTaskRequest = serde_json::from_value(request.params.unwrap())?;
            handle_tasks_cancel(&state, params).await
        }
        "tasks/list" => {
            let params: ListTasksRequest = serde_json::from_value(request.params.unwrap())?;
            handle_tasks_list(&state, params).await
        }
        _ => Err(JsonRpcError {
            code: METHOD_NOT_FOUND,
            message: format!("Method not found: {}", request.method),
            data: None,
        }),
    };
    
    Json(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: result.ok().map(|r| serde_json::to_value(r).unwrap()),
        error: result.err(),
        id: request.id,
    })
}

async fn handle_jsonrpc_stream(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::unbounded_channel();
    
    tokio::spawn(async move {
        if request.method == "message/stream" {
            let params: SendMessageRequest = serde_json::from_value(request.params.unwrap())?;
            handle_message_stream(state, params, tx).await
        }
    });
    
    let stream = UnboundedReceiverStream::new(rx).map(|response| {
        Ok(Event::default().data(serde_json::to_string(&response).unwrap()))
    });
    
    Sse::new(stream)
}
```

---

## Implementation Timeline

| Phase | Description | Effort |
|-------|-------------|--------|
| 1 | Agent Card | 1-2 hours |
| 2 | Type Definitions | 2-3 hours |
| 3 | JSON-RPC Handler | 3-4 hours |
| 4 | Core Endpoints | 4-6 hours |
| 5 | Router Setup | 1-2 hours |
| **Total** | | **11-17 hours** |

---

## Storage Requirements

Need to add task storage to track A2A tasks:

```rust
// src/a2a/store.rs

pub struct TaskStore {
    tasks: RwLock<HashMap<String, Task>>,
    // Also need persistence to ~/.local/share/crow/a2a/tasks/
}

impl TaskStore {
    pub fn create(&self, context_id: &str) -> Task { ... }
    pub fn get(&self, task_id: &str) -> Result<Task, String> { ... }
    pub fn update(&self, task: &Task) -> Result<(), String> { ... }
    pub fn list_by_context(&self, context_id: &str) -> Vec<Task> { ... }
}
```

---

## Testing

### Manual Testing

```bash
# Get agent card
curl http://localhost:7070/.well-known/agent.json | jq

# Send message (JSON-RPC)
curl -X POST http://localhost:7070/a2a \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "message/send",
    "params": {
      "message": {
        "messageId": "msg-123",
        "role": "ROLE_USER",
        "parts": [{"text": "What files are in this project?"}]
      }
    },
    "id": 1
  }'

# Stream message
curl -X POST http://localhost:7070/a2a/stream \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "message/stream",
    "params": {
      "message": {
        "messageId": "msg-456",
        "role": "ROLE_USER",
        "parts": [{"text": "Write a hello world in Rust"}]
      }
    },
    "id": 2
  }'
```

### Integration Test

```bash
# Test with another A2A agent
# (would need an A2A client implementation)
```

---

## Future Enhancements

1. **Push Notifications** - Webhook callbacks for async tasks
2. **Authentication** - API key, OAuth2 support
3. **Multi-skill routing** - Route to different crow agents based on skill
4. **File artifacts** - Return code files as FilePart
5. **Context persistence** - Long-running conversations across restarts

---

## File Structure

```
crow/packages/api/src/
├── a2a/
│   ├── mod.rs           # Module exports
│   ├── types.rs         # A2A type definitions
│   ├── agent_card.rs    # Agent card generation
│   ├── jsonrpc.rs       # JSON-RPC types and handlers
│   ├── router.rs        # Axum router setup
│   ├── handlers.rs      # Method implementations
│   ├── store.rs         # Task storage
│   └── convert.rs       # Crow ↔ A2A type conversion
├── server.rs            # Add A2A router
└── lib.rs               # Add mod a2a
```

---

## Summary

The A2A integration exposes crow as a compliant agent that can:

1. **Discover** - Serve agent card at `/.well-known/agent.json`
2. **Communicate** - Accept JSON-RPC requests at `/a2a`
3. **Stream** - Provide SSE updates at `/a2a/stream`
4. **Track** - Manage tasks with full lifecycle

This enables crow to be called by other A2A agents or any A2A-compatible client, enabling agent-to-agent workflows.
