//! LSP Server configurations and spawning

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::{Child, Command};

/// Handle to a spawned LSP server process
pub struct ServerHandle {
    pub process: Child,
    pub initialization: Option<Value>,
}

/// Configuration for an LSP server
#[derive(Clone)]
pub struct ServerConfig {
    pub id: String,
    pub extensions: Vec<String>,
}

impl ServerConfig {
    /// Find the root directory for this server given a file
    pub async fn find_root(&self, file: &Path, working_dir: &Path) -> Option<PathBuf> {
        match self.id.as_str() {
            "rust" => find_rust_root(file, working_dir).await,
            "typescript" => find_ts_root(file, working_dir).await,
            "gopls" => find_go_root(file, working_dir).await,
            "pyright" => find_python_root(file, working_dir).await,
            _ => Some(working_dir.to_path_buf()),
        }
    }

    /// Spawn the LSP server
    pub async fn spawn(&self, root: &Path) -> Result<ServerHandle, String> {
        match self.id.as_str() {
            "rust" => spawn_rust_analyzer(root).await,
            "typescript" => spawn_typescript(root).await,
            "gopls" => spawn_gopls(root).await,
            "pyright" => spawn_pyright(root).await,
            _ => Err(format!("Unknown server: {}", self.id)),
        }
    }
}

/// Find nearest file matching patterns, searching up from start to stop
async fn find_up(patterns: &[&str], start: &Path, stop: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        for pattern in patterns {
            let candidate = current.join(pattern);
            if candidate.exists() {
                return Some(current);
            }
        }

        if current == stop || !current.starts_with(stop) {
            break;
        }

        match current.parent() {
            Some(p) => current = p.to_path_buf(),
            None => break,
        }
    }

    None
}

/// Get all built-in server configurations
pub fn builtin_servers() -> Vec<ServerConfig> {
    vec![
        ServerConfig {
            id: "rust".to_string(),
            extensions: vec![".rs".to_string()],
        },
        ServerConfig {
            id: "typescript".to_string(),
            extensions: vec![
                ".ts".to_string(),
                ".tsx".to_string(),
                ".js".to_string(),
                ".jsx".to_string(),
                ".mjs".to_string(),
                ".cjs".to_string(),
                ".mts".to_string(),
                ".cts".to_string(),
            ],
        },
        ServerConfig {
            id: "gopls".to_string(),
            extensions: vec![".go".to_string()],
        },
        ServerConfig {
            id: "pyright".to_string(),
            extensions: vec![".py".to_string(), ".pyi".to_string()],
        },
    ]
}

// Root finders

async fn find_rust_root(file: &Path, working_dir: &Path) -> Option<PathBuf> {
    let start = file.parent().unwrap_or(file);

    // Find Cargo.toml
    let crate_root = find_up(&["Cargo.toml", "Cargo.lock"], start, working_dir).await?;

    // Look for workspace root
    let mut current = crate_root.clone();
    while current.starts_with(working_dir) {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&cargo_toml).await {
                if content.contains("[workspace]") {
                    return Some(current);
                }
            }
        }

        match current.parent() {
            Some(p) => current = p.to_path_buf(),
            None => break,
        }
    }

    Some(crate_root)
}

async fn find_ts_root(file: &Path, working_dir: &Path) -> Option<PathBuf> {
    let start = file.parent().unwrap_or(file);

    // Check for deno first (exclude)
    if find_up(&["deno.json", "deno.jsonc"], start, working_dir)
        .await
        .is_some()
    {
        return None;
    }

    // Find package root
    match find_up(
        &[
            "package-lock.json",
            "bun.lockb",
            "bun.lock",
            "pnpm-lock.yaml",
            "yarn.lock",
            "package.json",
        ],
        start,
        working_dir,
    )
    .await
    {
        Some(root) => Some(root),
        None => Some(working_dir.to_path_buf()),
    }
}

async fn find_go_root(file: &Path, working_dir: &Path) -> Option<PathBuf> {
    let start = file.parent().unwrap_or(file);

    // Check for go.work first (workspace)
    if let Some(root) = find_up(&["go.work"], start, working_dir).await {
        return Some(root);
    }

    // Then check for go.mod
    find_up(&["go.mod", "go.sum"], start, working_dir).await
}

async fn find_python_root(file: &Path, working_dir: &Path) -> Option<PathBuf> {
    let start = file.parent().unwrap_or(file);

    match find_up(
        &[
            "pyproject.toml",
            "setup.py",
            "setup.cfg",
            "requirements.txt",
            "Pipfile",
            "pyrightconfig.json",
        ],
        start,
        working_dir,
    )
    .await
    {
        Some(root) => Some(root),
        None => Some(working_dir.to_path_buf()),
    }
}

// Spawners

async fn spawn_rust_analyzer(root: &Path) -> Result<ServerHandle, String> {
    let bin = which::which("rust-analyzer").map_err(|_| "rust-analyzer not found in PATH")?;

    let process = Command::new(bin)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn rust-analyzer: {}", e))?;

    Ok(ServerHandle {
        process,
        initialization: None,
    })
}

async fn spawn_typescript(root: &Path) -> Result<ServerHandle, String> {
    // Try to find typescript-language-server
    let bin = which::which("typescript-language-server")
        .or_else(|_| which::which("npx"))
        .map_err(|_| "typescript-language-server not found")?;

    let is_npx = bin.to_string_lossy().contains("npx");

    let mut cmd = Command::new(&bin);
    cmd.current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    if is_npx {
        cmd.args(["typescript-language-server", "--stdio"]);
    } else {
        cmd.arg("--stdio");
    }

    let process = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn typescript-language-server: {}", e))?;

    Ok(ServerHandle {
        process,
        initialization: None,
    })
}

async fn spawn_gopls(root: &Path) -> Result<ServerHandle, String> {
    let bin = which::which("gopls").map_err(|_| "gopls not found in PATH")?;

    let process = Command::new(bin)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn gopls: {}", e))?;

    Ok(ServerHandle {
        process,
        initialization: None,
    })
}

async fn spawn_pyright(root: &Path) -> Result<ServerHandle, String> {
    // Try pyright-langserver directly
    let bin = which::which("pyright-langserver")
        .or_else(|_| which::which("npx"))
        .map_err(|_| "pyright-langserver not found")?;

    let is_npx = bin.to_string_lossy().contains("npx");

    let mut cmd = Command::new(&bin);
    cmd.current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    if is_npx {
        cmd.args(["pyright-langserver", "--stdio"]);
    } else {
        cmd.arg("--stdio");
    }

    let process = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn pyright: {}", e))?;

    // Check for virtual environment
    let mut initialization = serde_json::json!({});

    for venv_name in &[".venv", "venv"] {
        let python_path = root.join(venv_name).join("bin").join("python");
        if python_path.exists() {
            initialization["pythonPath"] = serde_json::json!(python_path.to_string_lossy());
            break;
        }
    }

    Ok(ServerHandle {
        process,
        initialization: Some(initialization),
    })
}
