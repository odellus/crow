//! MultiEdit tool - apply multiple edits to the same file sequentially

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use super::edit::EditTool;
use super::{Tool, ToolContext, ToolResult, ToolStatus};

pub struct MultiEditTool;

#[derive(Deserialize)]
struct MultiEditInput {
    #[serde(rename = "filePath")]
    file_path: String,
    edits: Vec<EditOperation>,
}

#[derive(Deserialize)]
struct EditOperation {
    #[serde(rename = "oldString")]
    old_string: String,
    #[serde(rename = "newString")]
    new_string: String,
    #[serde(default, rename = "replaceAll")]
    replace_all: bool,
}

#[async_trait]
impl Tool for MultiEditTool {
    fn name(&self) -> &str {
        "multiedit"
    }

    fn description(&self) -> &str {
        r#"Apply multiple edits to the same file sequentially.

This tool allows you to make several changes to a single file without re-reading it between edits.
Each edit is applied in order, so later edits see the results of earlier ones.

Use this when you need to:
- Make multiple related changes to a file
- Refactor code with several replacements
- Update multiple occurrences of different strings

Note: All edits must be for the same file. For editing multiple files, use the batch tool with individual edit calls."#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "The absolute path to the file to modify"
                },
                "edits": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "oldString": {
                                "type": "string",
                                "description": "The text to replace"
                            },
                            "newString": {
                                "type": "string",
                                "description": "The text to replace it with (must be different from oldString)"
                            },
                            "replaceAll": {
                                "type": "boolean",
                                "description": "Replace all occurrences of oldString (default false)"
                            }
                        },
                        "required": ["oldString", "newString"]
                    },
                    "description": "Array of edit operations to perform sequentially on the file"
                }
            },
            "required": ["filePath", "edits"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let input: MultiEditInput = match serde_json::from_value(input) {
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

        if input.edits.is_empty() {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some("Provide at least one edit operation".to_string()),
                metadata: json!({}),
            };
        }

        let edit_tool = EditTool;
        let mut results = Vec::new();
        let mut last_output = String::new();
        let mut all_succeeded = true;

        for (index, edit) in input.edits.iter().enumerate() {
            let edit_input = json!({
                "filePath": input.file_path,
                "oldString": edit.old_string,
                "newString": edit.new_string,
                "replaceAll": edit.replace_all
            });

            let result = edit_tool.execute(edit_input, ctx).await;

            let succeeded = result.status == ToolStatus::Completed;
            if !succeeded {
                all_succeeded = false;
            }

            results.push(json!({
                "index": index,
                "success": succeeded,
                "output": result.output,
                "error": result.error,
            }));

            last_output = result.output;

            // Stop on first error
            if !succeeded {
                break;
            }
        }

        let completed = results.len();
        let total = input.edits.len();

        ToolResult {
            status: if all_succeeded {
                ToolStatus::Completed
            } else {
                ToolStatus::Error
            },
            output: if all_succeeded {
                format!(
                    "Successfully applied {}/{} edits.\n\n{}",
                    completed, total, last_output
                )
            } else {
                format!(
                    "Applied {}/{} edits before error.\n\n{}",
                    completed - 1,
                    total,
                    last_output
                )
            },
            error: if all_succeeded {
                None
            } else {
                Some("One or more edits failed".to_string())
            },
            metadata: json!({
                "filePath": input.file_path,
                "totalEdits": total,
                "completedEdits": if all_succeeded { completed } else { completed - 1 },
                "results": results,
            }),
        }
    }
}
