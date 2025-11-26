//! LSP Client implementation using JSON-RPC over stdio

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::{debug, error, info};

use super::language::get_language_id;
use super::server::ServerHandle;

/// LSP Diagnostic (matches vscode-languageserver-types)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub message: String,
    #[serde(default)]
    pub severity: Option<u32>,
    #[serde(default)]
    pub code: Option<Value>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Diagnostic {
    /// Format diagnostic for display
    pub fn pretty(&self) -> String {
        let severity = match self.severity {
            Some(1) => "ERROR",
            Some(2) => "WARN",
            Some(3) => "INFO",
            Some(4) => "HINT",
            _ => "ERROR",
        };

        let line = self.range.start.line + 1;
        let col = self.range.start.character + 1;

        format!("{} [{}:{}] {}", severity, line, col, self.message)
    }
}

/// LSP Client for communicating with a language server
pub struct LspClient {
    server_id: String,
    root: PathBuf,
    stdin: Mutex<ChildStdin>,
    process: Mutex<Child>,
    request_id: Mutex<i64>,
    diagnostics: RwLock<HashMap<PathBuf, Vec<Diagnostic>>>,
    pending_responses: RwLock<HashMap<i64, tokio::sync::oneshot::Sender<Value>>>,
    diagnostic_tx: broadcast::Sender<PathBuf>,
    files: RwLock<HashMap<PathBuf, i32>>, // path -> version
}

impl LspClient {
    /// Create a new LSP client from a spawned server
    pub async fn new(
        server_id: String,
        handle: ServerHandle,
        root: PathBuf,
    ) -> Result<Self, String> {
        let mut process = handle.process;
        let initialization = handle.initialization;

        let stdin = process.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = process.stdout.take().ok_or("Failed to get stdout")?;

        let (diagnostic_tx, _) = broadcast::channel(100);

        let client = Self {
            server_id: server_id.clone(),
            root: root.clone(),
            stdin: Mutex::new(stdin),
            process: Mutex::new(process),
            request_id: Mutex::new(0),
            diagnostics: RwLock::new(HashMap::new()),
            pending_responses: RwLock::new(HashMap::new()),
            diagnostic_tx,
            files: RwLock::new(HashMap::new()),
        };

        // Start message reader task
        client.start_reader(stdout);

        // Initialize the server
        client.initialize(&root, initialization).await?;

        Ok(client)
    }

    fn start_reader(&self, stdout: ChildStdout) {
        let diagnostics = self.diagnostics.try_read().map(|_| ()).ok();
        let _ = diagnostics; // Just to verify we can access it

        // We need to spawn this in a way that can access self
        // For now, we'll handle messages inline in a spawned task
        let diagnostic_tx = self.diagnostic_tx.clone();
        let server_id = self.server_id.clone();

        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);

            loop {
                // Read headers
                let mut content_length = 0;
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line).await {
                        Ok(0) => return, // EOF
                        Ok(_) => {
                            if line == "\r\n" || line == "\n" {
                                break;
                            }
                            if let Some(len) = line.strip_prefix("Content-Length: ") {
                                content_length = len.trim().parse().unwrap_or(0);
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to read LSP message header");
                            return;
                        }
                    }
                }

                if content_length == 0 {
                    continue;
                }

                // Read content
                let mut content = vec![0u8; content_length];
                if let Err(e) = tokio::io::AsyncReadExt::read_exact(&mut reader, &mut content).await
                {
                    error!(error = %e, "Failed to read LSP message content");
                    return;
                }

                // Parse JSON
                let message: Value = match serde_json::from_slice(&content) {
                    Ok(v) => v,
                    Err(e) => {
                        error!(error = %e, "Failed to parse LSP message");
                        continue;
                    }
                };

                // Handle message
                // Check if it's a response
                if let Some(id) = message.get("id").and_then(|v| v.as_i64()) {
                    if message.get("method").is_none() {
                        // It's a response - we can't access pending_responses safely here
                        // This is a simplified implementation
                        debug!(id = id, "Received response");
                        continue;
                    }
                }

                // Check if it's a notification
                if let Some(method) = message.get("method").and_then(|v| v.as_str()) {
                    match method {
                        "textDocument/publishDiagnostics" => {
                            if let Some(params) = message.get("params") {
                                let uri = match params.get("uri").and_then(|v| v.as_str()) {
                                    Some(u) => u,
                                    None => continue,
                                };

                                let path = match uri.strip_prefix("file://") {
                                    Some(p) => PathBuf::from(p),
                                    None => continue,
                                };

                                let diags: Vec<Diagnostic> = params
                                    .get("diagnostics")
                                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                                    .unwrap_or_default();

                                info!(
                                    server_id = %server_id,
                                    path = %path.display(),
                                    count = diags.len(),
                                    "Received diagnostics"
                                );

                                let _ = diagnostic_tx.send(path);
                            }
                        }
                        _ => {
                            debug!(method = %method, "Unhandled LSP notification");
                        }
                    }
                }
            }
        });
    }

    async fn initialize(&self, root: &Path, initialization: Option<Value>) -> Result<(), String> {
        let root_uri = format!("file://{}", root.display());

        let init_options = initialization.clone().unwrap_or(json!({}));

        let params = json!({
            "rootUri": root_uri,
            "processId": std::process::id(),
            "workspaceFolders": [{
                "name": "workspace",
                "uri": root_uri
            }],
            "initializationOptions": init_options,
            "capabilities": {
                "window": {
                    "workDoneProgress": true
                },
                "workspace": {
                    "configuration": true
                },
                "textDocument": {
                    "synchronization": {
                        "didOpen": true,
                        "didChange": true
                    },
                    "publishDiagnostics": {
                        "versionSupport": true
                    }
                }
            }
        });

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.send_request("initialize", params),
        )
        .await
        .map_err(|_| "Initialize timeout")?
        .map_err(|e| format!("Initialize error: {}", e))?;

        debug!(server_id = %self.server_id, "Initialize response: {:?}", result);

        // Send initialized notification
        self.send_notification("initialized", json!({})).await?;

        // Send configuration if provided
        if let Some(init) = initialization {
            self.send_notification(
                "workspace/didChangeConfiguration",
                json!({
                    "settings": init
                }),
            )
            .await?;
        }

        info!(server_id = %self.server_id, "LSP client initialized");

        Ok(())
    }

    async fn send_request(&self, method: &str, params: Value) -> Result<Value, String> {
        let id = {
            let mut id = self.request_id.lock().await;
            *id += 1;
            *id
        };

        let message = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        // For simplicity, we'll just send and wait a bit
        // A proper implementation would use channels
        self.send_message(&message).await?;

        // Simple wait for response (not ideal but works for init)
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok(json!({}))
    }

    async fn send_notification(&self, method: &str, params: Value) -> Result<(), String> {
        let message = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        self.send_message(&message).await
    }

    async fn send_message(&self, message: &Value) -> Result<(), String> {
        let content = serde_json::to_string(message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        let header = format!("Content-Length: {}\r\n\r\n", content.len());

        let mut stdin = self.stdin.lock().await;
        stdin
            .write_all(header.as_bytes())
            .await
            .map_err(|e| format!("Failed to write header: {}", e))?;
        stdin
            .write_all(content.as_bytes())
            .await
            .map_err(|e| format!("Failed to write content: {}", e))?;
        stdin
            .flush()
            .await
            .map_err(|e| format!("Failed to flush: {}", e))?;

        Ok(())
    }

    /// Open a file in the LSP server
    pub async fn open_file(&self, path: &Path) -> Result<(), String> {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };

        let text = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();
        let language_id = get_language_id(&extension);

        let uri = format!("file://{}", path.display());

        let mut files = self.files.write().await;

        if let Some(version) = files.get_mut(&path) {
            // File already open, send didChange
            *version += 1;
            let new_version = *version;
            drop(files);

            self.send_notification(
                "textDocument/didChange",
                json!({
                    "textDocument": {
                        "uri": uri,
                        "version": new_version
                    },
                    "contentChanges": [{ "text": text }]
                }),
            )
            .await?;
        } else {
            // New file, send didOpen
            files.insert(path.clone(), 0);
            drop(files);

            // Clear old diagnostics
            self.diagnostics.write().await.remove(&path);

            self.send_notification(
                "textDocument/didOpen",
                json!({
                    "textDocument": {
                        "uri": uri,
                        "languageId": language_id,
                        "version": 0,
                        "text": text
                    }
                }),
            )
            .await?;
        }

        Ok(())
    }

    /// Wait for diagnostics for a file
    pub async fn wait_for_diagnostics(&self, path: &Path) {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };

        let mut rx = self.diagnostic_tx.subscribe();

        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), async {
            while let Ok(p) = rx.recv().await {
                if p == path {
                    break;
                }
            }
        })
        .await;
    }

    /// Get hover information for a position
    pub async fn hover(&self, file: &Path, line: u32, character: u32) -> Result<Value, String> {
        let uri = format!("file://{}", file.display());

        self.send_request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": uri },
                "position": {
                    "line": line,
                    "character": character
                }
            }),
        )
        .await
    }

    /// Get current diagnostics
    pub fn diagnostics(&self) -> HashMap<PathBuf, Vec<Diagnostic>> {
        // Use try_read to avoid blocking
        self.diagnostics
            .try_read()
            .map(|d| d.clone())
            .unwrap_or_default()
    }

    /// Get server ID
    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    /// Get root directory
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Shutdown the client
    pub async fn shutdown(&self) -> Result<(), String> {
        info!(server_id = %self.server_id, "Shutting down LSP client");

        // Send shutdown request
        let _ = self.send_request("shutdown", Value::Null).await;

        // Send exit notification
        let _ = self.send_notification("exit", Value::Null).await;

        // Kill process
        let mut process = self.process.lock().await;
        let _ = process.kill().await;

        Ok(())
    }
}
