//! Built-in agent definitions matching OpenCode
//! Based on opencode/packages/opencode/src/agent/agent.ts
//!
//! OpenCode built-in agents:
//! - build: Primary agent for implementation (default)
//! - plan: Primary agent for planning with read-only permissions
//! - general: Subagent for research and exploration (used via TaskTool)
//!
//! Additional agents can be added via config files:
//! - Global: ~/.config/crow/agent/*.md
//! - Project: .crow/agent/*.md

use super::types::{AgentInfo, AgentMode, AgentPermissions, Permission};
use std::collections::HashMap;

/// Create default agent permissions (allow everything)
fn default_permissions() -> AgentPermissions {
    let mut bash = HashMap::new();
    bash.insert("*".to_string(), Permission::Allow);

    AgentPermissions {
        edit: Permission::Allow,
        bash,
        webfetch: Some(Permission::Allow),
        doom_loop: Some(Permission::Ask),
        external_directory: Some(Permission::Ask),
    }
}

/// Create Plan agent permissions (read-only bash whitelist)
/// Matches OpenCode exactly from opencode/packages/opencode/src/agent/agent.ts:58-106
fn plan_permissions() -> AgentPermissions {
    let mut bash = HashMap::new();

    // Read-only commands (matches OpenCode exactly)
    bash.insert("cut*".to_string(), Permission::Allow);
    bash.insert("diff*".to_string(), Permission::Allow);
    bash.insert("du*".to_string(), Permission::Allow);
    bash.insert("file *".to_string(), Permission::Allow);

    // Find with dangerous options requires ask
    bash.insert("find * -delete*".to_string(), Permission::Ask);
    bash.insert("find * -exec*".to_string(), Permission::Ask);
    bash.insert("find * -fprint*".to_string(), Permission::Ask);
    bash.insert("find * -fls*".to_string(), Permission::Ask);
    bash.insert("find * -fprintf*".to_string(), Permission::Ask);
    bash.insert("find * -ok*".to_string(), Permission::Ask);
    bash.insert("find *".to_string(), Permission::Allow);

    // Git read-only
    bash.insert("git diff*".to_string(), Permission::Allow);
    bash.insert("git log*".to_string(), Permission::Allow);
    bash.insert("git show*".to_string(), Permission::Allow);
    bash.insert("git status*".to_string(), Permission::Allow);
    bash.insert("git branch".to_string(), Permission::Allow);
    bash.insert("git branch -v".to_string(), Permission::Allow);

    // Text processing
    bash.insert("grep*".to_string(), Permission::Allow);
    bash.insert("head*".to_string(), Permission::Allow);
    bash.insert("less*".to_string(), Permission::Allow);
    bash.insert("ls*".to_string(), Permission::Allow);
    bash.insert("more*".to_string(), Permission::Allow);
    bash.insert("pwd*".to_string(), Permission::Allow);
    bash.insert("rg*".to_string(), Permission::Allow);

    // Sort with output redirection requires ask
    bash.insert("sort --output=*".to_string(), Permission::Ask);
    bash.insert("sort -o *".to_string(), Permission::Ask);
    bash.insert("sort*".to_string(), Permission::Allow);

    bash.insert("stat*".to_string(), Permission::Allow);
    bash.insert("tail*".to_string(), Permission::Allow);

    // Tree with output redirection requires ask
    bash.insert("tree -o *".to_string(), Permission::Ask);
    bash.insert("tree*".to_string(), Permission::Allow);

    bash.insert("uniq*".to_string(), Permission::Allow);
    bash.insert("wc*".to_string(), Permission::Allow);
    bash.insert("whereis*".to_string(), Permission::Allow);
    bash.insert("which*".to_string(), Permission::Allow);

    // Ask for anything else (default catch-all)
    bash.insert("*".to_string(), Permission::Ask);

    AgentPermissions {
        edit: Permission::Deny, // Plan agent cannot edit files
        bash,
        webfetch: Some(Permission::Allow),
        doom_loop: Some(Permission::Ask),
        external_directory: Some(Permission::Ask),
    }
}

/// Get all built-in agents
pub fn get_builtin_agents() -> HashMap<String, AgentInfo> {
    let mut agents = HashMap::new();

    // General agent - for research and exploration
    let general = AgentInfo {
        name: "general".to_string(),
        description: Some("General-purpose agent for researching complex questions, searching for code, and executing multi-step tasks.".to_string()),
        mode: AgentMode::Subagent,
        built_in: true,
        temperature: None,
        top_p: None,
        color: None,
        permission: default_permissions(),
        model: None,
        prompt: None,
        tools: {
            let mut tools = HashMap::new();
            tools.insert("todoread".to_string(), false);
            tools.insert("todowrite".to_string(), false);
            tools
        },
        options: HashMap::new(),
    };
    agents.insert("general".to_string(), general);

    // Build agent - for implementation (matches OpenCode)
    let build = AgentInfo {
        name: "build".to_string(),
        description: Some("Implementation agent for executing code and build tasks.".to_string()),
        mode: AgentMode::Primary, // Primary agent for direct use
        built_in: true,
        temperature: None,
        top_p: None,
        color: None,
        permission: default_permissions(),
        model: None,
        prompt: None,
        tools: HashMap::new(), // All tools enabled by default
        options: HashMap::new(),
    };
    agents.insert("build".to_string(), build);

    // Plan agent - read-only analysis (matches OpenCode)
    let plan = AgentInfo {
        name: "plan".to_string(),
        description: Some("Planning and analysis agent with restricted permissions.".to_string()),
        mode: AgentMode::Primary,
        built_in: true,
        temperature: None,
        top_p: None,
        color: None,
        permission: plan_permissions(),
        model: None,
        prompt: None,
        tools: HashMap::new(),
        options: HashMap::new(),
    };
    agents.insert("plan".to_string(), plan);

    // Arbiter agent - verification agent for dual-agent system
    let arbiter = AgentInfo {
        name: "arbiter".to_string(),
        description: Some(
            "Verification agent that reviews work and calls task_complete when satisfied."
                .to_string(),
        ),
        mode: AgentMode::Primary, // Internal use only - NOT available as a subtask
        built_in: true,
        temperature: Some(0.3), // Lower temperature for more deterministic verification
        top_p: None,
        color: Some("#10B981".to_string()), // Green color for arbiter
        permission: default_permissions(),
        model: None,
        prompt: Some(ARBITER_PROMPT.to_string()),
        tools: {
            let mut tools = HashMap::new();
            // Enable task_complete (disabled by default for other agents)
            tools.insert("task_complete".to_string(), true);
            // Disable todoread/todowrite like general agent
            tools.insert("todoread".to_string(), false);
            tools.insert("todowrite".to_string(), false);
            tools
        },
        options: HashMap::new(),
    };
    agents.insert("arbiter".to_string(), arbiter);

    agents
}

/// System prompt for the arbiter agent in dual-agent mode
const ARBITER_PROMPT: &str = r#"You are the Arbiter agent in a dual-agent verification system.

You receive the Executor's full session showing everything it did - all thinking, tool calls, and outputs.

Your job is to VERIFY the work:

1. **Read carefully** - Understand what the Executor did and why
2. **Run tests** - Execute test commands (cargo test, npm test, pytest, etc.)
3. **Check the code** - Read modified files to verify correctness
4. **Verify requirements** - Ensure the original task requirements are met

## When to call task_complete

Call `task_complete` with summary and verification when ALL of these are true:
- Tests pass (if applicable)
- Code compiles/runs without errors
- The original requirements are satisfied
- No obvious bugs or issues

## When NOT to call task_complete

If there are problems:
- Explain clearly what's wrong and why
- Provide specific feedback the Executor can act on
- Your full response will be sent back to the Executor

## Important

- You have ALL the same tools as the Executor (bash, read, edit, etc.)
- Actually run tests - don't just assume they pass
- Be thorough but efficient - verify the critical paths
- If you find issues, be constructive in your feedback
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_agents_count() {
        let agents = get_builtin_agents();
        assert_eq!(agents.len(), 4); // general, build, plan, arbiter
    }

    #[test]
    fn test_general_agent() {
        let agents = get_builtin_agents();
        let general = agents.get("general").unwrap();

        assert!(general.built_in);
        assert_eq!(general.mode, AgentMode::Subagent);
        assert!(!general.is_tool_enabled("todowrite"));
        assert!(!general.is_tool_enabled("todoread"));
    }

    #[test]
    fn test_build_agent() {
        let agents = get_builtin_agents();
        let build = agents.get("build").unwrap();

        assert!(build.built_in);
        assert_eq!(build.mode, AgentMode::Primary);
        assert!(build.is_primary());
    }

    #[test]
    fn test_plan_agent_permissions() {
        let agents = get_builtin_agents();
        let plan = agents.get("plan").unwrap();

        assert_eq!(plan.permission.edit, Permission::Deny);
        assert_eq!(plan.mode, AgentMode::Primary);
    }

    #[test]
    fn test_arbiter_agent() {
        let agents = get_builtin_agents();
        let arbiter = agents.get("arbiter").unwrap();

        assert!(arbiter.built_in);
        assert_eq!(arbiter.mode, AgentMode::Primary); // Internal use only, not a subtask
        assert!(arbiter.is_tool_enabled("task_complete"));
        assert!(!arbiter.is_tool_enabled("todowrite"));
        assert!(!arbiter.is_tool_enabled("todoread"));
        assert!(arbiter.prompt.is_some());
        assert_eq!(arbiter.temperature, Some(0.3));
    }
}
