//! Dual-agent architecture for TRAE + Verification pattern
//! Based on DUAL_AGENT_CORE.md and DUAL_SESSION_COUPLING.md
//!
//! Two agents work in a conversation loop:
//! - Executor: Does the work (TRAE pattern with tools)
//! - Discriminator: Verifies work, provides feedback, calls task_done when satisfied

use serde::{Deserialize, Serialize};

/// Agent role in dual-agent conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentRole {
    /// The user who initiated the task
    User,
    /// The executor agent (does the work)
    Executor,
    /// The discriminator agent (verifies and provides feedback)
    Discriminator,
}

/// Session type - determines how messages are viewed and stored
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SessionType {
    /// Single standalone session (normal ReACT mode)
    #[serde(rename = "primary")]
    Primary { id: String },

    /// Executor session in a dual-agent pair
    #[serde(rename = "executor")]
    Executor {
        id: String,
        /// The discriminator session ID (my reflection)
        discriminator_id: String,
        /// The shared conversation ID (ground truth)
        shared_conversation_id: String,
    },

    /// Discriminator session in a dual-agent pair
    #[serde(rename = "discriminator")]
    Discriminator {
        id: String,
        /// The executor session ID (my reflection)
        executor_id: String,
        /// The shared conversation ID (ground truth)
        shared_conversation_id: String,
    },
}

impl SessionType {
    /// Get the session ID
    pub fn id(&self) -> &str {
        match self {
            SessionType::Primary { id } => id,
            SessionType::Executor { id, .. } => id,
            SessionType::Discriminator { id, .. } => id,
        }
    }

    /// Get the shared conversation ID (if dual session)
    pub fn shared_conversation_id(&self) -> Option<&str> {
        match self {
            SessionType::Executor {
                shared_conversation_id,
                ..
            }
            | SessionType::Discriminator {
                shared_conversation_id,
                ..
            } => Some(shared_conversation_id),
            SessionType::Primary { .. } => None,
        }
    }

    /// Get the reflection session ID (if dual session)
    pub fn reflection_id(&self) -> Option<&str> {
        match self {
            SessionType::Executor {
                discriminator_id, ..
            } => Some(discriminator_id),
            SessionType::Discriminator { executor_id, .. } => Some(executor_id),
            SessionType::Primary { .. } => None,
        }
    }

    /// Is this a dual session?
    pub fn is_dual(&self) -> bool {
        matches!(
            self,
            SessionType::Executor { .. } | SessionType::Discriminator { .. }
        )
    }

    /// Get the agent role for this session
    pub fn agent_role(&self) -> AgentRole {
        match self {
            SessionType::Executor { .. } => AgentRole::Executor,
            SessionType::Discriminator { .. } => AgentRole::Discriminator,
            SessionType::Primary { .. } => AgentRole::User, // Not in dual mode
        }
    }
}

/// Raw message in shared conversation (ground truth)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMessage {
    pub id: String,
    /// Which agent created this message
    pub agent: AgentRole,
    /// Message parts (text, tools, etc.)
    pub parts: Vec<crate::types::Part>,
    /// Timestamp
    pub timestamp: u64,
}

/// Shared conversation - ground truth for dual-agent sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedConversation {
    pub id: String,
    /// The original user task
    pub task: String,
    /// Raw messages with agent attribution
    pub messages: Vec<RawMessage>,
    /// Executor session ID
    pub executor_session_id: String,
    /// Discriminator session ID
    pub discriminator_session_id: String,
    /// Creation timestamp
    pub created: u64,
    /// Last update timestamp
    pub updated: u64,
}

impl SharedConversation {
    /// Create a new shared conversation for dual agents
    pub fn new(
        task: String,
        executor_session_id: String,
        discriminator_session_id: String,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            id: format!("conv-{}", uuid::Uuid::new_v4()),
            task,
            messages: vec![],
            executor_session_id,
            discriminator_session_id,
            created: now,
            updated: now,
        }
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, message: RawMessage) {
        self.messages.push(message);
        self.updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }
}

/// Result of dual-agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualAgentResult {
    /// Was the task completed successfully?
    pub completed: bool,
    /// Number of steps taken
    pub steps: usize,
    /// Final discriminator verdict (if completed)
    pub verdict: Option<String>,
    /// Shared conversation ID
    pub conversation_id: String,
    /// Executor session ID
    pub executor_session_id: String,
    /// Discriminator session ID
    pub discriminator_session_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_type_primary() {
        let st = SessionType::Primary {
            id: "ses-123".to_string(),
        };

        assert_eq!(st.id(), "ses-123");
        assert!(!st.is_dual());
        assert!(st.shared_conversation_id().is_none());
        assert!(st.reflection_id().is_none());
    }

    #[test]
    fn test_session_type_executor() {
        let st = SessionType::Executor {
            id: "ses-exec".to_string(),
            discriminator_id: "ses-disc".to_string(),
            shared_conversation_id: "conv-123".to_string(),
        };

        assert_eq!(st.id(), "ses-exec");
        assert!(st.is_dual());
        assert_eq!(st.shared_conversation_id(), Some("conv-123"));
        assert_eq!(st.reflection_id(), Some("ses-disc"));
        assert_eq!(st.agent_role(), AgentRole::Executor);
    }

    #[test]
    fn test_session_type_discriminator() {
        let st = SessionType::Discriminator {
            id: "ses-disc".to_string(),
            executor_id: "ses-exec".to_string(),
            shared_conversation_id: "conv-123".to_string(),
        };

        assert_eq!(st.id(), "ses-disc");
        assert!(st.is_dual());
        assert_eq!(st.shared_conversation_id(), Some("conv-123"));
        assert_eq!(st.reflection_id(), Some("ses-exec"));
        assert_eq!(st.agent_role(), AgentRole::Discriminator);
    }

    #[test]
    fn test_shared_conversation_creation() {
        let conv = SharedConversation::new(
            "Implement fibonacci".to_string(),
            "ses-exec".to_string(),
            "ses-disc".to_string(),
        );

        assert_eq!(conv.task, "Implement fibonacci");
        assert_eq!(conv.executor_session_id, "ses-exec");
        assert_eq!(conv.discriminator_session_id, "ses-disc");
        assert!(conv.messages.is_empty());
    }
}
