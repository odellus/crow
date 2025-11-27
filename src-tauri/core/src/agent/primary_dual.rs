//! Primary Dual-Agent Runtime for Planner ↔ Architect loop
//!
//! Like dual.rs but for the PRIMARY level (not subagent). Uses TWO separate sessions:
//! - Planner session: user messages are tasks/feedback, assistant messages are planner's work
//! - Architect session: user messages are planner's work to review, assistant messages are feedback
//!
//! The user can interrupt and "be" the Architect by typing feedback.
//!
//! Flow:
//! 1. Create TWO sessions (planner + architect)
//! 2. Send initial request to planner session
//! 3. Planner executes full ReACT turn
//! 4. Render planner's turn to markdown, send to architect session as user message
//! 5. Check for interrupt - if user typed, that becomes the architect's "response"
//! 6. Otherwise, AI Architect executes
//! 7. If Architect called task_complete → done
//! 8. Otherwise, render architect's turn to markdown, send to planner session as user message
//! 9. Loop back to step 3

use crate::agent::{AgentExecutor, AgentRegistry, ExecutionEvent};
use crate::providers::{ProviderClient, ProviderConfig};
use crate::session::export::render_turn_to_markdown;
use crate::session::{MessageWithParts, SessionLockManager, SessionStore};
use crate::tools::ToolRegistry;
use crate::types::{Message, MessageTime, Part, ToolState};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Result of primary dual-agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryDualResult {
    /// Was the task completed successfully (task_complete called)?
    pub completed: bool,
    /// Number of planner→architect steps taken
    pub steps: u32,
    /// Summary from task_complete (if completed)
    pub summary: Option<String>,
    /// Verification details from task_complete (if completed)
    pub verification: Option<String>,
    /// Planner's session ID
    pub planner_session_id: String,
    /// Architect's session ID
    pub architect_session_id: String,
    /// Total cost across all LLM calls
    pub total_cost: f64,
    /// Total input tokens
    pub total_input_tokens: u64,
    /// Total output tokens
    pub total_output_tokens: u64,
    /// Whether the loop was interrupted by user
    pub interrupted: bool,
}

/// Information extracted from task_complete tool call
#[derive(Debug, Clone)]
pub struct TaskCompleteInfo {
    pub summary: String,
    pub verification: String,
}

/// Events emitted during primary dual execution (for CLI rendering)
#[derive(Debug, Clone)]
pub enum PrimaryDualEvent {
    /// Planner is starting a turn
    PlannerTurnStart { step: u32 },
    /// Planner execution event (tool calls, text, etc.)
    PlannerEvent(ExecutionEvent),
    /// Planner turn completed
    PlannerTurnComplete { step: u32 },
    /// Waiting for Architect (AI or human interrupt)
    WaitingForArchitect { step: u32 },
    /// Architect is starting (AI mode)
    ArchitectTurnStart { step: u32, is_human: bool },
    /// Architect execution event
    ArchitectEvent(ExecutionEvent),
    /// Architect turn completed
    ArchitectTurnComplete {
        step: u32,
        is_human: bool,
        task_complete: bool,
    },
    /// Loop completed
    Complete(PrimaryDualResult),
    /// Error occurred
    Error(String),
}

/// Runtime for orchestrating primary dual-agent execution
pub struct PrimaryDualRuntime {
    session_store: Arc<SessionStore>,
    agent_registry: Arc<AgentRegistry>,
    lock_manager: Arc<SessionLockManager>,
    provider_config: ProviderConfig,
}

impl PrimaryDualRuntime {
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

    /// Run the primary dual-agent loop with streaming events
    ///
    /// # Arguments
    /// * `initial_prompt` - The user's task/request
    /// * `working_dir` - Working directory for tool execution
    /// * `max_steps` - Maximum planner→architect iterations
    /// * `event_tx` - Channel to send execution events
    /// * `interrupt_rx` - Channel to receive user interrupt messages
    ///
    /// # Returns
    /// PrimaryDualResult with completion status
    pub async fn run_streaming(
        &self,
        initial_prompt: &str,
        working_dir: &Path,
        max_steps: u32,
        event_tx: mpsc::UnboundedSender<PrimaryDualEvent>,
        mut interrupt_rx: mpsc::UnboundedReceiver<String>,
    ) -> Result<PrimaryDualResult, String> {
        let pair_id = format!("pair-{}", uuid::Uuid::new_v4());

        // Create TWO sessions - one for planner, one for architect
        let planner_session =
            self.create_dual_session(working_dir, "Primary: Planner", "planner", &pair_id)?;

        let architect_session =
            self.create_dual_session(working_dir, "Primary: Architect", "architect", &pair_id)?;

        // Link them as siblings
        self.link_siblings(&planner_session.id, &architect_session.id)?;

        // Create tool registry
        let tool_registry = ToolRegistry::new_with_deps(
            self.session_store.clone(),
            self.agent_registry.clone(),
            self.lock_manager.clone(),
            self.provider_config.clone(),
        )
        .await;

        // Share todo state between sessions
        tool_registry.share_todo_sessions(&planner_session.id, &architect_session.id);

        // Send initial prompt to PLANNER
        let initial_user_msg = self.add_user_message(&planner_session.id, initial_prompt)?;

        // Set up architect session
        // FROM ARCHITECT'S PERSPECTIVE:
        //   USER (planner): "How can I help you?"
        //   ASSISTANT (architect): (initial request)
        //   USER (planner): (planner's work)
        //   ASSISTANT (architect): (verification)
        self.add_user_message(&architect_session.id, "How can I help you?")?;
        self.add_assistant_message(&architect_session.id, initial_prompt)?;

        // Track costs
        let mut total_cost = 0.0f64;
        let mut total_input_tokens = 0u64;
        let mut total_output_tokens = 0u64;

        for step in 1..=max_steps {
            // === PLANNER TURN ===
            let _ = event_tx.send(PrimaryDualEvent::PlannerTurnStart { step });

            let planner_provider = ProviderClient::new(self.provider_config.clone())
                .map_err(|e| format!("Failed to create planner provider: {}", e))?;

            let planner_executor = AgentExecutor::new(
                planner_provider,
                tool_registry.clone(),
                self.session_store.clone(),
                self.agent_registry.clone(),
                self.lock_manager.clone(),
            );

            // Get the last user message for rendering
            let planner_user_msg = if step == 1 {
                initial_user_msg.clone()
            } else {
                self.get_last_user_message(&planner_session.id)?
            };

            // Create streaming channel for planner
            let (planner_tx, mut planner_rx) = mpsc::unbounded_channel::<ExecutionEvent>();

            // Execute planner turn in background
            let planner_session_id = planner_session.id.clone();
            let working_dir_clone = working_dir.to_path_buf();
            let planner_handle = tokio::spawn(async move {
                planner_executor
                    .execute_turn_streaming(
                        &planner_session_id,
                        "planner",
                        &working_dir_clone,
                        vec![], // Uses session history
                        planner_tx,
                    )
                    .await
            });

            // Forward planner events
            while let Some(event) = planner_rx.recv().await {
                let _ = event_tx.send(PrimaryDualEvent::PlannerEvent(event));
            }

            // Wait for planner to complete
            let planner_result = planner_handle
                .await
                .map_err(|e| format!("Planner task failed: {}", e))?
                .map_err(|e| format!("Planner execution failed: {}", e))?;

            // Track costs
            if let Message::Assistant { cost, tokens, .. } = &planner_result.info {
                total_cost += cost;
                total_input_tokens += tokens.input;
                total_output_tokens += tokens.output;
            }

            let _ = event_tx.send(PrimaryDualEvent::PlannerTurnComplete { step });

            // Render planner's turn to markdown
            let planner_turn_markdown = render_turn_to_markdown(&planner_user_msg, &planner_result);

            // === ARCHITECT TURN ===
            let _ = event_tx.send(PrimaryDualEvent::WaitingForArchitect { step });

            // Check for user interrupt
            let architect_input: Option<String> = tokio::select! {
                msg = interrupt_rx.recv() => msg,
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => None,
            };

            // Send planner's work to architect session as USER message
            // Architect sees planner as the "user" asking for verification
            self.add_user_message(&architect_session.id, &planner_turn_markdown)?;

            if let Some(user_feedback) = architect_input {
                // User interrupted - they become the Architect
                let _ = event_tx.send(PrimaryDualEvent::ArchitectTurnStart {
                    step,
                    is_human: true,
                });

                self.add_user_message(&planner_session.id, &user_feedback)?;

                let _ = event_tx.send(PrimaryDualEvent::ArchitectTurnComplete {
                    step,
                    is_human: true,
                    task_complete: false,
                });

                // Continue to next planner turn
                continue;
            }

            // No interrupt - run AI Architect
            let _ = event_tx.send(PrimaryDualEvent::ArchitectTurnStart {
                step,
                is_human: false,
            });

            let architect_provider = ProviderClient::new(self.provider_config.clone())
                .map_err(|e| format!("Failed to create architect provider: {}", e))?;

            let architect_executor = AgentExecutor::new(
                architect_provider,
                tool_registry.clone(),
                self.session_store.clone(),
                self.agent_registry.clone(),
                self.lock_manager.clone(),
            );

            // Get last user message for rendering
            let architect_user_msg = self.get_last_user_message(&architect_session.id)?;

            // Create streaming channel for architect
            let (architect_tx, mut architect_rx) = mpsc::unbounded_channel::<ExecutionEvent>();

            let architect_session_id = architect_session.id.clone();
            let working_dir_clone = working_dir.to_path_buf();
            let architect_handle = tokio::spawn(async move {
                architect_executor
                    .execute_turn_streaming(
                        &architect_session_id,
                        "architect",
                        &working_dir_clone,
                        vec![], // Uses session history
                        architect_tx,
                    )
                    .await
            });

            // Forward architect events
            while let Some(event) = architect_rx.recv().await {
                let _ = event_tx.send(PrimaryDualEvent::ArchitectEvent(event));
            }

            // Wait for architect to complete
            let architect_result = architect_handle
                .await
                .map_err(|e| format!("Architect task failed: {}", e))?
                .map_err(|e| format!("Architect execution failed: {}", e))?;

            // Track costs
            if let Message::Assistant { cost, tokens, .. } = &architect_result.info {
                total_cost += cost;
                total_input_tokens += tokens.input;
                total_output_tokens += tokens.output;
            }

            // Check if architect called task_complete
            if let Some(completion) = self.find_task_complete(&architect_result.parts) {
                let _ = event_tx.send(PrimaryDualEvent::ArchitectTurnComplete {
                    step,
                    is_human: false,
                    task_complete: true,
                });

                let result = PrimaryDualResult {
                    completed: true,
                    steps: step,
                    summary: Some(completion.summary),
                    verification: Some(completion.verification),
                    planner_session_id: planner_session.id,
                    architect_session_id: architect_session.id,
                    total_cost,
                    total_input_tokens,
                    total_output_tokens,
                    interrupted: false,
                };

                let _ = event_tx.send(PrimaryDualEvent::Complete(result.clone()));
                return Ok(result);
            }

            let _ = event_tx.send(PrimaryDualEvent::ArchitectTurnComplete {
                step,
                is_human: false,
                task_complete: false,
            });

            // Not complete - send architect's feedback to planner
            let architect_turn_markdown =
                render_turn_to_markdown(&architect_user_msg, &architect_result);
            self.add_user_message(&planner_session.id, &architect_turn_markdown)?;
        }

        // Max steps reached without completion
        let result = PrimaryDualResult {
            completed: false,
            steps: max_steps,
            summary: None,
            verification: None,
            planner_session_id: planner_session.id,
            architect_session_id: architect_session.id,
            total_cost,
            total_input_tokens,
            total_output_tokens,
            interrupted: false,
        };

        let _ = event_tx.send(PrimaryDualEvent::Complete(result.clone()));
        Ok(result)
    }

    /// Run without streaming (simpler API for testing)
    pub async fn run(
        &self,
        initial_prompt: &str,
        working_dir: &Path,
        max_steps: u32,
    ) -> Result<PrimaryDualResult, String> {
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let (_interrupt_tx, interrupt_rx) = mpsc::unbounded_channel();

        // Spawn the runtime
        let result_handle = {
            let initial_prompt = initial_prompt.to_string();
            let working_dir = working_dir.to_path_buf();
            let runtime = PrimaryDualRuntime::new(
                self.session_store.clone(),
                self.agent_registry.clone(),
                self.lock_manager.clone(),
                self.provider_config.clone(),
            );
            tokio::spawn(async move {
                runtime
                    .run_streaming(
                        &initial_prompt,
                        &working_dir,
                        max_steps,
                        event_tx,
                        interrupt_rx,
                    )
                    .await
            })
        };

        // Drain events (don't display them)
        while event_rx.recv().await.is_some() {}

        result_handle
            .await
            .map_err(|e| format!("Runtime task failed: {}", e))?
    }

    /// Create a session for dual-agent mode with appropriate metadata
    fn create_dual_session(
        &self,
        working_dir: &Path,
        title: &str,
        role: &str,
        pair_id: &str,
    ) -> Result<crate::types::Session, String> {
        let session = self.session_store.create(
            working_dir.to_string_lossy().to_string(),
            None, // No parent for primary dual
            Some(title.to_string()),
        )?;

        // Add dual-agent metadata
        self.session_store.update_metadata(
            &session.id,
            json!({
                "primary_dual": {
                    "role": role,
                    "pair_id": pair_id,
                }
            }),
        )?;

        Ok(session)
    }

    /// Link two sessions as siblings (planner ↔ architect)
    fn link_siblings(&self, planner_id: &str, architect_id: &str) -> Result<(), String> {
        // Update planner with architect sibling
        let planner = self.session_store.get(planner_id)?;
        let mut plan_meta = planner.metadata.unwrap_or(json!({}));
        if let Some(dual) = plan_meta.get_mut("primary_dual") {
            dual["sibling_id"] = json!(architect_id);
        }
        self.session_store.update_metadata(planner_id, plan_meta)?;

        // Update architect with planner sibling
        let architect = self.session_store.get(architect_id)?;
        let mut arch_meta = architect.metadata.unwrap_or(json!({}));
        if let Some(dual) = arch_meta.get_mut("primary_dual") {
            dual["sibling_id"] = json!(planner_id);
        }
        self.session_store
            .update_metadata(architect_id, arch_meta)?;

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

    /// Add an assistant message to a session (for prefilling architect context)
    fn add_assistant_message(
        &self,
        session_id: &str,
        text: &str,
    ) -> Result<MessageWithParts, String> {
        let msg_id = format!("msg-assistant-{}", uuid::Uuid::new_v4());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let message = MessageWithParts {
            info: Message::Assistant {
                id: msg_id.clone(),
                session_id: session_id.to_string(),
                parent_id: String::new(),
                model_id: String::new(),
                provider_id: String::new(),
                mode: "prefill".to_string(),
                time: MessageTime {
                    created: now,
                    completed: Some(now),
                },
                path: crate::types::MessagePath {
                    cwd: String::new(),
                    root: String::new(),
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

    /// Check if any part is a completed task_complete tool call
    fn find_task_complete(&self, parts: &[Part]) -> Option<TaskCompleteInfo> {
        for part in parts {
            if let Part::Tool { tool, state, .. } = part {
                if tool == "task_complete" {
                    if let ToolState::Completed { output, .. } = state {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primary_dual_result_serialization() {
        let result = PrimaryDualResult {
            completed: true,
            steps: 2,
            summary: Some("Implemented feature X".to_string()),
            verification: Some("All tests pass".to_string()),
            planner_session_id: "ses-plan-1".to_string(),
            architect_session_id: "ses-arch-1".to_string(),
            total_cost: 0.05,
            total_input_tokens: 1000,
            total_output_tokens: 500,
            interrupted: false,
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: PrimaryDualResult = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.completed, true);
        assert_eq!(parsed.steps, 2);
        assert_eq!(parsed.summary, Some("Implemented feature X".to_string()));
        assert_eq!(parsed.planner_session_id, "ses-plan-1");
        assert_eq!(parsed.architect_session_id, "ses-arch-1");
        assert!(!parsed.interrupted);
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
