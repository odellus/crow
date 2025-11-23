//! Read tool - reads file contents with line numbering
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
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize)]
struct ReadOutput {
    content: String,
    size: u64,
}

const DEFAULT_LINE_LIMIT: usize = 2000;
const MAX_LINE_LENGTH: usize = 2000;

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
                },
                "offset": {
                    "type": "integer",
                    "description": "Line number to start reading from (1-indexed, optional)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to read (optional, default 2000)"
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
        let content = match fs::read_to_string(&read_input.file_path).await {
            Ok(c) => c,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to read file: {}", e)),
                    metadata: json!({
                        "filepath": read_input.file_path,
                    }),
                };
            }
        };

        // Check if empty
        if content.is_empty() {
            return ToolResult {
                status: ToolStatus::Completed,
                output: String::new(),
                error: Some("Warning: File exists but has empty contents".to_string()),
                metadata: json!({
                    "filepath": read_input.file_path,
                    "size": 0,
                }),
            };
        }

        // Split into lines
        let all_lines: Vec<&str> = content.lines().collect();
        let total_lines = all_lines.len();

        // Apply offset and limit
        let offset = read_input.offset.unwrap_or(1).saturating_sub(1); // Convert to 0-indexed
        let limit = read_input.limit.unwrap_or(DEFAULT_LINE_LIMIT);

        let lines_to_show = all_lines
            .iter()
            .skip(offset)
            .take(limit)
            .enumerate()
            .map(|(idx, line)| {
                let line_number = offset + idx + 1; // Convert back to 1-indexed
                let truncated = if line.len() > MAX_LINE_LENGTH {
                    &line[..MAX_LINE_LENGTH]
                } else {
                    line
                };
                format!("{:6}\t{}", line_number, truncated)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let size = content.len() as u64;
        let output = ReadOutput {
            content: lines_to_show,
            size,
        };

        ToolResult {
            status: ToolStatus::Completed,
            output: serde_json::to_string(&output).unwrap_or_default(),
            error: None,
            metadata: json!({
                "filepath": read_input.file_path,
                "size": size,
                "total_lines": total_lines,
                "lines_shown": all_lines[offset..].iter().take(limit).count(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;

    #[tokio::test]
    async fn test_read_file_with_line_numbers() {
        let test_path = "/tmp/crow_test_read.txt";
        fs::write(test_path, "line 1\nline 2\nline 3")
            .await
            .unwrap();

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
        assert!(output.content.contains("     1\tline 1"));
        assert!(output.content.contains("     2\tline 2"));

        fs::remove_file(test_path).await.unwrap();
    }

    #[tokio::test]
    async fn test_read_with_offset_limit() {
        let test_path = "/tmp/crow_test_read_offset.txt";
        let content = (1..=100)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(test_path, content).await.unwrap();

        let tool = ReadTool;
        let input = json!({
            "filePath": test_path,
            "offset": 50,
            "limit": 10
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
        assert!(output.content.contains("    50\tline 50"));
        assert!(output.content.contains("    59\tline 59"));
        assert!(!output.content.contains("    60\tline 60"));

        fs::remove_file(test_path).await.unwrap();
    }

    #[tokio::test]
    async fn test_read_empty_file() {
        let test_path = "/tmp/crow_test_read_empty.txt";
        fs::write(test_path, "").await.unwrap();

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
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("empty contents"));

        fs::remove_file(test_path).await.unwrap();
    }

    #[tokio::test]
    async fn test_line_truncation() {
        let test_path = "/tmp/crow_test_read_long.txt";
        let long_line = "x".repeat(3000);
        fs::write(test_path, &long_line).await.unwrap();

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
        // Line should be truncated to 2000 chars (plus line number prefix)
        assert!(output.content.len() < long_line.len());

        fs::remove_file(test_path).await.unwrap();
    }
}
