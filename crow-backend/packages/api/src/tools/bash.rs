//! Bash tool - executes shell commands
//! Mirrors OpenCode's bash tool

use super::ToolContext;
use super::{Tool, ToolResult, ToolStatus};
use async_trait::async_trait;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::process::Command;

pub struct BashTool;

#[derive(Deserialize)]
struct BashInput {
    command: String,
    #[serde(default)]
    timeout: Option<u64>,
    #[serde(default)]
    description: Option<String>,
}

const DEFAULT_TIMEOUT_MS: u64 = 120_000; // 2 minutes
const MAX_TIMEOUT_MS: u64 = 600_000; // 10 minutes
const MAX_OUTPUT_LENGTH: usize = 30_000;

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
                    "description": "The command to execute"
                },
                "timeout": {
                    "type": "number",
                    "description": "Optional timeout in milliseconds (max 600000ms / 10 minutes, default 120000ms / 2 minutes)"
                },
                "description": {
                    "type": "string",
                    "description": "Clear, concise description of what this command does in 5-10 words"
                }
            },
            "required": ["command", "description"]
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

        // Calculate timeout
        let timeout_ms = bash_input
            .timeout
            .map(|t| t.min(MAX_TIMEOUT_MS))
            .unwrap_or(DEFAULT_TIMEOUT_MS);

        // Use working directory from context
        let cwd = &ctx.working_dir;

        // Execute command with cancellation support
        let child = Command::new("bash")
            .arg("-c")
            .arg(&bash_input.command)
            .current_dir(cwd)
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
                        "description": bash_input.description,
                    }),
                };
            }
        };

        // Wait for command with timeout and cancellation support
        let timeout_duration = std::time::Duration::from_millis(timeout_ms);

        // Get process ID for killing the process tree
        let pid = child.id();

        let output = if let Some(cancel_token) = &ctx.abort {
            let cancel_token = cancel_token.clone();
            let wait_future = child.wait_with_output();

            tokio::select! {
                biased;
                _ = cancel_token.cancelled() => {
                    // Kill the process tree like opencode does
                    if let Some(pid) = pid {
                        let pgid = Pid::from_raw(-(pid as i32)); // Negative PID = process group

                        // First try SIGTERM
                        let _ = signal::kill(pgid, Signal::SIGTERM);

                        // Wait a bit then force kill with SIGKILL if still running
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        let _ = signal::kill(pgid, Signal::SIGKILL);
                    }

                    return ToolResult {
                        status: ToolStatus::Error,
                        output: "(Command was aborted)".to_string(),
                        error: Some("Command was aborted".to_string()),
                        metadata: json!({
                            "command": bash_input.command,
                            "description": bash_input.description,
                            "aborted": true,
                        }),
                    };
                }
                _ = tokio::time::sleep(timeout_duration) => {
                    // Kill the process tree on timeout
                    if let Some(pid) = pid {
                        let pgid = Pid::from_raw(-(pid as i32));
                        let _ = signal::kill(pgid, Signal::SIGTERM);
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        let _ = signal::kill(pgid, Signal::SIGKILL);
                    }

                    return ToolResult {
                        status: ToolStatus::Error,
                        output: format!("Command timed out after {} ms", timeout_ms),
                        error: Some(format!("Command timed out after {} ms", timeout_ms)),
                        metadata: json!({
                            "command": bash_input.command,
                            "description": bash_input.description,
                            "timed_out": true,
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
                                    "description": bash_input.description,
                                }),
                            };
                        }
                    }
                }
            }
        } else {
            // No cancel token, just use timeout
            let wait_future = child.wait_with_output();
            tokio::select! {
                _ = tokio::time::sleep(timeout_duration) => {
                    // Kill the process tree on timeout
                    if let Some(pid) = pid {
                        let pgid = Pid::from_raw(-(pid as i32));
                        let _ = signal::kill(pgid, Signal::SIGTERM);
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        let _ = signal::kill(pgid, Signal::SIGKILL);
                    }

                    return ToolResult {
                        status: ToolStatus::Error,
                        output: format!("Command timed out after {} ms", timeout_ms),
                        error: Some(format!("Command timed out after {} ms", timeout_ms)),
                        metadata: json!({
                            "command": bash_input.command,
                            "description": bash_input.description,
                            "timed_out": true,
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
                                    "description": bash_input.description,
                                }),
                            };
                        }
                    }
                }
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        // Combine stdout and stderr like OpenCode does
        let mut combined_output = stdout.clone();
        if !stderr.is_empty() {
            if !combined_output.is_empty() {
                combined_output.push('\n');
            }
            combined_output.push_str(&stderr);
        }

        // Truncate output if too long
        if combined_output.len() > MAX_OUTPUT_LENGTH {
            combined_output = combined_output[..MAX_OUTPUT_LENGTH].to_string();
            combined_output.push_str("\n\n(Output was truncated due to length limit)");
        }

        let status = if exit_code == 0 {
            ToolStatus::Completed
        } else {
            ToolStatus::Error
        };

        ToolResult {
            status,
            output: combined_output.clone(),
            error: if exit_code != 0 {
                Some(format!("Command exited with code {}", exit_code))
            } else {
                None
            },
            metadata: json!({
                "output": combined_output,
                "exit": exit_code,
                "description": bash_input.description,
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
            "command": "echo 'hello world'",
            "description": "Print hello world"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("hello world"));
    }

    #[tokio::test]
    async fn test_bash_error() {
        let tool = BashTool;
        let input = json!({
            "command": "exit 1",
            "description": "Exit with error"
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

    #[tokio::test]
    async fn test_bash_stderr() {
        let tool = BashTool;
        let input = json!({
            "command": "echo 'error message' >&2",
            "description": "Print to stderr"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("error message"));
    }

    #[tokio::test]
    async fn test_bash_combined_output() {
        let tool = BashTool;
        let input = json!({
            "command": "echo 'stdout' && echo 'stderr' >&2",
            "description": "Print to both streams"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("stdout"));
        assert!(result.output.contains("stderr"));
    }

    #[tokio::test]
    async fn test_bash_pwd() {
        let tool = BashTool;
        let input = json!({
            "command": "pwd",
            "description": "Print working directory"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("/tmp"));
    }

    #[tokio::test]
    async fn test_bash_multiline_output() {
        let tool = BashTool;
        let input = json!({
            "command": "echo 'line1' && echo 'line2' && echo 'line3'",
            "description": "Print multiple lines"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("line1"));
        assert!(result.output.contains("line2"));
        assert!(result.output.contains("line3"));
    }

    #[tokio::test]
    async fn test_bash_exit_code() {
        let tool = BashTool;
        let input = json!({
            "command": "exit 42",
            "description": "Exit with code 42"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("42"));
    }

    #[tokio::test]
    async fn test_bash_env_vars() {
        let tool = BashTool;
        let input = json!({
            "command": "echo $HOME",
            "description": "Print HOME env var"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
        // HOME should be set
        assert!(!result.output.trim().is_empty());
    }

    #[tokio::test]
    async fn test_bash_chained_commands() {
        let tool = BashTool;
        let input = json!({
            "command": "true && echo 'success'",
            "description": "Chain commands with &&"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("success"));
    }

    #[tokio::test]
    async fn test_bash_pipe() {
        let tool = BashTool;
        let input = json!({
            "command": "echo 'hello world' | tr 'a-z' 'A-Z'",
            "description": "Pipe to uppercase"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("HELLO WORLD"));
    }

    #[tokio::test]
    async fn test_bash_command_substitution() {
        let tool = BashTool;
        let input = json!({
            "command": "echo \"Today is $(date +%A)\"",
            "description": "Use command substitution"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("Today is"));
    }

    #[tokio::test]
    async fn test_bash_arithmetic() {
        let tool = BashTool;
        let input = json!({
            "command": "echo $((2 + 2))",
            "description": "Arithmetic expansion"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("4"));
    }

    #[tokio::test]
    async fn test_bash_quotes() {
        let tool = BashTool;
        let input = json!({
            "command": "echo \"double quotes\" && echo 'single quotes'",
            "description": "Test quoting"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("double quotes"));
        assert!(result.output.contains("single quotes"));
    }

    #[tokio::test]
    async fn test_bash_with_custom_timeout() {
        let tool = BashTool;
        let input = json!({
            "command": "echo 'quick'",
            "description": "Quick command",
            "timeout": 5000
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("quick"));
    }

    #[tokio::test]
    async fn test_bash_metadata() {
        let tool = BashTool;
        let input = json!({
            "command": "echo 'test'",
            "description": "Test metadata"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);

        // Check metadata contains expected fields
        let metadata = result.metadata;
        assert!(metadata.get("exit").is_some());
        assert!(metadata.get("description").is_some());
        assert!(metadata.get("output").is_some());
    }

    #[tokio::test]
    async fn test_bash_special_characters() {
        let tool = BashTool;
        let input = json!({
            "command": "echo 'hello\\nworld'",
            "description": "Test special chars"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
    }

    #[tokio::test]
    async fn test_bash_empty_output() {
        let tool = BashTool;
        let input = json!({
            "command": "true",
            "description": "Command with no output"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Completed);
    }

    #[tokio::test]
    async fn test_bash_failed_command() {
        let tool = BashTool;
        let input = json!({
            "command": "false",
            "description": "Always fails"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Error);
    }

    #[tokio::test]
    async fn test_bash_nonexistent_command() {
        let tool = BashTool;
        let input = json!({
            "command": "nonexistent_command_xyz",
            "description": "Run nonexistent command"
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        let result = tool.execute(input, &ctx).await;
        assert_eq!(result.status, ToolStatus::Error);
    }

    #[tokio::test]
    async fn test_bash_abort_cancellation() {
        use tokio_util::sync::CancellationToken;

        let tool = BashTool;
        let input = json!({
            "command": "sleep 10",
            "description": "Long running command"
        });

        let cancel_token = CancellationToken::new();
        let mut ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );
        ctx.abort = Some(cancel_token.clone());

        // Cancel after 100ms
        let cancel_token_clone = cancel_token.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            cancel_token_clone.cancel();
        });

        let start = std::time::Instant::now();
        let result = tool.execute(input, &ctx).await;
        let elapsed = start.elapsed();

        // Should complete quickly due to cancellation (not 10 seconds)
        assert!(
            elapsed.as_millis() < 1000,
            "Should abort quickly, took {}ms",
            elapsed.as_millis()
        );
        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("aborted"));
        assert!(result.output.contains("aborted"));
    }

    #[tokio::test]
    async fn test_bash_timeout_kills_process() {
        let tool = BashTool;
        let input = json!({
            "command": "sleep 10",
            "description": "Long running command",
            "timeout": 200
        });

        let ctx = crate::tools::ToolContext::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
        );

        let start = std::time::Instant::now();
        let result = tool.execute(input, &ctx).await;
        let elapsed = start.elapsed();

        // Should timeout around 200ms + 200ms kill grace period
        assert!(
            elapsed.as_millis() < 1000,
            "Should timeout quickly, took {}ms",
            elapsed.as_millis()
        );
        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("timed out"));
    }
}
