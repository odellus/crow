# Priority Endpoints for Crow Backend

Ranked by importance for headless API usage and frontend development.

---

## Tier 1: Critical (Must Have Before Frontend)

### 1. `GET /event` - Global SSE Event Stream

**Why Critical:** Frontend needs real-time updates for session state, message progress, file changes, and permission requests. Without this, the UI cannot react to agent activity.

**OpenCode Events:**
- `server.connected` - Initial connection
- `session.updated` - Session metadata changed
- `message.created` - New message added
- `message.updated` - Message modified (streaming complete)
- `part.updated` - Tool execution progress
- `permission.requested` - Agent needs approval

**Implementation Approach:**
```rust
// Global event bus using tokio::broadcast
pub struct EventBus {
    tx: broadcast::Sender<ServerEvent>,
}

// SSE endpoint
async fn event_stream(State(state): State<AppState>) -> Sse<impl Stream<Item = Event>> {
    let mut rx = state.event_bus.subscribe();
    let stream = async_stream::stream! {
        yield Event::default().event("server.connected").data("{}");
        while let Ok(event) = rx.recv().await {
            yield event.to_sse();
        }
    };
    Sse::new(stream)
}
```

**Effort:** Medium (2-3 hours)

---

### 2. Permission System

**Endpoints:**
- `POST /session/:id/permissions/:permissionID` - Respond to permission request

**Why Critical:** Agents need user approval for file writes, bash commands, etc. Without this, we either auto-approve everything (unsafe) or block on missing feature.

**Required Changes:**
1. Add `Permission` type with request/response flow
2. Emit `permission.requested` event via event bus
3. Block tool execution until response received
4. Track pending permissions per session

**OpenCode Reference:** `opencode/packages/opencode/src/permission/`

**Effort:** High (4-6 hours)

---

### 3. `POST /session/:id/revert` - Revert Message

**Why Critical:** Users need to undo agent changes. This is the primary safety mechanism.

**Behavior:**
1. Mark message and all subsequent messages as "reverted"
2. Undo any file changes made by that message
3. Keep reverted messages in history (greyed out in UI)

**Also Needed:**
- `POST /session/:id/unrevert` - Restore reverted messages

**Effort:** Medium (3-4 hours)

---

## Tier 2: Important (Frontend Can Work Without)

### 4. File Search Endpoints

**Endpoints:**
- `GET /find?pattern=<pat>` - Search text in files
- `GET /find/file?query=<q>` - Find files by name

**Why Important:** Frontend file explorer needs search. Can use existing grep/glob tools internally.

**Implementation:**
```rust
async fn find_text(Query(params): Query<FindQuery>) -> Json<Vec<Match>> {
    // Use GrepTool internally
    let tool = GrepTool;
    let result = tool.execute(json!({ "pattern": params.pattern }), &ctx).await;
    // Parse and return matches
}
```

**Effort:** Low (1-2 hours)

---

### 5. `GET /file/status` - Git Status

**Why Important:** Frontend needs to show which files are modified, staged, untracked.

**Response Format:**
```json
[
  { "path": "src/main.rs", "status": "modified", "staged": false },
  { "path": "new-file.txt", "status": "untracked", "staged": false }
]
```

**Implementation:** Shell out to `git status --porcelain` and parse output.

**Effort:** Low (1 hour)

---

### 6. `POST /session/:id/shell` - Direct Shell Execution

**Why Important:** Frontend terminal pane. Run commands without going through agent.

**Behavior:**
- Execute command directly (no AI)
- Return stdout/stderr
- Create message with command output

**Effort:** Low (1 hour) - Already have BashTool

---

## Tier 3: Nice to Have

### 7. `POST /session/:id/summarize`

**Purpose:** Generate AI summary/title for session based on conversation.

**Effort:** Medium (uses LLM call)

---

### 8. `GET /app` and `POST /app/init`

**Purpose:** App version, initialization status.

**Effort:** Trivial (hardcoded values)

---

### 9. `PUT /auth/:id`

**Purpose:** Runtime credential management.

**Effort:** Low (write to auth.json)

---

### 10. `GET /doc` - OpenAPI Spec

**Purpose:** API documentation endpoint.

**Options:**
- Generate from code using `utoipa`
- Static JSON file

**Effort:** Medium

---

## Tier 4: Intentionally Skipped

### TUI Endpoints
Not needed for headless API. Frontend is the new TUI.

### Share/Unshare
Requires external service (OpenCode uses Anomaly's sharing service).

### LSP Endpoints (`/find/symbol`)
Too complex for initial implementation. Can add later.

---

## Implementation Priority Queue

### Week 1: Core Functionality
1. [ ] `GET /event` - Global SSE stream
2. [ ] Permission system basics
3. [ ] `POST /session/:id/revert`

### Week 2: File Operations
4. [ ] `GET /find` - Text search
5. [ ] `GET /find/file` - File name search
6. [ ] `GET /file/status` - Git status
7. [ ] `POST /session/:id/shell`

### Week 3+: Polish
8. [ ] `POST /session/:id/unrevert`
9. [ ] `POST /session/:id/summarize`
10. [ ] `GET /app`, `POST /app/init`
11. [ ] `PUT /auth/:id`
12. [ ] `GET /doc`

---

## Tools Used by OpenCode

Cross-reference with crow's tool registry to ensure we have coverage:

| Tool | OpenCode | Crow | Used By |
|------|----------|------|---------|
| bash | ✅ | ✅ | Command execution |
| read | ✅ | ✅ | File reading |
| write | ✅ | ✅ | File creation |
| edit | ✅ | ✅ | File modification |
| glob | ✅ | ✅ | File pattern matching |
| grep | ✅ | ✅ | Text search |
| list | ✅ | ✅ | Directory listing |
| task | ✅ | ✅ | Subagent spawning |
| todowrite | ✅ | ✅ | Task management |
| todoread | ✅ | ✅ | Task reading |
| webfetch | ✅ | ✅ | URL fetching |
| websearch | ✅ | ✅ | Web search |
| notebook_edit | ✅ | ❌ | Jupyter notebooks |
| mcp_* | ✅ | ❌ | MCP servers |

**Note:** Notebook and MCP tools are lower priority extensions.

---

## Serialization Notes

### Event Bus Message Format

OpenCode uses a discriminated union for events:

```typescript
type ServerEvent = 
  | { type: "session.updated", data: Session }
  | { type: "message.created", data: MessageWithParts }
  | { type: "permission.requested", data: PermissionRequest }
```

Crow should match this format for the `/event` endpoint:

```rust
#[derive(Serialize)]
#[serde(tag = "type", content = "data")]
enum ServerEvent {
    #[serde(rename = "session.updated")]
    SessionUpdated(Session),
    #[serde(rename = "message.created")]
    MessageCreated(MessageWithParts),
    // etc.
}
```

### Permission Request Format

```typescript
interface PermissionRequest {
  id: string;
  sessionID: string;
  messageID: string;
  toolName: string;
  toolArgs: object;
  risk: "low" | "medium" | "high";
  description: string;
}
```

### Permission Response Format

```typescript
interface PermissionResponse {
  response: "allow" | "deny" | "allowAll";
}
```
