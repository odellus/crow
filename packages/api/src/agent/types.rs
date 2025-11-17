//! Agent type definitions matching OpenCode's Agent.Info
//! Based on opencode/packages/opencode/src/agent/agent.ts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Permission level for tool execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    /// Always allow this action
    Allow,
    /// Always deny this action
    Deny,
    /// Ask user for permission before executing
    Ask,
}

impl Default for Permission {
    fn default() -> Self {
        Permission::Ask
    }
}

/// Agent permissions configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentPermissions {
    /// Permission for file editing
    pub edit: Permission,

    /// Bash command permissions (pattern -> permission)
    /// Patterns support wildcards: "git *" matches all git commands
    pub bash: HashMap<String, Permission>,

    /// Permission for web fetching
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webfetch: Option<Permission>,

    /// Permission for doom loop detection override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doom_loop: Option<Permission>,

    /// Permission for accessing directories outside project
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_directory: Option<Permission>,
}

impl Default for AgentPermissions {
    fn default() -> Self {
        let mut bash = HashMap::new();
        bash.insert("*".to_string(), Permission::Allow);

        Self {
            edit: Permission::Allow,
            bash,
            webfetch: Some(Permission::Allow),
            doom_loop: Some(Permission::Ask),
            external_directory: Some(Permission::Ask),
        }
    }
}

/// Agent mode - controls where agent can be used
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    /// Can only be used as a sub-agent (delegated to)
    Subagent,
    /// Can only be used as primary agent (top-level)
    Primary,
    /// Can be used as both primary and sub-agent
    All,
}

impl Default for AgentMode {
    fn default() -> Self {
        AgentMode::All
    }
}

/// Model configuration for agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentModel {
    #[serde(rename = "modelID")]
    pub model_id: String,

    #[serde(rename = "providerID")]
    pub provider_id: String,
}

/// Complete agent configuration (matches OpenCode's Agent.Info)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent identifier (e.g., "build", "supervisor", "general")
    pub name: String,

    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Agent mode (primary/subagent/all)
    pub mode: AgentMode,

    /// Is this a built-in agent?
    #[serde(rename = "builtIn")]
    pub built_in: bool,

    /// Sampling temperature (0.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Top-p sampling (0.0 - 1.0)
    #[serde(rename = "topP", skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// UI color for this agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Permission configuration
    pub permission: AgentPermissions,

    /// Model override (if None, uses default model)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<AgentModel>,

    /// Custom system prompt override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Tool allowlist/denylist (tool_name -> enabled)
    pub tools: HashMap<String, bool>,

    /// Additional agent-specific options
    #[serde(flatten)]
    pub options: HashMap<String, serde_json::Value>,
}

impl AgentInfo {
    /// Create a new agent with default settings
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            mode: AgentMode::All,
            built_in: false,
            temperature: None,
            top_p: None,
            color: None,
            permission: AgentPermissions::default(),
            model: None,
            prompt: None,
            tools: HashMap::new(),
            options: HashMap::new(),
        }
    }

    /// Check if a specific tool is enabled for this agent
    pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
        // If tools map is empty, allow all tools (build agent case)
        if self.tools.is_empty() {
            return true;
        }

        // Check explicit allow/deny
        // In OpenCode: undefined (not in map) = allow, false = deny, true = allow
        // So we return true (allow) for tools not in the map, unless explicitly set to false
        self.tools.get(tool_name).copied().unwrap_or(true)
    }

    /// Check if agent can be used in primary mode
    pub fn is_primary(&self) -> bool {
        matches!(self.mode, AgentMode::Primary | AgentMode::All)
    }

    /// Check if agent can be used as subagent
    pub fn is_subagent(&self) -> bool {
        matches!(self.mode, AgentMode::Subagent | AgentMode::All)
    }

    /// Get effective temperature (with default fallback)
    pub fn get_temperature(&self) -> f32 {
        self.temperature.unwrap_or(0.7)
    }

    /// Get effective top_p (with default fallback)
    pub fn get_top_p(&self) -> f32 {
        self.top_p.unwrap_or(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_info_creation() {
        let agent = AgentInfo::new("test-agent");
        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.built_in, false);
        assert_eq!(agent.mode, AgentMode::All);
    }

    #[test]
    fn test_tool_enabled() {
        let mut agent = AgentInfo::new("test");
        agent.tools.insert("bash".to_string(), true);
        agent.tools.insert("edit".to_string(), false);

        assert!(agent.is_tool_enabled("bash")); // Explicitly enabled
        assert!(!agent.is_tool_enabled("edit")); // Explicitly disabled
        assert!(agent.is_tool_enabled("unknown")); // Not in map = allow by default (matches OpenCode)

        // Empty tools map allows everything
        let agent2 = AgentInfo::new("test2");
        assert!(agent2.is_tool_enabled("anything"));
    }

    #[test]
    fn test_agent_modes() {
        let mut agent = AgentInfo::new("test");

        agent.mode = AgentMode::All;
        assert!(agent.is_primary());
        assert!(agent.is_subagent());

        agent.mode = AgentMode::Primary;
        assert!(agent.is_primary());
        assert!(!agent.is_subagent());

        agent.mode = AgentMode::Subagent;
        assert!(!agent.is_primary());
        assert!(agent.is_subagent());
    }
}
