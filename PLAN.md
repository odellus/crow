# Crow-Tauri Development Plan

## Current Status: CLI Complete, Frontend Pending

The core Rust backend and CLI are fully functional. Next step is wiring up the Tauri commands and React frontend.

---

## Completed

### Phase 1-3: Core Infrastructure
- [x] Workspace structure (core + app crates)
- [x] Core library without dioxus/axum dependencies
- [x] XDG directory setup (~/.local/share/crow/, etc.)
- [x] CLI binary with full observability
- [x] Streaming output with colors
- [x] Interactive REPL mode
- [x] Session persistence
- [x] Shadow git snapshots
- [x] Multi-provider support (Anthropic, OpenAI, Moonshot)
- [x] Tool implementations (bash, edit, read, write, grep, glob, etc.)
- [x] Agent system (build, plan, general agents)
- [x] Task tool for sub-agent spawning

---

## In Progress

### Phase 4: Tauri Commands

Wire up crow_core to Tauri commands so the React frontend can call them.

```
app/src/commands/
├── mod.rs
├── session.rs    # list, create, get, delete
├── message.rs    # send (streaming), list
└── file.rs       # list, read
```

Key commands needed:
- `list_sessions()` -> Vec<Session>
- `create_session(title)` -> Session
- `send_message(session_id, text)` -> streaming via Channel
- `list_messages(session_id)` -> Vec<Message>

---

## Pending

### Phase 5: Frontend Migration

- Replace fetch() with Tauri invoke()
- Handle streaming via Tauri Channels
- Update useEventStream hook
- Wire up ChatView, SessionList components

### Phase 6: Polish

- Error handling
- Loading states
- Build & package

---

## Quick Reference

### Build & Run
```bash
cd crow-tauri/src-tauri
cargo build --release --bin crow-cli
./target/release/crow-cli repl
```

### Storage Paths
```
~/.local/share/crow/storage/    # Sessions
~/.local/share/crow/snapshots/  # Git snapshots
~/.local/share/crow/logs/       # Logs
~/.config/crow/                 # Config
```

### Environment
```bash
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
RUST_LOG=debug
```
