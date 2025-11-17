//! Task tool - launches subagents to handle complex tasks
//! Based on opencode/packages/opencode/src/tool/task.ts

use crate::agent::{AgentExecutor, AgentRegistry};
use crate::providers::ProviderClient;
use crate::session::SessionStore;
use crate::tools::{Tool, ToolResult, ToolStatus};
use crate::types::{Message, MessagePath, MessageTime, Part, Session, SessionTime};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

/// Task tool description template (shameless copy from OpenCode's task.txt)
/// The {agents} placeholder gets replaced with actual agent list at runtime
const DESCRIPTION: &str = r#"Launch a new agent to handle complex, multi-step tasks autonomously.

Available agent types and the tools they have access to:
{agents}

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

pub struct TaskTool {
    session_store: Arc<SessionStore>,
    agent_registry: Arc<AgentRegistry>,
    tool_registry: Arc<std::sync::RwLock<Option<Arc<crate::tools::ToolRegistry>>>>,
    lock_manager: Arc<crate::session::SessionLockManager>,
    provider_config: crate::providers::ProviderConfig,
    description: String,
}

impl TaskTool {
    pub async fn new(
        session_store: Arc<SessionStore>,
        agent_registry: Arc<AgentRegistry>,
        tool_registry: Arc<std::sync::RwLock<Option<Arc<crate::tools::ToolRegistry>>>>,
        lock_manager: Arc<crate::session::SessionLockManager>,
        provider_config: crate::providers::ProviderConfig,
    ) -> Self {
        // Build dynamic description like OpenCode does
        // Get all subagents (non-primary agents)
        let agents = agent_registry.get_subagents().await;
        let agent_list = agents
            .iter()
            .map(|a| {
                let desc = a
                    .description
                    .as_ref()
                    .map(|d| d.as_str())
                    .unwrap_or("This subagent should only be called manually by the user.");
                format!("- {}: {}", a.name, desc)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let description = DESCRIPTION.replace("{agents}", &agent_list);

        Self {
            session_store,
            agent_registry,
            tool_registry,
            lock_manager,
            provider_config,
            description,
        }
    }
}

#[async_trait]
impl Tool for TaskTool {
    fn name(&self) -> &str {
        "task"
    }

    fn description(&self) -> &str {
        &self.description
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

    async fn execute(&self, input: Value, ctx: &crate::tools::ToolContext) -> ToolResult {
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

        // Create child session ID
        let child_session_id = format!("ses-{}", uuid::Uuid::new_v4());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Use working_dir from context for child session
        let working_dir = ctx.working_dir.clone();

        // Create child session using store's create method
        let child_session = match self.session_store.create(
            working_dir.to_string_lossy().to_string(),
            Some(ctx.session_id.clone()),
            Some(format!("Subagent: {}", task_input.description)),
        ) {
            Ok(s) => s,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to create child session: {}", e)),
                    metadata: json!({}),
                };
            }
        };

        // Create initial user message with the task prompt
        let user_message_id = format!("msg-{}", uuid::Uuid::new_v4());
        let user_message = crate::session::MessageWithParts {
            info: Message::User {
                id: user_message_id.clone(),
                session_id: child_session.id.clone(),
                time: MessageTime {
                    created: now,
                    completed: Some(now),
                },
                summary: None,
                metadata: None,
            },
            parts: vec![Part::Text {
                id: format!("part-{}", uuid::Uuid::new_v4()),
                session_id: child_session.id.clone(),
                message_id: user_message_id.clone(),
                text: task_input.prompt.clone(),
            }],
        };

        // Store user message
        if let Err(e) = self
            .session_store
            .add_message(&child_session.id, user_message)
        {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!("Failed to add user message: {}", e)),
                metadata: json!({}),
            };
        }

        // Create provider client
        let provider = match ProviderClient::new(self.provider_config.clone()) {
            Ok(p) => p,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to create provider: {}", e)),
                    metadata: json!({}),
                };
            }
        };

        // Get tool registry from the RwLock
        let tool_registry = match self.tool_registry.read() {
            Ok(guard) => match guard.as_ref() {
                Some(reg) => reg.clone(),
                None => {
                    return ToolResult {
                        status: ToolStatus::Error,
                        output: String::new(),
                        error: Some("Tool registry not initialized".to_string()),
                        metadata: json!({}),
                    };
                }
            },
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to access tool registry: {}", e)),
                    metadata: json!({}),
                };
            }
        };

        // Create executor
        let executor = AgentExecutor::new(
            provider,
            tool_registry,
            self.session_store.clone(),
            self.agent_registry.clone(),
            self.lock_manager.clone(),
        );

        // Execute the child agent
        let result = executor
            .execute_turn(
                &child_session.id,
                &task_input.subagent_type,
                &working_dir,
                vec![], // No additional user parts - already stored
            )
            .await;

        match result {
            Ok(response) => {
                // Extract text from response parts
                let output = response
                    .parts
                    .iter()
                    .filter_map(|p| match p {
                        Part::Text { text, .. } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                ToolResult {
                    status: ToolStatus::Completed,
                    output,
                    error: None,
                    metadata: json!({
                        "title": task_input.description,
                        "sessionId": child_session.id,
                        "subagent": task_input.subagent_type,
                    }),
                }
            }
            Err(e) => ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!("Subagent execution failed: {}", e)),
                metadata: json!({
                    "sessionId": child_session.id,
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_tool() -> TaskTool {
        let session_store = Arc::new(SessionStore::new());
        let agent_registry = Arc::new(AgentRegistry::new());
        let tool_registry = Arc::new(std::sync::RwLock::new(Some(Arc::new(
            crate::tools::ToolRegistry::new(),
        ))));
        let lock_manager = Arc::new(crate::session::SessionLockManager::new());
        let provider_config = crate::providers::ProviderConfig {
            provider: "moonshotai".to_string(),
            model: "kimi-k2-thinking".to_string(),
            api_key: "test-key".to_string(),
            base_url: None,
        };

        TaskTool::new(
            session_store,
            agent_registry,
            tool_registry,
            lock_manager,
            provider_config,
        )
    }

    fn create_test_context() -> crate::tools::ToolContext {
        crate::tools::ToolContext {
            session_id: "test-session".to_string(),
            message_id: "test-message".to_string(),
            agent: "build".to_string(),
            working_dir: PathBuf::from("/tmp/test"),
        }
    }

    #[tokio::test]
    async fn test_task_tool_validation() {
        let tool = create_test_tool();
        let ctx = create_test_context();

        // Test missing parameters
        let result = tool.execute(json!({"description": "test"}), &ctx).await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_task_tool_invalid_agent() {
        let tool = create_test_tool();
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "description": "test task",
                    "prompt": "do something",
                    "subagent_type": "invalid-agent"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("Unknown agent type"));
    }
}
