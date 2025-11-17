//! Session export to markdown

use super::store::SessionStore;
use crate::types::Part;
use std::path::Path;

pub struct SessionExport;

impl SessionExport {
    /// Export session to markdown
    pub fn to_markdown(session_store: &SessionStore, session_id: &str) -> Result<String, String> {
        let session = session_store.get(session_id)?;
        let messages = session_store.get_messages(session_id)?;

        let mut md = format!("# {}\n\n", session.title);
        md.push_str(&format!("**Session ID:** `{}`\n", session.id));
        md.push_str(&format!(
            "**Created:** {}\n",
            format_timestamp(session.time.created)
        ));
        md.push_str(&format!("**Directory:** {}\n\n", session.directory));

        md.push_str("---\n\n");

        // Export each message
        for (i, msg) in messages.iter().enumerate() {
            md.push_str(&format!("## Message {}\n\n", i + 1));

            let role = msg.info.role();
            let created = msg.info.created();

            md.push_str(&format!(
                "**Role:** {} | **Time:** {}\n\n",
                role,
                format_timestamp(created)
            ));

            // Token usage for assistant
            if role == "assistant" {
                if let Some(tokens) = msg.info.tokens() {
                    md.push_str(&format!(
                        "**Tokens:** {} in / {} out\n\n",
                        tokens.input, tokens.output
                    ));
                }
            }

            // Render parts
            for part in &msg.parts {
                match part {
                    Part::Text { text, .. } => {
                        md.push_str(&format!("{}\n\n", text));
                    }
                    Part::Thinking { text, .. } => {
                        md.push_str(&format!(
                            "<details>\n<summary>💭 Thinking</summary>\n\n{}\n</details>\n\n",
                            text
                        ));
                    }
                    Part::Tool {
                        tool,
                        state,
                        call_id,
                        ..
                    } => {
                        md.push_str(&format!("🔧 **{}** ({})\n\n", tool, call_id));
                        md.push_str("```json\n");
                        md.push_str(&serde_json::to_string_pretty(&state).unwrap_or_default());
                        md.push_str("\n```\n\n");
                    }
                    Part::File { name, .. } => {
                        md.push_str(&format!("📄 **File:** {}\n\n", name));
                    }
                    Part::Image { .. } => {
                        md.push_str("🖼️ **Image**\n\n");
                    }
                }
            }

            md.push_str("---\n\n");
        }

        Ok(md)
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
                .map_err(|e| format!("Failed to create parent dir: {}", e))?;
        }

        std::fs::write(path, markdown).map_err(|e| format!("Failed to write markdown: {}", e))?;

        Ok(())
    }
}

fn format_timestamp(ms: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};

    let duration = Duration::from_millis(ms);
    let datetime = UNIX_EPOCH + duration;

    // Format as ISO 8601
    match datetime.duration_since(UNIX_EPOCH) {
        Ok(d) => {
            let secs = d.as_secs();
            let days = secs / 86400;
            let hours = (secs % 86400) / 3600;
            let minutes = (secs % 3600) / 60;
            let seconds = secs % 60;

            // Simple ISO-like format
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                1970 + (days / 365),
                ((days % 365) / 30) + 1,
                (days % 30) + 1,
                hours,
                minutes,
                seconds
            )
        }
        Err(_) => "Unknown".to_string(),
    }
}

// Helper trait to get info from Message enum
trait MessageInfo {
    fn role(&self) -> &str;
    fn created(&self) -> u64;
    fn tokens(&self) -> Option<&crate::types::TokenUsage>;
}

impl MessageInfo for crate::types::Message {
    fn role(&self) -> &str {
        match self {
            crate::types::Message::User { .. } => "user",
            crate::types::Message::Assistant { .. } => "assistant",
        }
    }

    fn created(&self) -> u64 {
        match self {
            crate::types::Message::User { time, .. } => time.created,
            crate::types::Message::Assistant { time, .. } => time.created,
        }
    }

    fn tokens(&self) -> Option<&crate::types::TokenUsage> {
        match self {
            crate::types::Message::Assistant { tokens, .. } => Some(tokens),
            _ => None,
        }
    }
}
