//! Configuration types for Crow
//! Based on OpenCode's config/config.ts schema

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// JSON schema reference
    #[serde(rename = "$schema")]
    pub schema: Option<String>,

    /// Theme name
    pub theme: Option<String>,

    /// Default model in "provider/model" format
    pub model: Option<String>,

    /// Small model for lightweight tasks
    pub small_model: Option<String>,

    /// Custom username display
    pub username: Option<String>,

    /// Provider configurations
    pub provider: Option<HashMap<String, ProviderConfig>>,

    /// Agent configurations
    pub agent: Option<HashMap<String, AgentConfig>>,

    /// Global tool enable/disable settings
    pub tools: Option<HashMap<String, bool>>,

    /// Global permissions
    pub permission: Option<PermissionConfig>,

    /// Custom commands
    pub command: Option<HashMap<String, CommandConfig>>,

    /// Glob patterns for instruction files
    pub instructions: Option<Vec<String>>,

    /// Disabled provider names
    pub disabled_providers: Option<Vec<String>>,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    /// Model configurations
    pub models: Option<HashMap<String, ModelConfig>>,

    /// Provider options
    pub options: Option<ProviderOptions>,
}

/// Model-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelConfig {
    /// Display label
    pub label: Option<String>,
}

/// Provider connection options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderOptions {
    /// API key (can use {env:VAR} syntax)
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,

    /// Base URL for API
    #[serde(rename = "baseURL")]
    pub base_url: Option<String>,

    /// Request timeout in milliseconds
    pub timeout: Option<u64>,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    /// Agent name (auto-filled from key or filename)
    pub name: Option<String>,

    /// Description of when to use this agent
    pub description: Option<String>,

    /// Custom system prompt
    pub prompt: Option<String>,

    /// Override model for this agent
    pub model: Option<String>,

    /// Agent mode: "primary", "subagent", or "all"
    pub mode: Option<String>,

    /// Temperature for LLM (0.0-1.0)
    pub temperature: Option<f32>,

    /// Top-p for LLM
    pub top_p: Option<f32>,

    /// Tool enable/disable overrides
    pub tools: Option<HashMap<String, bool>>,

    /// Permission overrides for this agent
    pub permission: Option<PermissionConfig>,

    /// Display color (hex format)
    pub color: Option<String>,

    /// Whether to disable this agent
    pub disable: Option<bool>,
}

/// Permission configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionConfig {
    /// File edit permission
    pub edit: Option<String>,

    /// Bash command permission (simple or targeted)
    pub bash: Option<BashPermission>,

    /// Web fetch permission
    pub webfetch: Option<String>,

    /// Doom loop detection permission
    pub doom_loop: Option<String>,

    /// External directory access permission
    pub external_directory: Option<String>,
}

/// Bash permission - either simple ("allow"/"ask"/"deny") or targeted by command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BashPermission {
    /// Simple permission for all bash commands
    Simple(String),
    /// Targeted permissions by command pattern
    Targeted(HashMap<String, String>),
}

impl Default for BashPermission {
    fn default() -> Self {
        BashPermission::Simple("ask".to_string())
    }
}

/// Command configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    /// Prompt template (required)
    pub template: String,

    /// Description for UI
    pub description: Option<String>,

    /// Agent to execute command
    pub agent: Option<String>,

    /// Override model for command
    pub model: Option<String>,
}

impl Config {
    /// Parse model string in "provider/model" format
    pub fn parse_model(model_str: &str) -> (String, String) {
        if let Some((provider, model)) = model_str.split_once('/') {
            (provider.to_string(), model.to_string())
        } else {
            // Default provider if not specified
            ("default".to_string(), model_str.to_string())
        }
    }

    /// Get the default model, falling back to hardcoded default
    pub fn get_default_model(&self) -> (String, String) {
        match &self.model {
            Some(model_str) => Self::parse_model(model_str),
            None => ("moonshotai".to_string(), "kimi-k2-thinking".to_string()),
        }
    }

    /// Check if a provider is disabled
    pub fn is_provider_disabled(&self, provider: &str) -> bool {
        self.disabled_providers
            .as_ref()
            .map(|list| list.iter().any(|p| p == provider))
            .unwrap_or(false)
    }
}

impl AgentConfig {
    /// Get mode with default fallback
    pub fn get_mode(&self) -> &str {
        self.mode.as_deref().unwrap_or("all")
    }

    /// Get temperature with default fallback
    pub fn get_temperature(&self) -> Option<f32> {
        self.temperature
    }

    /// Check if agent is disabled
    pub fn is_disabled(&self) -> bool {
        self.disable.unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_model() {
        let (provider, model) = Config::parse_model("anthropic/claude-sonnet-4");
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-sonnet-4");

        let (provider, model) = Config::parse_model("gpt-4o");
        assert_eq!(provider, "default");
        assert_eq!(model, "gpt-4o");
    }

    #[test]
    fn test_deserialize_config() {
        let json = r#"{
            "model": "moonshotai/kimi-k2-thinking",
            "theme": "dracula",
            "permission": {
                "edit": "allow",
                "bash": {
                    "git push": "ask",
                    "*": "allow"
                }
            },
            "agent": {
                "reviewer": {
                    "mode": "subagent",
                    "temperature": 0.1,
                    "tools": {
                        "write": false,
                        "edit": false
                    }
                }
            }
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.model,
            Some("moonshotai/kimi-k2-thinking".to_string())
        );
        assert_eq!(config.theme, Some("dracula".to_string()));

        let agents = config.agent.unwrap();
        let reviewer = agents.get("reviewer").unwrap();
        assert_eq!(reviewer.mode, Some("subagent".to_string()));
        assert_eq!(reviewer.temperature, Some(0.1));
    }

    #[test]
    fn test_bash_permission_variants() {
        // Simple permission
        let json = r#"{"bash": "allow"}"#;
        let perm: PermissionConfig = serde_json::from_str(json).unwrap();
        match perm.bash.unwrap() {
            BashPermission::Simple(s) => assert_eq!(s, "allow"),
            _ => panic!("Expected Simple"),
        }

        // Targeted permission
        let json = r#"{"bash": {"git push": "ask", "*": "allow"}}"#;
        let perm: PermissionConfig = serde_json::from_str(json).unwrap();
        match perm.bash.unwrap() {
            BashPermission::Targeted(map) => {
                assert_eq!(map.get("git push"), Some(&"ask".to_string()));
                assert_eq!(map.get("*"), Some(&"allow".to_string()));
            }
            _ => panic!("Expected Targeted"),
        }
    }
}
