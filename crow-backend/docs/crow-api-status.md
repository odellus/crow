# Crow API Status vs OpenCode

## Completeness Score: ~65%

Crow has the core agent execution and session management implemented, but is missing several OpenCode endpoints that are needed for full API parity.

---

## Feature Matrix

### App Endpoints

| Endpoint | OpenCode | Crow | Status |
|----------|----------|------|--------|
| `GET /app` | Get app info | Not implemented | ❌ |
| `POST /app/init` | Initialize the app | Not implemented | ❌ |

**Notes:** Low priority for headless API usage. Can return hardcoded values.

---

### Config Endpoints

| Endpoint | OpenCode | Crow | Status |
|----------|----------|------|--------|
| `GET /config` | Get config info | `GET /config` - Returns basic info | 🚧 Partial |
| `GET /config/providers` | List providers with defaults | `GET /config/providers` - Returns hardcoded list | 🚧 Partial |

**Missing:**
- Full config structure (disabled tools, keybinds, etc.)
- Dynamic provider list based on auth.json
- Default model per provider mapping

---

### Sessions Endpoints

| Endpoint | OpenCode | Crow | Status |
|----------|----------|------|--------|
| `GET /session` | List sessions | ✅ Implemented | ✅ |
| `GET /session/:id` | Get session | ✅ Implemented | ✅ |
| `GET /session/:id/children` | List child sessions | ✅ Implemented | ✅ |
| `POST /session` | Create session | ✅ Implemented | ✅ |
| `DELETE /session/:id` | Delete session | ✅ Implemented | ✅ |
| `PATCH /session/:id` | Update session | ✅ Implemented | ✅ |
| `POST /session/:id/init` | Analyze app, create AGENTS.md | Not implemented | ❌ |
| `POST /session/:id/abort` | Abort running session | ✅ Implemented | ✅ |
| `POST /session/:id/share` | Share session | Not implemented | ⏭️ Skip |
| `DELETE /session/:id/share` | Unshare session | Not implemented | ⏭️ Skip |
| `POST /session/:id/summarize` | Summarize session | Not implemented | ❌ |
| `GET /session/:id/message` | List messages | ✅ Implemented | ✅ |
| `GET /session/:id/message/:messageID` | Get message | ✅ Implemented | ✅ |
| `POST /session/:id/message` | Send chat message | ✅ Implemented | ✅ |
| `POST /session/:id/shell` | Run shell command | Not implemented | ❌ |
| `POST /session/:id/revert` | Revert a message | Not implemented | ❌ |
| `POST /session/:id/unrevert` | Restore reverted messages | Not implemented | ❌ |
| `POST /session/:id/permissions/:permissionID` | Permission response | Not implemented | ❌ |

**Missing Critical:**
- `POST /session/:id/shell` - Direct shell execution without agent
- `POST /session/:id/revert` - Important for undo functionality
- Permission system for approve/reject workflow

**Intentionally Skipped:**
- Share/unshare - Requires external service integration

---

### Messages & Streaming

| Feature | OpenCode | Crow | Status |
|---------|----------|------|--------|
| Non-streaming message | `POST /session/:id/message` | ✅ Implemented | ✅ |
| Streaming message | Via `/event` SSE | `POST /session/:id/message/stream` | 🚧 Different |
| noReply flag | Supported | ✅ Implemented | ✅ |

**Note:** Crow uses a dedicated streaming endpoint instead of OpenCode's global `/event` bus. This is a design divergence but functionally equivalent for agent workflows.

---

### Files Endpoints

| Endpoint | OpenCode | Crow | Status |
|----------|----------|------|--------|
| `GET /find?pattern=<pat>` | Search text in files | Not implemented | ❌ |
| `GET /find/file?query=<q>` | Find files by name | Not implemented | ❌ |
| `GET /find/symbol?query=<q>` | Find workspace symbols | Not implemented | ⏭️ Skip (LSP) |
| `GET /file?path=<path>` | Read a file | `GET /file/content?path=` | ✅ Different path |
| `GET /file/status` | Get tracked file status | Not implemented | ❌ |

**Missing Critical:**
- `/find` - Text search (can use grep tool internally)
- `/find/file` - File name search (can use glob tool)
- `/file/status` - Git status for tracked files

**Intentionally Skipped:**
- `/find/symbol` - Requires LSP integration

---

### Logging Endpoints

| Endpoint | OpenCode | Crow | Status |
|----------|----------|------|--------|
| `POST /log` | Write log entry | Not implemented | ❌ |

**Notes:** Low priority. Can be added later for client-side logging.

---

### Agents Endpoints

| Endpoint | OpenCode | Crow | Status |
|----------|----------|------|--------|
| `GET /agent` | List all agents | ✅ Implemented | ✅ |

---

### TUI Endpoints

All TUI endpoints are intentionally skipped as crow focuses on headless API usage:

| Endpoint | Status |
|----------|--------|
| `POST /tui/append-prompt` | ⏭️ Skip |
| `POST /tui/open-help` | ⏭️ Skip |
| `POST /tui/open-sessions` | ⏭️ Skip |
| `POST /tui/open-themes` | ⏭️ Skip |
| `POST /tui/open-models` | ⏭️ Skip |
| `POST /tui/submit-prompt` | ⏭️ Skip |
| `POST /tui/clear-prompt` | ⏭️ Skip |
| `POST /tui/execute-command` | ⏭️ Skip |
| `POST /tui/show-toast` | ⏭️ Skip |
| `GET /tui/control/next` | ⏭️ Skip |
| `POST /tui/control/response` | ⏭️ Skip |

---

### Auth Endpoints

| Endpoint | OpenCode | Crow | Status |
|----------|----------|------|--------|
| `PUT /auth/:id` | Set auth credentials | Not implemented | ❌ |

**Notes:** Crow reads from `~/.local/share/crow/auth.json` directly. API endpoint would allow runtime credential management.

---

### Events Endpoints

| Endpoint | OpenCode | Crow | Status |
|----------|----------|------|--------|
| `GET /event` | Global SSE stream | Not implemented | ❌ |

**Notes:** OpenCode uses a global event bus that broadcasts all system events. Crow currently uses per-request streaming (`/session/:id/message/stream`). For frontend development, we need this global event stream.

---

### Docs Endpoints

| Endpoint | OpenCode | Crow | Status |
|----------|----------|------|--------|
| `GET /doc` | OpenAPI spec | Not implemented | ❌ |

**Notes:** Low priority but nice for API exploration.

---

## Crow-Specific Endpoints (Not in OpenCode)

| Endpoint | Description | Purpose |
|----------|-------------|---------|
| `POST /session/dual` | Run dual-agent workflow | Experimental discriminator pattern |
| `POST /session/:id/fork` | Fork session at message | Branch conversations |
| `GET /session/:id/todo` | Get todo list | Task tracking |
| `GET /experimental/tool/ids` | List tool IDs | Development/debugging |
| `GET /experimental/tool` | List tools with schemas | Development/debugging |
| `POST /test/tool/:name` | Test tool directly | Development/debugging |

---

## Critical Gaps for Frontend Development

### Must Have (Blocking)

1. **`GET /event` - Global SSE stream**
   - Frontend needs real-time updates for all session activity
   - Current per-request streaming insufficient for multi-session UI

2. **Permission system**
   - `POST /session/:id/permissions/:permissionID`
   - Approve/reject workflow for file edits
   - Critical for safe agent operation

3. **Message revert**
   - `POST /session/:id/revert`
   - Undo agent changes
   - Essential for user control

### Should Have (Important)

4. **File search endpoints**
   - `GET /find` - Text search
   - `GET /find/file` - File name search
   - Frontend file explorer functionality

5. **Git status**
   - `GET /file/status`
   - Show modified files in UI

6. **Shell endpoint**
   - `POST /session/:id/shell`
   - Direct command execution for frontend terminal

### Nice to Have

7. **Session summarize** - AI-generated session titles
8. **App endpoints** - Version info, initialization status
9. **Auth endpoint** - Runtime credential management
10. **OpenAPI doc** - API exploration

---

## Serialization Format Compatibility

### Must Match Exactly

1. **Session object structure**
   - `id`, `parentID`, `title`, `directory`, `createdAt`, `updatedAt`
   - Crow matches this format ✅

2. **Message structure**
   - `info` (User/Assistant message metadata)
   - `parts` array (Text, Tool, Thinking parts)
   - Crow matches this format ✅

3. **Part types**
   - `Text`, `Tool`, `Thinking`
   - Tool state: `Pending`, `Running`, `Completed`, `Error`
   - Crow matches this format ✅

### Divergences (Acceptable)

1. **Streaming endpoint**
   - OpenCode: `GET /event` (global bus)
   - Crow: `POST /session/:id/message/stream` (per-request)
   - Justification: Simpler implementation, works for single-session use

2. **SSE event names**
   - OpenCode: `session.updated`, `message.created`, etc.
   - Crow: `message.start`, `text.delta`, `part`, `message.complete`
   - Justification: More granular streaming events

---

## Completeness by Category

| Category | Implemented | Total | Percentage |
|----------|------------|-------|------------|
| App | 0 | 2 | 0% |
| Config | 2 | 2 | 100% (partial) |
| Sessions | 12 | 17 | 71% |
| Files | 2 | 5 | 40% |
| Logging | 0 | 1 | 0% |
| Agents | 1 | 1 | 100% |
| TUI | 0 | 11 | 0% (intentional) |
| Auth | 0 | 1 | 0% |
| Events | 0 | 1 | 0% |
| Docs | 0 | 1 | 0% |
| **Overall** | **17** | **42** | **40%** |

Excluding intentionally skipped (TUI, Share, LSP):
| **Core API** | **17** | **26** | **65%** |
