//! Glob tool for file pattern matching
use super::ToolContext;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::process::Command;

use crate::tools::{Tool, ToolResult, ToolStatus};

/// Glob tool for finding files by pattern
#[derive(Clone)]
pub struct GlobTool;

#[derive(Deserialize)]
struct GlobInput {
    pattern: String,
    #[serde(default)]
    path: Option<String>,
}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Fast file pattern matching tool that works with any codebase size. Supports glob patterns like \"**/*.js\" or \"src/**/*.ts\". Returns matching file paths sorted by modification time."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The glob pattern to match files against"
                },
                "path": {
                    "type": "string",
                    "description": "The directory to search in. If not specified, the current working directory will be used. IMPORTANT: Omit this field to use the default directory. DO NOT enter \"undefined\" or \"null\" - simply omit it for the default behavior. Must be a valid directory path if provided."
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> ToolResult {
        let input: GlobInput = match serde_json::from_value(input) {
            Ok(i) => i,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Invalid input: {}", e)),
                    metadata: serde_json::json!({}),
                }
            }
        };

        let search_path = input.path.unwrap_or_else(|| ".".to_string());

        // Use ripgrep with --files and --glob for fast file listing
        let output = Command::new("rg")
            .arg("--files")
            .arg("--glob")
            .arg(&input.pattern)
            .current_dir(&search_path)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let files: Vec<&str> = stdout.lines().collect();
                    let count = files.len();

                    let truncated = count > 100;
                    let limit = if truncated { 100 } else { count };

                    let mut output_lines = Vec::new();
                    if count == 0 {
                        output_lines.push("No files found".to_string());
                    } else {
                        for file in files.iter().take(limit) {
                            output_lines.push(file.to_string());
                        }
                        if truncated {
                            output_lines.push(String::new());
                            output_lines.push("(Results are truncated. Consider using a more specific path or pattern.)".to_string());
                        }
                    }

                    ToolResult {
                        status: ToolStatus::Completed,
                        output: output_lines.join("\n"),
                        error: None,
                        metadata: serde_json::json!({
                            "count": count,
                            "truncated": truncated
                        }),
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    ToolResult {
                        status: ToolStatus::Error,
                        output: String::new(),
                        error: Some(stderr),
                        metadata: serde_json::json!({}),
                    }
                }
            }
            Err(e) => ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!("Failed to execute glob: {}", e)),
                metadata: serde_json::json!({}),
            },
        }
    }
}
