//! Perspective transformation for dual-agent sessions
//! Transforms raw messages into the correct view for executor vs discriminator

use super::dual::{AgentRole, RawMessage};
use crate::types::{CacheTokens, Message, MessagePath, MessageTime, Part, TokenUsage};

/// Transform raw messages for executor's perspective
///
/// Executor sees:
/// - Their own messages as "assistant"
/// - Discriminator's messages as "user" (feedback)
/// - Original user message as "user"
pub fn transform_for_executor(raw_messages: &[RawMessage], session_id: &str) -> Vec<Message> {
    raw_messages
        .iter()
        .map(|raw| match raw.agent {
            AgentRole::Executor => {
                // My messages appear as "assistant"
                Message::Assistant {
                    id: raw.id.clone(),
                    session_id: session_id.to_string(),
                    parent_id: String::new(), // TODO: track parent properly
                    model_id: "executor-model".to_string(),
                    provider_id: "default".to_string(),
                    mode: "executor".to_string(),
                    time: MessageTime {
                        created: raw.timestamp,
                        completed: Some(raw.timestamp),
                    },
                    path: MessagePath {
                        cwd: ".".to_string(),
                        root: ".".to_string(),
                    },
                    cost: 0.0,
                    tokens: TokenUsage {
                        input: 0,
                        output: 0,
                        reasoning: 0,
                        cache: CacheTokens { read: 0, write: 0 },
                    },
                    error: None,
                    summary: None,
                    metadata: None,
                }
            }
            AgentRole::Discriminator => {
                // Discriminator's messages appear as "user" (feedback)
                Message::User {
                    id: raw.id.clone(),
                    session_id: session_id.to_string(),
                    time: MessageTime {
                        created: raw.timestamp,
                        completed: Some(raw.timestamp),
                    },
                    summary: None,
                    metadata: None,
                }
            }
            AgentRole::User => {
                // Original user message
                Message::User {
                    id: raw.id.clone(),
                    session_id: session_id.to_string(),
                    time: MessageTime {
                        created: raw.timestamp,
                        completed: Some(raw.timestamp),
                    },
                    summary: None,
                    metadata: None,
                }
            }
        })
        .collect()
}

/// Transform raw messages for discriminator's perspective
///
/// Discriminator sees:
/// - Their own messages as "assistant"
/// - Executor's work as "user" (showing what was done)
/// - Original user message as "user"
pub fn transform_for_discriminator(raw_messages: &[RawMessage], session_id: &str) -> Vec<Message> {
    raw_messages
        .iter()
        .map(|raw| match raw.agent {
            AgentRole::Discriminator => {
                // My messages appear as "assistant"
                Message::Assistant {
                    id: raw.id.clone(),
                    session_id: session_id.to_string(),
                    parent_id: String::new(),
                    model_id: "discriminator-model".to_string(),
                    provider_id: "default".to_string(),
                    mode: "discriminator".to_string(),
                    time: MessageTime {
                        created: raw.timestamp,
                        completed: Some(raw.timestamp),
                    },
                    path: MessagePath {
                        cwd: ".".to_string(),
                        root: ".".to_string(),
                    },
                    cost: 0.0,
                    tokens: TokenUsage {
                        input: 0,
                        output: 0,
                        reasoning: 0,
                        cache: CacheTokens { read: 0, write: 0 },
                    },
                    error: None,
                    summary: None,
                    metadata: None,
                }
            }
            AgentRole::Executor => {
                // Executor's work appears as "user" (showing what they did)
                Message::User {
                    id: raw.id.clone(),
                    session_id: session_id.to_string(),
                    time: MessageTime {
                        created: raw.timestamp,
                        completed: Some(raw.timestamp),
                    },
                    summary: None,
                    metadata: None,
                }
            }
            AgentRole::User => {
                // Original user message
                Message::User {
                    id: raw.id.clone(),
                    session_id: session_id.to_string(),
                    time: MessageTime {
                        created: raw.timestamp,
                        completed: Some(raw.timestamp),
                    },
                    summary: None,
                    metadata: None,
                }
            }
        })
        .collect()
}

/// Render a message with its parts for display
/// Used when showing executor's work to discriminator
pub fn render_message_with_tools(raw: &RawMessage) -> String {
    let mut output = String::new();

    for part in &raw.parts {
        match part {
            Part::Text { text, .. } => {
                output.push_str(text);
                output.push_str("\n\n");
            }
            Part::Thinking { text, .. } => {
                output.push_str("*Thinking: ");
                output.push_str(text);
                output.push_str("*\n\n");
            }
            Part::Tool { tool, state, .. } => {
                use crate::types::ToolState;
                output.push_str(&format!("### Tool: {}\n", tool));

                match state {
                    ToolState::Pending { input, .. } => {
                        output.push_str(&format!("**Status:** Pending\n**Input:** {}\n\n", input));
                    }
                    ToolState::Running { input, title, .. } => {
                        output.push_str(&format!("**Status:** Running\n"));
                        if let Some(t) = title {
                            output.push_str(&format!("**Title:** {}\n", t));
                        }
                        output.push_str(&format!("**Input:** {}\n\n", input));
                    }
                    ToolState::Completed {
                        input,
                        output: result,
                        title,
                        ..
                    } => {
                        output.push_str(&format!("**Status:** Completed\n"));
                        output.push_str(&format!("**Title:** {}\n", title));
                        output.push_str(&format!("**Input:** {}\n", input));
                        output.push_str(&format!("**Output:** {}\n\n", result));
                    }
                    ToolState::Error { input, error, .. } => {
                        output.push_str(&format!("**Status:** Error\n"));
                        output.push_str(&format!("**Input:** {}\n", input));
                        output.push_str(&format!("**Error:** {}\n\n", error));
                    }
                }
            }
            Part::File { filename, url, .. } => {
                output.push_str(&format!(
                    "**File:** {}\n",
                    filename.as_ref().unwrap_or(&"unknown".to_string())
                ));
                output.push_str(&format!("**URL:** {}\n\n", url));
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_raw_message(agent: AgentRole, text: &str) -> RawMessage {
        RawMessage {
            id: format!("msg-{}", uuid::Uuid::new_v4()),
            agent,
            parts: vec![Part::Text {
                id: format!("part-{}", uuid::Uuid::new_v4()),
                session_id: "ses-test".to_string(),
                message_id: "msg-test".to_string(),
                text: text.to_string(),
            }],
            timestamp: 1000,
        }
    }

    #[test]
    fn test_executor_perspective() {
        let raw_messages = vec![
            create_test_raw_message(AgentRole::User, "Do task"),
            create_test_raw_message(AgentRole::Executor, "Working on it"),
            create_test_raw_message(AgentRole::Discriminator, "Looks good"),
        ];

        let messages = transform_for_executor(&raw_messages, "ses-exec");

        assert_eq!(messages.len(), 3);

        // User message stays as user
        assert!(matches!(messages[0], Message::User { .. }));

        // Executor's message becomes assistant
        assert!(matches!(messages[1], Message::Assistant { .. }));

        // Discriminator's message becomes user (feedback)
        assert!(matches!(messages[2], Message::User { .. }));
    }

    #[test]
    fn test_discriminator_perspective() {
        let raw_messages = vec![
            create_test_raw_message(AgentRole::User, "Do task"),
            create_test_raw_message(AgentRole::Executor, "Working on it"),
            create_test_raw_message(AgentRole::Discriminator, "Looks good"),
        ];

        let messages = transform_for_discriminator(&raw_messages, "ses-disc");

        assert_eq!(messages.len(), 3);

        // User message stays as user
        assert!(matches!(messages[0], Message::User { .. }));

        // Executor's message becomes user (showing work)
        assert!(matches!(messages[1], Message::User { .. }));

        // Discriminator's message becomes assistant
        assert!(matches!(messages[2], Message::Assistant { .. }));
    }
}
