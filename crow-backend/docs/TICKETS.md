# Crow Development Tickets

## Project Goal
Rust rewrite of OpenCode with a Dioxus web UI (not TUI).

---

## Epic: Core Infrastructure

### CROW-001: Fix WASM Runtime Error
**Priority:** P0 (Blocker)  
**Estimate:** 1-2 hours

**Problem:** `unreachable executed` error in WASM, likely hydration mismatch between SSR and client.

**Acceptance Criteria:**
- [ ] Web UI loads without console errors
- [ ] Sessions list renders correctly
- [ ] No hydration warnings in console

---

### CROW-002: Rename CrowStorage to Storage
**Priority:** P1  
**Estimate:** 30 min

**Problem:** Naming doesn't match OpenCode conventions.

**Acceptance Criteria:**
- [ ] `CrowStorage` renamed to `Storage` in all files
- [ ] All imports updated
- [ ] Tests pass
- [ ] Build succeeds

---

## Epic: Session Management

### CROW-003: Session Navigation
**Priority:** P1  
**Estimate:** 1-2 hours

**Problem:** Clicking sessions in sidebar does nothing.

**Acceptance Criteria:**
- [ ] Clicking session navigates to `/session/{id}`
- [ ] Session detail view loads correct session
- [ ] URL reflects current session
- [ ] Back button works

---

### CROW-004: Create New Session
**Priority:** P1  
**Estimate:** 1 hour

**Problem:** "New Session" button doesn't work.

**Acceptance Criteria:**
- [ ] Button calls `create_session` server function
- [ ] New session appears in sidebar
- [ ] Navigates to new session automatically
- [ ] Session persists to disk

---

### CROW-005: Delete Session
**Priority:** P2  
**Estimate:** 1 hour

**Acceptance Criteria:**
- [ ] Delete button/action on session
- [ ] Confirmation dialog
- [ ] Session removed from storage
- [ ] UI updates to reflect deletion

---

## Epic: Chat Interface

### CROW-006: Display Message History
**Priority:** P1  
**Estimate:** 2-3 hours

**Problem:** Session detail shows no messages.

**Acceptance Criteria:**
- [ ] Load messages for selected session
- [ ] Display user messages (right-aligned, blue)
- [ ] Display assistant messages (left-aligned, gray)
- [ ] Show timestamps
- [ ] Auto-scroll to bottom

---

### CROW-007: Message Input Box
**Priority:** P1  
**Estimate:** 1-2 hours

**Problem:** No way to send messages.

**Acceptance Criteria:**
- [ ] Text input at bottom of chat
- [ ] Send button (or Enter key)
- [ ] Calls `send_message` server function
- [ ] Input clears after send
- [ ] Loading state while processing

---

### CROW-008: Tool Output Rendering
**Priority:** P2  
**Estimate:** 3-4 hours

**Problem:** Tool results not displayed.

**Acceptance Criteria:**
- [ ] Render `Part::Tool` with tool name and status
- [ ] Collapsible tool output
- [ ] Syntax highlighting for code (Bash, Read results)
- [ ] Error state styling (red)
- [ ] Success state styling (green)

---

### CROW-009: Thinking Display
**Priority:** P3  
**Estimate:** 1 hour

**Acceptance Criteria:**
- [ ] Render `Part::Thinking` in italics/muted
- [ ] Collapsible by default
- [ ] Distinguishable from regular text

---

## Epic: Streaming & Real-time

### CROW-010: Streaming Responses
**Priority:** P2  
**Estimate:** 4-6 hours

**Problem:** Messages appear all at once after completion.

**Acceptance Criteria:**
- [ ] Server sends SSE/WebSocket stream
- [ ] Client renders tokens as they arrive
- [ ] Tool executions show progress
- [ ] Graceful handling of disconnects

---

## Epic: File Handling

### CROW-011: File References (@ mentions)
**Priority:** P3  
**Estimate:** 3-4 hours

**Acceptance Criteria:**
- [ ] Type `@` to trigger file picker
- [ ] Autocomplete file paths
- [ ] Attach file content to message
- [ ] Display attached files in message

---

## Epic: Configuration

### CROW-012: Provider Selection
**Priority:** P3  
**Estimate:** 2-3 hours

**Acceptance Criteria:**
- [ ] Settings page for LLM provider
- [ ] Support multiple providers (OpenAI, Anthropic, Moonshot)
- [ ] API key management
- [ ] Persist to config file

---

### CROW-013: Model Selection
**Priority:** P3  
**Estimate:** 1-2 hours

**Acceptance Criteria:**
- [ ] Dropdown to select model
- [ ] Show available models per provider
- [ ] Persist selection

---

## Epic: Polish & UX

### CROW-014: Loading States
**Priority:** P2  
**Estimate:** 2 hours

**Acceptance Criteria:**
- [ ] Skeleton loaders for sessions list
- [ ] Spinner for message sending
- [ ] Progress indicator for long operations

---

### CROW-015: Error Handling UI
**Priority:** P2  
**Estimate:** 2 hours

**Acceptance Criteria:**
- [ ] Toast notifications for errors
- [ ] Inline error messages
- [ ] Retry buttons where appropriate

---

### CROW-016: Keyboard Shortcuts
**Priority:** P3  
**Estimate:** 2 hours

**Acceptance Criteria:**
- [ ] `Cmd/Ctrl+N` - New session
- [ ] `Cmd/Ctrl+Enter` - Send message
- [ ] `Escape` - Cancel/close
- [ ] Show shortcuts in help modal

---

## Recommended Order

**Phase 1 - Make it Work:**
1. CROW-001 (Fix WASM error)
2. CROW-003 (Session navigation)
3. CROW-006 (Display messages)
4. CROW-007 (Message input)

**Phase 2 - Make it Complete:**
5. CROW-004 (Create session)
6. CROW-008 (Tool output)
7. CROW-002 (Rename storage)
8. CROW-014 (Loading states)

**Phase 3 - Make it Nice:**
9. CROW-010 (Streaming)
10. CROW-015 (Error handling)
11. CROW-005 (Delete session)
12. CROW-009 (Thinking display)

**Phase 4 - Make it Powerful:**
13. CROW-011 (File references)
14. CROW-012 (Provider selection)
15. CROW-013 (Model selection)
16. CROW-016 (Keyboard shortcuts)
