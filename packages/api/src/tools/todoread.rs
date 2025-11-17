//! TodoRead tool - reads existing todo lists
//! Retrieves the current task list state

use super::todowrite::{TodoItem, TodoWriteTool};
use super::{Tool, ToolResult, ToolStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

pub struct TodoReadTool {
    todo_write_tool: Arc<TodoWriteTool>,
}

impl TodoReadTool {
    pub fn new(todo_write_tool: Arc<TodoWriteTool>) -> Self {
        Self { todo_write_tool }
    }
}

#[derive(Deserialize)]
struct TodoReadInput {
    #[serde(default = "default_session_id")]
    session_id: String,
}

fn default_session_id() -> String {
    "default".to_string()
}

#[derive(Serialize, Deserialize)]
struct TodoReadOutput {
    count: usize,
    todos: Vec<TodoItem>,
}

#[async_trait]
impl Tool for TodoReadTool {
    fn name(&self) -> &str {
        "todoread"
    }

    fn description(&self) -> &str {
        "Read existing todo lists. Retrieves the current task list state to track pending or completed items."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Session ID for the todo list (optional, defaults to 'default')"
                }
            }
        })
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let todo_input: TodoReadInput = match serde_json::from_value(input) {
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

        let todos = self.todo_write_tool.get_todos(&todo_input.session_id);

        let output = TodoReadOutput {
            count: todos.len(),
            todos,
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
