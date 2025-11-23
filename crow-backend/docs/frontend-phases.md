# Frontend Implementation Phases

Detailed breakdown of the crow frontend build plan.

---

## Phase 0: Absolute MVP (3-5 days)

**Goal:** Basic chat interface that can communicate with crow backend.

### Tasks

#### Day 1: Project Setup
- [ ] Initialize Vite + React + TypeScript project
- [ ] Configure Tailwind CSS
- [ ] Set up ESLint + Prettier
- [ ] Create basic project structure
- [ ] Environment variables for API URL

#### Day 2: API Layer
- [ ] Create API client module
- [ ] Implement session endpoints (list, create, get)
- [ ] Implement message endpoints (list, send)
- [ ] Add error handling and types

#### Day 3: Core UI
- [ ] App shell with dark theme
- [ ] Session selector dropdown
- [ ] Basic chat message list
- [ ] Message input with send button
- [ ] User/Assistant message styling

#### Day 4: Streaming
- [ ] SSE connection to `/session/:id/message/stream`
- [ ] Real-time message streaming display
- [ ] Text delta handling
- [ ] Loading/sending states
- [ ] Abort button

#### Day 5: File Viewing
- [ ] Basic CodeMirror 6 setup
- [ ] Single file view
- [ ] Language detection
- [ ] Read-only display
- [ ] Minimal file tree (hardcoded)

### Deliverables
- Working React app that connects to crow
- Can create sessions and send messages
- Streaming responses display in real-time
- Basic code viewing capability

### Dependencies
```json
{
  "dependencies": {
    "react": "^18.3.0",
    "react-dom": "^18.3.0",
    "@codemirror/state": "^6.4.0",
    "@codemirror/view": "^6.26.0",
    "@codemirror/language": "^6.10.0",
    "@codemirror/commands": "^6.5.0",
    "@codemirror/lang-javascript": "^6.2.0",
    "@codemirror/theme-one-dark": "^6.1.0"
  },
  "devDependencies": {
    "typescript": "^5.4.0",
    "vite": "^5.4.0",
    "@types/react": "^18.3.0",
    "tailwindcss": "^3.4.0",
    "postcss": "^8.4.0",
    "autoprefixer": "^10.4.0"
  }
}
```

---

## Phase 1: Core Functionality (5-7 days)

**Goal:** Fully functional agent interface with file operations.

### Tasks

#### Session Management
- [ ] Session list sidebar
- [ ] Create new session button
- [ ] Delete session with confirmation
- [ ] Session title editing
- [ ] Fork session functionality
- [ ] Persist selected session in localStorage

#### Message Features
- [ ] Message timestamps
- [ ] Tool call display (expandable)
- [ ] Tool status indicators (pending, running, completed, error)
- [ ] Thinking/reasoning display
- [ ] Copy message to clipboard
- [ ] Syntax highlighting in code blocks

#### File Tree
- [ ] Dynamic file tree loading
- [ ] Directory expansion/collapse
- [ ] File type icons
- [ ] Git status indicators
- [ ] Search/filter files
- [ ] Refresh button

#### Editor Enhancements
- [ ] Tab bar for multiple files
- [ ] Close tabs
- [ ] Unsaved indicator
- [ ] Line numbers
- [ ] Language-specific highlighting
- [ ] More languages (Rust, Python, JSON, YAML, etc.)

#### Diff Viewer
- [ ] Show file changes from agent
- [ ] Side-by-side diff
- [ ] Line-by-line additions/deletions
- [ ] Accept/reject changes

#### Approve/Reject Workflow
- [ ] Permission dialog modal
- [ ] Tool name and arguments display
- [ ] Allow/Deny buttons
- [ ] "Always allow" option
- [ ] Risk level indicator

### Additional Dependencies
```json
{
  "react-arborist": "^3.4.0",
  "@codemirror/merge": "^6.6.0",
  "@codemirror/lang-rust": "^6.0.0",
  "@codemirror/lang-python": "^6.1.0",
  "@codemirror/lang-json": "^6.0.0",
  "@codemirror/lang-markdown": "^6.2.0",
  "lucide-react": "^0.400.0"
}
```

### Backend Requirements
For Phase 1, crow backend needs:
- [ ] `GET /event` - Global SSE stream
- [ ] Permission system
- [ ] `POST /session/:id/revert`

---

## Phase 2: Polish (5-7 days)

**Goal:** Production-ready UI with good UX.

### Tasks

#### Multi-file Editing
- [ ] Drag-and-drop tab reordering
- [ ] Split panes (horizontal/vertical)
- [ ] File save (Ctrl+S)
- [ ] Dirty file indicator

#### Search
- [ ] Global text search
- [ ] File name search
- [ ] Search results panel
- [ ] Click to open result

#### Settings UI
- [ ] Settings modal
- [ ] API URL configuration
- [ ] Model selector
- [ ] Theme toggle (light/dark)
- [ ] Keybindings display

#### Keyboard Shortcuts
- [ ] Ctrl+N - New session
- [ ] Ctrl+Enter - Send message
- [ ] Ctrl+P - Quick file open
- [ ] Ctrl+Shift+F - Global search
- [ ] Escape - Cancel/close

#### Responsive Design
- [ ] Mobile-friendly layout
- [ ] Collapsible panels
- [ ] Touch support

#### Loading States
- [ ] Skeleton loaders
- [ ] Progress indicators
- [ ] Error states with retry

#### Accessibility
- [ ] Keyboard navigation
- [ ] ARIA labels
- [ ] Focus management
- [ ] Screen reader support

### Additional Dependencies
```json
{
  "@radix-ui/react-dialog": "^1.0.0",
  "@radix-ui/react-dropdown-menu": "^2.0.0",
  "@radix-ui/react-tabs": "^1.0.0",
  "@tanstack/react-virtual": "^3.5.0",
  "cmdk": "^1.0.0"
}
```

---

## Phase 3: Advanced Features (Future)

**Goal:** Extended functionality for power users.

### Potential Features

#### Terminal Pane
- [ ] Integrated terminal
- [ ] Shell command execution via `/session/:id/shell`
- [ ] Terminal history
- [ ] Multiple terminal tabs

#### Git Integration
- [ ] Commit viewer
- [ ] Diff against HEAD
- [ ] Stage/unstage files
- [ ] Commit from UI

#### Todo List
- [ ] Display agent's task list
- [ ] Check off completed items
- [ ] Add custom todos
- [ ] Priority sorting

#### Session History
- [ ] Timeline view
- [ ] Branch visualization
- [ ] Restore from point

#### Collaboration
- [ ] Share session link
- [ ] Real-time collaboration
- [ ] Comments on code

#### AI Enhancements
- [ ] Model comparison
- [ ] Cost tracking
- [ ] Token usage display
- [ ] Prompt templates

---

## Milestone Definitions

### MVP Complete (Phase 0)
- ✅ Can create and switch sessions
- ✅ Can send messages and receive streaming responses
- ✅ Basic file viewing works

### Beta Ready (Phase 1)
- ✅ Full session management
- ✅ File tree with dynamic loading
- ✅ Multi-file tabs in editor
- ✅ Diff viewer for agent changes
- ✅ Permission approve/reject workflow
- ✅ Message revert functionality

### Production Ready (Phase 2)
- ✅ Polished UI with consistent styling
- ✅ All keyboard shortcuts working
- ✅ Error handling and recovery
- ✅ Responsive on mobile
- ✅ Accessible

---

## Estimated Timeline

| Phase | Duration | Cumulative |
|-------|----------|------------|
| Phase 0 | 3-5 days | 3-5 days |
| Phase 1 | 5-7 days | 8-12 days |
| Phase 2 | 5-7 days | 13-19 days |

**Total: ~2-3 weeks for production-ready frontend**

---

## Risk Factors

### Backend Dependencies
Phase 1 requires backend enhancements that may take additional time:
- Global event stream (`GET /event`)
- Permission system
- Message revert

**Mitigation:** Start backend work in parallel with Phase 0 frontend.

### CodeMirror Complexity
CodeMirror 6 has a steep learning curve for advanced features.

**Mitigation:** Keep editor features minimal in Phase 0/1, enhance in Phase 2.

### State Management
Complex state with multiple concurrent updates (streaming, permissions, files).

**Mitigation:** Clear reducer structure, thorough testing.

---

## Testing Strategy

### Unit Tests
- API client functions
- State reducer logic
- Utility functions

### Integration Tests
- SSE connection handling
- Message flow end-to-end
- Permission workflow

### E2E Tests (Playwright)
- Full user flows
- Error scenarios
- Responsive testing

### Manual Testing Checklist
- [ ] Create session
- [ ] Send message
- [ ] Abort message
- [ ] View streaming response
- [ ] Open file from tree
- [ ] View diff
- [ ] Accept change
- [ ] Reject change
- [ ] Switch sessions
- [ ] Delete session
