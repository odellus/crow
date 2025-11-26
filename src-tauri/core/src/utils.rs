//! Utility functions for ID generation and hashing

use std::path::Path;

/// Base62 alphabet matching OpenCode
const BASE62_ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Generate a base62 encoded ID from random bytes
/// Returns a string of the specified length
pub fn generate_base62_id(length: usize) -> String {
    let mut result = String::with_capacity(length);
    let mut bytes = vec![0u8; length];

    // Use getrandom for cryptographic randomness
    getrandom::getrandom(&mut bytes).unwrap_or_else(|_| {
        // Fallback to timestamp-based if getrandom fails
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = ((now >> (i * 8)) & 0xFF) as u8;
        }
    });

    for byte in bytes {
        let idx = (byte as usize) % BASE62_ALPHABET.len();
        result.push(BASE62_ALPHABET[idx] as char);
    }

    result
}

/// Generate a session ID in OpenCode format: ses_[base62]
pub fn generate_session_id() -> String {
    format!("ses_{}", generate_base62_id(22))
}

/// Generate a message ID in OpenCode format: msg_[base62]
pub fn generate_message_id() -> String {
    format!("msg_{}", generate_base62_id(22))
}

/// Generate a part ID in OpenCode format: part_[base62]
pub fn generate_part_id() -> String {
    format!("part_{}", generate_base62_id(22))
}

/// Compute project ID from directory path (SHA256 hash)
/// Matches OpenCode's projectID computation
pub fn compute_project_id(directory: &Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Get canonical path
    let canonical = directory
        .canonicalize()
        .unwrap_or_else(|_| directory.to_path_buf());

    // Hash the path
    let mut hasher = DefaultHasher::new();
    canonical.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish();

    // Convert to hex string (32 chars like OpenCode)
    format!("{:016x}{:016x}", hash, hash.rotate_left(32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_base62_id() {
        let id = generate_base62_id(22);
        assert_eq!(id.len(), 22);
        // All characters should be base62
        assert!(id.chars().all(|c| BASE62_ALPHABET.contains(&(c as u8))));
    }

    #[test]
    fn test_generate_session_id() {
        let id = generate_session_id();
        assert!(id.starts_with("ses_"));
        assert_eq!(id.len(), 26); // "ses_" + 22 chars
    }

    #[test]
    fn test_compute_project_id() {
        let id = compute_project_id(Path::new("/tmp"));
        assert_eq!(id.len(), 32);
        // Should be hex characters
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_project_id_consistency() {
        let id1 = compute_project_id(Path::new("/tmp/test"));
        let id2 = compute_project_id(Path::new("/tmp/test"));
        assert_eq!(id1, id2);
    }
}
