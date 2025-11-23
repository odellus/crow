# OpenCode TUI → Dioxus Web Migration Checklist

## Overview
This document provides a structured plan for replicating OpenCode's TUI implementation in a Dioxus web app.

---

## Phase 1: Foundation & Architecture (Weeks 1-2)

### Core Infrastructure
- [ ] Set up Dioxus project with routing
- [ ] Implement context provider system (similar to OpenCode's nested providers)
- [ ] Create theme/color system (replicate 40+ color palette)
- [ ] Set up global state management (Signals + Contexts)
- [ ] Implement route navigation (home vs session views)

**Files to Reference:**
- `app.tsx` - Provider structure
- `context/theme.tsx` - Color system
- `context/route.tsx` - Routing pattern

### Key Deliverables:
- [ ] Root component with provider stack
- [ ] Theme context with 23+ theme definitions
- [ ] Dark/light mode toggle
- [ ] Route navigation working

---

## Phase 2: State Management (Weeks 2-3)

### Global State Providers
- [ ] Route provider (home, session navigation)
- [ ] Sync provider (remote data synchronization)
- [ ] Local preferences provider (model, agent, sidebar state)
- [ ] Dialog/modal manager
- [ ] Keybind system with leader key support
- [ ] Command palette system

**Files to Reference:**
- `context/sync.tsx` - Data synchronization
- `context/local.tsx` - User preferences
- `context/keybind.tsx` - Keyboard handling
- `ui/dialog.tsx` - Modal stack
- `component/dialog-command.tsx` - Command palette

### Key Deliverables:
- [ ] WebSocket connection to backend
- [ ] Message/session data structures
- [ ] User preference persistence
- [ ] Modal stack management (open/close/replace)
- [ ] Keybind matching with leader key (2-sec timeout)
- [ ] Command palette with dynamic registration

---

## Phase 3: UI Components - Basic (Weeks 3-4)

### Simple Components
- [ ] Toast notifications (info/success/error/warning)
- [ ] Dialog alerts (message display)
- [ ] Dialog confirm (yes/no prompt)
- [ ] Dialog prompt (text input)
- [ ] Loading shimmer animation
- [ ] Logo/branding

**Files to Reference:**
- `ui/toast.tsx`
- `ui/dialog-alert.tsx`
- `ui/dialog-confirm.tsx`
- `ui/dialog-prompt.tsx`
- `ui/shimmer.tsx`
- `component/logo.tsx`

### Key Deliverables:
- [ ] Toast system with auto-dismiss
- [ ] Modal backdrop with ESC handling
- [ ] Dialog size variants (medium/large)
- [ ] Loading indicator

---

## Phase 4: Main Screens (Weeks 4-5)

### Home Screen
- [ ] Display logo
- [ ] Show help hints (keybinds for common actions)
- [ ] MCP server connection indicators
- [ ] Input field for new prompt
- [ ] Session creation

**Files to Reference:**
- `routes/home.tsx`

### Session Screen - Layout
- [ ] Two-column layout (narrow <120px: single; wide >120px: with sidebar)
- [ ] Header: session title + context/cost info
- [ ] Messages scrollable area (sticky to bottom)
- [ ] Prompt input at bottom
- [ ] Sidebar: MCP, LSP, Todo, Modified Files (collapsible)

**Files to Reference:**
- `routes/session/index.tsx`
- `routes/session/header.tsx`
- `routes/session/sidebar.tsx`

### Key Deliverables:
- [ ] Responsive layout (mobile/tablet/desktop)
- [ ] Scrollable message area
- [ ] Auto-scroll to bottom on new messages
- [ ] Collapsible sidebar sections

---

## Phase 5: Message Rendering System (Weeks 5-7)

### Message Components
- [ ] User message: text + file attachments with badges
- [ ] Assistant message dispatcher
- [ ] Text part: markdown rendering
- [ ] Reasoning part: thinking/internal monologue
- [ ] Error display

**Files to Reference:**
- `routes/session/index.tsx` - UserMessage, AssistantMessage, TextPart

### Tool Renderers (Tool Registry Pattern)
- [ ] Bash: command + output
- [ ] Read: file path indicator
- [ ] Write: filename + syntax-highlighted code + diagnostics
- [ ] Edit: split diff view (old/new side-by-side)
- [ ] Glob: pattern + match count
- [ ] Grep: pattern + match count
- [ ] List: directory listing
- [ ] Task: subagent delegation
- [ ] Patch: patch application
- [ ] WebFetch: URL display
- [ ] TodoWrite: todo items with status

**Files to Reference:**
- `routes/session/index.tsx` - ToolRegistry pattern

### Key Deliverables:
- [ ] Tool registry system (dynamic dispatch)
- [ ] Syntax highlighting (highlight.js/Prism)
- [ ] Diff viewer (unified format)
- [ ] Streaming indicator
- [ ] Tool error messages
- [ ] Permission prompts UI

---

## Phase 6: User Input System (Weeks 6-7)

### Prompt Input Component
- [ ] Textarea with multiline support (min 1, max 6 lines)
- [ ] Syntax highlighting (markdown mode)
- [ ] History navigation (up/down at start/end)
- [ ] Virtual badges for file attachments (`[Image 1]`, `[File: path]`)
- [ ] Image/file paste support
- [ ] Autocomplete suggestions
- [ ] Shell mode (prefix with `!`)

**Files to Reference:**
- `component/prompt/index.tsx` - Main prompt component
- `component/prompt/autocomplete.tsx` - Suggestions
- `component/prompt/history.tsx` - History management

### Key Deliverables:
- [ ] Multiline textarea with custom styling
- [ ] Virtual text overlays for files (absolutely positioned badges)
- [ ] File drag-drop support
- [ ] Clipboard paste handling (OSC 52 equivalent)
- [ ] Command history with navigation
- [ ] Context-aware autocomplete
- [ ] Shell mode toggle

---

## Phase 7: Dialog Systems (Week 7-8)

### Fuzzy Search Dialog
- [ ] Searchable list with `/` activation
- [ ] Category grouping
- [ ] Footer text (timestamps, status)
- [ ] Keyboard navigation (arrow keys, j/k)
- [ ] Custom keybinds per option
- [ ] Highlighted current selection

**Files to Reference:**
- `ui/dialog-select.tsx`

### Dialog Variants
- [ ] Session list (with delete/rename keybinds)
- [ ] Model picker
- [ ] Agent picker
- [ ] Theme selector
- [ ] Help/keybinds reference
- [ ] Session rename form
- [ ] Status/info display

**Files to Reference:**
- `component/dialog-session-list.tsx`
- `component/dialog-model.tsx`
- `component/dialog-agent.tsx`
- `component/dialog-theme-list.tsx`
- `ui/dialog-help.tsx`
- `component/dialog-session-rename.tsx`
- `component/dialog-status.tsx`

### Key Deliverables:
- [ ] Fuzzy search with fuse.js or similar
- [ ] Dialog stack with focus restoration
- [ ] Modal animation (fade in/out)
- [ ] Keyboard-first navigation

---

## Phase 8: Advanced Features (Weeks 8-10)

### Session Management
- [ ] Undo/revert messages
- [ ] Redo (restore reverted messages)
- [ ] Session renaming
- [ ] Session sharing (copy URL)
- [ ] Session compacting/summarization
- [ ] Sidebar showing modified files (git diff stats)

**Files to Reference:**
- `routes/session/index.tsx` - Command registration

### Message Operations
- [ ] Copy last assistant message
- [ ] Copy entire session transcript
- [ ] Export session to file
- [ ] Jump to message (timeline)
- [ ] Message editing
- [ ] Code concealment toggle

**Files to Reference:**
- `routes/session/index.tsx` - Command implementations

### Sidebar Features
- [ ] MCP server list with status
- [ ] LSP connections display
- [ ] Todo items (collapsible)
- [ ] Modified files list with diff stats
- [ ] Context usage display

**Files to Reference:**
- `routes/session/sidebar.tsx`

### Key Deliverables:
- [ ] Revert/redo UI with diff display
- [ ] File export functionality
- [ ] Git integration for diff display
- [ ] Todo management UI

---

## Phase 9: Polish & Optimization (Weeks 10-11)

### Performance
- [ ] Virtual scrolling (if >1000 messages)
- [ ] Code splitting for large components
- [ ] Image lazy loading
- [ ] Syntax highlighting caching
- [ ] Memoization of expensive computations

### Accessibility
- [ ] Semantic HTML
- [ ] ARIA labels for dialogs
- [ ] Keyboard navigation throughout
- [ ] Focus indicators
- [ ] Color contrast (WCAG AA)

### Responsive Design
- [ ] Mobile layout (single column)
- [ ] Tablet layout (sidebar hidden by default)
- [ ] Desktop layout (sidebar visible)
- [ ] Touch support (if applicable)

### Browser Compatibility
- [ ] Modern browsers (Chrome, Firefox, Safari, Edge)
- [ ] WebSocket support
- [ ] Clipboard API
- [ ] CSS Grid/Flexbox support

### Key Deliverables:
- [ ] Performance benchmarks
- [ ] Lighthouse score >90
- [ ] Mobile responsiveness verified
- [ ] Cross-browser testing

---

## Phase 10: Documentation & Launch (Week 11-12)

### Documentation
- [ ] Component API documentation
- [ ] Theme customization guide
- [ ] Keybind configuration docs
- [ ] Architecture overview
- [ ] Deployment guide

### Testing
- [ ] Unit tests for state management
- [ ] Integration tests for key flows
- [ ] E2E tests for user workflows
- [ ] Visual regression testing

### Launch Preparation
- [ ] Performance optimization
- [ ] Error handling & logging
- [ ] Analytics integration
- [ ] User feedback system
- [ ] Release notes

---

## Feature Comparison Matrix

| Feature | TUI | Web | Priority |
|---------|-----|-----|----------|
| **Core Messaging** | ✓ | [ ] | P0 |
| **Tool Rendering** | ✓ | [ ] | P0 |
| **User Input** | ✓ | [ ] | P0 |
| **Theme System** | ✓ | [ ] | P0 |
| **Sessions Management** | ✓ | [ ] | P1 |
| **Dialogs/Modals** | ✓ | [ ] | P1 |
| **Keybinds** | ✓ | [ ] | P1 |
| **Sidebar** | ✓ | [ ] | P1 |
| **Undo/Redo** | ✓ | [ ] | P2 |
| **Code Concealment** | ✓ | [ ] | P2 |
| **Virtual Annotations** | ✓ | [ ] | P2 |
| **MCP Integration** | ✓ | [ ] | P3 |
| **LSP Integration** | ✓ | [ ] | P3 |

---

## Architecture Decisions to Make

### 1. Styling Approach
- [ ] **Tailwind CSS** (recommended for speed)
- [ ] **CSS Modules** (for component encapsulation)
- [ ] **CSS-in-JS** (emotion, styled-components)
- [ ] **CSS Grid/Flexbox** (raw CSS)

### 2. Syntax Highlighting
- [ ] **highlight.js** (simple, many languages)
- [ ] **Prism.js** (customizable)
- [ ] **Shiki** (VS Code engine, slow for web)
- [ ] **Custom (tree-sitter WASM)** (complex)

### 3. Virtual Scrolling
- [ ] **react-window / tanstack/react-virtual** equivalent for Dioxus
- [ ] **Intersection Observer API** (manual)
- [ ] **CSS containment** (browser native)

### 4. State Management
- [ ] **Dioxus Signals** (built-in, recommended)
- [ ] **Redux** (if team familiar)
- [ ] **Zustand** (JavaScript option)
- [ ] **Context alone** (simpler but less efficient)

### 5. WebSocket Library
- [ ] **web-sys** (low-level)
- [ ] **gloo-net** (Dioxus ecosystem)
- [ ] **tungstenite-rs** (if native support needed)
- [ ] **tokio-tungstenite** (full-featured)

---

## Testing Strategy

### Unit Tests (Components)
```
tests/
├── components/
│   ├── prompt.rs
│   ├── message.rs
│   └── dialog.rs
├── contexts/
│   ├── theme.rs
│   ├── sync.rs
│   └── keybind.rs
└── utils/
    ├── formatting.rs
    └── keybind_parser.rs
```

### Integration Tests
- User flow: Login → Create session → Send message → View response
- Session flow: Load session → Edit → Save → Export
- Dialog flow: Open command palette → Search → Execute

### E2E Tests (if using Playwright/Cypress)
- Full user workflows
- Cross-browser compatibility
- Mobile responsiveness

---

## Success Criteria

### Functionality
- [ ] All core features from TUI work in web
- [ ] Zero functional regressions
- [ ] Command palette fully operational
- [ ] All tool renderers working
- [ ] Real-time sync with backend

### Performance
- [ ] Page load <2 seconds
- [ ] Message render <100ms
- [ ] Smooth scrolling (60 FPS)
- [ ] No layout shifts (CLS <0.1)

### User Experience
- [ ] Keyboard-first (TUI parity)
- [ ] Mouse support (web enhancement)
- [ ] Touch-friendly (mobile)
- [ ] Accessible (WCAG AA)

### Quality
- [ ] No console errors
- [ ] Comprehensive test coverage >80%
- [ ] Lighthouse score ≥90
- [ ] No memory leaks

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Virtual text overlays complexity | Start with simple badges, iterate on positioning |
| Real-time sync lag | Optimistic UI updates, error recovery |
| Theme color compatibility | Use CSS variables, test across browsers |
| Keybind conflicts | Standardize keybind format, document conflicts |
| Large message history | Implement virtual scrolling early |
| Syntax highlighting performance | Lazy load, cache results, worker threads |

---

## Next Steps

1. **Week 1:** Begin Phase 1 (Foundation)
2. **Weekly:** Review progress against checklist
3. **Every 2 weeks:** Checkpoint with stakeholders
4. **Mid-project:** Evaluate tool renderer complexity
5. **Final week:** Buffer for polish & bugfixes

---

## Resources to Reference

### OpenCode TUI Files
- Entry: `/tui/app.tsx` (provider structure)
- Messages: `/tui/routes/session/index.tsx` (rendering, commands)
- Input: `/tui/component/prompt/index.tsx` (textarea, virtual text)
- Theme: `/tui/context/theme.tsx` (color system)
- Dialog: `/tui/ui/dialog.tsx` (modal stack)
- Commands: `/tui/component/dialog-command.tsx` (dynamic commands)

### Dioxus Documentation
- https://dioxuslabs.com/learn/0.5/
- https://github.com/DioxusLabs/dioxus
- Signals guide
- Router guide
- CSS styling

### Web Libraries
- Syntax highlighting: highlight.js / Prism
- Fuzzy search: fuse.js
- UUID: uuid crate
- Date formatting: chrono + humantime
- Diff display: diff-match-patch

