//! DualAgentRuntime - orchestrates executor and discriminator agents
//! Based on DUAL_AGENT_CORE.md and DUAL_SESSION_COUPLING.md

use super::dual::{AgentRole, DualAgentResult, RawMessage, SessionType, SharedConversation};
use super::executor::AgentExecutor;
use super::perspective::{transform_for_discriminator, transform_for_executor};
use super::types::AgentInfo;
use super::AgentRegistry;
use crate::providers::ProviderClient;
use crate::session::{MessageWithParts, SessionExport, SessionStore};
use crate::storage::CrowStorage;
use crate::tools::ToolRegistry;
use crate::types::{Message, Part};
use std::path::PathBuf;
use std::sync::Arc;

const MAX_STEPS: usize = 20;

pub struct DualAgentRuntime {
    tools: Arc<ToolRegistry>,
    sessions: Arc<SessionStore>,
    agents: Arc<AgentRegistry>,
    lock_manager: Arc<crate::session::SessionLockManager>,
}

impl DualAgentRuntime {
    pub fn new(
        tools: Arc<ToolRegistry>,
        sessions: Arc<SessionStore>,
        agents: Arc<AgentRegistry>,
        lock_manager: Arc<crate::session::SessionLockManager>,
    ) -> Self {
        Self {
            tools,
            sessions,
            agents,
            lock_manager,
        }
    }

    /// Create a new dual-agent pair for a task
    pub fn create_sessions(
        task: String,
        project_id: String,
    ) -> (SessionType, SessionType, SharedConversation) {
        let executor_id = format!("ses-{}", uuid::Uuid::new_v4());
        let discriminator_id = format!("ses-{}", uuid::Uuid::new_v4());

        let conv =
            SharedConversation::new(task.clone(), executor_id.clone(), discriminator_id.clone());

        let executor_session = SessionType::Executor {
            id: executor_id.clone(),
            discriminator_id: discriminator_id.clone(),
            shared_conversation_id: conv.id.clone(),
        };

        let discriminator_session = SessionType::Discriminator {
            id: discriminator_id.clone(),
            executor_id: executor_id.clone(),
            shared_conversation_id: conv.id.clone(),
        };

        (executor_session, discriminator_session, conv)
    }

    /// Run the dual-agent loop
    ///
    /// Flow:
    /// 1. Executor does work (uses tools)
    /// 2. Discriminator reviews work and provides feedback
    /// 3. If discriminator calls work_completed(ready=true), finish
    /// 4. Otherwise executor continues with discriminator feedback
    /// 5. Repeat until max steps or completion
    pub async fn run(
        &self,
        shared_conversation: &mut SharedConversation,
        working_dir: &PathBuf,
    ) -> Result<DualAgentResult, String> {
        let executor_agent = self
            .agents
            .get("build")
            .await
            .ok_or("Build agent not found")?;
        let discriminator_agent = self
            .agents
            .get("discriminator")
            .await
            .ok_or("Discriminator agent not found")?;

        // Create providers for each executor
        let provider_config = crate::providers::ProviderConfig::moonshot();
        let executor_provider = ProviderClient::new(provider_config.clone())
            .map_err(|e| format!("Failed to create executor provider: {}", e))?;
        let discriminator_provider = ProviderClient::new(provider_config)
            .map_err(|e| format!("Failed to create discriminator provider: {}", e))?;

        // Create executor for each agent
        let executor_exec = AgentExecutor::new(
            executor_provider,
            self.tools.clone(),
            self.sessions.clone(),
            self.agents.clone(),
            self.lock_manager.clone(),
        );

        let discriminator_exec = AgentExecutor::new(
            discriminator_provider,
            self.tools.clone(),
            self.sessions.clone(),
            self.agents.clone(),
            self.lock_manager.clone(),
        );
        let mut current_agent = AgentRole::Executor;
        let mut steps = 0;
        let mut completed = false;
        let mut verdict = None;

        // Add initial user message
        let initial_message = RawMessage {
            id: format!("msg-{}", uuid::Uuid::new_v4()),
            agent: AgentRole::User,
            parts: vec![Part::Text {
                id: format!("part-{}", uuid::Uuid::new_v4()),
                session_id: shared_conversation.executor_session_id.clone(),
                message_id: format!("msg-{}", uuid::Uuid::new_v4()),
                text: shared_conversation.task.clone(),
            }],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };
        shared_conversation.add_message(initial_message);

        while steps < MAX_STEPS && !completed {
            match current_agent {
                AgentRole::Executor => {
                    // Executor turn - get their view of the conversation
                    let executor_view = transform_for_executor(
                        &shared_conversation.messages,
                        &shared_conversation.executor_session_id,
                    );

                    // Execute with build agent
                    let response = executor_exec
                        .execute_turn(
                            &shared_conversation.executor_session_id,
                            "build",
                            working_dir,
                            vec![], // No new user parts - continuing from discriminator feedback
                        )
                        .await?;

                    // Add executor's work to shared conversation
                    let executor_message = RawMessage {
                        id: response.info.id().to_string(),
                        agent: AgentRole::Executor,
                        parts: response.parts,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    };

                    shared_conversation.add_message(executor_message);
                    current_agent = AgentRole::Discriminator;
                }
                AgentRole::Discriminator => {
                    // Discriminator turn - get their view of the conversation
                    let discriminator_view = transform_for_discriminator(
                        &shared_conversation.messages,
                        &shared_conversation.discriminator_session_id,
                    );

                    // Execute with discriminator agent
                    let response = discriminator_exec
                        .execute_turn(
                            &shared_conversation.discriminator_session_id,
                            "discriminator",
                            working_dir,
                            vec![], // No new user parts - reviewing executor's work
                        )
                        .await?;

                    // Check if discriminator called work_completed
                    let mut found_work_completed = false;
                    let mut summary_text = String::new();

                    for part in &response.parts {
                        match part {
                            Part::Text { text, .. } => {
                                // Look for work_completed tool call in text
                                if text.contains("🔧 work_completed") {
                                    found_work_completed = true;
                                }
                                // Collect summary text
                                if !summary_text.is_empty() {
                                    summary_text.push('\n');
                                }
                                summary_text.push_str(text);
                            }
                            Part::Tool { tool, .. } => {
                                if tool == "work_completed" {
                                    found_work_completed = true;
                                }
                            }
                            _ => {}
                        }
                    }

                    // Add discriminator's review to shared conversation
                    let discriminator_message = RawMessage {
                        id: response.info.id().to_string(),
                        agent: AgentRole::Discriminator,
                        parts: response.parts,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    };

                    shared_conversation.add_message(discriminator_message);

                    if found_work_completed {
                        completed = true;
                        verdict = Some(summary_text);
                    } else {
                        // Continue with executor incorporating discriminator feedback
                        current_agent = AgentRole::Executor;
                    }

                    steps += 1;
                }
                AgentRole::User => {
                    // Should never happen
                    break;
                }
            }
        }

        Ok(DualAgentResult {
            completed,
            steps,
            verdict,
            conversation_id: shared_conversation.id.clone(),
            executor_session_id: shared_conversation.executor_session_id.clone(),
            discriminator_session_id: shared_conversation.discriminator_session_id.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sessions() {
        let (executor, discriminator, conv) =
            DualAgentRuntime::create_sessions("Test task".to_string(), "proj-123".to_string());

        assert!(executor.is_dual());
        assert!(discriminator.is_dual());
        assert_eq!(conv.task, "Test task");

        // Sessions should reference each other
        assert_eq!(executor.reflection_id(), Some(discriminator.id()));
        assert_eq!(discriminator.reflection_id(), Some(executor.id()));

        // Both should reference same conversation
        assert_eq!(executor.shared_conversation_id(), Some(conv.id.as_str()));
        assert_eq!(
            discriminator.shared_conversation_id(),
            Some(conv.id.as_str())
        );
    }
}
