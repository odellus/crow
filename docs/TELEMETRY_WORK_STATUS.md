# Telemetry Work Status

## What Was Completed

### 1. Added Metadata to Sessions and Messages ✅
- Added `metadata: Option<serde_json::Value>` to `Session` struct (types.rs)
- Added `metadata: Option<serde_json::Value>` to both `User` and `Assistant` message variants
- Added `SessionStore.update_metadata()` method

### 2. Created SessionExport Module ✅
- File: `crow/packages/api/src/session/export.rs`
- `SessionExport::to_markdown()` - converts session to markdown
- `SessionExport::write_to_file()` - writes markdown to disk
- Formats messages with roles, timestamps, tokens, tool calls

### 3. Created CrowStorage ✅
- File: `crow/packages/api/src/storage/crow.rs`
- Creates `.crow/` directory in server's CWD (not ~/.crow)
- Structure:
  ```
  .crow/
  ├── sessions/       # Session markdown exports
  ├── conversations/  # Dual-agent conversations
  ├── logs/          # Server logs
  └── .gitignore     # Auto-generated
  ```
- `CrowStorage::session_export_path()` - get path for session markdown

### 4. Added Imports to Runtime ✅
- Imported `SessionExport` and `CrowStorage` into runtime.rs
- Ready to add export calls

## What Was NOT Done

### Export Calls After Each Turn ❌
We need to add in `DualAgentRuntime::run()`:
```rust
let storage = CrowStorage::new()?;

// After executor turn:
let executor_path = storage.session_export_path(&executor_session_id);
SessionExport::write_to_file(&self.sessions, &executor_session_id, &executor_path)?;

// After discriminator turn:
let discriminator_path = storage.session_export_path(&discriminator_session_id);
SessionExport::write_to_file(&self.sessions, &discriminator_session_id, &discriminator_path)?;
```

### Testing ❌
Haven't tested the markdown export yet.

## How to Complete This

If you want to finish telemetry:
1. Add the export calls after each turn in runtime.rs
2. Test with `POST /session/dual`
3. Check `.crow/sessions/ses-XXX.md` files are created

If you want to revert:
```bash
cd crow
git checkout packages/api/src/types.rs
git checkout packages/api/src/session/store.rs
git checkout packages/api/src/agent/runtime.rs
rm packages/api/src/session/export.rs
rm packages/api/src/storage/crow.rs
```

## Why We Stopped

**Critical discovery:** Dual-agent in OpenCode is invoked as a **subtask**, not a top-level API endpoint.

We need to:
1. Understand OpenCode's subtask/subagent system
2. Implement subtask spawning
3. Make dual-agent the **default subtask** for BUILD agent
4. BUILD agent should spawn dual-agent automatically

The telemetry work is still valid - we'll export both executor and discriminator sessions to markdown - but the invocation model needs to change.
