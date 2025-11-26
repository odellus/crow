//! Configuration loading and merging
//! Matches OpenCode's config loading patterns

use super::types::{AgentConfig, Config};
use crate::global::GlobalPaths;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Configuration loader with merging support
pub struct ConfigLoader {
    paths: GlobalPaths,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self {
            paths: GlobalPaths::new(),
        }
    }

    /// Load configuration from all sources with merging
    /// Order (lowest to highest precedence):
    /// 1. Global config (~/.config/crow/crow.json)
    /// 2. Project config (crow.json in project root)
    /// 3. .crow directory config
    /// 4. Environment variable (CROW_CONFIG_CONTENT)
    pub fn load(&self) -> Result<Config, String> {
        let mut config = Config::default();

        // 1. Load global config
        let global_path = self.paths.config.join("crow.json");
        if global_path.exists() {
            let global = self.load_file(&global_path)?;
            config = self.merge(config, global);
        }

        // 2. Load project config (search upward to git root)
        let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
        if let Some(project_path) = self.find_project_config(&cwd)? {
            let project = self.load_file(&project_path)?;
            config = self.merge(config, project);
        }

        // 3. Load from .crow directory
        let crow_dir = cwd.join(".crow");
        if crow_dir.exists() {
            // Load config.jsonc or crow.json from .crow directory
            let config_jsonc = crow_dir.join("config.jsonc");
            let crow_config = crow_dir.join("crow.json");
            let config_path = if config_jsonc.exists() {
                Some(config_jsonc)
            } else if crow_config.exists() {
                Some(crow_config)
            } else {
                None
            };

            if let Some(path) = config_path {
                let dir_config = self.load_file(&path)?;
                config = self.merge(config, dir_config);
            }

            // Load agent/*.md files
            let agent_dir = crow_dir.join("agent");
            if agent_dir.exists() {
                let agents = self.load_agent_directory(&agent_dir)?;
                if !agents.is_empty() {
                    let mut existing_agents = config.agent.unwrap_or_default();
                    for (name, agent) in agents {
                        existing_agents.insert(name, agent);
                    }
                    config.agent = Some(existing_agents);
                }
            }

            // Load command/*.md files
            let command_dir = crow_dir.join("command");
            if command_dir.exists() {
                let commands = self.load_command_directory(&command_dir)?;
                if !commands.is_empty() {
                    let mut existing_commands = config.command.unwrap_or_default();
                    for (name, cmd) in commands {
                        existing_commands.insert(name, cmd);
                    }
                    config.command = Some(existing_commands);
                }
            }
        }

        // 4. Environment variable override
        if let Ok(content) = std::env::var("CROW_CONFIG_CONTENT") {
            let env_config: Config = serde_json::from_str(&content)
                .map_err(|e| format!("Invalid CROW_CONFIG_CONTENT: {}", e))?;
            config = self.merge(config, env_config);
        }

        // Substitute variables ({env:VAR}, {file:path})
        self.substitute_variables(&mut config)?;

        Ok(config)
    }

    /// Load config from a JSON file
    fn load_file(&self, path: &Path) -> Result<Config, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        // Support JSONC (JSON with comments) by stripping comments
        let content = self.strip_json_comments(&content);

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
    }

    /// Strip C-style comments from JSON (for JSONC support)
    fn strip_json_comments(&self, content: &str) -> String {
        let mut result = String::new();
        let mut in_string = false;
        let mut in_line_comment = false;
        let mut in_block_comment = false;
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            if in_line_comment {
                if ch == '\n' {
                    in_line_comment = false;
                    result.push(ch);
                }
                continue;
            }

            if in_block_comment {
                if ch == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    in_block_comment = false;
                }
                continue;
            }

            if ch == '"' && !in_string {
                in_string = true;
                result.push(ch);
                continue;
            }

            if ch == '"' && in_string {
                // Check for escaped quote
                let escaped = result.ends_with('\\');
                if !escaped {
                    in_string = false;
                }
                result.push(ch);
                continue;
            }

            if in_string {
                result.push(ch);
                continue;
            }

            if ch == '/' {
                if chars.peek() == Some(&'/') {
                    chars.next();
                    in_line_comment = true;
                    continue;
                }
                if chars.peek() == Some(&'*') {
                    chars.next();
                    in_block_comment = true;
                    continue;
                }
            }

            result.push(ch);
        }

        result
    }

    /// Find project config file (search upward to git root)
    fn find_project_config(&self, start: &Path) -> Result<Option<PathBuf>, String> {
        let mut current = start.to_path_buf();

        loop {
            // Check for crow.json or crow.jsonc
            for filename in &["crow.json", "crow.jsonc"] {
                let candidate = current.join(filename);
                if candidate.exists() {
                    return Ok(Some(candidate));
                }
            }

            // Stop at git root
            if current.join(".git").exists() {
                break;
            }

            // Stop at filesystem root
            if !current.pop() {
                break;
            }
        }

        Ok(None)
    }

    /// Load agent definitions from markdown files
    fn load_agent_directory(&self, dir: &Path) -> Result<HashMap<String, AgentConfig>, String> {
        let mut agents = HashMap::new();

        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read {}: {}", dir.display(), e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                let (name, agent) = self.load_agent_md(&path)?;
                agents.insert(name, agent);
            }
        }

        Ok(agents)
    }

    /// Load a single agent from markdown file with YAML frontmatter
    fn load_agent_md(&self, path: &Path) -> Result<(String, AgentConfig), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let (frontmatter, body) = self.parse_frontmatter(&content)?;

        let mut agent: AgentConfig = if frontmatter.is_empty() {
            AgentConfig::default()
        } else {
            serde_yaml::from_str(&frontmatter)
                .map_err(|e| format!("Invalid frontmatter in {}: {}", path.display(), e))?
        };

        // Body becomes the prompt
        let body = body.trim();
        if !body.is_empty() {
            agent.prompt = Some(body.to_string());
        }

        // Name from filename (without extension)
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Invalid filename: {}", path.display()))?
            .to_string();

        Ok((name, agent))
    }

    /// Load command definitions from markdown files
    fn load_command_directory(
        &self,
        dir: &Path,
    ) -> Result<HashMap<String, super::types::CommandConfig>, String> {
        let mut commands = HashMap::new();

        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read {}: {}", dir.display(), e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                let (name, cmd) = self.load_command_md(&path)?;
                commands.insert(name, cmd);
            }
        }

        Ok(commands)
    }

    /// Load a single command from markdown file
    fn load_command_md(
        &self,
        path: &Path,
    ) -> Result<(String, super::types::CommandConfig), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let (frontmatter, body) = self.parse_frontmatter(&content)?;

        // Parse frontmatter for metadata
        #[derive(serde::Deserialize, Default)]
        struct CommandFrontmatter {
            description: Option<String>,
            agent: Option<String>,
            model: Option<String>,
        }

        let meta: CommandFrontmatter = if frontmatter.is_empty() {
            CommandFrontmatter::default()
        } else {
            serde_yaml::from_str(&frontmatter)
                .map_err(|e| format!("Invalid frontmatter in {}: {}", path.display(), e))?
        };

        let cmd = super::types::CommandConfig {
            template: body.trim().to_string(),
            description: meta.description,
            agent: meta.agent,
            model: meta.model,
        };

        // Name from filename
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Invalid filename: {}", path.display()))?
            .to_string();

        Ok((name, cmd))
    }

    /// Parse YAML frontmatter from markdown
    fn parse_frontmatter(&self, content: &str) -> Result<(String, String), String> {
        let content = content.trim();

        if !content.starts_with("---") {
            return Ok((String::new(), content.to_string()));
        }

        // Find closing ---
        let after_first = &content[3..];
        if let Some(end_idx) = after_first.find("\n---") {
            let frontmatter = after_first[..end_idx].trim().to_string();
            let body = after_first[end_idx + 4..].to_string();
            Ok((frontmatter, body))
        } else {
            Err("Unclosed frontmatter (missing closing ---)".to_string())
        }
    }

    /// Deep merge two configs (other takes precedence)
    fn merge(&self, base: Config, other: Config) -> Config {
        // Convert to JSON values for deep merge
        let base_json = serde_json::to_value(&base).unwrap_or(serde_json::Value::Null);
        let other_json = serde_json::to_value(&other).unwrap_or(serde_json::Value::Null);

        let merged = self.merge_json(base_json, other_json);

        serde_json::from_value(merged).unwrap_or(base)
    }

    /// Recursively merge JSON values
    fn merge_json(&self, base: serde_json::Value, other: serde_json::Value) -> serde_json::Value {
        use serde_json::Value;

        match (base, other) {
            (Value::Object(mut base_map), Value::Object(other_map)) => {
                for (key, other_val) in other_map {
                    // Skip null values from other - don't override base with null
                    if other_val.is_null() {
                        continue;
                    }
                    let merged = if let Some(base_val) = base_map.remove(&key) {
                        self.merge_json(base_val, other_val)
                    } else {
                        other_val
                    };
                    base_map.insert(key, merged);
                }
                Value::Object(base_map)
            }
            (base, Value::Null) => base, // Keep base if other is null
            (_, other) => other,         // Other types: other wins
        }
    }

    /// Substitute {env:VAR} and {file:path} patterns
    fn substitute_variables(&self, config: &mut Config) -> Result<(), String> {
        // Convert to JSON, substitute, convert back
        let mut json = serde_json::to_value(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        self.substitute_in_value(&mut json)?;

        *config = serde_json::from_value(json)
            .map_err(|e| format!("Failed to deserialize config: {}", e))?;

        Ok(())
    }

    /// Recursively substitute variables in JSON value
    fn substitute_in_value(&self, value: &mut serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::String(s) => {
                *s = self.substitute_string(s)?;
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.substitute_in_value(item)?;
                }
            }
            serde_json::Value::Object(map) => {
                for (_, v) in map {
                    self.substitute_in_value(v)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Substitute variables in a string
    fn substitute_string(&self, s: &str) -> Result<String, String> {
        let mut result = s.to_string();

        // Substitute {env:VAR_NAME}
        while let Some(start) = result.find("{env:") {
            let end = result[start..].find('}').ok_or("Unclosed {env:} pattern")?;
            let var_name = &result[start + 5..start + end];
            let value = std::env::var(var_name).unwrap_or_default();
            result = format!(
                "{}{}{}",
                &result[..start],
                value,
                &result[start + end + 1..]
            );
        }

        // Substitute {file:path}
        while let Some(start) = result.find("{file:") {
            let end = result[start..]
                .find('}')
                .ok_or("Unclosed {file:} pattern")?;
            let file_path = &result[start + 6..start + end];

            // Expand ~ to home directory
            let expanded_path = if file_path.starts_with("~/") {
                if let Ok(home) = std::env::var("HOME") {
                    format!("{}{}", home, &file_path[1..])
                } else {
                    file_path.to_string()
                }
            } else {
                file_path.to_string()
            };

            let content = std::fs::read_to_string(&expanded_path)
                .map_err(|e| format!("Failed to read {}: {}", expanded_path, e))?;

            result = format!(
                "{}{}{}",
                &result[..start],
                content.trim(),
                &result[start + end + 1..]
            );
        }

        Ok(result)
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_json_comments() {
        let loader = ConfigLoader::new();

        // Line comments
        let input = r#"{
            "key": "value" // comment
        }"#;
        let result = loader.strip_json_comments(input);
        assert!(!result.contains("comment"));
        assert!(result.contains("value"));

        // Block comments
        let input = r#"{
            "key": /* comment */ "value"
        }"#;
        let result = loader.strip_json_comments(input);
        assert!(!result.contains("comment"));

        // Comments in strings should be preserved
        let input = r#"{"key": "value // not a comment"}"#;
        let result = loader.strip_json_comments(input);
        assert!(result.contains("// not a comment"));
    }

    #[test]
    fn test_parse_frontmatter() {
        let loader = ConfigLoader::new();

        let content = r#"---
description: Test agent
mode: subagent
---

This is the prompt body.
"#;

        let (frontmatter, body) = loader.parse_frontmatter(content).unwrap();
        assert!(frontmatter.contains("description: Test agent"));
        assert!(body.contains("This is the prompt body"));
    }

    #[test]
    fn test_merge_configs() {
        let loader = ConfigLoader::new();

        let base = Config {
            theme: Some("default".to_string()),
            model: Some("old-model".to_string()),
            ..Default::default()
        };

        let other = Config {
            model: Some("new-model".to_string()),
            username: Some("user".to_string()),
            ..Default::default()
        };

        let merged = loader.merge(base, other);

        assert_eq!(merged.theme, Some("default".to_string())); // From base
        assert_eq!(merged.model, Some("new-model".to_string())); // Overridden
        assert_eq!(merged.username, Some("user".to_string())); // From other
    }

    #[test]
    fn test_substitute_env() {
        let loader = ConfigLoader::new();

        std::env::set_var("TEST_VAR", "test_value");

        let result = loader.substitute_string("{env:TEST_VAR}").unwrap();
        assert_eq!(result, "test_value");

        let result = loader
            .substitute_string("prefix_{env:TEST_VAR}_suffix")
            .unwrap();
        assert_eq!(result, "prefix_test_value_suffix");

        std::env::remove_var("TEST_VAR");
    }
}
