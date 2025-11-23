# Frontend Technical Specifications

Detailed API contracts and data flows for the crow frontend.

---

## API Endpoints

### Sessions

#### List Sessions
```http
GET /session
```

Response:
```json
[
  {
    "id": "ses_abc123",
    "parentID": null,
    "title": "Debug auth flow",
    "directory": "/home/user/project",
    "createdAt": 1700000000000,
    "updatedAt": 1700000100000
  }
]
```

#### Create Session
```http
POST /session
Content-Type: application/json

{
  "directory": "/home/user/project",
  "title": "New session",
  "parentID": null
}
```

Response: `Session` object

#### Get Session
```http
GET /session/:id
```

Response: `Session` object

#### Update Session
```http
PATCH /session/:id
Content-Type: application/json

{
  "title": "Updated title"
}
```

Response: `Session` object

#### Delete Session
```http
DELETE /session/:id
```

Response: `204 No Content`

#### Abort Session
```http
POST /session/:id/abort
```

Response:
```json
{
  "aborted": true,
  "session_id": "ses_abc123"
}
```

#### Fork Session
```http
POST /session/:id/fork
Content-Type: application/json

{
  "messageID": "msg_xyz789"
}
```

Response: New `Session` object

---

### Messages

#### List Messages
```http
GET /session/:id/message
```

Response:
```json
[
  {
    "info": {
      "type": "user",
      "id": "msg_001",
      "sessionID": "ses_abc",
      "time": { "created": 1700000000, "completed": null },
      "summary": null,
      "metadata": null
    },
    "parts": [
      {
        "type": "text",
        "id": "prt_001",
        "sessionID": "ses_abc",
        "messageID": "msg_001",
        "text": "Hello"
      }
    ]
  },
  {
    "info": {
      "type": "assistant",
      "id": "msg_002",
      "sessionID": "ses_abc",
      "parentID": "msg_001",
      "modelID": "kimi-k2-thinking",
      "providerID": "moonshotai",
      "mode": "build",
      "time": { "created": 1700000001, "completed": 1700000010 },
      "path": { "cwd": "/project", "root": "/project" },
      "cost": 0.001234,
      "tokens": { "input": 500, "output": 200, "reasoning": 0, "cache": { "read": 0, "write": 0 } },
      "error": null,
      "summary": null,
      "metadata": null
    },
    "parts": [
      {
        "type": "text",
        "id": "prt_002",
        "sessionID": "ses_abc",
        "messageID": "msg_002",
        "text": "Hello! How can I help?"
      }
    ]
  }
]
```

#### Send Message (Non-streaming)
```http
POST /session/:id/message
Content-Type: application/json

{
  "agent": "build",
  "parts": [
    { "type": "text", "text": "What files are in this project?" }
  ],
  "noReply": false
}
```

Response: `MessageWithParts` (assistant response)

#### Send Message (Streaming)
```http
POST /session/:id/message/stream
Content-Type: application/json

{
  "agent": "build",
  "parts": [
    { "type": "text", "text": "What files are in this project?" }
  ]
}
```

Response: Server-Sent Events stream

---

### SSE Events

#### Connection Endpoint
```http
GET /event
Accept: text/event-stream
```

#### Event Types

**server.connected**
```
event: server.connected
data: {"status": "connected"}
```

**session.updated**
```
event: session.updated
data: {"id": "ses_abc", "title": "Updated", ...}
```

**message.created**
```
event: message.created
data: {"info": {...}, "parts": [...]}
```

**message.updated**
```
event: message.updated
data: {"info": {...}, "parts": [...]}
```

**text.delta** (streaming only)
```
event: text.delta
data: {"id": "prt_001", "delta": "Hello"}
```

**part.updated**
```
event: part.updated
data: {"type": "tool", "id": "prt_003", "state": "completed", ...}
```

**permission.requested**
```
event: permission.requested
data: {
  "id": "perm_001",
  "sessionID": "ses_abc",
  "messageID": "msg_002",
  "toolName": "write",
  "toolArgs": {"file_path": "/src/main.rs", "content": "..."},
  "risk": "medium",
  "description": "Write to /src/main.rs"
}
```

**message.start** (streaming only)
```
event: message.start
data: {"status": "starting"}
```

**message.complete** (streaming only)
```
event: message.complete
data: {"info": {...}, "parts": [...]}
```

**error**
```
event: error
data: {"error": "Session not found"}
```

---

### Files

#### List Files
```http
GET /file?path=/project/src
```

Response:
```json
{
  "path": "/project/src",
  "entries": [
    { "name": "main.rs", "path": "/project/src/main.rs", "is_dir": false, "size": 1234 },
    { "name": "lib", "path": "/project/src/lib", "is_dir": true, "size": null }
  ],
  "count": 2
}
```

#### Read File
```http
GET /file/content?path=/project/src/main.rs
```

Response: Raw file content (text/plain)

#### Search Files (to be implemented)
```http
GET /find?pattern=TODO
```

Response:
```json
[
  {
    "path": "/project/src/main.rs",
    "line_number": 42,
    "lines": "// TODO: Fix this",
    "submatches": [{ "start": 3, "end": 7 }]
  }
]
```

#### Find Files (to be implemented)
```http
GET /find/file?query=main
```

Response:
```json
["/project/src/main.rs", "/project/examples/main.rs"]
```

---

### Permissions (to be implemented)

#### Respond to Permission
```http
POST /session/:id/permissions/:permissionID
Content-Type: application/json

{
  "response": "allow"
}
```

Possible responses: `"allow"`, `"deny"`, `"allowAll"`

Response: `200 OK`

---

### Config

#### Get Config
```http
GET /config
```

Response:
```json
{
  "version": "0.1.0",
  "providers": ["moonshotai"],
  "agents": ["build", "plan"]
}
```

#### List Providers
```http
GET /config/providers
```

Response:
```json
[
  {
    "id": "moonshotai",
    "name": "Moonshot AI",
    "models": ["kimi-k2-thinking", "kimi-k2-thinking-turbo"]
  }
]
```

---

### Agents

#### List Agents
```http
GET /agent
```

Response:
```json
[
  {
    "id": "build",
    "name": "build",
    "description": "General-purpose coding agent",
    "mode": "primary",
    "builtIn": true
  },
  {
    "id": "plan",
    "name": "plan",
    "description": "Read-only planning agent",
    "mode": "primary",
    "builtIn": true
  }
]
```

---

## TypeScript Types

```typescript
// src/types/index.ts

export interface Session {
  id: string;
  parentID: string | null;
  title: string;
  directory: string;
  createdAt: number;
  updatedAt: number;
}

export interface MessageWithParts {
  info: Message;
  parts: Part[];
}

export type Message = UserMessage | AssistantMessage;

export interface UserMessage {
  type: 'user';
  id: string;
  sessionID: string;
  time: MessageTime;
  summary: string | null;
  metadata: Record<string, unknown> | null;
}

export interface AssistantMessage {
  type: 'assistant';
  id: string;
  sessionID: string;
  parentID: string;
  modelID: string;
  providerID: string;
  mode: string;
  time: MessageTime;
  path: MessagePath;
  cost: number;
  tokens: TokenUsage;
  error: string | null;
  summary: string | null;
  metadata: Record<string, unknown> | null;
}

export interface MessageTime {
  created: number;
  completed: number | null;
}

export interface MessagePath {
  cwd: string;
  root: string;
}

export interface TokenUsage {
  input: number;
  output: number;
  reasoning: number;
  cache: { read: number; write: number };
}

export type Part = TextPart | ToolPart | ThinkingPart;

export interface TextPart {
  type: 'text';
  id: string;
  sessionID: string;
  messageID: string;
  text: string;
}

export interface ThinkingPart {
  type: 'thinking';
  id: string;
  sessionID: string;
  messageID: string;
  text: string;
}

export interface ToolPart {
  type: 'tool';
  id: string;
  sessionID: string;
  messageID: string;
  callID: string;
  tool: string;
  state: ToolState;
}

export type ToolState = 
  | { status: 'pending'; input: unknown; raw: string }
  | { status: 'running'; input: unknown; raw: string }
  | { 
      status: 'completed'; 
      input: unknown; 
      output: string; 
      title: string; 
      time: { start: number; end: number | null };
    }
  | { status: 'error'; input: unknown; error: string };

export interface PermissionRequest {
  id: string;
  sessionID: string;
  messageID: string;
  toolName: string;
  toolArgs: Record<string, unknown>;
  risk: 'low' | 'medium' | 'high';
  description: string;
}

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number | null;
}

export interface SearchMatch {
  path: string;
  line_number: number;
  lines: string;
  submatches: { start: number; end: number }[];
}

export interface Agent {
  id: string;
  name: string;
  description: string;
  mode: 'primary' | 'subagent' | 'all';
  builtIn: boolean;
}

export interface Provider {
  id: string;
  name: string;
  models: string[];
}
```

---

## File Operation Flows

### View File
1. User clicks file in tree
2. Frontend calls `GET /file/content?path=...`
3. Detect language from extension
4. Open new tab with CodeMirror
5. Load content into editor

### Agent Modifies File
1. Agent calls write/edit tool
2. Backend emits `part.updated` with tool state
3. Frontend detects file path in tool args
4. Fetch original file content
5. Show diff view in editor
6. (If permission system) Wait for approval

### Accept Change
1. User clicks "Accept" in diff view
2. Frontend closes diff, shows new content
3. Update file tree if needed (new file)

### Reject Change
1. User clicks "Reject"
2. Frontend calls `POST /session/:id/revert` with messageID
3. Remove pending changes
4. Show original file content

### Save File
1. User edits file in CodeMirror
2. Mark tab as dirty
3. User presses Ctrl+S
4. Frontend calls agent: "Save this content to [path]"
5. Agent uses write tool
6. Mark tab as saved

---

## Error Handling

### HTTP Errors

| Status | Meaning | UI Action |
|--------|---------|-----------|
| 400 | Bad request | Show inline error |
| 401 | Unauthorized | Show auth dialog |
| 404 | Not found | Show "not found" message |
| 409 | Conflict (locked) | Show "session busy" |
| 500 | Server error | Show error toast, retry button |

### SSE Errors

| Event | Action |
|-------|--------|
| Connection lost | Show "reconnecting" banner |
| Reconnect failed | Show "offline" with retry button |
| Parse error | Log and ignore |

### Application Errors

| Error | UI Action |
|-------|-----------|
| File not found | Show placeholder in editor |
| Permission denied | Show in permission dialog |
| Tool failed | Show error in tool part, expandable |
| Abort | Show "aborted" status on message |

---

## Performance Requirements

### Response Times
- Initial load: < 2s
- Session switch: < 500ms
- Message send: < 100ms to show "sending"
- File open: < 200ms for < 1MB file
- Search results: < 1s

### Bundle Size
- Initial JS: < 500KB gzipped
- CSS: < 50KB gzipped
- Code splitting for:
  - Editor languages
  - Settings modal
  - Diff viewer

### Memory
- Support 1000+ messages per session
- Virtualized lists for large files
- Dispose editors when tabs close

---

## Security Considerations

### XSS Prevention
- Sanitize markdown rendering
- Never use `dangerouslySetInnerHTML` with user content
- CSP headers (handled by Vite)

### CORS
- Backend must allow frontend origin
- Currently using permissive CORS

### File Access
- Files read through backend only
- No direct filesystem access
- Path validation on backend

### Secrets
- API keys never sent to frontend
- Auth handled by backend

---

## Browser Compatibility

### Target Browsers
- Chrome 90+
- Firefox 90+
- Safari 15+
- Edge 90+

### Required Features
- ES2020+
- CSS Grid/Flexbox
- EventSource (SSE)
- ResizeObserver
- IntersectionObserver

### No Support
- IE11
- Opera Mini
- UC Browser
