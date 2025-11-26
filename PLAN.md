# Crow-Tauri Development Plan

## Current Status: Testing Phase

The core Rust backend and CLI are functional. **Current focus: comprehensive testing.**

---

## 🚨 NEXT STEP: Testing Framework

**See: [TESTING_FRAMEWORK_PLAN.md](./TESTING_FRAMEWORK_PLAN.md) for detailed test specs.**

### Critical Priority (Do First)
1. **Task Tool** - 0 tests → 30+ tests (MOST IMPORTANT)
2. **TodoWrite/TodoRead** - 1 test → 25+ tests (CRITICAL)

### High Priority
3. Glob, List, Batch, Patch, Session Store, Grep (all 0 tests → 15+ each)

### Testing Rules
- ⚠️ **ALWAYS create new session** for each test - reusing causes weird behavior
- ⚠️ **Verify XDG paths** - check files in `~/.local/share/crow/`
- ⚠️ **Compare with OpenCode** but IGNORE LSP and permissions features
- Every tool needs: Unit tests, Integration tests, E2E tests

---

## Completed

### Core Infrastructure
- [x] Workspace structure (core + app crates)
- [x] Core library without dioxus/axum dependencies
- [x] XDG directory setup (~/.local/share/crow/, etc.)
- [x] CLI binary with full observability
- [x] Streaming output with colors and cost tracking
- [x] Interactive REPL mode
- [x] Session persistence
- [x] Shadow git snapshots
- [x] Multi-provider support (Anthropic, OpenAI, Moonshot)
- [x] Tool implementations (bash, edit, read, write, grep, glob, etc.)
- [x] Agent system (build, plan, general agents)
- [x] Task tool for sub-agent spawning
- [x] Tool call logging to JSONL
- [x] Cost and context usage display

---

## Pending

### Testing (Current Focus)
- [ ] Task tool tests (CRITICAL)
- [ ] TodoWrite/TodoRead tests (CRITICAL)
- [ ] All other tools (see TESTING_FRAMEWORK_PLAN.md)

### Phase 4: Tauri Commands
Wire up crow_core to Tauri commands.

### Phase 5: Frontend Migration
Replace fetch() with Tauri invoke().

---

## Quick Reference

### Build & Run CLI
```bash
cd crow-tauri/src-tauri
cargo build --release --bin crow-cli
./target/release/crow-cli chat "your message"
```

### Run Tests
```bash
cd crow-tauri/src-tauri
cargo test --package crow_core
```

### Storage Paths (XDG)
```
~/.local/share/crow/storage/session/   # Sessions
~/.local/share/crow/storage/todo/      # Todos
~/.local/share/crow/snapshots/         # Git snapshots
~/.local/state/crow/logs/              # Logs
~/.config/crow/                        # Config
```

### Environment
```bash
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
RUST_LOG=debug
```
