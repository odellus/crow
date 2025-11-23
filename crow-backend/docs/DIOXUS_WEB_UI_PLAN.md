# Dioxus Web UI Implementation Plan

**Goal:** Replicate OpenCode TUI capabilities in a Dioxus-powered web UI

---

## OpenCode TUI Architecture Analysis

The TUI has **64 files** organized as:

### Core Structure
```
tui/
├── app.tsx              # Main app with providers
├── event.ts             # Event system
├── spawn.ts             # Process spawning
├── thread.ts            # Worker threads
├── worker.ts            # Background processing
├── attach.ts            # Terminal attachment
├── context/             # 9 React contexts (state management)
├── component/           # Reusable UI components
├── routes/              # Page views
├── ui/                  # Base UI primitives
└── util/                # Helpers (clipboard, editor, terminal)
```

### Context Providers (State Management)
1. **args** - CLI arguments
2. **exit** - Exit handling
3. **helper** - UI helper functions
4. **keybind** - Keyboard shortcuts
5. **kv** - Key-value storage
6. **local** - Local state
7. **route** - Navigation routing
8. **sdk** - API client (ACP connection)
9. **sync** - State synchronization
10. **theme** - Theming (24 themes)

### UI Components
- **Dialogs:** agent, command, model, session-list, session-rename, status, tag, theme-list
- **Prompt:** Input with autocomplete and history
- **Base UI:** dialog, dialog-select, dialog-prompt, dialog-help, dialog-confirm, dialog-alert, toast, shimmer

### Routes/Views
- **home.tsx** - Landing/dashboard
- **session/** - Session view with:
  - index.tsx - Main session view
  - header.tsx - Session header
  - sidebar.tsx - Session list sidebar
  - dialog-timeline.tsx - Message timeline
  - dialog-message.tsx - Message details

---

## Dioxus Web UI Plan

### Phase 1: Foundation (Core Shell)

**1.1 Project Setup**
- Dioxus fullstack with SSR
- Tailwind CSS for styling
- Route system setup

**1.2 State Management**
Create Dioxus equivalents of React contexts:
```rust
// Global signals for app state
struct AppState {
    sessions: Signal<Vec<Session>>,
    current_session: Signal<Option<String>>,
    messages: Signal<Vec<MessageWithParts>>,
    theme: Signal<Theme>,
    loading: Signal<bool>,
}
```

**1.3 API Client**
Server functions wrapping Crow API:
```rust
#[server]
async fn list_sessions() -> Result<Vec<Session>, ServerFnError>

#[server]
async fn get_session(id: String) -> Result<Session, ServerFnError>

#[server]
async fn send_message(session_id: String, content: String) -> Result<MessageWithParts, ServerFnError>

#[server]
async fn create_session() -> Result<Session, ServerFnError>
```

### Phase 2: Core Views

**2.1 Layout Component**
```
┌─────────────────────────────────────────┐
│ Header (logo, agent selector, model)    │
├──────────┬──────────────────────────────┤
│ Sidebar  │ Main Content                 │
│ Sessions │ (Messages/Chat)              │
│          │                              │
│          │                              │
│          ├──────────────────────────────┤
│          │ Input Area                   │
└──────────┴──────────────────────────────┘
```

**2.2 Session List (Sidebar)**
- List all sessions sorted by time
- Click to navigate
- New session button
- Delete session option
- Current session highlight

**2.3 Message Display**
- User messages (right-aligned)
- Assistant messages (left-aligned)
- Tool execution blocks (collapsible)
- Thinking/reasoning blocks (collapsible, muted)
- Auto-scroll to bottom

**2.4 Message Input**
- Textarea with send button
- Enter to send (Shift+Enter for newline)
- Loading state while processing

### Phase 3: Tool Rendering

**3.1 Tool Output Components**
Each tool type needs a renderer:

```rust
fn render_tool_part(part: &Part) -> Element {
    match &part.tool_state {
        ToolState::Pending { .. } => render_pending(),
        ToolState::Running { title, .. } => render_running(title),
        ToolState::Completed { output, .. } => render_completed(output),
        ToolState::Error { error, .. } => render_error(error),
    }
}
```

**3.2 Tool-Specific Renderers**
- **bash** - Terminal-style output with syntax highlighting
- **read** - Code block with filename header
- **write/edit** - Diff view showing changes
- **grep/glob** - File list with matches
- **todowrite** - Todo list visualization
- **task** - Subagent execution indicator

### Phase 4: Dialogs & Modals

**4.1 Essential Dialogs**
- Session list (quick switch)
- Model selector
- Agent selector
- Confirmation dialogs
- Error alerts

**4.2 Dialog System**
```rust
enum Dialog {
    None,
    SessionList,
    ModelSelector,
    AgentSelector,
    Confirm { message: String, on_confirm: Callback },
}

static DIALOG: GlobalSignal<Dialog> = Signal::global(|| Dialog::None);
```

### Phase 5: Real-time Features

**5.1 Streaming (SSE)**
- Connect to `/session/:id/message/stream`
- Update messages as tokens arrive
- Show tool execution progress

**5.2 Auto-refresh**
- Poll for session updates
- Sync todo list changes
- Handle concurrent sessions

### Phase 6: Polish

**6.1 Keyboard Shortcuts**
- `Cmd/Ctrl+N` - New session
- `Cmd/Ctrl+K` - Command palette
- `Escape` - Close dialogs
- Arrow keys in lists

**6.2 Theming**
- Port OpenCode themes (24 available)
- CSS variables for colors
- Theme switcher dialog

**6.3 Loading States**
- Skeleton loaders for sessions
- Spinner for message sending
- Progress for tool execution

---

## Component Mapping: TUI → Dioxus

| TUI Component | Dioxus Equivalent | Priority |
|---------------|-------------------|----------|
| `routes/session/index.tsx` | `SessionView` | P0 |
| `routes/session/sidebar.tsx` | `SessionSidebar` | P0 |
| `component/prompt/index.tsx` | `MessageInput` | P0 |
| `routes/session/header.tsx` | `SessionHeader` | P1 |
| `ui/dialog.tsx` | `Dialog` | P1 |
| `component/dialog-session-list.tsx` | `SessionListDialog` | P1 |
| `component/dialog-model.tsx` | `ModelDialog` | P2 |
| `component/dialog-agent.tsx` | `AgentDialog` | P2 |
| `ui/toast.tsx` | `Toast` | P2 |
| `context/theme.tsx` | `ThemeProvider` | P3 |

---

## File Structure

```
crow/packages/web/src/
├── main.rs                 # Entry point
├── app.rs                  # Root component
├── state.rs                # Global signals
├── api.rs                  # Server functions
├── routes/
│   ├── mod.rs
│   ├── home.rs             # Landing page
│   └── session.rs          # Session view
├── components/
│   ├── mod.rs
│   ├── layout.rs           # App shell
│   ├── sidebar.rs          # Session list
│   ├── message.rs          # Message bubble
│   ├── tool_output.rs      # Tool result rendering
│   ├── input.rs            # Message input
│   └── header.rs           # Session header
├── ui/
│   ├── mod.rs
│   ├── dialog.rs           # Modal system
│   ├── toast.rs            # Notifications
│   └── button.rs           # Button variants
└── theme/
    ├── mod.rs
    └── themes.rs           # Theme definitions
```

---

## Implementation Order

### Sprint 1: Minimum Viable Chat (3-4 days)
1. Layout with sidebar + main content
2. Session list (load from API)
3. Session navigation (click to view)
4. Message display (user + assistant)
5. Message input + send

**Deliverable:** Can create sessions, send messages, see responses

### Sprint 2: Tool Visualization (2-3 days)
6. Tool output rendering (all states)
7. Collapsible tool blocks
8. Syntax highlighting for code
9. Loading states

**Deliverable:** Full visibility into what agents are doing

### Sprint 3: Session Management (1-2 days)
10. Create new session
11. Delete session
12. Session title editing
13. Session list search/filter

**Deliverable:** Complete session lifecycle

### Sprint 4: Agent Features (2-3 days)
14. Agent selector
15. Model selector
16. Todo list display
17. Child session navigation

**Deliverable:** Agent control and planning visibility

### Sprint 5: Polish (2-3 days)
18. Keyboard shortcuts
19. Theming
20. Error handling UI
21. Streaming (if API ready)

**Deliverable:** Production-quality UX

---

## Key Differences from TUI

### Advantages of Web
- No terminal constraints
- Better text rendering
- Easier styling (CSS)
- Copy/paste just works
- Multi-window support
- Shareable URLs

### Simplifications
- No terminal size handling
- No ANSI escape codes
- Standard input handling
- No cursor management

### New Opportunities
- Drag-and-drop files
- Rich markdown rendering
- Image display in messages
- Split views for tool output
- Persistent layouts

---

## Dependencies

```toml
[dependencies]
dioxus = { version = "0.6", features = ["fullstack", "router"] }
dioxus-ssr = "0.6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }
```

**Styling:** Tailwind CSS via CDN or build step

---

## Success Criteria

1. **Functional:** Can replicate a full agent session (send message → see tools → get response)
2. **Visible:** All tool executions clearly displayed with state indicators
3. **Navigable:** Switch between sessions, see history
4. **Responsive:** Works on different screen sizes
5. **Fast:** Sub-100ms interactions, efficient re-renders

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| SSE not ready | Start with polling, add streaming later |
| Complex tool output | Start with plain text, enhance progressively |
| State management | Use Dioxus signals, simple global state |
| Hydration issues | Test SSR/client carefully, use `client::spawn` |

---

## Next Action

Start with **Sprint 1, Item 1**: Create the basic layout component with sidebar and main content area. This establishes the visual foundation for everything else.
