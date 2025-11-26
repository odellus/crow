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

        let lines_shown = if offset >= all_lines.len() {
            0
        } else {
            all_lines[offset..].iter().take(limit).count()
        };

        ToolResult {
            status: ToolStatus::Completed,
            output: serde_json::to_string(&output).unwrap_or_default(),
            error: None,
            metadata: json!({
                "filepath": read_input.file_path,
                "size": size,
                "total_lines": total_lines,
                "lines_shown": lines_shown,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_context() -> ToolContext {
        ToolContext::new(
            "test-session".to_string(),
            "test-message".to_string(),
            "build".to_string(),
            PathBuf::from("/tmp"),
        )
    }

    // ==================== Tool Interface Tests ====================

    #[tokio::test]
    async fn test_read_tool_name() {
        let tool = ReadTool;
        assert_eq!(tool.name(), "read");
    }

    #[tokio::test]
    async fn test_read_tool_description() {
        let tool = ReadTool;
        let desc = tool.description();
        assert!(desc.contains("file"));
        assert!(desc.contains("read"));
    }

    #[tokio::test]
    async fn test_read_parameters_schema() {
        let tool = ReadTool;
        let schema = tool.parameters_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["filePath"].is_object());
        assert!(schema["properties"]["offset"].is_object());
        assert!(schema["properties"]["limit"].is_object());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("filePath")));
    }

    // ==================== Input Validation Tests ====================

    #[tokio::test]
    async fn test_read_missing_file_path() {
        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool.execute(json!({}), &ctx).await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("Invalid input"));
    }

    #[tokio::test]
    async fn test_read_invalid_input_type() {
        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool.execute(json!("not an object"), &ctx).await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_read_file_not_found() {
        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": "/nonexistent/path/file.txt"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("Failed to read"));
    }

    // ==================== Basic Read Tests ====================

    #[tokio::test]
    async fn test_read_file_with_line_numbers() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "line 1\nline 2\nline 3").unwrap();

        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: ReadOutput = serde_json::from_str(&result.output).unwrap();
        assert!(output.content.contains("     1\tline 1"));
        assert!(output.content.contains("     2\tline 2"));
        assert!(output.content.contains("     3\tline 3"));
    }

    #[tokio::test]
    async fn test_read_single_line_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("single.txt");
        std::fs::write(&file_path, "single line").unwrap();

        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: ReadOutput = serde_json::from_str(&result.output).unwrap();
        assert!(output.content.contains("single line"));
    }

    #[tokio::test]
    async fn test_read_empty_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("empty.txt");
        std::fs::write(&file_path, "").unwrap();

        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("empty contents"));
    }

    // ==================== Offset/Limit Tests ====================

    #[tokio::test]
    async fn test_read_with_offset() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("offset.txt");
        let content = (1..=10)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&file_path, content).unwrap();

        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap(),
                    "offset": 5
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: ReadOutput = serde_json::from_str(&result.output).unwrap();
        assert!(output.content.contains("line 5"));
        assert!(!output.content.contains("line 4"));
    }

    #[tokio::test]
    async fn test_read_with_limit() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("limit.txt");
        let content = (1..=100)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&file_path, content).unwrap();

        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap(),
                    "limit": 5
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: ReadOutput = serde_json::from_str(&result.output).unwrap();
        assert!(output.content.contains("line 1"));
        assert!(output.content.contains("line 5"));
        assert!(!output.content.contains("line 6"));
    }

    #[tokio::test]
    async fn test_read_with_offset_and_limit() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("offset_limit.txt");
        let content = (1..=100)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&file_path, content).unwrap();

        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap(),
                    "offset": 50,
                    "limit": 10
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: ReadOutput = serde_json::from_str(&result.output).unwrap();
        assert!(output.content.contains("    50\tline 50"));
        assert!(output.content.contains("    59\tline 59"));
        assert!(!output.content.contains("    60\tline 60"));
    }

    #[tokio::test]
    async fn test_read_offset_beyond_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("short.txt");
        std::fs::write(&file_path, "line 1\nline 2").unwrap();

        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap(),
                    "offset": 100
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: ReadOutput = serde_json::from_str(&result.output).unwrap();
        assert!(output.content.is_empty());
    }

    // ==================== Truncation Tests ====================

    #[tokio::test]
    async fn test_line_truncation() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("long_line.txt");
        let long_line = "x".repeat(3000);
        std::fs::write(&file_path, &long_line).unwrap();

        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: ReadOutput = serde_json::from_str(&result.output).unwrap();
        // Line should be truncated to MAX_LINE_LENGTH (2000)
        assert!(output.content.len() < long_line.len());
    }

    // ==================== Metadata Tests ====================

    #[tokio::test]
    async fn test_read_returns_metadata() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("meta.txt");
        std::fs::write(&file_path, "line 1\nline 2\nline 3").unwrap();

        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.metadata["filepath"].as_str().is_some());
        assert!(result.metadata["size"].as_u64().is_some());
        assert_eq!(result.metadata["total_lines"], 3);
        assert_eq!(result.metadata["lines_shown"], 3);
    }

    #[tokio::test]
    async fn test_read_metadata_with_limit() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("meta_limit.txt");
        let content = (1..=50)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&file_path, content).unwrap();

        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap(),
                    "limit": 10
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        assert_eq!(result.metadata["total_lines"], 50);
        assert_eq!(result.metadata["lines_shown"], 10);
    }

    // ==================== Content Tests ====================

    #[tokio::test]
    async fn test_read_preserves_whitespace() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("whitespace.txt");
        std::fs::write(&file_path, "  indented\n\ttabbed").unwrap();

        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: ReadOutput = serde_json::from_str(&result.output).unwrap();
        assert!(output.content.contains("  indented"));
        assert!(output.content.contains("\ttabbed"));
    }

    #[tokio::test]
    async fn test_read_unicode_content() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("unicode.txt");
        std::fs::write(&file_path, "Hello 世界 🌍").unwrap();

        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: ReadOutput = serde_json::from_str(&result.output).unwrap();
        assert!(output.content.contains("世界"));
        assert!(output.content.contains("🌍"));
    }

    #[tokio::test]
    async fn test_read_returns_file_size() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("size.txt");
        let content = "exactly 20 bytes!!!";
        std::fs::write(&file_path, content).unwrap();

        let tool = ReadTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let output: ReadOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.size, content.len() as u64);
    }
}
