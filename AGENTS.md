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

### Run Tests
```bash
cd crow-tauri/src-tauri
cargo test --package crow_core
cargo test --package crow_core -- test_name  # Specific test
```

### E2E Testing Pattern
```bash
# 1. Run command (fresh session)
crow-cli chat --json "test prompt"

# 2. Verify XDG files created
ls ~/.local/share/crow/storage/session/*/
cat ~/.local/state/crow/logs/tool-calls.jsonl | tail -1 | jq

# 3. Check session export
cat .crow/sessions/*.md | tail -20
```

## Current Focus: Testing

**Priority order for testing (see TESTING_FRAMEWORK_PLAN.md):**

1. 🚨 **Task Tool** - Most critical, 0 tests currently
2. 🚨 **TodoWrite/TodoRead** - Critical for task tracking
3. Glob, List, Batch, Patch - No tests
4. Session Store - No tests
5. Everything else

## Key Files to Know

- `tools/task.rs` - Subagent spawning (CRITICAL)
- `tools/todowrite.rs` - Task tracking (CRITICAL)
- `agent/executor.rs` - Main ReACT loop
- `session/store.rs` - Session persistence
- `snapshot/mod.rs` - Shadow git tracking
- `bin/crow-cli.rs` - CLI implementation

## Development Workflow

1. Make changes to code
2. Build: `cargo build --package crow_core`
3. Test: `cargo test --package crow_core`
4. E2E test: `crow-cli chat "test the change"`
5. Verify XDG paths have correct data
6. Compare behavior with OpenCode (ignore LSP/permissions)
