//! work_completed tool - used by discriminator to signal work is done
//! Signals completion of dual-pair session (matches OpenCode's task_done)

use super::{Tool, ToolResult, ToolStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Arguments for work_completed tool (matches OpenCode's task_done API)
#[derive(Debug, Deserialize, Serialize)]
pub struct WorkCompletedArgs {
    /// Confirm the work is complete (must be true)
    pub ready: bool,
}

/// work_completed tool - discriminator signals work is done
pub struct WorkCompletedTool;

#[async_trait]
impl Tool for WorkCompletedTool {
    fn name(&self) -> &str {
        "work_completed"
    }

    fn description(&self) -> &str {
        "Signal that the work is complete and satisfactory. Use this when all requirements are met, tests pass, and code quality is good."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "ready": {
                    "type": "boolean",
                    "const": true,
                    "description": "Confirm the task is complete"
                }
            },
            "required": ["ready"]
        })
    }

    async fn execute(&self, args: Value, _ctx: &super::ToolContext) -> ToolResult {
        let args: WorkCompletedArgs = match serde_json::from_value(args) {
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

        // Just mark completion - summary is in the discriminator's text response
        ToolResult {
            status: ToolStatus::Completed,
            output: "Work marked as complete".to_string(),
            error: None,
            metadata: json!({
                "ready": args.ready,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_work_completed() {
        let tool = WorkCompletedTool;
        let args = json!({
            "ready": true
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(args, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("complete"));
        assert_eq!(result.metadata["ready"], true);
    }

    #[test]
    fn test_work_completed_schema() {
        let tool = WorkCompletedTool;
        let schema = tool.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["ready"].is_object());
        assert_eq!(schema["properties"]["ready"]["const"], true);
        assert!(schema["required"].is_array());
    }
}
