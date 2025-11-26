//! Write tool - writes file contents
//! Mirrors OpenCode's write tool

use super::ToolContext;
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

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> ToolResult {
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
    async fn test_write_tool_name() {
        let tool = WriteTool;
        assert_eq!(tool.name(), "write");
    }

    #[tokio::test]
    async fn test_write_tool_description() {
        let tool = WriteTool;
        let desc = tool.description();
        assert!(desc.contains("file"));
        assert!(desc.contains("write"));
    }

    #[tokio::test]
    async fn test_write_parameters_schema() {
        let tool = WriteTool;
        let schema = tool.parameters_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["filePath"].is_object());
        assert!(schema["properties"]["content"].is_object());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("filePath")));
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("content")));
    }

    // ==================== Input Validation Tests ====================

    #[tokio::test]
    async fn test_write_missing_file_path() {
        let tool = WriteTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "content": "some content"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("Invalid input"));
    }

    #[tokio::test]
    async fn test_write_missing_content() {
        let tool = WriteTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": "/tmp/test.txt"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("Invalid input"));
    }

    #[tokio::test]
    async fn test_write_invalid_input_type() {
        let tool = WriteTool;
        let ctx = create_test_context();

        let result = tool.execute(json!("not an object"), &ctx).await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    // ==================== Basic Write Tests ====================

    #[tokio::test]
    async fn test_write_new_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("new_file.txt");

        let tool = WriteTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap(),
                    "content": "hello world"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: WriteOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.bytes_written, 11);
        assert!(!output.existed);

        // Verify file was created
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn test_write_overwrite_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("existing.txt");
        std::fs::write(&file_path, "old content").unwrap();

        let tool = WriteTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap(),
                    "content": "new content"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: WriteOutput = serde_json::from_str(&result.output).unwrap();
        assert!(output.existed);

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "new content");
    }

    #[tokio::test]
    async fn test_write_empty_content() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("empty.txt");

        let tool = WriteTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap(),
                    "content": ""
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: WriteOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.bytes_written, 0);

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.is_empty());
    }

    #[tokio::test]
    async fn test_write_creates_parent_directories() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("deep/nested/dir/file.txt");

        let tool = WriteTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap(),
                    "content": "nested content"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        assert!(file_path.exists());

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "nested content");
    }

    // ==================== Content Tests ====================

    #[tokio::test]
    async fn test_write_unicode_content() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("unicode.txt");

        let tool = WriteTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap(),
                    "content": "Hello 世界 🌍"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("世界"));
        assert!(content.contains("🌍"));
    }

    #[tokio::test]
    async fn test_write_multiline_content() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("multiline.txt");

        let tool = WriteTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap(),
                    "content": "line 1\nline 2\nline 3"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content.lines().count(), 3);
    }

    #[tokio::test]
    async fn test_write_preserves_whitespace() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("whitespace.txt");

        let tool = WriteTool;
        let ctx = create_test_context();

        let whitespace_content = "  indented\n\ttabbed\n\n\nspaced";
        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap(),
                    "content": whitespace_content
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, whitespace_content);
    }

    // ==================== Metadata Tests ====================

    #[tokio::test]
    async fn test_write_returns_metadata() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("meta.txt");

        let tool = WriteTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap(),
                    "content": "test content"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.metadata["filepath"].as_str().is_some());
        assert!(result.metadata["bytes_written"].as_u64().is_some());
        assert!(result.metadata["existed"].as_bool().is_some());
    }

    #[tokio::test]
    async fn test_write_bytes_written_accurate() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("bytes.txt");

        let tool = WriteTool;
        let ctx = create_test_context();

        let content = "exactly 20 bytes!!!";
        let result = tool
            .execute(
                json!({
                    "filePath": file_path.to_str().unwrap(),
                    "content": content
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: WriteOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.bytes_written, content.len());
    }

    // ==================== Error Handling Tests ====================

    #[tokio::test]
    async fn test_write_invalid_path() {
        let tool = WriteTool;
        let ctx = create_test_context();

        // Try to write to a path that can't be created (root-level on most systems)
        let result = tool
            .execute(
                json!({
                    "filePath": "/nonexistent_root_dir_xyz/file.txt",
                    "content": "test"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
    }
}
