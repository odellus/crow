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
        tools: {
            let mut tools = HashMap::new();
            // Disable task_complete - only arbiter should use this
            tools.insert("task_complete".to_string(), false);
            tools
        },
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

    // Executor agent - implementation agent for dual-agent system
    // Like build, but no Task tool (prevent nesting) and uses shared todos
    let executor = AgentInfo {
        name: "executor".to_string(),
        description: Some(
            "Executor agent for dual-agent verified tasks. Does the implementation work."
                .to_string(),
        ),
        mode: AgentMode::Subagent, // Only used internally by DualAgentRuntime
        built_in: true,
        temperature: None,
        top_p: None,
        color: Some("#3B82F6".to_string()), // Blue color for executor
        permission: default_permissions(),
        model: None,
        prompt: None, // Uses default build-like behavior
        tools: {
            let mut tools = HashMap::new();
            // Disable task - prevent infinite subagent nesting
            tools.insert("task".to_string(), false);
            // Disable task_complete - only arbiter can complete
            tools.insert("task_complete".to_string(), false);
            // Note: todoread/todowrite enabled - shared with arbiter via ToolRegistry
            tools
        },
        options: HashMap::new(),
    };
    agents.insert("executor".to_string(), executor);

    // Arbiter agent - verification agent for dual-agent system
    let arbiter = AgentInfo {
        name: "arbiter".to_string(),
        description: Some(
            "Verification agent that reviews work and calls task_complete when satisfied."
                .to_string(),
        ),
        mode: AgentMode::Subagent, // Only used internally by DualAgentRuntime
        built_in: true,
        temperature: Some(0.3), // Lower temperature for more deterministic verification
        top_p: None,
        color: Some("#10B981".to_string()), // Green color for arbiter
        permission: default_permissions(),
        model: None,
        prompt: Some(ARBITER_PROMPT.to_string()),
        tools: {
            let mut tools = HashMap::new();
            // Disable task - prevent infinite subagent nesting
            tools.insert("task".to_string(), false);
            // Enable task_complete - only arbiter can complete the task
            tools.insert("task_complete".to_string(), true);
            // Note: todoread/todowrite enabled - shared with executor via ToolRegistry
            tools
        },
        options: HashMap::new(),
    };
    agents.insert("arbiter".to_string(), arbiter);

    // Planner agent - primary agent for dual-agent mode (the "executor" role at primary level)
    // Does the actual work, responds to architect feedback
    let planner = AgentInfo {
        name: "planner".to_string(),
        description: Some(
            "Primary planning agent that executes tasks. Works with Architect in dual-agent mode."
                .to_string(),
        ),
        mode: AgentMode::Primary,
        built_in: true,
        temperature: None,
        top_p: None,
        color: Some("#3B82F6".to_string()), // Blue color
        permission: default_permissions(),
        model: None,
        prompt: None, // Uses default build-like behavior
        tools: {
            let mut tools = HashMap::new();
            // Disable task - no subagents for primary dual mode (for now)
            tools.insert("task".to_string(), false);
            // Disable task_complete - only architect can complete
            tools.insert("task_complete".to_string(), false);
            tools
        },
        options: HashMap::new(),
    };
    agents.insert("planner".to_string(), planner);

    // Architect agent - verifier for primary dual-agent mode
    // Reviews planner's work, can call task_complete to end the loop
    // User can also "be" the architect by interrupting
    let architect = AgentInfo {
        name: "architect".to_string(),
        description: Some(
            "Verification agent that reviews Planner's work. Calls task_complete when satisfied."
                .to_string(),
        ),
        mode: AgentMode::Primary, // Primary because it's used at the top level
        built_in: true,
        temperature: Some(0.3), // Lower temperature for more deterministic verification
        top_p: None,
        color: Some("#10B981".to_string()), // Green color
        permission: default_permissions(),
        model: None,
        prompt: Some(ARCHITECT_PROMPT.to_string()),
        tools: {
            let mut tools = HashMap::new();
            // Disable task - no subagents for primary dual mode
            tools.insert("task".to_string(), false);
            // Enable task_complete - only architect can complete!
            tools.insert("task_complete".to_string(), true);
            tools
        },
        options: HashMap::new(),
    };
    agents.insert("architect".to_string(), architect);

    agents
}

/// System prompt for the architect agent in primary dual-agent mode
/// Used when user runs `crow-cli chat --auto`
const ARCHITECT_PROMPT: &str = r#"You are the Architect agent in a dual-agent system.

You review the Planner's work. Either call task_complete OR provide feedback. Nothing else.

## CRITICAL RULES

1. When you call `task_complete`, STOP IMMEDIATELY. Do not generate any more text or tool calls after it.
2. You get ONE response per turn. Make your decision and execute it.
3. Do NOT explain what you're going to do. Just do it.

## Your job

Verify the Planner's work:
- Read the code/files if needed
- Run tests if applicable (cargo test, npm test, etc.)
- Check if requirements are met

## Decision

**If work is complete and correct:** Call `task_complete` with summary and verification. STOP.

**If work has issues:** Provide brief, specific feedback. The Planner will fix it.

## Example good responses

GOOD (complete):
```
<calls task_complete with summary>
```

GOOD (needs work):
```
Tests fail: `cargo test` shows 2 failures in auth module. Fix the token validation.
```

BAD:
```
Let me verify the work... I'll check the files... Now I'll run tests... The task is complete, let me call task_complete... <calls task_complete> Great, I've verified everything!
```

Be terse. One action per turn.
"#;

/// System prompt for the arbiter agent in subagent dual-agent mode
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
        assert_eq!(agents.len(), 7); // general, build, plan, executor, arbiter, planner, architect
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
    fn test_executor_agent() {
        let agents = get_builtin_agents();
        let executor = agents.get("executor").unwrap();

        assert!(executor.built_in);
        assert_eq!(executor.mode, AgentMode::Subagent);
        // Executor cannot spawn subagents or complete tasks
        assert!(!executor.is_tool_enabled("task"));
        assert!(!executor.is_tool_enabled("task_complete"));
        // But can use todos (shared with arbiter)
        assert!(executor.is_tool_enabled("todowrite"));
        assert!(executor.is_tool_enabled("todoread"));
    }

    #[test]
    fn test_arbiter_agent() {
        let agents = get_builtin_agents();
        let arbiter = agents.get("arbiter").unwrap();

        assert!(arbiter.built_in);
        assert_eq!(arbiter.mode, AgentMode::Subagent);
        // Arbiter cannot spawn subagents but CAN complete tasks
        assert!(!arbiter.is_tool_enabled("task"));
        assert!(arbiter.is_tool_enabled("task_complete"));
        // Can use todos (shared with executor)
        assert!(arbiter.is_tool_enabled("todowrite"));
        assert!(arbiter.is_tool_enabled("todoread"));
        assert!(arbiter.prompt.is_some());
        assert_eq!(arbiter.temperature, Some(0.3));
    }

    #[test]
    fn test_planner_agent() {
        let agents = get_builtin_agents();
        let planner = agents.get("planner").unwrap();

        assert!(planner.built_in);
        assert_eq!(planner.mode, AgentMode::Primary);
        // Planner cannot spawn subagents or complete tasks
        assert!(!planner.is_tool_enabled("task"));
        assert!(!planner.is_tool_enabled("task_complete"));
        // Can use todos
        assert!(planner.is_tool_enabled("todowrite"));
        assert!(planner.is_tool_enabled("todoread"));
    }

    #[test]
    fn test_architect_agent() {
        let agents = get_builtin_agents();
        let architect = agents.get("architect").unwrap();

        assert!(architect.built_in);
        assert_eq!(architect.mode, AgentMode::Primary);
        // Architect cannot spawn subagents but CAN complete tasks
        assert!(!architect.is_tool_enabled("task"));
        assert!(architect.is_tool_enabled("task_complete"));
        // Can use todos
        assert!(architect.is_tool_enabled("todowrite"));
        assert!(architect.is_tool_enabled("todoread"));
        assert!(architect.prompt.is_some());
        assert_eq!(architect.temperature, Some(0.3));
    }
}
