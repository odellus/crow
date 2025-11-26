//! Crow storage - XDG-based JSON file storage matching OpenCode's storage.ts
//!
//! Storage layout (matching OpenCode):
//!   ~/.local/share/crow/storage/
//!     session/{projectID}/{sessionID}.json
//!     message/{sessionID}/{messageID}.json
//!     part/{messageID}/{partID}.json
//!     project/{projectID}.json

use crate::global::GlobalPaths;
use crate::session::store::MessageWithParts;
use crate::types::{Message, Part, Session};
use std::path::{Path, PathBuf};

/// Helper to get ID from Message enum
fn message_id(msg: &Message) -> &str {
    match msg {
        Message::User { id, .. } => id,
        Message::Assistant { id, .. } => id,
    }
}

/// Helper to get session_id from Message enum
fn message_session_id(msg: &Message) -> &str {
    match msg {
        Message::User { session_id, .. } => session_id,
        Message::Assistant { session_id, .. } => session_id,
    }
}

/// Helper to get ID from Part enum
fn part_id(part: &Part) -> &str {
    match part {
        Part::Text { id, .. } => id,
        Part::Thinking { id, .. } => id,
        Part::Tool { id, .. } => id,
        Part::File { id, .. } => id,
        Part::Patch { id, .. } => id,
    }
}

/// Crow storage using XDG directories (like OpenCode)
pub struct CrowStorage {
    /// Root storage directory: ~/.local/share/crow/storage
    root: PathBuf,
}

impl CrowStorage {
    /// Create storage using XDG data directory
    pub fn new() -> Result<Self, String> {
        let paths = GlobalPaths::new();
        let root = paths.data.join("storage");

        // Create directory structure
        std::fs::create_dir_all(&root)
            .map_err(|e| format!("Failed to create storage dir: {}", e))?;

        std::fs::create_dir_all(root.join("session"))
            .map_err(|e| format!("Failed to create session dir: {}", e))?;

        std::fs::create_dir_all(root.join("message"))
            .map_err(|e| format!("Failed to create message dir: {}", e))?;

        std::fs::create_dir_all(root.join("part"))
            .map_err(|e| format!("Failed to create part dir: {}", e))?;

        std::fs::create_dir_all(root.join("project"))
            .map_err(|e| format!("Failed to create project dir: {}", e))?;

        Ok(Self { root })
    }

    /// Initialize storage (async for compatibility)
    pub async fn init(&self) -> Result<(), String> {
        Ok(())
    }

    /// Initialize storage synchronously
    pub fn init_sync(&self) -> Result<(), String> {
        Ok(())
    }

    /// Get project ID for a directory (using git root commit hash like OpenCode)
    pub fn project_id(directory: &Path) -> String {
        // Try to get git root commit hash
        if let Ok(output) = std::process::Command::new("git")
            .args(["rev-list", "--max-parents=0", "--all"])
            .current_dir(directory)
            .output()
        {
            if output.status.success() {
                let commits: Vec<&str> = std::str::from_utf8(&output.stdout)
                    .unwrap_or("")
                    .lines()
                    .collect();
                if let Some(first) = commits.first() {
                    return first.trim().to_string();
                }
            }
        }

        // Fallback: hash the directory path
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        directory.hash(&mut hasher);
        format!("proj-{:x}", hasher.finish())
    }

    /// List all sessions for a project
    pub async fn list_sessions(&self) -> Result<Vec<Session>, String> {
        self.list_sessions_sync()
    }

    /// List all sessions synchronously (across all projects)
    pub fn list_sessions_sync(&self) -> Result<Vec<Session>, String> {
        let session_dir = self.root.join("session");
        let mut sessions = Vec::new();

        if !session_dir.exists() {
            return Ok(sessions);
        }

        // Iterate through project directories
        if let Ok(project_entries) = std::fs::read_dir(&session_dir) {
            for project_entry in project_entries.flatten() {
                if !project_entry
                    .file_type()
                    .map(|t| t.is_dir())
                    .unwrap_or(false)
                {
                    continue;
                }

                let project_dir = project_entry.path();
                if let Ok(session_entries) = std::fs::read_dir(&project_dir) {
                    for entry in session_entries.flatten() {
                        let path = entry.path();
                        if path.extension().map_or(false, |e| e == "json") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(session) = serde_json::from_str::<Session>(&content) {
                                    sessions.push(session);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort by creation time, newest first
        sessions.sort_by(|a, b| b.time.created.cmp(&a.time.created));
        Ok(sessions)
    }

    /// List sessions for a specific project
    pub fn list_sessions_for_project(&self, project_id: &str) -> Result<Vec<Session>, String> {
        let session_dir = self.root.join("session").join(project_id);
        let mut sessions = Vec::new();

        if !session_dir.exists() {
            return Ok(sessions);
        }

        if let Ok(entries) = std::fs::read_dir(&session_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(session) = serde_json::from_str::<Session>(&content) {
                            sessions.push(session);
                        }
                    }
                }
            }
        }

        sessions.sort_by(|a, b| b.time.created.cmp(&a.time.created));
        Ok(sessions)
    }

    /// Load messages for a session
    pub async fn load_messages(&self, session_id: &str) -> Result<Vec<MessageWithParts>, String> {
        self.load_messages_sync(session_id)
    }

    /// Load messages for a session synchronously
    pub fn load_messages_sync(&self, session_id: &str) -> Result<Vec<MessageWithParts>, String> {
        let message_dir = self.root.join("message").join(session_id);
        let mut messages = Vec::new();

        if !message_dir.exists() {
            return Ok(messages);
        }

        if let Ok(entries) = std::fs::read_dir(&message_dir) {
            let mut message_files: Vec<_> = entries
                .flatten()
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .map_or(false, |s| s.ends_with(".json"))
                })
                .collect();

            message_files.sort_by_key(|e| e.file_name());

            for entry in message_files {
                let path = entry.path();
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(message_info) = serde_json::from_str::<Message>(&content) {
                        let msg_id = message_id(&message_info).to_string();
                        let parts = self.load_parts_sync(&msg_id)?;

                        messages.push(MessageWithParts {
                            info: message_info,
                            parts,
                        });
                    }
                }
            }
        }

        Ok(messages)
    }

    /// Load parts for a message synchronously
    fn load_parts_sync(&self, message_id: &str) -> Result<Vec<Part>, String> {
        let part_dir = self.root.join("part").join(message_id);
        let mut parts = Vec::new();

        if !part_dir.exists() {
            return Ok(parts);
        }

        if let Ok(entries) = std::fs::read_dir(&part_dir) {
            let mut part_files: Vec<_> = entries
                .flatten()
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .map_or(false, |s| s.ends_with(".json"))
                })
                .collect();

            part_files.sort_by_key(|e| e.file_name());

            for entry in part_files {
                let path = entry.path();
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(part) = serde_json::from_str::<Part>(&content) {
                        parts.push(part);
                    }
                }
            }
        }

        Ok(parts)
    }

    /// Save session info
    pub async fn save_session(&self, session: &Session) -> Result<(), String> {
        // Create project directory if needed
        let session_dir = self.root.join("session").join(&session.project_id);
        std::fs::create_dir_all(&session_dir)
            .map_err(|e| format!("Failed to create session dir: {}", e))?;

        let path = session_dir.join(format!("{}.json", session.id));

        let json = serde_json::to_string_pretty(session)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;

        std::fs::write(&path, json).map_err(|e| format!("Failed to write session file: {}", e))?;

        Ok(())
    }

    /// Save message with parts
    pub async fn save_message(&self, message: &MessageWithParts) -> Result<(), String> {
        let session_id = message_session_id(&message.info);
        let msg_id = message_id(&message.info);

        // Save message info
        let message_dir = self.root.join("message").join(session_id);
        std::fs::create_dir_all(&message_dir)
            .map_err(|e| format!("Failed to create message dir: {}", e))?;

        let message_path = message_dir.join(format!("{}.json", msg_id));
        let message_json = serde_json::to_string_pretty(&message.info)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        std::fs::write(&message_path, message_json)
            .map_err(|e| format!("Failed to write message file: {}", e))?;

        // Save parts
        let part_dir = self.root.join("part").join(msg_id);
        std::fs::create_dir_all(&part_dir)
            .map_err(|e| format!("Failed to create part dir: {}", e))?;

        for part in &message.parts {
            let p_id = part_id(part);
            let part_path = part_dir.join(format!("{}.json", p_id));
            let part_json = serde_json::to_string_pretty(part)
                .map_err(|e| format!("Failed to serialize part: {}", e))?;

            std::fs::write(&part_path, part_json)
                .map_err(|e| format!("Failed to write part file: {}", e))?;
        }

        Ok(())
    }

    /// Get storage root directory
    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_paths() {
        // Just verify paths are constructed correctly
        let storage = CrowStorage::new().unwrap();
        assert!(storage.root().to_string_lossy().contains("crow"));
        assert!(storage.root().to_string_lossy().contains("storage"));
    }
}
