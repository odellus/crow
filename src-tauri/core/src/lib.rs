//! Crow Core - AI coding assistant backend logic
//!
//! This crate contains all the core functionality for Crow:
//! - Agent execution and orchestration
//! - Tool implementations (bash, edit, read, write, etc.)
//! - LLM provider clients
//! - Session and message management
//! - Configuration and storage

pub mod types;
pub use types::*;

pub mod providers;
pub use providers::*;

pub mod session;
pub use session::*;

pub mod agent;
pub mod auth;
pub mod bus;
pub mod config;
pub mod global;
pub mod logging;
pub mod lsp;
pub mod snapshot;
pub mod storage;
pub mod tools;
pub mod utils;

// Re-export commonly used items
pub use agent::{AgentExecutor, AgentRegistry};
pub use config::{Config, ConfigLoader};
pub use storage::CrowStorage;
pub use tools::ToolRegistry;
