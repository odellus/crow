# Crow Agent Development Guide

This document provides comprehensive instructions for AI agents working on the Crow codebase.

## Project Overview

Crow is a Tauri-based AI coding assistant, similar to Claude Code / OpenCode. It provides:
- **crow_core**: Rust library with all backend logic (agents, tools, providers, sessions)
- **crow-cli**: Command-line interface for testing and direct usage
- **Tauri app**: Desktop application (frontend in progress)

## Architecture

```
crow-tauri/
├── src-tauri/
│   └── core/                    # crow_core library
│       ├── src/
│       │   ├── agent/           # Agent system (ReACT loop, prompts)
│       │   ├── bin/crow-cli.rs  # CLI binary
│       │   ├── bus/             # Event bus for real-time updates
│       │   ├── config/          # Configuration loading
│       │   ├── prompts/         # System prompt templates
│       │   ├── providers/       # LLM provider clients
│       │   ├── session/         # Session & message storage
│       │   ├── snapshot/        # Shadow git for file tracking
│       │   ├── storage/         # XDG-based persistence
│       │   └── tools/           # Tool implementations
│       └── Cargo.toml
└── AGENTS.md                    # This file
```

## Key Modules

### Agent System (`agent/`)
- `executor.rs` - Main ReACT loop, orchestrates tool execution
- `prompt.rs` - System prompt builder with environment context
- `registry.rs` - Agent configurations (build, supervisor, etc.)
- `builtins.rs` - Built-in agent definitions
- `doom_loop.rs` - Detection/prevention of infinite loops

### Tools (`tools/`)
Each tool implements the `Tool` trait:
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult;
}
```

Available tools:
- `bash` - Shell command execution
- `edit` - File modification with fuzzy matching
- `read` - File reading with line numbers
- `write` - File creation
- `grep` - Content search with regex
- `glob` - File pattern matching
- `list` - Directory listing
- `todoread/todowrite` - Task management
- `webfetch/websearch` - Web operations
- `task` - Sub-agent spawning
- `batch` - Parallel tool execution
- `multiedit` - Multiple file edits
- `patch` - Unified diff application

### Snapshot System (`snapshot/`)
Shadow git for tracking file changes:
- Stores snapshots in `~/.local/share/crow/snapshots/{project_id}/`
- Project ID derived from git root commit hash
- Enables undo/redo and change visualization
- Auto-initialized in agent executor for every working directory

### Session Storage (`session/`)
- Sessions stored in `~/.local/share/crow/sessions/`
- SQLite-backed message history
- Lock manager prevents concurrent modifications

### Providers (`providers/`)
LLM provider abstraction:
- Anthropic (Claude)
- OpenAI
- Moonshot (default fallback)
- Custom endpoints via config

## CLI Usage

```bash
# Build
cd crow-tauri/src-tauri
cargo build --release --bin crow-cli

# The binary is at:
./target/release/crow-cli

# Or add to PATH for convenience:
export PATH="$PATH:/home/thomas/src/projects/opencode-project/crow-tauri/src-tauri/target/release"
```

### Basic Commands
```bash
crow-cli chat "your message"           # Full verbose streaming
crow-cli chat --quiet "message"        # Just response  
crow-cli chat --json "message"         # JSON output for scripting
crow-cli chat --session <id> "msg"     # Continue existing session
crow-cli sessions                      # List sessions
crow-cli new "Session Title"           # Create new session
crow-cli messages <session-id>         # View full history with parts
crow-cli paths                         # Show XDG storage paths
crow-cli logs [count]                  # Show recent agent execution logs
crow-cli prompt [agent]                # Dump full system prompt for agent
crow-cli repl [session-id]             # Interactive REPL (for humans only!)
```

### For AI Agents: Chained CLI Calls (NOT REPL)

**IMPORTANT:** AI agents should NEVER use `crow-cli repl`. Instead, chain CLI calls:

```bash
# Create a session and capture the ID (first line of output)
SESSION_ID=$(crow-cli new "Test Session" 2>/dev/null | head -1)
echo "Session: $SESSION_ID"

# Send messages to that session (full verbose output - see everything!)
crow-cli chat --session "$SESSION_ID" "list files"

# Continue the conversation
crow-cli chat --session "$SESSION_ID" "now create hello.txt"

# Check session history
crow-cli messages "$SESSION_ID"

# Inspect XDG storage directly
ls -la ~/.local/share/crow/sessions/
cat ~/.local/share/crow/sessions/${SESSION_ID}.json | jq '.title'
cat ~/.local/share/crow/sessions/${SESSION_ID}.json | jq '.messages | length'
```

**For scripting (less preferred):** Use `--json` mode when you need structured parsing:
```bash
crow-cli chat --session "$SESSION_ID" --json "list files" | jq '.response'
crow-cli chat --session "$SESSION_ID" --json "create file" | jq '.tools'
```

**Why not REPL for agents?**
- REPL requires interactive stdin which agents can't provide reliably
- Chained calls are explicit and debuggable
- Can inspect XDG storage between calls
- Full verbose output shows thinking, tools, and response

### REPL Mode (For Humans Only)

The REPL is for **human** interactive use:

```bash
# Start new REPL session
crow-cli repl

# Resume existing session  
crow-cli repl ses_abc123
```

**REPL Commands:** `/exit`, `/new`, `/session`, `/help`

**During Execution:**
- **Type + Enter** - Interrupt agent and send new message
- **Ctrl+C** - Abort execution
- **Ctrl+D** - Exit (at prompt)

### Debugging & Verification
```bash
# Show XDG paths and verify storage locations
crow-cli paths

# Enable debug logging (shows model selection, config loading)
RUST_LOG=debug crow-cli chat "test"

# Dump system prompt to verify agent config is loaded
crow-cli prompt build      # Default build agent
crow-cli prompt general    # General subagent (should show custom prompt if configured)
crow-cli prompt plan       # Plan agent with restricted permissions

# JSON output for machine analysis
crow-cli chat --json "list files" | jq '.tools'

# Test specific session continuity
crow-cli new "Test Session"           # Note the session ID
crow-cli chat --session ses_xxx "hello"
crow-cli chat --session ses_xxx "what did I just say?"
crow-cli messages ses_xxx             # Verify history persisted
```

## Development Preferences

### CLI Output Colors
- Blue: Agent thinking/reasoning
- Green: Tool calls / success
- Yellow: Tool results
- Red: Errors  
- White: Response text
- Cyan: Headers, tool names

### Tool Icons in CLI
```
🔧 bash      - Shell commands
📝 edit      - File modifications (+green/-red diff)
📖 read      - File reading
🔍 grep      - Search results
📁 list/glob - Directory listing
📋 todo      - Task management
🌐 web       - Web operations
```

### XDG Directories
All data persists across projects/sessions:
```
~/.local/share/crow/     # Data (sessions, snapshots)
~/.config/crow/          # Configuration (agents, providers)
~/.cache/crow/           # Cache
~/.local/state/crow/     # Logs
```

## Verifying XDG Storage

### Check Directory Structure
```bash
# Show all crow directories
crow-cli paths

# Verify directories exist
ls -la ~/.local/share/crow/
ls -la ~/.config/crow/
ls -la ~/.local/state/crow/

# Expected structure:
tree ~/.local/share/crow/
# ~/.local/share/crow/
# ├── sessions/           # Session JSON files
# │   └── ses_xxx.json
# └── snapshots/          # Shadow git per project
#     └── {project_id}/
#         └── .git/

tree ~/.config/crow/
# ~/.config/crow/
# ├── config.json         # User config (optional)
# └── agent/              # Custom agent definitions
#     └── general.md      # Override general agent

tree ~/.local/state/crow/
# ~/.local/state/crow/
# └── logs/
#     └── agent.log       # Execution logs
```

### Verify Agent Config Loading
```bash
# Create a custom agent config
mkdir -p ~/.config/crow/agent
cat > ~/.config/crow/agent/general.md << 'EOF'
---
description: Custom general agent for research
mode: subagent
tools:
  todoread: false
  todowrite: false
---

You are a custom research agent with special instructions.
This prompt should appear in the system prompt dump.
EOF

# Verify it loads (look for "Custom general agent" or custom prompt text)
crow-cli prompt general

# The custom prompt should appear in System Message 2
# If you only see the default qwen.txt/anthropic.txt content, config loading is broken
```

### Verify Session Persistence
```bash
# Create a session and send a message
SESSION=$(crow-cli new "Persistence Test" 2>&1 | head -1)
echo "Created session: $SESSION"

crow-cli chat --session "$SESSION" "Remember the code word is BANANA"

# Check session file exists
ls -la ~/.local/share/crow/sessions/

# Verify message persisted (should show BANANA in history)  
crow-cli messages "$SESSION"

# Test continuity - agent should remember BANANA
crow-cli chat --session "$SESSION" "What was the code word?"
```

### Verify Snapshot System
```bash
# Go to a test directory with git
cd /tmp
mkdir snapshot-test && cd snapshot-test
git init
echo "original" > test.txt
git add . && git commit -m "initial"

# Run agent that modifies a file
crow-cli chat "Write 'modified by agent' to test.txt"

# Check snapshot was created
PROJECT_ID=$(git rev-list --max-parents=0 HEAD | head -c 8)
ls -la ~/.local/share/crow/snapshots/

# The snapshot dir should exist and contain .git
ls -la ~/.local/share/crow/snapshots/${PROJECT_ID}*/

# Verify the file was actually modified
cat test.txt
```

### Verify Logging
```bash
# Run a chat and check logs
crow-cli chat "list files"

# Check agent log
cat ~/.local/state/crow/logs/agent.log | tail -20

# Log should show: timestamp, session_id, agent, model, tokens, cost
```

## Testing

```bash
# Check for warnings (should be zero)
cargo check 2>&1 | grep warning

# Run tests
cargo test

# Test CLI interactively
cd /path/to/test/project
ANTHROPIC_API_KEY=sk-... crow-cli chat "list files"

# Test with JSON output for analysis
crow-cli chat --json "test" | jq .
```

## Code Style

### Rust Conventions
- Use `colored` crate for terminal colors
- Prefer `tracing` over `println!` for logs
- Use `async_trait` for async trait methods
- Error handling: Return `Result<T, String>` from tools
- Use `serde_json::json!` for JSON construction

### Adding a New Tool
1. Create `tools/newtool.rs`
2. Implement `Tool` trait
3. Add `pub mod newtool;` to `tools/mod.rs`
4. Add `pub use newtool::NewTool;` to `tools/mod.rs`
5. Register in `ToolRegistry::new()` in `tools/mod.rs`

### Common Patterns

**Reading files safely:**
```rust
let content = tokio::fs::read_to_string(&path).await
    .map_err(|e| format!("Failed to read {}: {}", path, e))?;
```

**Tool result construction:**
```rust
ToolResult {
    status: ToolStatus::Completed,
    output: serde_json::to_string(&output).unwrap_or_default(),
    error: None,
    metadata: json!({ "filepath": path }),
}
```

**Streaming events:**
```rust
let _ = event_tx.send(ExecutionEvent::TextDelta {
    id: part_id.clone(),
    delta: text.clone(),
});
```

## Environment Variables

```bash
ANTHROPIC_API_KEY    # Anthropic Claude API key
OPENAI_API_KEY       # OpenAI API key  
RUST_LOG=debug       # Enable debug logging
CROW_VERBOSE_LOG=1   # Log full requests/responses to disk
```

## Reference Implementation

This codebase follows patterns from OpenCode (`../opencode/`):
- Tool implementations: `packages/opencode/src/tool/`
- Agent configs: `packages/opencode/src/agent/`
- Session management: `packages/opencode/src/session/`

## Current Status

- [x] Core library with all tools
- [x] CLI with full observability
- [x] XDG storage persistence
- [x] Shadow git snapshots
- [x] Streaming with thinking tokens
- [x] Interactive REPL mode
- [x] Type-to-interrupt during execution
- [x] Session continuity in REPL
- [ ] Structured trace logging (Langfuse/Phoenix style)
- [ ] Tauri commands (pending)
- [ ] Frontend migration (pending)
