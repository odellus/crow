//! List tool - displays files and directories
//! Accepts glob patterns to filter results

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

    async fn execute(&self, input: Value) -> ToolResult {
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
        collect_entries(path, pattern, entries).await?;

        // Get directories to recurse into
        let dirs: Vec<_> = entries
            .iter()
            .filter(|e| e.is_dir)
            .map(|e| e.path.clone())
            .collect();

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
