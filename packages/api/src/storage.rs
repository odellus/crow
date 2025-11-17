//! File-based storage for sessions, messages, and parts
//! Mirrors OpenCode's storage structure at ~/.local/share/crow/storage/

use crate::{
    session::MessageWithParts,
    types::{Message, Part, Session},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

/// Get the storage directory path
pub fn storage_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("crow")
        .join("storage")
}

/// Storage manager for file-based persistence
pub struct Storage {
    base_dir: PathBuf,
}

impl Storage {
    /// Create a new storage instance
    pub fn new() -> Self {
        Self {
            base_dir: storage_dir(),
        }
    }

    /// Initialize storage directories
    pub async fn init(&self) -> Result<(), String> {
        fs::create_dir_all(&self.base_dir)
            .await
            .map_err(|e| format!("Failed to create storage directory: {}", e))?;

        // Create subdirectories
        for subdir in &["session", "message", "part", "session_diff"] {
            fs::create_dir_all(self.base_dir.join(subdir))
                .await
                .map_err(|e| format!("Failed to create {} directory: {}", subdir, e))?;
        }

        Ok(())
    }

    // ========================================================================
    // Session Storage
    // ========================================================================

    /// Save a session to disk
    pub async fn save_session(&self, session: &Session) -> Result<(), String> {
        let project_dir = self.base_dir.join("session").join(&session.project_id);
        fs::create_dir_all(&project_dir)
            .await
            .map_err(|e| format!("Failed to create project directory: {}", e))?;

        let file_path = project_dir.join(format!("{}.json", session.id));
        let json = serde_json::to_string_pretty(session)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;

        fs::write(&file_path, json)
            .await
            .map_err(|e| format!("Failed to write session file: {}", e))?;

        Ok(())
    }

    /// Load a session from disk
    pub async fn load_session(&self, session_id: &str) -> Result<Session, String> {
        // Search all project directories for the session
        let session_dir = self.base_dir.join("session");
        let mut entries = fs::read_dir(&session_dir)
            .await
            .map_err(|e| format!("Failed to read session directory: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Failed to read directory entry: {}", e))?
        {
            if entry
                .file_type()
                .await
                .map_err(|e| format!("Failed to get file type: {}", e))?
                .is_dir()
            {
                let file_path = entry.path().join(format!("{}.json", session_id));
                if file_path.exists() {
                    let content = fs::read_to_string(&file_path)
                        .await
                        .map_err(|e| format!("Failed to read session file: {}", e))?;
                    return serde_json::from_str(&content)
                        .map_err(|e| format!("Failed to parse session: {}", e));
                }
            }
        }

        Err(format!("Session not found: {}", session_id))
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<Session>, String> {
        let mut sessions = Vec::new();
        let session_dir = self.base_dir.join("session");

        if !session_dir.exists() {
            return Ok(sessions);
        }

        let mut entries = fs::read_dir(&session_dir)
            .await
            .map_err(|e| format!("Failed to read session directory: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Failed to read directory entry: {}", e))?
        {
            if entry
                .file_type()
                .await
                .map_err(|e| format!("Failed to get file type: {}", e))?
                .is_dir()
            {
                let project_dir = entry.path();
                let mut project_entries = fs::read_dir(&project_dir)
                    .await
                    .map_err(|e| format!("Failed to read project directory: {}", e))?;

                while let Some(session_file) = project_entries
                    .next_entry()
                    .await
                    .map_err(|e| format!("Failed to read session file: {}", e))?
                {
                    if session_file.path().extension().and_then(|s| s.to_str()) == Some("json") {
                        let content = fs::read_to_string(session_file.path())
                            .await
                            .map_err(|e| format!("Failed to read session file: {}", e))?;
                        let session: Session = serde_json::from_str(&content)
                            .map_err(|e| format!("Failed to parse session: {}", e))?;
                        sessions.push(session);
                    }
                }
            }
        }

        Ok(sessions)
    }

    // ========================================================================
    // Message Storage
    // ========================================================================

    /// Save a message to disk
    pub async fn save_message(&self, message: &MessageWithParts) -> Result<(), String> {
        let session_id = match &message.info {
            Message::User { session_id, .. } | Message::Assistant { session_id, .. } => session_id,
        };

        let message_dir = self.base_dir.join("message").join(session_id);
        fs::create_dir_all(&message_dir)
            .await
            .map_err(|e| format!("Failed to create message directory: {}", e))?;

        let message_id = match &message.info {
            Message::User { id, .. } | Message::Assistant { id, .. } => id,
        };

        let file_path = message_dir.join(format!("{}.json", message_id));
        let json = serde_json::to_string_pretty(message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        fs::write(&file_path, json)
            .await
            .map_err(|e| format!("Failed to write message file: {}", e))?;

        Ok(())
    }

    /// Load all messages for a session
    pub async fn load_messages(&self, session_id: &str) -> Result<Vec<MessageWithParts>, String> {
        let mut messages = Vec::new();
        let message_dir = self.base_dir.join("message").join(session_id);

        if !message_dir.exists() {
            return Ok(messages);
        }

        let mut entries = fs::read_dir(&message_dir)
            .await
            .map_err(|e| format!("Failed to read message directory: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Failed to read directory entry: {}", e))?
        {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(entry.path())
                    .await
                    .map_err(|e| format!("Failed to read message file: {}", e))?;
                let message: MessageWithParts = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse message: {}", e))?;
                messages.push(message);
            }
        }

        // Sort by creation time
        messages.sort_by_key(|m| match &m.info {
            Message::User { time, .. } => time.created,
            Message::Assistant { time, .. } => time.created,
        });

        Ok(messages)
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}
