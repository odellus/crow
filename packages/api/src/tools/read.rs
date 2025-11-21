//! Read tool - reads file contents
//! Mirrors OpenCode's read tool

use super::ToolContext;
use super::{Tool, ToolResult, ToolStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::fs;

pub struct ReadTool;

#[derive(Deserialize)]
struct ReadInput {
    #[serde(rename = "filePath")]
    file_path: String,
}

#[derive(Serialize, Deserialize)]
struct ReadOutput {
    content: String,
    size: u64,
}

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        r#"Reads a file from the local filesystem. You can access any file directly by using this tool.
Assume this tool is able to read all files on the machine. If the User provides a path to a file assume that path is valid. It is okay to read a file that does not exist; an error will be returned.

Usage:
- The filePath parameter must be an absolute path, not a relative path
- By default, it reads up to 2000 lines starting from the beginning of the file
- You can optionally specify a line offset and limit (especially handy for long files), but it's recommended to read the whole file by not providing these parameters
- Any lines longer than 2000 characters will be truncated
- Results are returned using cat -n format, with line numbers starting at 1
- You have the capability to call multiple tools in a single response. It is always better to speculatively read multiple files as a batch that are potentially useful.
- If you read a file that exists but has empty contents you will receive a system reminder warning in place of file contents.
- You can read image files using this tool."#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "The path to the file to read"
                }
            },
            "required": ["filePath"]
        })
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> ToolResult {
        // Parse input
        let read_input: ReadInput = match serde_json::from_value(input) {
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
        match fs::read_to_string(&read_input.file_path).await {
            Ok(content) => {
                let size = content.len() as u64;
                let output = ReadOutput {
                    content: content.clone(),
                    size,
                };

                ToolResult {
                    status: ToolStatus::Completed,
                    output: serde_json::to_string(&output).unwrap_or_default(),
                    error: None,
                    metadata: json!({
                        "filepath": read_input.file_path,
                        "size": size,
                    }),
                }
            }
            Err(e) => ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!("Failed to read file: {}", e)),
                metadata: json!({
                    "filepath": read_input.file_path,
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
    async fn test_read_file() {
        // Create a test file
        let test_path = "/tmp/crow_test_read.txt";
        fs::write(test_path, "test content").await.unwrap();

        let tool = ReadTool;
        let input = json!({
            "filePath": test_path
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);

        let output: ReadOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.content, "test content");

        // Cleanup
        fs::remove_file(test_path).await.unwrap();
    }

    #[tokio::test]
    async fn test_read_nonexistent() {
        let tool = ReadTool;
        let input = json!({
            "filePath": "/tmp/nonexistent_file_xyz.txt"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }
}
