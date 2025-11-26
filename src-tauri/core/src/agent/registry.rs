//! Agent registry for managing and loading agents
//! Based on opencode/packages/opencode/src/agent/agent.ts
//!
//! Loads agents from:
//! 1. Built-in agents (general, build, plan, supervisor, architect, discriminator)
//! 2. Project config: `.crow/agent/*.md` files
//! 3. Global config: `~/.config/crow/agent/*.md` files

use super::builtins::get_builtin_agents;
use super::types::{AgentInfo, AgentMode, AgentPermissions, Permission};
use crate::global::GlobalPaths;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Agent registry - manages all available agents
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
}

impl AgentRegistry {
    /// Create a new agent registry with built-in agents only
    pub fn new() -> Self {
        let agents = get_builtin_agents();
        Self {
            agents: Arc::new(RwLock::new(agents)),
        }
    }

    /// Create registry and load agents from config directories
    pub async fn new_with_config(working_dir: &Path) -> Self {
        let mut agents = get_builtin_agents();

        // Load from global config: ~/.config/crow/agent/*.md
        let global_paths = GlobalPaths::new();
        let global_agent_dir = global_paths.config.join("agent");
        if let Ok(loaded) = Self::load_agents_from_dir(&global_agent_dir).await {
            for agent in loaded {
                debug!("Loaded global agent: {}", agent.name);
                agents.insert(agent.name.clone(), agent);
            }
        }

        // Load from project config: .crow/agent/*.md (higher priority)
        let project_agent_dir = working_dir.join(".crow").join("agent");
        if let Ok(loaded) = Self::load_agents_from_dir(&project_agent_dir).await {
            for agent in loaded {
                debug!("Loaded project agent: {}", agent.name);
                agents.insert(agent.name.clone(), agent);
            }
        }

        Self {
            agents: Arc::new(RwLock::new(agents)),
        }
    }

    /// Load agents from markdown files in a directory
    async fn load_agents_from_dir(dir: &Path) -> Result<Vec<AgentInfo>, String> {
        let mut agents = Vec::new();

        if !dir.exists() {
            return Ok(agents);
        }

        let entries =
            std::fs::read_dir(dir).map_err(|e| format!("Failed to read agent directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                match Self::load_agent_from_file(&path).await {
                    Ok(agent) => agents.push(agent),
                    Err(e) => warn!("Failed to load agent from {:?}: {}", path, e),
                }
            }
        }

        Ok(agents)
    }

    /// Load a single agent from a markdown file
    /// Format:
    /// ```markdown
    /// ---
    /// description: When to use this agent
    /// mode: subagent | primary | all
    /// temperature: 0.7
    /// top_p: 1.0
    /// tools:
    ///   bash: true
    ///   edit: false
    /// permission:
    ///   edit: allow | deny | ask
    ///   bash:
    ///     "*": allow
    ///     "rm *": deny
    /// ---
    ///
    /// Custom system prompt content here
    /// ```
    async fn load_agent_from_file(path: &Path) -> Result<AgentInfo, String> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Extract agent name from filename (without .md extension)
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid filename")?
            .to_string();

        // Parse frontmatter and content
        let (frontmatter, prompt) = Self::parse_markdown(&content)?;

        // Build agent from frontmatter
        let mut agent = AgentInfo::new(&name);
        agent.built_in = false;

        if let Some(desc) = frontmatter.get("description").and_then(|v| v.as_str()) {
            agent.description = Some(desc.to_string());
        }

        if let Some(mode) = frontmatter.get("mode").and_then(|v| v.as_str()) {
            agent.mode = match mode {
                "subagent" => AgentMode::Subagent,
                "primary" => AgentMode::Primary,
                _ => AgentMode::All,
            };
        }

        if let Some(temp) = frontmatter.get("temperature").and_then(|v| v.as_f64()) {
            agent.temperature = Some(temp as f32);
        }

        if let Some(top_p) = frontmatter.get("top_p").and_then(|v| v.as_f64()) {
            agent.top_p = Some(top_p as f32);
        }

        if let Some(color) = frontmatter.get("color").and_then(|v| v.as_str()) {
            agent.color = Some(color.to_string());
        }

        // Parse tools map
        if let Some(tools) = frontmatter.get("tools").and_then(|v| v.as_object()) {
            for (tool_name, enabled) in tools {
                if let Some(enabled) = enabled.as_bool() {
                    agent.tools.insert(tool_name.clone(), enabled);
                }
            }
        }

        // Parse permissions
        if let Some(perms) = frontmatter.get("permission").and_then(|v| v.as_object()) {
            agent.permission = Self::parse_permissions(perms);
        }

        // Set custom prompt if content exists (after frontmatter)
        if !prompt.trim().is_empty() {
            agent.prompt = Some(prompt);
        }

        Ok(agent)
    }

    /// Parse markdown with YAML frontmatter
    fn parse_markdown(
        content: &str,
    ) -> Result<(serde_json::Map<String, serde_json::Value>, String), String> {
        let content = content.trim();

        if !content.starts_with("---") {
            // No frontmatter, entire content is prompt
            return Ok((serde_json::Map::new(), content.to_string()));
        }

        // Find end of frontmatter
        let rest = &content[3..];
        let end = rest.find("---").ok_or("Unclosed frontmatter")?;
        let yaml_content = &rest[..end].trim();
        let prompt = rest[end + 3..].trim().to_string();

        // Parse YAML as JSON (serde_yaml -> serde_json)
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_content)
            .map_err(|e| format!("Invalid YAML frontmatter: {}", e))?;

        let json_value: serde_json::Value = serde_json::to_value(yaml_value)
            .map_err(|e| format!("Failed to convert YAML to JSON: {}", e))?;

        let map = json_value
            .as_object()
            .ok_or("Frontmatter must be a YAML object")?
            .clone();

        Ok((map, prompt))
    }

    /// Parse permissions from frontmatter
    fn parse_permissions(perms: &serde_json::Map<String, serde_json::Value>) -> AgentPermissions {
        let mut result = AgentPermissions::default();

        if let Some(edit) = perms.get("edit").and_then(|v| v.as_str()) {
            result.edit = Self::parse_permission(edit);
        }

        if let Some(bash) = perms.get("bash") {
            if let Some(bash_str) = bash.as_str() {
                // Simple permission for all bash
                result.bash.clear();
                result
                    .bash
                    .insert("*".to_string(), Self::parse_permission(bash_str));
            } else if let Some(bash_map) = bash.as_object() {
                // Pattern-based permissions
                result.bash.clear();
                for (pattern, perm) in bash_map {
                    if let Some(perm_str) = perm.as_str() {
                        result
                            .bash
                            .insert(pattern.clone(), Self::parse_permission(perm_str));
                    }
                }
            }
        }

        if let Some(webfetch) = perms.get("webfetch").and_then(|v| v.as_str()) {
            result.webfetch = Some(Self::parse_permission(webfetch));
        }

        if let Some(doom_loop) = perms.get("doom_loop").and_then(|v| v.as_str()) {
            result.doom_loop = Some(Self::parse_permission(doom_loop));
        }

        if let Some(external) = perms.get("external_directory").and_then(|v| v.as_str()) {
            result.external_directory = Some(Self::parse_permission(external));
        }

        result
    }

    fn parse_permission(s: &str) -> Permission {
        match s.to_lowercase().as_str() {
            "allow" => Permission::Allow,
            "deny" => Permission::Deny,
            _ => Permission::Ask,
        }
    }

    /// Get an agent by ID
    pub async fn get(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// Get all agents
    pub async fn get_all(&self) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// Get all primary agents (can be used as top-level agents)
    pub async fn get_primary_agents(&self) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|a| a.is_primary())
            .cloned()
            .collect()
    }

    /// Get all subagents (can be delegated to)
    pub async fn get_subagents(&self) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|a| a.is_subagent())
            .cloned()
            .collect()
    }

    /// Get agents by mode
    pub async fn get_by_mode(&self, mode: AgentMode) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|a| match mode {
                AgentMode::Primary => a.is_primary(),
                AgentMode::Subagent => a.is_subagent(),
                AgentMode::All => true,
            })
            .cloned()
            .collect()
    }

    /// Register a new custom agent
    pub async fn register(&self, agent: AgentInfo) {
        let mut agents = self.agents.write().await;
        agents.insert(agent.name.clone(), agent);
    }

    /// Remove an agent (built-in agents cannot be removed)
    pub async fn unregister(&self, agent_id: &str) -> Result<(), String> {
        let mut agents = self.agents.write().await;

        if let Some(agent) = agents.get(agent_id) {
            if agent.built_in {
                return Err(format!("Cannot remove built-in agent: {}", agent_id));
            }
        }

        agents.remove(agent_id);
        Ok(())
    }

    /// Check if an agent exists
    pub async fn exists(&self, agent_id: &str) -> bool {
        let agents = self.agents.read().await;
        agents.contains_key(agent_id)
    }

    /// Get agent count
    pub async fn count(&self) -> usize {
        let agents = self.agents.read().await;
        agents.len()
    }

    /// List all agent IDs
    pub async fn list_ids(&self) -> Vec<String> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = AgentRegistry::new();
        let count = registry.count().await;
        assert_eq!(count, 4); // 4 built-in agents (general, build, plan, arbiter)
    }

    #[tokio::test]
    async fn test_get_agent() {
        let registry = AgentRegistry::new();
        let agent = registry.get("build").await;
        assert!(agent.is_some());
        assert_eq!(agent.unwrap().name, "build");
    }

    #[tokio::test]
    async fn test_get_nonexistent_agent() {
        let registry = AgentRegistry::new();
        let agent = registry.get("nonexistent").await;
        assert!(agent.is_none());
    }

    #[tokio::test]
    async fn test_get_primary_agents() {
        let registry = AgentRegistry::new();
        let primary = registry.get_primary_agents().await;

        // build (primary), plan (primary)
        assert_eq!(primary.len(), 2);

        let names: Vec<String> = primary.iter().map(|a| a.name.clone()).collect();
        assert!(names.contains(&"build".to_string()));
        assert!(names.contains(&"plan".to_string()));
    }

    #[tokio::test]
    async fn test_get_subagents() {
        let registry = AgentRegistry::new();
        let subagents = registry.get_subagents().await;

        // general (subagent), arbiter (subagent)
        assert_eq!(subagents.len(), 2);

        let names: Vec<String> = subagents.iter().map(|a| a.name.clone()).collect();
        assert!(names.contains(&"general".to_string()));
        assert!(names.contains(&"arbiter".to_string()));
    }

    #[tokio::test]
    async fn test_register_custom_agent() {
        let registry = AgentRegistry::new();

        let custom = AgentInfo::new("custom-agent");
        registry.register(custom).await;

        let count = registry.count().await;
        assert_eq!(count, 5); // 4 built-in + 1 custom

        let agent = registry.get("custom-agent").await;
        assert!(agent.is_some());
    }

    #[tokio::test]
    async fn test_cannot_remove_builtin() {
        let registry = AgentRegistry::new();
        let result = registry.unregister("build").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_can_remove_custom() {
        let registry = AgentRegistry::new();

        let custom = AgentInfo::new("custom");
        registry.register(custom).await;

        let result = registry.unregister("custom").await;
        assert!(result.is_ok());

        let exists = registry.exists("custom").await;
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_list_ids() {
        let registry = AgentRegistry::new();
        let ids = registry.list_ids().await;

        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&"general".to_string()));
        assert!(ids.contains(&"build".to_string()));
        assert!(ids.contains(&"plan".to_string()));
        assert!(ids.contains(&"arbiter".to_string()));
    }
}
