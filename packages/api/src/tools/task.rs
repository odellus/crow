//! Task tool - launches subagents to handle complex tasks
//! Based on opencode/packages/opencode/src/tool/task.ts

use crate::tools::{Tool, ToolResult, ToolStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Task tool description template
const DESCRIPTION: &str = r#"Launch a new agent to handle complex, multi-step tasks autonomously.

The Task tool launches specialized agents (subprocesses) that autonomously handle complex tasks. Each agent type has specific capabilities and tools available to it.

Available agent types and the tools they have access to:
- general-purpose: General-purpose agent for researching complex questions, searching for code, and executing multi-step tasks. When you are searching for a keyword or file and are not confident that you will find the right match in the first few tries use this agent to perform the search for you. (Tools: *)
- build: Implementation agent for executing code and build tasks (Tools: all except task_done)

When using the Task tool, you must specify a subagent_type parameter to select which agent type to use.

When NOT to use the Task tool:
- If you want to read a specific file path, use the Read or Glob tool instead of the Task tool, to find the match more quickly
- If you are searching for a specific class definition like "class Foo", use the Glob tool instead, to find the match more quickly
- If you are searching for code within a specific file or set of 2-3 files, use the Read tool instead of the Task tool, to find the match more quickly
- Other tasks that are not related to the agent descriptions above

Usage notes:
- Launch multiple agents concurrently whenever possible, to maximize performance; to do that, use a single message with multiple tool uses
- When the agent is done, it will return a single message back to you. The result returned by the agent is not visible to the user. To show the user the result, you should send a text message back to the user with a concise summary of the result.
- Each agent invocation is stateless. You will not be able to send additional messages to the agent, nor will the agent be able to communicate with you outside of its final report. Therefore, your prompt should contain a highly detailed task description for the agent to perform autonomously and you should specify exactly what information the agent should return back to you in its final and only message to you.
- Agents with "access to current context" can see the full conversation history before the tool call. When using these agents, you can write concise prompts that reference earlier context (e.g., "investigate the error discussed above") instead of repeating information. The agent will receive all prior messages and understand the context.
- The agent's outputs should generally be trusted
- Clearly tell the agent whether you expect it to write code or just to do research (search, file reads, web fetches, etc.), since it is not aware of the user's intent
- If the agent description mentions that it should be used proactively, then you should try your best to use it without the user having to ask for it first. Use your judgement.
- If the user specifies that they want you to run agents "in parallel", you MUST send a single message with multiple Task tool use content blocks. For example, if you need to launch both a code-reviewer agent and a test-runner agent in parallel, send a single message with both tool calls.

Example usage:

<example_agent_descriptions>
"code-reviewer": use this agent after you are done writing a signficant piece of code
"greeting-responder": use this agent when to respond to user greetings with a friendly joke
</example_agent_description>

<example>
user: "Please write a function that checks if a number is prime"
assistant: Sure let me write a function that checks if a number is prime
assistant: First let me use the Write tool to write a function that checks if a number is prime
assistant: I'm going to use the Write tool to write the following code:
<code>
function isPrime(n) {
  if (n <= 1) return false
  for (let i = 2; i * i <= n; i++) {
    if (n % i === 0) return false
  }
  return true
}
</code>
<commentary>
Since a signficant piece of code was written and the task was completed, now use the code-reviewer agent to review the code
</commentary>
assistant: Now let me use the code-reviewer agent to review the code
assistant: Uses the Task tool to launch the code-reviewer agent
</example>

<example>
user: "Hello"
<commentary>
Since the user is greeting, use the greeting-responder agent to respond with a friendly joke
</commentary>
assistant: "I'm going to use the Task tool to launch the greeting-responder agent"
</example>
"#;

#[derive(Deserialize)]
struct TaskInput {
    description: String,
    prompt: String,
    subagent_type: String,
}

pub struct TaskTool;

#[async_trait]
impl Tool for TaskTool {
    fn name(&self) -> &str {
        "task"
    }

    fn description(&self) -> &str {
        DESCRIPTION
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "description": {
                    "type": "string",
                    "description": "A short (3-5 word) description of the task"
                },
                "prompt": {
                    "type": "string",
                    "description": "The task for the agent to perform"
                },
                "subagent_type": {
                    "type": "string",
                    "description": "The type of specialized agent to use for this task"
                }
            },
            "required": ["description", "prompt", "subagent_type"]
        })
    }

    async fn execute(&self, input: Value) -> ToolResult {
        // Parse input
        let task_input: TaskInput = match serde_json::from_value(input) {
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

        // Validate subagent type
        let valid_agents = vec!["general", "build"];
        if !valid_agents.contains(&task_input.subagent_type.as_str()) {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!(
                    "Unknown agent type: '{}'. Valid types: {}",
                    task_input.subagent_type,
                    valid_agents.join(", ")
                )),
                metadata: json!({}),
            };
        }

        // NOTE: Full implementation requires executor integration
        // For now, return a structured response indicating the task was received
        // The actual subagent spawning will be implemented when we wire up the executor

        let child_session_id = format!("ses-{}", uuid::Uuid::new_v4());

        ToolResult {
            status: ToolStatus::Completed,
            output: format!(
                "Task delegated to {} agent: {}\n\nPrompt: {}\n\n[Note: Full subagent execution pending executor integration. Child session ID: {}]",
                task_input.subagent_type,
                task_input.description,
                task_input.prompt,
                child_session_id
            ),
            error: None,
            metadata: json!({
                "title": task_input.description,
                "sessionId": child_session_id,
                "subagent": task_input.subagent_type,
                "prompt": task_input.prompt,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_tool_validation() {
        let tool = TaskTool;

        // Test missing parameters
        let result = tool.execute(json!({"description": "test"})).await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_task_tool_invalid_agent() {
        let tool = TaskTool;

        let result = tool
            .execute(json!({
                "description": "test task",
                "prompt": "do something",
                "subagent_type": "invalid-agent"
            }))
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("Unknown agent type"));
    }

    #[tokio::test]
    async fn test_task_tool_valid() {
        let tool = TaskTool;

        let result = tool
            .execute(json!({
                "description": "test task",
                "prompt": "do something",
                "subagent_type": "general"
            }))
            .await;

        assert_eq!(result.status, ToolStatus::Completed);
        assert!(result.output.contains("Task delegated"));
        assert!(result.output.contains("general"));
    }
}
