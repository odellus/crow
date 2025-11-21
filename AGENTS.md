# Agent Architecture for Crow

**Last Updated:** 2025-11-17  
**Status:** ~80% Core Functionality Complete

## Current Status

### What Crow Has Now ✅

**Core Infrastructure:**
- ✅ **LLM Integration** - Moonshot kimi-k2-thinking (262k context)
- ✅ **Auth System** - `~/.local/share/crow/auth.json` (matching OpenCode)
- ✅ **XDG Storage** - Full directory structure matching OpenCode
- ✅ **REST API** - Complete endpoints (`http://localhost:7070`)

**Agents:**
- ✅ **6 built-in agents:** general, build, plan, supervisor, architect, discriminator
- ✅ **Agent registry** with dynamic tool permissions
- ✅ **Agent executor** with system prompt building
- ✅ **Subagent spawning** via Task tool

**Tools (12 total):**
- ✅ **File ops:** bash, edit, write, read, grep, glob, list
- ✅ **Planning:** todowrite, todoread (session-specific storage)
- ✅ **Subagents:** task (spawns child sessions with dynamic agent list)
- ✅ **Web:** websearch (SearXNG integration)
- ✅ **Dual-agent:** work_completed (discriminator tool)

**Session Management:**
- ✅ **Session store** with XDG persistence
- ✅ **Parent/child linking** (parentID)
- ✅ **Message persistence** per session
- ✅ **Todo persistence** per session (fixed: uses ctx.session_id)
- ✅ **Session export** to markdown

**API Endpoints:**
- ✅ `POST /session` - create session
- ✅ `GET /session` - list sessions  
- ✅ `GET /session/:id` - get session
- ✅ `DELETE /session/:id` - delete session
- ✅ `POST /session/:id/message` - send message to agent
- ✅ `GET /session/:id/message` - list messages
- ✅ `GET /session/:id/children` - list child sessions
- ✅ `GET /experimental/tool/ids` - list tool IDs

### What's Missing ❌

**High Priority:**
- ❌ **Dioxus Web UI** - Frontend to visualize sessions/tools/agents
- ❌ **Streaming (SSE)** - Real-time message streaming
- ❌ **Background Bash** - BashOutput, KillShell tools
- ❌ **Plan/Explore agents** - Need OpenCode system prompts

**Medium Priority:**
- ❌ **Model switching** - Currently hardcoded, need session-level config
- ❌ **Markdown telemetry** - Export format exists but needs refinement
- ❌ **System prompt verification** - Compare against OpenCode exactly

**Low Priority:**
- ❌ **Dual-agent endpoint** - `POST /session/dual` (may not be needed)
- ❌ **MCP support** - Model Context Protocol
- ❌ **LSP support** - Language Server Protocol

## The Architecture We Have

### 1. Task Tool ✅ IMPLEMENTED

**Purpose:** Allows agents to spawn subagents autonomously

**Location:** `crow/packages/api/src/tools/task.rs`

**How it works:**
1. Parent agent calls task tool with subagent_type
2. Task tool creates child session with `parentID` link
3. Spawns subagent of specified type
4. Returns result to parent agent
5. Parent synthesizes for user

**Dynamic Agent List:**
```rust
// Tool description is built dynamically from agent registry
let agents = agent_registry.get_subagents().await;
let description = DESCRIPTION.replace("{agents}", &agent_list);

// Agents available:
// - general: General-purpose agent
// - build: Build agent for implementation
// - plan: Planning agent
// (etc - pulled from registry at runtime)
```

**Example Flow:**
```
User: "Implement fibonacci function"
  ↓
BUILD agent: "I'll spawn a subagent"
  🔧 task(
    description="Implement fibonacci", 
    prompt="Write fibonacci with tests",
    subagent_type="general"
  )
  ↓
Task tool:
  - Creates child session (ses-xxx)
  - Sets parentID to current session
  - Executes general agent
  - Returns output
  ↓
BUILD agent: "Fibonacci implemented"
```

### 2. Agent Modes ✅ EXISTS

```rust
pub enum AgentMode {
    Primary,   // Can be used directly by user
    Subagent,  // Only via task tool
    All,       // Both
}
```

**Current agents:**
- `general` - Primary (default)
- `build` - Primary  
- `plan` - Subagent
- `supervisor` - Primary
- `architect` - Primary
- `discriminator` - Subagent (dual-agent only)

### 3. Session Hierarchy ✅ WORKS

```rust
pub struct Session {
    pub id: String,
    pub parent_id: Option<String>,  // ✅ Working
    pub directory: String,
    pub title: Option<String>,
    pub version: String,
    pub time: SessionTime,
    pub metadata: Option<Value>,
}
```

**Child sessions:**
- Created by Task tool
- Linked via `parent_id`
- Listed via `GET /session/:id/children`
- Persistent to XDG storage

### 4. Tool Execution ✅ WORKS

**All tools use ToolContext:**
```rust
pub struct ToolContext {
    pub session_id: String,  // From execution context
    pub message_id: String,
    pub agent: String,
    pub working_dir: PathBuf,
}
```

**Recent Fix:** TodoWrite now uses `ctx.session_id` instead of asking LLM for it.

### 5. Storage Structure ✅ MATCHES OPENCODE

```
~/.local/share/crow/
├── auth.json              # API keys
├── log/                   # Server logs
└── storage/
    ├── message/           # Per-session messages
    │   └── {session-id}/
    │       ├── {msg-id}.json
    │       └── ...
    ├── session/           # Session metadata
    │   └── {session-id}/
    │       └── session.json
    └── todo/              # Per-session todos
        ├── {session-id}.json
        └── ...
```

## What We're Building Next: Dioxus Web UI

**Why Web First:**
- OpenCode's TUI is terminal-only
- We want cross-platform from day one
- Dioxus compiles to web/desktop/mobile from same code
- Web UI = easier testing, better UX, multi-user

**Project Structure:**
```
crow/packages/
├── api/     ✅ Done - REST backend
├── ui/      🎯 Next - shared components  
└── web/     🎯 Next - web frontend
```

**What the UI needs to show:**
1. **Session List** - All sessions with titles
2. **Message View** - Messages with tool execution
3. **Todo Panel** - Real-time todo updates
4. **Session Tree** - Parent/child relationships
5. **Tool Execution** - Visual display of tool calls
6. **Agent Status** - Which agent is working

**Architecture:**
```
Browser → Dioxus Web (port 8080)
            ↓ HTTP
         Crow API (port 7070)
            ↓
         Agents + Tools
```

## Implementation Status by Component

### Backend (API) - 95% Complete ✅

| Component | Status | Notes |
|-----------|--------|-------|
| Agent Registry | ✅ Done | Dynamic tool permissions |
| Agent Executor | ✅ Done | System prompts, tool execution |
| Session Store | ✅ Done | XDG persistence |
| Message Store | ✅ Done | Per-session storage |
| Tool Registry | ✅ Done | 12 tools registered |
| Task Tool | ✅ Done | Dynamic agent spawning |
| TodoWrite | ✅ Fixed | Uses session context |
| WebSearch | ✅ Done | SearXNG integration |
| REST API | ✅ Done | All endpoints working |
| Streaming | ❌ TODO | SSE endpoint stub exists |

### Frontend (Web) - 0% Complete 🎯

| Component | Status | Notes |
|-----------|--------|-------|
| Session List | ❌ TODO | List all sessions |
| Message View | ❌ TODO | Show conversation |
| Tool Renderer | ❌ TODO | Display tool calls |
| Todo Panel | ❌ TODO | Live todo updates |
| Session Tree | ❌ TODO | Parent/child view |
| Agent Selector | ❌ TODO | Choose agent |
| Model Selector | ❌ TODO | Switch models |
| Streaming | ❌ TODO | Real-time messages |

### Tools - 80% Complete

| Tool | Status | Notes |
|------|--------|-------|
| bash | ✅ Done | Command execution |
| read | ✅ Done | File reading |
| write | ✅ Done | File writing |
| edit | ✅ Done | File editing |
| grep | ✅ Done | Content search |
| glob | ✅ Done | File pattern matching |
| list | ✅ Done | Directory listing |
| todowrite | ✅ Fixed | Session-aware |
| todoread | ✅ Done | Read session todos |
| task | ✅ Done | Spawn subagents |
| websearch | ✅ Done | Internet search |
| work_completed | ✅ Done | Dual-agent tool |
| BashOutput | ❌ TODO | Background bash |
| KillShell | ❌ TODO | Kill background |

## Running Crow

### Prerequisites

1. **SearXNG** running on `localhost:8082` (for websearch tool)
2. **Auth configured** at `~/.local/share/crow/auth.json`:
   ```json
   {"moonshotai":{"type":"api","key":"your-api-key-here"}}
   ```

### Build & Run

```bash
# Build the server
cd crow/packages/api
cargo build --release --features server --bin crow-serve

# Run the server
cd crow
./target/release/crow-serve
# Server starts on http://127.0.0.1:7070
```

### Testing the API

```bash
# Create a session
SESSION=$(curl -s -X POST http://127.0.0.1:7070/session \
  -H "Content-Type: application/json" \
  -d '{"directory": "/tmp"}' | jq -r '.id')

# Send a message
curl -s -X POST "http://127.0.0.1:7070/session/$SESSION/message" \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "build",
    "parts": [{"type": "text", "text": "Use websearch to find Rust 2024 features"}]
  }' | jq '.parts[] | select(.type == "tool") | .tool'

# List all tools
curl -s http://127.0.0.1:7070/experimental/tool/ids | jq '.'
```

### Troubleshooting

**Lock poisoning error:** `Lock error: poisoned lock: another task failed inside`

This happens when the server panics during a request. The RwLock becomes poisoned and all subsequent requests fail.

**Solution:** Restart the server
```bash
pkill -9 crow-serve
./target/release/crow-serve
```

**Empty responses or timeouts:**
- Check that Moonshot API key is valid in `~/.local/share/crow/auth.json`
- Check that SearXNG is running: `curl http://localhost:8082/search?q=test&format=json`
- Check server logs: `~/.local/share/crow/log/`

### Storage Locations

- **Sessions & Messages:** `~/.local/share/crow/storage/`
- **Auth:** `~/.local/share/crow/auth.json`
- **Logs:** `~/.local/share/crow/log/`

---

## Testing Results (2025-11-19)

**Successful Tests:**
```bash
✅ LLM call with kimi-k2-thinking
✅ Tool execution (all 12 tools)
✅ Subagent spawning via Task tool
✅ Child session creation with parentID
✅ Todo persistence to session-specific files
✅ WebSearch with SearXNG (verified end-to-end)
✅ Session management (CRUD)
✅ Message persistence
✅ Auth.json reading
```

**Example Session:**
```
Session: ses-f1e2753a-d0e4-4c1d-90ca-b5f38eab25b0
├── Created 7-step plan with TodoWrite
├── Spawned child session: ses-369a9dea-e0bd-4d18-af21-1a19f83efd75
├── Used kimi-k2-thinking (262k context)
├── Saved todos to ses-f1e2753a....json ✅
└── All tools executed successfully
```

## Next Session: Build the Web UI 🚀

**Goal:** Translate OpenCode's TUI to Dioxus web components

**Approach:**
1. Read `opencode/packages/opencode/src/cli/cmd/tui/`
2. Port React/Ink patterns to Dioxus RSX
3. Connect to Crow's REST API
4. Start with: Session List → Message View → Tool Display

**Why this is the right next step:**
- Backend is solid and tested
- We need visibility into what agents are doing
- Web UI = better debugging
- Foundation for desktop/mobile later

**Reference Implementation:**
- OpenCode TUI: `opencode/packages/opencode/src/cli/cmd/tui/`
- Our API: `crow/packages/api/src/server.rs`
- Our Components: `crow/packages/web/src/` (to be built)

The backend is ready. Time to make it visible! 🦅
