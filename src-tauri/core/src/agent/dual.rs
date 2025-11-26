//! Dual-agent architecture for Executor + Arbiter verification pattern
//!
//! Two agents, each with their OWN session, taking turns:
//! - Executor: Does the work (standard build agent with full ReACT loop)
//! - Arbiter: Verifies work, runs tests, calls task_complete when satisfied
//!
//! Key insight: Each agent has ONE persistent session. At each step, only the
//! LATEST TURN (user message + assistant ReACT loop) is rendered to markdown
//! and sent to the other agent as a role:user message.
//!
//! The flow:
//! 1. Executor receives task, does ReACT loop (tool calls until done)
//! 2. Render executor's LATEST TURN to markdown
//! 3. Send markdown as role:user to Arbiter's session
//! 4. Arbiter does ReACT loop (can run tests, read files, etc.)
//! 5. If arbiter calls task_complete → done
//! 6. Otherwise, render arbiter's LATEST TURN to markdown
//! 7. Send markdown as role:user to Executor's session
//! 8. Repeat until task_complete or max_steps

use crate::agent::{AgentExecutor, AgentRegistry};
use crate::providers::{ProviderClient, ProviderConfig};
use crate::session::export::render_turn_to_markdown;
use crate::session::{MessageWithParts, SessionLockManager, SessionStore};
use crate::tools::ToolRegistry;
use crate::types::{Message, MessageTime, Part};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;

/// A tool call made by an agent during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolCall {
    pub agent: String,    // "executor" or "arbiter"
    pub tool: String,     // tool name
    pub step: u32,        // which step this was in
    pub duration_ms: u64, // how long it took
    pub success: bool,    // did it succeed
}

/// Result of dual-agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualAgentResult {
    /// Was the task completed successfully (task_complete called)?
    pub completed: bool,
    /// Number of executor→arbiter steps taken
    pub steps: u32,
    /// Summary from task_complete (if completed)
    pub summary: Option<String>,
    /// Verification details from task_complete (if completed)
    pub verification: Option<String>,
    /// Executor's session ID (single session, persists across all steps)
    pub executor_session_id: String,
    /// Arbiter's session ID (single session, persists across all steps)
    pub arbiter_session_id: String,
    /// The pair ID linking both sessions
    pub pair_id: String,
    /// All tool calls made during execution (for CLI visibility)
    pub tool_calls: Vec<AgentToolCall>,
    /// Total cost across all LLM calls
    pub total_cost: f64,
    /// Total input tokens
    pub total_input_tokens: u64,
    /// Total output tokens
    pub total_output_tokens: u64,
}

/// Information extracted from task_complete tool call
#[derive(Debug, Clone)]
struct TaskCompleteInfo {
    summary: String,
    verification: String,
}

/// Runtime for orchestrating dual-agent execution
pub struct DualAgentRuntime {
    session_store: Arc<SessionStore>,
    agent_registry: Arc<AgentRegistry>,
    lock_manager: Arc<SessionLockManager>,
    provider_config: ProviderConfig,
}

impl DualAgentRuntime {
    pub fn new(
        session_store: Arc<SessionStore>,
        agent_registry: Arc<AgentRegistry>,
        lock_manager: Arc<SessionLockManager>,
        provider_config: ProviderConfig,
    ) -> Self {
        Self {
            session_store,
            agent_registry,
            lock_manager,
            provider_config,
        }
    }

    /// Run the dual-agent loop
    ///
    /// # Arguments
    /// * `initial_prompt` - The user's task/request
    /// * `parent_session_id` - Optional parent session to link to
    /// * `working_dir` - Working directory for tool execution
    /// * `executor_agent` - Agent ID for executor (usually "build")
    /// * `arbiter_agent` - Agent ID for arbiter (usually "arbiter")
    /// * `max_steps` - Maximum executor→arbiter iterations
    ///
    /// # Returns
    /// DualAgentResult with completion status and session IDs
    pub async fn run(
        &self,
        initial_prompt: &str,
        parent_session_id: Option<&str>,
        working_dir: &Path,
        executor_agent: &str,
        arbiter_agent: &str,
        max_steps: u32,
    ) -> Result<DualAgentResult, String> {
        let pair_id = format!("pair-{}", uuid::Uuid::new_v4());

        // Create ONE session for executor (persists across all steps)
        let executor_session = self.create_dual_session(
            working_dir,
            parent_session_id,
            "Verified: Executor",
            "executor",
            &pair_id,
        )?;

        // Create ONE session for arbiter (persists across all steps)
        let arbiter_session = self.create_dual_session(
            working_dir,
            parent_session_id,
            "Verified: Arbiter",
            "arbiter",
            &pair_id,
        )?;

        // Link them as siblings
        self.link_siblings(&executor_session.id, &arbiter_session.id)?;

        // Create tool registry with all dependencies
        let tool_registry = ToolRegistry::new_with_deps(
            self.session_store.clone(),
            self.agent_registry.clone(),
            self.lock_manager.clone(),
            self.provider_config.clone(),
        )
        .await;

        // Share todo state between executor and arbiter sessions
        tool_registry.share_todo_sessions(&executor_session.id, &arbiter_session.id);

        // Step 0: Send initial prompt to BOTH sessions
        // Both executor and arbiter need to know what the task is
        let initial_user_msg = self.add_user_message(&executor_session.id, initial_prompt)?;
        self.add_user_message(&arbiter_session.id, initial_prompt)?;

        // Track tool calls and costs - both for agent visibility AND CLI streaming
        let mut all_tool_calls: Vec<AgentToolCall> = vec![];
        let mut total_cost = 0.0f64;
        let mut total_input_tokens = 0u64;
        let mut total_output_tokens = 0u64;

        for step in 1..=max_steps {
            // === EXECUTOR TURN ===
            let executor_provider = ProviderClient::new(self.provider_config.clone())
                .map_err(|e| format!("Failed to create executor provider: {}", e))?;

            let executor = AgentExecutor::new(
                executor_provider,
                tool_registry.clone(),
                self.session_store.clone(),
                self.agent_registry.clone(),
                self.lock_manager.clone(),
            );

            // Get the last user message for this turn (for rendering)
            let executor_user_msg = if step == 1 {
                initial_user_msg.clone()
            } else {
                self.get_last_user_message(&executor_session.id)?
            };

            // Execute executor's ReACT loop
            let executor_result = executor
                .execute_turn(&executor_session.id, executor_agent, working_dir, vec![])
                .await?;

            // Track costs and tool calls from executor
            if let Message::Assistant { cost, tokens, .. } = &executor_result.info {
                total_cost += cost;
                total_input_tokens += tokens.input;
                total_output_tokens += tokens.output;
            }
            all_tool_calls.extend(self.extract_tool_calls(
                &executor_result.parts,
                "executor",
                step,
            ));

            // Render executor's LATEST TURN to markdown (not full session!)
            let executor_turn_markdown =
                render_turn_to_markdown(&executor_user_msg, &executor_result);

            // === ARBITER TURN ===
            // Send executor's turn as a role:user message to arbiter
            // Just the markdown - like a conversation. Arbiter's system prompt tells it what to do.
            let arbiter_user_msg =
                self.add_user_message(&arbiter_session.id, &executor_turn_markdown)?;

            let arbiter_provider = ProviderClient::new(self.provider_config.clone())
                .map_err(|e| format!("Failed to create arbiter provider: {}", e))?;

            let arbiter_exec = AgentExecutor::new(
                arbiter_provider,
                tool_registry.clone(),
                self.session_store.clone(),
                self.agent_registry.clone(),
                self.lock_manager.clone(),
            );

            // Execute arbiter's ReACT loop
            let arbiter_result = arbiter_exec
                .execute_turn(&arbiter_session.id, arbiter_agent, working_dir, vec![])
                .await?;

            // Track costs and tool calls from arbiter
            if let Message::Assistant { cost, tokens, .. } = &arbiter_result.info {
                total_cost += cost;
                total_input_tokens += tokens.input;
                total_output_tokens += tokens.output;
            }
            all_tool_calls.extend(self.extract_tool_calls(&arbiter_result.parts, "arbiter", step));

            // Check if arbiter called task_complete
            if let Some(completion) = self.find_task_complete(&arbiter_result.parts) {
                // Mark sessions as complete
                self.mark_pair_complete(
                    &executor_session.id,
                    &arbiter_session.id,
                    &completion,
                    step,
                )?;

                return Ok(DualAgentResult {
                    completed: true,
                    steps: step,
                    summary: Some(completion.summary),
                    verification: Some(completion.verification),
                    executor_session_id: executor_session.id,
                    arbiter_session_id: arbiter_session.id,
                    pair_id,
                    tool_calls: all_tool_calls,
                    total_cost,
                    total_input_tokens,
                    total_output_tokens,
                });
            }

            // Not complete - render arbiter's LATEST TURN and send to executor
            let arbiter_turn_markdown = render_turn_to_markdown(&arbiter_user_msg, &arbiter_result);

            let executor_feedback = format!(
                "# Arbiter Feedback (Step {})\n\n\
                 The arbiter reviewed your work and has feedback. Address the issues below.\n\n\
                 ---\n\n{}",
                step, arbiter_turn_markdown
            );

            // Add arbiter's feedback as a user message to executor's session
            self.add_user_message(&executor_session.id, &executor_feedback)?;
        }

        // Max steps reached without completion
        Ok(DualAgentResult {
            completed: false,
            steps: max_steps,
            summary: None,
            verification: None,
            executor_session_id: executor_session.id,
            arbiter_session_id: arbiter_session.id,
            pair_id,
            tool_calls: all_tool_calls,
            total_cost,
            total_input_tokens,
            total_output_tokens,
        })
    }

    /// Create a session for dual-agent mode with appropriate metadata
    fn create_dual_session(
        &self,
        working_dir: &Path,
        parent_id: Option<&str>,
        title: &str,
        role: &str,
        pair_id: &str,
    ) -> Result<crate::types::Session, String> {
        let session = self.session_store.create(
            working_dir.to_string_lossy().to_string(),
            parent_id.map(|s| s.to_string()),
            Some(title.to_string()),
        )?;

        // Add dual-agent metadata
        self.session_store.update_metadata(
            &session.id,
            json!({
                "dual_agent": {
                    "role": role,
                    "pair_id": pair_id,
                }
            }),
        )?;

        Ok(session)
    }

    /// Link two sessions as siblings (executor ↔ arbiter)
    fn link_siblings(&self, executor_id: &str, arbiter_id: &str) -> Result<(), String> {
        // Update executor with arbiter sibling
        let executor = self.session_store.get(executor_id)?;
        let mut exec_meta = executor.metadata.unwrap_or(json!({}));
        if let Some(dual) = exec_meta.get_mut("dual_agent") {
            dual["sibling_id"] = json!(arbiter_id);
        }
        self.session_store.update_metadata(executor_id, exec_meta)?;

        // Update arbiter with executor sibling
        let arbiter = self.session_store.get(arbiter_id)?;
        let mut arb_meta = arbiter.metadata.unwrap_or(json!({}));
        if let Some(dual) = arb_meta.get_mut("dual_agent") {
            dual["sibling_id"] = json!(executor_id);
        }
        self.session_store.update_metadata(arbiter_id, arb_meta)?;

        Ok(())
    }

    /// Add a user message to a session and return it
    fn add_user_message(&self, session_id: &str, text: &str) -> Result<MessageWithParts, String> {
        let msg_id = format!("msg-user-{}", uuid::Uuid::new_v4());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let message = MessageWithParts {
            info: Message::User {
                id: msg_id.clone(),
                session_id: session_id.to_string(),
                time: MessageTime {
                    created: now,
                    completed: Some(now),
                },
                summary: None,
                metadata: None,
            },
            parts: vec![Part::Text {
                id: format!("part-{}", uuid::Uuid::new_v4()),
                session_id: session_id.to_string(),
                message_id: msg_id,
                text: text.to_string(),
            }],
        };

        self.session_store
            .add_message(session_id, message.clone())?;
        Ok(message)
    }

    /// Get the last user message from a session
    fn get_last_user_message(&self, session_id: &str) -> Result<MessageWithParts, String> {
        let messages = self.session_store.get_messages(session_id)?;
        messages
            .into_iter()
            .rev()
            .find(|m| matches!(m.info, Message::User { .. }))
            .ok_or_else(|| "No user message found in session".to_string())
    }

    /// Extract tool calls from message parts for agent visibility
    fn extract_tool_calls(&self, parts: &[Part], agent: &str, step: u32) -> Vec<AgentToolCall> {
        parts
            .iter()
            .filter_map(|part| {
                if let Part::Tool { tool, state, .. } = part {
                    // Only count completed or error states (not pending/running)
                    let (success, duration_ms) = match state {
                        crate::types::ToolState::Completed { time, .. } => (
                            true,
                            time.end.unwrap_or(time.start).saturating_sub(time.start),
                        ),
                        crate::types::ToolState::Error { time, .. } => (
                            false,
                            time.end.unwrap_or(time.start).saturating_sub(time.start),
                        ),
                        _ => return None, // Skip pending/running
                    };
                    Some(AgentToolCall {
                        agent: agent.to_string(),
                        tool: tool.clone(),
                        step,
                        duration_ms,
                        success,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if any part is a completed task_complete tool call
    fn find_task_complete(&self, parts: &[Part]) -> Option<TaskCompleteInfo> {
        for part in parts {
            if let Part::Tool { tool, state, .. } = part {
                if tool == "task_complete" {
                    if let crate::types::ToolState::Completed { output, .. } = state {
                        // The output contains the formatted message like:
                        // "Task complete.\n\nSummary: ...\n\nVerification: ..."
                        let summary = output
                            .split("Summary: ")
                            .nth(1)
                            .and_then(|s| s.split("\n\nVerification:").next())
                            .unwrap_or("")
                            .to_string();

                        let verification = output
                            .split("Verification: ")
                            .nth(1)
                            .unwrap_or("")
                            .to_string();

                        if output.contains("Task complete") {
                            return Some(TaskCompleteInfo {
                                summary,
                                verification,
                            });
                        }
                    }
                }
            }
        }
        None
    }

    /// Mark both sessions in the pair as complete
    fn mark_pair_complete(
        &self,
        executor_id: &str,
        arbiter_id: &str,
        completion: &TaskCompleteInfo,
        final_step: u32,
    ) -> Result<(), String> {
        for session_id in [executor_id, arbiter_id] {
            let session = self.session_store.get(session_id)?;
            let mut meta = session.metadata.unwrap_or(json!({}));

            if let Some(obj) = meta.as_object_mut() {
                obj.insert("dualPairComplete".to_string(), json!(true));
                obj.insert("dualPairFinalStep".to_string(), json!(final_step));
                obj.insert(
                    "completionSummary".to_string(),
                    json!(completion.summary.clone()),
                );
                obj.insert(
                    "completionVerification".to_string(),
                    json!(completion.verification.clone()),
                );
            }

            self.session_store.update_metadata(session_id, meta)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_agent_result_serialization() {
        let result = DualAgentResult {
            completed: true,
            steps: 2,
            summary: Some("Implemented feature X".to_string()),
            verification: Some("All tests pass".to_string()),
            executor_session_id: "ses-exec-1".to_string(),
            arbiter_session_id: "ses-arb-1".to_string(),
            pair_id: "pair-123".to_string(),
            tool_calls: vec![AgentToolCall {
                agent: "executor".to_string(),
                tool: "Write".to_string(),
                step: 1,
                duration_ms: 50,
                success: true,
            }],
            total_cost: 0.05,
            total_input_tokens: 1000,
            total_output_tokens: 500,
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: DualAgentResult = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.completed, true);
        assert_eq!(parsed.steps, 2);
        assert_eq!(parsed.summary, Some("Implemented feature X".to_string()));
        assert_eq!(parsed.executor_session_id, "ses-exec-1");
        assert_eq!(parsed.arbiter_session_id, "ses-arb-1");
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.tool_calls[0].tool, "Write");
    }

    #[test]
    fn test_task_complete_info() {
        let info = TaskCompleteInfo {
            summary: "Done".to_string(),
            verification: "Tests pass".to_string(),
        };
        assert_eq!(info.summary, "Done");
        assert_eq!(info.verification, "Tests pass");
    }
}
