//! Write tool - writes file contents
//! Mirrors OpenCode's write tool

use super::{Tool, ToolResult, ToolStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::fs;

pub struct WriteTool;

#[derive(Deserialize)]
struct WriteInput {
    #[serde(rename = "filePath")]
    file_path: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct WriteOutput {
    filepath: String,
    bytes_written: usize,
    existed: bool,
}

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        r#"Writes a file to the local filesystem.

Usage:
- This tool will overwrite the existing file if there is one at the provided path.
- If this is an existing file, you MUST use the Read tool first to read the file's contents. This tool will fail if you did not read the file first.
- ALWAYS prefer editing existing files in the codebase. NEVER write new files unless explicitly required.
- NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.
- Only use emojis if the user explicitly requests it. Avoid writing emojis to files unless asked."#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "The path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                }
            },
            "required": ["filePath", "content"]
        })
    }

    async fn execute(&self, input: Value) -> ToolResult {
        // Parse input
        let write_input: WriteInput = match serde_json::from_value(input) {
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

        // Check if file exists
        let existed = fs::metadata(&write_input.file_path).await.is_ok();

        // Create parent directories if needed
        if let Some(parent) = std::path::Path::new(&write_input.file_path).parent() {
            if let Err(e) = fs::create_dir_all(parent).await {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to create parent directory: {}", e)),
                    metadata: json!({
                        "filepath": write_input.file_path,
                    }),
                };
            }
        }

        // Write file
        match fs::write(&write_input.file_path, &write_input.content).await {
            Ok(_) => {
                let bytes_written = write_input.content.len();
                let output = WriteOutput {
                    filepath: write_input.file_path.clone(),
                    bytes_written,
                    existed,
                };

                ToolResult {
                    status: ToolStatus::Completed,
                    output: serde_json::to_string(&output).unwrap_or_default(),
                    error: None,
                    metadata: json!({
                        "filepath": write_input.file_path,
                        "bytes_written": bytes_written,
                        "existed": existed,
                    }),
                }
            }
            Err(e) => ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!("Failed to write file: {}", e)),
                metadata: json!({
                    "filepath": write_input.file_path,
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;

    #[tokio::test]
    async fn test_write_new_file() {
        let test_path = "/tmp/crow_test_write_new.txt";

        // Make sure file doesn't exist
        let _ = fs::remove_file(test_path).await;

        let tool = WriteTool;
        let input = json!({
            "filePath": test_path,
            "content": "hello world"
        });

        let result = tool.execute(input).await;
        assert_eq!(result.status, ToolStatus::Completed);

        let output: WriteOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.bytes_written, 11);
        assert_eq!(output.existed, false);

        // Verify file was created
        let content = fs::read_to_string(test_path).await.unwrap();
        assert_eq!(content, "hello world");

        // Cleanup
        fs::remove_file(test_path).await.unwrap();
    }

    #[tokio::test]
    async fn test_write_overwrite_file() {
        let test_path = "/tmp/crow_test_write_overwrite.txt";

        // Create existing file
        fs::write(test_path, "old content").await.unwrap();

        let tool = WriteTool;
        let input = json!({
            "filePath": test_path,
            "content": "new content"
        });

        let result = tool.execute(input).await;
        assert_eq!(result.status, ToolStatus::Completed);

        let output: WriteOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.existed, true);

        // Verify file was overwritten
        let content = fs::read_to_string(test_path).await.unwrap();
        assert_eq!(content, "new content");

        // Cleanup
        fs::remove_file(test_path).await.unwrap();
    }
}
