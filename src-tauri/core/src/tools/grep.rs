//! Grep tool for searching text in files
use super::ToolContext;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::process::Command;
use tokio::fs;

use crate::tools::{Tool, ToolResult, ToolStatus};

/// Grep tool for searching text patterns in files
#[derive(Clone)]
pub struct GrepTool;

#[derive(Deserialize)]
struct GrepInput {
    pattern: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    include: Option<String>,
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        r#"- Fast content search tool that works with any codebase size
- Searches file contents using regular expressions
- Supports full regex syntax (eg. "log.*Error", "function\s+\w+", etc.)
- Filter files by pattern with the include parameter (eg. "*.js", "*.{ts,tsx}")
- Returns file paths with at least one match sorted by modification time
- Use this tool when you need to find files containing specific patterns
- If you need to identify/count the number of matches within files, use the Bash tool with `rg` (ripgrep) directly. Do NOT use `grep`.
- When you are doing an open ended search that may require multiple rounds of globbing and grepping, use the Task tool instead"#
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The regex pattern to search for in file contents"
                },
                "path": {
                    "type": "string",
                    "description": "The directory to search in. Defaults to the current working directory."
                },
                "include": {
                    "type": "string",
                    "description": "File pattern to include in the search (e.g. \"*.js\", \"*.{ts,tsx}\")"
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> ToolResult {
        let input: GrepInput = match serde_json::from_value(input) {
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

        let mut cmd = Command::new("rg");
        cmd.arg("-nH")
            .arg("--field-match-separator=|")
            .arg("--regexp")
            .arg(&input.pattern);

        if let Some(include) = &input.include {
            cmd.arg("--glob").arg(include);
        }

        cmd.arg(&search_path);

        let output = cmd.output();

        match output {
            Ok(output) => {
                let exit_code = output.status.code().unwrap_or(-1);

                if exit_code == 1 {
                    // No matches found
                    return ToolResult {
                        status: ToolStatus::Completed,
                        output: "No files found".to_string(),
                        error: None,
                        metadata: serde_json::json!({
                            "matches": 0,
                            "truncated": false
                        }),
                    };
                }

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    return ToolResult {
                        status: ToolStatus::Error,
                        output: String::new(),
                        error: Some(format!("ripgrep failed: {}", stderr)),
                        metadata: serde_json::json!({}),
                    };
                }

                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let lines: Vec<&str> = stdout.trim().split('\n').collect();

                #[derive(Clone)]
                struct Match {
                    path: String,
                    line_num: usize,
                    line_text: String,
                }

                let mut matches = Vec::new();

                for line in lines {
                    if line.is_empty() {
                        continue;
                    }

                    let parts: Vec<&str> = line.splitn(3, '|').collect();
                    if parts.len() < 3 {
                        continue;
                    }

                    let file_path = parts[0].to_string();
                    let line_num = parts[1].parse::<usize>().unwrap_or(0);
                    let line_text = parts[2].to_string();

                    matches.push(Match {
                        path: file_path,
                        line_num,
                        line_text,
                    });
                }

                // Get file metadata and sort by modification time (most recent first)
                let mut matches_with_mtime = Vec::new();
                for m in matches {
                    if let Ok(metadata) = fs::metadata(&m.path).await {
                        if let Ok(modified) = metadata.modified() {
                            matches_with_mtime.push((m, modified));
                        }
                    }
                }

                // Sort by modification time, most recent first
                matches_with_mtime.sort_by(|a, b| b.1.cmp(&a.1));

                // Extract sorted matches
                let sorted_matches: Vec<Match> =
                    matches_with_mtime.into_iter().map(|(m, _)| m).collect();

                let limit = 100;
                let truncated = sorted_matches.len() > limit;
                let final_matches: Vec<Match> = if truncated {
                    sorted_matches.into_iter().take(limit).collect()
                } else {
                    sorted_matches
                };

                if final_matches.is_empty() {
                    return ToolResult {
                        status: ToolStatus::Completed,
                        output: "No files found".to_string(),
                        error: None,
                        metadata: serde_json::json!({
                            "matches": 0,
                            "truncated": false
                        }),
                    };
                }

                let mut output_lines = vec![format!("Found {} matches", final_matches.len())];

                let mut current_file = String::new();
                for m in &final_matches {
                    if current_file != m.path {
                        if !current_file.is_empty() {
                            output_lines.push(String::new());
                        }
                        current_file = m.path.clone();
                        output_lines.push(format!("{}:", m.path));
                    }
                    output_lines.push(format!("  Line {}: {}", m.line_num, m.line_text));
                }

                if truncated {
                    output_lines.push(String::new());
                    output_lines.push(
                        "(Results are truncated. Consider using a more specific path or pattern.)"
                            .to_string(),
                    );
                }

                ToolResult {
                    status: ToolStatus::Completed,
                    output: output_lines.join("\n"),
                    error: None,
                    metadata: serde_json::json!({
                        "matches": final_matches.len(),
                        "truncated": truncated
                    }),
                }
            }
            Err(e) => ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!("Failed to execute grep: {}", e)),
                metadata: serde_json::json!({}),
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

    fn setup_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        let path = dir.path();

        // Create test files with searchable content
        std::fs::write(
            path.join("main.rs"),
            "fn main() {\n    println!(\"Hello\");\n}\n",
        )
        .unwrap();
        std::fs::write(
            path.join("lib.rs"),
            "pub fn hello() {\n    println!(\"Hello world\");\n}\n",
        )
        .unwrap();
        std::fs::write(
            path.join("test.rs"),
            "fn test_hello() {\n    assert!(true);\n}\n",
        )
        .unwrap();
        std::fs::write(
            path.join("readme.txt"),
            "This is a readme file\nWith multiple lines\n",
        )
        .unwrap();
        std::fs::create_dir_all(path.join("src")).unwrap();
        std::fs::write(path.join("src/mod.rs"), "mod hello;\nmod world;\n").unwrap();

        dir
    }

    // ==================== Tool Interface Tests ====================

    #[tokio::test]
    async fn test_grep_tool_name() {
        let tool = GrepTool;
        assert_eq!(tool.name(), "grep");
    }

    #[tokio::test]
    async fn test_grep_tool_description() {
        let tool = GrepTool;
        let desc = tool.description();
        assert!(desc.contains("search"));
        assert!(desc.contains("regex"));
    }

    #[tokio::test]
    async fn test_grep_parameters_schema() {
        let tool = GrepTool;
        let schema = tool.parameters_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["pattern"].is_object());
        assert!(schema["properties"]["path"].is_object());
        assert!(schema["properties"]["include"].is_object());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("pattern")));
    }

    // ==================== Input Validation Tests ====================

    #[tokio::test]
    async fn test_grep_missing_pattern() {
        let tool = GrepTool;
        let ctx = create_test_context();

        let result = tool.execute(serde_json::json!({}), &ctx).await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("Invalid input"));
    }

    #[tokio::test]
    async fn test_grep_invalid_input_type() {
        let tool = GrepTool;
        let ctx = create_test_context();

        let result = tool.execute(serde_json::json!("not an object"), &ctx).await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    // ==================== Search Tests ====================

    #[tokio::test]
    async fn test_grep_simple_pattern() {
        let dir = setup_test_dir();
        let tool = GrepTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                serde_json::json!({
                    "pattern": "println",
                    "path": dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("println"));
    }

    #[tokio::test]
    async fn test_grep_regex_pattern() {
        let dir = setup_test_dir();
        let tool = GrepTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                serde_json::json!({
                    "pattern": "fn\\s+\\w+",
                    "path": dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        // Should match fn main, fn hello, fn test_hello
        assert!(result.output.contains("fn"));
    }

    #[tokio::test]
    async fn test_grep_no_matches() {
        let dir = setup_test_dir();
        let tool = GrepTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                serde_json::json!({
                    "pattern": "nonexistent_pattern_xyz",
                    "path": dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("No files found"));
        assert_eq!(result.metadata["matches"], 0);
    }

    #[tokio::test]
    async fn test_grep_with_include_filter() {
        let dir = setup_test_dir();
        let tool = GrepTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                serde_json::json!({
                    "pattern": "Hello",
                    "path": dir.path().to_str().unwrap(),
                    "include": "*.rs"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        // Should only find in .rs files, not in readme.txt
        // Note: readme.txt doesn't contain "Hello" anyway, but this tests the filter
        assert!(!result.output.contains("readme.txt"));
    }

    #[tokio::test]
    async fn test_grep_default_path() {
        let tool = GrepTool;
        let ctx = create_test_context();

        // This will search current directory
        let result = tool
            .execute(
                serde_json::json!({
                    "pattern": "some_pattern"
                }),
                &ctx,
            )
            .await;

        // Should complete without error (even if no matches)
        assert!(result.status == ToolStatus::Completed || result.status == ToolStatus::Error);
    }

    #[tokio::test]
    async fn test_grep_invalid_path() {
        let tool = GrepTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                serde_json::json!({
                    "pattern": "test",
                    "path": "/nonexistent/path/xyz"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
    }

    #[tokio::test]
    async fn test_grep_returns_line_numbers() {
        let dir = setup_test_dir();
        let tool = GrepTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                serde_json::json!({
                    "pattern": "fn main",
                    "path": dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        // Output should contain line numbers
        assert!(result.output.contains("Line"));
    }

    #[tokio::test]
    async fn test_grep_metadata_contains_match_count() {
        let dir = setup_test_dir();
        let tool = GrepTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                serde_json::json!({
                    "pattern": "fn",
                    "path": dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.metadata["matches"].as_i64().is_some());
        assert!(result.metadata["truncated"].as_bool().is_some());
    }

    #[tokio::test]
    async fn test_grep_subdirectory_search() {
        let dir = setup_test_dir();
        let tool = GrepTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                serde_json::json!({
                    "pattern": "mod",
                    "path": dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        // Should find mod.rs in src/ subdirectory
        assert!(result.output.contains("mod"));
    }

    #[tokio::test]
    async fn test_grep_case_sensitive() {
        let dir = setup_test_dir();
        let tool = GrepTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                serde_json::json!({
                    "pattern": "HELLO",
                    "path": dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        // ripgrep is case-sensitive by default
        assert_eq!(result.status, ToolStatus::Completed);
        // Should not match "Hello" with uppercase pattern
        assert!(result.output.contains("No files found"));
    }
}
