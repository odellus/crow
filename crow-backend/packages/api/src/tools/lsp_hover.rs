//! LSP Hover tool - get type hints and documentation at a position

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

use super::{Tool, ToolContext, ToolResult, ToolStatus};

pub struct LspHoverTool;

#[derive(Deserialize)]
struct LspHoverInput {
    file: String,
    line: u32,
    character: u32,
}

#[async_trait]
impl Tool for LspHoverTool {
    fn name(&self) -> &str {
        "lsp_hover"
    }

    fn description(&self) -> &str {
        r#"Get hover information (type hints, documentation) at a specific position in a file.

This tool provides IDE-like hover tooltips showing:
- Type information for variables, functions, etc.
- Documentation comments
- Function signatures

Use this when you need to understand the type or documentation of code at a specific location."#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file": {
                    "type": "string",
                    "description": "Path to the file (absolute or relative to working directory)"
                },
                "line": {
                    "type": "number",
                    "description": "Line number (0-indexed)"
                },
                "character": {
                    "type": "number",
                    "description": "Character position on the line (0-indexed)"
                }
            },
            "required": ["file", "line", "character"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let input: LspHoverInput = match serde_json::from_value(input) {
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
        let file_path = if PathBuf::from(&input.file).is_absolute() {
            PathBuf::from(&input.file)
        } else {
            ctx.working_dir.join(&input.file)
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

        // Touch file first to ensure it's open in the LSP server
        lsp.touch_file(&file_path, &ctx.working_dir, true).await;

        // Get hover information
        let results = lsp
            .hover(&file_path, &ctx.working_dir, input.line, input.character)
            .await;

        if results.is_empty() {
            return ToolResult {
                status: ToolStatus::Completed,
                output: "No hover information available at this position.".to_string(),
                error: None,
                metadata: json!({}),
            };
        }

        // Format results
        let mut output = String::new();

        for result in results {
            if let Some(contents) = result.get("contents") {
                let text = format_hover_contents(contents);
                if !text.is_empty() {
                    if !output.is_empty() {
                        output.push_str("\n---\n");
                    }
                    output.push_str(&text);
                }
            }
        }

        if output.is_empty() {
            ToolResult {
                status: ToolStatus::Completed,
                output: "No hover information available at this position.".to_string(),
                error: None,
                metadata: json!({}),
            }
        } else {
            ToolResult {
                status: ToolStatus::Completed,
                output,
                error: None,
                metadata: json!({}),
            }
        }
    }
}

/// Format hover contents from LSP response
fn format_hover_contents(contents: &Value) -> String {
    match contents {
        Value::String(s) => s.clone(),
        Value::Object(obj) => {
            // MarkedString or MarkupContent
            if let Some(value) = obj.get("value").and_then(|v| v.as_str()) {
                value.to_string()
            } else if let Some(language) = obj.get("language").and_then(|v| v.as_str()) {
                if let Some(value) = obj.get("value").and_then(|v| v.as_str()) {
                    format!("```{}\n{}\n```", language, value)
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        }
        Value::Array(arr) => arr
            .iter()
            .map(format_hover_contents)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_hover_string() {
        let contents = json!("Simple text");
        assert_eq!(format_hover_contents(&contents), "Simple text");
    }

    #[test]
    fn test_format_hover_markup() {
        let contents = json!({
            "kind": "markdown",
            "value": "**bold** text"
        });
        assert_eq!(format_hover_contents(&contents), "**bold** text");
    }

    #[test]
    fn test_format_hover_marked_string() {
        let contents = json!({
            "language": "rust",
            "value": "fn main() {}"
        });
        assert_eq!(
            format_hover_contents(&contents),
            "```rust\nfn main() {}\n```"
        );
    }

    #[test]
    fn test_format_hover_array() {
        let contents = json!(["first", "second"]);
        assert_eq!(format_hover_contents(&contents), "first\n\nsecond");
    }
}
