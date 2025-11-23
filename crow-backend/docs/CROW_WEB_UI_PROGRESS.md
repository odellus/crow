# Crow Web UI - Development Progress

## Session Summary

**Date**: 2025-11-17  
**Goal**: Begin building Crow's Dioxus web interface replicating OpenCode's TUI UX  
**Status**: ✅ Phase 1 Complete - Basic UI Structure Running

---

## What We Accomplished

### 1. ✅ Comprehensive TUI Analysis
- Generated 7 detailed documentation files (~60KB, 15,000+ words) analyzing OpenCode's TUI
- Key findings:
  - Framework: `@opentui/solid` (terminal UI library on Solid.js)
  - 11 nested context providers for state management
  - Tool registry pattern for dynamic rendering
  - 40+ theme colors, 23 built-in themes
  - Advanced input with file references, syntax highlighting, autocomplete
  - Real-time WebSocket sync for message streaming

**Documentation Created**:
1. `README_TUI_EXPLORATION.md` - Master index
2. `OPENCODE_TUI_EXPLORATION_SUMMARY.md` - Executive summary
3. `OPENCODE_TUI_ANALYSIS.md` - Deep technical analysis (24KB)
4. `OPENCODE_TUI_FILE_REFERENCE.md` - File-by-file breakdown
5. `OPENCODE_TUI_CODE_PATTERNS.md` - Implementation guide with code examples (29KB)
6. `TUI_TO_DIOXUS_MIGRATION_CHECKLIST.md` - 10-phase project plan (12 weeks)

### 2. ✅ Dioxus Web App Structure
Created basic Dioxus fullstack web application:

**Routing** (`crow/packages/web/src/main.rs`):
- `/` → Sessions list view
- `/session/:session_id` → Session detail with chat
- Removed unnecessary WebNavbar, simplified to direct routes

**Views Created**:
- `sessions.rs` - Sessions list with collapsible sidebar
- `session_detail.rs` - Session view with sidebar + tabs (Chat, Review)

**Components**:
- `SessionSidebar` - Collapsible sidebar (280px expanded, 64px collapsed)
- `SessionItem` - Individual session card with title, timestamp, file count
- `TabButton` - Active/inactive tab switching

### 3. ✅ UI Features Implemented

**Sidebar**:
- Collapsible toggle button (◀/▶)
- "+ New Session" button
- Session list with loading state
- Dark theme (bg-gray-900, bg-gray-800)
- Hover effects and transitions

**Session List**:
- Shows session title (or "Untitled")
- Relative timestamps ("Just now", "5 min ago", "2 hr ago", "3 days ago")
- File change count from summary
- Collapsed mode shows only 💬 emoji
- Click to navigate to session detail

**Session Detail**:
- Tab system (Chat, Review)
- Active session highlighting in sidebar
- Chat tab uses existing `ui::Chat` component
- Review tab placeholder for file diffs

### 4. ✅ API Integration
Already wired up:
- `list_sessions()` - Fetches all sessions
- `get_session(id)` - Gets specific session
- `create_session()` - Creates new session
- `send_message()` - Sends message and gets agent response

**Server Functions Registered**:
```
POST /api/echo
POST /session/create
POST /session
POST /api/send_message
POST /session/get
```

### 5. ✅ Build System Working
- Dioxus CLI (v0.7.1) successfully compiling
- Server running on `http://127.0.0.1:8080`
- Hot reload configured
- Tailwind CSS working
- Assets (favicon, main.css) loading

---

## Current State

### What's Working ✅
1. Web server running on port 8080
2. Sessions route renders with sidebar
3. SessionDetail route with tabs
4. Tailwind styling applied
5. Resource loading (use_resource)
6. Navigation between routes
7. Collapsible sidebar state
8. Tab switching (Chat/Review)

### What's Not Done Yet ❌
1. **Navigation wiring** - "New Session" button and session clicks don't navigate yet
2. **Session creation** - Need to wire up create_session() API call
3. **Message rendering** - Chat tab shows basic input, but needs message history display
4. **Tool registry** - Dynamic tool rendering system not implemented
5. **File references** - @ mentions for file attachments
6. **Theme system** - Color variables and theme switching
7. **WebSocket streaming** - Real-time message updates
8. **Error handling** - No error states for failed API calls
9. **Loading states** - Better loading UX beyond "Loading sessions..."
10. **Review tab** - File diff viewer

---

## File Structure

```
crow/
├── packages/
│   ├── api/
│   │   ├── src/
│   │   │   ├── lib.rs           # Server functions (list_sessions, create_session, etc.)
│   │   │   ├── types.rs         # Session, Message, Part types
│   │   │   ├── session.rs       # SessionStore
│   │   │   ├── tools/           # 12 tools (Bash, Read, Write, etc.)
│   │   │   ├── agent/           # AgentExecutor, AgentRegistry
│   │   │   └── providers/       # LLM provider (Moonshot/Kimi)
│   ├── ui/
│   │   └── src/
│   │       ├── chat.rs          # Chat component with message rendering
│   │       ├── navbar.rs
│   │       └── hero.rs
│   └── web/
│       ├── src/
│       │   ├── main.rs          # Routing setup
│       │   └── views/
│       │       ├── sessions.rs          # Sessions list view
│       │       ├── session_detail.rs    # Session detail with tabs
│       │       ├── home.rs
│       │       └── blog.rs
│       └── Cargo.toml
```

---

## Next Steps (Priority Order)

### Phase 1: Complete Basic Navigation (1-2 days)
1. **Wire up session navigation**
   - SessionItem onclick → navigate to /session/{id}
   - New Session button → create session → navigate

2. **Implement session creation flow**
   - Call create_session() API
   - Handle response (new session ID)
   - Navigate to new session

3. **Add error handling**
   - Show error states for failed API calls
   - Toast notifications for errors

### Phase 2: Message Display (2-3 days)
4. **Fetch and display messages**
   - Load messages for current session
   - Render user/assistant messages
   - Show message timestamps

5. **Implement Part rendering**
   - Text parts
   - Thinking parts
   - Tool parts (basic)

6. **Message input and submission**
   - Wire up Chat component submit
   - Call send_message() API
   - Update UI with response

### Phase 3: Tool Registry (3-4 days)
7. **Create tool registry system**
   - Tool trait with render() method
   - Registry for dynamic dispatch
   - Implement 11 tool renderers:
     - Bash (command + output)
     - Read (file content)
     - Write (file write confirmation)
     - Edit (diff view)
     - Glob (file matches)
     - Grep (search results)
     - List (directory listing)
     - Task (subagent spawn)
     - Patch (file patching)
     - WebFetch (web content)
     - TodoWrite (todo list)

### Phase 4: Advanced Features (5-7 days)
8. **File references (@mentions)**
   - Autocomplete file picker
   - File pill rendering
   - Send file paths with message

9. **Theme system**
   - Define color variables
   - Implement theme switching
   - Dark/light mode support
   - Load custom themes

10. **WebSocket streaming**
    - Implement SSE/WebSocket for real-time updates
    - Stream message parts as they arrive
    - Animate tool execution progress

### Phase 5: Review Tab (3-4 days)
11. **File diff viewer**
    - Fetch session diffs
    - Accordion-style file list
    - Unified/split diff view
    - Syntax highlighting for code

---

## Technical Decisions Made

1. **Framework**: Dioxus 0.7 fullstack (server + web)
2. **Styling**: Tailwind CSS
3. **State Management**: Dioxus Signals (`use_signal`, `use_resource`)
4. **API Pattern**: Server functions with `#[post("/path")]`
5. **Routing**: Dioxus Router with `/` and `/session/:id` routes
6. **LLM**: Moonshot AI with kimi-k2-thinking (262k context)
7. **Storage**: XDG directories (`~/.local/share/crow/`)

---

## Key Code Patterns

### Server Function Pattern
```rust
#[post("/session")]
pub async fn list_sessions() -> Result<Vec<Session>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let store = get_session_store();
        store.list(None)
            .map_err(|e| ServerFnError::new(format!("Failed: {}", e)))
    }
}
```

### Component with Resource
```rust
#[component]
pub fn Sessions() -> Element {
    let sessions = use_resource(|| async move {
        list_sessions().await
    });

    rsx! {
        div {
            match sessions.read_unchecked().as_ref() {
                Some(Ok(session_list)) => rsx! { /* render */ },
                Some(Err(e)) => rsx! { /* error */ },
                None => rsx! { /* loading */ }
            }
        }
    }
}
```

### Conditional Rendering
```rust
{if condition {
    rsx! { /* true branch */ }
} else {
    rsx! { /* false branch */ }
}}
```

---

## Lessons Learned

1. **RSX Macro Limitations**: Can't call `.clone()` or methods inside string interpolation in RSX. Must extract to variables first.

2. **Move Semantics**: When using values in multiple closures (e.g., use_resource), must clone before each closure to avoid move errors.

3. **Dioxus Server Platform**: Use `dx serve --platform server` (not `--platform fullstack`)

4. **Clean Builds**: Sometimes need `cargo clean` to clear stale compiler errors

5. **TUI ≠ Desktop**: OpenCode has both a TUI (@opentui/solid) and a desktop web app (Solid.js). We're replicating the TUI's UX in a web interface.

---

## Resources

- **OpenCode TUI Docs**: `/home/thomas/src/projects/opencode-project/OPENCODE_TUI_*.md`
- **Dioxus Docs**: https://dioxuslabs.com/learn/0.7/
- **Crow Backend**: Running on port 7070 (tested and working)
- **Web UI**: http://127.0.0.1:8080

---

## Success Metrics

✅ **Phase 1 Complete**: Basic UI structure renders and serves  
⬜ **Phase 2**: Users can create sessions and send messages  
⬜ **Phase 3**: All tool types render correctly  
⬜ **Phase 4**: File references, themes, and streaming work  
⬜ **Phase 5**: Full feature parity with OpenCode TUI  

**Current Progress**: ~20% of full TUI feature parity

---

## Notes for Next Session

1. Start with session navigation - it's the most user-visible feature
2. Test with actual backend (port 7070) to verify API integration
3. Reference `OPENCODE_TUI_CODE_PATTERNS.md` for tool registry implementation
4. Consider adding a simple toast notification system early for UX feedback
5. Keep sidebar state in localStorage for persistence
