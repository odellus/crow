//! Agent executor - implements the ReACT (Reasoning + Acting) loop
//! This is the core of the agent system that:
//! 1. Builds system prompt with environment context
//! 2. Calls the LLM with agent-specific configuration
//! 3. Parses tool calls
//! 4. Executes tools (with permission checking)
//! 5. Loops until complete

use crate::logging::log_agent_execution;
use crate::{
    agent::{AgentInfo, AgentRegistry, SystemPromptBuilder},
    providers::ProviderClient,
    session::{MessageWithParts, SessionStore},
    snapshot::SnapshotManager,
    tools::ToolRegistry,
    types::{Message, MessageTime, Part},
};
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionRequestUserMessageContent, ChatCompletionRequestUserMessageContentPart,
};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// Events emitted during execution for streaming
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "event", content = "data")]
pub enum ExecutionEvent {
    /// Text delta (streaming token by token)
    TextDelta { id: String, delta: String },
    /// A new part was created (tool call started, text generated, etc.)
    Part(Part),
    /// Execution completed with final message
    Complete(MessageWithParts),
    /// An error occurred
    Error(String),
}

pub struct AgentExecutor {
    provider: ProviderClient,
    tools: Arc<ToolRegistry>,
    session_store: Arc<SessionStore>,
    agent_registry: Arc<AgentRegistry>,
    #[allow(dead_code)]
    lock_manager: Arc<crate::session::SessionLockManager>,
    cancellation: CancellationToken,
}

impl AgentExecutor {
    pub fn new(
        provider: ProviderClient,
        tools: Arc<ToolRegistry>,
        session_store: Arc<SessionStore>,
        agent_registry: Arc<AgentRegistry>,
        lock_manager: Arc<crate::session::SessionLockManager>,
    ) -> Self {
        Self {
            provider,
            tools,
            session_store,
            agent_registry,
            lock_manager,
            cancellation: CancellationToken::new(),
        }
    }

    /// Get a clone of the cancellation token (for sharing with abort endpoint)
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    /// Set the cancellation token (to link with session lock)
    pub fn set_cancellation_token(&mut self, token: CancellationToken) {
        self.cancellation = token;
    }

    /// Abort the current execution
    pub fn abort(&self) {
        self.cancellation.cancel();
    }

    /// Check if execution has been aborted
    pub fn is_aborted(&self) -> bool {
        self.cancellation.is_cancelled()
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
        tracing::info!(
            "Executing turn: session={}, agent={}, working_dir={}",
            session_id,
            agent_id,
            working_dir.display()
        );

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
        let model_id = self.provider.config().default_model.clone();
        let llm_messages = self.build_llm_context(session_id, &agent, working_dir, &model_id)?;

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

        // Insert agent-specific reminders into last user message (like OpenCode does)
        Self::insert_reminders(&mut llm_messages, &agent.name);

        // Create snapshot manager for this working directory (shadow git for file tracking)
        let snapshot_manager = SnapshotManager::for_directory(working_dir.to_path_buf());

        // Track snapshot before executing tools
        let snapshot_hash = match snapshot_manager.track().await {
            Ok(hash) => hash,
            Err(e) => {
                tracing::warn!("Failed to track snapshot: {}", e);
                None
            }
        };

        // ReACT loop
        let mut parts = vec![];

        // Track token usage across all iterations
        let mut total_input_tokens = 0u64;
        let mut total_output_tokens = 0u64;

        // Get tool definitions (filtered by agent permissions)
        let tool_defs = self.get_agent_tools(&agent);

        let max_iterations = 10;
        for _iteration in 0..max_iterations {
            // Check abort at start of each iteration
            if self.is_aborted() {
                return Err("Session aborted".to_string());
            }

            // Call LLM with tools, using agent's temperature and top_p
            let response = self
                .provider
                .chat_with_tools(llm_messages.clone(), tool_defs.clone(), None)
                .await
                .map_err(|e| format!("LLM call failed: {}", e))?;

            // Track token usage from this call
            if let Some(usage) = &response.usage {
                total_input_tokens += usage.prompt_tokens as u64;
                total_output_tokens += usage.completion_tokens as u64;
            }

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

                    // Create tool part with pending state
                    let tool_part_id = format!("part-tool-{}", uuid::Uuid::new_v4());
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;

                    parts.push(Part::Tool {
                        id: tool_part_id.clone(),
                        session_id: session_id.to_string(),
                        message_id: message_id.clone(),
                        call_id: tool_call.id.clone(),
                        tool: tool_name.clone(),
                        state: crate::types::ToolState::Pending {
                            input: args.clone(),
                            raw: tool_args.clone(),
                        },
                    });

                    // Check if aborted before executing tool
                    if self.is_aborted() {
                        return Err("Session aborted".to_string());
                    }

                    // Execute tool with context
                    let tool_ctx = crate::tools::ToolContext {
                        session_id: session_id.to_string(),
                        message_id: message_id.clone(),
                        agent: agent_id.to_string(),
                        working_dir: working_dir.to_path_buf(),
                        project_root: crate::tools::find_project_root(working_dir),
                        call_id: Some(tool_call.id.clone()),
                        provider_id: Some(self.provider.config().name.clone()),
                        model_id: Some(self.provider.config().default_model.clone()),
                        abort: Some(self.cancellation.clone()),
                    };

                    tracing::info!("Executing tool: {} with args: {:?}", tool_name, args);

                    let tool_result = self
                        .tools
                        .execute(tool_name, args.clone(), &tool_ctx)
                        .await
                        .map_err(|e| format!("Tool execution failed: {}", e))?;

                    tracing::info!(
                        "Tool {} completed: status={:?}",
                        tool_name,
                        tool_result.status
                    );

                    let end_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;

                    // Create patch for file-modifying tools
                    if let Some(ref hash) = snapshot_hash {
                        let is_file_modifying =
                            matches!(tool_name.as_str(), "edit" | "write" | "bash");
                        if is_file_modifying {
                            match snapshot_manager.patch(hash).await {
                                Ok(patch) if !patch.files.is_empty() => {
                                    parts.push(Part::Patch {
                                        id: format!("part-patch-{}", uuid::Uuid::new_v4()),
                                        session_id: session_id.to_string(),
                                        message_id: message_id.clone(),
                                        hash: patch.hash,
                                        files: patch
                                            .files
                                            .iter()
                                            .map(|p| p.to_string_lossy().to_string())
                                            .collect(),
                                    });
                                }
                                Ok(_) => {} // No files changed
                                Err(e) => {
                                    tracing::warn!("Failed to create patch: {}", e);
                                }
                            }
                        }
                    }

                    // Update tool part with completed state
                    if let Some(part) = parts.last_mut() {
                        *part = Part::Tool {
                            id: tool_part_id.clone(),
                            session_id: session_id.to_string(),
                            message_id: message_id.clone(),
                            call_id: tool_call.id.clone(),
                            tool: tool_name.clone(),
                            state: crate::types::ToolState::Completed {
                                input: args.clone(),
                                output: tool_result.output.clone(),
                                title: tool_name.clone(),
                                time: crate::types::ToolTime {
                                    start: now,
                                    end: Some(end_time),
                                },
                            },
                        };
                    }

                    // Check for doom loop after adding tool part
                    if let Err(warning) = crate::agent::DoomLoopDetector::check(&parts) {
                        eprintln!("{}", warning);
                        // Add warning as text part for visibility
                        parts.push(Part::Text {
                            id: format!("part-warning-{}", uuid::Uuid::new_v4()),
                            session_id: session_id.to_string(),
                            message_id: message_id.clone(),
                            text: warning.clone(),
                        });
                        // Also add to LLM context so agent knows to stop
                        llm_messages.push(ChatCompletionRequestMessage::User(
                            ChatCompletionRequestUserMessageArgs::default()
                                .content(warning)
                                .build()
                                .map_err(|e| format!("Failed to build warning message: {}", e))?,
                        ));
                    }

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

        // Calculate cost based on kimi k2 pricing
        // Input: $0.15 per million tokens
        // Output: $2.50 per million tokens
        let input_cost = (total_input_tokens as f64 / 1_000_000.0) * 0.15;
        let output_cost = (total_output_tokens as f64 / 1_000_000.0) * 2.50;
        let total_cost = input_cost + output_cost;

        tracing::info!(
            "Turn complete: session={}, tokens(in={}, out={}), cost=${:.6}",
            session_id,
            total_input_tokens,
            total_output_tokens,
            total_cost
        );

        // Log to structured logs (XDG state directory)
        log_agent_execution(
            session_id,
            agent_id,
            &self.provider.config().name,
            &self.provider.config().default_model,
            total_input_tokens,
            total_output_tokens,
            total_cost,
            0, // TODO: track duration
            parts
                .iter()
                .filter(|p| matches!(p, Part::Tool { .. }))
                .count(),
            None,
        );

        // Create assistant message with actual token counts and cost
        let assistant_message = MessageWithParts {
            info: Message::Assistant {
                id: message_id.clone(),
                session_id: session_id.to_string(),
                parent_id: "".to_string(), // TODO: Get actual parent
                model_id: "kimi-k2-thinking".to_string(),
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
                cost: total_cost,
                tokens: crate::types::TokenUsage {
                    input: total_input_tokens,
                    output: total_output_tokens,
                    reasoning: 0,
                    cache: crate::types::CacheTokens { read: 0, write: 0 },
                },
                error: None,
                summary: None,
                metadata: None,
            },
            parts,
        };

        // Store the assistant message
        self.session_store
            .add_message(session_id, assistant_message.clone())?;

        Ok(assistant_message)
    }

    /// Execute a turn with streaming - emits events as parts are created
    pub async fn execute_turn_streaming(
        &self,
        session_id: &str,
        agent_id: &str,
        working_dir: &Path,
        user_parts: Vec<Part>,
        event_tx: mpsc::UnboundedSender<ExecutionEvent>,
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

        // Build LLM context
        let model_id = self.provider.config().default_model.clone();
        let mut llm_messages =
            self.build_llm_context(session_id, &agent, working_dir, &model_id)?;

        // Add user message to context
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
                        .content(user_text)
                        .build()
                        .map_err(|e| format!("Failed to build user message: {}", e))?,
                ));
            }
        }

        Self::insert_reminders(&mut llm_messages, &agent.name);

        // Create snapshot manager for this working directory (shadow git for file tracking)
        let snapshot_manager = SnapshotManager::for_directory(working_dir.to_path_buf());

        // Track snapshot before executing tools
        let snapshot_hash = match snapshot_manager.track().await {
            Ok(hash) => hash,
            Err(e) => {
                tracing::warn!("Failed to track snapshot: {}", e);
                None
            }
        };

        let mut parts = vec![];
        let mut total_input_tokens = 0u64;
        let mut total_output_tokens = 0u64;
        let tool_defs = self.get_agent_tools(&agent);

        let max_iterations = 10;
        for _iteration in 0..max_iterations {
            if self.is_aborted() {
                return Err("Session aborted".to_string());
            }

            // Create channel for streaming deltas
            let (delta_tx, mut delta_rx) = mpsc::unbounded_channel();

            // Clone what we need for the streaming task
            let provider = self.provider.clone();
            let msgs = llm_messages.clone();
            let tools = tool_defs.clone();

            // Spawn streaming task with cancellation support
            let cancel_token = self.cancellation.clone();
            let stream_handle = tokio::spawn(async move {
                provider
                    .chat_with_tools_stream(msgs, tools, None, delta_tx, Some(cancel_token))
                    .await
            });

            // Collect the response while streaming deltas
            let mut accumulated_text = String::new();
            let mut accumulated_reasoning = String::new();
            let mut tool_calls: std::collections::HashMap<usize, (String, String, String)> =
                std::collections::HashMap::new();
            let text_part_id = format!("part-text-{}", uuid::Uuid::new_v4());
            let reasoning_part_id = format!("part-thinking-{}", uuid::Uuid::new_v4());

            loop {
                // Use select to allow cancellation during delta reception
                let delta = tokio::select! {
                    biased;
                    _ = self.cancellation.cancelled() => {
                        // Abort the stream handle and return error
                        stream_handle.abort();
                        return Err("Session aborted".to_string());
                    }
                    delta = delta_rx.recv() => delta,
                };

                let Some(delta) = delta else {
                    break;
                };

                match delta {
                    crate::providers::StreamDelta::Reasoning(text) => {
                        // Emit reasoning delta as a special text delta
                        let _ = event_tx.send(ExecutionEvent::TextDelta {
                            id: reasoning_part_id.clone(),
                            delta: text.clone(),
                        });

                        // Publish to global bus as thinking part
                        crate::bus::publish(
                            crate::bus::events::MESSAGE_PART_UPDATED,
                            serde_json::json!({
                                "part": {
                                    "type": "thinking",
                                    "id": reasoning_part_id,
                                    "session_id": session_id,
                                    "message_id": message_id,
                                },
                                "delta": text,
                            }),
                        );

                        accumulated_reasoning.push_str(&text);
                    }
                    crate::providers::StreamDelta::Text(text) => {
                        // Emit text delta event
                        let _ = event_tx.send(ExecutionEvent::TextDelta {
                            id: text_part_id.clone(),
                            delta: text.clone(),
                        });

                        // Publish to global bus
                        crate::bus::publish(
                            crate::bus::events::MESSAGE_PART_UPDATED,
                            serde_json::json!({
                                "part": {
                                    "type": "text",
                                    "id": text_part_id,
                                    "session_id": session_id,
                                    "message_id": message_id,
                                },
                                "delta": text,
                            }),
                        );

                        accumulated_text.push_str(&text);
                    }
                    crate::providers::StreamDelta::ToolCall {
                        index,
                        id,
                        name,
                        arguments,
                    } => {
                        let entry = tool_calls
                            .entry(index)
                            .or_insert_with(|| (String::new(), String::new(), String::new()));
                        if let Some(id) = id {
                            entry.0 = id;
                        }
                        if let Some(name) = name {
                            entry.1 = name;
                        }
                        entry.2.push_str(&arguments);
                    }
                    crate::providers::StreamDelta::Usage { input, output } => {
                        total_input_tokens += input;
                        total_output_tokens += output;
                    }
                    crate::providers::StreamDelta::Done => break,
                }
            }

            // Wait for stream to complete and check for abort errors
            match stream_handle.await {
                Ok(Err(e)) if e.contains("aborted") => {
                    return Err("Session aborted".to_string());
                }
                _ => {}
            }

            // Process tool calls if any
            if !tool_calls.is_empty() {
                // Build tool calls for context
                let mut openai_tool_calls = vec![];
                for (_index, (id, name, args)) in &tool_calls {
                    openai_tool_calls.push(async_openai::types::ChatCompletionMessageToolCall {
                        id: id.clone(),
                        r#type: async_openai::types::ChatCompletionToolType::Function,
                        function: async_openai::types::FunctionCall {
                            name: name.clone(),
                            arguments: args.clone(),
                        },
                    });
                }

                // Add assistant message with tool calls
                llm_messages.push(ChatCompletionRequestMessage::Assistant(
                    ChatCompletionRequestAssistantMessageArgs::default()
                        .tool_calls(openai_tool_calls.clone())
                        .build()
                        .map_err(|e| format!("Failed to build assistant message: {}", e))?,
                ));

                // Execute each tool
                for (_index, (tool_id, tool_name, tool_args_str)) in tool_calls {
                    let args: serde_json::Value =
                        serde_json::from_str(&tool_args_str).unwrap_or(serde_json::json!({}));

                    // Create tool part
                    let tool_part_id = format!("part-tool-{}", uuid::Uuid::new_v4());
                    let start_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;

                    // Execute tool
                    let tool_ctx = crate::tools::ToolContext {
                        session_id: session_id.to_string(),
                        message_id: message_id.clone(),
                        agent: agent_id.to_string(),
                        working_dir: working_dir.to_path_buf(),
                        project_root: crate::tools::find_project_root(working_dir),
                        call_id: Some(tool_id.clone()),
                        provider_id: Some(self.provider.config().name.clone()),
                        model_id: Some(self.provider.config().default_model.clone()),
                        abort: Some(self.cancellation.clone()),
                    };

                    let tool_result = self
                        .tools
                        .execute(&tool_name, args.clone(), &tool_ctx)
                        .await
                        .map_err(|e| format!("Tool execution failed: {}", e))?;

                    let end_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;

                    // Create patch for file-modifying tools
                    if let Some(ref hash) = snapshot_hash {
                        let is_file_modifying =
                            matches!(tool_name.as_str(), "edit" | "write" | "bash");
                        if is_file_modifying {
                            match snapshot_manager.patch(hash).await {
                                Ok(patch) if !patch.files.is_empty() => {
                                    let patch_part = Part::Patch {
                                        id: format!("part-patch-{}", uuid::Uuid::new_v4()),
                                        session_id: session_id.to_string(),
                                        message_id: message_id.clone(),
                                        hash: patch.hash,
                                        files: patch
                                            .files
                                            .iter()
                                            .map(|p| p.to_string_lossy().to_string())
                                            .collect(),
                                    };
                                    let _ = event_tx.send(ExecutionEvent::Part(patch_part.clone()));
                                    parts.push(patch_part);
                                }
                                Ok(_) => {} // No files changed
                                Err(e) => {
                                    tracing::warn!("Failed to create patch: {}", e);
                                }
                            }
                        }
                    }

                    // Create completed tool part
                    let tool_part = Part::Tool {
                        id: tool_part_id,
                        session_id: session_id.to_string(),
                        message_id: message_id.clone(),
                        call_id: tool_id.clone(),
                        tool: tool_name.clone(),
                        state: crate::types::ToolState::Completed {
                            input: args,
                            output: tool_result.output.clone(),
                            title: tool_name.clone(),
                            time: crate::types::ToolTime {
                                start: start_time,
                                end: Some(end_time),
                            },
                        },
                    };

                    // Emit tool part
                    let _ = event_tx.send(ExecutionEvent::Part(tool_part.clone()));

                    // Publish to global bus
                    crate::bus::publish(
                        crate::bus::events::MESSAGE_PART_UPDATED,
                        serde_json::json!({ "part": tool_part }),
                    );

                    parts.push(tool_part);

                    // Add tool result to context
                    llm_messages.push(ChatCompletionRequestMessage::Tool(
                        async_openai::types::ChatCompletionRequestToolMessageArgs::default()
                            .content(tool_result.output)
                            .tool_call_id(tool_id)
                            .build()
                            .map_err(|e| format!("Failed to build tool message: {}", e))?,
                    ));
                }

                continue;
            }

            // Save reasoning/thinking part if present
            if !accumulated_reasoning.is_empty() {
                let thinking_part = Part::Thinking {
                    id: reasoning_part_id,
                    session_id: session_id.to_string(),
                    message_id: message_id.clone(),
                    text: accumulated_reasoning,
                };
                let _ = event_tx.send(ExecutionEvent::Part(thinking_part.clone()));

                // Publish to global bus
                crate::bus::publish(
                    crate::bus::events::MESSAGE_PART_UPDATED,
                    serde_json::json!({ "part": thinking_part }),
                );

                parts.push(thinking_part);
            }

            // No tool calls - we have final text
            if !accumulated_text.is_empty() {
                let text_part = Part::Text {
                    id: text_part_id,
                    session_id: session_id.to_string(),
                    message_id: message_id.clone(),
                    text: accumulated_text,
                };
                let _ = event_tx.send(ExecutionEvent::Part(text_part.clone()));

                // Publish to global bus
                crate::bus::publish(
                    crate::bus::events::MESSAGE_PART_UPDATED,
                    serde_json::json!({ "part": text_part }),
                );

                parts.push(text_part);
            }

            break;
        }

        // Calculate cost
        let input_cost = (total_input_tokens as f64 / 1_000_000.0) * 0.15;
        let output_cost = (total_output_tokens as f64 / 1_000_000.0) * 2.50;
        let total_cost = input_cost + output_cost;

        // Create assistant message
        let assistant_message = MessageWithParts {
            info: Message::Assistant {
                id: message_id.clone(),
                session_id: session_id.to_string(),
                parent_id: "".to_string(),
                model_id: self.provider.config().default_model.clone(),
                provider_id: self.provider.config().name.clone(),
                mode: agent_id.to_string(),
                time: MessageTime {
                    created: now,
                    completed: Some(now),
                },
                path: crate::types::MessagePath {
                    cwd: working_dir.to_string_lossy().to_string(),
                    root: working_dir.to_string_lossy().to_string(),
                },
                cost: total_cost,
                tokens: crate::types::TokenUsage {
                    input: total_input_tokens,
                    output: total_output_tokens,
                    reasoning: 0,
                    cache: crate::types::CacheTokens { read: 0, write: 0 },
                },
                error: None,
                summary: None,
                metadata: None,
            },
            parts,
        };

        // Store message
        self.session_store
            .add_message(session_id, assistant_message.clone())?;

        // Emit completion
        let _ = event_tx.send(ExecutionEvent::Complete(assistant_message.clone()));

        Ok(assistant_message)
    }

    /// Build LLM context from session history with agent-specific system prompt
    fn build_llm_context(
        &self,
        session_id: &str,
        agent: &AgentInfo,
        working_dir: &Path,
        model_id: &str,
    ) -> Result<Vec<ChatCompletionRequestMessage>, String> {
        let mut messages = vec![];

        // Build system prompt using SystemPromptBuilder
        let prompt_builder = SystemPromptBuilder::new(
            agent.clone(),
            working_dir.to_path_buf(),
            self.provider.config().name.clone(),
        );

        let system_prompts = prompt_builder.build(model_id);

        tracing::debug!(
            "System prompts for agent '{}' (2 messages, {} + {} chars)",
            agent.name,
            system_prompts[0].len(),
            system_prompts[1].len()
        );

        // Add 2 system messages (matching OpenCode exactly)
        for system_prompt in system_prompts {
            messages.push(ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system_prompt)
                    .build()
                    .map_err(|e| format!("Failed to build system message: {}", e))?,
            ));
        }

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
                    // Check if this message has tool calls
                    let tool_parts: Vec<_> = msg
                        .parts
                        .iter()
                        .filter_map(|p| {
                            if let Part::Tool {
                                call_id,
                                tool,
                                state,
                                ..
                            } = p
                            {
                                Some((call_id, tool, state))
                            } else {
                                None
                            }
                        })
                        .collect();

                    if !tool_parts.is_empty() {
                        // Message had tool calls - reconstruct assistant message with tool_calls
                        let mut openai_tool_calls = vec![];
                        for (call_id, tool_name, state) in &tool_parts {
                            // Get the input from the state (all states have input)
                            let input = match state {
                                crate::types::ToolState::Pending { input, .. } => input,
                                crate::types::ToolState::Running { input, .. } => input,
                                crate::types::ToolState::Completed { input, .. } => input,
                                crate::types::ToolState::Error { input, .. } => input,
                            };

                            openai_tool_calls.push(
                                async_openai::types::ChatCompletionMessageToolCall {
                                    id: (*call_id).clone(),
                                    r#type: async_openai::types::ChatCompletionToolType::Function,
                                    function: async_openai::types::FunctionCall {
                                        name: (*tool_name).clone(),
                                        arguments: serde_json::to_string(input)
                                            .unwrap_or_else(|_| "{}".to_string()),
                                    },
                                },
                            );
                        }

                        // Add assistant message with tool calls
                        messages.push(ChatCompletionRequestMessage::Assistant(
                            ChatCompletionRequestAssistantMessageArgs::default()
                                .tool_calls(openai_tool_calls)
                                .build()
                                .map_err(|e| format!("Failed to build assistant message: {}", e))?,
                        ));

                        // Now add tool response messages for each completed tool
                        for (call_id, _tool_name, state) in &tool_parts {
                            let output = match state {
                                crate::types::ToolState::Completed { output, .. } => output.clone(),
                                crate::types::ToolState::Error { error, .. } => {
                                    format!("Error: {}", error)
                                }
                                _ => {
                                    // Pending/Running shouldn't be in history, but handle gracefully
                                    "Tool execution incomplete".to_string()
                                }
                            };

                            messages.push(ChatCompletionRequestMessage::Tool(
                                async_openai::types::ChatCompletionRequestToolMessageArgs::default(
                                )
                                .tool_call_id((*call_id).clone())
                                .content(output)
                                .build()
                                .map_err(|e| format!("Failed to build tool message: {}", e))?,
                            ));
                        }
                    }

                    // Also extract any text content (assistant might have text alongside tool calls)
                    let text = msg
                        .parts
                        .iter()
                        .filter_map(|p| match p {
                            Part::Text { text, .. } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    // Only add text message if there's text AND no tool calls were in this message
                    // (if there were tool calls, the text would be part of the response after tools)
                    if !text.is_empty() && tool_parts.is_empty() {
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

        // Insert agent-specific reminders (matches OpenCode's insertReminders)
        // For PLAN agent, inject read-only reminder into last user message
        if let Some(reminder) = crate::session::prompt::insert_reminders(&agent.name) {
            if let Some(last_msg) = messages.last_mut() {
                if let ChatCompletionRequestMessage::User(user_msg) = last_msg {
                    // Append reminder to existing content
                    let current_content = user_msg.content.clone();
                    let mut content_parts = vec![];

                    match current_content {
                        ChatCompletionRequestUserMessageContent::Text(text) => {
                            content_parts.push(text);
                        }
                        ChatCompletionRequestUserMessageContent::Array(parts) => {
                            for part in parts {
                                if let ChatCompletionRequestUserMessageContentPart::Text(
                                    text_part,
                                ) = part
                                {
                                    content_parts.push(text_part.text);
                                }
                            }
                        }
                    }

                    content_parts.push(reminder);

                    *user_msg = ChatCompletionRequestUserMessageArgs::default()
                        .content(content_parts.join("\n\n"))
                        .build()
                        .map_err(|e| {
                            format!("Failed to build user message with reminder: {}", e)
                        })?;
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

    /// Insert agent-specific reminders into last user message
    /// This is how OpenCode does it - NOT in system prompt!
    /// See: opencode/packages/opencode/src/session/prompt.ts:insertReminders()
    fn insert_reminders(messages: &mut Vec<ChatCompletionRequestMessage>, agent_name: &str) {
        // Find last user message
        let last_user_idx = messages
            .iter()
            .rposition(|m| matches!(m, ChatCompletionRequestMessage::User(_)));

        if let Some(idx) = last_user_idx {
            let reminder_text = match agent_name {
                "plan" => {
                    tracing::debug!("Inserting plan reminder into last user message");
                    Some(include_str!("../prompts/plan.txt"))
                }
                "build" => {
                    // Check if there was a previous assistant message from plan mode
                    // For now, we'll skip this complexity - can add later
                    None
                }
                _ => None,
            };

            if let Some(reminder) = reminder_text {
                // Get the existing message and append the reminder
                if let ChatCompletionRequestMessage::User(user_msg) = &messages[idx] {
                    // Extract current content
                    let current_content = match &user_msg.content {
                        ChatCompletionRequestUserMessageContent::Text(text) => text.clone(),
                        ChatCompletionRequestUserMessageContent::Array(parts) => {
                            // Combine all text parts
                            parts
                                .iter()
                                .filter_map(|p| match p {
                                    ChatCompletionRequestUserMessageContentPart::Text(t) => {
                                        Some(t.text.clone())
                                    }
                                    _ => None,
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                        }
                    };

                    // Create new message with appended reminder
                    let new_content = format!("{}\n\n{}", current_content, reminder);

                    if let Ok(new_msg) = ChatCompletionRequestUserMessageArgs::default()
                        .content(new_content)
                        .build()
                    {
                        messages[idx] = ChatCompletionRequestMessage::User(new_msg);
                        tracing::debug!("Successfully inserted {} reminder", agent_name);
                    }
                }
            }
        }
    }
}
