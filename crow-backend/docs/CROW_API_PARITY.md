# Crow API Parity with OpenCode

This document tracks the differences between Crow and OpenCode APIs to achieve full compatibility.

## Testing Setup

Start both servers from the same directory for comparison:

```bash
# Terminal 1: OpenCode on port 4096
cd test-dummy && opencode serve -p 4096

# Terminal 2: Crow on port 4097
cd test-dummy && crow-serve -p 4097
```

Then compare responses:
```bash
# Compare session endpoints
curl -s http://localhost:4096/session | jq '.[0]'
curl -s http://localhost:4097/session | jq '.[0]'
```

---

## Endpoint Comparison

### GET /session

| Field | OpenCode | Crow | Status |
|-------|----------|------|--------|
| id format | `ses_[base62]` | `ses-[uuid]` | **DIFF** - Need base62 IDs |
| projectID | Hash of directory | `"default"` | **DIFF** - Need project hash |
| version | `"1.0.65"` (package version) | `"1.0.0"` | **DIFF** - Need version tracking |
| title | `"New session - {ISO date}"` | `"New Session"` | **DIFF** - Need date in title |
| parentID | Present when forked | Present | OK |
| summary | File change stats | Present | OK |
| time.created | Unix ms | Unix ms | OK |
| time.updated | Unix ms | Unix ms | OK |

### GET /config

| Field | OpenCode | Crow | Status |
|-------|----------|------|--------|
| agent | Agent configurations | Missing | **MISSING** |
| mode | Mode configurations | Missing | **MISSING** |
| plugin | Plugin list | Missing | **MISSING** |
| command | Custom commands | Missing | **MISSING** |
| username | System username | Missing | **MISSING** |
| keybinds | Key bindings | Missing | **MISSING** |
| providers | Provider list | Present | OK |
| version | Package version | Present | OK |

**Crow returns:** `{"agents":["default"],"providers":["moonshotai"],"version":"0.1.0"}`

**OpenCode returns:** Full config with keybinds, plugins, modes, etc.

### GET /agent

| Field | OpenCode | Crow | Status |
|-------|----------|------|--------|
| name | Agent name | Present | OK |
| description | Full description | Present | OK |
| builtIn | Boolean | Present | OK |
| id | Agent ID | Present | OK |
| mode | "subagent" etc | Present | OK |

Both return similar structure - OK

### GET /experimental/tool/ids

| OpenCode | Crow | Status |
|----------|------|--------|
| invalid | Missing | **MISSING** - Invalid tool placeholder |
| bash | Present | OK |
| edit | Present | OK |
| webfetch | Missing | **MISSING** - Web fetch tool |
| glob | Present | OK |
| grep | Present | OK |
| list | Present | OK |
| read | Present | OK |
| write | Present | OK |
| todowrite | Present | OK |
| todoread | Present | OK |
| task | Present | OK |
| websearch | Present | Extra (Crow-specific) |
| work_completed | Present | Extra (Crow-specific) |

### POST /session/{id}/message

| Field | OpenCode | Crow | Status |
|-------|----------|------|--------|
| Request format | `{"parts": [...]}` | `{"parts": [...]}` | OK |
| Streaming | SSE support | SSE support | OK |
| Response format | MessageWithParts | MessageWithParts | OK |

### POST /session/{id}/abort

| Feature | OpenCode | Crow | Status |
|---------|----------|------|--------|
| Abort running session | Yes | Yes | OK |
| Kill processes | Yes | Partial | **PARTIAL** - Process orphaning |

---

## Storage & Paths

### OpenCode Paths (XDG)
- Config: `~/.config/opencode/`
- Data: `~/.local/share/opencode/`
- Logs: `~/.local/share/opencode/log/`
- Sessions: `~/.local/share/opencode/sessions/`

### Crow Paths (XDG)
- Config: `~/.config/crow/`
- Data: `~/.local/share/crow/`
- Sessions: `~/.local/share/crow/sessions/`
- Logs: stderr (needs log directory)

| Feature | OpenCode | Crow | Status |
|---------|----------|------|--------|
| XDG compliance | Yes | Yes | OK |
| Log files | Timestamped files | stderr only | **MISSING** |
| Session persistence | SQLite-like | JSON files | **DIFF** |

---

## ACP (Agent Communication Protocol) Compatibility

For ACP to work with Crow, we need:

### Required Endpoints
- [x] `GET /session` - List sessions
- [x] `POST /session` - Create session
- [x] `GET /session/{id}` - Get session
- [x] `DELETE /session/{id}` - Delete session
- [x] `PATCH /session/{id}` - Update session
- [x] `POST /session/{id}/message` - Send message
- [x] `GET /session/{id}/message` - List messages
- [x] `POST /session/{id}/abort` - Abort session
- [x] `POST /session/{id}/fork` - Fork session
- [ ] `GET /openapi.json` - OpenAPI spec (for discovery)

### Required Response Format Parity
- [ ] Session ID format (base62 vs UUID)
- [ ] ProjectID computation (hash vs default)
- [ ] Message path tracking (cwd, root)
- [ ] Cost/token tracking format

### Tool Parity for ACP
ACP uses these tools - all must match OpenCode behavior:
- [x] bash - Execute commands
- [x] read - Read files
- [x] write - Write files
- [x] edit - Edit files
- [x] glob - Find files
- [x] grep - Search content
- [x] list - List directory
- [ ] webfetch - Fetch URLs (missing in Crow)
- [x] task - Launch subagents
- [x] todowrite - Update todos
- [x] todoread - Read todos

---

## Priority Fixes for Parity

### P0 - Critical for ACP
1. **Session ID format** - Use base62 like OpenCode
2. **ProjectID computation** - Hash the directory path
3. **webfetch tool** - Implement URL fetching
4. **OpenAPI endpoint** - Add /openapi.json for discovery

### P1 - Important for Compatibility
1. **Config endpoint** - Return full config structure
2. **Log file support** - Write logs to XDG data directory
3. **Version tracking** - Use proper version from Cargo.toml
4. **Session title format** - Include ISO date

### P2 - Nice to Have
1. **Session persistence format** - Consider SQLite
2. **Keybind configuration** - For TUI compatibility
3. **Plugin system** - For extensibility

---

## Development Workflow

### Side-by-Side Testing
```bash
# Run both servers
cd test-dummy
opencode serve -p 4096 &
crow-serve -p 4097 &

# Compare any endpoint
diff <(curl -s localhost:4096/session | jq .) <(curl -s localhost:4097/session | jq .)

# Test message sending
curl -X POST localhost:4096/session -H "Content-Type: application/json" -d '{}'
curl -X POST localhost:4097/session -H "Content-Type: application/json" -d '{}'
```

### Log Comparison
```bash
# OpenCode logs
tail -f ~/.local/share/opencode/log/*.log

# Crow logs (currently stderr)
crow-serve -p 4097 2>&1 | tee crow.log
```

### Request/Response Capture
Use a proxy to capture and compare:
```bash
# mitmproxy for detailed inspection
mitmproxy --mode reverse:http://localhost:4096 -p 8096
mitmproxy --mode reverse:http://localhost:4097 -p 8097
```

---

## Testing Checklist

### Session Management
- [ ] Create session returns same format
- [ ] List sessions returns same format
- [ ] Fork session works identically
- [ ] Delete session cleans up properly
- [ ] Session persistence across restarts

### Message Handling
- [ ] User message format matches
- [ ] Assistant message format matches
- [ ] Tool call format matches
- [ ] Token/cost tracking matches
- [ ] Streaming SSE format matches

### Tool Execution
- [ ] bash - Same output format
- [ ] read - Same content format
- [ ] write - Same response format
- [ ] edit - Same diff format
- [ ] glob - Same file list format
- [ ] grep - Same match format

### Error Handling
- [ ] Same error response format
- [ ] Same HTTP status codes
- [ ] Same error messages

---

## Implementation Notes

### Base62 ID Generation
OpenCode uses a custom base62 encoding for session IDs:
```typescript
// OpenCode's format: ses_[13 char base62]
const alphabet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
```

Crow should implement the same for compatibility.

### ProjectID Hash
OpenCode hashes the project directory:
```typescript
// SHA256 of canonical path
const projectID = sha256(path.resolve(directory));
```

### Message Path Tracking
Every assistant message should include:
```json
{
  "path": {
    "cwd": "/current/working/directory",
    "root": "/project/root"
  }
}
```

This enables audit trails and context restoration.

---

## Related Documents
- [CROW_CONFIG_PLAN.md](./CROW_CONFIG_PLAN.md) - Configuration system
- [CROW_CONTEXT_PLAN.md](./CROW_CONTEXT_PLAN.md) - Context propagation
- [CROW_PROGRESS_REPORT.md](./CROW_PROGRESS_REPORT.md) - Overall progress
