//! Edit tool - modifies existing files using exact string replacements
//! This is the primary way the LLM modifies code

use super::{Tool, ToolResult, ToolStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::fs;

pub struct EditTool;

#[derive(Deserialize)]
struct EditInput {
    #[serde(rename = "filePath")]
    file_path: String,
    #[serde(rename = "oldString")]
    old_string: String,
    #[serde(rename = "newString")]
    new_string: String,
    #[serde(default)]
    replace_all: bool,
}

#[derive(Serialize, Deserialize)]
struct EditOutput {
    filepath: String,
    replacements: usize,
    old_length: usize,
    new_length: usize,
}

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        r#"Performs exact string replacements in files.

Usage:
- You must use your `Read` tool at least once in the conversation before editing. This tool will error if you attempt an edit without reading the file.
- When editing text from Read tool output, ensure you preserve the exact indentation (tabs/spaces) as it appears AFTER the line number prefix. The line number prefix format is: spaces + line number + tab. Everything after that tab is the actual file content to match. Never include any part of the line number prefix in the oldString or newString.
- ALWAYS prefer editing existing files in the codebase. NEVER write new files unless explicitly required.
- Only use emojis if the user explicitly requests it. Avoid adding emojis to files unless asked.
- The edit will FAIL if `oldString` is not found in the file with an error "oldString not found in content".
- The edit will FAIL if `oldString` is found multiple times in the file with an error "oldString found multiple times and requires more code context to uniquely identify the intended match". Either provide a larger string with more surrounding context to make it unique or use `replaceAll` to change every instance of `oldString`.
- Use `replaceAll` for replacing and renaming strings across the file. This parameter is useful if you want to rename a variable for instance."#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "oldString": {
                    "type": "string",
                    "description": "Exact string to replace (must match exactly including whitespace)"
                },
                "newString": {
                    "type": "string",
                    "description": "New string to replace with"
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "Replace all occurrences (default: false, replaces only first match)"
                }
            },
            "required": ["filePath", "oldString", "newString"]
        })
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let edit_input: EditInput = match serde_json::from_value(input) {
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

        // Read file
        let content = match fs::read_to_string(&edit_input.file_path).await {
            Ok(c) => c,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to read file: {}", e)),
                    metadata: json!({
                        "filepath": edit_input.file_path,
                    }),
                };
            }
        };

        // Count occurrences
        let occurrences = content.matches(&edit_input.old_string).count();

        if occurrences == 0 {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!(
                    "String not found in file. Make sure old_string matches EXACTLY."
                )),
                metadata: json!({
                    "filepath": edit_input.file_path,
                    "old_string_length": edit_input.old_string.len(),
                }),
            };
        }

        // Check for ambiguity (multiple matches when not replace_all)
        if !edit_input.replace_all && occurrences > 1 {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!(
                    "String appears {} times in file. Use replace_all:true or provide more context to make it unique.",
                    occurrences
                )),
                metadata: json!({
                    "filepath": edit_input.file_path,
                    "occurrences": occurrences,
                }),
            };
        }

        // Perform replacement
        let new_content = if edit_input.replace_all {
            content.replace(&edit_input.old_string, &edit_input.new_string)
        } else {
            content.replacen(&edit_input.old_string, &edit_input.new_string, 1)
        };

        let replacements = if edit_input.replace_all {
            occurrences
        } else {
            1
        };

        // Write file
        if let Err(e) = fs::write(&edit_input.file_path, &new_content).await {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!("Failed to write file: {}", e)),
                metadata: json!({
                    "filepath": edit_input.file_path,
                }),
            };
        }

        let output = EditOutput {
            filepath: edit_input.file_path.clone(),
            replacements,
            old_length: content.len(),
            new_length: new_content.len(),
        };

        ToolResult {
            status: ToolStatus::Completed,
            output: serde_json::to_string(&output).unwrap_or_default(),
            error: None,
            metadata: json!({
                "filepath": edit_input.file_path,
                "replacements": replacements,
                "size_delta": new_content.len() as i64 - content.len() as i64,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_edit_single_replacement() {
        let test_path = "/tmp/crow_test_edit.txt";
        fs::write(test_path, "Hello World\nHello Rust")
            .await
            .unwrap();

        let tool = EditTool;
        let input = json!({
            "filePath": test_path,
            "oldString": "Hello World",
            "newString": "Hello Crow"
        });

        let result = tool.execute(input).await;
        assert_eq!(result.status, ToolStatus::Completed);

        let content = fs::read_to_string(test_path).await.unwrap();
        assert_eq!(content, "Hello Crow\nHello Rust");

        fs::remove_file(test_path).await.unwrap();
    }

    #[tokio::test]
    async fn test_edit_replace_all() {
        let test_path = "/tmp/crow_test_edit_all.txt";
        fs::write(test_path, "foo bar\nfoo baz").await.unwrap();

        let tool = EditTool;
        let input = json!({
            "filePath": test_path,
            "oldString": "foo",
            "newString": "bar",
            "replace_all": true
        });

        let result = tool.execute(input).await;
        assert_eq!(result.status, ToolStatus::Completed);

        let output: EditOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.replacements, 2);

        fs::remove_file(test_path).await.unwrap();
    }
}
