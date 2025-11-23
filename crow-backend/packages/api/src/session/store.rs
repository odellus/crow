use crate::types::{Message, Part, Session, SessionTime};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "server")]
use parking_lot::RwLock;

#[cfg(not(feature = "server"))]
use std::sync::RwLock;

#[cfg(feature = "server")]
use crate::storage::CrowStorage;

/// Represents a complete message with its info and parts
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MessageWithParts {
    pub info: Message,
    pub parts: Vec<Part>,
}

/// Session store with file persistence
#[derive(Clone)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    /// Messages grouped by session ID
    /// Key: session_id, Value: Vec of messages
    messages: Arc<RwLock<HashMap<String, Vec<MessageWithParts>>>>,
    #[cfg(feature = "server")]
    storage: Option<Arc<CrowStorage>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "server")]
            storage: CrowStorage::new().ok().map(Arc::new),
        }
    }

    /// Initialize storage and load existing sessions
    #[cfg(feature = "server")]
    pub async fn init(&self) -> Result<(), String> {
        let storage = self.storage.as_ref().ok_or("Storage not available")?;

        // Initialize storage directories
        storage.init().await?;

        // Load existing sessions
        let sessions = storage.list_sessions().await?;
        {
            let mut sessions_lock = self.sessions.write();
            for session in sessions {
                sessions_lock.insert(session.id.clone(), session);
            }
        }

        // Load messages for each session
        let session_ids: Vec<String> = {
            let sessions_lock = self.sessions.read();
            sessions_lock.keys().cloned().collect()
        };

        for session_id in session_ids {
            let messages = storage.load_messages(&session_id).await?;
            self.messages.write().insert(session_id, messages);
        }

        Ok(())
    }

    /// Initialize storage synchronously (for use in OnceLock init)
    #[cfg(feature = "server")]
    pub fn init_sync(&self) -> Result<(), String> {
        let storage = self.storage.as_ref().ok_or("Storage not available")?;

        // Initialize storage directories synchronously
        storage.init_sync()?;

        // Load existing sessions
        let sessions = storage.list_sessions_sync()?;
        {
            let mut sessions_lock = self.sessions.write();
            for session in sessions {
                sessions_lock.insert(session.id.clone(), session);
            }
        }

        // Load messages for each session
        let session_ids: Vec<String> = {
            let sessions_lock = self.sessions.read();
            sessions_lock.keys().cloned().collect()
        };

        for session_id in session_ids {
            let messages = storage.load_messages_sync(&session_id)?;
            self.messages.write().insert(session_id, messages);
        }

        Ok(())
    }

    /// Create a new session
    pub fn create(
        &self,
        directory: String,
        parent_id: Option<String>,
        title: Option<String>,
    ) -> Result<Session, String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?
            .as_millis() as u64;

        let id = crate::utils::generate_session_id();
        let project_id = crate::utils::compute_project_id(std::path::Path::new(&directory));

        // Generate title with timestamp like OpenCode
        let title = title.unwrap_or_else(|| {
            format!(
                "New session - {}",
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ")
            )
        });

        let session = Session {
            id: id.clone(),
            project_id,
            directory: directory.clone(),
            parent_id,
            summary: None,
            share: None,
            title,
            version: "1.0.0".to_string(),
            time: SessionTime {
                created: now,
                updated: now,
                compacting: None,
            },
            revert: None,
            metadata: None,
        };

        let mut sessions = self.sessions.write();
        sessions.insert(id.clone(), session.clone());

        // Persist to disk in background
        #[cfg(feature = "server")]
        if let Some(storage) = &self.storage {
            let storage = storage.clone();
            let session_clone = session.clone();
            tokio::spawn(async move {
                if let Err(e) = storage.save_session(&session_clone).await {
                    eprintln!("Failed to persist session: {}", e);
                }
            });
        }

        // Publish event
        #[cfg(feature = "server")]
        crate::bus::publish(
            crate::bus::events::SESSION_CREATED,
            serde_json::json!({ "info": session }),
        );

        Ok(session)
    }

    /// Get a session by ID
    pub fn get(&self, id: &str) -> Result<Session, String> {
        let sessions = self.sessions.read();
        sessions
            .get(id)
            .cloned()
            .ok_or_else(|| format!("Session not found: {}", id))
    }

    /// List all sessions
    pub fn list(&self, directory: Option<String>) -> Result<Vec<Session>, String> {
        let sessions = self.sessions.read();

        let mut result: Vec<Session> = sessions
            .values()
            .filter(|s| {
                if let Some(ref dir) = directory {
                    &s.directory == dir
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // Sort by creation time, newest first
        result.sort_by(|a, b| b.time.created.cmp(&a.time.created));

        Ok(result)
    }

    /// Update a session
    pub fn update(&self, id: &str, title: Option<String>) -> Result<Session, String> {
        let mut sessions = self.sessions.write();

        let session = sessions
            .get_mut(id)
            .ok_or_else(|| format!("Session not found: {}", id))?;

        if let Some(new_title) = title {
            session.title = new_title;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?
            .as_millis() as u64;
        session.time.updated = now;

        let updated_session = session.clone();

        // Persist to disk in background
        #[cfg(feature = "server")]
        if let Some(storage) = &self.storage {
            let storage = storage.clone();
            let session_clone = updated_session.clone();
            tokio::spawn(async move {
                if let Err(e) = storage.save_session(&session_clone).await {
                    eprintln!("Failed to persist session update: {}", e);
                }
            });
        }

        // Publish event
        #[cfg(feature = "server")]
        crate::bus::publish(
            crate::bus::events::SESSION_UPDATED,
            serde_json::json!({ "info": updated_session }),
        );

        Ok(updated_session)
    }

    /// Update session metadata
    pub fn update_metadata(
        &self,
        id: &str,
        metadata: serde_json::Value,
    ) -> Result<Session, String> {
        self.update_with(id, |s| {
            s.metadata = Some(metadata);
        })
    }

    /// Update session with a closure that can modify any field
    /// This is the preferred method for complex updates like revert state
    pub fn update_with<F>(&self, id: &str, f: F) -> Result<Session, String>
    where
        F: FnOnce(&mut Session),
    {
        let mut sessions = self.sessions.write();

        let session = sessions
            .get_mut(id)
            .ok_or_else(|| format!("Session not found: {}", id))?;

        // Apply the update function
        f(session);

        // Update timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?
            .as_millis() as u64;
        session.time.updated = now;

        let updated_session = session.clone();

        // Persist to disk in background
        #[cfg(feature = "server")]
        if let Some(storage) = &self.storage {
            let storage = storage.clone();
            let session_clone = updated_session.clone();
            tokio::spawn(async move {
                if let Err(e) = storage.save_session(&session_clone).await {
                    eprintln!("Failed to persist session update: {}", e);
                }
            });
        }

        // Publish event
        #[cfg(feature = "server")]
        crate::bus::publish(
            crate::bus::events::SESSION_UPDATED,
            serde_json::json!({ "info": updated_session }),
        );

        Ok(updated_session)
    }

    /// Delete a session
    pub fn delete(&self, id: &str) -> Result<bool, String> {
        let mut sessions = self.sessions.write();
        let removed = sessions.remove(id);

        // Publish event if session was removed
        #[cfg(feature = "server")]
        if let Some(session) = &removed {
            crate::bus::publish(
                crate::bus::events::SESSION_DELETED,
                serde_json::json!({ "info": session }),
            );
        }

        Ok(removed.is_some())
    }

    /// Get child sessions
    pub fn get_children(&self, parent_id: &str) -> Result<Vec<Session>, String> {
        let sessions = self.sessions.read();

        let mut children: Vec<Session> = sessions
            .values()
            .filter(|s| s.parent_id.as_ref().map(|p| p.as_str()) == Some(parent_id))
            .cloned()
            .collect();

        children.sort_by(|a, b| b.time.created.cmp(&a.time.created));

        Ok(children)
    }

    // ========================================================================
    // Message Management
    // ========================================================================

    /// Add a message to a session
    pub fn add_message(&self, session_id: &str, message: MessageWithParts) -> Result<(), String> {
        // Verify session exists
        let session = self.get(session_id)?;

        let mut messages = self.messages.write();

        messages
            .entry(session_id.to_string())
            .or_insert_with(Vec::new)
            .push(message.clone());

        // Update session timestamp
        self.update(session_id, None)?;

        // Persist to disk in background
        #[cfg(feature = "server")]
        if let Some(storage) = &self.storage {
            let storage = storage.clone();
            let message_clone = message.clone();
            tokio::spawn(async move {
                if let Err(e) = storage.save_message(&message_clone).await {
                    eprintln!("Failed to persist message: {}", e);
                }
            });
        }

        // Publish event
        #[cfg(feature = "server")]
        crate::bus::publish(
            crate::bus::events::MESSAGE_UPDATED,
            serde_json::json!({ "info": message.info }),
        );

        // STREAMING EXPORT: Export session to markdown after every message
        // This maintains real-time .crow/sessions/{id}.md files
        #[cfg(feature = "server")]
        {
            use super::export::SessionExport;
            use std::path::PathBuf;

            let session_dir = PathBuf::from(&session.directory); // Use PathBuf for owned value
            let store_clone = self.clone();
            let session_id_clone = session_id.to_string();

            eprintln!("[EXPORT] Starting export for session {}", session_id_clone);

            // Export in background to avoid blocking
            tokio::spawn(async move {
                eprintln!(
                    "[EXPORT] Inside tokio::spawn for session {}",
                    session_id_clone
                );
                match SessionExport::stream_to_file(&store_clone, &session_id_clone, &session_dir) {
                    Ok(_) => eprintln!(
                        "[EXPORT] ✅ Successfully exported session {}",
                        session_id_clone
                    ),
                    Err(e) => eprintln!(
                        "[EXPORT] ❌ Failed to export session {}: {}",
                        session_id_clone, e
                    ),
                }
            });
        }

        Ok(())
    }

    /// Get all messages for a session
    pub fn get_messages(&self, session_id: &str) -> Result<Vec<MessageWithParts>, String> {
        // Verify session exists
        self.get(session_id)?;

        let messages = self.messages.read();
        Ok(messages.get(session_id).cloned().unwrap_or_else(Vec::new))
    }

    /// Get a specific message by ID
    pub fn get_message(
        &self,
        session_id: &str,
        message_id: &str,
    ) -> Result<MessageWithParts, String> {
        let messages = self.get_messages(session_id)?;

        messages
            .into_iter()
            .find(|m| match &m.info {
                Message::User { id, .. } => id == message_id,
                Message::Assistant { id, .. } => id == message_id,
            })
            .ok_or_else(|| format!("Message not found: {}", message_id))
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}
