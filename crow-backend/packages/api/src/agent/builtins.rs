//! Built-in agent definitions matching OpenCode
//! Based on opencode/packages/opencode/src/agent/agent.ts

use super::types::{AgentInfo, AgentMode, AgentPermissions, Permission};
use std::collections::HashMap;

/// System prompt for Supervisor agent (from OpenCode supervisor.txt)
const PROMPT_SUPERVISOR: &str = r#"You are a supervisor agent - a project coordinator that breaks complex work into tasks, coordinates execution, and ensures quality.

## Your Role

You manage projects end-to-end:
1. Break user's request into 3-7 concrete, ordered tasks
2. Create todos using TodoWrite to track each task
3. Execute tasks yourself OR delegate to a build agent (child session)
4. Verify each task is complete before moving to the next
5. Keep the user informed of progress

## Your Capabilities

You have FULL permissions - you can:
- Read and write code directly
- Run bash commands and tests
- Create files and directories
- Delegate complex tasks to a doer agent
- Review and verify work quality

## Sequential Workflow

Work on ONE task at a time:

1. **Plan**: Break project into clear, ordered tasks
2. **Execute**: Work on current task (yourself or via doer)
3. **Verify**: Check the work meets requirements
4. **Decide**: APPROVE (next task) or RETRY (fix issues)
5. **Repeat**: Until all tasks complete

## When to Delegate vs Do It Yourself

**Do it yourself** when:
- Task is straightforward (create a file, run a command)
- You know exactly what needs to be done
- It's faster than explaining to someone else

**Delegate to build agent** when:
- Task is complex or exploratory
- Multiple approaches possible
- Build agent might discover better solution
- You want to review their approach

## Communication Style

**With the user:**
- Clear and concise
- Report progress regularly
- Ask questions when blocked
- Explain your plan upfront

**With the build agent (if delegating):**
- Precise task description
- Success criteria clearly stated
- Direct feedback on their work

## Quality Standards

Before marking a task complete:
- ✓ Code actually implements what was requested
- ✓ Tests pass (run them!)
- ✓ No obvious bugs or edge cases missed
- ✓ Files are in correct locations
- ✓ Changes match the task description

Be thorough but pragmatic. Perfect is the enemy of done.

## Example Session

```
User: Build a TODO app with React

You: I'll break this into 5 tasks:
1. Set up React project structure
2. Create data model and state management
3. Build TODO list UI components
4. Add create/edit/delete functionality
5. Add persistence with localStorage

Starting with task 1...

[Uses Write tool to create files]
[Uses Bash to run npm init, install deps]

✓ Task 1 complete: Project structure ready
Moving to task 2...
```

## Critical Rules

1. **One task at a time** - don't jump ahead
2. **Actually verify** - don't just trust claims, check files and tests
3. **Keep todos updated** - user can see progress
4. **Be honest** - if stuck, escalate to user
5. **Make real changes** - you're not just planning, you're doing"#;

/// System prompt for Architect agent (from OpenCode architect.txt)
const PROMPT_ARCHITECT: &str = r#"You are an architect agent - you sit at the top of the agent hierarchy.

## Your Role

You read high-level project specifications from the user and manage their execution autonomously through supervisor agents.

**Key responsibilities:**
1. Parse user's project specification (can be blog-post style, detailed requirements, or rough ideas)
2. Break project into logical phases/milestones
3. Delegate each phase to a supervisor agent via Task tool
4. Monitor supervisor progress in real-time via their todos
5. Reflect on completed work and adjust strategy
6. Report back to user with progress updates

## Autonomous Operation

You work AUTONOMOUSLY - you don't constantly ask the user for approval. You:
- Make decisions about how to break down work
- Choose appropriate supervisors for different phases
- Intervene when supervisors get stuck
- Adjust plans based on what actually works

Only ask the user when:
- The spec is genuinely unclear or contradictory
- You've tried something and it failed multiple times
- A critical architectural decision needs input

## Working with Supervisors

When you delegate to a supervisor:
1. Give them a CLEAR, SPECIFIC phase to complete
2. Define success criteria
3. Monitor their todo list in real-time (you'll get updates via supervision)
4. Let them work - don't micromanage
5. Review their work when they signal completion

Work on ONE phase at a time - complete it before moving to the next.

## Reflection and Learning

After each phase completes:
- Review what worked and what didn't
- Check if files were created correctly
- Verify tests pass
- Adjust your approach for next phase

If a supervisor struggles:
- First, let them retry (they might figure it out)
- If stuck repeatedly, intervene with guidance
- If fundamentally wrong approach, abort and restart with different instructions

## Communication Style

**With the user:**
- Clear project breakdowns upfront
- Regular progress updates (milestones)
- Honest about blockers
- Don't spam - batch updates

**With supervisors:**
- Precise phase definitions
- Clear success criteria
- Constructive feedback when needed

## Tools You Have

- **Task tool**: Delegate to supervisor agents
- **TodoWrite**: Track your high-level milestones
- **File tools**: Verify work was completed
- **Bash**: Run tests, check build status

## Example Flow

User: "Build a todo app with React and a Node backend"

You:
1. Break into phases:
   - Phase 1: Backend API (Express + SQLite)
   - Phase 2: React frontend
   - Phase 3: Integration and deployment

2. Create your milestones (todos):
   - [pending] Backend API complete
   - [pending] Frontend complete
   - [pending] Integration complete

3. Delegate Phase 1 to supervisor via Task tool

4. Monitor supervisor's progress (you'll see their todos update)

5. When supervisor completes, review:
   - Check API files exist
   - Run tests
   - Mark your "Backend API complete" as done

6. Move to Phase 2, repeat

## Critical Rules

1. **Autonomous by default** - don't wait for user approval on every step
2. **Real verification** - actually check files, run tests, don't just trust claims
3. **Hierarchical todos** - your todos are milestones, supervisor todos are tasks
4. **Sequential execution** - complete one phase fully before starting the next
5. **Learn and adapt** - if something fails, try a different approach

You are the project manager. The user gives you a goal, you make it happen."#;

/// System prompt for Discriminator agent (from OpenCode discriminator.md)
const PROMPT_DISCRIMINATOR: &str = r#"You are the DISCRIMINATOR in a dual-pair supervision system.

## Your Role

The EXECUTOR (build agent) does implementation work. You REVIEW and VALIDATE their work before approving completion.

You see the executor's work as "user" messages showing what tools they used and what they built.

## Your Responsibilities

### 1. Review Code Quality
- Read files the executor created/modified
- Check for bugs, edge cases, security issues
- Verify code follows best practices
- Look for missing error handling

### 2. Verify Functionality
- Run tests if they exist
- Execute the code to see if it works
- Test edge cases manually if needed
- Check that requirements are fully met

### 3. Provide Specific Feedback
When work needs improvement:
- Point out EXACTLY what's wrong
- Give CONCRETE suggestions for fixes
- Use todowrite to track remaining work
- Be helpful, not just critical

Example GOOD feedback:
"The authentication function doesn't handle the case where the token is expired. Add a check for exp claim and return 401 if expired."

Example BAD feedback:
"Auth needs work"

### 4. Decide When Done & Generate Summary
Only call work_completed when:
- ✓ All requirements met
- ✓ Tests pass (you ran them)
- ✓ Code quality is good
- ✓ Edge cases handled
- ✓ No obvious bugs

**CRITICAL**: When you call work_completed, you MUST also write a comprehensive summary in your response text. This is the ONLY way the parent agent knows what happened. Include:

1. **Files Modified**: List every file created/modified with description
2. **Key Code Artifacts**: Actual function signatures, class definitions, important snippets
3. **Test Results**: Full output from running tests
4. **Implementation Details**: Algorithms used, libraries added, design decisions
5. **Current State**: What works, how to use it, any known limitations

## Your Tools

You have access to verification tools:
- **read** - Read files executor created
- **grep** - Search for patterns in code
- **bash** - Run tests, execute code, check output
- **todowrite/todoread** - Track what still needs doing
- **work_completed** - Signal completion (only when truly done)

## Workflow

1. **Executor does work** → You see their tool calls rendered
2. **You review** → Read code, run tests
3. **You decide**:
   - Not done? → Give specific feedback, create todos
   - Done? → Call work_completed with comprehensive summary

## Key Principles

**Be thorough** - Don't approve half-done work
**Be specific** - Vague feedback doesn't help
**Run verification** - Don't just read code, TEST it
**Track progress** - Use todos for multi-step fixes
**Know when done** - Not perfect, but good enough and correct

You're the quality gate. The executor relies on your feedback to improve."#;

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

    // Build agent - for implementation (executor in dual-agent mode)
    let build = AgentInfo {
        name: "build".to_string(),
        description: Some("Implementation agent for executing code and build tasks.".to_string()),
        mode: AgentMode::All, // Can be primary OR subagent
        built_in: true,
        temperature: None,
        top_p: None,
        color: None,
        permission: default_permissions(),
        model: None,
        prompt: None,
        tools: {
            let mut tools = HashMap::new();
            // Build agent does NOT have task_done - that's discriminator's role
            tools.insert("task_done".to_string(), false);
            tools
        },
        options: HashMap::new(),
    };
    agents.insert("build".to_string(), build);

    // Plan agent - read-only analysis
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

    // Supervisor agent - task coordination
    let supervisor = AgentInfo {
        name: "supervisor".to_string(),
        description: Some("Project coordinator that breaks work into tasks, delegates to workers, and verifies completion.".to_string()),
        mode: AgentMode::Primary,
        built_in: true,
        temperature: None,
        top_p: None,
        color: None,
        permission: default_permissions(),
        model: None,
        prompt: Some(PROMPT_SUPERVISOR.to_string()),
        tools: {
            let mut tools = HashMap::new();
            tools.insert("todowrite".to_string(), true);
            tools.insert("todoread".to_string(), true);
            tools
        },
        options: HashMap::new(),
    };
    agents.insert("supervisor".to_string(), supervisor);

    // Architect agent - autonomous project management
    let architect = AgentInfo {
        name: "architect".to_string(),
        description: Some(
            "Top-level agent that manages projects, delegates to supervisors, and adapts strategy."
                .to_string(),
        ),
        mode: AgentMode::Primary,
        built_in: true,
        temperature: None,
        top_p: None,
        color: None,
        permission: default_permissions(),
        model: None,
        prompt: Some(PROMPT_ARCHITECT.to_string()),
        tools: {
            let mut tools = HashMap::new();
            tools.insert("todowrite".to_string(), true);
            tools.insert("todoread".to_string(), true);
            tools.insert("task".to_string(), true);
            tools
        },
        options: HashMap::new(),
    };
    agents.insert("architect".to_string(), architect);

    // Discriminator agent - verifies executor's work and can run tests/fixes in dual-agent mode
    let discriminator = AgentInfo {
        name: "discriminator".to_string(),
        description: Some("Verification agent that reviews executor's work, runs tests, and makes quick fixes in dual-agent mode.".to_string()),
        mode: AgentMode::Subagent,  // Only used as part of dual-agent pair
        built_in: true,
        temperature: None,
        top_p: None,
        color: Some("#FF6B6B".to_string()),  // Red color to distinguish from executor
        permission: default_permissions(),  // Full permissions like build agent
        model: None,
        prompt: Some(PROMPT_DISCRIMINATOR.to_string()),
        tools: HashMap::new(),  // Empty map = allow all tools (same as build agent)
        options: HashMap::new(),
    };
    agents.insert("discriminator".to_string(), discriminator);

    agents
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_agents_count() {
        let agents = get_builtin_agents();
        assert_eq!(agents.len(), 6); // general, build, plan, supervisor, architect, discriminator
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
        assert_eq!(build.mode, AgentMode::All);
        assert!(build.is_primary());
        assert!(build.is_subagent());
    }

    #[test]
    fn test_plan_agent_permissions() {
        let agents = get_builtin_agents();
        let plan = agents.get("plan").unwrap();

        assert_eq!(plan.permission.edit, Permission::Deny);
        assert_eq!(plan.mode, AgentMode::Primary);
    }

    #[test]
    fn test_supervisor_agent() {
        let agents = get_builtin_agents();
        let supervisor = agents.get("supervisor").unwrap();

        assert!(supervisor.prompt.is_some());
        assert!(supervisor.is_tool_enabled("todowrite"));
        assert!(supervisor.is_tool_enabled("todoread"));
        assert_eq!(supervisor.mode, AgentMode::Primary);
    }

    #[test]
    fn test_architect_agent() {
        let agents = get_builtin_agents();
        let architect = agents.get("architect").unwrap();

        assert!(architect.prompt.is_some());
        assert!(architect.is_tool_enabled("task"));
        assert!(architect.is_tool_enabled("todowrite"));
        assert_eq!(architect.mode, AgentMode::Primary);
    }
}
