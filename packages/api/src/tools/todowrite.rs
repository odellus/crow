//! TodoWrite tool - manages todo lists during coding sessions
//! Critical for agent planning and progress tracking

use super::ToolContext;
use super::{Tool, ToolResult, ToolStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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

    pub fn get_todos(&self, session_id: &str) -> Vec<TodoItem> {
        self.todos
            .read()
            .unwrap()
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Deserialize)]
struct TodoWriteInput {
    #[serde(default = "default_session_id")]
    session_id: String,
    todos: Vec<TodoItem>,
}

fn default_session_id() -> String {
    "default".to_string()
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

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> ToolResult {
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

        // Store todos
        {
            let mut todos = self.todos.write().unwrap();
            todos.insert(todo_input.session_id.clone(), todo_input.todos.clone());
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
                "session_id": todo_input.session_id,
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

        let ctx = crate::tools::ToolContext { session_id: "test".to_string(), message_id: "test".to_string(), agent: "test".to_string(), working_dir: std::path::PathBuf::from("/tmp") };
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);

        let output: TodoWriteOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.count, 2);
    }
}
