//! Session locking and abort functionality
//! Prevents race conditions and allows cancelling runaway agents

use std::collections::HashMap;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[cfg(feature = "server")]
use parking_lot::RwLock;

#[cfg(not(feature = "server"))]
use std::sync::RwLock;

/// Session lock with abort signal
pub struct SessionLock {
    pub session_id: String,
    pub locked_at: u64,
    pub cancellation_token: CancellationToken,
}

impl SessionLock {
    pub fn new(session_id: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            session_id,
            locked_at: now,
            cancellation_token: CancellationToken::new(),
        }
    }

    pub fn is_locked(&self) -> bool {
        !self.cancellation_token.is_cancelled()
    }

    pub fn abort(&self) {
        eprintln!("[LOCK] Aborting session {}", self.session_id);
        self.cancellation_token.cancel();
    }

    pub fn should_abort(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    /// Get a clone of the cancellation token for use by executor
    pub fn token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }
}

/// Global lock manager for all sessions
pub struct SessionLockManager {
    locks: Arc<RwLock<HashMap<String, Arc<SessionLock>>>>,
}

impl SessionLockManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Acquire lock for a session
    #[cfg(feature = "server")]
    pub fn acquire(&self, session_id: &str) -> Result<Arc<SessionLock>, String> {
        let mut locks = self.locks.write();

        if locks.contains_key(session_id) {
            return Err(format!("Session {} is already locked", session_id));
        }

        let lock = Arc::new(SessionLock::new(session_id.to_string()));
        locks.insert(session_id.to_string(), lock.clone());

        eprintln!("[LOCK] Acquired lock for session {}", session_id);
        Ok(lock)
    }

    #[cfg(not(feature = "server"))]
    pub fn acquire(&self, session_id: &str) -> Result<Arc<SessionLock>, String> {
        let mut locks = self
            .locks
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;

        if locks.contains_key(session_id) {
            return Err(format!("Session {} is already locked", session_id));
        }

        let lock = Arc::new(SessionLock::new(session_id.to_string()));
        locks.insert(session_id.to_string(), lock.clone());

        eprintln!("[LOCK] Acquired lock for session {}", session_id);
        Ok(lock)
    }

    /// Release lock for a session
    #[cfg(feature = "server")]
    pub fn release(&self, session_id: &str) {
        let mut locks = self.locks.write();
        locks.remove(session_id);
        eprintln!("[LOCK] Released lock for session {}", session_id);
    }

    #[cfg(not(feature = "server"))]
    pub fn release(&self, session_id: &str) {
        if let Ok(mut locks) = self.locks.write() {
            locks.remove(session_id);
            eprintln!("[LOCK] Released lock for session {}", session_id);
        }
    }

    /// Get existing lock (if any)
    #[cfg(feature = "server")]
    pub fn get(&self, session_id: &str) -> Option<Arc<SessionLock>> {
        let locks = self.locks.read();
        locks.get(session_id).cloned()
    }

    #[cfg(not(feature = "server"))]
    pub fn get(&self, session_id: &str) -> Option<Arc<SessionLock>> {
        let locks = self.locks.read().ok()?;
        locks.get(session_id).cloned()
    }

    /// Abort a session (sets abort signal on its lock)
    #[cfg(feature = "server")]
    pub fn abort(&self, session_id: &str) -> Result<(), String> {
        let locks = self.locks.read();
        if let Some(lock) = locks.get(session_id) {
            lock.abort();
            Ok(())
        } else {
            Err(format!(
                "Session {} is not locked (not running)",
                session_id
            ))
        }
    }

    #[cfg(not(feature = "server"))]
    pub fn abort(&self, session_id: &str) -> Result<(), String> {
        let locks = self
            .locks
            .read()
            .map_err(|e| format!("Lock error: {}", e))?;
        if let Some(lock) = locks.get(session_id) {
            lock.abort();
            Ok(())
        } else {
            Err(format!(
                "Session {} is not locked (not running)",
                session_id
            ))
        }
    }

    /// Check if session is currently locked
    #[cfg(feature = "server")]
    pub fn is_locked(&self, session_id: &str) -> bool {
        let locks = self.locks.read();
        locks.contains_key(session_id)
    }

    #[cfg(not(feature = "server"))]
    pub fn is_locked(&self, session_id: &str) -> bool {
        if let Ok(locks) = self.locks.read() {
            locks.contains_key(session_id)
        } else {
            false
        }
    }
}

impl Default for SessionLockManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_acquire_and_release() {
        let manager = SessionLockManager::new();
        let session_id = "test-session";

        // Acquire lock
        let lock = manager.acquire(session_id).unwrap();
        assert!(lock.is_locked());
        assert!(manager.is_locked(session_id));

        // Can't acquire again
        assert!(manager.acquire(session_id).is_err());

        // Release
        manager.release(session_id);
        assert!(!manager.is_locked(session_id));

        // Can acquire again after release
        assert!(manager.acquire(session_id).is_ok());
    }

    #[test]
    fn test_abort() {
        let manager = SessionLockManager::new();
        let session_id = "test-session";

        let lock = manager.acquire(session_id).unwrap();
        assert!(!lock.should_abort());

        // Abort
        manager.abort(session_id).unwrap();
        assert!(lock.should_abort());
    }
}
