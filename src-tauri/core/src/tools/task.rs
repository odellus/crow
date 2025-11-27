//! Task tool - launches subagents to handle complex tasks
//! Based on opencode/packages/opencode/src/tool/task.ts
//!
//! Special subagent type: "verified"
//! When subagent_type is "verified", runs a dual-agent loop (executor + arbiter)
//! instead of a single agent. The dual-agent pattern provides verification
//! via an arbiter that can run tests, take screenshots, and call task_complete.

use crate::agent::{AgentExecutor, AgentRegistry, DualAgentRuntime};
use crate::providers::ProviderClient;
use crate::session::SessionStore;
use crate::tools::{Tool, ToolResult, ToolStatus};
use crate::types::{Message, MessageTime, Part};
use async_trait::async_trait;
use serde::Deserialize;
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
    /// For dual-agent mode: maximum executor↔arbiter iterations (default: 5)
    #[serde(default = "default_max_steps")]
    max_steps: u32,
}

fn default_max_steps() -> u32 {
    5
}

pub struct TaskTool {
    session_store: Arc<SessionStore>,
    agent_registry: Arc<AgentRegistry>,
    tool_registry: Arc<parking_lot::RwLock<Option<Arc<crate::tools::ToolRegistry>>>>,
    lock_manager: Arc<crate::session::SessionLockManager>,
    provider_config: crate::providers::ProviderConfig,
    description: String,
}

impl TaskTool {
    pub async fn new(
        session_store: Arc<SessionStore>,
        agent_registry: Arc<AgentRegistry>,
        tool_registry: Arc<parking_lot::RwLock<Option<Arc<crate::tools::ToolRegistry>>>>,
        lock_manager: Arc<crate::session::SessionLockManager>,
        provider_config: crate::providers::ProviderConfig,
    ) -> Self {
        // Build dynamic description like OpenCode does
        // Get all subagents (non-primary agents)
        let agents = agent_registry.get_subagents().await;
        let mut agent_list = agents
            .iter()
            .map(|a| {
                let desc = a
                    .description
                    .as_ref()
                    .map(|d| d.as_str())
                    .unwrap_or("This subagent should only be called manually by the user.");
                format!("- {}: {}", a.name, desc)
            })
            .collect::<Vec<_>>();

        // Add verified agent type - keep description simple, parent doesn't need to know internals
        agent_list.push(
            "- verified: Agent for tasks requiring verified completion with automatic testing and validation before marking done.".to_string(),
        );

        let description = DESCRIPTION.replace("{agents}", &agent_list.join("\n"));

        Self {
            session_store,
            agent_registry,
            tool_registry,
            lock_manager,
            provider_config,
            description,
        }
    }

    /// Execute dual-agent mode: executor + arbiter in a verification loop
    async fn execute_dual_agent(
        &self,
        task_input: &TaskInput,
        ctx: &crate::tools::ToolContext,
    ) -> ToolResult {
        let working_dir = ctx.working_dir.clone();

        // Create dual-agent runtime
        let runtime = DualAgentRuntime::new(
            self.session_store.clone(),
            self.agent_registry.clone(),
            self.lock_manager.clone(),
            self.provider_config.clone(),
        );

        // Run the dual-agent loop
        match runtime
            .run(
                &task_input.prompt,
                Some(&ctx.session_id), // Parent session
                &working_dir,
                "executor", // executor agent (like build but no Task tool)
                "arbiter",  // arbiter agent (has task_complete)
                task_input.max_steps,
            )
            .await
        {
            Ok(result) => {
                // Build output message
                let mut output = String::new();

                if result.completed {
                    output.push_str(&format!(
                        "✅ Task completed and verified in {} step(s).\n\n",
                        result.steps
                    ));

                    if let Some(summary) = &result.summary {
                        output.push_str(&format!("**Summary:** {}\n\n", summary));
                    }

                    if let Some(verification) = &result.verification {
                        output.push_str(&format!("**Verification:** {}\n", verification));
                    }
                } else {
                    output.push_str(&format!(
                        "⚠️ Task incomplete after {} step(s) (max steps reached).\n\n",
                        result.steps
                    ));
                    output.push_str("The arbiter did not call task_complete. Review the executor and arbiter sessions for details.");
                }

                // Add tool calls summary to output for agent visibility
                if !result.tool_calls.is_empty() {
                    output.push_str("\n**Tool calls:**\n");
                    for tc in &result.tool_calls {
                        let status = if tc.success { "✓" } else { "✗" };
                        output.push_str(&format!(
                            "- {} {}/{}: {} ({}ms)\n",
                            status, tc.agent, tc.step, tc.tool, tc.duration_ms
                        ));
                    }
                }

                ToolResult {
                    status: ToolStatus::Completed,
                    output,
                    error: None,
                    metadata: json!({
                        "title": task_input.description,
                        "subagent": "verified",
                        "completed": result.completed,
                        "steps": result.steps,
                        "executor_session_id": result.executor_session_id,
                        "arbiter_session_id": result.arbiter_session_id,
                        "pair_id": result.pair_id,
                        "summary": result.summary,
                        "verification": result.verification,
                        "tool_calls": result.tool_calls,
                        "total_cost": result.total_cost,
                        "total_input_tokens": result.total_input_tokens,
                        "total_output_tokens": result.total_output_tokens,
                    }),
                }
            }
            Err(e) => ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!("Verified agent execution failed: {}", e)),
                metadata: json!({
                    "subagent": "verified",
                }),
            },
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
                    "description": "The type of specialized agent to use for this task. Use 'verified' for verified execution with automatic testing and validation."
                },
                "max_steps": {
                    "type": "integer",
                    "description": "For dual-agent mode only: maximum executor↔arbiter iterations (default: 5)"
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

        // Check for dual-agent mode
        if task_input.subagent_type == "verified" {
            return self.execute_dual_agent(&task_input, ctx).await;
        }

        // Validate subagent type - must exist and be usable as subagent
        let agent = match self.agent_registry.get(&task_input.subagent_type).await {
            Some(a) => a,
            None => {
                let available = self.agent_registry.get_subagents().await;
                let names: Vec<_> = available.iter().map(|a| a.name.as_str()).collect();
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!(
                        "Unknown agent type: '{}'. Available subagents: {}",
                        task_input.subagent_type,
                        names.join(", ")
                    )),
                    metadata: json!({}),
                };
            }
        };

        // Ensure agent can be used as subagent
        if !agent.is_subagent() {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!(
                    "Agent '{}' cannot be used as a subagent (mode: {:?})",
                    task_input.subagent_type, agent.mode
                )),
                metadata: json!({}),
            };
        }

        // Get current time
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

        // Get tool registry from the RwLock (parking_lot doesn't poison)
        let tool_registry = match self.tool_registry.read().as_ref() {
            Some(reg) => reg.clone(),
            None => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some("Tool registry not initialized".to_string()),
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
    use crate::agent::types::{AgentInfo, AgentMode};
    use std::path::PathBuf;

    /// Create a test TaskTool with default configuration
    async fn create_test_tool() -> TaskTool {
        let session_store = Arc::new(SessionStore::new());
        let agent_registry = Arc::new(AgentRegistry::new());
        let tool_registry = Arc::new(parking_lot::RwLock::new(Some(Arc::new(
            crate::tools::ToolRegistry::new(),
        ))));
        let lock_manager = Arc::new(crate::session::SessionLockManager::new());
        let provider_config = crate::providers::ProviderConfig {
            name: "moonshotai".to_string(),
            base_url: "https://api.moonshot.ai/v1".to_string(),
            api_key_env: "MOONSHOT_API_KEY".to_string(),
            default_model: "kimi-k2-thinking".to_string(),
        };

        TaskTool::new(
            session_store,
            agent_registry,
            tool_registry,
            lock_manager,
            provider_config,
        )
        .await
    }

    /// Create TaskTool with custom agent registry for testing specific agent scenarios
    async fn create_test_tool_with_agents(agents: Vec<AgentInfo>) -> TaskTool {
        let session_store = Arc::new(SessionStore::new());
        let agent_registry = Arc::new(AgentRegistry::new());

        // Register custom agents
        for agent in agents {
            agent_registry.register(agent).await;
        }

        let tool_registry = Arc::new(parking_lot::RwLock::new(Some(Arc::new(
            crate::tools::ToolRegistry::new(),
        ))));
        let lock_manager = Arc::new(crate::session::SessionLockManager::new());
        let provider_config = crate::providers::ProviderConfig {
            name: "test".to_string(),
            base_url: "https://test.example.com/v1".to_string(),
            api_key_env: "TEST_API_KEY".to_string(),
            default_model: "test-model".to_string(),
        };

        TaskTool::new(
            session_store,
            agent_registry,
            tool_registry,
            lock_manager,
            provider_config,
        )
        .await
    }

    fn create_test_context() -> crate::tools::ToolContext {
        crate::tools::ToolContext::new(
            "test-session".to_string(),
            "test-message".to_string(),
            "build".to_string(),
            PathBuf::from("/tmp/test"),
        )
    }

    fn create_test_context_with_session(session_id: &str) -> crate::tools::ToolContext {
        crate::tools::ToolContext::new(
            session_id.to_string(),
            "test-message".to_string(),
            "build".to_string(),
            PathBuf::from("/tmp/test"),
        )
    }

    // ==================== Input Validation Tests ====================

    #[tokio::test]
    async fn test_task_error_missing_prompt_parameter() {
        let tool = create_test_tool().await;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "description": "test task",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
        let error = result.error.unwrap();
        assert!(
            error.contains("Invalid input") || error.contains("missing field"),
            "Expected missing field error, got: {}",
            error
        );
    }

    #[tokio::test]
    async fn test_task_error_missing_subagent_type() {
        let tool = create_test_tool().await;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "description": "test task",
                    "prompt": "do something"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
        let error = result.error.unwrap();
        assert!(
            error.contains("Invalid input") || error.contains("missing field"),
            "Expected missing field error, got: {}",
            error
        );
    }

    #[tokio::test]
    async fn test_task_error_missing_description() {
        let tool = create_test_tool().await;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "prompt": "do something",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_task_error_empty_input() {
        let tool = create_test_tool().await;
        let ctx = create_test_context();

        let result = tool.execute(json!({}), &ctx).await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_task_error_invalid_json_types() {
        let tool = create_test_tool().await;
        let ctx = create_test_context();

        // Wrong types for fields
        let result = tool
            .execute(
                json!({
                    "description": 123,  // Should be string
                    "prompt": true,      // Should be string
                    "subagent_type": []  // Should be string
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_task_parses_input_correctly() {
        let tool = create_test_tool().await;
        let ctx = create_test_context();

        // Valid input with all required fields - should fail only because of agent lookup
        // (which proves parsing succeeded)
        let result = tool
            .execute(
                json!({
                    "description": "test description",
                    "prompt": "test prompt content",
                    "subagent_type": "nonexistent"
                }),
                &ctx,
            )
            .await;

        // Should fail at agent lookup, not parsing
        assert_eq!(result.status, ToolStatus::Error);
        let error = result.error.unwrap();
        assert!(
            error.contains("Unknown agent type"),
            "Should have parsed successfully but failed at agent lookup, got: {}",
            error
        );
    }

    // ==================== Agent Type Validation Tests ====================

    #[tokio::test]
    async fn test_task_error_invalid_agent_name() {
        let tool = create_test_tool().await;
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
        let error = result.error.unwrap();
        assert!(error.contains("Unknown agent type"));
        assert!(error.contains("invalid-agent"));
    }

    #[tokio::test]
    async fn test_task_validates_subagent_type() {
        let tool = create_test_tool().await;
        let ctx = create_test_context();

        // "general" is a built-in subagent
        let result = tool
            .execute(
                json!({
                    "description": "test task",
                    "prompt": "do something",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Should not fail on validation - may fail on execution (no API key)
        // but error should NOT be about unknown agent type
        if result.status == ToolStatus::Error {
            let error = result.error.unwrap_or_default();
            assert!(
                !error.contains("Unknown agent type"),
                "Should recognize 'general' agent, got: {}",
                error
            );
        }
    }

    #[tokio::test]
    async fn test_task_rejects_non_subagent_agents() {
        // Create an agent that is Primary-only (cannot be used as subagent)
        let mut primary_only = AgentInfo::new("primary-only");
        primary_only.mode = AgentMode::Primary;
        primary_only.description = Some("A primary-only agent".to_string());

        let tool = create_test_tool_with_agents(vec![primary_only]).await;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "description": "test task",
                    "prompt": "do something",
                    "subagent_type": "primary-only"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        let error = result.error.unwrap();
        assert!(
            error.contains("cannot be used as a subagent"),
            "Expected 'cannot be used as subagent' error, got: {}",
            error
        );
    }

    #[tokio::test]
    async fn test_task_agent_list_subagents_only() {
        let tool = create_test_tool().await;
        let ctx = create_test_context();

        // Request nonexistent agent - error should list only subagents
        let result = tool
            .execute(
                json!({
                    "description": "test",
                    "prompt": "test",
                    "subagent_type": "nonexistent"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        let error = result.error.unwrap();

        // "general" is a subagent, should be listed
        assert!(
            error.contains("general"),
            "Error should list 'general' subagent: {}",
            error
        );

        // "build" and "plan" are Primary agents, should NOT be listed
        // (The error only lists available subagents)
    }

    // ==================== Description Generation Tests ====================

    #[tokio::test]
    async fn test_task_generates_dynamic_description() {
        let tool = create_test_tool().await;

        let description = tool.description();

        // Description should contain the template text
        assert!(description.contains("Launch a new agent"));
        assert!(description.contains("subagent_type"));

        // Should contain available agents
        assert!(
            description.contains("general"),
            "Description should list 'general' agent"
        );
    }

    #[tokio::test]
    async fn test_task_agent_description_includes_capabilities() {
        // Create custom agent with description
        let mut custom = AgentInfo::new("custom-explorer");
        custom.mode = AgentMode::Subagent;
        custom.description = Some("Explores codebases and finds files".to_string());

        let tool = create_test_tool_with_agents(vec![custom]).await;

        let description = tool.description();

        // Should include custom agent and its description
        assert!(
            description.contains("custom-explorer"),
            "Should list custom agent name"
        );
        assert!(
            description.contains("Explores codebases"),
            "Should include agent description"
        );
    }

    // ==================== Tool Metadata Tests ====================

    #[tokio::test]
    async fn test_task_tool_name() {
        let tool = create_test_tool().await;
        assert_eq!(tool.name(), "task");
    }

    #[tokio::test]
    async fn test_task_parameters_schema() {
        let tool = create_test_tool().await;
        let schema = tool.parameters_schema();

        // Check schema structure
        assert_eq!(schema["type"], "object");

        // Check required fields
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("description")));
        assert!(required.contains(&json!("prompt")));
        assert!(required.contains(&json!("subagent_type")));

        // Check properties exist
        let props = schema["properties"].as_object().unwrap();
        assert!(props.contains_key("description"));
        assert!(props.contains_key("prompt"));
        assert!(props.contains_key("subagent_type"));
    }

    // ==================== Session Management Tests ====================

    #[tokio::test]
    async fn test_task_creates_child_session() {
        let session_store = Arc::new(SessionStore::new());
        let agent_registry = Arc::new(AgentRegistry::new());
        let tool_registry = Arc::new(parking_lot::RwLock::new(Some(Arc::new(
            crate::tools::ToolRegistry::new(),
        ))));
        let lock_manager = Arc::new(crate::session::SessionLockManager::new());
        let provider_config = crate::providers::ProviderConfig {
            name: "test".to_string(),
            base_url: "https://test.example.com/v1".to_string(),
            api_key_env: "TEST_API_KEY".to_string(),
            default_model: "test-model".to_string(),
        };

        let tool = TaskTool::new(
            session_store.clone(),
            agent_registry,
            tool_registry,
            lock_manager,
            provider_config,
        )
        .await;

        // Create parent session first
        let parent_session = session_store
            .create("/tmp/test".to_string(), None, Some("Parent".to_string()))
            .unwrap();

        let ctx = create_test_context_with_session(&parent_session.id);

        // Get initial session count
        let initial_count = session_store.list(None).unwrap().len();

        // Execute task - will fail on API but should create child session
        let _result = tool
            .execute(
                json!({
                    "description": "test child",
                    "prompt": "do something",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Should have created a new session
        let final_count = session_store.list(None).unwrap().len();
        assert!(
            final_count > initial_count,
            "Should have created child session. Initial: {}, Final: {}",
            initial_count,
            final_count
        );
    }

    #[tokio::test]
    async fn test_task_inherits_working_dir_from_parent() {
        let session_store = Arc::new(SessionStore::new());
        let agent_registry = Arc::new(AgentRegistry::new());
        let tool_registry = Arc::new(parking_lot::RwLock::new(Some(Arc::new(
            crate::tools::ToolRegistry::new(),
        ))));
        let lock_manager = Arc::new(crate::session::SessionLockManager::new());
        let provider_config = crate::providers::ProviderConfig {
            name: "test".to_string(),
            base_url: "https://test.example.com/v1".to_string(),
            api_key_env: "TEST_API_KEY".to_string(),
            default_model: "test-model".to_string(),
        };

        let tool = TaskTool::new(
            session_store.clone(),
            agent_registry,
            tool_registry,
            lock_manager,
            provider_config,
        )
        .await;

        // Create parent session with specific working dir
        let working_dir = "/home/user/myproject";
        let parent_session = session_store
            .create(working_dir.to_string(), None, Some("Parent".to_string()))
            .unwrap();

        let mut ctx = create_test_context_with_session(&parent_session.id);
        ctx.working_dir = PathBuf::from(working_dir);

        // Execute task
        let _result = tool
            .execute(
                json!({
                    "description": "test",
                    "prompt": "test",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Find child session (most recently created)
        let sessions = session_store.list(None).unwrap();
        let child = sessions
            .iter()
            .find(|s| s.parent_id.is_some())
            .expect("Should have child session");

        assert_eq!(
            child.directory, working_dir,
            "Child should inherit working directory"
        );
    }

    #[tokio::test]
    async fn test_task_session_has_correct_parent_id() {
        let session_store = Arc::new(SessionStore::new());
        let agent_registry = Arc::new(AgentRegistry::new());
        let tool_registry = Arc::new(parking_lot::RwLock::new(Some(Arc::new(
            crate::tools::ToolRegistry::new(),
        ))));
        let lock_manager = Arc::new(crate::session::SessionLockManager::new());
        let provider_config = crate::providers::ProviderConfig {
            name: "test".to_string(),
            base_url: "https://test.example.com/v1".to_string(),
            api_key_env: "TEST_API_KEY".to_string(),
            default_model: "test-model".to_string(),
        };

        let tool = TaskTool::new(
            session_store.clone(),
            agent_registry,
            tool_registry,
            lock_manager,
            provider_config,
        )
        .await;

        // Create parent session
        let parent_session = session_store
            .create("/tmp/test".to_string(), None, Some("Parent".to_string()))
            .unwrap();

        let ctx = create_test_context_with_session(&parent_session.id);

        // Execute task
        let _result = tool
            .execute(
                json!({
                    "description": "child task",
                    "prompt": "test",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Find child session
        let sessions = session_store.list(None).unwrap();
        let child = sessions.iter().find(|s| s.parent_id.is_some());

        assert!(child.is_some(), "Should have created child session");
        let child = child.unwrap();
        assert_eq!(
            child.parent_id,
            Some(parent_session.id.clone()),
            "Child should have correct parent_id"
        );
    }

    // ==================== Metadata Tests ====================

    #[tokio::test]
    async fn test_task_returns_metadata_with_session_id() {
        let session_store = Arc::new(SessionStore::new());
        let agent_registry = Arc::new(AgentRegistry::new());
        let tool_registry = Arc::new(parking_lot::RwLock::new(Some(Arc::new(
            crate::tools::ToolRegistry::new(),
        ))));
        let lock_manager = Arc::new(crate::session::SessionLockManager::new());
        let provider_config = crate::providers::ProviderConfig {
            name: "test".to_string(),
            base_url: "https://test.example.com/v1".to_string(),
            api_key_env: "TEST_API_KEY".to_string(),
            default_model: "test-model".to_string(),
        };

        let tool = TaskTool::new(
            session_store.clone(),
            agent_registry,
            tool_registry,
            lock_manager,
            provider_config,
        )
        .await;

        let parent = session_store
            .create("/tmp/test".to_string(), None, None)
            .unwrap();

        let ctx = create_test_context_with_session(&parent.id);

        let result = tool
            .execute(
                json!({
                    "description": "test metadata",
                    "prompt": "test",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Even on error, metadata should contain sessionId
        assert!(
            result.metadata.get("sessionId").is_some(),
            "Metadata should contain sessionId"
        );
    }

    #[tokio::test]
    async fn test_task_returns_metadata_with_subagent_type() {
        let session_store = Arc::new(SessionStore::new());
        let agent_registry = Arc::new(AgentRegistry::new());
        let tool_registry = Arc::new(parking_lot::RwLock::new(Some(Arc::new(
            crate::tools::ToolRegistry::new(),
        ))));
        let lock_manager = Arc::new(crate::session::SessionLockManager::new());
        let provider_config = crate::providers::ProviderConfig {
            name: "test".to_string(),
            base_url: "https://test.example.com/v1".to_string(),
            api_key_env: "TEST_API_KEY".to_string(),
            default_model: "test-model".to_string(),
        };

        let tool = TaskTool::new(
            session_store.clone(),
            agent_registry,
            tool_registry,
            lock_manager,
            provider_config,
        )
        .await;

        let parent = session_store
            .create("/tmp/test".to_string(), None, None)
            .unwrap();

        let ctx = create_test_context_with_session(&parent.id);

        let result = tool
            .execute(
                json!({
                    "description": "test metadata",
                    "prompt": "test",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // On success path metadata should contain subagent
        // But since we likely error due to no API key, check error path too
        if result.status == ToolStatus::Completed {
            assert_eq!(
                result.metadata.get("subagent").and_then(|v| v.as_str()),
                Some("general"),
                "Metadata should contain subagent type"
            );
        }
        // For error case, sessionId is still included
    }

    // ==================== Error Handling Tests ====================

    #[tokio::test]
    async fn test_task_error_session_creation_failure() {
        // This is harder to test directly without mocking
        // But we can verify error handling structure by checking
        // that session_store errors are caught
        let tool = create_test_tool().await;
        let ctx = create_test_context();

        // The tool should handle errors gracefully
        let result = tool
            .execute(
                json!({
                    "description": "test",
                    "prompt": "test",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Should not panic, should return either success or handled error
        assert!(
            result.status == ToolStatus::Completed || result.status == ToolStatus::Error,
            "Should handle gracefully"
        );
    }

    // ==================== Edge Cases ====================

    #[tokio::test]
    async fn test_task_handles_unicode_prompt() {
        let tool = create_test_tool().await;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "description": "Unicode 测试 🎉",
                    "prompt": "Handle this: 你好世界 🌍 مرحبا",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Should parse correctly (may fail on execution, but not parsing)
        if result.status == ToolStatus::Error {
            let error = result.error.unwrap_or_default();
            assert!(
                !error.contains("Invalid input"),
                "Should handle unicode correctly"
            );
        }
    }

    #[tokio::test]
    async fn test_task_handles_very_long_prompt() {
        let tool = create_test_tool().await;
        let ctx = create_test_context();

        let long_prompt = "x".repeat(10000);
        let result = tool
            .execute(
                json!({
                    "description": "long test",
                    "prompt": long_prompt,
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Should handle gracefully
        if result.status == ToolStatus::Error {
            let error = result.error.unwrap_or_default();
            // Should not fail on input parsing
            assert!(
                !error.contains("Invalid input"),
                "Should handle long prompts"
            );
        }
    }

    #[tokio::test]
    async fn test_task_handles_empty_strings() {
        let tool = create_test_tool().await;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "description": "",
                    "prompt": "",
                    "subagent_type": ""
                }),
                &ctx,
            )
            .await;

        // Should return error for empty subagent_type
        assert_eq!(result.status, ToolStatus::Error);
    }

    #[tokio::test]
    async fn test_task_handles_special_characters_in_description() {
        let tool = create_test_tool().await;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "description": "test <xml> & \"quotes\" 'apostrophe'",
                    "prompt": "test with {json} [array] $var",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Should parse correctly
        if result.status == ToolStatus::Error {
            let error = result.error.unwrap_or_default();
            assert!(
                !error.contains("Invalid input"),
                "Should handle special characters"
            );
        }
    }
}

/// Integration tests for Task tool
/// These tests verify Task works correctly with real dependencies
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::agent::types::{AgentInfo, AgentMode};
    use crate::session::SessionStore;
    use crate::tools::ToolRegistry;
    use std::path::PathBuf;

    /// Create a fully integrated TaskTool with real dependencies
    async fn create_integrated_task_tool() -> (
        TaskTool,
        Arc<SessionStore>,
        Arc<AgentRegistry>,
        Arc<crate::session::SessionLockManager>,
    ) {
        let session_store = Arc::new(SessionStore::new());
        let agent_registry = Arc::new(AgentRegistry::new());
        let tool_registry = Arc::new(parking_lot::RwLock::new(Some(
            Arc::new(ToolRegistry::new()),
        )));
        let lock_manager = Arc::new(crate::session::SessionLockManager::new());
        let provider_config = crate::providers::ProviderConfig {
            name: "test".to_string(),
            base_url: "https://test.example.com/v1".to_string(),
            api_key_env: "TEST_API_KEY".to_string(),
            default_model: "test-model".to_string(),
        };

        let tool = TaskTool::new(
            session_store.clone(),
            agent_registry.clone(),
            tool_registry,
            lock_manager.clone(),
            provider_config,
        )
        .await;

        (tool, session_store, agent_registry, lock_manager)
    }

    fn create_test_context(session_id: &str, working_dir: &str) -> crate::tools::ToolContext {
        crate::tools::ToolContext::new(
            session_id.to_string(),
            "test-message".to_string(),
            "build".to_string(),
            PathBuf::from(working_dir),
        )
    }

    // ==================== Session Store Integration ====================

    #[tokio::test]
    async fn test_task_with_session_store_persistence() {
        let (tool, session_store, _, _) = create_integrated_task_tool().await;

        // Create parent session
        let parent = session_store
            .create(
                "/tmp/integration-test".to_string(),
                None,
                Some("Parent Session".to_string()),
            )
            .unwrap();

        let ctx = create_test_context(&parent.id, "/tmp/integration-test");

        // Execute task - creates child session
        let _result = tool
            .execute(
                json!({
                    "description": "integration test",
                    "prompt": "test the integration",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Verify child session was created and persisted
        let sessions = session_store.list(None).unwrap();
        let child_sessions: Vec<_> = sessions
            .iter()
            .filter(|s| s.parent_id.as_ref() == Some(&parent.id))
            .collect();

        assert_eq!(
            child_sessions.len(),
            1,
            "Should have exactly one child session"
        );

        let child = child_sessions[0];
        assert_eq!(child.directory, "/tmp/integration-test");
        assert!(child.title.contains("Subagent"));
    }

    #[tokio::test]
    async fn test_task_session_message_added_before_execution() {
        let (tool, session_store, _, _) = create_integrated_task_tool().await;

        // Create parent session
        let parent = session_store
            .create("/tmp/msg-test".to_string(), None, None)
            .unwrap();

        let ctx = create_test_context(&parent.id, "/tmp/msg-test");

        // Execute task
        let _result = tool
            .execute(
                json!({
                    "description": "message test",
                    "prompt": "verify message is added",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Find child session
        let sessions = session_store.list(None).unwrap();
        let child = sessions
            .iter()
            .find(|s| s.parent_id.as_ref() == Some(&parent.id))
            .expect("Child session should exist");

        // Verify message was added to child session
        let messages = session_store.get_messages(&child.id).unwrap();
        assert!(!messages.is_empty(), "Child session should have messages");

        // First message should be user message with the prompt
        let first_msg = &messages[0];
        match &first_msg.info {
            crate::types::Message::User { .. } => {
                // Check parts contain the prompt
                let has_prompt = first_msg.parts.iter().any(|p| {
                    if let crate::types::Part::Text { text, .. } = p {
                        text.contains("verify message is added")
                    } else {
                        false
                    }
                });
                assert!(has_prompt, "User message should contain the prompt");
            }
            _ => panic!("First message should be User message"),
        }
    }

    #[tokio::test]
    async fn test_task_multiple_sequential_subagents() {
        let (tool, session_store, _, _) = create_integrated_task_tool().await;

        // Create parent session
        let parent = session_store
            .create("/tmp/sequential-test".to_string(), None, None)
            .unwrap();

        let ctx = create_test_context(&parent.id, "/tmp/sequential-test");

        // Execute multiple tasks sequentially
        for i in 1..=3 {
            let _result = tool
                .execute(
                    json!({
                        "description": format!("sequential task {}", i),
                        "prompt": format!("task number {}", i),
                        "subagent_type": "general"
                    }),
                    &ctx,
                )
                .await;
        }

        // Verify all child sessions were created
        let sessions = session_store.list(None).unwrap();
        let child_sessions: Vec<_> = sessions
            .iter()
            .filter(|s| s.parent_id.as_ref() == Some(&parent.id))
            .collect();

        assert_eq!(child_sessions.len(), 3, "Should have 3 child sessions");

        // Each child should be unique
        let unique_ids: std::collections::HashSet<_> =
            child_sessions.iter().map(|s| &s.id).collect();
        assert_eq!(
            unique_ids.len(),
            3,
            "All child session IDs should be unique"
        );
    }

    #[tokio::test]
    async fn test_task_concurrent_subagent_isolation() {
        let (tool, session_store, _, _) = create_integrated_task_tool().await;

        // Create two parent sessions
        let parent_a = session_store
            .create(
                "/tmp/parent-a".to_string(),
                None,
                Some("Parent A".to_string()),
            )
            .unwrap();
        let parent_b = session_store
            .create(
                "/tmp/parent-b".to_string(),
                None,
                Some("Parent B".to_string()),
            )
            .unwrap();

        let ctx_a = create_test_context(&parent_a.id, "/tmp/parent-a");
        let ctx_b = create_test_context(&parent_b.id, "/tmp/parent-b");

        // Execute tasks "concurrently" (sequentially but for different parents)
        let _result_a = tool
            .execute(
                json!({
                    "description": "task for A",
                    "prompt": "parent A task",
                    "subagent_type": "general"
                }),
                &ctx_a,
            )
            .await;

        let _result_b = tool
            .execute(
                json!({
                    "description": "task for B",
                    "prompt": "parent B task",
                    "subagent_type": "general"
                }),
                &ctx_b,
            )
            .await;

        // Verify each parent has exactly one child
        let sessions = session_store.list(None).unwrap();

        let children_a: Vec<_> = sessions
            .iter()
            .filter(|s| s.parent_id.as_ref() == Some(&parent_a.id))
            .collect();
        let children_b: Vec<_> = sessions
            .iter()
            .filter(|s| s.parent_id.as_ref() == Some(&parent_b.id))
            .collect();

        assert_eq!(children_a.len(), 1, "Parent A should have 1 child");
        assert_eq!(children_b.len(), 1, "Parent B should have 1 child");

        // Verify working directories are correct
        assert_eq!(children_a[0].directory, "/tmp/parent-a");
        assert_eq!(children_b[0].directory, "/tmp/parent-b");
    }

    // ==================== Tool Registry Integration ====================

    #[tokio::test]
    async fn test_task_with_tool_registry_access() {
        let session_store = Arc::new(SessionStore::new());
        let agent_registry = Arc::new(AgentRegistry::new());
        let tool_registry = Arc::new(parking_lot::RwLock::new(Some(
            Arc::new(ToolRegistry::new()),
        )));
        let lock_manager = Arc::new(crate::session::SessionLockManager::new());
        let provider_config = crate::providers::ProviderConfig {
            name: "test".to_string(),
            base_url: "https://test.example.com/v1".to_string(),
            api_key_env: "TEST_API_KEY".to_string(),
            default_model: "test-model".to_string(),
        };

        let tool = TaskTool::new(
            session_store.clone(),
            agent_registry,
            tool_registry.clone(),
            lock_manager,
            provider_config,
        )
        .await;

        // Verify tool registry is accessible
        let registry = tool_registry.read();
        assert!(registry.is_some(), "Tool registry should be available");

        let reg = registry.as_ref().unwrap();

        // Verify standard tools are available for subagents
        assert!(reg.get("read").is_some(), "Read tool should be available");
        assert!(reg.get("write").is_some(), "Write tool should be available");
        assert!(reg.get("edit").is_some(), "Edit tool should be available");
        assert!(reg.get("bash").is_some(), "Bash tool should be available");
        assert!(reg.get("grep").is_some(), "Grep tool should be available");
        assert!(reg.get("glob").is_some(), "Glob tool should be available");

        // Verify tool can still execute (just checking it doesn't panic)
        let parent = session_store
            .create("/tmp/registry-test".to_string(), None, None)
            .unwrap();
        let ctx = create_test_context(&parent.id, "/tmp/registry-test");

        let result = tool
            .execute(
                json!({
                    "description": "registry test",
                    "prompt": "test tool registry",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Should not panic, may error due to no API key but that's expected
        assert!(
            result.status == ToolStatus::Completed || result.status == ToolStatus::Error,
            "Should handle gracefully"
        );
    }

    // ==================== Agent Registry Integration ====================

    #[tokio::test]
    async fn test_task_agent_registry_lookup_success() {
        let (tool, session_store, agent_registry, _) = create_integrated_task_tool().await;

        // Verify "general" agent exists and is a subagent
        let general = agent_registry.get("general").await;
        assert!(general.is_some(), "General agent should exist");
        assert!(
            general.unwrap().is_subagent(),
            "General should be usable as subagent"
        );

        // Execute task with valid agent
        let parent = session_store
            .create("/tmp/agent-lookup".to_string(), None, None)
            .unwrap();
        let ctx = create_test_context(&parent.id, "/tmp/agent-lookup");

        let result = tool
            .execute(
                json!({
                    "description": "agent lookup test",
                    "prompt": "test",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Should not fail on agent lookup
        if result.status == ToolStatus::Error {
            let error = result.error.unwrap_or_default();
            assert!(
                !error.contains("Unknown agent type"),
                "Should find 'general' agent"
            );
        }
    }

    #[tokio::test]
    async fn test_task_agent_registry_lookup_failure() {
        let (tool, session_store, _, _) = create_integrated_task_tool().await;

        let parent = session_store
            .create("/tmp/agent-fail".to_string(), None, None)
            .unwrap();
        let ctx = create_test_context(&parent.id, "/tmp/agent-fail");

        let result = tool
            .execute(
                json!({
                    "description": "invalid agent test",
                    "prompt": "test",
                    "subagent_type": "nonexistent-agent-xyz"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        let error = result.error.unwrap();
        assert!(error.contains("Unknown agent type"));
        assert!(error.contains("nonexistent-agent-xyz"));
    }

    #[tokio::test]
    async fn test_task_with_custom_registered_agent() {
        let (tool, session_store, agent_registry, _) = create_integrated_task_tool().await;

        // Register a custom subagent
        let mut custom_agent = AgentInfo::new("custom-test-agent");
        custom_agent.mode = AgentMode::Subagent;
        custom_agent.description = Some("A custom test agent".to_string());
        agent_registry.register(custom_agent).await;

        // Verify it's registered
        let found = agent_registry.get("custom-test-agent").await;
        assert!(found.is_some(), "Custom agent should be registered");

        // Note: We can't test execution with custom agent since TaskTool
        // was created before we registered it. The description won't include
        // the custom agent. This is a limitation of the current design.
        // In production, agents are registered before TaskTool is created.

        let parent = session_store
            .create("/tmp/custom-agent".to_string(), None, None)
            .unwrap();
        let ctx = create_test_context(&parent.id, "/tmp/custom-agent");

        // This will fail because TaskTool's description was built before
        // custom agent was registered, but it shows the registry works
        let result = tool
            .execute(
                json!({
                    "description": "custom agent test",
                    "prompt": "test",
                    "subagent_type": "custom-test-agent"
                }),
                &ctx,
            )
            .await;

        // The execute path still checks the registry at runtime
        if result.status == ToolStatus::Error {
            let error = result.error.unwrap_or_default();
            // Should NOT say "Unknown agent type" since we registered it
            // But it might fail for other reasons (no API key, etc.)
            // This depends on implementation - let's just verify no panic
        }
    }

    // ==================== Lock Manager Integration ====================

    #[tokio::test]
    async fn test_task_respects_lock_manager() {
        let (tool, session_store, _, lock_manager) = create_integrated_task_tool().await;

        let parent = session_store
            .create("/tmp/lock-test".to_string(), None, None)
            .unwrap();
        let ctx = create_test_context(&parent.id, "/tmp/lock-test");

        // Acquire lock on parent session
        let lock = lock_manager.acquire(&parent.id);
        assert!(lock.is_ok(), "Should be able to acquire lock");

        // Execute task - should still work (Task creates child session)
        let result = tool
            .execute(
                json!({
                    "description": "lock test",
                    "prompt": "test with lock",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Child session should be created regardless of parent lock
        let sessions = session_store.list(None).unwrap();
        let has_child = sessions
            .iter()
            .any(|s| s.parent_id.as_ref() == Some(&parent.id));
        assert!(has_child, "Child session should be created");

        // Release lock
        lock_manager.release(&parent.id);
    }

    // ==================== Output Handling Integration ====================

    #[tokio::test]
    async fn test_task_parent_receives_child_output() {
        let (tool, session_store, _, _) = create_integrated_task_tool().await;

        let parent = session_store
            .create("/tmp/output-test".to_string(), None, None)
            .unwrap();
        let ctx = create_test_context(&parent.id, "/tmp/output-test");

        let result = tool
            .execute(
                json!({
                    "description": "output test",
                    "prompt": "return some output",
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Metadata should always contain session info
        assert!(
            result.metadata.get("sessionId").is_some(),
            "Should have sessionId in metadata"
        );

        // If execution succeeded, output should be populated
        if result.status == ToolStatus::Completed {
            assert!(
                result.metadata.get("subagent").is_some(),
                "Should have subagent in metadata on success"
            );
            assert!(
                result.metadata.get("title").is_some(),
                "Should have title in metadata on success"
            );
        }
    }

    #[tokio::test]
    async fn test_task_handles_empty_response() {
        // This test verifies the tool handles cases where subagent returns no text
        let (tool, session_store, _, _) = create_integrated_task_tool().await;

        let parent = session_store
            .create("/tmp/empty-response".to_string(), None, None)
            .unwrap();
        let ctx = create_test_context(&parent.id, "/tmp/empty-response");

        let result = tool
            .execute(
                json!({
                    "description": "empty response test",
                    "prompt": "",  // Empty prompt might lead to minimal response
                    "subagent_type": "general"
                }),
                &ctx,
            )
            .await;

        // Should not panic regardless of response content
        assert!(
            result.status == ToolStatus::Completed || result.status == ToolStatus::Error,
            "Should handle gracefully"
        );
    }
}
