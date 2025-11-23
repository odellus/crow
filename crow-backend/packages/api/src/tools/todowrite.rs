//! TodoWrite tool - manages todo lists during coding sessions
//! Critical for agent planning and progress tracking

use super::ToolContext;
use super::{Tool, ToolResult, ToolStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "server")]
use parking_lot::RwLock;

#[cfg(not(feature = "server"))]
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub content: String,
    pub status: TodoStatus,
    #[serde(rename = "activeForm")]
    pub active_form: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TodoStatus {
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    Completed,
}

#[derive(Clone)]
pub struct TodoWriteTool {
    // Store todos per session
    todos: Arc<RwLock<HashMap<String, Vec<TodoItem>>>>,
}

impl TodoWriteTool {
    pub fn new() -> Self {
        Self {
            todos: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[cfg(feature = "server")]
    pub fn get_todos(&self, session_id: &str) -> Vec<TodoItem> {
        self.todos
            .read()
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }

    #[cfg(not(feature = "server"))]
    pub fn get_todos(&self, session_id: &str) -> Vec<TodoItem> {
        self.todos
            .read()
            .ok()
            .and_then(|todos| todos.get(session_id).cloned())
            .unwrap_or_default()
    }
}

#[derive(Deserialize)]
struct TodoWriteInput {
    todos: Vec<TodoItem>,
}

#[derive(Serialize, Deserialize)]
struct TodoWriteOutput {
    count: usize,
    todos: Vec<TodoItem>,
}

#[async_trait]
impl Tool for TodoWriteTool {
    fn name(&self) -> &str {
        "todowrite"
    }

    fn description(&self) -> &str {
        "Manage todo lists during coding sessions. Creates and updates task lists for tracking progress during complex operations. CRITICAL: Use this frequently to plan and track your work!"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Session ID for this todo list (optional, defaults to 'default')"
                },
                "todos": {
                    "type": "array",
                    "description": "Array of todo items with content, status, and activeForm",
                    "items": {
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string",
                                "description": "Description of the task"
                            },
                            "status": {
                                "type": "string",
                                "enum": ["pending", "in_progress", "completed"],
                                "description": "Current status of the task"
                            },
                            "activeForm": {
                                "type": "string",
                                "description": "Present continuous form (e.g., 'Building feature')"
                            }
                        },
                        "required": ["content", "status", "activeForm"]
                    }
                }
            },
            "required": ["todos"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let todo_input: TodoWriteInput = match serde_json::from_value(input) {
            Ok(i) => i,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Invalid input: {}", e)),
                    metadata: json!({}),
                };
            }
        };

        // Use session_id from context, NOT from LLM input
        let session_id = &ctx.session_id;

        // Store todos in memory
        #[cfg(feature = "server")]
        {
            let mut todos = self.todos.write();
            todos.insert(session_id.clone(), todo_input.todos.clone());
        }
        #[cfg(not(feature = "server"))]
        {
            if let Ok(mut todos) = self.todos.write() {
                todos.insert(session_id.clone(), todo_input.todos.clone());
            }
        }

        // Persist to disk (like OpenCode does)
        // Write to ~/.local/share/crow/storage/todo/{sessionID}.json
        if let Ok(global_paths) = std::env::var("HOME").map(|home| {
            let data_home =
                std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{}/.local/share", home));
            std::path::PathBuf::from(data_home).join("crow/storage/todo")
        }) {
            if let Err(e) = std::fs::create_dir_all(&global_paths) {
                tracing::warn!("Failed to create todo storage directory: {}", e);
            } else {
                let todo_file = global_paths.join(format!("{}.json", session_id));
                if let Ok(json) = serde_json::to_string_pretty(&todo_input.todos) {
                    if let Err(e) = std::fs::write(&todo_file, json) {
                        tracing::warn!("Failed to write todo file: {}", e);
                    } else {
                        tracing::debug!("Saved todos to {}", todo_file.display());
                    }
                }
            }
        }

        let output = TodoWriteOutput {
            count: todo_input.todos.len(),
            todos: todo_input.todos,
        };

        ToolResult {
            status: ToolStatus::Completed,
            output: serde_json::to_string(&output).unwrap_or_default(),
            error: None,
            metadata: json!({
                "session_id": session_id,
                "count": output.count,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_todowrite() {
        let tool = TodoWriteTool::new();
        let input = json!({
            "session_id": "test-session",
            "todos": [
                {
                    "content": "Implement feature X",
                    "status": "in_progress",
                    "activeForm": "Implementing feature X"
                },
                {
                    "content": "Write tests",
                    "status": "pending",
                    "activeForm": "Writing tests"
                }
            ]
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);

        let output: TodoWriteOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.count, 2);
    }
}
