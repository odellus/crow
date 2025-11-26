# Crow Agent Development Guide

This document provides instructions for AI agents working on the Crow codebase.

## 🚨 CRITICAL RULES FOR AGENTS

### CLI Usage - ALWAYS Fresh Sessions
```bash
# ⚠️ ALWAYS create new session for testing - reusing causes weird model behavior!
crow-cli chat "your message"           # Creates NEW session automatically
crow-cli chat --session ID "message"   # Only if you NEED to continue a specific session

# ❌ DON'T reuse sessions unless specifically testing session continuity
# ✅ DO create fresh sessions for each test/task
```

### XDG Verification
After any tool that persists data, verify the XDG locations:
```bash
# Sessions
ls ~/.local/share/crow/storage/session/*/

# Todos  
ls ~/.local/share/crow/storage/todo/

# Snapshots
ls ~/.local/share/crow/snapshots/

# Logs
ls ~/.local/state/crow/logs/
cat ~/.local/state/crow/logs/tool-calls.jsonl | tail -5

# Agent execution log
cat ~/.local/state/crow/logs/agent.log | tail -5
```

### Compare with OpenCode
When implementing/testing features, compare with OpenCode's implementation:
- **DO compare**: Feature behavior, XDG paths, output format
- **IGNORE**: LSP integration, Permission system (we're not implementing these)

---

## Project Overview

Crow is a Tauri-based AI coding assistant. It provides:
- **crow_core**: Rust library with all backend logic (agents, tools, providers, sessions)
- **crow-cli**: Command-line interface for testing and direct usage
- **Tauri app**: Desktop application (frontend in React)

## Architecture

```
crow-tauri/
├── src/                          # React frontend
├── src-tauri/
│   ├── app/                      # Tauri application crate
│   └── core/                     # crow_core library
│       └── src/
│           ├── lib.rs
│           ├── bin/crow-cli.rs   # CLI binary
│           ├── agent/            # Agent system (executor, registry, prompts)
│           ├── bus/              # Event bus for real-time updates
│           ├── config/           # Configuration loading
│           ├── prompts/          # System prompt templates
│           ├── providers/        # LLM provider clients
│           ├── session/          # Session & message storage
│           ├── snapshot/         # Shadow git for file tracking
│           ├── storage/          # XDG-based persistence
│           └── tools/            # Tool implementations
├── PLAN.md                       # Current development status
├── TESTING_FRAMEWORK_PLAN.md     # Detailed testing specs
└── AGENTS.md                     # This file
```

## CLI Commands

### Build
```bash
cd crow-tauri/src-tauri
cargo build --release --bin crow-cli
export PATH="$PATH:$(pwd)/target/release"
```

### Chat Commands (Main Usage)
```bash
crow-cli chat "message"              # New session, full verbose streaming
crow-cli chat --quiet "msg"          # Just the final response
crow-cli chat --json "msg"           # JSON output, no streaming
crow-cli chat --session ID "msg"     # Continue specific session (use sparingly!)
```

### Session Commands
```bash
crow-cli sessions                    # List all sessions
crow-cli new "Title"                 # Create new session explicitly
crow-cli session info <id>           # Session details
crow-cli session history <id>        # Message timeline
crow-cli session todo <id>           # Todo list state
```

### Debug Commands
```bash
crow-cli logs [count]                # Recent agent logs
crow-cli prompt [agent]              # Dump full system prompt
crow-cli paths                       # Show storage paths
crow-cli snapshot list               # List snapshots
crow-cli snapshot diff               # Show file changes
```

## CLI Output Format

After each response, CLI shows:
```
═══════════════════════════════════════════════════════════════
✓ ~183 thinking, ~68 response | 1 tool calls | 37.4s
Cost: $0.0036 | Context: 19k/128k (15.3%)
Session: ses_Vdh2gwl6PAsSbthwhpzGQo
═══════════════════════════════════════════════════════════════
```

## XDG Storage Paths

| Component | Path |
|-----------|------|
| Sessions | `~/.local/share/crow/storage/session/{projectID}/{sessionID}.json` |
| Messages | `~/.local/share/crow/storage/message/{sessionID}/{messageID}.json` |
| Todos | `~/.local/share/crow/storage/todo/{sessionID}.json` |
| Snapshots | `~/.local/share/crow/snapshots/{project_id}/` |
| Agent Log | `~/.local/state/crow/logs/agent.log` |
| Tool Calls | `~/.local/state/crow/logs/tool-calls.jsonl` |
| Config | `~/.config/crow/AGENTS.md` |
| Session Export | `.crow/sessions/{sessionID}.md` (project-relative) |

## Environment Variables

```bash
ANTHROPIC_API_KEY    # Anthropic Claude API key
OPENAI_API_KEY       # OpenAI API key
MOONSHOT_API_KEY     # Moonshot/Kimi API key
RUST_LOG=debug       # Enable debug logging
```

## Testing

Crow has three levels of testing: **Unit Tests**, **Integration Tests**, and **E2E Tests**. 

### Quick Reference
```bash
# Unit/Integration tests (fast, no API key needed)
cd crow-tauri/src-tauri/core
cargo test

# E2E tests (requires API key, tests real agent behavior)
bash crow-tauri/src-tauri/core/tests/e2e/test_crow_e2e.sh

# Interactive E2E testing
crow-cli new "My Test"  # Create fresh session
crow-cli chat --session ses_xxx "Test something"
```

---

## 1. Unit Tests (Rust)

Run all unit tests:
```bash
cd crow-tauri/src-tauri/core
cargo test
```

Run specific test file:
```bash
cargo test --lib tools::bash    # All bash tool tests
cargo test --lib tools::edit    # All edit tool tests
cargo test --lib session::store # Session store tests
```

Run single test:
```bash
cargo test test_bash_echo
cargo test test_edit_simple_replacement
```

**Current test counts (~384 tests):**
| Module | Tests |
|--------|-------|
| tools/bash.rs | 21 |
| tools/edit.rs | 51 |
| tools/read.rs | 21 |
| tools/write.rs | 17 |
| tools/grep.rs | 15 |
| tools/glob.rs | 16 |
| tools/list.rs | 18 |
| tools/patch.rs | 23 |
| tools/webfetch.rs | 17 |
| tools/websearch.rs | 14 |
| session/store.rs | 30 |
| snapshot/mod.rs | 10 |

---

## 2. E2E Tests (Agent-Level)

E2E tests use `crow-cli` to run the actual agent with real API calls. These tests verify that tools work correctly when invoked by an LLM.

### Running E2E Tests
```bash
# Automated test suite (8 tests, ~3-5 minutes)
bash crow-tauri/src-tauri/core/tests/e2e/test_crow_e2e.sh
```

The script tests:
- Session creation/listing
- Bash tool (echo, pwd, pipes)
- Write tool (file creation)
- Read tool (file reading)
- Edit tool (find/replace)
- Grep tool (pattern search)
- Glob tool (file finding)
- List tool (directory listing)

### Interactive E2E Testing

For debugging or exploring behavior, run tests interactively:

```bash
# Step 1: Create a fresh test directory and session
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
crow-cli new "Interactive Test"
# Output: ses_xxxxx

# Step 2: Run test commands (use the session ID from step 1)
SESSION="ses_xxxxx"

# Test Bash
crow-cli chat --session "$SESSION" "Run: echo HELLO_TEST"

# Test Write
crow-cli chat --session "$SESSION" "Create file test.txt with content: Hello World"
cat test.txt  # Verify

# Test Edit
crow-cli chat --session "$SESSION" "Edit test.txt: change World to Crow"
cat test.txt  # Verify: should say "Hello Crow"

# Test Grep
echo "ERROR: bad" > log1.txt
echo "INFO: ok" > log2.txt
crow-cli chat --session "$SESSION" "Grep for ERROR"

# Step 3: Check session history and tool calls
crow-cli session history "$SESSION"
cat ~/.local/state/crow/logs/tool-calls.jsonl | tail -10
```

---

## 3. What Agents Should Look For in E2E Tests

E2E tests catch issues that unit tests cannot:

### ✅ Things to Verify
1. **Tool output appears in response** - Did the agent see and report the result?
2. **Files actually created/modified** - Check filesystem after write/edit operations
3. **Snapshots created** - `ls ~/.local/share/crow/snapshots/` should show new project
4. **Session persisted** - `crow-cli sessions` should list the session
5. **Reasonable token usage** - Check the cost/context stats in output

### 🚨 Red Flags to Watch For
1. **Tool called multiple times unnecessarily** - Model confusion, wastes tokens
   ```
   ✓ ~861 thinking, ~0 response | 10 tool calls | 76.0s  # BAD: 10 calls for simple task
   ✓ ~82 thinking, ~5 response | 1 tool calls | 9.5s    # GOOD: 1 call
   ```
2. **Empty response after tool call** - Tool succeeded but agent didn't report it
3. **Context growing too fast** - Check `Context: 63k/128k (50.2%)` in output
4. **Agent not using the right tool** - Asked for grep but used bash grep instead
5. **Timeout or SIGHUP** - Process killed, usually means infinite loop or hang

### 📋 E2E Test Checklist for New Features
When adding a new tool or feature:
1. [ ] Add unit tests in `tools/new_tool.rs`
2. [ ] Create a fresh session: `crow-cli new "Test NewTool"`
3. [ ] Run simple case: `crow-cli chat --session $S "Use newtool to do X"`
4. [ ] Verify output shows expected result
5. [ ] Verify side effects (files created, APIs called, etc.)
6. [ ] Check tool wasn't called multiple times
7. [ ] Verify session history: `crow-cli session history $S`
8. [ ] Check logs: `cat ~/.local/state/crow/logs/tool-calls.jsonl | tail -5`

---

## 4. Debugging Failed Tests

### Check Logs
```bash
# Recent tool calls (JSONL format)
cat ~/.local/state/crow/logs/tool-calls.jsonl | tail -10 | jq

# Agent execution log
cat ~/.local/state/crow/logs/agent.log | tail -50

# Enable debug logging
RUST_LOG=debug crow-cli chat "test"
```

### Check Session State
```bash
# List all sessions
crow-cli sessions

# Session details
crow-cli session info ses_xxxxx

# Message history with timestamps
crow-cli session history ses_xxxxx

# Todo state
crow-cli session todo ses_xxxxx
```

### Check Snapshots
```bash
# List all project snapshots
crow-cli snapshot list

# Check specific project
ls -la ~/.local/share/crow/snapshots/proj-*/

# See git status in snapshot
cd ~/.local/share/crow/snapshots/proj-xxxxx
git status
git log --oneline
```

---

## 5. Provider Configuration

Crow auto-detects providers based on environment variables:
1. `ANTHROPIC_API_KEY` → Anthropic Claude
2. `OPENAI_API_KEY` → OpenAI
3. `MOONSHOT_API_KEY` → Moonshot/Kimi (default fallback)

E2E tests work with any provider. Moonshot is the default if no keys are set.

## Agent Execution Model

### The ReACT Loop (execute_turn)

A single "turn" is NOT a single LLM call. It's a **ReACT loop** that keeps going until the agent stops calling tools.

```
execute_turn() {
    for iteration in 0..max_iterations (10) {
        response = LLM.call(messages, tools)
        
        if response.has_tool_calls() {
            for tool_call in response.tool_calls {
                result = execute_tool(tool_call)
                messages.push(tool_result)
            }
            continue  // <-- KEEPS LOOPING
        }
        
        // No tool calls - agent is done with this turn
        return response.text
        break
    }
}
```

So when you call `execute_turn`, the agent might:
1. Call `read` tool → get result → continue
2. Call `write` tool → get result → continue  
3. Call `bash` tool → get result → continue
4. Return final text → **turn ends**

One turn = full ReACT loop until agent responds with just text (no tools).

### Dual-Agent Architecture (subagent_type: "verified")

The dual-agent system pairs an **Executor** (build agent) with an **Arbiter** (verification agent).

```
┌─────────────────────────────────────────────────────────────┐
│  DUAL-AGENT TASK (single invocation via Task tool)         │
│                                                             │
│  ┌─────────────────┐         ┌─────────────────┐           │
│  │ EXECUTOR        │         │ ARBITER         │           │
│  │ (build agent)   │         │ (arbiter agent) │           │
│  │                 │         │                 │           │
│  │ Own session     │ ──────► │ Own session     │           │
│  │ Full ReACT loop │  turn   │ Full ReACT loop │           │
│  │                 │ ◄────── │                 │           │
│  └─────────────────┘ feedback└─────────────────┘           │
│                                      │                      │
│                              task_complete?                 │
│                                      │                      │
│                              YES ────┴───► DONE             │
└─────────────────────────────────────────────────────────────┘
```

**Flow:**
1. Task tool receives `subagent_type: "verified"`
2. Creates TWO sessions (once, not per step): Executor + Arbiter
3. Loop:
   - Executor does full ReACT turn (may call many tools)
   - Executor's turn rendered to markdown
   - Markdown sent to Arbiter as user message
   - Arbiter does full ReACT turn (can run tests, read files)
   - If Arbiter calls `task_complete` → done, return result
   - Otherwise, Arbiter's feedback sent to Executor
   - Repeat (max 5 steps)

**Key points:**
- Each agent has ONE session that persists across all steps
- Each "step" is a full ReACT loop, not a single LLM call
- Only the LATEST turn is passed between agents (not full session history)
- Arbiter has `task_complete` tool to signal verified completion
- Arbiter is NOT a subtask - it's internal, not visible to parent agents

**Files:**
- `agent/dual.rs` - DualAgentRuntime orchestration
- `agent/executor.rs` - ReACT loop (execute_turn)
- `tools/task.rs` - Task tool, handles `subagent_type: "verified"`
- `session/export.rs` - render_turn_to_markdown()

## Key Files to Know

- `tools/task.rs` - Subagent spawning (CRITICAL)
- `tools/todowrite.rs` - Task tracking (CRITICAL)
- `agent/executor.rs` - Main ReACT loop
- `agent/dual.rs` - Dual-agent orchestration
- `session/store.rs` - Session persistence
- `session/export.rs` - Turn rendering for dual-agent handoff
- `snapshot/mod.rs` - Shadow git tracking
- `bin/crow-cli.rs` - CLI implementation

## Development Workflow

1. Make changes to code
2. Build: `cargo build --package crow_core`
3. Test: `cargo test --package crow_core`
4. E2E test: `crow-cli chat "test the change"`
5. Verify XDG paths have correct data
6. Compare behavior with OpenCode (ignore LSP/permissions)
