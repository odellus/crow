//! task_complete tool - used by arbiter to signal dual-agent loop completion
//!
//! This tool is only available to the arbiter agent in dual-agent mode.
//! When called, it signals that the work has been verified and the loop should terminate.

use super::{Tool, ToolContext, ToolResult, ToolStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Arguments for task_complete tool
#[derive(Debug, Deserialize, Serialize)]
pub struct TaskCompleteArgs {
    /// Summary of what was accomplished
    pub summary: String,
    /// How the work was verified (tests run, manual checks, etc.)
    pub verification: String,
}

/// task_complete tool - arbiter signals work is done and verified
pub struct TaskCompleteTool;

#[async_trait]
impl Tool for TaskCompleteTool {
    fn name(&self) -> &str {
        "task_complete"
    }

    fn description(&self) -> &str {
        "Mark the task as complete. Call ONCE with summary and verification details. \
         After calling this tool, write a final response summarizing the completed work, \
         what was verified, and any relevant file paths or documentation links. \
         This final response is what the calling agent sees. Do not call any more tools after this."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "summary": {
                    "type": "string",
                    "description": "Summary of what was accomplished"
                },
                "verification": {
                    "type": "string",
                    "description": "How the work was verified (e.g., 'ran cargo test - all 15 tests pass')"
                }
            },
            "required": ["summary", "verification"]
        })
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> ToolResult {
        let args: TaskCompleteArgs = match serde_json::from_value(args) {
            Ok(a) => a,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Invalid arguments: {}", e)),
                    metadata: json!({}),
                }
            }
        };

        ToolResult {
            status: ToolStatus::Completed,
            output: format!(
                "Task complete.\n\nSummary: {}\n\nVerification: {}",
                args.summary, args.verification
            ),
            error: None,
            metadata: json!({
                "task_complete": true,
                "summary": args.summary,
                "verification": args.verification,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_task_complete_success() {
        let tool = TaskCompleteTool;
        let args = json!({
            "summary": "Implemented fibonacci function with memoization",
            "verification": "Ran cargo test - all 5 tests pass including edge cases"
        });

        let ctx = ToolContext::new(
            "test-session".to_string(),
            "test-message".to_string(),
            "arbiter".to_string(),
            PathBuf::from("/tmp"),
        );

        let result = tool.execute(args, &ctx).await;

        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("Task complete"));
        assert!(result.output.contains("fibonacci"));
        assert!(result.output.contains("cargo test"));
        assert_eq!(result.metadata["task_complete"], true);
        assert_eq!(
            result.metadata["summary"],
            "Implemented fibonacci function with memoization"
        );
    }

    #[tokio::test]
    async fn test_task_complete_missing_fields() {
        let tool = TaskCompleteTool;
        let args = json!({
            "summary": "Did the thing"
            // missing verification
        });

        let ctx = ToolContext::new(
            "test-session".to_string(),
            "test-message".to_string(),
            "arbiter".to_string(),
            PathBuf::from("/tmp"),
        );

        let result = tool.execute(args, &ctx).await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_task_complete_schema() {
        let tool = TaskCompleteTool;
        let schema = tool.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["summary"].is_object());
        assert!(schema["properties"]["verification"].is_object());

        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("summary")));
        assert!(required.contains(&json!("verification")));
    }
}
