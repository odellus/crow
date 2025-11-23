//! LSP Diagnostics tool - get errors and warnings for a file

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

use super::{Tool, ToolContext, ToolResult, ToolStatus};

pub struct LspDiagnosticsTool;

#[derive(Deserialize)]
struct LspDiagnosticsInput {
    path: String,
}

#[async_trait]
impl Tool for LspDiagnosticsTool {
    fn name(&self) -> &str {
        "lsp_diagnostics"
    }

    fn description(&self) -> &str {
        r#"Get language server diagnostics (errors, warnings) for a file.

This tool retrieves diagnostic information from the language server including:
- Syntax errors
- Type errors
- Warnings
- Lint issues

Use this to check for problems in code before or after making changes."#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file (absolute or relative to working directory)"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let input: LspDiagnosticsInput = match serde_json::from_value(input) {
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

        // Resolve file path
        let file_path = if PathBuf::from(&input.path).is_absolute() {
            PathBuf::from(&input.path)
        } else {
            ctx.working_dir.join(&input.path)
        };

        if !file_path.exists() {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!("File not found: {}", file_path.display())),
                metadata: json!({}),
            };
        }

        // Get LSP manager from context
        let lsp = match &ctx.lsp {
            Some(lsp) => lsp,
            None => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some("LSP not available".to_string()),
                    metadata: json!({}),
                };
            }
        };

        // Touch file to ensure it's open and wait for diagnostics
        lsp.touch_file(&file_path, &ctx.working_dir, true).await;

        // Get all diagnostics
        let all_diagnostics = lsp.diagnostics().await;

        // Filter to just this file
        let file_diagnostics = all_diagnostics.get(&file_path);

        match file_diagnostics {
            None => ToolResult {
                status: ToolStatus::Completed,
                output: "No diagnostics found for this file.".to_string(),
                error: None,
                metadata: json!({}),
            },
            Some(diagnostics) if diagnostics.is_empty() => ToolResult {
                status: ToolStatus::Completed,
                output: "No diagnostics found for this file.".to_string(),
                error: None,
                metadata: json!({}),
            },
            Some(diagnostics) => {
                let mut output = format!("Found {} diagnostic(s):\n\n", diagnostics.len());

                for diag in diagnostics {
                    output.push_str(&diag.pretty());
                    output.push('\n');
                }

                ToolResult {
                    status: ToolStatus::Completed,
                    output,
                    error: None,
                    metadata: json!({}),
                }
            }
        }
    }
}
