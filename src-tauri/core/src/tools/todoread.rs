//! TodoRead tool - reads existing todo lists
//! Retrieves the current task list state

use super::todowrite::{TodoItem, TodoWriteTool};
use super::ToolContext;
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
#[allow(dead_code)]
struct TodoReadInput {
    #[serde(default = "default_session_id")]
    session_id: String,
}

#[allow(dead_code)]
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

    async fn execute(&self, _input: Value, ctx: &ToolContext) -> ToolResult {
        // Use session_id from context, NOT from LLM input (matching todowrite behavior)
        let session_id = &ctx.session_id;

        let todos = self.todo_write_tool.get_todos(session_id);

        let output = TodoReadOutput {
            count: todos.len(),
            todos,
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
