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
