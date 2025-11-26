# Crow-Tauri Session Memory

## What Is This Project

Crow is a Rust rewrite of OpenCode (TypeScript). It's an AI coding agent with CLI and Tauri desktop app. The core is in `crow-tauri/src-tauri/core/`.

## Current State

- **384 unit tests passing**
- **E2E tests working** - see `crow-tauri/src-tauri/core/tests/e2e/`
- **CLI working** - `crow-cli chat`, `crow-cli sessions`, etc.
- **Provider**: Moonshot (kimi-k2-thinking) is default fallback when no ANTHROPIC/OPENAI keys
- **Agent config system ALREADY WORKS** - loads from `~/.config/crow/agent/*.md` and `.crow/agent/*.md`

## What We're Building: Dual Agent System

Two full agents in a loop:
- **Executor**: Does the work (standard build agent)
- **Arbiter**: Verifies the work, has `task_complete` tool to terminate loop

NOT a lightweight reviewer. Arbiter has ALL the same tools plus `task_complete`. Both run full ReACT loops.

### The Loop

```
User Request
     │
     ▼
┌─────────────────────────────────────┐
│ EXECUTOR (build agent)              │
│ - Full ReACT loop                   │
│ - All standard tools                │
│ - Stops when it thinks it's done    │
└─────────────────────────────────────┘
     │
     ▼
[Render executor session to markdown]
     │
     ▼
┌─────────────────────────────────────┐
│ ARBITER (build + task_complete)     │
│ - Full ReACT loop                   │
│ - Runs tests, verifies work         │
│ - task_complete OR continues        │
└─────────────────────────────────────┘
     │
     ├── task_complete? ──► DONE
     │
     ▼
[Render arbiter session to markdown]
     │
     ▼
Back to EXECUTOR with arbiter's FULL session
     │
    ...
```

**CRITICAL**: Arbiter's ENTIRE session (all thinking, tool calls, outputs) goes to executor, not just "feedback".

### Session Linking

Via metadata, NOT ID parsing:

```json
{
  "dual_agent": {
    "role": "executor",
    "pair_id": "pair_xyz",
    "sibling_id": "ses_arbiter_abc",
    "step": 1
  }
}
```

## Implementation Plan

Read `DUAL_AGENT_CLI_PLAN.md` for full details.

### Agent Config - ALREADY DONE

`AgentRegistry::new_with_config()` already loads from:
1. Built-in agents (general, build, plan)
2. Global: `~/.config/crow/agent/*.md`
3. Project: `.crow/agent/*.md`

Project overrides global. Format is markdown with YAML frontmatter.

**Just need to make sure `new_with_config()` is called at startup.**

### What We Need to Build

**1. task_complete tool**
- `tools/task_complete.rs` - NEW
- Returns metadata with `task_complete: true`
- DualAgentRuntime checks for this to terminate loop

**2. Arbiter agent config**
- Either add to `builtins.rs` OR create `~/.config/crow/agent/arbiter.md`
- Same as build agent but with `task_complete: true` in tools

**3. DualAgentRuntime**
- `agent/dual.rs` - NEW
- Orchestrates executor → arbiter → executor loop
- Creates linked sessions with sibling_id metadata
- Renders sessions to markdown between agents

**4. CLI --dual flag**
- `bin/crow-cli.rs` - Add `--dual` flag
- Different colors: Executor (cyan), Arbiter (green)
- No extra markdown rendering for display - ReACT loop already renders

## Key Files Reference

```
crow-tauri/src-tauri/core/src/
  agent/
    executor.rs      # ReACT loop implementation
    registry.rs      # Agent registry
    builtins.rs      # Built-in agents (general, build, plan)
    types.rs         # AgentInfo, AgentMode, AgentPermissions
  tools/
    mod.rs           # Tool registry
    task.rs          # Current Task tool (spawns subagents)
    bash.rs, edit.rs, read.rs, write.rs, etc.
  session/
    store.rs         # Session CRUD, message storage
    export.rs        # Markdown export
  storage/
    crow.rs          # XDG storage (~/.local/share/crow/storage/)
  config/
    loader.rs        # Config loading
    types.rs         # Config structs
  global.rs          # XDG paths
```

## Storage Locations

```
~/.config/crow/           # Config (crow.json, agent/*.md)
~/.local/share/crow/      # Data (storage/session/, storage/message/, etc.)
~/.local/state/crow/      # State (logs/)
.crow/                    # Project-local config and sessions
```

## OpenCode Reference

Crow mirrors OpenCode's architecture. When in doubt, check:
- `opencode/packages/opencode/src/agent/` - Agent system
- `opencode/packages/opencode/src/config/` - Config loading
- `opencode/packages/opencode/src/session/` - Session management

## Testing

E2E tests in `crow-tauri/src-tauri/core/tests/e2e/`:
- `TEST_AGENTS.md` - Instructions for agents running E2E tests
- `tests/01_session.sh` through `tests/11_task.sh`

Run manually:
```bash
cd crow-tauri/src-tauri
cargo build --bin crow-cli
./target/debug/crow-cli chat "echo hello"
```

## User Preferences

- NO mocking in E2E tests - real API calls only
- Config MUST be outside source code (markdown files, not hardcoded)
- `task_complete` is REQUIRED for arbiter - raise error if missing
- Vision tools are future work - focus on CLI first

## Documents to Read

1. `DUAL_AGENT_PLAN.md` - Full architecture vision (includes future vision tools)
2. `DUAL_AGENT_CLI_PLAN.md` - Minimal CLI implementation plan (start here)
3. `AGENTS.md` - Project overview and testing info

## Next Steps

1. Add `task_complete` tool (`tools/task_complete.rs`)
2. Add arbiter agent (builtin or config file)
3. Add `DualAgentRuntime` (`agent/dual.rs`)
4. Add `--dual` CLI flag
5. Test with real tasks
