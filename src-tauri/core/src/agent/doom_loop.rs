//! Doom loop detection - prevents agents from calling the same tool repeatedly
//! Based on opencode/packages/opencode/src/session/prompt.ts:1233-1260

use crate::types::Part;

const DOOM_LOOP_THRESHOLD: usize = 3;

#[derive(Debug, Clone)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub args: serde_json::Value,
}

pub struct DoomLoopDetector;

impl DoomLoopDetector {
    /// Check if the last N tool calls are identical (doom loop)
    ///
    /// Returns Ok(()) if no doom loop detected
    /// Returns Err(warning_message) if doom loop detected
    pub fn check(parts: &[Part]) -> Result<(), String> {
        // Get the last DOOM_LOOP_THRESHOLD parts
        if parts.len() < DOOM_LOOP_THRESHOLD {
            return Ok(());
        }

        let recent_parts = &parts[parts.len() - DOOM_LOOP_THRESHOLD..];

        // Extract tool call records from parts
        let tool_calls: Vec<Option<ToolCallRecord>> = recent_parts
            .iter()
            .map(|part| Self::extract_tool_call(part))
            .collect();

        // Check if all are Some and identical
        if tool_calls.iter().all(|tc| tc.is_some()) {
            let first = tool_calls[0].as_ref().unwrap();

            let all_identical = tool_calls.iter().all(|tc| {
                if let Some(call) = tc {
                    call.tool_name == first.tool_name && call.args == first.args
                } else {
                    false
                }
            });

            if all_identical {
                return Err(format!(
                    "⚠️  DOOM LOOP DETECTED: Tool '{}' called {} times with identical arguments. This usually indicates the agent is stuck.",
                    first.tool_name,
                    DOOM_LOOP_THRESHOLD
                ));
            }
        }

        Ok(())
    }

    /// Extract tool call information from a part
    fn extract_tool_call(part: &Part) -> Option<ToolCallRecord> {
        match part {
            Part::Tool { tool, state, .. } => {
                // Get input from state and check if not pending
                match state {
                    crate::types::ToolState::Pending { .. } => None,
                    crate::types::ToolState::Running { input, .. }
                    | crate::types::ToolState::Completed { input, .. }
                    | crate::types::ToolState::Error { input, .. } => Some(ToolCallRecord {
                        tool_name: tool.clone(),
                        args: input.clone(),
                    }),
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tool_part(tool_name: &str, args: serde_json::Value, status: &str) -> Part {
        use crate::types::{ToolState, ToolTime};

        let state = match status {
            "pending" => ToolState::Pending {
                input: args.clone(),
                raw: serde_json::to_string(&args).unwrap(),
            },
            "completed" => ToolState::Completed {
                input: args.clone(),
                output: "success".to_string(),
                title: "Test".to_string(),
                time: ToolTime {
                    start: 0,
                    end: Some(100),
                },
            },
            _ => panic!("Unsupported status in test"),
        };

        Part::Tool {
            id: format!("part-{}", uuid::Uuid::new_v4()),
            session_id: "session-1".to_string(),
            message_id: "msg-1".to_string(),
            call_id: format!("call-{}", uuid::Uuid::new_v4()),
            tool: tool_name.to_string(),
            state,
        }
    }

    #[test]
    fn test_no_doom_loop_with_few_parts() {
        let parts = vec![
            make_tool_part("read", serde_json::json!({"path": "file.rs"}), "completed"),
            make_tool_part("read", serde_json::json!({"path": "file.rs"}), "completed"),
        ];

        assert!(DoomLoopDetector::check(&parts).is_ok());
    }

    #[test]
    fn test_no_doom_loop_with_different_tools() {
        let parts = vec![
            make_tool_part("read", serde_json::json!({"path": "file.rs"}), "completed"),
            make_tool_part("write", serde_json::json!({"path": "file.rs"}), "completed"),
            make_tool_part("bash", serde_json::json!({"command": "ls"}), "completed"),
        ];

        assert!(DoomLoopDetector::check(&parts).is_ok());
    }

    #[test]
    fn test_no_doom_loop_with_different_args() {
        let parts = vec![
            make_tool_part("read", serde_json::json!({"path": "file1.rs"}), "completed"),
            make_tool_part("read", serde_json::json!({"path": "file2.rs"}), "completed"),
            make_tool_part("read", serde_json::json!({"path": "file3.rs"}), "completed"),
        ];

        assert!(DoomLoopDetector::check(&parts).is_ok());
    }

    #[test]
    fn test_doom_loop_detected() {
        let args = serde_json::json!({"path": "file.rs"});
        let parts = vec![
            make_tool_part("read", args.clone(), "completed"),
            make_tool_part("read", args.clone(), "completed"),
            make_tool_part("read", args.clone(), "completed"),
        ];

        let result = DoomLoopDetector::check(&parts);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("DOOM LOOP DETECTED"));
        assert!(err.contains("read"));
    }

    #[test]
    fn test_doom_loop_ignores_pending() {
        let args = serde_json::json!({"path": "file.rs"});
        let parts = vec![
            make_tool_part("read", args.clone(), "pending"),
            make_tool_part("read", args.clone(), "completed"),
            make_tool_part("read", args.clone(), "completed"),
        ];

        // Should not detect doom loop because first one is pending
        assert!(DoomLoopDetector::check(&parts).is_ok());
    }

    #[test]
    fn test_doom_loop_with_text_parts_mixed() {
        let args = serde_json::json!({"path": "file.rs"});
        let parts = vec![
            Part::Text {
                id: "text-1".to_string(),
                session_id: "session-1".to_string(),
                message_id: "msg-1".to_string(),
                text: "Some text".to_string(),
            },
            make_tool_part("read", args.clone(), "completed"),
            make_tool_part("read", args.clone(), "completed"),
            make_tool_part("read", args.clone(), "completed"),
        ];

        // Should detect doom loop in the last 3 parts
        let result = DoomLoopDetector::check(&parts);
        assert!(result.is_err());
    }
}
