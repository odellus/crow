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
cd crow-tauri/src-tauri/core
cargo build --release --bin crow-cli

# Run
crow-cli chat "your message"           # Full verbose streaming
crow-cli chat --quiet "message"        # Just response
crow-cli chat --json "message"         # JSON output for scripting
crow-cli sessions                      # List sessions
crow-cli messages <session-id>         # View history
crow-cli paths                         # Show storage paths
crow-cli prompt [agent]                # Dump system prompt
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
~/.config/crow/          # Configuration
~/.cache/crow/           # Cache
~/.local/state/crow/     # Logs
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
- [ ] Tauri commands (in progress)
- [ ] Frontend migration
