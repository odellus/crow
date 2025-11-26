# Crow Agent Development Guide

This document provides instructions for AI agents working on the Crow codebase.

## Project Overview

Crow is a Tauri-based AI coding assistant. It provides:
- **crow_core**: Rust library with all backend logic (agents, tools, providers, sessions)
- **crow-cli**: Command-line interface for testing and direct usage
- **Tauri app**: Desktop application (frontend in React)

## Architecture

```
crow-tauri/
├── src/                          # React frontend
│   ├── components/               # UI components (ChatView, FileTree, etc.)
│   ├── hooks/                    # React hooks (useEventStream)
│   └── pages/                    # Page components
├── src-tauri/
│   ├── app/                      # Tauri application crate
│   │   └── src/
│   │       ├── main.rs
│   │       └── lib.rs
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
└── docs/                         # Documentation
```

## Key Modules

### Agent System (`agent/`)
- `executor.rs` - Main ReACT loop, orchestrates tool execution
- `prompt.rs` - System prompt builder with environment context
- `registry.rs` - Agent configurations (build, plan, general, etc.)
- `builtins.rs` - Built-in agent definitions
- `doom_loop.rs` - Detection/prevention of infinite loops

### Tools (`tools/`)
Available tools: `bash`, `edit`, `read`, `write`, `grep`, `glob`, `list`, `todoread`, `todowrite`, `webfetch`, `websearch`, `task`, `batch`, `multiedit`, `patch`

### Storage
- Sessions: `~/.local/share/crow/storage/`
- Snapshots: `~/.local/share/crow/snapshots/{project_id}/`
- Logs: `~/.local/share/crow/logs/`

## CLI Usage

```bash
# Build
cd crow-tauri/src-tauri
cargo build --release --bin crow-cli

# Add to PATH
export PATH="$PATH:$(pwd)/target/release"

# Commands
crow-cli repl                      # Interactive REPL
crow-cli chat "message"            # Single message
crow-cli chat --session ID "msg"   # Continue session
crow-cli sessions                  # List sessions
crow-cli new "Title"               # Create session
crow-cli messages <id>             # View history
crow-cli paths                     # Show storage paths
crow-cli logs [count]              # Recent logs
crow-cli prompt [agent]            # Dump system prompt
```

### REPL Commands
- `/exit` - Exit REPL
- `/new` - Create new session
- `/session` - Show current session ID
- `/help` - Show help

## Environment Variables

```bash
ANTHROPIC_API_KEY    # Anthropic Claude API key
OPENAI_API_KEY       # OpenAI API key
RUST_LOG=debug       # Enable debug logging
```

## Development

### Adding a New Tool
1. Create `tools/newtool.rs`
2. Implement `Tool` trait
3. Add to `tools/mod.rs`
4. Register in `ToolRegistry::new()`

### Testing
```bash
cargo check 2>&1 | grep warning
cargo test
crow-cli chat "test message"
```

## Current Status

- [x] Core library with all tools
- [x] CLI with full observability (streaming, colors)
- [x] XDG storage persistence
- [x] Shadow git snapshots
- [x] Interactive REPL mode
- [x] Multi-provider support (Anthropic, OpenAI, Moonshot)
- [ ] Tauri commands layer
- [ ] Frontend wiring
