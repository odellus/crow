# Next Steps: Building the Dioxus Web UI for Crow

**Created:** 2025-11-17  
**Status:** Backend Complete (~95%) - Ready for Frontend  
**Goal:** Build a beautiful web interface to visualize agent execution in real-time

---

## Why We're Ready

### Backend is Solid ✅

The Crow backend is feature-complete and battle-tested:

- ✅ **12 working tools** including WebSearch, Task, TodoWrite/Read
- ✅ **REST API** on port 7070 with all necessary endpoints
- ✅ **Agent execution** with kimi-k2-thinking (262k context)
- ✅ **Session management** with parent/child linking
- ✅ **XDG storage** matching OpenCode's structure
- ✅ **Tool execution loop** fully functional
- ✅ **Subagent spawning** via Task tool with dynamic agent registry

**We can see it working via curl, but we're flying blind.** Time to build eyes.

---

## The Vision: OpenCode's UX in a Modern Web App

### What OpenCode's TUI Does Well

OpenCode has a terminal UI (`opencode tui`) that shows:

1. **Session list** - All conversations
2. **Message stream** - User + assistant messages
3. **Tool execution display** - Real-time tool calls with state
4. **Todo panel** - Planning steps updating live
5. **Keyboard navigation** - Vim-like shortcuts
6. **Theme support** - Color schemes
7. **Session switching** - Navigate between conversations

**Location:** `opencode/packages/opencode/src/cli/cmd/tui/`

### What We'll Build Better

**Crow Web UI advantages:**

- 🌐 **Browser-based** - Open in any browser, no terminal needed
- 🎨 **Better styling** - CSS instead of terminal colors
- 🖱️ **Mouse support** - Click, drag, resize panels
- 👥 **Multi-user** - Multiple browsers can connect
- 📱 **Responsive** - Works on desktop, tablet, mobile
- 🔄 **Hot reload** - Dioxus dev experience
- 🚀 **Cross-platform** - Same code → Web, Desktop, Mobile

**And we keep the good parts:**

- ✅ Real-time updates (via polling or SSE)
- ✅ Keyboard shortcuts (progressively enhanced)
- ✅ Same information density
- ✅ Fast, native feel

---

## Project Structure

### Current Dioxus Setup

```
crow/
├── packages/
│   ├── api/          ✅ Backend (done)
│   │   └── src/
│   │       ├── server.rs      # REST API endpoints
│   │       ├── agent/         # Agent execution
│   │       ├── session/       # Session management
│   │       ├── tools/         # Tool implementations
│   │       └── ...
│   │
│   ├── ui/           🎯 Shared components (to build)
│   │   └── src/
│   │       ├── components/    # Reusable UI components
│   │       │   ├── message.rs
│   │       │   ├── tool_call.rs
│   │       │   ├── session_list.rs
│   │       │   └── todo_panel.rs
│   │       └── lib.rs
│   │
│   └── web/          🎯 Web frontend (to build)
│       └── src/
│           ├── main.rs        # Web app entry
│           ├── app.rs         # Root component
│           ├── routes/        # Page routes
│           │   ├── home.rs    # Session list
│           │   └── session.rs # Session detail view
│           └── api/           # API client
│               └── client.rs  # HTTP requests to :7070
```

### Why This Structure?

**`packages/ui/`** - Shared components
- Used by web, desktop, mobile
- Pure Dioxus RSX, no platform-specific code
- Reusable across all frontends

**`packages/web/`** - Web-specific code
- Axum server for serving web app
- Web-specific API calls (fetch)
- Browser entry point

**`packages/api/`** - Backend (already done)
- REST API server
- Agent execution
- Tool registry
- Session storage

---

## Architecture: How It All Connects

```
┌─────────────────────────────────────────────────────────────┐
│                         Browser                              │
│  ┌────────────────────────────────────────────────────┐     │
│  │           Dioxus Web App (port 8080)                │     │
│  │                                                      │     │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │     │
│  │  │ Session List │  │ Message View │  │ Todo Panel│ │     │
│  │  └──────────────┘  └──────────────┘  └──────────┘ │     │
│  │                                                      │     │
│  │  Components from packages/ui/                       │     │
│  └────────────────────────────────────────────────────┘     │
│                          │                                    │
│                          │ HTTP Requests                     │
│                          ▼                                    │
└─────────────────────────────────────────────────────────────┘
                           │
                           │
┌──────────────────────────┼──────────────────────────────────┐
│                          ▼                                    │
│              Crow API Server (port 7070)                     │
│  ┌────────────────────────────────────────────────────┐     │
│  │                 REST Endpoints                      │     │
│  │  GET  /session              - List sessions        │     │
│  │  POST /session              - Create session       │     │
│  │  GET  /session/:id          - Get session          │     │
│  │  POST /session/:id/message  - Send message         │     │
│  │  GET  /session/:id/message  - List messages        │     │
│  │  GET  /session/:id/children - List child sessions  │     │
│  └────────────────────────────────────────────────────┘     │
│                          │                                    │
│                          ▼                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │              Agent Executor                         │     │
│  │   ┌──────────┐    ┌──────────┐    ┌──────────┐   │     │
│  │   │  Build   │    │ General  │    │   Plan   │   │     │
│  │   └──────────┘    └──────────┘    └──────────┘   │     │
│  └────────────────────────────────────────────────────┘     │
│                          │                                    │
│                          ▼                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │              Tool Registry (12 tools)               │     │
│  │   bash, read, write, edit, grep, glob, list,       │     │
│  │   todowrite, todoread, task, websearch,            │     │
│  │   work_completed                                    │     │
│  └────────────────────────────────────────────────────┘     │
│                          │                                    │
│                          ▼                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │          XDG Storage (~/.local/share/crow)         │     │
│  │   sessions/, messages/, todos/                     │     │
│  └────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

**Key Points:**

1. **Two Servers:**
   - Crow API (7070) - Backend we built
   - Dioxus Web (8080) - Frontend we're building

2. **Separation of Concerns:**
   - API handles all logic
   - Web just renders and makes HTTP requests
   - No business logic in frontend

3. **Real-time Updates:**
   - Option 1: Polling (simple, works everywhere)
   - Option 2: SSE (better, need to implement)
   - Option 3: WebSockets (overkill for now)

---

## Implementation Plan: Phase by Phase

### Phase 1: Basic Setup (First Session) 🎯

**Goal:** Get something rendering in the browser

**Tasks:**
1. ✅ Verify `packages/web/` Cargo.toml is set up
2. ✅ Create basic `main.rs` and `app.rs`
3. ✅ Add simple "Hello Crow" page
4. ✅ Run `dx serve` and see it in browser
5. ✅ Add basic routing (home, session detail)

**Success Criteria:**
- Browser shows something at `http://localhost:8080`
- Can navigate between routes
- No crashes, builds cleanly

**Time Estimate:** 1-2 hours

---

### Phase 2: API Client (Second Session)

**Goal:** Connect to Crow backend

**Tasks:**
1. Create `api/client.rs` with HTTP client
2. Implement session fetching:
   ```rust
   async fn list_sessions() -> Result<Vec<Session>, Error>
   async fn get_session(id: &str) -> Result<Session, Error>
   async fn get_messages(id: &str) -> Result<Vec<Message>, Error>
   ```
3. Test API calls work
4. Add error handling

**Success Criteria:**
- Can fetch sessions from API
- Can fetch messages from API
- Errors display nicely

**Time Estimate:** 1-2 hours

---

### Phase 3: Session List View (Third Session)

**Goal:** Show all sessions

**Tasks:**
1. Create `SessionList` component in `packages/ui/`
2. Fetch sessions on load
3. Display session cards with:
   - Title
   - Created date
   - Message count
   - Agent type
4. Click to open session
5. Style it nicely

**Success Criteria:**
- Sessions render in a list
- Clicking opens session detail
- Looks decent (basic CSS)

**Time Estimate:** 2-3 hours

**Reference:**
- OpenCode: `cli/cmd/tui/routes/session/list.tsx`

---

### Phase 4: Message View (Fourth Session)

**Goal:** Show conversation

**Tasks:**
1. Create `MessageView` component
2. Fetch messages for session
3. Render user messages
4. Render assistant messages
5. Show tool calls (basic version)
6. Auto-scroll to bottom
7. Add send message input

**Success Criteria:**
- Messages display in order
- User vs assistant clearly distinguished
- Tool calls visible (even if basic)
- Can send new messages

**Time Estimate:** 3-4 hours

**Reference:**
- OpenCode: `cli/cmd/tui/routes/session/index.tsx`

---

### Phase 5: Tool Execution Display (Fifth Session)

**Goal:** Visualize tool calls beautifully

**Tasks:**
1. Create `ToolCall` component
2. Handle different tool states:
   - Pending
   - Completed
   - Error
3. Show tool input/output
4. Collapsible details
5. Syntax highlighting for code
6. Special rendering for:
   - bash (command + output)
   - read/write (file operations)
   - todowrite (todo list)
   - websearch (search results)

**Success Criteria:**
- Each tool type renders nicely
- State changes are visible
- Output is readable
- Errors are clear

**Time Estimate:** 4-5 hours

**Reference:**
- OpenCode: `session/tool-renderer.ts`
- OpenCode: `cli/cmd/tui/component/prompt/tool.tsx`

---

### Phase 6: Todo Panel (Sixth Session)

**Goal:** Show agent's planning

**Tasks:**
1. Create `TodoPanel` component
2. Fetch todos for session
3. Display todo list with status:
   - ✓ Completed
   - → In Progress
   - ○ Pending
4. Update in real-time (polling)
5. Collapsible panel

**Success Criteria:**
- Todos visible while agent works
- Status updates show progress
- Clean, scannable design

**Time Estimate:** 2-3 hours

---

### Phase 7: Real-time Updates (Seventh Session)

**Goal:** Updates without refresh

**Tasks:**
1. Add polling for active sessions
2. Poll messages every 1 second
3. Poll todos every 1 second
4. Smart polling (only when session is active)
5. OR: Implement SSE streaming
6. Update UI smoothly (no flicker)

**Success Criteria:**
- New messages appear automatically
- Todos update as agent works
- No page refresh needed
- Doesn't feel janky

**Time Estimate:** 2-3 hours

---

### Phase 8: Session Tree View (Eighth Session)

**Goal:** Show parent/child relationships

**Tasks:**
1. Create `SessionTree` component
2. Fetch child sessions
3. Render tree structure:
   ```
   Session: Build feature X
   ├── Child: General agent - implement Y
   └── Child: Plan - design Z
   ```
4. Click to navigate tree
5. Visual indicators for active session

**Success Criteria:**
- Parent/child relationships visible
- Can navigate tree
- Clear visual hierarchy

**Time Estimate:** 2-3 hours

---

### Phase 9: Polish & UX (Ninth Session)

**Goal:** Make it production-ready

**Tasks:**
1. Add loading states
2. Add empty states
3. Error boundaries
4. Keyboard shortcuts
5. Responsive design
6. Dark mode
7. Settings panel
8. Model selector
9. Agent selector

**Success Criteria:**
- Feels polished
- No rough edges
- Works on mobile
- Fast and responsive

**Time Estimate:** 4-5 hours

---

## Technical Decisions

### State Management

**Use Dioxus Signals:**
```rust
// Global state
static SESSIONS: GlobalSignal<Vec<Session>> = Signal::global(|| vec![]);
static CURRENT_SESSION: GlobalSignal<Option<String>> = Signal::global(|| None);

// Component state
let messages = use_signal(|| vec![]);
let loading = use_signal(|| false);
```

**Why:**
- Built into Dioxus
- Reactive updates
- No external dependencies
- Easy to reason about

### Styling

**Use Tailwind CSS:**
```rust
rsx! {
    div { class: "flex flex-col h-screen bg-gray-900",
        div { class: "flex-1 overflow-y-auto p-4",
            // Messages
        }
    }
}
```

**Why:**
- Fast development
- Consistent design
- Mobile-friendly
- Well-documented

**Alternative:** Vanilla CSS if you prefer full control

### API Calls

**Use reqwest:**
```rust
async fn list_sessions() -> Result<Vec<Session>, Error> {
    let response = reqwest::get("http://localhost:7070/session")
        .await?
        .json::<Vec<Session>>()
        .await?;
    Ok(response)
}
```

**Why:**
- Standard Rust HTTP client
- Async/await support
- JSON deserialization
- Works in WASM

### Real-time Updates

**Start with Polling:**
```rust
use_future(|| async move {
    loop {
        if let Ok(msgs) = fetch_messages(&session_id).await {
            messages.set(msgs);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
});
```

**Later: SSE Streaming:**
- API already has stub for this
- Better UX (instant updates)
- Lower latency
- Implement in Phase 7

---

## Reference Materials

### OpenCode TUI Source

**Must read before starting:**

1. **Component Structure:**
   - `cli/cmd/tui/routes/session/index.tsx` - Main session view
   - `cli/cmd/tui/component/prompt/tool.tsx` - Tool rendering
   - `cli/cmd/tui/component/prompt/index.tsx` - Message rendering

2. **State Management:**
   - `cli/cmd/tui/context/session.tsx` - Session context
   - `cli/cmd/tui/context/theme.tsx` - Theme system

3. **API Integration:**
   - `session/prompt.ts` - How they call their backend
   - `server/tui.ts` - Their SSE implementation

### Dioxus Documentation

**Essential reads:**

1. [Dioxus Book](https://dioxuslabs.com/learn/0.5/) - Core concepts
2. [Fullstack Guide](https://dioxuslabs.com/learn/0.5/guide/fullstack) - Web setup
3. [Signals](https://dioxuslabs.com/learn/0.5/reference/signals) - State management
4. [Router](https://dioxuslabs.com/learn/0.5/router) - Routing

### Crow Backend API

**Endpoints we'll use:**

```
GET  /session              → Vec<Session>
POST /session              → Session
GET  /session/:id          → Session
POST /session/:id/message  → Message
GET  /session/:id/message  → Vec<Message>
GET  /session/:id/children → Vec<Session>
```

**Types (already defined):**
- `Session` - in `crow/packages/api/src/types.rs`
- `Message` - same file
- `Part` - message parts (text, tool calls)

---

## Success Metrics

### We'll know it's working when:

1. ✅ Open browser → see sessions
2. ✅ Click session → see messages
3. ✅ Send message → agent responds
4. ✅ Watch tools execute in real-time
5. ✅ See todos update as agent plans
6. ✅ Navigate session tree
7. ✅ No page refreshes needed
8. ✅ Feels as good as OpenCode TUI (or better!)

### Performance Targets:

- **Initial load:** < 500ms
- **Navigation:** < 100ms
- **Message send:** < 200ms (before LLM)
- **Real-time updates:** < 1s latency
- **Bundle size:** < 500KB (gzipped)

---

## Common Pitfalls to Avoid

### 1. Don't Duplicate Business Logic

**❌ Wrong:**
```rust
// In frontend
fn validate_message(text: &str) -> bool {
    text.len() > 0 && text.len() < 10000
}
```

**✅ Right:**
```rust
// Let backend handle validation
// Frontend just sends and displays errors
```

### 2. Don't Build Everything at Once

**❌ Wrong:**
```rust
// Trying to build entire UI in one session
```

**✅ Right:**
```rust
// Phase 1: Basic setup
// Phase 2: API client
// Phase 3: Session list
// ... incremental progress
```

### 3. Don't Ignore Errors

**❌ Wrong:**
```rust
let sessions = fetch_sessions().await.unwrap();
```

**✅ Right:**
```rust
let sessions = match fetch_sessions().await {
    Ok(s) => s,
    Err(e) => {
        // Show error to user
        return rsx! { div { "Error: {e}" } };
    }
};
```

### 4. Don't Hardcode URLs

**❌ Wrong:**
```rust
reqwest::get("http://localhost:7070/session")
```

**✅ Right:**
```rust
const API_BASE: &str = env!("API_URL", "http://localhost:7070");
reqwest::get(format!("{}/session", API_BASE))
```

---

## Development Workflow

### Running Everything

**Terminal 1 - Backend:**
```bash
cd crow
cargo run --bin crow serve -p 7070
# or
crow serve -p 7070
```

**Terminal 2 - Frontend:**
```bash
cd crow/packages/web
dx serve --port 8080
```

**Browser:**
```
http://localhost:8080  # Web UI
http://localhost:7070  # API (for testing)
```

### Hot Reload

Dioxus supports hot reload out of the box:
- Edit `.rs` files
- Browser updates automatically
- No manual refresh needed

### Debugging

**Backend logs:**
```bash
RUST_LOG=debug crow serve -p 7070
```

**Frontend:**
- Browser DevTools
- Console.log via `web_sys::console::log_1`
- Dioxus DevTools (if available)

---

## Next Session Checklist

Before starting Phase 1, verify:

- [ ] Crow API running on port 7070
- [ ] Can curl endpoints successfully
- [ ] `packages/web/Cargo.toml` exists
- [ ] Dioxus CLI installed (`cargo install dioxus-cli`)
- [ ] Read OpenCode TUI source (at least session view)
- [ ] Have this document open for reference

**First command to run:**
```bash
cd crow/packages/web
dx serve --port 8080
```

**Goal for first session:**
See "Hello Crow" in browser at `http://localhost:8080` 🦅

---

## Final Thoughts

The backend is solid. We've proven the core loop works. We have tools, agents, sessions, everything.

**Now we need to make it visible.**

Building the web UI isn't about adding features—it's about **revealing** what's already there. The agents are working, the tools are executing, the sessions are being created.

We just can't see it yet.

**Let's build the eyes.** 🦅

---

**Ready to start?** Begin with Phase 1 in a fresh session.

**Questions?** All the answers are in:
- This document
- `crow/AGENTS.md` (architecture)
- `crow/docs/IMMEDIATE_NEXT_STEP_CONTEXT_SESSION_TOOL_FIX.md` (current status)
- OpenCode TUI source (`opencode/packages/opencode/src/cli/cmd/tui/`)

**Let's fucking go.** 🚀
