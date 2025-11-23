//! LSP (Language Server Protocol) support for Crow
//!
//! This module provides LSP client management, server spawning, and diagnostic aggregation
//! mirroring the OpenCode LSP implementation.

pub mod client;
pub mod language;
pub mod server;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use client::{Diagnostic, LspClient};
use server::ServerConfig;

/// LSP state manager - handles client lifecycle and server spawning
pub struct LspState {
    /// Active LSP clients
    clients: Vec<Arc<LspClient>>,
    /// Available server configurations
    servers: HashMap<String, ServerConfig>,
    /// Broken server IDs (failed to spawn/initialize)
    broken: std::collections::HashSet<String>,
    /// In-flight spawn tasks
    spawning: HashMap<String, tokio::task::JoinHandle<Option<Arc<LspClient>>>>,
}

impl LspState {
    pub fn new() -> Self {
        let mut servers = HashMap::new();

        // Register built-in server configurations
        for config in server::builtin_servers() {
            servers.insert(config.id.clone(), config);
        }

        info!(
            server_ids = ?servers.keys().collect::<Vec<_>>(),
            "Enabled LSP servers"
        );

        Self {
            clients: Vec::new(),
            servers,
            broken: std::collections::HashSet::new(),
            spawning: HashMap::new(),
        }
    }

    /// Get or spawn clients for a file
    pub async fn get_clients(&mut self, file: &Path, working_dir: &Path) -> Vec<Arc<LspClient>> {
        let extension = file
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();

        let mut result = Vec::new();

        for server in self.servers.values() {
            // Check if server handles this extension
            if !server.extensions.is_empty() && !server.extensions.contains(&extension) {
                continue;
            }

            // Find root directory for this server
            let root = match server.find_root(file, working_dir).await {
                Some(r) => r,
                None => continue,
            };

            let key = format!("{}{}", root.display(), server.id);

            // Skip if broken
            if self.broken.contains(&key) {
                continue;
            }

            // Check for existing client
            if let Some(client) = self
                .clients
                .iter()
                .find(|c| c.root() == root && c.server_id() == server.id)
            {
                result.push(Arc::clone(client));
                continue;
            }

            // Spawn new client
            match self.spawn_client(server.clone(), root.clone()).await {
                Some(client) => {
                    result.push(Arc::clone(&client));
                    self.clients.push(client);
                }
                None => {
                    self.broken.insert(key);
                }
            }
        }

        result
    }

    async fn spawn_client(&self, server: ServerConfig, root: PathBuf) -> Option<Arc<LspClient>> {
        info!(server_id = %server.id, root = %root.display(), "Spawning LSP server");

        let handle = match server.spawn(&root).await {
            Ok(h) => h,
            Err(e) => {
                error!(server_id = %server.id, error = %e, "Failed to spawn LSP server");
                return None;
            }
        };

        match LspClient::new(server.id.clone(), handle, root).await {
            Ok(client) => {
                info!(server_id = %server.id, "LSP client initialized");
                Some(Arc::new(client))
            }
            Err(e) => {
                error!(server_id = %server.id, error = %e, "Failed to initialize LSP client");
                None
            }
        }
    }

    /// Get all diagnostics from all clients
    pub fn diagnostics(&self) -> HashMap<PathBuf, Vec<Diagnostic>> {
        let mut results: HashMap<PathBuf, Vec<Diagnostic>> = HashMap::new();

        for client in &self.clients {
            for (path, diags) in client.diagnostics() {
                results.entry(path).or_default().extend(diags);
            }
        }

        results
    }

    /// Get hover information for a position
    pub async fn hover(&self, file: &Path, line: u32, character: u32) -> Vec<serde_json::Value> {
        let mut results = Vec::new();

        for client in &self.clients {
            if let Ok(result) = client.hover(file, line, character).await {
                if !result.is_null() {
                    results.push(result);
                }
            }
        }

        results
    }

    /// Shutdown all clients
    pub async fn shutdown(&mut self) {
        for client in self.clients.drain(..) {
            if let Err(e) = client.shutdown().await {
                error!(error = %e, "Failed to shutdown LSP client");
            }
        }
    }
}

impl Default for LspState {
    fn default() -> Self {
        Self::new()
    }
}

/// Global LSP manager with thread-safe access
pub struct Lsp {
    state: Arc<RwLock<LspState>>,
}

impl Lsp {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(LspState::new())),
        }
    }

    /// Touch a file (open it in relevant LSP servers)
    pub async fn touch_file(&self, file: &Path, working_dir: &Path, wait_for_diagnostics: bool) {
        let mut state = self.state.write().await;
        let clients = state.get_clients(file, working_dir).await;

        for client in clients {
            if let Err(e) = client.open_file(file).await {
                error!(error = %e, file = %file.display(), "Failed to open file in LSP");
            }

            if wait_for_diagnostics {
                client.wait_for_diagnostics(file).await;
            }
        }
    }

    /// Get all diagnostics
    pub async fn diagnostics(&self) -> HashMap<PathBuf, Vec<Diagnostic>> {
        self.state.read().await.diagnostics()
    }

    /// Get hover information
    pub async fn hover(
        &self,
        file: &Path,
        working_dir: &Path,
        line: u32,
        character: u32,
    ) -> Vec<serde_json::Value> {
        let mut state = self.state.write().await;
        let _ = state.get_clients(file, working_dir).await;
        drop(state);

        self.state.read().await.hover(file, line, character).await
    }

    /// Shutdown all LSP clients
    pub async fn shutdown(&self) {
        self.state.write().await.shutdown().await;
    }
}

impl Default for Lsp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_state_new() {
        let state = LspState::new();
        assert!(!state.servers.is_empty());
    }
}
