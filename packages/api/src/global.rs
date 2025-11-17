//! Global paths and initialization
//! Mirrors OpenCode's global/index.ts - uses XDG base directories

use std::path::PathBuf;

/// Global paths for crow
/// Matches OpenCode's directory structure
pub struct GlobalPaths {
    pub home: PathBuf,
    pub data: PathBuf,
    pub bin: PathBuf,
    pub log: PathBuf,
    pub cache: PathBuf,
    pub config: PathBuf,
    pub state: PathBuf,
}

impl GlobalPaths {
    /// Initialize global paths using XDG base directories
    /// Just like OpenCode does
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("Failed to get home directory");

        // Use XDG directories, defaulting to standard locations
        let data_home = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".local/share"));

        let config_home = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".config"));

        let state_home = std::env::var("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".local/state"));

        let cache_home = std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".cache"));

        let data = data_home.join("crow");
        let config = config_home.join("crow");
        let state = state_home.join("crow");
        let cache = cache_home.join("crow");

        Self {
            home,
            bin: data.join("bin"),
            log: data.join("log"),
            data,
            config,
            state,
            cache,
        }
    }

    /// Create all necessary directories
    pub fn init(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.data)?;
        std::fs::create_dir_all(&self.config)?;
        std::fs::create_dir_all(&self.state)?;
        std::fs::create_dir_all(&self.log)?;
        std::fs::create_dir_all(&self.bin)?;
        std::fs::create_dir_all(&self.cache)?;
        Ok(())
    }
}

impl Default for GlobalPaths {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_paths() {
        let paths = GlobalPaths::new();
        assert!(paths.home.exists());
        // Don't actually create dirs in test
        assert!(paths.data.to_string_lossy().ends_with("crow"));
        assert!(paths.config.to_string_lossy().contains("crow"));
    }
}
