//! Crow server-local storage (.crow directory)
//! Simple JSON file storage matching OpenCode's storage.ts

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
    }
}

/// Crow storage in server's working directory
pub struct CrowStorage {
    root: PathBuf, // {server_cwd}/.crow
}

impl CrowStorage {
    /// Create or open .crow directory in server's CWD
    pub fn new() -> Result<Self, String> {
        let cwd = std::env::current_dir().map_err(|e| format!("Failed to get CWD: {}", e))?;

        let root = cwd.join(".crow");

        // Create directory structure
        std::fs::create_dir_all(&root).map_err(|e| format!("Failed to create .crow: {}", e))?;

        std::fs::create_dir_all(root.join("sessions"))
            .map_err(|e| format!("Failed to create sessions dir: {}", e))?;

        std::fs::create_dir_all(root.join("conversations"))
            .map_err(|e| format!("Failed to create conversations dir: {}", e))?;

        std::fs::create_dir_all(root.join("logs"))
            .map_err(|e| format!("Failed to create logs dir: {}", e))?;

        std::fs::create_dir_all(root.join("storage/session"))
            .map_err(|e| format!("Failed to create storage/session: {}", e))?;

        std::fs::create_dir_all(root.join("storage/message"))
            .map_err(|e| format!("Failed to create storage/message: {}", e))?;

        std::fs::create_dir_all(root.join("storage/part"))
            .map_err(|e| format!("Failed to create storage/part: {}", e))?;

        // Create .gitignore if it doesn't exist
        let gitignore_path = root.join(".gitignore");
        if !gitignore_path.exists() {
            std::fs::write(&gitignore_path, "*\n!.gitignore\n")
                .map_err(|e| format!("Failed to create .gitignore: {}", e))?;
        }

        Ok(Self { root })
    }

    /// Initialize storage (async for compatibility)
    pub async fn init(&self) -> Result<(), String> {
        // Already initialized in new(), but this matches OpenCode API
        Ok(())
    }

    /// Initialize storage synchronously
    pub fn init_sync(&self) -> Result<(), String> {
        // Already initialized in new()
        Ok(())
    }

    /// List all sessions (loads full session objects)
    pub async fn list_sessions(&self) -> Result<Vec<Session>, String> {
        let session_dir = self.root.join("storage/session");

        let mut sessions = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&session_dir) {
            let mut session_files: Vec<_> = entries
                .flatten()
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .map_or(false, |s| s.ends_with(".json"))
                })
                .collect();

            session_files.sort_by_key(|e| e.file_name());

            for entry in session_files {
                let path = entry.path();
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<Session>(&content) {
                        sessions.push(session);
                    }
                }
            }
        }

        Ok(sessions)
    }

    /// List all sessions synchronously
    pub fn list_sessions_sync(&self) -> Result<Vec<Session>, String> {
        let session_dir = self.root.join("storage/session");

        let mut sessions = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&session_dir) {
            let mut session_files: Vec<_> = entries
                .flatten()
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .map_or(false, |s| s.ends_with(".json"))
                })
                .collect();

            session_files.sort_by_key(|e| e.file_name());

            for entry in session_files {
                let path = entry.path();
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<Session>(&content) {
                        sessions.push(session);
                    }
                }
            }
        }

        Ok(sessions)
    }

    /// Load messages for a session
    pub async fn load_messages(&self, session_id: &str) -> Result<Vec<MessageWithParts>, String> {
        let message_dir = self.root.join("storage/message").join(session_id);

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

            // Sort by filename (message IDs are sortable)
            message_files.sort_by_key(|e| e.file_name());

            for entry in message_files {
                let path = entry.path();
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(message_info) = serde_json::from_str::<Message>(&content) {
                        // Load parts for this message
                        let msg_id = message_id(&message_info).to_string();
                        let parts = self.load_parts(&msg_id).await?;

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

    /// Load messages for a session synchronously
    pub fn load_messages_sync(&self, session_id: &str) -> Result<Vec<MessageWithParts>, String> {
        let message_dir = self.root.join("storage/message").join(session_id);

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

    /// Load parts for a message
    async fn load_parts(&self, message_id: &str) -> Result<Vec<Part>, String> {
        let part_dir = self.root.join("storage/part").join(message_id);

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

            // Sort by filename (part IDs are sortable)
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

    /// Load parts for a message synchronously
    fn load_parts_sync(&self, message_id: &str) -> Result<Vec<Part>, String> {
        let part_dir = self.root.join("storage/part").join(message_id);

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
        let path = self
            .root
            .join("storage/session")
            .join(format!("{}.json", session.id));

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
        let message_dir = self.root.join("storage/message").join(session_id);
        std::fs::create_dir_all(&message_dir)
            .map_err(|e| format!("Failed to create message dir: {}", e))?;

        let message_path = message_dir.join(format!("{}.json", msg_id));
        let message_json = serde_json::to_string_pretty(&message.info)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        std::fs::write(&message_path, message_json)
            .map_err(|e| format!("Failed to write message file: {}", e))?;

        // Save parts
        let part_dir = self.root.join("storage/part").join(msg_id);
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

    /// Get path to session export markdown
    pub fn session_export_path(&self, session_id: &str) -> PathBuf {
        self.root
            .join("sessions")
            .join(format!("{}.md", session_id))
    }

    /// Get path to conversation directory
    pub fn conversation_dir(&self, conversation_id: &str) -> PathBuf {
        self.root.join("conversations").join(conversation_id)
    }

    /// Get root .crow directory
    pub fn root(&self) -> &Path {
        &self.root
    }
}
