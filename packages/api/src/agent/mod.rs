//! Agent module - manages different agent types and execution
//!
//! This module provides:
//! - Agent type definitions (AgentInfo, AgentMode, Permission)
//! - Built-in agents (general, build, plan, supervisor, architect, discriminator)
//! - Agent registry for managing available agents
//! - Dual-agent architecture (executor + discriminator)
//! - Agent execution with ReACT loop
//!
//! Based on OpenCode's agent system from:
//! opencode/packages/opencode/src/agent/

pub mod builtins;
pub mod dual;
pub mod executor;
pub mod perspective;
pub mod prompt;
pub mod registry;
pub mod runtime;
pub mod types;

pub use builtins::get_builtin_agents;
pub use dual::{AgentRole, DualAgentResult, RawMessage, SessionType, SharedConversation};
pub use executor::AgentExecutor;
pub use prompt::SystemPromptBuilder;
pub use registry::AgentRegistry;
pub use runtime::DualAgentRuntime;
pub use types::{AgentInfo, AgentMode, AgentModel, AgentPermissions, Permission};
