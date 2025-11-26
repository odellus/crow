use crate::types::{Message, Part, Session, SessionTime};
use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

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
    storage: Option<Arc<CrowStorage>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(HashMap::new())),
            storage: CrowStorage::new().ok().map(Arc::new),
        }
    }

    /// Initialize storage and load existing sessions
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
        crate::bus::publish(
            crate::bus::events::MESSAGE_UPDATED,
            serde_json::json!({ "info": message.info }),
        );

        // STREAMING EXPORT: Export session to markdown after every message
        // This maintains real-time .crow/sessions/{id}.md files
        {
            use super::export::SessionExport;
            use std::path::PathBuf;

            let session_dir = PathBuf::from(&session.directory); // Use PathBuf for owned value
            let store_clone = self.clone();
            let session_id_clone = session_id.to_string();

            tracing::debug!("Starting export for session {}", session_id_clone);

            // Export in background to avoid blocking
            tokio::spawn(async move {
                tracing::debug!("Inside tokio::spawn for session {}", session_id_clone);
                match SessionExport::stream_to_file(&store_clone, &session_id_clone, &session_dir) {
                    Ok(_) => {
                        tracing::debug!("Successfully exported session {}", session_id_clone)
                    }
                    Err(e) => {
                        tracing::warn!("Failed to export session {}: {}", session_id_clone, e)
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CacheTokens, Message, MessagePath, MessageTime, Part, TokenUsage};

    // ==================== Store Creation Tests ====================

    #[tokio::test]
    async fn test_session_store_new() {
        let store = SessionStore::new();
        // Should create without error
        let sessions = store.list(None).unwrap();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_session_store_default() {
        let store = SessionStore::default();
        let sessions = store.list(None).unwrap();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_session_store_clone() {
        let store = SessionStore::new();
        store.create("/tmp/test".to_string(), None, None).unwrap();

        let cloned = store.clone();
        let sessions = cloned.list(None).unwrap();
        assert_eq!(sessions.len(), 1);
    }

    // ==================== Session Creation Tests ====================

    #[tokio::test]
    async fn test_create_session_basic() {
        let store = SessionStore::new();
        let session = store.create("/tmp/test".to_string(), None, None).unwrap();

        assert!(!session.id.is_empty());
        assert_eq!(session.directory, "/tmp/test");
        assert!(session.parent_id.is_none());
        assert!(!session.title.is_empty());
    }

    #[tokio::test]
    async fn test_create_session_with_title() {
        let store = SessionStore::new();
        let session = store
            .create(
                "/tmp/test".to_string(),
                None,
                Some("My Custom Title".to_string()),
            )
            .unwrap();

        assert_eq!(session.title, "My Custom Title");
    }

    #[tokio::test]
    async fn test_create_session_with_parent() {
        let store = SessionStore::new();
        let parent = store.create("/tmp/test".to_string(), None, None).unwrap();

        let child = store
            .create("/tmp/test".to_string(), Some(parent.id.clone()), None)
            .unwrap();

        assert_eq!(child.parent_id, Some(parent.id));
    }

    #[tokio::test]
    async fn test_create_session_generates_unique_ids() {
        let store = SessionStore::new();
        let s1 = store.create("/tmp/test1".to_string(), None, None).unwrap();
        let s2 = store.create("/tmp/test2".to_string(), None, None).unwrap();

        assert_ne!(s1.id, s2.id);
    }

    #[tokio::test]
    async fn test_create_session_sets_timestamps() {
        let store = SessionStore::new();
        let session = store.create("/tmp/test".to_string(), None, None).unwrap();

        assert!(session.time.created > 0);
        assert!(session.time.updated > 0);
        assert_eq!(session.time.created, session.time.updated);
    }

    // ==================== Session Get Tests ====================

    #[tokio::test]
    async fn test_get_session_exists() {
        let store = SessionStore::new();
        let created = store.create("/tmp/test".to_string(), None, None).unwrap();

        let retrieved = store.get(&created.id).unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.directory, created.directory);
    }

    #[tokio::test]
    async fn test_get_session_not_found() {
        let store = SessionStore::new();
        let result = store.get("nonexistent-id");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // ==================== Session List Tests ====================

    #[tokio::test]
    async fn test_list_sessions_empty() {
        let store = SessionStore::new();
        let sessions = store.list(None).unwrap();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_list_sessions_all() {
        let store = SessionStore::new();
        store.create("/tmp/test1".to_string(), None, None).unwrap();
        store.create("/tmp/test2".to_string(), None, None).unwrap();

        let sessions = store.list(None).unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_list_sessions_by_directory() {
        let store = SessionStore::new();
        store.create("/tmp/dir1".to_string(), None, None).unwrap();
        store.create("/tmp/dir2".to_string(), None, None).unwrap();
        store.create("/tmp/dir1".to_string(), None, None).unwrap();

        let dir1_sessions = store.list(Some("/tmp/dir1".to_string())).unwrap();
        assert_eq!(dir1_sessions.len(), 2);

        let dir2_sessions = store.list(Some("/tmp/dir2".to_string())).unwrap();
        assert_eq!(dir2_sessions.len(), 1);
    }

    #[tokio::test]
    async fn test_list_sessions_sorted_by_creation_time() {
        let store = SessionStore::new();

        // Create sessions with slight delays to ensure different timestamps
        let s1 = store
            .create("/tmp/test".to_string(), None, Some("First".to_string()))
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let s2 = store
            .create("/tmp/test".to_string(), None, Some("Second".to_string()))
            .unwrap();

        let sessions = store.list(None).unwrap();
        // Should be sorted newest first
        assert_eq!(sessions[0].id, s2.id);
        assert_eq!(sessions[1].id, s1.id);
    }

    // ==================== Session Update Tests ====================

    #[tokio::test]
    async fn test_update_session_title() {
        let store = SessionStore::new();
        let session = store.create("/tmp/test".to_string(), None, None).unwrap();

        let updated = store
            .update(&session.id, Some("New Title".to_string()))
            .unwrap();

        assert_eq!(updated.title, "New Title");
    }

    #[tokio::test]
    async fn test_update_session_updates_timestamp() {
        let store = SessionStore::new();
        let session = store.create("/tmp/test".to_string(), None, None).unwrap();
        let original_updated = session.time.updated;

        std::thread::sleep(std::time::Duration::from_millis(10));

        let updated = store.update(&session.id, Some("New".to_string())).unwrap();
        assert!(updated.time.updated > original_updated);
    }

    #[tokio::test]
    async fn test_update_session_not_found() {
        let store = SessionStore::new();
        let result = store.update("nonexistent", Some("Title".to_string()));

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_with_closure() {
        let store = SessionStore::new();
        let session = store.create("/tmp/test".to_string(), None, None).unwrap();

        let updated = store
            .update_with(&session.id, |s| {
                s.title = "Updated via closure".to_string();
            })
            .unwrap();

        assert_eq!(updated.title, "Updated via closure");
    }

    #[tokio::test]
    async fn test_update_metadata() {
        let store = SessionStore::new();
        let session = store.create("/tmp/test".to_string(), None, None).unwrap();

        let metadata = serde_json::json!({
            "key": "value",
            "count": 42
        });

        let updated = store
            .update_metadata(&session.id, metadata.clone())
            .unwrap();
        assert_eq!(updated.metadata, Some(metadata));
    }

    // ==================== Session Delete Tests ====================

    #[tokio::test]
    async fn test_delete_session_exists() {
        let store = SessionStore::new();
        let session = store.create("/tmp/test".to_string(), None, None).unwrap();

        let deleted = store.delete(&session.id).unwrap();
        assert!(deleted);

        // Verify it's gone
        let result = store.get(&session.id);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_session_not_found() {
        let store = SessionStore::new();
        let deleted = store.delete("nonexistent").unwrap();
        assert!(!deleted);
    }

    // ==================== Child Sessions Tests ====================

    #[tokio::test]
    async fn test_get_children_empty() {
        let store = SessionStore::new();
        let parent = store.create("/tmp/test".to_string(), None, None).unwrap();

        let children = store.get_children(&parent.id).unwrap();
        assert!(children.is_empty());
    }

    #[tokio::test]
    async fn test_get_children_with_children() {
        let store = SessionStore::new();
        let parent = store.create("/tmp/test".to_string(), None, None).unwrap();

        store
            .create("/tmp/test".to_string(), Some(parent.id.clone()), None)
            .unwrap();
        store
            .create("/tmp/test".to_string(), Some(parent.id.clone()), None)
            .unwrap();

        let children = store.get_children(&parent.id).unwrap();
        assert_eq!(children.len(), 2);
    }

    // ==================== Message Tests ====================

    fn create_test_message(role: &str, content: &str) -> MessageWithParts {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let msg_id = crate::utils::generate_message_id();
        let part_id = crate::utils::generate_part_id();

        let message = if role == "user" {
            Message::User {
                id: msg_id.clone(),
                session_id: "test-session".to_string(),
                time: MessageTime {
                    created: now,
                    completed: None,
                },
                summary: None,
                metadata: None,
            }
        } else {
            Message::Assistant {
                id: msg_id.clone(),
                session_id: "test-session".to_string(),
                parent_id: "parent-msg".to_string(),
                model_id: "test-model".to_string(),
                provider_id: "test-provider".to_string(),
                mode: "build".to_string(),
                time: MessageTime {
                    created: now,
                    completed: None,
                },
                path: MessagePath {
                    cwd: "/tmp".to_string(),
                    root: "/tmp".to_string(),
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
        };

        MessageWithParts {
            info: message,
            parts: vec![Part::Text {
                id: part_id,
                session_id: "test-session".to_string(),
                message_id: msg_id,
                text: content.to_string(),
            }],
        }
    }

    #[tokio::test]
    async fn test_add_message() {
        let store = SessionStore::new();
        let session = store.create("/tmp/test".to_string(), None, None).unwrap();

        let message = create_test_message("user", "Hello");
        let result = store.add_message(&session.id, message);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_message_session_not_found() {
        let store = SessionStore::new();
        let message = create_test_message("user", "Hello");

        let result = store.add_message("nonexistent", message);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_messages_empty() {
        let store = SessionStore::new();
        let session = store.create("/tmp/test".to_string(), None, None).unwrap();

        let messages = store.get_messages(&session.id).unwrap();
        assert!(messages.is_empty());
    }

    #[tokio::test]
    async fn test_get_messages_with_messages() {
        let store = SessionStore::new();
        let session = store.create("/tmp/test".to_string(), None, None).unwrap();

        store
            .add_message(&session.id, create_test_message("user", "Hello"))
            .unwrap();
        store
            .add_message(&session.id, create_test_message("assistant", "Hi there"))
            .unwrap();

        let messages = store.get_messages(&session.id).unwrap();
        assert_eq!(messages.len(), 2);
    }

    #[tokio::test]
    async fn test_get_message_by_id() {
        let store = SessionStore::new();
        let session = store.create("/tmp/test".to_string(), None, None).unwrap();

        let msg = create_test_message("user", "Hello");
        let msg_id = match &msg.info {
            Message::User { id, .. } => id.clone(),
            Message::Assistant { id, .. } => id.clone(),
        };

        store.add_message(&session.id, msg).unwrap();

        let retrieved = store.get_message(&session.id, &msg_id).unwrap();
        match &retrieved.parts[0] {
            Part::Text { text, .. } => assert_eq!(text, "Hello"),
            _ => panic!("Expected Text part"),
        }
    }

    #[tokio::test]
    async fn test_get_message_not_found() {
        let store = SessionStore::new();
        let session = store.create("/tmp/test".to_string(), None, None).unwrap();

        let result = store.get_message(&session.id, "nonexistent-msg-id");
        assert!(result.is_err());
    }

    // ==================== Message Isolation Tests ====================

    #[tokio::test]
    async fn test_messages_are_session_isolated() {
        let store = SessionStore::new();
        let s1 = store.create("/tmp/test1".to_string(), None, None).unwrap();
        let s2 = store.create("/tmp/test2".to_string(), None, None).unwrap();

        store
            .add_message(&s1.id, create_test_message("user", "Message for S1"))
            .unwrap();
        store
            .add_message(&s2.id, create_test_message("user", "Message for S2"))
            .unwrap();

        let s1_messages = store.get_messages(&s1.id).unwrap();
        let s2_messages = store.get_messages(&s2.id).unwrap();

        assert_eq!(s1_messages.len(), 1);
        assert_eq!(s2_messages.len(), 1);
        match &s1_messages[0].parts[0] {
            Part::Text { text, .. } => assert_eq!(text, "Message for S1"),
            _ => panic!("Expected Text part"),
        }
        match &s2_messages[0].parts[0] {
            Part::Text { text, .. } => assert_eq!(text, "Message for S2"),
            _ => panic!("Expected Text part"),
        }
    }
}
