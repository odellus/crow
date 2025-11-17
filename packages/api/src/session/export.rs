//! Session export to markdown - matching OpenCode format exactly

#[cfg(feature = "server")]
use super::store::{MessageWithParts, SessionStore};
#[cfg(feature = "server")]
use crate::types::{Message, Part, ToolState};
#[cfg(feature = "server")]
use std::path::{Path, PathBuf};

pub struct SessionExport;

#[cfg(feature = "server")]
impl SessionExport {
    /// Export session to markdown format (matches OpenCode exactly)
    pub fn to_markdown(session_store: &SessionStore, session_id: &str) -> Result<String, String> {
        let session = session_store.get(session_id)?;
        let messages = session_store.get_messages(session_id)?;

        let mut markdown = format!("# {}\n\n", session.title);
        markdown.push_str(&format!("**Session ID:** `{}`\n", session_id));
        markdown.push_str(&format!(
            "**Created:** {}\n",
            format_iso8601(session.time.created)
        ));
        markdown.push_str(&format!("**Project:** {}\n\n", session.directory));

        // Add dual-pair metadata if present
        if let Some(metadata) = &session.metadata {
            if metadata.get("dualPairAgent").is_some() || metadata.get("dualPairStep").is_some() {
                markdown.push_str("## Dual-Pair Session\n\n");

                if metadata
                    .get("dualPairComplete")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    markdown.push_str("✅ **Status:** Completed\n");
                    if let Some(step) = metadata.get("dualPairStep") {
                        markdown.push_str(&format!("**Steps:** {}\n", step));
                    }
                    if let Some(summary) =
                        metadata.get("completionSummary").and_then(|v| v.as_str())
                    {
                        markdown.push_str(&format!("**Summary:** {}\n", summary));
                    }
                } else {
                    markdown.push_str("🔄 **Status:** In Progress\n");
                }

                markdown.push_str("\n");
            }
        }

        markdown.push_str("---\n\n");

        // Export each message
        for msg in &messages {
            markdown.push_str(&Self::format_message(msg));
            markdown.push_str("\n---\n\n");
        }

        Ok(markdown)
    }

    /// Format a single message as markdown (matches OpenCode formatMessage)
    fn format_message(msg: &MessageWithParts) -> String {
        let role = match &msg.info {
            Message::User { .. } => "user",
            Message::Assistant { .. } => "assistant",
        };

        let agent = match &msg.info {
            Message::User { .. } => "user".to_string(),
            Message::Assistant { mode, .. } => mode.clone(),
        };

        let timestamp = match &msg.info {
            Message::User { time, .. } => time.created,
            Message::Assistant { time, .. } => time.created,
        };

        let mut markdown = format!(
            "## {} ({})\n\n",
            if role == "user" {
                "👤 User"
            } else {
                "🤖 Assistant"
            },
            agent
        );
        markdown.push_str(&format!("*{}*\n\n", format_iso8601(timestamp)));

        // Token info for assistant messages
        if let Message::Assistant { tokens, .. } = &msg.info {
            markdown.push_str(&format!(
                "**Tokens:** {} in / {} out",
                tokens.input, tokens.output
            ));
            if tokens.cache.read > 0 {
                markdown.push_str(&format!(" ({} cached)", tokens.cache.read));
            }
            markdown.push_str("\n\n");
        }

        // Separate text and tool parts
        let text_parts: Vec<&Part> = msg
            .parts
            .iter()
            .filter(|p| matches!(p, Part::Text { .. } | Part::Thinking { .. }))
            .collect();

        let tool_parts: Vec<&Part> = msg
            .parts
            .iter()
            .filter(|p| matches!(p, Part::Tool { .. }))
            .collect();

        // Render text content
        for part in text_parts {
            match part {
                Part::Text { text, .. } => {
                    markdown.push_str(&format!("{}\n\n", text));
                }
                Part::Thinking { text, .. } => {
                    // Include thinking as collapsed section
                    markdown.push_str(&format!(
                        "<details>\n<summary>💭 Thinking</summary>\n\n{}\n</details>\n\n",
                        text
                    ));
                }
                _ => {}
            }
        }

        // Render tool calls
        if !tool_parts.is_empty() {
            markdown.push_str(&Self::render_tools_as_markdown(&tool_parts));
            markdown.push_str("\n\n");
        }

        markdown
    }

    /// Render tool calls as markdown (matches OpenCode ToolRenderer)
    fn render_tools_as_markdown(tool_parts: &[&Part]) -> String {
        if tool_parts.is_empty() {
            return String::new();
        }

        let mut markdown = "\n\n## Tools Used\n\n".to_string();

        for part in tool_parts {
            if let Part::Tool { tool, state, .. } = part {
                markdown.push_str(&format!("### {}\n\n", tool));

                // Input (available in all states except pending)
                match state {
                    ToolState::Pending { input, .. } => {
                        // Show raw input for pending
                        markdown.push_str(&format!(
                            "**Input:**\n```json\n{}\n```\n\n",
                            serde_json::to_string_pretty(input).unwrap_or_default()
                        ));
                    }
                    ToolState::Running { input, title, .. } => {
                        markdown.push_str(&format!(
                            "**Input:**\n```json\n{}\n```\n\n",
                            serde_json::to_string_pretty(input).unwrap_or_default()
                        ));
                        if let Some(title) = title {
                            markdown.push_str(&format!("**Title:** {}\n\n", title));
                        }
                        markdown.push_str("**Status:** Running...\n\n");
                    }
                    ToolState::Completed {
                        input,
                        output,
                        title,
                        ..
                    } => {
                        markdown.push_str(&format!(
                            "**Input:**\n```json\n{}\n```\n\n",
                            serde_json::to_string_pretty(input).unwrap_or_default()
                        ));
                        markdown.push_str(&format!("**Output:**\n```\n{}\n```\n\n", output));
                        if !title.is_empty() {
                            markdown.push_str(&format!("**Title:** {}\n\n", title));
                        }
                    }
                    ToolState::Error { input, error, .. } => {
                        markdown.push_str(&format!(
                            "**Input:**\n```json\n{}\n```\n\n",
                            serde_json::to_string_pretty(input).unwrap_or_default()
                        ));
                        markdown.push_str(&format!("**Error:**\n```\n{}\n```\n\n", error));
                    }
                }
            }
        }

        markdown.trim().to_string()
    }

    /// Write session export to file
    pub fn write_to_file(
        session_store: &SessionStore,
        session_id: &str,
        path: &Path,
    ) -> Result<(), String> {
        let markdown = Self::to_markdown(session_store, session_id)?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create export directory: {}", e))?;
        }

        std::fs::write(path, markdown)
            .map_err(|e| format!("Failed to write markdown file: {}", e))?;

        Ok(())
    }

    /// Get default export path for a session (matches OpenCode: .crow/sessions/{id}.md)
    pub fn get_default_path(session_id: &str, base_dir: &Path) -> PathBuf {
        base_dir
            .join(".crow")
            .join("sessions")
            .join(format!("{}.md", session_id))
    }

    /// Stream session to file (export after every message)
    /// This is called after add_message() to maintain real-time exports
    pub fn stream_to_file(
        session_store: &SessionStore,
        session_id: &str,
        base_dir: &Path,
    ) -> Result<(), String> {
        let export_path = Self::get_default_path(session_id, base_dir);
        Self::write_to_file(session_store, session_id, &export_path)
    }
}

/// Format timestamp as ISO 8601 (matches OpenCode's toISOString())
#[cfg(feature = "server")]
fn format_iso8601(ms: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};

    let duration = Duration::from_millis(ms);
    let _datetime = UNIX_EPOCH + duration;

    // Use chrono-like formatting
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();

    // Calculate components
    let days_since_epoch = secs / 86400;
    let secs_today = secs % 86400;

    let hours = secs_today / 3600;
    let minutes = (secs_today % 3600) / 60;
    let seconds = secs_today % 60;
    let millis = nanos / 1_000_000;

    // Simplified date calculation (good enough for recent dates)
    let mut year = 1970;
    let mut days = days_since_epoch;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let days_in_months = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for &days_in_month in &days_in_months {
        if days < days_in_month {
            break;
        }
        days -= days_in_month;
        month += 1;
    }

    let day = days + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, month, day, hours, minutes, seconds, millis
    )
}

#[cfg(feature = "server")]
fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "server")]
    fn test_iso8601_formatting() {
        // 2024-01-01 00:00:00.000 UTC = 1704067200000ms
        let timestamp = 1704067200000;
        let formatted = format_iso8601(timestamp);
        assert_eq!(formatted, "2024-01-01T00:00:00.000Z");
    }

    #[test]
    #[cfg(feature = "server")]
    fn test_default_path() {
        let path = SessionExport::get_default_path("ses-123", Path::new("/tmp"));
        assert_eq!(path, PathBuf::from("/tmp/.crow/sessions/ses-123.md"));
    }
}
