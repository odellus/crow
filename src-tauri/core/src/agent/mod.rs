//! Agent module - manages different agent types and execution
//!
//! This module provides:
//! - Agent type definitions (AgentInfo, AgentMode, Permission)
//! - Built-in agents (general, build, plan, arbiter, planner, architect)
//! - Agent registry for managing available agents
//! - Dual-agent architecture (executor + arbiter for subagents)
//! - Primary dual-agent architecture (planner + architect for main loop)
//! - Agent execution with ReACT loop
//!
//! Based on OpenCode's agent system from:
//! opencode/packages/opencode/src/agent/

pub mod builtins;
pub mod doom_loop;
pub mod dual;
pub mod executor;
pub mod primary_dual;
pub mod prompt;
pub mod registry;
pub mod types;

pub use builtins::get_builtin_agents;
pub use doom_loop::DoomLoopDetector;
pub use dual::{DualAgentResult, DualAgentRuntime};
pub use executor::{AgentExecutor, ExecutionEvent};
pub use primary_dual::{PrimaryDualEvent, PrimaryDualResult, PrimaryDualRuntime};
pub use prompt::SystemPromptBuilder;
pub use registry::AgentRegistry;
pub use types::{AgentInfo, AgentMode, AgentModel, AgentPermissions, Permission};
