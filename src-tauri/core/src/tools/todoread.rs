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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::todowrite::{TodoStatus, TodoWriteTool};
    use std::path::PathBuf;

    fn create_test_context(session_id: &str) -> ToolContext {
        ToolContext::new(
            session_id.to_string(),
            "test-message".to_string(),
            "build".to_string(),
            PathBuf::from("/tmp/test"),
        )
    }

    // ==================== Basic Functionality ====================

    #[tokio::test]
    async fn test_todoread_returns_empty_when_none() {
        let todo_write = Arc::new(TodoWriteTool::new());
        let tool = TodoReadTool::new(todo_write);
        let ctx = create_test_context("empty-session");

        let result = tool.execute(json!({}), &ctx).await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: TodoReadOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.count, 0);
        assert!(output.todos.is_empty());
    }

    #[tokio::test]
    async fn test_todoread_returns_current_todos() {
        let todo_write = Arc::new(TodoWriteTool::new());
        let tool = TodoReadTool::new(todo_write.clone());
        let ctx = create_test_context("read-test");

        // First write some todos
        todo_write
            .execute(
                json!({
                    "todos": [
                        {"content": "Task 1", "status": "pending", "activeForm": "Task 1"},
                        {"content": "Task 2", "status": "in_progress", "activeForm": "Task 2"}
                    ]
                }),
                &ctx,
            )
            .await;

        // Now read
        let result = tool.execute(json!({}), &ctx).await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: TodoReadOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.count, 2);
        assert_eq!(output.todos[0].content, "Task 1");
        assert_eq!(output.todos[1].content, "Task 2");
    }

    #[tokio::test]
    async fn test_todoread_reflects_updates() {
        let todo_write = Arc::new(TodoWriteTool::new());
        let tool = TodoReadTool::new(todo_write.clone());
        let ctx = create_test_context("update-test");

        // Initial write
        todo_write
            .execute(
                json!({
                    "todos": [{"content": "Initial", "status": "pending", "activeForm": "Initial"}]
                }),
                &ctx,
            )
            .await;

        // First read
        let result1 = tool.execute(json!({}), &ctx).await;
        let output1: TodoReadOutput = serde_json::from_str(&result1.output).unwrap();
        assert_eq!(output1.count, 1);

        // Update
        todo_write
            .execute(
                json!({
                    "todos": [
                        {"content": "Updated 1", "status": "completed", "activeForm": "Updated 1"},
                        {"content": "Updated 2", "status": "pending", "activeForm": "Updated 2"},
                        {"content": "Updated 3", "status": "pending", "activeForm": "Updated 3"}
                    ]
                }),
                &ctx,
            )
            .await;

        // Second read should reflect update
        let result2 = tool.execute(json!({}), &ctx).await;
        let output2: TodoReadOutput = serde_json::from_str(&result2.output).unwrap();
        assert_eq!(output2.count, 3);
        assert_eq!(output2.todos[0].content, "Updated 1");
    }

    // ==================== Session Isolation ====================

    #[tokio::test]
    async fn test_todoread_session_isolation() {
        let todo_write = Arc::new(TodoWriteTool::new());
        let tool = TodoReadTool::new(todo_write.clone());

        let ctx_a = create_test_context("read-session-A");
        let ctx_b = create_test_context("read-session-B");

        // Write different todos to each session
        todo_write
            .execute(
                json!({
                    "todos": [{"content": "Session A Task", "status": "pending", "activeForm": "A"}]
                }),
                &ctx_a,
            )
            .await;

        todo_write
            .execute(
                json!({
                    "todos": [
                        {"content": "Session B Task 1", "status": "pending", "activeForm": "B1"},
                        {"content": "Session B Task 2", "status": "pending", "activeForm": "B2"}
                    ]
                }),
                &ctx_b,
            )
            .await;

        // Read from session A
        let result_a = tool.execute(json!({}), &ctx_a).await;
        let output_a: TodoReadOutput = serde_json::from_str(&result_a.output).unwrap();

        // Read from session B
        let result_b = tool.execute(json!({}), &ctx_b).await;
        let output_b: TodoReadOutput = serde_json::from_str(&result_b.output).unwrap();

        // Verify isolation
        assert_eq!(output_a.count, 1);
        assert_eq!(output_a.todos[0].content, "Session A Task");

        assert_eq!(output_b.count, 2);
        assert_eq!(output_b.todos[0].content, "Session B Task 1");
    }

    // ==================== Status Preservation ====================

    #[tokio::test]
    async fn test_todoread_preserves_status() {
        let todo_write = Arc::new(TodoWriteTool::new());
        let tool = TodoReadTool::new(todo_write.clone());
        let ctx = create_test_context("status-test");

        todo_write
            .execute(
                json!({
                    "todos": [
                        {"content": "Pending", "status": "pending", "activeForm": "P"},
                        {"content": "In Progress", "status": "in_progress", "activeForm": "IP"},
                        {"content": "Completed", "status": "completed", "activeForm": "C"}
                    ]
                }),
                &ctx,
            )
            .await;

        let result = tool.execute(json!({}), &ctx).await;
        let output: TodoReadOutput = serde_json::from_str(&result.output).unwrap();

        assert!(matches!(output.todos[0].status, TodoStatus::Pending));
        assert!(matches!(output.todos[1].status, TodoStatus::InProgress));
        assert!(matches!(output.todos[2].status, TodoStatus::Completed));
    }

    // ==================== Metadata Tests ====================

    #[tokio::test]
    async fn test_todoread_returns_correct_metadata() {
        let todo_write = Arc::new(TodoWriteTool::new());
        let tool = TodoReadTool::new(todo_write.clone());
        let ctx = create_test_context("metadata-read-test");

        todo_write
            .execute(
                json!({
                    "todos": [
                        {"content": "Task 1", "status": "pending", "activeForm": "T1"},
                        {"content": "Task 2", "status": "pending", "activeForm": "T2"}
                    ]
                }),
                &ctx,
            )
            .await;

        let result = tool.execute(json!({}), &ctx).await;

        assert_eq!(
            result.metadata.get("session_id").and_then(|v| v.as_str()),
            Some("metadata-read-test")
        );
        assert_eq!(
            result.metadata.get("count").and_then(|v| v.as_u64()),
            Some(2)
        );
    }

    // ==================== Tool Interface Tests ====================

    #[tokio::test]
    async fn test_todoread_tool_name() {
        let todo_write = Arc::new(TodoWriteTool::new());
        let tool = TodoReadTool::new(todo_write);
        assert_eq!(tool.name(), "todoread");
    }

    #[tokio::test]
    async fn test_todoread_tool_description() {
        let todo_write = Arc::new(TodoWriteTool::new());
        let tool = TodoReadTool::new(todo_write);
        let desc = tool.description();
        assert!(desc.contains("todo") || desc.contains("Read"));
    }

    #[tokio::test]
    async fn test_todoread_parameters_schema() {
        let todo_write = Arc::new(TodoWriteTool::new());
        let tool = TodoReadTool::new(todo_write);
        let schema = tool.parameters_schema();

        assert_eq!(schema["type"], "object");
    }

    // ==================== Edge Cases ====================

    #[tokio::test]
    async fn test_todoread_handles_empty_input() {
        let todo_write = Arc::new(TodoWriteTool::new());
        let tool = TodoReadTool::new(todo_write);
        let ctx = create_test_context("empty-input-test");

        // Should work with empty object
        let result = tool.execute(json!({}), &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
    }

    #[tokio::test]
    async fn test_todoread_ignores_extra_input_fields() {
        let todo_write = Arc::new(TodoWriteTool::new());
        let tool = TodoReadTool::new(todo_write.clone());
        let ctx = create_test_context("extra-fields-test");

        todo_write
            .execute(
                json!({
                    "todos": [{"content": "Task", "status": "pending", "activeForm": "Task"}]
                }),
                &ctx,
            )
            .await;

        // Extra fields should be ignored
        let result = tool
            .execute(
                json!({
                    "extra_field": "ignored",
                    "another": 123
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: TodoReadOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.count, 1);
    }

    #[tokio::test]
    async fn test_todoread_preserves_unicode_content() {
        let todo_write = Arc::new(TodoWriteTool::new());
        let tool = TodoReadTool::new(todo_write.clone());
        let ctx = create_test_context("unicode-read-test");

        todo_write
            .execute(
                json!({
                    "todos": [{
                        "content": "Unicode 测试 🚀 العربية",
                        "status": "pending",
                        "activeForm": "Unicode test"
                    }]
                }),
                &ctx,
            )
            .await;

        let result = tool.execute(json!({}), &ctx).await;
        let output: TodoReadOutput = serde_json::from_str(&result.output).unwrap();

        assert!(output.todos[0].content.contains("🚀"));
        assert!(output.todos[0].content.contains("测试"));
    }

    #[tokio::test]
    async fn test_todoread_preserves_activeform() {
        let todo_write = Arc::new(TodoWriteTool::new());
        let tool = TodoReadTool::new(todo_write.clone());
        let ctx = create_test_context("activeform-test");

        todo_write
            .execute(
                json!({
                    "todos": [{
                        "content": "Task content",
                        "status": "in_progress",
                        "activeForm": "Working on task content"
                    }]
                }),
                &ctx,
            )
            .await;

        let result = tool.execute(json!({}), &ctx).await;
        let output: TodoReadOutput = serde_json::from_str(&result.output).unwrap();

        assert_eq!(output.todos[0].active_form, "Working on task content");
    }

    // ==================== Write-Read Roundtrip ====================

    #[tokio::test]
    async fn test_todowrite_then_todoread_roundtrip() {
        let todo_write = Arc::new(TodoWriteTool::new());
        let tool = TodoReadTool::new(todo_write.clone());
        let ctx = create_test_context("roundtrip-test");

        let original_todos = vec![
            json!({"content": "First task", "status": "completed", "activeForm": "First"}),
            json!({"content": "Second task", "status": "in_progress", "activeForm": "Second"}),
            json!({"content": "Third task", "status": "pending", "activeForm": "Third"}),
        ];

        // Write
        todo_write
            .execute(
                json!({
                    "todos": original_todos
                }),
                &ctx,
            )
            .await;

        // Read
        let result = tool.execute(json!({}), &ctx).await;
        let output: TodoReadOutput = serde_json::from_str(&result.output).unwrap();

        // Verify roundtrip
        assert_eq!(output.count, 3);
        assert_eq!(output.todos[0].content, "First task");
        assert!(matches!(output.todos[0].status, TodoStatus::Completed));
        assert_eq!(output.todos[1].content, "Second task");
        assert!(matches!(output.todos[1].status, TodoStatus::InProgress));
        assert_eq!(output.todos[2].content, "Third task");
        assert!(matches!(output.todos[2].status, TodoStatus::Pending));
    }
}
