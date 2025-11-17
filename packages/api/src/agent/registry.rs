//! Agent registry for managing and loading agents
//! Based on opencode/packages/opencode/src/agent/agent.ts

use super::builtins::get_builtin_agents;
use super::types::{AgentInfo, AgentMode};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent registry - manages all available agents
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
}

impl AgentRegistry {
    /// Create a new agent registry with built-in agents
    pub fn new() -> Self {
        let agents = get_builtin_agents();
        Self {
            agents: Arc::new(RwLock::new(agents)),
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
        assert_eq!(count, 6); // 6 built-in agents (general, build, plan, supervisor, architect, discriminator)
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

        // build (all), plan (primary), supervisor (primary), architect (primary)
        assert!(primary.len() >= 4);

        let names: Vec<String> = primary.iter().map(|a| a.name.clone()).collect();
        assert!(names.contains(&"build".to_string()));
        assert!(names.contains(&"supervisor".to_string()));
    }

    #[tokio::test]
    async fn test_get_subagents() {
        let registry = AgentRegistry::new();
        let subagents = registry.get_subagents().await;

        // general (subagent), build (all)
        assert!(subagents.len() >= 2);

        let names: Vec<String> = subagents.iter().map(|a| a.name.clone()).collect();
        assert!(names.contains(&"general".to_string()));
        assert!(names.contains(&"build".to_string()));
    }

    #[tokio::test]
    async fn test_register_custom_agent() {
        let registry = AgentRegistry::new();

        let custom = AgentInfo::new("custom-agent");
        registry.register(custom).await;

        let count = registry.count().await;
        assert_eq!(count, 7); // 6 built-in + 1 custom

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

        assert_eq!(ids.len(), 6);
        assert!(ids.contains(&"general".to_string()));
        assert!(ids.contains(&"build".to_string()));
        assert!(ids.contains(&"plan".to_string()));
        assert!(ids.contains(&"supervisor".to_string()));
        assert!(ids.contains(&"architect".to_string()));
        assert!(ids.contains(&"discriminator".to_string()));
    }
}
