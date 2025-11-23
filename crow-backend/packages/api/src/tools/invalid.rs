//! Invalid tool - placeholder for error handling
//! Mirrors OpenCode's invalid tool

use async_trait::async_trait;
use serde_json::{json, Value};

use super::{Tool, ToolContext, ToolResult, ToolStatus};

/// Invalid tool - used for error handling when tool arguments are invalid
pub struct InvalidTool;

#[async_trait]
impl Tool for InvalidTool {
    fn name(&self) -> &str {
        "invalid"
    }

    fn description(&self) -> &str {
        "Do not use"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "tool": {
                    "type": "string",
                    "description": "The tool name"
                },
                "error": {
                    "type": "string",
                    "description": "The error message"
                }
            },
            "required": ["tool", "error"]
        })
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> ToolResult {
        let error = input
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");

        ToolResult {
            status: ToolStatus::Error,
            output: format!("The arguments provided to the tool are invalid: {}", error),
            error: Some(error.to_string()),
            metadata: json!({}),
        }
    }
}
