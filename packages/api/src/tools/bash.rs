//! Bash tool - executes shell commands
//! Mirrors OpenCode's bash tool

use super::ToolContext;
use super::{Tool, ToolResult, ToolStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::process::Command;

pub struct BashTool;

#[derive(Deserialize)]
struct BashInput {
    command: String,
    #[serde(default = "default_cwd")]
    cwd: String,
}

fn default_cwd() -> String {
    std::env::current_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

#[derive(Serialize, Deserialize)]
struct BashOutput {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        r#"Executes a given bash command in a persistent shell session with optional timeout, ensuring proper handling and security measures.

Before executing the command, please follow these steps:

1. Directory Verification:
   - If the command will create new directories or files, first use the List tool to verify the parent directory exists and is the correct location
   - For example, before running "mkdir foo/bar", first use List to check that "foo" exists and is the intended parent directory

2. Command Execution:
   - Always quote file paths that contain spaces with double quotes (e.g., cd "path with spaces/file.txt")
   - Examples of proper quoting:
     - cd "/Users/name/My Documents" (correct)
     - cd /Users/name/My Documents (incorrect - will fail)
     - python "/path/with spaces/script.py" (correct)
     - python /path/with spaces/script.py (incorrect - will fail)
   - After ensuring proper quoting, execute the command.
   - Capture the output of the command.

Usage notes:
  - The command argument is required.
  - You can specify an optional timeout in milliseconds (up to 600000ms / 10 minutes). If not specified, commands will timeout after 120000ms (2 minutes).
  - It is very helpful if you write a clear, concise description of what this command does in 5-10 words.
  - If the output exceeds 30000 characters, output will be truncated before being returned to you.
  - VERY IMPORTANT: You MUST avoid using search commands like `find` and `grep`. Instead use Grep, Glob, or Task to search. You MUST avoid read tools like `cat`, `head`, `tail`, and `ls`, and use Read and List to read files.
  - If you _still_ need to run `grep`, STOP. ALWAYS USE ripgrep at `rg` (or /usr/bin/rg) first, which all opencode users have pre-installed.
  - When issuing multiple commands, use the ';' or '&&' operator to separate them. DO NOT use newlines (newlines are ok in quoted strings).
  - Try to maintain your current working directory throughout the session by using absolute paths and avoiding usage of `cd`. You may use `cd` if the User explicitly requests it.

# Committing changes with git

When the user asks you to create a new git commit, follow these steps carefully:

1. You have the capability to call multiple tools in a single response. When multiple independent pieces of information are requested, batch your tool calls together for optimal performance. ALWAYS run the following bash commands in parallel, each using the Bash tool:
   - Run a git status command to see all untracked files.
   - Run a git diff command to see both staged and unstaged changes that will be committed.
   - Run a git log command to see recent commit messages, so that you can follow this repository's commit message style.

2. Analyze all staged changes (both previously staged and newly added) and draft a commit message. Wrap your analysis process in <commit_analysis> tags.

3. You have the capability to call multiple tools in a single response. When multiple independent pieces of information are requested, batch your tool calls together for optimal performance. ALWAYS run the following commands in parallel:
   - Add relevant untracked files to the staging area.
   - Run git status to make sure the commit succeeded.

4. If the commit fails due to pre-commit hook changes, retry the commit ONCE to include these automated changes.

# Creating pull requests

Use the gh command via the Bash tool for ALL GitHub-related tasks including working with issues, pull requests, checks, and releases. If given a Github URL use the gh command to get the information needed.

IMPORTANT: When the user asks you to create a pull request, follow these steps carefully:

1. Run bash commands in parallel using the Bash tool to understand the current state of the branch since it diverged from the main branch.

2. Analyze all changes that will be included in the pull request, making sure to look at all relevant commits. Wrap your analysis process in <pr_analysis> tags.

3. Run the following commands in parallel:
   - Create new branch if needed
   - Push to remote with -u flag if needed
   - Create PR using gh pr create"#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The bash command to execute"
                },
                "cwd": {
                    "type": "string",
                    "description": "Working directory for command execution (optional, defaults to current directory)"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        // Check abort before starting
        if ctx.should_abort() {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some("Aborted".to_string()),
                metadata: json!({}),
            };
        }

        // Parse input
        let bash_input: BashInput = match serde_json::from_value(input) {
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

        // Execute command with cancellation support
        let child = Command::new("bash")
            .arg("-c")
            .arg(&bash_input.command)
            .current_dir(&bash_input.cwd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        let child = match child {
            Ok(child) => child,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to spawn command: {}", e)),
                    metadata: json!({
                        "command": bash_input.command,
                        "cwd": bash_input.cwd,
                    }),
                };
            }
        };

        // Wait for command with cancellation support
        let output = if let Some(cancel_token) = &ctx.abort {
            let cancel_token = cancel_token.clone();
            let wait_future = child.wait_with_output();

            tokio::select! {
                biased;
                _ = cancel_token.cancelled() => {
                    // Note: child is moved into wait_future, but we can't kill it after select
                    // The process will be orphaned but will eventually complete
                    return ToolResult {
                        status: ToolStatus::Error,
                        output: String::new(),
                        error: Some("Aborted".to_string()),
                        metadata: json!({
                            "command": bash_input.command,
                            "cwd": bash_input.cwd,
                            "aborted": true,
                        }),
                    };
                }
                result = wait_future => {
                    match result {
                        Ok(output) => output,
                        Err(e) => {
                            return ToolResult {
                                status: ToolStatus::Error,
                                output: String::new(),
                                error: Some(format!("Failed to execute command: {}", e)),
                                metadata: json!({
                                    "command": bash_input.command,
                                    "cwd": bash_input.cwd,
                                }),
                            };
                        }
                    }
                }
            }
        } else {
            match child.wait_with_output().await {
                Ok(output) => output,
                Err(e) => {
                    return ToolResult {
                        status: ToolStatus::Error,
                        output: String::new(),
                        error: Some(format!("Failed to execute command: {}", e)),
                        metadata: json!({
                            "command": bash_input.command,
                            "cwd": bash_input.cwd,
                        }),
                    };
                }
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        let bash_output = BashOutput {
            stdout: stdout.clone(),
            stderr: stderr.clone(),
            exit_code,
        };

        let status = if exit_code == 0 {
            ToolStatus::Completed
        } else {
            ToolStatus::Error
        };

        ToolResult {
            status,
            output: serde_json::to_string(&bash_output).unwrap_or_default(),
            error: if exit_code != 0 {
                Some(format!("Command exited with code {}", exit_code))
            } else {
                None
            },
            metadata: json!({
                "command": bash_input.command,
                "cwd": bash_input.cwd,
                "exit_code": exit_code,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bash_echo() {
        let tool = BashTool;
        let input = json!({
            "command": "echo 'hello world'"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);

        let output: BashOutput = serde_json::from_str(&result.output).unwrap();
        assert_eq!(output.stdout.trim(), "hello world");
        assert_eq!(output.exit_code, 0);
    }

    #[tokio::test]
    async fn test_bash_error() {
        let tool = BashTool;
        let input = json!({
            "command": "exit 1"
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
