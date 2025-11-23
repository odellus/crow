# Crow Configuration System Plan

**Goal:** Replicate OpenCode's configuration system in Crow

---

## Current State in Crow

### What Exists
- `GlobalPaths` - XDG directory structure (`~/.local/share/crow/`, etc.)
- `ProviderConfig` - Hardcoded Moonshot/OpenAI configs
- `auth.json` - API key storage at `~/.local/share/crow/auth.json`
- `AgentInfo` with permissions - Agent definitions with tool/bash permissions

### What's Missing
- No `crow.json` config file loading
- No config merging (global + project)
- No markdown agent definitions
- No custom commands
- Provider/model hardcoded (can't switch without code change)
- No environment variable substitution
- No instructions loading (AGENTS.md, etc.)

---

## Implementation Plan

### Phase 1: Core Config Loading

**1.1 Config Schema (types)**

Create `crow/packages/api/src/config/mod.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Theme name
    pub theme: Option<String>,
    
    /// Default model ("provider/model" format)
    pub model: Option<String>,
    
    /// Small model for lightweight tasks
    pub small_model: Option<String>,
    
    /// Provider configurations
    pub provider: Option<HashMap<String, ProviderConfig>>,
    
    /// Agent configurations
    pub agent: Option<HashMap<String, AgentConfig>>,
    
    /// Global tool settings
    pub tools: Option<HashMap<String, bool>>,
    
    /// Global permissions
    pub permission: Option<PermissionConfig>,
    
    /// Custom commands
    pub command: Option<HashMap<String, CommandConfig>>,
    
    /// Glob patterns for instruction files
    pub instructions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub models: Option<HashMap<String, ModelConfig>>,
    pub options: Option<ProviderOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderOptions {
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    #[serde(rename = "baseURL")]
    pub base_url: Option<String>,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub prompt: Option<String>,
    pub model: Option<String>,
    pub mode: Option<String>,  // "primary" | "subagent" | "all"
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub tools: Option<HashMap<String, bool>>,
    pub permission: Option<PermissionConfig>,
    pub color: Option<String>,
    pub disable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Permission {
    Simple(String),  // "allow" | "ask" | "deny"
    Targeted(HashMap<String, String>),  // { "git push": "ask", "*": "allow" }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    pub edit: Option<String>,
    pub bash: Option<Permission>,
    pub webfetch: Option<String>,
    pub doom_loop: Option<String>,
    pub external_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    pub template: String,
    pub description: Option<String>,
    pub agent: Option<String>,
    pub model: Option<String>,
}
```

**1.2 Config Loading**

Create `crow/packages/api/src/config/loader.rs`:

```rust
impl Config {
    /// Load config from all sources with merging
    pub fn load() -> Result<Self, String> {
        let paths = GlobalPaths::new();
        let mut config = Config::default();
        
        // 1. Load global config
        let global_path = paths.config.join("crow.json");
        if global_path.exists() {
            let global = Self::load_file(&global_path)?;
            config = config.merge(global);
        }
        
        // 2. Load project config (search upward to git root)
        if let Some(project_path) = Self::find_project_config()? {
            let project = Self::load_file(&project_path)?;
            config = config.merge(project);
        }
        
        // 3. Load from .crow directory
        let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
        let crow_dir = cwd.join(".crow");
        if crow_dir.exists() {
            // Load crow.json
            let crow_config = crow_dir.join("crow.json");
            if crow_config.exists() {
                let dir_config = Self::load_file(&crow_config)?;
                config = config.merge(dir_config);
            }
            
            // Load agent/*.md files
            config.load_agent_directory(&crow_dir.join("agent"))?;
            
            // Load command/*.md files
            config.load_command_directory(&crow_dir.join("command"))?;
        }
        
        // 4. Environment variable overrides
        if let Ok(content) = std::env::var("CROW_CONFIG_CONTENT") {
            let env_config: Config = serde_json::from_str(&content)
                .map_err(|e| format!("Invalid CROW_CONFIG_CONTENT: {}", e))?;
            config = config.merge(env_config);
        }
        
        Ok(config)
    }
    
    /// Deep merge configs (other takes precedence)
    pub fn merge(self, other: Config) -> Config {
        // Implementation using serde_json for deep merge
    }
    
    /// Load markdown agent definition
    fn load_agent_md(path: &Path) -> Result<(String, AgentConfig), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        
        // Parse YAML frontmatter + markdown body
        let (frontmatter, body) = parse_frontmatter(&content)?;
        
        let mut agent: AgentConfig = serde_yaml::from_str(&frontmatter)
            .map_err(|e| format!("Invalid frontmatter in {}: {}", path.display(), e))?;
        
        // Body becomes the prompt
        agent.prompt = Some(body.trim().to_string());
        
        // Name from filename
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid filename")?
            .to_string();
        
        Ok((name, agent))
    }
}
```

**1.3 Variable Substitution**

```rust
impl Config {
    /// Substitute {env:VAR} and {file:path} patterns
    pub fn substitute_variables(&mut self) -> Result<(), String> {
        // Walk all string fields and substitute
        // {env:VAR_NAME} -> std::env::var("VAR_NAME")
        // {file:./path} -> std::fs::read_to_string(path)
    }
}
```

### Phase 2: Config File Locations

**Directory Structure:**

```
~/.config/crow/
├── crow.json           # Global config
├── agent/              # Global agent definitions
│   ├── reviewer.md
│   └── architect.md
└── command/            # Global commands
    └── test.md

~/.local/share/crow/
├── auth.json           # API credentials
├── log/
└── storage/

Project:
├── .crow/
│   ├── crow.json       # Project config
│   ├── agent/          # Project agents
│   └── command/        # Project commands
├── crow.json           # Alternative location
└── AGENTS.md           # Instructions
```

### Phase 3: Integration

**3.1 Update Server Initialization**

```rust
// In server.rs
pub async fn create_router_with_storage() -> Result<Router, String> {
    // Load config
    let config = Config::load()?;
    
    // Create provider from config
    let provider_config = config.get_provider_config()?;
    let provider = ProviderClient::new(provider_config)?;
    
    // Create agent registry from config
    let agent_registry = AgentRegistry::from_config(&config)?;
    
    // ... rest of initialization
}
```

**3.2 Update Agent Registry**

```rust
impl AgentRegistry {
    pub fn from_config(config: &Config) -> Result<Self, String> {
        let mut registry = Self::new();
        
        // Load built-in agents first
        registry.load_builtins();
        
        // Override/add from config
        if let Some(agents) = &config.agent {
            for (name, agent_config) in agents {
                let info = AgentInfo::from_config(name, agent_config)?;
                registry.register(info);
            }
        }
        
        Ok(registry)
    }
}
```

**3.3 Model Selection**

```rust
// Parse "provider/model" format
pub fn parse_model(model_str: &str) -> (String, String) {
    if let Some((provider, model)) = model_str.split_once('/') {
        (provider.to_string(), model.to_string())
    } else {
        ("default".to_string(), model_str.to_string())
    }
}
```

### Phase 4: Custom Instructions

**4.1 Load AGENTS.md/CLAUDE.md/CONTEXT.md**

```rust
impl Config {
    /// Load custom instructions from well-known files
    pub fn load_instructions(&self, working_dir: &Path) -> Vec<String> {
        let mut instructions = vec![];
        
        // Well-known instruction files
        let known_files = ["AGENTS.md", "CLAUDE.md", "CONTEXT.md"];
        
        for filename in &known_files {
            let path = working_dir.join(filename);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    instructions.push(content);
                }
            }
        }
        
        // Also load from config.instructions globs
        if let Some(patterns) = &self.instructions {
            for pattern in patterns {
                // Glob and load files
            }
        }
        
        instructions
    }
}
```

**4.2 Inject into System Prompt**

Update `SystemPromptBuilder` to include custom instructions.

---

## Example Config Files

**~/.config/crow/crow.json (Global):**
```json
{
  "model": "moonshotai/kimi-k2-thinking",
  "theme": "dracula",
  "permission": {
    "edit": "allow",
    "bash": {
      "git push": "ask",
      "*": "allow"
    }
  },
  "provider": {
    "moonshotai": {
      "options": {
        "baseURL": "https://api.moonshot.ai/v1"
      }
    },
    "openai": {
      "options": {
        "apiKey": "{env:OPENAI_API_KEY}"
      }
    }
  }
}
```

**Project .crow/crow.json:**
```json
{
  "model": "openai/gpt-4o",
  "agent": {
    "build": {
      "temperature": 0.3,
      "tools": {
        "websearch": false
      }
    }
  },
  "instructions": ["./docs/*.md"]
}
```

**Agent Markdown (.crow/agent/reviewer.md):**
```markdown
---
description: Code review specialist
mode: subagent
temperature: 0.1
tools:
  write: false
  edit: false
permission:
  edit: deny
  bash:
    git diff: allow
    "*": deny
---

You are a code reviewer. Analyze code for:
- Bugs and edge cases
- Performance issues
- Security vulnerabilities
- Best practices

Provide feedback without making changes.
```

---

## Implementation Order

### Sprint 1: Basic Config Loading
1. Create config types/schema
2. Load crow.json from global and project
3. Deep merge implementation
4. Wire into server initialization

### Sprint 2: Agent Config
5. Load agent/*.md files with frontmatter
6. Override built-in agents from config
7. Support custom agents

### Sprint 3: Provider Config
8. Parse "provider/model" format
9. Load provider options from config
10. Support multiple providers

### Sprint 4: Permissions & Instructions
11. Config-driven permissions
12. Load AGENTS.md/CLAUDE.md
13. Variable substitution ({env:}, {file:})

### Sprint 5: Commands
14. Load command/*.md files
15. Command execution with templates
16. Slash command support

---

## Migration from Current Code

1. **ProviderConfig** - Keep but load from config instead of hardcoding
2. **AgentRegistry** - Load from config, merge with built-ins
3. **auth.json** - Keep as-is, already working
4. **GlobalPaths** - Keep as-is, already XDG compliant

---

## Testing Strategy

1. Unit tests for config merging
2. Unit tests for markdown parsing
3. Integration tests with sample configs
4. Test environment variable substitution
5. Test permission inheritance

---

## Benefits

- **User customization** without code changes
- **Project-specific** settings
- **Shareable configs** via .crow directory
- **OpenCode compatibility** - similar config format
- **Extensibility** - custom agents, commands, providers
