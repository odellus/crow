//! Batch tool - execute multiple tools in parallel

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use super::{Tool, ToolContext, ToolRegistry, ToolResult, ToolStatus};

pub struct BatchTool {
    registry: Arc<ToolRegistry>,
}

impl BatchTool {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }
}

#[derive(Deserialize)]
struct BatchInput {
    tool_calls: Vec<ToolCall>,
}

#[derive(Deserialize, Clone)]
struct ToolCall {
    tool: String,
    parameters: Value,
}

const DISALLOWED: &[&str] = &["batch", "edit", "todoread"];
const MAX_BATCH_SIZE: usize = 10;

#[async_trait]
impl Tool for BatchTool {
    fn name(&self) -> &str {
        "batch"
    }

    fn description(&self) -> &str {
        r#"Execute up to 10 tool calls in parallel for optimal performance.

Use this tool when you need to run multiple independent operations simultaneously.
This is especially useful for:
- Reading multiple files at once
- Running multiple grep/glob searches
- Fetching multiple web pages
- Any combination of independent tool calls

Restrictions:
- Maximum 10 tools per batch
- Cannot batch: batch, edit, todoread
- All tools execute in parallel, so they must be independent

Keep using the batch tool for optimal performance!"#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "tool_calls": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "tool": {
                                "type": "string",
                                "description": "The name of the tool to execute"
                            },
                            "parameters": {
                                "type": "object",
                                "description": "Parameters for the tool"
                            }
                        },
                        "required": ["tool", "parameters"]
                    },
                    "minItems": 1,
                    "description": "Array of tool calls to execute in parallel"
                }
            },
            "required": ["tool_calls"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let input: BatchInput = match serde_json::from_value(input) {
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

        if input.tool_calls.is_empty() {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some("Provide at least one tool call".to_string()),
                metadata: json!({}),
            };
        }

        // Limit to MAX_BATCH_SIZE
        let tool_calls: Vec<_> = input.tool_calls.into_iter().take(MAX_BATCH_SIZE).collect();
        let total_calls = tool_calls.len();

        // Execute all tools in parallel
        let mut handles = Vec::new();

        for call in tool_calls {
            // Check for disallowed tools
            if DISALLOWED.contains(&call.tool.as_str()) {
                let tool_name = call.tool.clone();
                handles.push(tokio::spawn(async move {
                    (
                        tool_name,
                        ToolResult {
                            status: ToolStatus::Error,
                            output: String::new(),
                            error: Some(format!(
                                "Tool '{}' is not allowed in batch. Disallowed tools: {}",
                                call.tool,
                                DISALLOWED.join(", ")
                            )),
                            metadata: json!({}),
                        },
                    )
                }));
                continue;
            }

            let registry = Arc::clone(&self.registry);
            let ctx = ctx.clone();
            let tool_name = call.tool.clone();
            let params = call.parameters.clone();

            handles.push(tokio::spawn(async move {
                let result = registry
                    .execute(&tool_name, params, &ctx)
                    .await
                    .unwrap_or_else(|e| ToolResult {
                        status: ToolStatus::Error,
                        output: String::new(),
                        error: Some(e),
                        metadata: serde_json::json!({}),
                    });
                (tool_name, result)
            }));
        }

        // Collect results
        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;

        for handle in handles {
            match handle.await {
                Ok((tool_name, result)) => {
                    let success = result.status == ToolStatus::Completed;
                    if success {
                        successful += 1;
                    } else {
                        failed += 1;
                    }
                    results.push(json!({
                        "tool": tool_name,
                        "success": success,
                        "output": result.output,
                        "error": result.error,
                    }));
                }
                Err(e) => {
                    failed += 1;
                    results.push(json!({
                        "tool": "unknown",
                        "success": false,
                        "error": format!("Task panicked: {}", e),
                    }));
                }
            }
        }

        let output = if failed > 0 {
            format!(
                "Executed {}/{} tools successfully. {} failed.",
                successful, total_calls, failed
            )
        } else {
            format!(
                "All {} tools executed successfully.\n\nKeep using the batch tool for optimal performance in your next response!",
                successful
            )
        };

        ToolResult {
            status: if failed == total_calls {
                ToolStatus::Error
            } else {
                ToolStatus::Completed
            },
            output,
            error: None,
            metadata: json!({
                "totalCalls": total_calls,
                "successful": successful,
                "failed": failed,
                "details": results,
            }),
        }
    }
}
