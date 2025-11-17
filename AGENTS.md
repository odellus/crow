# Agent Architecture for Crow

## Current Status

### What Crow Has Now ✅
- **Single-agent execution** working with LLM integration
- **6 built-in agents:** general, build, plan, supervisor, architect, discriminator
- **9 working tools:** bash, edit, write, read, grep, glob, todowrite, todoread, work_completed
- **Permission system** with tool filtering
- **Project directory isolation** - agents can spawn in any directory
- **Session storage** with persistence to `~/.crow/sessions/`
- **Dual-agent runtime** - executor + discriminator loop (basic implementation)
- **API endpoints:** 
  - `POST /session` - create session
  - `POST /session/:id/message` - send message to agent
  - `POST /session/dual` - run dual-agent (currently top-level, needs refactoring)

### What's Missing ❌
- **Task tool** - agents can't spawn subagents yet
- **Proper dual-agent invocation** - currently top-level, should be subagent via task tool
- **Session hierarchy** - parent/child session linking works but task tool doesn't use it
- **Agent mode enforcement** - subagent vs primary distinction exists but not enforced
- **Markdown telemetry** - started but not complete (see TELEMETRY_WORK_STATUS.md)
- **Frontend** - Dioxus fullstack setup exists but is broken/neglected

## The Architecture We Need

Based on OpenCode's proven pattern (see OPENCODE_SUBAGENT_SYSTEM.md):

### 1. Task Tool (CRITICAL - Not Yet Implemented)

**Purpose:** Allows agents to spawn subagents autonomously

**Signature:**
```rust
task(
  description: "Short 3-5 word task description",
  prompt: "Detailed task for the subagent",
  subagent_type: "dual-agent" | "explore" | "plan" | "docs"
)
```

**How it works:**
1. Parent agent calls task tool in their response
2. Task tool creates child session with `parentID` link
3. If `subagent_type == "dual-agent"` → run DualAgentRuntime
4. Otherwise → run single agent with SessionPrompt
5. Return summary + output to parent agent
6. Parent agent synthesizes result for user

**Example:**
```
User: "Implement fibonacci function"
  ↓
BUILD agent: "I'll use dual-agent for supervised execution"
  🔧 task(
    description="Implement fibonacci", 
    prompt="Write fibonacci with tests",
    subagent_type="dual-agent"
  )
  ↓
Task tool:
  - Creates child session (ses-child-123)
  - Runs DualAgentRuntime (executor + discriminator loop)
  - Exports to .crow/sessions/ses-child-123.md
  - Returns: { output: "Fibonacci implemented", metadata: {...} }
  ↓
BUILD agent: "Fibonacci function has been implemented with tests. All tests passing."
```

### 2. Agent Modes (Already Exists, Needs Enforcement)

```rust
pub enum AgentMode {
    Primary,   // Can be used directly by user
    Subagent,  // Only via task tool
    All,       // Both
}
```

**Current agents:**
- `general` - Primary (default for user)
- `build` - Primary (but should default to spawning dual-agent)
- `plan` - Subagent (read-only planning)
- `supervisor` - Primary (task management)
- `architect` - Primary (project management)
- `discriminator` - Subagent (only used in dual-agent)
- **`dual-agent`** - Subagent (NEW - needs to be registered)

### 3. Dual-Agent as Subagent (Needs Refactoring)

**Current problem:** `POST /session/dual` is a top-level endpoint

**What we need:**
- Remove `POST /session/dual` endpoint
- Register dual-agent as a subagent
- BUILD agent's system prompt should say: "Use task tool with subagent_type='dual-agent' for implementation tasks"
- Task tool handles the spawning

**Dual-agent prompt:**
```markdown
---
description: Supervised execution with executor/discriminator
mode: subagent
---

You are part of a DUAL-AGENT supervision system.

## How This Works
- EXECUTOR (build agent) - Implements the task
- DISCRIMINATOR (supervisor agent) - Reviews and provides feedback
- You see different perspectives of the same conversation
- Only discriminator can call work_completed

[Rest of dual-agent.md from OpenCode]
```

### 4. Session Hierarchy (Already Works)

```rust
pub struct Session {
    pub id: String,
    pub parent_id: Option<String>,  // ✅ Already exists
    pub directory: String,
    pub metadata: Option<serde_json::Value>,  // ✅ Added
    // ...
}
```

**Task tool creates:**
```rust
Session {
    id: "ses-child-123",
    parent_id: Some("ses-parent-456"),
    title: "Implement fibonacci (@dual-agent subagent)",
    metadata: Some(json!({
        "includeParentContext": true,
        "subagentType": "dual-agent",
    })),
}
```

### 5. Markdown Telemetry (Partially Done)

**Status:** SessionExport and CrowStorage exist but not wired up

**What's needed:**
- Export both executor and discriminator sessions after each turn
- Store in `.crow/sessions/{session-id}.md`
- Include system prompts, tools available, all messages, tool calls

See TELEMETRY_WORK_STATUS.md for details.

## Agent Delegation Rules

**We should implement these rules in the task tool:**

```rust
// BUILD can spawn any subagent
if calling_agent == "build" {
    // Allow dual-agent, explore, plan, docs, etc.
}

// SUPERVISOR can only spawn build
if calling_agent == "supervisor" {
    if subagent_type != "build" {
        return Err("supervisor can only delegate to build agent")
    }
}

// DISCRIMINATOR can spawn for verification
if calling_agent == "discriminator" {
    // Allow bash, read, grep, write, edit for fixes
}
```

**Hierarchy:**
```
User
  └─> General/Primary agents
       └─> Supervisor
            └─> Build (default: spawns dual-agent for implementation)
                 └─> Dual-Agent (executor + discriminator)
                 └─> Explore (research/search)
                 └─> Plan (read-only planning)
                 └─> Docs (documentation)
```

## Default Behavior

**Key insight from OpenCode:** BUILD agent should DEFAULT to using dual-agent

**BUILD agent system prompt should include:**
```
When implementing code or making significant changes, use the task tool 
with subagent_type='dual-agent' for supervised execution. This ensures 
quality through discriminator review.

Example:
🔧 task(
  description="Implement feature X",
  prompt="Write feature X with tests and error handling",
  subagent_type="dual-agent"
)
```

## Frontend Situation

**Current state:**
- Dioxus 0.7.1 fullstack setup in `crow/packages/web/`, `crow/packages/desktop/`
- Not actively developed
- Broken/neglected
- Uses Axum backend in `crow/packages/api/src/server.rs`

**What we need eventually:**
- Dioxus frontend consuming REST API
- Session list view
- Message stream view
- Tool call rendering (like OpenCode's UI)
- Fork/branch visualization
- Agent selection UI

**Priority:** LOW - focus on backend/agent system first

See crow/AGENTS.md (the Dioxus reference) for framework patterns.

## Implementation Priority for Next Session

1. **Implement Task Tool** ⭐ CRITICAL
   - Create `crow/packages/api/src/tools/task.rs`
   - Spawn child session with parentID
   - Check subagent_type
   - If dual-agent → DualAgentRuntime
   - Return summary to parent

2. **Register Dual-Agent Subagent**
   - Create `.crow/agent/dual-agent.md`
   - Set mode: subagent
   - Remove `POST /session/dual` endpoint

3. **Update BUILD Agent Prompt**
   - Add task tool usage guidance
   - Default to dual-agent for implementation

4. **Wire Up Telemetry**
   - Add export calls in DualAgentRuntime
   - Export both sessions after each turn

5. **Test End-to-End**
   - User → BUILD → spawns dual-agent → returns result
   - Check `.crow/sessions/` for markdown exports

## Files to Give Next Agent

### Must Read First:
1. **OPENCODE_SUBAGENT_SYSTEM.md** - How task tool and subagents work
2. **TELEMETRY_WORK_STATUS.md** - Current telemetry implementation status

### Reference:
3. **STATUS.md** - What's already implemented in Crow
4. **This file (AGENTS.md)** - Agent architecture overview

### Codebase Entry Points:
- `crow/packages/api/src/tools/` - Tool implementations
- `crow/packages/api/src/agent/` - Agent executor, registry, runtime
- `crow/packages/api/src/server.rs` - API endpoints
- `crow/packages/api/src/session/` - Session management

## Success Criteria

**Task tool is working when:**
1. BUILD agent can call `task(subagent_type="dual-agent", ...)`
2. Child session is created with parentID
3. DualAgentRuntime executes in child session
4. Both executor and discriminator sessions export to `.crow/sessions/`
5. Task tool returns summary to BUILD agent
6. BUILD agent shows user the result

**Full system is working when:**
1. User sends "Implement feature X" to BUILD agent
2. BUILD agent automatically spawns dual-agent
3. Executor implements, discriminator reviews
4. Both sessions exported to markdown
5. Task tool returns to BUILD agent
6. BUILD agent tells user "Feature X implemented and verified"

This is the architecture. Let's build it.
