# OpenCode TUI Implementation Analysis

## Executive Summary

OpenCode's TUI is built with **@opentui/solid** - a Solid.js-based terminal UI library that provides low-level rendering primitives. The architecture is modular, state-driven, and uses context providers for global state management. The TUI differs significantly from a web UI by focusing on keyboard-first interaction, terminal constraints, and ASCII/ANSI rendering.

---

## 1. Terminal UI Framework

### Framework: @opentui/solid (v0.1.42)

**Key Dependencies:**
```json
"@opentui/core": "0.1.42",
"@opentui/solid": "0.1.42",
"solid-js": "catalog:",
```

**Why OpenTUI?**
- Low-level terminal rendering with full control over ANSI codes
- Component-based system (Solid.js) similar to React/Dioxus
- Supports syntax highlighting via `SyntaxStyle`
- Advanced features: scrolling, mouse events, extmarks (virtual text annotations), text selection
- Custom scroll acceleration support
- Keyboard event handling with modifier keys (ctrl, meta, shift)

**Key OpenTUI Primitives:**
- `<box>` - Layout container (flexbox-like)
- `<text>` - Styled text rendering
- `<textarea>` - Multi-line input with extmarks support
- `<code>` - Syntax-highlighted code blocks
- `<scrollbox>` - Scrollable container with acceleration
- Renderable objects for low-level control

---

## 2. Complete TUI File Structure (46 files)

### Core Entry Point
**`/tui/app.tsx`** (Main TUI application)
- Entry point: `tui()` function
- Renders nested context providers stack
- Detects terminal background color (dark/light mode)
- Handles global keyboard shortcuts (Ctrl+C exit, selection copying)
- Root layout: header (top bar with version/agent info) + routes + footer

### Routes (Screen-level components)
```
/tui/routes/
├── home.tsx              # Home screen - Logo, prompt, help hints
└── session/
    ├── index.tsx         # Main session view - messages, prompt, sidebar
    ├── header.tsx        # Session title, context tokens, cost
    ├── sidebar.tsx       # Collapsible sections: MCP servers, LSP, todo, diffs
    └── dialog-message.tsx # Modal for editing/viewing single messages
```

**Session Route (`index.tsx`) - 1564+ lines:**
- Central hub for all session interactions
- Displays conversation history with scrolling
- Manages message rendering (user/assistant/tool outputs)
- Handles permissions/confirmations
- Contains tool-specific renderers via `ToolRegistry`
- Supports code concealment toggle, sidebar visibility, child session navigation
- Command registration for session operations (rename, compact, share, undo/redo)

### UI Components (Primitives & Dialogs)

**Dialog System** (`/tui/ui/`):
```
dialog.tsx                 # Base Dialog wrapper - modal overlay with ESC handling
dialog-select.tsx          # Fuzzy searchable list (sessions, models, agents)
dialog-prompt.tsx          # Single text input with validation
dialog-confirm.tsx         # Yes/no confirmation
dialog-alert.tsx           # Message display
dialog-help.tsx            # Keybinds reference
toast.tsx                  # Non-modal notifications (info/success/error)
shimmer.tsx                # Loading animation with text
```

**Component Library** (`/tui/component/`):
```
prompt/index.tsx           # Main user input - textarea with file drag support
prompt/autocomplete.tsx    # Context-aware suggestions
prompt/history.tsx         # Command history management
dialog-session-list.tsx    # Session selector with delete/rename
dialog-session-rename.tsx  # Rename session form
dialog-model.tsx           # Model selection
dialog-agent.tsx           # Agent selection (disabled in current version)
dialog-command.tsx         # Command palette - dynamic keybind mapping
dialog-status.tsx          # System status display
dialog-theme-list.tsx      # Theme picker with preview
dialog-tag.tsx             # Tag management
border.tsx                 # Split/border styles (SplitBorder component)
logo.tsx                   # ASCII logo
```

### Context Providers (Global State)

**`/tui/context/`** - 11 context managers:

| File | Purpose | Key Features |
|------|---------|--------------|
| `route.tsx` | Navigation state | home/session routes, browser-like routing |
| `theme.tsx` | Color/styling | 23+ built-in themes, dark/light mode, syntax highlighting |
| `keybind.tsx` | Keyboard shortcuts | Leader key support, custom keybind parsing |
| `local.tsx` | User preferences | Current model, agent, sidebar state |
| `sync.tsx` | Remote data sync | Messages, sessions, providers, config |
| `sdk.tsx` | API client | Backend HTTP client wrapper |
| `dialog.tsx` (ui/) | Modal management | Stack-based dialog system |
| `kv.tsx` | Local storage | Persistent key-value data |
| `args.tsx` | CLI arguments | Startup options (agent, model, sessionID, prompt) |
| `helper.tsx` | Context utilities | Simple context creation pattern |
| `command.tsx` (component/) | Command palette | Dynamic command registration |

### Message Rendering System

**Tool-Specific Renderers** (in `routes/session/index.tsx`):
```typescript
ToolRegistry.register({
  name: "bash",      // Bash command + output
  name: "read",      // File read operations
  name: "write",     // File modifications with syntax highlighting
  name: "edit",      // Diffs (split view or stacked)
  name: "glob",      // File search
  name: "grep",      // Content search
  name: "list",      // Directory listing
  name: "task",      // Subagent task delegation
  name: "patch",     // Patch application
  name: "webfetch",  // Web fetch operations
  name: "todowrite", // Todo updates
})
```

Each tool renderer is a Solid component that displays:
- Tool icon + title
- Input parameters
- Execution metadata
- Output/results
- Error handling
- Permission prompts

### Utilities

```
/tui/util/
├── editor.ts      # External editor integration ($EDITOR env var)
├── terminal.ts    # Terminal capabilities detection
├── clipboard.ts   # System clipboard (OSC 52 + clipboardy)
└── (unused: worker.ts, spawn.ts, thread.ts, attach.ts, event.ts)
```

---

## 3. Architecture Patterns

### A. Context Provider Stack

```jsx
<ArgsProvider>
  <ExitProvider>
    <KVProvider>
      <ToastProvider>
        <RouteProvider>
          <SDKProvider>
            <SyncProvider>
              <ThemeProvider>
                <LocalProvider>
                  <KeybindProvider>
                    <DialogProvider>
                      <CommandProvider>
                        <PromptHistoryProvider>
                          <App />
```

**Pattern Benefits:**
- Each provider handles one concern (state, effects, event listeners)
- Providers can use other providers' hooks
- Child components access state via `useContext()` hooks
- Async initialization (themes, config) happens in providers

### B. State Management Pattern

Uses **Solid.js stores** with `createStore()`:
```typescript
const [store, setStore] = createStore({ /* initial state */ })
// Access: store.value
// Update: setStore("value", newValue) or setStore(produce(...))
```

**Example (Theme Provider):**
```typescript
const [store, setStore] = createStore({
  themes: DEFAULT_THEMES,      // All available themes
  mode: props.mode,             // dark/light
  active: 'opencode',           // Selected theme name
  ready: false,                 // Async loading flag
})
```

### C. Hook Pattern for Data Access

```typescript
// Usage in components:
const route = useRoute()           // Navigation state
const sync = useSync()             // Remote data
const { theme } = useTheme()       // Styling
const keybind = useKeybind()       // Shortcuts
const local = useLocal()           // User preferences
const dialog = useDialog()         // Modal stack
const command = useCommandDialog() // Command palette
```

### D. Message Rendering Pattern

**UserMessage Component:**
- Displays user input text
- Shows file attachments with badges
- Renders timestamp or "QUEUED" status for pending messages
- Left border color indicates queued/normal state

**AssistantMessage Component:**
- Maps message `parts` to tool-specific renderers via `ToolRegistry`
- Shows thinking/reasoning text (if available)
- Displays streaming status with model info
- Shows errors with red styling

**Tool Rendering:**
- Each tool registers: name, container type (block/inline), render function
- Render function receives: input params, metadata, permissions, output
- Permission system blocks tool execution (user approval UI)
- Margins/spacing calculated based on previous element height

### E. Prompt Input System

**Textarea Component with Extmarks:**
- Supports multiline input with history navigation (up/down arrows)
- Extmarks = virtual text annotations for file/agent references
- Pasted images/files shown as `[Image 1]` badges
- Syntax highlighting for commands and special tokens
- Autocomplete with agent suggestions
- Shell mode (prefix with `!`) for direct bash commands
- Keybind-driven text transformation (newline vs submit)

**Autocomplete System:**
- Context-aware suggestions
- File path completion
- Agent name completion
- Triggered by typing or hotkeys

---

## 4. UI Component Breakdown

### Dialogs (Modal System)

**Base Dialog:**
- Semi-transparent black overlay (RGBA 0,0,0,150)
- Centered box with configurable width (medium: 60, large: 80)
- ESC to close with cleanup
- Mouse event bubbling prevention
- Focus restoration after close

**DialogSelect:**
- Fuzzy searchable list
- Category grouping (e.g., "Today" for sessions)
- Footer text (timestamps, status)
- Custom keybinds per option (delete, rename, etc.)
- Highlighted current selection

**Session List Dialog:**
- Lists sessions grouped by date
- Delete with double confirmation
- Rename via ctx+r
- Max 150 sessions displayed
- Current session highlighted

**Theme Selection Dialog:**
- Shows all available themes
- Dark/light variant switching
- System theme generation (terminal colors)
- Custom theme loading from `.opencode/themes/`

### Home Screen

**Layout:**
```
┌─────────────────────────────┐
│      OpenCode Logo          │
│                             │
│  Help Row 1: Commands       │
│  Help Row 2: Sessions       │
│  Help Row 3: Models         │
│  Help Row 4: Agents         │
│                             │
│  ┌─────────────────────┐    │
│  │  > Prompt Input     │    │
│  └─────────────────────┘    │
│  Provider/Model | Keybind   │
└─────────────────────────────┘
```

**Features:**
- MCP server connection indicators
- Prompt with pre-filled text from CLI args
- Dynamic help hints based on loaded configs

### Session Screen

**Layout (narrow terminal, <120 width):**
```
┌──────────────────────────┐
│  Session Header          │ (title, cost, sharing info)
├──────────────────────────┤
│  Messages (scrollable)    │ (user/assistant messages)
│  - UserMessage           │
│  - AssistantMessage      │
│  - ToolOutput            │
├──────────────────────────┤
│  > Prompt Input          │ (textarea)
│  Provider/Model | Status │
└──────────────────────────┘
```

**Layout (wide terminal, >120 width):**
```
┌──────────────────────────┬──────────────┐
│                          │              │
│  Messages (scrollable)    │ Sidebar      │
│  - UserMessage           │ (collapsed   │
│  - AssistantMessage      │  sections)   │
│  - ToolOutput            │              │
│                          │ - MCP        │
├──────────────────────────┤ - LSP        │
│  > Prompt Input          │ - Todo       │
│  Provider | Status       │ - Modified   │
│                          │   Files      │
└──────────────────────────┴──────────────┘
```

**Sidebar Collapsible Sections:**
1. **MCP** - Server connection status (connected/failed/disabled)
2. **LSP** - Language server connections
3. **Todo** - Tasks with status indicators (✓ completed)
4. **Modified Files** - Git diff stats (+/-) per file

### Input Components

**Prompt Textarea:**
- Placeholder with keybind hints
- Syntax highlighting (markdown/code-aware)
- Multi-line support (min 1, max 6 lines)
- Cursor color changes based on focus
- Model selector in footer

**Status Indicators (Prompt Footer):**
- "compacting..." - Session summarization in progress
- "esc interrupt" / "again to interrupt" - Stop running agent
- Dynamic hints based on state

---

## 5. Key UI Patterns for Dioxus Replication

### Pattern 1: Layered Context Providers
Create a nested provider structure for state management. In Dioxus:
```rust
fn root() -> Element {
    rsx! {
        RouteProvider {
            ThemeProvider {
                SyncProvider {
                    DialogProvider {
                        App {}
                    }
                }
            }
        }
    }
}
```

### Pattern 2: Flexbox-Based Layout
OpenTUI uses flexbox properties similar to CSS:
```jsx
<box 
  flexDirection="row"        // row/column
  flexGrow={1}              // flex-grow
  flexShrink={0}            // flex-shrink
  gap={1}                   // spacing
  justifyContent="space-between"
  alignItems="center"
/>
```

In Dioxus web, replicate with Tailwind or CSS Grid.

### Pattern 3: Modal Stack System
Dialog system maintains an array of modals:
```typescript
type DialogStack = Array<{
  element: JSX.Element
  onClose?: () => void
}>;
```

Methods:
- `clear()` - Close all
- `replace()` - Replace top dialog
- `push()` - Stack new dialog (not used, but would be useful)

### Pattern 4: Theme as Context Computed Values
Theme context provides:
- Raw color values (RGBA objects)
- Syntax highlighting system
- Memoized color palette
- Dynamic mode switching (dark/light)

### Pattern 5: Keybind Matching System
Keybinds stored as arrays of possible key combinations:
```typescript
interface Keybind {
  ctrl?: boolean
  meta?: boolean
  shift?: boolean
  name: string
  leader?: boolean
}

// Match with parsed keyboard event
if (keybind.match("session_list", evt)) { ... }

// Print human-readable (Ctrl+O, <leader>s, etc.)
const text = keybind.print("session_list")
```

### Pattern 6: Dynamic Command Palette
Command system with:
- Category grouping
- Keybind bindings
- Conditional enable/disable
- Real-time callback triggers
- Custom action handlers

### Pattern 7: Streaming/Incremental Rendering
Code blocks support streaming attribute:
```jsx
<code 
  filetype="javascript"
  streaming={true}          // Indicates content is streaming
  content={props.content}   // Updates trigger re-render
/>
```

### Pattern 8: Tool Registry Pattern
Self-registering component system:
```typescript
interface ToolRegistration {
  name: string
  container: "inline" | "block"
  render: Component<ToolProps>
}

const ToolRegistry = {
  register<T>(registration: ToolRegistration<T>) { ... },
  render(name: string) { ... },
}
```

### Pattern 9: Extmarks (Virtual Annotations)
Text editor with virtual text overlays for file/agent references:
- Each extmark has ID, start/end positions, virtual text
- Styled differently (warning color for files, secondary for agents)
- Positioned over actual text without consuming space
- Moved/deleted when surrounding text changes

In web equivalent: Absolute positioned badges over text input.

### Pattern 10: Async Provider Initialization
Providers load async data in effects:
```typescript
createEffect(async () => {
  const themes = await getCustomThemes()
  setStore(produce(draft => {
    Object.assign(draft.themes, themes)
    draft.ready = true
  }))
})
```

---

## 6. Differences Between TUI and Web UI

| Aspect | TUI (OpenTUI) | Web (Dioxus) |
|--------|---------------|-------------|
| **Rendering** | ANSI/ASCII output to terminal | HTML/CSS/JavaScript |
| **Colors** | 256-color or RGB RGBA objects | CSS colors, unlimited palette |
| **Layout** | Flexbox-like terminal boxes | CSS Flexbox/Grid |
| **Input** | Keyboard-first, raw key events | Form inputs, mouse/touch events |
| **Scrolling** | Custom scroll logic, acceleration | Browser native + CSS overflow |
| **Text Selection** | Manual implementation via OSC 52 | Browser native (Ctrl+A, click-drag) |
| **Styling** | Border characters, colors, attributes (bold, italic, underline) | Full CSS |
| **Modal System** | Overlay with semi-transparent bg | CSS modal (backdrop-filter, z-index) |
| **Focus Management** | Terminal focus, blur, manual tracking | Browser focus events |
| **Syntax Highlighting** | Tree-sitter based, custom rendering | Highlight.js or Prism |
| **Responsiveness** | Terminal width/height constraints | Viewport pixels, responsive breakpoints |
| **Async Loading** | Blocks rendering until ready | Shows skeleton/loading state |
| **Persistence** | KV store (files or DB) | Browser localStorage/IndexedDB |
| **Clipboard** | OSC 52 escape codes + clipboardy | navigator.clipboard API |

---

## 7. Key Implementation Decisions

### A. Why No Reactive Message Sync?
The session displays messages from a synced store (`sync.data.message[sessionID]`). When new messages arrive:
1. Backend pushes via WebSocket event
2. `SyncProvider` updates store
3. Solid reactivity triggers component re-render
4. `scrollbox.scrollBy(100_000)` snaps to bottom

### B. Tool Container System
Tools can be "inline" (part of message flow) or "block" (full-width cards):
- Inline: Read, Glob, Grep, List, WebFetch - show in message stream
- Block: Bash, Write, Edit, Task, Patch, TodoWrite - dedicated sections

This controls visual hierarchy and spacing.

### C. Permission System
Some tools require approval before execution:
- Shows permission prompt in message
- User presses Enter/A/D to approve/deny
- Permission metadata persists in message history

### D. Undo/Revert System
Sessions track reversion state:
- `session.revert.messageID` - revert point
- `session.revert.diff` - git diff of changes since revert
- Reverting shows "X messages reverted" bar with diff stats
- Redo available until a new message sent

### E. Shell Mode
Typing `!` at prompt start activates shell mode:
- Sends directly to bash tool
- Bypasses agent/model selection
- Exit with Escape or backspace at start

---

## 8. Theme System Deep Dive

### Theme Colors (40+ properties)

**Primary Colors:**
- `primary` - Cyan accent color (command highlights)
- `secondary` - Magenta (user input, secondary actions)
- `accent` - Cyan (important highlights, queued state)

**Status Colors:**
- `error` - Red
- `warning` - Yellow
- `success` - Green
- `info` - Cyan

**UI Colors:**
- `text` - Default foreground
- `textMuted` - Reduced emphasis (50% opacity equivalent)
- `background` - Terminal background
- `backgroundPanel` - Slightly lighter for contrast
- `backgroundElement` - Raised surface (buttons, inputs)
- `border` - Border lines
- `borderActive` - Active state borders
- `borderSubtle` - Very faint borders

**Diff Colors:**
- `diffAdded/Removed/Context` - Green/red/gray
- `diffHunkHeader/HighlightAdded/HighlightRemoved` - Same colors
- `diffAddedBg/RemovedBg/ContextBg` - Background versions
- `diffLineNumber/Added/RemovedLineNumberBg` - Gutter

**Markdown Colors:**
- Heading, Link, Code, BlockQuote, Emphasis, Strong - For rendered markdown

**Syntax Colors:**
- Comment, Keyword, Function, Variable, String, Number, Type, Operator, Punctuation

### Theme Loading

1. **Default Themes** - 23 built-in themes (23 KB each)
2. **System Theme** - Generated from terminal palette (ANSI colors)
3. **Custom Themes** - Loaded from `~/.opencode/config/themes/*.json`
4. **Dark/Light Variants** - Each theme specifies colors per mode

---

## 9. Keyboard Interaction Model

### Global Shortcuts (Always Available)
- `Ctrl+C` - Exit TUI
- `<leader>` key - Activates leader mode (2-second timeout)
- Command palette trigger - Default unset, but registered

### Context-Specific Shortcuts

**Session Screen:**
- `Ctrl+X s` - View status
- `Ctrl+X ?` - Help
- `Tab` / `Shift+Tab` - Navigate/search commands
- Arrow keys - History navigation, message scrolling
- `Ctrl+K` - Clear prompt
- `!` at start - Shell mode
- `<leader>t` - Timeline (jump to message)
- `<leader>c` - Copy last assistant message
- `Ctrl+D` twice - Delete session

**Prompt Input:**
- `Ctrl+<newline>` - Add newline
- `Enter` - Submit
- `Up/Down` - History (at start/end)
- `Ctrl+U` - Clear line
- `Ctrl+W` - Delete word
- `Ctrl+R` - Paste (if not OSC 52)

**Dialog Navigation:**
- `Up/Down` / `j/k` - Move selection
- `Enter` - Select
- `/` - Start search
- `Escape` - Close

---

## 10. Performance Considerations

### Scrolling Optimization
- `stickyScroll` attribute keeps view at bottom when content added
- Custom scroll acceleration: MacOSScrollAccel or CustomSpeedScroll
- Visible scrollbar only on hover
- Track background color changes on scroll

### Message Rendering
- Messages rendered via `For each` (Solid efficiency)
- Tool components lazy-loaded via Dynamic
- Streaming code updates trigger partial re-renders
- Concealment toggle hides sensitive content without removing from DOM

### State Updates
- `createStore` + `produce` for immutable updates
- Memoized derived values to prevent cascading renders
- Effects run selectively on dependencies
- Context consumers only subscribe to used values

### Memory Usage
- Session data capped at 150 displayed sessions
- Messages streamed from backend (not stored client-side)
- Extmarks cleared after input submission
- Theme assets lazy-loaded

---

## 11. Recommended Dioxus Web Architecture

### Directory Structure
```
src/
├── app.tsx              # Root with providers
├── routes/
│   ├── home.tsx
│   └── session/
│       ├── mod.tsx
│       ├── header.tsx
│       ├── sidebar.tsx
│       └── messages.tsx
├── contexts/
│   ├── route.rs
│   ├── theme.rs
│   ├── dialog.rs
│   └── sync.rs
├── components/
│   ├── prompt.rs
│   ├── dialog_select.rs
│   └── tool_renderers/
│       ├── bash.rs
│       ├── write.rs
│       └── edit.rs
└── styles/
    ├── theme.css
    └── components.css
```

### State Management Strategy
1. **Context Providers** - Global state (theme, routes, dialogs)
2. **Use Signals** - Component-level reactive state
3. **Selectors** - Memoized derived values
4. **Event System** - For cross-component communication

### Component Patterns
1. **Functional Components** - All components as functions
2. **Props Structs** - Type-safe component props
3. **Hooks** - Custom hooks for state logic
4. **Composition** - Render functions for dynamic content

---

## Summary: What Makes OpenCode's TUI Special

1. **Terminal-First Design** - Every interaction optimized for keyboard + terminal constraints
2. **Rich Text Rendering** - Syntax highlighting, markdown, virtual annotations
3. **Async-Aware** - Streaming output, pending states, real-time sync
4. **Tool-Centric** - Each tool has custom UI, organized via registry
5. **Modal Stack** - Professional dialog management with focus restoration
6. **Keybind System** - Vim-like leader key + customizable shortcuts
7. **Theme System** - 23 themes + dynamic generation + custom support
8. **Message Streaming** - Incremental rendering as agent responds
9. **Permission System** - User approval for sensitive operations
10. **Undo/Revert** - Full session history with diff tracking

Replicating these in Dioxus web requires:
- Theming system (CSS variables or Tailwind)
- Modal management (Portals + z-index stack)
- Keybind handler (keydown event listener)
- Syntax highlighting (highlight.js/Prism)
- Virtual scrolling (if many messages)
- Real-time updates (WebSocket with Signals)
- Tool renderer registry
- Permission dialogs

