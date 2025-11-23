//! Authentication management for providers
//! Reads auth.json from XDG data directory like OpenCode

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum AuthInfo {
    Api {
        key: String,
    },
    Oauth {
        refresh: String,
        access: String,
        expires: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        enterprise_url: Option<String>,
    },
    WellKnown {
        key: String,
        token: String,
    },
}

/// Get authentication for a provider from auth.json
pub fn get(provider_id: &str) -> Result<Option<AuthInfo>, String> {
    let auth_path = get_auth_path();

    if !auth_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&auth_path)
        .map_err(|e| format!("Failed to read auth.json: {}", e))?;

    let all_auth: HashMap<String, AuthInfo> =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse auth.json: {}", e))?;

    Ok(all_auth.get(provider_id).cloned())
}

/// Get all authentication entries
pub fn all() -> Result<HashMap<String, AuthInfo>, String> {
    let auth_path = get_auth_path();

    if !auth_path.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(&auth_path)
        .map_err(|e| format!("Failed to read auth.json: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse auth.json: {}", e))
}

/// Set authentication for a provider
pub fn set(provider_id: &str, info: AuthInfo) -> Result<(), String> {
    let auth_path = get_auth_path();

    // Create parent directory if needed
    if let Some(parent) = auth_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create auth directory: {}", e))?;
    }

    let mut all_auth = all().unwrap_or_default();
    all_auth.insert(provider_id.to_string(), info);

    let content = serde_json::to_string_pretty(&all_auth)
        .map_err(|e| format!("Failed to serialize auth: {}", e))?;

    std::fs::write(&auth_path, content).map_err(|e| format!("Failed to write auth.json: {}", e))?;

    // Set permissions to 0o600 (owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&auth_path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    Ok(())
}

/// Remove authentication for a provider
pub fn remove(provider_id: &str) -> Result<(), String> {
    let auth_path = get_auth_path();

    if !auth_path.exists() {
        return Ok(());
    }

    let mut all_auth = all()?;
    all_auth.remove(provider_id);

    let content = serde_json::to_string_pretty(&all_auth)
        .map_err(|e| format!("Failed to serialize auth: {}", e))?;

    std::fs::write(&auth_path, content).map_err(|e| format!("Failed to write auth.json: {}", e))?;

    Ok(())
}

/// Get the path to auth.json
fn get_auth_path() -> PathBuf {
    crate::global::GlobalPaths::new().data.join("auth.json")
}
