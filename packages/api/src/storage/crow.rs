//! Crow server-local storage (.crow directory)

use std::path::PathBuf;

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

        // Create .gitignore if it doesn't exist
        let gitignore_path = root.join(".gitignore");
        if !gitignore_path.exists() {
            std::fs::write(&gitignore_path, "*\n!.gitignore\n")
                .map_err(|e| format!("Failed to create .gitignore: {}", e))?;
        }

        Ok(Self { root })
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

use std::path::Path;
