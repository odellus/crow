//! Agent executor - implements the ReACT (Reasoning + Acting) loop
//! This is the core of the agent system that:
//! 1. Builds system prompt with environment context
//! 2. Calls the LLM with agent-specific configuration
//! 3. Parses tool calls
//! 4. Executes tools (with permission checking)
//! 5. Loops until complete

use crate::{
    agent::{AgentInfo, AgentRegistry, SystemPromptBuilder},
    providers::ProviderClient,
    session::{MessageWithParts, SessionStore},
    tools::ToolRegistry,
    types::{Message, MessageTime, Part},
};
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct AgentExecutor {
    provider: ProviderClient,
    tools: Arc<ToolRegistry>,
    session_store: Arc<SessionStore>,
    agent_registry: Arc<AgentRegistry>,
}

impl AgentExecutor {
    pub fn new(
        provider: ProviderClient,
        tools: Arc<ToolRegistry>,
        session_store: Arc<SessionStore>,
        agent_registry: Arc<AgentRegistry>,
    ) -> Self {
        Self {
            provider,
            tools,
            session_store,
            agent_registry,
        }
    }

    /// Execute a turn - this is the main ReACT loop
    ///
    /// Arguments:
    /// - session_id: The session to execute in
    /// - agent_id: Which agent to use (build, supervisor, discriminator, etc.)
    /// - working_dir: Working directory for environment context
    /// - user_parts: New user message parts (if any)
    pub async fn execute_turn(
        &self,
        session_id: &str,
        agent_id: &str,
        working_dir: &Path,
        user_parts: Vec<Part>,
    ) -> Result<MessageWithParts, String> {
        // Get the agent
        let agent = self
            .agent_registry
            .get(agent_id)
            .await
            .ok_or_else(|| format!("Agent not found: {}", agent_id))?;

        // Get current time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?
            .as_millis() as u64;

        // Generate message ID
        let message_id = format!("msg-{}", uuid::Uuid::new_v4());

        // Build LLM context with agent-specific system prompt
        let llm_messages = self.build_llm_context(session_id, &agent, working_dir)?;

        // Add user message to context if provided
        let mut llm_messages = llm_messages;
        if !user_parts.is_empty() {
            let user_text = user_parts
                .iter()
                .filter_map(|p| match p {
                    Part::Text { text, .. } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");

            if !user_text.is_empty() {
                llm_messages.push(ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(user_text.clone())
                        .build()
                        .map_err(|e| format!("Failed to build user message: {}", e))?,
                ));
            }
        }

        // ReACT loop
        let mut parts = vec![];

        // Get tool definitions (filtered by agent permissions)
        let tool_defs = self.get_agent_tools(&agent);

        let max_iterations = 10;
        for _iteration in 0..max_iterations {
            // Call LLM with tools, using agent's temperature and top_p
            let response = self
                .provider
                .chat_with_tools(llm_messages.clone(), tool_defs.clone(), None)
                .await
                .map_err(|e| format!("LLM call failed: {}", e))?;

            let choice = response
                .choices
                .first()
                .ok_or_else(|| "No response from LLM".to_string())?;

            // Check if there are tool calls
            if let Some(tool_calls) = &choice.message.tool_calls {
                // First, add the assistant's message with tool calls to context
                llm_messages.push(ChatCompletionRequestMessage::Assistant(
                    ChatCompletionRequestAssistantMessageArgs::default()
                        .tool_calls(tool_calls.clone())
                        .build()
                        .map_err(|e| format!("Failed to build assistant message: {}", e))?,
                ));

                // Execute tools and add results
                for tool_call in tool_calls {
                    let tool_name = &tool_call.function.name;
                    let tool_args = &tool_call.function.arguments;

                    // Parse arguments
                    let args: serde_json::Value = serde_json::from_str(tool_args)
                        .map_err(|e| format!("Failed to parse tool arguments: {}", e))?;

                    // Check permission before executing
                    if let Err(e) = self.check_tool_permission(&agent, tool_name, &args) {
                        // Tool not allowed - add error to context
                        parts.push(Part::Text {
                            id: format!("part-error-{}", uuid::Uuid::new_v4()),
                            session_id: session_id.to_string(),
                            message_id: message_id.clone(),
                            text: format!("❌ Tool '{}' denied: {}", tool_name, e),
                        });

                        llm_messages.push(ChatCompletionRequestMessage::Tool(
                            async_openai::types::ChatCompletionRequestToolMessageArgs::default()
                                .content(format!("Error: {}", e))
                                .tool_call_id(tool_call.id.clone())
                                .build()
                                .map_err(|e| format!("Failed to build tool message: {}", e))?,
                        ));
                        continue;
                    }

                    // Execute tool
                    let tool_result = self
                        .tools
                        .execute(tool_name, args)
                        .await
                        .map_err(|e| format!("Tool execution failed: {}", e))?;

                    // Add tool call part
                    parts.push(Part::Text {
                        id: format!("part-tool-{}", uuid::Uuid::new_v4()),
                        session_id: session_id.to_string(),
                        message_id: message_id.clone(),
                        text: format!("🔧 {}: {}", tool_name, tool_result.output),
                    });

                    // Add tool result to LLM context
                    llm_messages.push(ChatCompletionRequestMessage::Tool(
                        async_openai::types::ChatCompletionRequestToolMessageArgs::default()
                            .content(tool_result.output)
                            .tool_call_id(tool_call.id.clone())
                            .build()
                            .map_err(|e| format!("Failed to build tool message: {}", e))?,
                    ));
                }

                // Continue loop to get next LLM response
                continue;
            }

            // No tool calls - get final text response
            if let Some(content) = &choice.message.content {
                parts.push(Part::Text {
                    id: format!("part-text-{}", uuid::Uuid::new_v4()),
                    session_id: session_id.to_string(),
                    message_id: message_id.clone(),
                    text: content.clone(),
                });
            }

            // Done - no more tool calls
            break;
        }

        // Create assistant message
        let assistant_message = MessageWithParts {
            info: Message::Assistant {
                id: message_id.clone(),
                session_id: session_id.to_string(),
                parent_id: "".to_string(), // TODO: Get actual parent
                model_id: "moonshot-v1-8k".to_string(),
                provider_id: "moonshotai".to_string(),
                mode: agent_id.to_string(),
                time: MessageTime {
                    created: now,
                    completed: Some(now),
                },
                path: crate::types::MessagePath {
                    cwd: working_dir.to_string_lossy().to_string(),
                    root: working_dir.to_string_lossy().to_string(),
                },
                cost: 0.0,
                tokens: crate::types::TokenUsage {
                    input: 0,
                    output: 0,
                    reasoning: 0,
                    cache: crate::types::CacheTokens { read: 0, write: 0 },
                },
                error: None,
                summary: None,
            },
            parts,
        };

        // Store the assistant message
        self.session_store
            .add_message(session_id, assistant_message.clone())?;

        Ok(assistant_message)
    }

    /// Build LLM context from session history with agent-specific system prompt
    fn build_llm_context(
        &self,
        session_id: &str,
        agent: &AgentInfo,
        working_dir: &Path,
    ) -> Result<Vec<ChatCompletionRequestMessage>, String> {
        let mut messages = vec![];

        // Build system prompt using SystemPromptBuilder
        let prompt_builder = SystemPromptBuilder::new(
            agent.clone(),
            working_dir.to_path_buf(),
            "moonshotai".to_string(),
        );

        let system_prompt = prompt_builder.build();

        // Add system message
        messages.push(ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()
                .map_err(|e| format!("Failed to build system message: {}", e))?,
        ));

        // Get session messages
        let session_messages = self.session_store.get_messages(session_id)?;

        // Convert to LLM format
        for msg in session_messages {
            match msg.info {
                Message::User { .. } => {
                    // Extract text from parts
                    let text = msg
                        .parts
                        .iter()
                        .filter_map(|p| match p {
                            Part::Text { text, .. } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    if !text.is_empty() {
                        messages.push(ChatCompletionRequestMessage::User(
                            ChatCompletionRequestUserMessageArgs::default()
                                .content(text)
                                .build()
                                .map_err(|e| format!("Failed to build user message: {}", e))?,
                        ));
                    }
                }
                Message::Assistant { .. } => {
                    // Extract text from parts
                    let text = msg
                        .parts
                        .iter()
                        .filter_map(|p| match p {
                            Part::Text { text, .. } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    if !text.is_empty() {
                        messages.push(ChatCompletionRequestMessage::Assistant(
                            ChatCompletionRequestAssistantMessageArgs::default()
                                .content(text)
                                .build()
                                .map_err(|e| format!("Failed to build assistant message: {}", e))?,
                        ));
                    }
                }
            }
        }

        Ok(messages)
    }

    /// Get tools available to this agent (filtered by agent.tools)
    fn get_agent_tools(&self, agent: &AgentInfo) -> Vec<async_openai::types::ChatCompletionTool> {
        let all_tools = self.tools.to_openai_tools();

        // Filter based on agent.is_tool_enabled()
        all_tools
            .into_iter()
            .filter(|tool| agent.is_tool_enabled(&tool.function.name))
            .collect()
    }

    /// Check if agent has permission to use this tool
    fn check_tool_permission(
        &self,
        agent: &AgentInfo,
        tool_name: &str,
        args: &serde_json::Value,
    ) -> Result<(), String> {
        // Check if tool is enabled for this agent
        if !agent.is_tool_enabled(tool_name) {
            return Err(format!(
                "Tool '{}' is not enabled for agent '{}'",
                tool_name, agent.name
            ));
        }

        // Special permission checks
        match tool_name {
            "edit" | "write" => {
                use crate::agent::Permission;
                if agent.permission.edit == Permission::Deny {
                    return Err("Edit permission denied for this agent".to_string());
                }
            }
            "bash" => {
                // Check bash command against patterns
                if let Some(command) = args.get("command").and_then(|v| v.as_str()) {
                    self.check_bash_permission(agent, command)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Check bash command permission against agent's bash patterns
    fn check_bash_permission(&self, agent: &AgentInfo, command: &str) -> Result<(), String> {
        use crate::agent::Permission;

        // Check against patterns in order
        for (pattern, permission) in &agent.permission.bash {
            if self.matches_bash_pattern(pattern, command) {
                match permission {
                    Permission::Allow => return Ok(()),
                    Permission::Deny => {
                        return Err(format!(
                            "Bash command '{}' denied by pattern '{}'",
                            command, pattern
                        ))
                    }
                    Permission::Ask => {
                        // TODO: Implement permission request flow
                        return Err("Permission request not implemented yet".to_string());
                    }
                }
            }
        }

        // No matching pattern - deny by default
        Err(format!(
            "No matching bash permission for command: {}",
            command
        ))
    }

    /// Check if bash command matches a pattern (supports wildcards)
    fn matches_bash_pattern(&self, pattern: &str, command: &str) -> bool {
        // Simple wildcard matching
        if pattern == "*" {
            return true;
        }

        // Pattern ends with * - prefix match
        if let Some(prefix) = pattern.strip_suffix('*') {
            return command.starts_with(prefix);
        }

        // Pattern starts with * - suffix match
        if let Some(suffix) = pattern.strip_prefix('*') {
            return command.ends_with(suffix);
        }

        // Exact match
        pattern == command
    }
}
