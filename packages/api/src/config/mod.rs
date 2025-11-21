//! Configuration system for Crow
//! Matches OpenCode's configuration patterns from config/config.ts

mod loader;
mod types;

pub use loader::ConfigLoader;
pub use types::*;
