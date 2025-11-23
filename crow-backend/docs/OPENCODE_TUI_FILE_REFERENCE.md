# OpenCode TUI - Quick File Reference

## Core Files to Study (in order)

### 1. Main Entry Point
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/app.tsx`

**What it does:**
- Detects terminal background color (dark vs light)
- Wraps entire app in 11 nested context providers
- Renders root layout: flex column with routes + bottom footer
- Handles global events: Ctrl+C exit, text selection with OSC 52 clipboard
- Manages command palette registrations

**Key functions:**
- `getTerminalBackgroundColor()` - Terminal color detection
- `tui(input)` - Main entry function returns Promise
- `<App />` - Root component with all providers
- `<ErrorComponent />` - Fallback error UI

**Learn this for:** Understanding provider architecture and app initialization

---

### 2. Message Display System
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/routes/session/index.tsx`

**What it does:**
- Displays conversation history (user + assistant messages)
- Renders each message type via `<For each={messages()}>`
- Dynamically dispatches to tool renderers via `ToolRegistry`
- Manages scrolling with sticky bottom
- Handles message interactions (click to edit)
- Implements tool permission system

**Key components:**
- `<Session />` - Main route component (1564 lines!)
- `<UserMessage />` - Displays user input with file attachments
- `<AssistantMessage />` - Dispatches parts to renderers
- `<TextPart />` - Renders markdown text
- `<ToolPart />` - Renders tool execution (bash, write, edit, etc.)
- `<ReasoningPart />` - Shows thinking/reasoning

**Tool Renderers (examples):**
```
bash:     Command + output
write:    Filename + syntax-highlighted code with diagnostics
edit:     Old/new code side-by-side (split view) or stacked
read:     File path + read operation
glob:     Pattern + match count
grep:     Pattern + match count
task:     Subagent delegation info
webfetch: URL display
```

**Learn this for:** Understanding message rendering, tool registry pattern, scrolling UI

---

### 3. User Input System
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/component/prompt/index.tsx`

**What it does:**
- Multi-line textarea input with history
- Syntax highlighting + markdown mode
- File/image drag-paste support with virtual badges
- Extmarks system for virtual annotations
- Command submission with model + agent selection
- Shell mode (prefix with `!`)
- Autocomplete with file/agent suggestions

**Key concepts:**
- `Extmarks` - Virtual text overlays (files show as `[Image 1]`)
- `PromptInfo` - State type with input string + parts array
- `TextareaRenderable` - OpenTUI textarea component
- History management via `usePromptHistory()`

**Key methods:**
- `submit()` - Send prompt to backend
- `restoreExtmarksFromParts()` - Restore badges from saved state
- `syncExtmarksWithPromptParts()` - Update positions when text changes
- `pasteImage()` - Add image badge to prompt

**Learn this for:** Understanding input handling, virtual text, state management

---

### 4. Theme & Styling System
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/context/theme.tsx`

**What it does:**
- Manages 23 built-in + custom themes
- Dark/light mode toggling
- Terminal color palette generation
- Syntax highlighting via `SyntaxStyle`
- Persists theme preference

**Key features:**
- `DEFAULT_THEMES` - 23 theme objects (aura, dracula, gruvbox, etc.)
- `resolveTheme()` - Resolves theme colors for dark/light mode
- `generateSystem()` - Creates theme from terminal ANSI palette
- `generateGrayScale()` - Smart grayscale generation
- `generateSyntax()` - Builds syntax highlighting rules

**Color categories (40+ colors):**
```
Primary:     primary, secondary, accent
Status:      error, warning, success, info
UI:          text, textMuted, background, backgroundPanel, backgroundElement
Borders:     border, borderActive, borderSubtle
Diff:        diffAdded, diffRemoved, diffContext, diffHunkHeader
Markdown:    markdownText, markdownHeading, markdownLink, markdownCode
Syntax:      syntaxComment, syntaxKeyword, syntaxFunction, syntaxVariable, syntaxString
```

**Learn this for:** Color theming, syntax highlighting, dark/light mode support

---

### 5. Routing & Navigation
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/context/route.tsx`

**What it does:**
- Simple route state management
- Two route types: home | session
- Navigate between screens
- Stores route in env variable

**Route types:**
```typescript
type HomeRoute = { type: "home" }
type SessionRoute = { type: "session"; sessionID: string }
type Route = HomeRoute | SessionRoute
```

**Learn this for:** Simple routing pattern, SolidJS store usage

---

### 6. Dialog/Modal System
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/ui/dialog.tsx`

**What it does:**
- Base dialog component with overlay
- Stack-based dialog management
- ESC to close
- Focus restoration after close

**Methods:**
- `clear()` - Close all dialogs
- `replace(element)` - Replace top dialog with new one
- Stack property - Array of dialogs

**Learn this for:** Modal management pattern, focus handling

---

### 7. Dialog Select (List Picker)
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/ui/dialog-select.tsx`

**What it does:**
- Fuzzy searchable list
- Category grouping
- Footer text
- Custom keybinds per option
- Highlighted current selection

**Learn this for:** Advanced dialog pattern, fuzzy search, custom keybinds

---

### 8. Keybind System
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/context/keybind.tsx`

**What it does:**
- Parses keybind strings (e.g., "ctrl+k", "<leader>s")
- Leader key support (2-second timeout)
- Fuzzy matching with modifiers (ctrl, shift, meta)
- Prints human-readable keybind strings

**Key methods:**
- `match(keyName, evt)` - Check if event matches keybind
- `print(keyName)` - Get readable string
- `parse(evt)` - Normalize key event

**Learn this for:** Keybind handling, Vim-style leader key

---

### 9. Data Sync (Remote State)
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/context/sync.tsx`

**What it does:**
- Fetches initial state from backend
- Keeps local copy in sync with backend
- Provides accessors for sessions, messages, config
- Listens for WebSocket events

**Data structure (simplified):**
```
sync.data = {
  session: Session[],
  message: Record<sessionID, Message[]>,
  part: Record<messageID, Part[]>,
  provider: Provider[],
  config: Config,
  mcp: Record<name, MCPStatus>,
  todo: Record<sessionID, Todo[]>,
  session_diff: Record<sessionID, FileDiff[]>,
}
```

**Learn this for:** Real-time data synchronization pattern

---

### 10. Local User Preferences
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/context/local.tsx`

**What it does:**
- Tracks current model + agent selection
- Recent models list for cycling
- Color mapping for agent names
- Parsed model display (provider/model split)

**Methods:**
- `model.set()` - Change model
- `model.cycle()` - Switch to next/previous model
- `model.current()` - Get selected model
- `agent.set()` - Change agent
- `agent.current()` - Get selected agent

**Learn this for:** User preference management

---

### 11. Command Palette
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/component/dialog-command.tsx`

**What it does:**
- Fuzzy searchable command list
- Dynamic command registration
- Keybind-based triggering
- Category grouping
- Conditional enable/disable

**Command structure:**
```typescript
type Command = {
  title: string
  value: string
  keybind?: keyof KeybindsConfig
  category: string
  disabled?: boolean
  onSelect: (dialog: DialogContext) => void
}
```

**Learn this for:** Command palette pattern, dynamic keybind management

---

### 12. Toast Notifications
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/ui/toast.tsx`

**What it does:**
- Non-modal notifications
- Variants: info, success, warning, error
- Auto-hide with duration
- Stack multiple toasts

**Learn this for:** Notification system pattern

---

### 13. Home Screen
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/routes/home.tsx`

**What it does:**
- Logo display
- Help hints for common operations
- Prompt input for new session
- MCP server status indicator

**Learn this for:** Landing page pattern

---

### 14. Session Sidebar
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/routes/session/sidebar.tsx`

**What it does:**
- Collapsible sections: MCP, LSP, Todo, Modified Files
- Context token usage display
- Cost calculation
- File diff stats

**Learn this for:** Collapsible panel pattern, info display

---

### 15. Session Header
**File:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/cli/cmd/tui/routes/session/header.tsx`

**What it does:**
- Session title
- Share URL or share prompt
- Context tokens and cost
- Share link display

**Learn this for:** Header/info bar pattern

---

## Supporting Files

### Utilities
- `util/editor.ts` - External $EDITOR integration
- `util/clipboard.ts` - OSC 52 + clipboardy fallback
- `util/terminal.ts` - Terminal capability detection

### Context Helpers
- `context/helper.tsx` - Simple context creation pattern
- `context/args.tsx` - CLI argument parsing
- `context/local.tsx` - User preferences
- `context/kv.tsx` - Local key-value storage
- `context/sdk.tsx` - API client wrapper
- `context/exit.tsx` - App exit handling

### Dialog Variants
- `ui/dialog-alert.tsx` - Simple message
- `ui/dialog-confirm.tsx` - Yes/no prompt
- `ui/dialog-prompt.tsx` - Text input
- `ui/dialog-help.tsx` - Keybinds reference
- `ui/shimmer.tsx` - Loading animation

### Components
- `component/border.tsx` - Border styling (SplitBorder)
- `component/logo.tsx` - ASCII art
- `component/dialog-model.tsx` - Model picker
- `component/dialog-agent.tsx` - Agent picker
- `component/dialog-theme-list.tsx` - Theme picker
- `component/dialog-session-list.tsx` - Session picker
- `component/dialog-session-rename.tsx` - Rename form
- `component/prompt/autocomplete.tsx` - Suggestion system
- `component/prompt/history.tsx` - History management

---

## Study Path

### Beginner (TUI Fundamentals)
1. `app.tsx` - See provider structure
2. `context/route.tsx` - Understand routing
3. `routes/home.tsx` - Simple screen
4. `ui/toast.tsx` - Simple notification
5. `ui/dialog.tsx` - Modal system

### Intermediate (Data & Input)
6. `context/theme.tsx` - Theming system
7. `context/sync.tsx` - Data synchronization
8. `component/prompt/index.tsx` - Text input
9. `routes/session/header.tsx` - Info display
10. `component/dialog-command.tsx` - Command palette

### Advanced (Complex Rendering)
11. `routes/session/index.tsx` - Message rendering
12. `context/local.tsx` - State management
13. `context/keybind.tsx` - Keyboard handling
14. `routes/session/sidebar.tsx` - Collapsible UI

---

## Key Concepts to Understand

### OpenTUI Primitive Elements
- `<box>` - Container with flexbox properties
- `<text>` - Styled text with colors/attributes
- `<textarea>` - Multi-line input with extmarks
- `<code>` - Syntax-highlighted code blocks
- `<scrollbox>` - Scrollable container

### State Management
- `createStore()` - Solid.js reactive store
- `produce()` - Immutable updates
- `createMemo()` - Computed values
- `createEffect()` - Side effects
- `createSignal()` - Simple reactive state

### Context API
- `createContext()` - Define context
- `useContext()` - Consume context
- `Provider` - Wrap children with context

### Component Patterns
- Render props (functions that return elements)
- Dynamic components via `<Dynamic>`
- Conditional rendering with `<Show>`, `<Switch>`
- List rendering with `<For>`

---

## Files by Responsibility

### Routing & Navigation
- `context/route.tsx`
- `routes/home.tsx`
- `routes/session/index.tsx`

### State Management
- `context/local.tsx` - User preferences
- `context/sync.tsx` - Remote data
- `context/theme.tsx` - Colors/styles
- `context/keybind.tsx` - Keyboard config
- `context/kv.tsx` - Persistent storage
- `context/sdk.tsx` - API client

### User Interaction
- `component/prompt/index.tsx` - Text input
- `component/prompt/autocomplete.tsx` - Suggestions
- `context/keybind.tsx` - Keyboard handling
- `component/dialog-command.tsx` - Command palette

### UI Display
- `routes/session/index.tsx` - Messages + tools
- `routes/session/header.tsx` - Session info
- `routes/session/sidebar.tsx` - Panels
- `ui/dialog.tsx` - Modal system
- `ui/dialog-select.tsx` - List picker
- `ui/toast.tsx` - Notifications
- `context/theme.tsx` - Color theming

### Utilities
- `util/editor.ts` - External editor
- `util/clipboard.ts` - Copy/paste
- `util/terminal.ts` - Terminal info

