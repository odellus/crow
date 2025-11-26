//! List tool - displays files and directories
//! Accepts glob patterns to filter results

use super::ToolContext;
use super::{Tool, ToolResult, ToolStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use tokio::fs;

pub struct ListTool;

#[derive(Deserialize)]
struct ListInput {
    path: String,
    #[serde(default)]
    pattern: Option<String>,
    #[serde(default)]
    recursive: bool,
}

#[derive(Serialize, Deserialize)]
struct ListOutput {
    path: String,
    entries: Vec<FileEntry>,
    count: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
}

#[async_trait]
impl Tool for ListTool {
    fn name(&self) -> &str {
        "list"
    }

    fn description(&self) -> &str {
        "Display files and directories in a given path. Accepts glob patterns to filter results (e.g., '*.rs', 'src/**/*.ts')."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to list (defaults to current directory)"
                },
                "pattern": {
                    "type": "string",
                    "description": "Optional glob pattern to filter results (e.g., '*.rs')"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "List files recursively (default: false)"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> ToolResult {
        let list_input: ListInput = match serde_json::from_value(input) {
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

        let path = Path::new(&list_input.path);

        // Check if path exists
        if !path.exists() {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!("Path does not exist: {}", list_input.path)),
                metadata: json!({
                    "path": list_input.path,
                }),
            };
        }

        let mut entries = Vec::new();

        // List directory
        if list_input.recursive {
            if let Err(e) = collect_entries_recursive(path, &list_input.pattern, &mut entries).await
            {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to list directory: {}", e)),
                    metadata: json!({
                        "path": list_input.path,
                    }),
                };
            }
        } else {
            if let Err(e) = collect_entries(path, &list_input.pattern, &mut entries).await {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to list directory: {}", e)),
                    metadata: json!({
                        "path": list_input.path,
                    }),
                };
            }
        }

        // Sort entries: directories first, then by name
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        let output = ListOutput {
            path: list_input.path.clone(),
            entries: entries.clone(),
            count: entries.len(),
        };

        ToolResult {
            status: ToolStatus::Completed,
            output: serde_json::to_string(&output).unwrap_or_default(),
            error: None,
            metadata: json!({
                "path": list_input.path,
                "count": entries.len(),
            }),
        }
    }
}

async fn collect_entries(
    path: &Path,
    pattern: &Option<String>,
    entries: &mut Vec<FileEntry>,
) -> Result<(), String> {
    let mut dir = fs::read_dir(path)
        .await
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read entry: {}", e))?
    {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files
        if name.starts_with('.') {
            continue;
        }

        // Apply pattern filter
        if let Some(pat) = pattern {
            if !matches_pattern(&name, pat) {
                continue;
            }
        }

        let metadata = entry.metadata().await.ok();
        let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = if !is_dir {
            metadata.as_ref().map(|m| m.len())
        } else {
            None
        };

        entries.push(FileEntry {
            name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            size,
        });
    }

    Ok(())
}

fn collect_entries_recursive<'a>(
    path: &'a Path,
    pattern: &'a Option<String>,
    entries: &'a mut Vec<FileEntry>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + 'a>> {
    Box::pin(async move {
        // Collect entries at this level into a temporary vec
        let mut local_entries = Vec::new();
        collect_entries(path, pattern, &mut local_entries).await?;

        // Get directories to recurse into before adding to main entries
        let dirs: Vec<_> = local_entries
            .iter()
            .filter(|e| e.is_dir)
            .map(|e| e.path.clone())
            .collect();

        // Add local entries to main collection
        entries.extend(local_entries);

        // Recurse into subdirectories
        for dir_path in dirs {
            let sub_path = Path::new(&dir_path);
            collect_entries_recursive(sub_path, pattern, entries).await?;
        }

        Ok(())
    })
}

fn matches_pattern(name: &str, pattern: &str) -> bool {
    // Simple glob matching (*, ?)
    let re_pattern = pattern
        .replace(".", "\\.")
        .replace("*", ".*")
        .replace("?", ".");

    if let Ok(re) = regex::Regex::new(&format!("^{}$", re_pattern)) {
        re.is_match(name)
    } else {
        // Fallback to simple contains
        name.contains(pattern)
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

        // Create test file structure
        std::fs::write(path.join("main.rs"), "fn main() {}").unwrap();
        std::fs::write(path.join("lib.rs"), "pub fn lib() {}").unwrap();
        std::fs::write(path.join("README.md"), "# Test").unwrap();
        std::fs::write(path.join("config.json"), "{}").unwrap();
        std::fs::create_dir_all(path.join("src")).unwrap();
        std::fs::write(path.join("src/mod.rs"), "mod test;").unwrap();
        std::fs::create_dir_all(path.join("tests")).unwrap();
        std::fs::write(path.join("tests/test1.rs"), "").unwrap();

        dir
    }

    // ==================== Tool Interface Tests ====================

    #[tokio::test]
    async fn test_list_tool_name() {
        let tool = ListTool;
        assert_eq!(tool.name(), "list");
    }

    #[tokio::test]
    async fn test_list_tool_description() {
        let tool = ListTool;
        let desc = tool.description();
        assert!(desc.contains("files") || desc.contains("directories"));
    }

    #[tokio::test]
    async fn test_list_parameters_schema() {
        let tool = ListTool;
        let schema = tool.parameters_schema();

        assert_eq!(schema["type"], "object");
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("path")));
    }

    // ==================== Input Validation Tests ====================

    #[tokio::test]
    async fn test_list_missing_path() {
        let tool = ListTool;
        let ctx = create_test_context();

        let result = tool.execute(json!({}), &ctx).await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_list_invalid_path() {
        let tool = ListTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "path": "/nonexistent/path/xyz"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("does not exist"));
    }

    // ==================== Basic Functionality Tests ====================

    #[tokio::test]
    async fn test_list_basic_directory() {
        let dir = setup_test_dir();
        let tool = ListTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "path": dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: ListOutput = serde_json::from_str(&result.output).unwrap();
        assert!(output.count > 0);

        // Should find our test files
        let names: Vec<&str> = output.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"main.rs"));
        assert!(names.contains(&"README.md"));
    }

    #[tokio::test]
    async fn test_list_empty_directory() {
        let dir = TempDir::new().unwrap();
        let tool = ListTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "path": dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: ListOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.count, 0);
    }

    #[tokio::test]
    async fn test_list_with_pattern() {
        let dir = setup_test_dir();
        let tool = ListTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "path": dir.path().to_str().unwrap(),
                    "pattern": "*.rs"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: ListOutput = serde_json::from_str(&result.output).unwrap();

        // Should only have .rs files
        for entry in &output.entries {
            if !entry.is_dir {
                assert!(
                    entry.name.ends_with(".rs"),
                    "Expected .rs file, got {}",
                    entry.name
                );
            }
        }
    }

    #[tokio::test]
    async fn test_list_recursive() {
        let dir = setup_test_dir();
        let tool = ListTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "path": dir.path().to_str().unwrap(),
                    "recursive": true
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: ListOutput = serde_json::from_str(&result.output).unwrap();

        // Should find files in subdirectories
        let paths: Vec<&str> = output.entries.iter().map(|e| e.path.as_str()).collect();
        let has_nested = paths
            .iter()
            .any(|p| p.contains("src") || p.contains("tests"));
        assert!(has_nested, "Should find files in subdirectories");
    }

    #[tokio::test]
    async fn test_list_non_recursive() {
        let dir = setup_test_dir();
        let tool = ListTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "path": dir.path().to_str().unwrap(),
                    "recursive": false
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: ListOutput = serde_json::from_str(&result.output).unwrap();

        // Should NOT have files from subdirectories (only dirs themselves)
        for entry in &output.entries {
            if !entry.is_dir {
                // Non-directory entries should be at top level
                let path_parts: Vec<&str> = entry.path.split('/').collect();
                let name = path_parts.last().unwrap();
                assert_eq!(*name, entry.name);
            }
        }
    }

    // ==================== Sorting Tests ====================

    #[tokio::test]
    async fn test_list_directories_first() {
        let dir = setup_test_dir();
        let tool = ListTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "path": dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: ListOutput = serde_json::from_str(&result.output).unwrap();

        // Find first non-directory
        let mut seen_file = false;
        for entry in &output.entries {
            if !entry.is_dir {
                seen_file = true;
            } else if seen_file {
                panic!("Directory found after file - should be sorted directories first");
            }
        }
    }

    #[tokio::test]
    async fn test_list_alphabetical_sort() {
        let dir = setup_test_dir();
        let tool = ListTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "path": dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: ListOutput = serde_json::from_str(&result.output).unwrap();

        // Check directories are alphabetically sorted among themselves
        let dirs: Vec<&str> = output
            .entries
            .iter()
            .filter(|e| e.is_dir)
            .map(|e| e.name.as_str())
            .collect();

        let mut sorted_dirs = dirs.clone();
        sorted_dirs.sort();
        assert_eq!(
            dirs, sorted_dirs,
            "Directories should be alphabetically sorted"
        );
    }

    // ==================== Metadata Tests ====================

    #[tokio::test]
    async fn test_list_file_size_metadata() {
        let dir = setup_test_dir();
        let tool = ListTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "path": dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: ListOutput = serde_json::from_str(&result.output).unwrap();

        // Files should have size, directories should not
        for entry in &output.entries {
            if entry.is_dir {
                assert!(entry.size.is_none(), "Directories should not have size");
            } else {
                assert!(entry.size.is_some(), "Files should have size");
            }
        }
    }

    #[tokio::test]
    async fn test_list_is_dir_flag() {
        let dir = setup_test_dir();
        let tool = ListTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "path": dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: ListOutput = serde_json::from_str(&result.output).unwrap();

        // Verify is_dir flag is correct
        for entry in &output.entries {
            if entry.name == "src" || entry.name == "tests" {
                assert!(entry.is_dir, "{} should be a directory", entry.name);
            } else if entry.name.ends_with(".rs") || entry.name.ends_with(".md") {
                assert!(!entry.is_dir, "{} should not be a directory", entry.name);
            }
        }
    }

    // ==================== Hidden Files Tests ====================

    #[tokio::test]
    async fn test_list_hidden_files_excluded() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join(".hidden"), "secret").unwrap();
        std::fs::write(dir.path().join("visible.txt"), "public").unwrap();

        let tool = ListTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "path": dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Completed);

        let output: ListOutput = serde_json::from_str(&result.output).unwrap();

        let names: Vec<&str> = output.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(
            !names.contains(&".hidden"),
            "Hidden files should be excluded"
        );
        assert!(
            names.contains(&"visible.txt"),
            "Visible files should be included"
        );
    }

    // ==================== Pattern Matching Tests ====================

    #[test]
    fn test_matches_pattern_wildcard() {
        assert!(matches_pattern("test.rs", "*.rs"));
        assert!(matches_pattern("foo.rs", "*.rs"));
        assert!(!matches_pattern("test.md", "*.rs"));
    }

    #[test]
    fn test_matches_pattern_question_mark() {
        assert!(matches_pattern("test1.rs", "test?.rs"));
        assert!(matches_pattern("testA.rs", "test?.rs"));
        assert!(!matches_pattern("test12.rs", "test?.rs"));
    }

    #[test]
    fn test_matches_pattern_exact() {
        assert!(matches_pattern("README.md", "README.md"));
        assert!(!matches_pattern("readme.md", "README.md"));
    }
}
