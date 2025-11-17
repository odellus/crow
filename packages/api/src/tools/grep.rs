//! Grep tool for searching text in files

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::process::Command;

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

    async fn execute(&self, input: Value) -> ToolResult {
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

                let limit = 100;
                let truncated = matches.len() > limit;
                let final_matches: Vec<Match> = if truncated {
                    matches.into_iter().take(limit).collect()
                } else {
                    matches
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
