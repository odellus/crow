# Crow Master Plan

## Current State

Crow is at ~65% API parity with OpenCode. Core streaming, sessions, and tools work. But critical gaps block a usable product:

- No `/event` SSE stream (global event bus)
- No permission system
- No message revert/checkpoint system
- No frontend

## The Problem with A2A

A2A protocol was considered but is premature:
1. No product to distribute yet
2. Adds JSON-RPC complexity without clear benefit
3. Protocol is young (March 2025), still evolving
4. Doesn't solve actual gaps

## The Real Gap: Code-Tailored Protocol

OpenCode's strength isn't just the API - it's the **Agentic Control Protocol (ACP)** design philosophy:
- Checkpoint/revert for code changes
- File operation batching
- Diff-based editing with conflict resolution
- Tool approval workflows

We need something similar but exposed via **SSE or JSON-RPC 2.0** for cleaner client integration.

---

## Phase 1: Core Infrastructure (1-2 weeks)

### 1.1 Global Event Bus
Replace per-request streaming with global SSE stream.

```
GET /event?session_id={id}
```

Events:
- `message.start` / `message.delta` / `message.complete`
- `tool.start` / `tool.complete`
- `session.lock` / `session.unlock`
- `error`

### 1.2 Snapshot Infrastructure (NOT a tool)
**Important:** Snapshots are internal plumbing, not an LLM tool. The agent never calls "snapshot" - the system tracks changes automatically.

Called from:
- Session processor - `track()` before agent runs
- Edit/Write/Bash tools - record patches after file changes
- Revert endpoint - uses patches to undo

```rust
// Internal infrastructure - not in tool registry
struct SnapshotManager {
    git_dir: PathBuf,      // .crow/snapshot/{project_id}
    work_tree: PathBuf,
}

struct Patch {
    hash: String,          // git tree hash before change
    files: Vec<PathBuf>,   // files modified
}
```

Endpoints:
- `POST /session/{id}/revert/{message_id}` - revert to message
- `POST /session/{id}/unrevert` - undo revert
- `GET /session/{id}/diff` - get current diff from snapshot

### 1.3 Permission System
Tool execution requires approval for dangerous operations.

```rust
enum PermissionLevel {
    Auto,      // Always allow (read, grep)
    Once,      // Ask once per session (write, edit)  
    Always,    // Ask every time (bash, delete)
}
```

Flow:
1. Tool requests permission via event
2. Client approves/denies via endpoint
3. Tool proceeds or aborts

Endpoint:
- `POST /session/{id}/permission` - approve/deny pending permission

---

## Phase 2: Git-Based Snapshot System (1-2 weeks)

OpenCode's approach is simple and battle-tested: use git itself as the snapshot engine.

### 2.1 Shadow Git Repository
Create a hidden git repo to track all file changes:

```rust
struct SnapshotManager {
    git_dir: PathBuf,      // e.g., .crow/snapshot/{project_id}
    work_tree: PathBuf,    // project root
}

impl SnapshotManager {
    // Create tree hash before changes
    async fn track(&self) -> Option<String> {
        // git add . && git write-tree
    }
    
    // Get list of changed files since snapshot
    async fn patch(&self, hash: &str) -> Patch {
        // git diff --name-only {hash}
    }
    
    // Restore specific files to snapshot state
    async fn revert(&self, patches: Vec<Patch>) {
        // git checkout {hash} -- {file}
    }
    
    // Full restore to snapshot
    async fn restore(&self, hash: &str) {
        // git read-tree {hash} && git checkout-index -a -f
    }
    
    // Get unified diff for display
    async fn diff(&self, hash: &str) -> String {
        // git diff {hash}
    }
}
```

### 2.2 Patch Tracking
Each tool that modifies files records a patch:

```rust
struct Patch {
    hash: String,           // git tree hash before change
    files: Vec<PathBuf>,    // files that were modified
}
```

Store patches as message parts so revert knows what to undo.

### 2.3 Revert Flow
1. User clicks revert on message N
2. Collect all patches from messages N+1 to end
3. For each file in patches, `git checkout {hash} -- {file}`
4. Delete messages N+1 to end
5. Clear revert state

### 2.4 Diff Display
Show users what changed:
- `git diff {snapshot}` for unified diff
- Parse into per-file diffs with additions/deletions count
- Display in frontend with syntax highlighting

No custom diff algorithms needed - git handles everything.

---

## Phase 3: Frontend MVP (1-2 weeks)

### Stack
- React 18 + TypeScript
- Tailwind CSS
- CodeMirror 6
- Vite

### Core Features
1. Session management (create, list, switch)
2. Chat interface with streaming
3. Tool call display with approval UI
4. File diff viewer
5. Checkpoint/revert controls

### Architecture
```
src/
  api/           # API client, SSE handler
  context/       # SessionContext, EventContext
  components/
    Chat/        # Messages, input, streaming
    Tools/       # Tool calls, approvals
    Editor/      # CodeMirror, diffs
    Session/     # List, create, switch
  hooks/         # useSession, useEvents, usePermission
```

---

## Phase 4: Polish & Extensions (1-2 weeks)

### 4.1 Multi-file Context
- File tree with selection
- Drag-drop to add context
- Token counting

### 4.2 Model Switching
- Provider configuration UI
- Model selection per session
- Cost tracking

### 4.3 Keyboard Shortcuts
- Vim-style navigation
- Quick actions (approve all, revert, etc.)

### 4.4 Persistence
- Session history
- User preferences
- Recent files

---

## Implementation Order

| Week | Focus | Deliverable |
|------|-------|-------------|
| 1 | Event bus + Snapshots | Global SSE, git-based snapshots |
| 2 | Revert + Permissions | Revert API, approval flow |
| 3 | Basic Frontend | Chat UI, streaming, tool display |
| 4 | Frontend + Diff viewer | CodeMirror, diff display |
| 5 | Polish | Multi-file, shortcuts, persistence |

---

## Why Not A2A (Yet)

A2A makes sense when:
- Crow is stable and feature-complete
- Other agents/systems want to integrate
- The protocol matures

For now, focus on:
1. Making crow actually usable for coding
2. Code-specific features (revert, diffs, transactions)
3. Clean frontend experience

A2A can be added as an adapter layer later without changing core architecture.

---

## Success Criteria

MVP is complete when:
- [ ] Can start session and chat with streaming
- [ ] Tool calls display with approve/deny
- [ ] Can revert to any previous message
- [ ] File edits show as diffs
- [ ] Multiple sessions work independently
- [ ] Keyboard-driven workflow possible

---

## Open Questions

1. **Storage format** - SQLite vs flat files for checkpoints?
2. **Diff algorithm** - Use existing crate or custom for code-aware diffing?
3. **Permission persistence** - Remember approvals across sessions?
4. **WebSocket vs SSE** - SSE simpler but WebSocket allows bidirectional

---

## Files to Create/Modify

### New Files
- `crow/packages/api/src/events/mod.rs` - Event bus
- `crow/packages/api/src/checkpoint/mod.rs` - Checkpoint system
- `crow/packages/api/src/permission/mod.rs` - Permission system
- `crow/packages/api/src/transaction/mod.rs` - File transactions
- `crow/packages/web/` - Frontend application

### Modified Files
- `crow/packages/api/src/server.rs` - New endpoints
- `crow/packages/api/src/agent/executor.rs` - Event emission
- `crow/packages/api/src/tools/*.rs` - Permission checks
- `crow/packages/api/src/session/mod.rs` - Checkpoint integration
