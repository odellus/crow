//! Patch tool - apply unified diff patches to modify files

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;

use super::{Tool, ToolContext, ToolResult, ToolStatus};

pub struct PatchTool;

#[derive(Deserialize)]
struct PatchInput {
    #[serde(rename = "patchText")]
    patch_text: String,
}

/// Parsed hunk from a unified diff
#[derive(Debug)]
struct DiffHunk {
    file_path: String,
    old_content: Vec<String>,
    new_content: Vec<String>,
    is_new_file: bool,
    is_delete: bool,
}

#[async_trait]
impl Tool for PatchTool {
    fn name(&self) -> &str {
        "patch"
    }

    fn description(&self) -> &str {
        r#"Apply a unified diff patch to modify multiple files at once.

This tool parses and applies patches in unified diff format, supporting:
- Adding new files
- Modifying existing files
- Deleting files

The patch format should follow standard unified diff format with:
- --- a/path/to/file (old file)
- +++ b/path/to/file (new file)
- @@ -start,count +start,count @@ (hunk header)
- Lines starting with - (removed)
- Lines starting with + (added)
- Lines starting with space (context)

Use this for bulk modifications across multiple files."#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "patchText": {
                    "type": "string",
                    "description": "The full patch text in unified diff format"
                }
            },
            "required": ["patchText"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let input: PatchInput = match serde_json::from_value(input) {
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

        if input.patch_text.trim().is_empty() {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some("patchText is required".to_string()),
                metadata: json!({}),
            };
        }

        // Parse the patch
        let hunks = match parse_unified_diff(&input.patch_text) {
            Ok(h) => h,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to parse patch: {}", e)),
                    metadata: json!({}),
                };
            }
        };

        if hunks.is_empty() {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some("No file changes found in patch".to_string()),
                metadata: json!({}),
            };
        }

        // Apply each hunk
        let mut changed_files = Vec::new();
        let mut errors = Vec::new();

        for hunk in hunks {
            let file_path = if PathBuf::from(&hunk.file_path).is_absolute() {
                PathBuf::from(&hunk.file_path)
            } else {
                ctx.working_dir.join(&hunk.file_path)
            };

            let result = apply_hunk(&file_path, &hunk).await;

            match result {
                Ok(_) => {
                    changed_files.push(file_path.display().to_string());
                }
                Err(e) => {
                    errors.push(format!("{}: {}", hunk.file_path, e));
                }
            }
        }

        let total = changed_files.len() + errors.len();
        let success = changed_files.len();

        if errors.is_empty() {
            ToolResult {
                status: ToolStatus::Completed,
                output: format!(
                    "Patch applied successfully. {} files changed:\n{}",
                    success,
                    changed_files
                        .iter()
                        .map(|p| format!("  {}", p))
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
                error: None,
                metadata: json!({
                    "filesChanged": success,
                    "files": changed_files,
                }),
            }
        } else {
            ToolResult {
                status: if success > 0 {
                    ToolStatus::Completed
                } else {
                    ToolStatus::Error
                },
                output: format!(
                    "Patch partially applied. {}/{} files changed.\nErrors:\n{}",
                    success,
                    total,
                    errors
                        .iter()
                        .map(|e| format!("  {}", e))
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
                error: Some(format!("{} files failed", errors.len())),
                metadata: json!({
                    "filesChanged": success,
                    "filesFailed": errors.len(),
                    "files": changed_files,
                    "errors": errors,
                }),
            }
        }
    }
}

/// Parse a unified diff into hunks
fn parse_unified_diff(patch: &str) -> Result<Vec<DiffHunk>, String> {
    let mut hunks = Vec::new();
    let lines: Vec<&str> = patch.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        // Look for --- line
        if lines[i].starts_with("---") {
            let old_file = lines[i].strip_prefix("--- ").unwrap_or("").trim();
            let old_file = old_file.strip_prefix("a/").unwrap_or(old_file);

            // Next should be +++ line
            i += 1;
            if i >= lines.len() || !lines[i].starts_with("+++") {
                return Err("Expected +++ line after ---".to_string());
            }

            let new_file = lines[i].strip_prefix("+++ ").unwrap_or("").trim();
            let new_file = new_file.strip_prefix("b/").unwrap_or(new_file);

            // Determine file path and operation
            let (file_path, is_new_file, is_delete) = if old_file == "/dev/null" {
                (new_file.to_string(), true, false)
            } else if new_file == "/dev/null" {
                (old_file.to_string(), false, true)
            } else {
                (new_file.to_string(), false, false)
            };

            // Parse hunk content
            let mut old_content = Vec::new();
            let mut new_content = Vec::new();

            i += 1;
            while i < lines.len() {
                let line = lines[i];

                if line.starts_with("@@") {
                    // Hunk header, skip it
                    i += 1;
                    continue;
                }

                if line.starts_with("---") || line.starts_with("diff ") {
                    // Next file
                    break;
                }

                if line.starts_with('-') {
                    old_content.push(line[1..].to_string());
                } else if line.starts_with('+') {
                    new_content.push(line[1..].to_string());
                } else if line.starts_with(' ') || line.is_empty() {
                    let content = if line.is_empty() { "" } else { &line[1..] };
                    old_content.push(content.to_string());
                    new_content.push(content.to_string());
                }

                i += 1;
            }

            hunks.push(DiffHunk {
                file_path,
                old_content,
                new_content,
                is_new_file,
                is_delete,
            });
        } else {
            i += 1;
        }
    }

    Ok(hunks)
}

/// Apply a single hunk to a file
async fn apply_hunk(file_path: &PathBuf, hunk: &DiffHunk) -> Result<(), String> {
    if hunk.is_delete {
        // Delete the file
        fs::remove_file(file_path)
            .await
            .map_err(|e| format!("Failed to delete: {}", e))?;
        return Ok(());
    }

    if hunk.is_new_file {
        // Create new file
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let content = hunk.new_content.join("\n");
        fs::write(file_path, content)
            .await
            .map_err(|e| format!("Failed to write: {}", e))?;
        return Ok(());
    }

    // Update existing file
    let current_content = fs::read_to_string(file_path)
        .await
        .map_err(|e| format!("Failed to read: {}", e))?;

    let current_lines: Vec<&str> = current_content.lines().collect();
    let old_lines: Vec<&str> = hunk.old_content.iter().map(|s| s.as_str()).collect();

    // Simple approach: find and replace the old content with new content
    let new_content = if hunk.old_content.is_empty() {
        // Append mode
        let mut result = current_content;
        if !result.ends_with('\n') && !hunk.new_content.is_empty() {
            result.push('\n');
        }
        result.push_str(&hunk.new_content.join("\n"));
        result
    } else {
        // Find the old content in the file and replace
        let old_text = hunk.old_content.join("\n");
        let new_text = hunk.new_content.join("\n");

        if current_content.contains(&old_text) {
            current_content.replacen(&old_text, &new_text, 1)
        } else {
            // Try line-by-line matching with some fuzzing
            match find_and_replace_lines(&current_lines, &old_lines, &hunk.new_content) {
                Some(result) => result,
                None => {
                    return Err("Could not find matching content to replace".to_string());
                }
            }
        }
    };

    fs::write(file_path, new_content)
        .await
        .map_err(|e| format!("Failed to write: {}", e))?;

    Ok(())
}

/// Find old lines in current content and replace with new lines
fn find_and_replace_lines(current: &[&str], old: &[&str], new: &[String]) -> Option<String> {
    if old.is_empty() {
        return None;
    }

    // Find where old content starts
    for i in 0..=current.len().saturating_sub(old.len()) {
        let mut matches = true;
        for (j, old_line) in old.iter().enumerate() {
            if current[i + j].trim() != old_line.trim() {
                matches = false;
                break;
            }
        }

        if matches {
            // Build new content
            let mut result = Vec::new();
            result.extend(current[..i].iter().map(|s| s.to_string()));
            result.extend(new.iter().cloned());
            result.extend(current[i + old.len()..].iter().map(|s| s.to_string()));
            return Some(result.join("\n"));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_context(working_dir: PathBuf) -> ToolContext {
        ToolContext::new(
            "test-session".to_string(),
            "test-message".to_string(),
            "build".to_string(),
            working_dir,
        )
    }

    // ==================== Tool Interface Tests ====================

    #[tokio::test]
    async fn test_patch_tool_name() {
        let tool = PatchTool;
        assert_eq!(tool.name(), "patch");
    }

    #[tokio::test]
    async fn test_patch_tool_description() {
        let tool = PatchTool;
        let desc = tool.description();
        assert!(desc.contains("unified diff"));
        assert!(desc.contains("patch"));
    }

    #[tokio::test]
    async fn test_patch_parameters_schema() {
        let tool = PatchTool;
        let schema = tool.parameters_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["patchText"].is_object());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("patchText")));
    }

    // ==================== Input Validation Tests ====================

    #[tokio::test]
    async fn test_patch_empty_patch_text() {
        let dir = TempDir::new().unwrap();
        let tool = PatchTool;
        let ctx = create_test_context(dir.path().to_path_buf());

        let result = tool
            .execute(
                json!({
                    "patchText": ""
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("patchText is required"));
    }

    #[tokio::test]
    async fn test_patch_whitespace_only() {
        let dir = TempDir::new().unwrap();
        let tool = PatchTool;
        let ctx = create_test_context(dir.path().to_path_buf());

        let result = tool
            .execute(
                json!({
                    "patchText": "   \n\t  "
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("patchText is required"));
    }

    #[tokio::test]
    async fn test_patch_invalid_input() {
        let dir = TempDir::new().unwrap();
        let tool = PatchTool;
        let ctx = create_test_context(dir.path().to_path_buf());

        let result = tool.execute(json!({"wrong_field": "value"}), &ctx).await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("Invalid input"));
    }

    #[tokio::test]
    async fn test_patch_no_hunks_found() {
        let dir = TempDir::new().unwrap();
        let tool = PatchTool;
        let ctx = create_test_context(dir.path().to_path_buf());

        let result = tool
            .execute(
                json!({
                    "patchText": "This is not a valid patch format"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("No file changes found"));
    }

    // ==================== Diff Parsing Tests ====================

    #[test]
    fn test_parse_unified_diff_new_file() {
        let patch = r#"--- /dev/null
+++ b/new_file.txt
@@ -0,0 +1,3 @@
+line 1
+line 2
+line 3"#;

        let hunks = parse_unified_diff(patch).unwrap();
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].file_path, "new_file.txt");
        assert!(hunks[0].is_new_file);
        assert!(!hunks[0].is_delete);
        assert_eq!(hunks[0].new_content, vec!["line 1", "line 2", "line 3"]);
    }

    #[test]
    fn test_parse_unified_diff_delete_file() {
        let patch = r#"--- a/old_file.txt
+++ /dev/null
@@ -1,3 +0,0 @@
-line 1
-line 2
-line 3"#;

        let hunks = parse_unified_diff(patch).unwrap();
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].file_path, "old_file.txt");
        assert!(!hunks[0].is_new_file);
        assert!(hunks[0].is_delete);
        assert_eq!(hunks[0].old_content, vec!["line 1", "line 2", "line 3"]);
    }

    #[test]
    fn test_parse_unified_diff_modify_file() {
        let patch = r#"--- a/file.txt
+++ b/file.txt
@@ -1,3 +1,3 @@
 context line
-old line
+new line
 another context"#;

        let hunks = parse_unified_diff(patch).unwrap();
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].file_path, "file.txt");
        assert!(!hunks[0].is_new_file);
        assert!(!hunks[0].is_delete);
        assert!(hunks[0].old_content.contains(&"old line".to_string()));
        assert!(hunks[0].new_content.contains(&"new line".to_string()));
    }

    #[test]
    fn test_parse_unified_diff_multiple_files() {
        let patch = r#"--- a/file1.txt
+++ b/file1.txt
@@ -1 +1 @@
-old
+new
--- a/file2.txt
+++ b/file2.txt
@@ -1 +1 @@
-foo
+bar"#;

        let hunks = parse_unified_diff(patch).unwrap();
        assert_eq!(hunks.len(), 2);
        assert_eq!(hunks[0].file_path, "file1.txt");
        assert_eq!(hunks[1].file_path, "file2.txt");
    }

    #[test]
    fn test_parse_unified_diff_strips_a_b_prefixes() {
        let patch = r#"--- a/src/main.rs
+++ b/src/main.rs
@@ -1 +1 @@
-old
+new"#;

        let hunks = parse_unified_diff(patch).unwrap();
        assert_eq!(hunks[0].file_path, "src/main.rs");
    }

    // ==================== Apply Patch Tests ====================

    #[tokio::test]
    async fn test_patch_create_new_file() {
        let dir = TempDir::new().unwrap();
        let tool = PatchTool;
        let ctx = create_test_context(dir.path().to_path_buf());

        let patch = r#"--- /dev/null
+++ b/newfile.txt
@@ -0,0 +1,2 @@
+Hello
+World"#;

        let result = tool
            .execute(
                json!({
                    "patchText": patch
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let file_path = dir.path().join("newfile.txt");
        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("Hello"));
        assert!(content.contains("World"));
    }

    #[tokio::test]
    async fn test_patch_delete_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("to_delete.txt");
        std::fs::write(&file_path, "content").unwrap();
        assert!(file_path.exists());

        let tool = PatchTool;
        let ctx = create_test_context(dir.path().to_path_buf());

        let patch = r#"--- a/to_delete.txt
+++ /dev/null
@@ -1 +0,0 @@
-content"#;

        let result = tool
            .execute(
                json!({
                    "patchText": patch
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_patch_modify_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("modify.txt");
        std::fs::write(&file_path, "line 1\nold line\nline 3").unwrap();

        let tool = PatchTool;
        let ctx = create_test_context(dir.path().to_path_buf());

        let patch = r#"--- a/modify.txt
+++ b/modify.txt
@@ -1,3 +1,3 @@
 line 1
-old line
+new line
 line 3"#;

        let result = tool
            .execute(
                json!({
                    "patchText": patch
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("new line"));
        assert!(!content.contains("old line"));
    }

    #[tokio::test]
    async fn test_patch_creates_parent_directories() {
        let dir = TempDir::new().unwrap();
        let tool = PatchTool;
        let ctx = create_test_context(dir.path().to_path_buf());

        let patch = r#"--- /dev/null
+++ b/deep/nested/dir/file.txt
@@ -0,0 +1 @@
+content"#;

        let result = tool
            .execute(
                json!({
                    "patchText": patch
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        let file_path = dir.path().join("deep/nested/dir/file.txt");
        assert!(file_path.exists());
    }

    #[tokio::test]
    async fn test_patch_returns_metadata() {
        let dir = TempDir::new().unwrap();
        let tool = PatchTool;
        let ctx = create_test_context(dir.path().to_path_buf());

        let patch = r#"--- /dev/null
+++ b/file.txt
@@ -0,0 +1 @@
+content"#;

        let result = tool
            .execute(
                json!({
                    "patchText": patch
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        assert_eq!(result.metadata["filesChanged"], 1);
        assert!(result.metadata["files"].as_array().is_some());
    }

    #[tokio::test]
    async fn test_patch_file_not_found_for_modify() {
        let dir = TempDir::new().unwrap();
        let tool = PatchTool;
        let ctx = create_test_context(dir.path().to_path_buf());

        let patch = r#"--- a/nonexistent.txt
+++ b/nonexistent.txt
@@ -1 +1 @@
-old
+new"#;

        let result = tool
            .execute(
                json!({
                    "patchText": patch
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.metadata["filesFailed"].as_i64().unwrap() > 0);
    }

    // ==================== Line Matching Tests ====================

    #[test]
    fn test_find_and_replace_lines_exact_match() {
        let current = vec!["line 1", "old", "line 3"];
        let old = vec!["old"];
        let new = vec!["new".to_string()];

        let result = find_and_replace_lines(&current, &old, &new);
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.contains("new"));
        assert!(!result.contains("old"));
    }

    #[test]
    fn test_find_and_replace_lines_multiple_lines() {
        let current = vec!["a", "b", "c", "d"];
        let old = vec!["b", "c"];
        let new = vec!["x".to_string(), "y".to_string(), "z".to_string()];

        let result = find_and_replace_lines(&current, &old, &new);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result, "a\nx\ny\nz\nd");
    }

    #[test]
    fn test_find_and_replace_lines_not_found() {
        let current = vec!["a", "b", "c"];
        let old = vec!["x", "y"];
        let new = vec!["z".to_string()];

        let result = find_and_replace_lines(&current, &old, &new);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_and_replace_lines_empty_old() {
        let current = vec!["a", "b"];
        let old: Vec<&str> = vec![];
        let new = vec!["c".to_string()];

        let result = find_and_replace_lines(&current, &old, &new);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_and_replace_lines_whitespace_tolerance() {
        let current = vec!["  line 1  ", "old", "line 3"];
        let old = vec!["line 1"];
        let new = vec!["replaced".to_string()];

        // Should match trimmed content
        let result = find_and_replace_lines(&current, &old, &new);
        assert!(result.is_some());
    }
}
