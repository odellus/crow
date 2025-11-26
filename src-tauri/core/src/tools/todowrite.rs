//! TodoWrite tool - manages todo lists during coding sessions
//! Critical for agent planning and progress tracking
//!
//! Sibling sessions (dual-agent executor/arbiter) share the same todo list
//! via share_sessions() which maps both session IDs to a shared key.

use super::ToolContext;
use super::{Tool, ToolResult, ToolStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

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
    // Store todos per session (or shared key for siblings)
    todos: Arc<RwLock<HashMap<String, Vec<TodoItem>>>>,
    // Maps session_id -> shared_key for sibling sessions
    shared_keys: Arc<RwLock<HashMap<String, String>>>,
}

impl TodoWriteTool {
    pub fn new() -> Self {
        Self {
            todos: Arc::new(RwLock::new(HashMap::new())),
            shared_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Make two sessions share the same todo state (for dual-agent mode)
    /// Both session IDs will map to the same underlying storage key
    pub fn share_sessions(&self, session_a: &str, session_b: &str) {
        let shared_key = session_a.to_string(); // Use first session as the shared key
        let mut keys = self.shared_keys.write();
        keys.insert(session_a.to_string(), shared_key.clone());
        keys.insert(session_b.to_string(), shared_key);
    }

    /// Get the effective storage key for a session
    /// Returns shared key if session is part of a sibling pair, otherwise session_id
    fn get_todo_key(&self, session_id: &str) -> String {
        self.shared_keys
            .read()
            .get(session_id)
            .cloned()
            .unwrap_or_else(|| session_id.to_string())
    }

    pub fn get_todos(&self, session_id: &str) -> Vec<TodoItem> {
        let todo_key = self.get_todo_key(session_id);
        self.todos
            .read()
            .get(&todo_key)
            .cloned()
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

        // Use shared key for sibling sessions (dual-agent), otherwise session_id
        let todo_key = self.get_todo_key(&ctx.session_id);

        // Store todos in memory
        {
            let mut todos = self.todos.write();
            todos.insert(todo_key.clone(), todo_input.todos.clone());
        }

        // Persist to disk (like OpenCode does)
        // Write to ~/.local/share/crow/storage/todo/{todo_key}.json
        if let Ok(global_paths) = std::env::var("HOME").map(|home| {
            let data_home =
                std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{}/.local/share", home));
            std::path::PathBuf::from(data_home).join("crow/storage/todo")
        }) {
            if let Err(e) = std::fs::create_dir_all(&global_paths) {
                tracing::warn!("Failed to create todo storage directory: {}", e);
            } else {
                let todo_file = global_paths.join(format!("{}.json", todo_key));
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
                "session_id": ctx.session_id,
                "todo_key": todo_key,
                "count": output.count,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    async fn test_todowrite_creates_new_todo() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-session-1");

        let result = tool
            .execute(
                json!({
                    "todos": [{
                        "content": "Implement feature",
                        "status": "pending",
                        "activeForm": "Implementing feature"
                    }]
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: TodoWriteOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.count, 1);
        assert_eq!(output.todos[0].content, "Implement feature");
    }

    #[tokio::test]
    async fn test_todowrite_handles_multiple_todos() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-session-2");

        let result = tool
            .execute(
                json!({
                    "todos": [
                        {"content": "Task 1", "status": "completed", "activeForm": "Doing task 1"},
                        {"content": "Task 2", "status": "in_progress", "activeForm": "Doing task 2"},
                        {"content": "Task 3", "status": "pending", "activeForm": "Doing task 3"},
                        {"content": "Task 4", "status": "pending", "activeForm": "Doing task 4"},
                        {"content": "Task 5", "status": "pending", "activeForm": "Doing task 5"}
                    ]
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: TodoWriteOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.count, 5);
    }

    #[tokio::test]
    async fn test_todowrite_replaces_entire_list() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-session-replace");

        // First write
        tool.execute(
            json!({
                "todos": [
                    {"content": "Old task 1", "status": "pending", "activeForm": "Old"},
                    {"content": "Old task 2", "status": "pending", "activeForm": "Old"}
                ]
            }),
            &ctx,
        )
        .await;

        // Second write should replace
        let result = tool
            .execute(
                json!({
                    "todos": [
                        {"content": "New task", "status": "in_progress", "activeForm": "New"}
                    ]
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: TodoWriteOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.count, 1);
        assert_eq!(output.todos[0].content, "New task");
    }

    #[tokio::test]
    async fn test_todowrite_empty_list_clears_todos() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-session-clear");

        // First write some todos
        tool.execute(
            json!({
                "todos": [
                    {"content": "Task 1", "status": "pending", "activeForm": "Task 1"}
                ]
            }),
            &ctx,
        )
        .await;

        // Clear with empty list
        let result = tool
            .execute(
                json!({
                    "todos": []
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: TodoWriteOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.count, 0);
        assert!(output.todos.is_empty());
    }

    #[tokio::test]
    async fn test_todowrite_preserves_order() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-session-order");

        let result = tool
            .execute(
                json!({
                    "todos": [
                        {"content": "First", "status": "pending", "activeForm": "First"},
                        {"content": "Second", "status": "pending", "activeForm": "Second"},
                        {"content": "Third", "status": "pending", "activeForm": "Third"}
                    ]
                }),
                &ctx,
            )
            .await;

        let output: TodoWriteOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.todos[0].content, "First");
        assert_eq!(output.todos[1].content, "Second");
        assert_eq!(output.todos[2].content, "Third");
    }

    // ==================== Status Tests ====================

    #[tokio::test]
    async fn test_todowrite_validates_status_pending() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-status-pending");

        let result = tool
            .execute(
                json!({
                    "todos": [{
                        "content": "Task",
                        "status": "pending",
                        "activeForm": "Task"
                    }]
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: TodoWriteOutput = serde_json::from_str(&result.output).unwrap();
        assert!(matches!(output.todos[0].status, TodoStatus::Pending));
    }

    #[tokio::test]
    async fn test_todowrite_validates_status_in_progress() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-status-in-progress");

        let result = tool
            .execute(
                json!({
                    "todos": [{
                        "content": "Task",
                        "status": "in_progress",
                        "activeForm": "Task"
                    }]
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: TodoWriteOutput = serde_json::from_str(&result.output).unwrap();
        assert!(matches!(output.todos[0].status, TodoStatus::InProgress));
    }

    #[tokio::test]
    async fn test_todowrite_validates_status_completed() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-status-completed");

        let result = tool
            .execute(
                json!({
                    "todos": [{
                        "content": "Task",
                        "status": "completed",
                        "activeForm": "Task"
                    }]
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: TodoWriteOutput = serde_json::from_str(&result.output).unwrap();
        assert!(matches!(output.todos[0].status, TodoStatus::Completed));
    }

    #[tokio::test]
    async fn test_todowrite_rejects_invalid_status() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-invalid-status");

        let result = tool
            .execute(
                json!({
                    "todos": [{
                        "content": "Task",
                        "status": "invalid_status",
                        "activeForm": "Task"
                    }]
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    // ==================== Input Validation ====================

    #[tokio::test]
    async fn test_todowrite_validates_content_required() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-missing-content");

        let result = tool
            .execute(
                json!({
                    "todos": [{
                        "status": "pending",
                        "activeForm": "Task"
                    }]
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_todowrite_validates_activeform_required() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-missing-activeform");

        let result = tool
            .execute(
                json!({
                    "todos": [{
                        "content": "Task",
                        "status": "pending"
                    }]
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_todowrite_invalid_json_input() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-invalid-json");

        let result = tool
            .execute(
                json!({
                    "todos": "not an array"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_todowrite_missing_todos_field() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-missing-todos");

        let result = tool.execute(json!({}), &ctx).await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    // ==================== Content Edge Cases ====================

    #[tokio::test]
    async fn test_todowrite_handles_unicode_content() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-unicode");

        let result = tool
            .execute(
                json!({
                    "todos": [{
                        "content": "实现功能 🚀 مرحبا",
                        "status": "pending",
                        "activeForm": "实现功能中 🔧"
                    }]
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: TodoWriteOutput = serde_json::from_str(&result.output).unwrap();
        assert!(output.todos[0].content.contains("🚀"));
    }

    #[tokio::test]
    async fn test_todowrite_handles_special_characters() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-special-chars");

        let result = tool
            .execute(
                json!({
                    "todos": [{
                        "content": "Fix <xml> & \"quotes\" 'apostrophes'",
                        "status": "pending",
                        "activeForm": "Fixing {json} [array] $var"
                    }]
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: TodoWriteOutput = serde_json::from_str(&result.output).unwrap();
        assert!(output.todos[0].content.contains("<xml>"));
    }

    #[tokio::test]
    async fn test_todowrite_handles_very_long_content() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-long-content");

        let long_content = "x".repeat(10000);
        let result = tool
            .execute(
                json!({
                    "todos": [{
                        "content": long_content,
                        "status": "pending",
                        "activeForm": "Processing"
                    }]
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: TodoWriteOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.todos[0].content.len(), 10000);
    }

    #[tokio::test]
    async fn test_todowrite_handles_newlines_in_content() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("test-newlines");

        let result = tool
            .execute(
                json!({
                    "todos": [{
                        "content": "Line 1\nLine 2\nLine 3",
                        "status": "pending",
                        "activeForm": "Multi-line task"
                    }]
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: TodoWriteOutput = serde_json::from_str(&result.output).unwrap();
        assert!(output.todos[0].content.contains("\n"));
    }

    // ==================== Session Isolation ====================

    #[tokio::test]
    async fn test_todo_session_isolation() {
        let tool = TodoWriteTool::new();

        let ctx_a = create_test_context("session-A");
        let ctx_b = create_test_context("session-B");

        // Write to session A
        tool.execute(
            json!({
                "todos": [{"content": "Task A", "status": "pending", "activeForm": "A"}]
            }),
            &ctx_a,
        )
        .await;

        // Write to session B
        tool.execute(
            json!({
                "todos": [{"content": "Task B", "status": "pending", "activeForm": "B"}]
            }),
            &ctx_b,
        )
        .await;

        // Verify isolation
        let todos_a = tool.get_todos("session-A");
        let todos_b = tool.get_todos("session-B");

        assert_eq!(todos_a.len(), 1);
        assert_eq!(todos_a[0].content, "Task A");

        assert_eq!(todos_b.len(), 1);
        assert_eq!(todos_b[0].content, "Task B");
    }

    #[tokio::test]
    async fn test_todo_different_sessions_independent() {
        let tool = TodoWriteTool::new();

        let ctx_1 = create_test_context("independent-1");
        let ctx_2 = create_test_context("independent-2");

        // Write 3 todos to session 1
        tool.execute(
            json!({
                "todos": [
                    {"content": "1A", "status": "pending", "activeForm": "1A"},
                    {"content": "1B", "status": "pending", "activeForm": "1B"},
                    {"content": "1C", "status": "pending", "activeForm": "1C"}
                ]
            }),
            &ctx_1,
        )
        .await;

        // Write 1 todo to session 2
        tool.execute(
            json!({
                "todos": [{"content": "2A", "status": "completed", "activeForm": "2A"}]
            }),
            &ctx_2,
        )
        .await;

        // Verify counts are independent
        assert_eq!(tool.get_todos("independent-1").len(), 3);
        assert_eq!(tool.get_todos("independent-2").len(), 1);
    }

    // ==================== Metadata Tests ====================

    #[tokio::test]
    async fn test_todowrite_returns_correct_metadata() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("metadata-test");

        let result = tool
            .execute(
                json!({
                    "todos": [
                        {"content": "Task 1", "status": "pending", "activeForm": "Task 1"},
                        {"content": "Task 2", "status": "pending", "activeForm": "Task 2"}
                    ]
                }),
                &ctx,
            )
            .await;

        assert_eq!(
            result.metadata.get("session_id").and_then(|v| v.as_str()),
            Some("metadata-test")
        );
        assert_eq!(
            result.metadata.get("count").and_then(|v| v.as_u64()),
            Some(2)
        );
    }

    // ==================== Tool Interface Tests ====================

    #[tokio::test]
    async fn test_todowrite_tool_name() {
        let tool = TodoWriteTool::new();
        assert_eq!(tool.name(), "todowrite");
    }

    #[tokio::test]
    async fn test_todowrite_tool_description() {
        let tool = TodoWriteTool::new();
        let desc = tool.description();
        assert!(desc.contains("todo"));
        assert!(desc.contains("CRITICAL"));
    }

    #[tokio::test]
    async fn test_todowrite_parameters_schema() {
        let tool = TodoWriteTool::new();
        let schema = tool.parameters_schema();

        assert_eq!(schema["type"], "object");

        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("todos")));

        let props = schema["properties"].as_object().unwrap();
        assert!(props.contains_key("todos"));
    }

    // ==================== get_todos Method Tests ====================

    #[tokio::test]
    async fn test_get_todos_returns_current_state() {
        let tool = TodoWriteTool::new();
        let ctx = create_test_context("get-todos-test");

        tool.execute(
            json!({
                "todos": [
                    {"content": "Task 1", "status": "in_progress", "activeForm": "Task 1"}
                ]
            }),
            &ctx,
        )
        .await;

        let todos = tool.get_todos("get-todos-test");
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].content, "Task 1");
        assert!(matches!(todos[0].status, TodoStatus::InProgress));
    }

    #[tokio::test]
    async fn test_get_todos_returns_empty_for_unknown_session() {
        let tool = TodoWriteTool::new();
        let todos = tool.get_todos("nonexistent-session");
        assert!(todos.is_empty());
    }

    // ==================== Legacy test (original) ====================

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
