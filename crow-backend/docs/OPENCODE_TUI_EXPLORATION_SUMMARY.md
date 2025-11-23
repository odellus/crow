# OpenCode TUI Exploration - Complete Summary

## Project Completed: ✓

This document summarizes the comprehensive exploration of OpenCode's Terminal User Interface (TUI) implementation to inform Dioxus web app replication.

---

## Exploration Scope

**Target Directory:**
```
/opencode/packages/opencode/src/cli/cmd/tui/
```

**Total Files Analyzed:** 46 TypeScript/TSX files
**Total Lines of Code:** ~1,500+ lines in key files alone
**Framework:** @opentui/solid (v0.1.42)
**UI Library:** Solid.js (v catalog)
**Terminal Rendering:** ANSI/ASCII via @opentui/core

---

## Key Findings

### 1. TUI Framework: @opentui/solid

OpenCode uses a specialized terminal UI library built on Solid.js:

```json
{
  "@opentui/core": "0.1.42",
  "@opentui/solid": "0.1.42",
  "solid-js": "catalog:"
}
```

**Why @opentui?**
- Low-level terminal control (full ANSI/cursor manipulation)
- Component-based (like React/Dioxus)
- Supports advanced features: extmarks (virtual text), scrolling, mouse events
- Syntax highlighting via SyntaxStyle
- Real-time rendering at 60 FPS

### 2. Architecture Pattern: Nested Context Providers

OpenCode uses 11 stacked context providers:

```
ArgsProvider
  ExitProvider
    KVProvider
      ToastProvider
        RouteProvider
          SDKProvider
            SyncProvider
              ThemeProvider
                LocalProvider
                  KeybindProvider
                    DialogProvider
                      CommandProvider
                        PromptHistoryProvider
                          <App />
```

Each provider handles one concern (state, effects, subscriptions).

### 3. State Management: Solid.js Stores

Uses `createStore()` + `produce()` for immutable updates:

```typescript
const [store, setStore] = createStore({
  value: "initial",
  nested: { flag: false }
})

// Access
console.log(store.value)

// Update
setStore("value", "new")
setStore(produce(draft => {
  draft.nested.flag = true
}))
```

### 4. Two Core Routes

- **Home:** Logo, help hints, MCP status, new session creation
- **Session:** Full conversation view with messages, tools, sidebar

### 5. Message Rendering via Tool Registry

Tool outputs rendered dynamically via a self-registering pattern:

```typescript
ToolRegistry.register({
  name: "bash",
  container: "block",
  render: (props) => /* JSX component */
})
```

Supports 11 tool types: bash, read, write, edit, glob, grep, list, task, patch, webfetch, todowrite

### 6. Advanced Input System

Prompt component features:
- **Virtual Annotations (Extmarks)** - Files shown as `[Image 1]` badges without consuming text
- **Multiline Support** - Min 1, max 6 lines with history navigation
- **Syntax Highlighting** - Markdown mode aware
- **Paste Handling** - Image drag-drop, file references, OSC 52 clipboard
- **Autocomplete** - Context-aware suggestions
- **Shell Mode** - Prefix with `!` for direct bash

### 7. Dialog Stack System

Modal management with:
- Stack-based dialogs (only top visible)
- ESC to close with cleanup
- Focus restoration after close
- Size variants (medium/large)

### 8. Theme System

**40+ Color Properties:**
- 12 primary/status colors (primary, secondary, accent, error, warning, success, info)
- 12 UI colors (text, background, borders)
- 12 diff colors (added, removed, context)
- 4 markdown colors
- 8 syntax colors

**Features:**
- 23 built-in themes (dracula, gruvbox, nord, tokyonight, etc.)
- Dark/light mode variants
- Custom theme loading from `~/.opencode/config/themes/`
- Terminal palette generation

### 9. Keybind System

Sophisticated keyboard handling:
- **Leader Key** - Vim-style prefix key (2-second timeout)
- **Modifier Support** - Ctrl, Shift, Meta
- **Keybind Matching** - Fuzzy matching with custom notation
- **Printable Format** - Converts to readable strings for UI display

### 10. Real-time Data Sync

`SyncProvider` maintains local mirror of backend data:
- Sessions, messages, parts, providers, config
- MCP server status, LSP connections, todo items
- Session diffs (git tracking)
- WebSocket event listeners

---

## Generated Documentation Files

Four comprehensive documents created in project root:

### 1. **OPENCODE_TUI_ANALYSIS.md** (Primary Reference)
- 11 sections covering framework, architecture, UI patterns
- Complete file structure (46 files organized by purpose)
- Architecture patterns with code examples
- UI component breakdown
- Differences between TUI and web
- Implementation decisions
- Theme system deep dive
- Keyboard interaction model
- Performance considerations
- Recommended Dioxus architecture

### 2. **OPENCODE_TUI_FILE_REFERENCE.md** (Quick Reference)
- File-by-file breakdown (15 core files explained)
- What each file does
- Key functions/components
- Learn objectives per file
- Study path (beginner → intermediate → advanced)
- Files organized by responsibility
- 46 total files catalogued

### 3. **OPENCODE_TUI_CODE_PATTERNS.md** (Implementation Guide)
- 8 major code patterns with OpenCode examples + Dioxus equivalents:
  1. Context Provider Structure
  2. Message Rendering with Tool Registry
  3. Modal/Dialog Stack System
  4. Reactive Input with Virtual Annotations
  5. Keybind Matching with Leader Key
  6. Async Provider Initialization
  7. Computed State with Memoization
  8. Command Palette Dynamic Registration

### 4. **TUI_TO_DIOXUS_MIGRATION_CHECKLIST.md** (Project Plan)
- 10-phase implementation plan (12 weeks)
- Phase-by-phase tasks with file references
- Feature comparison matrix
- Architecture decision framework
- Testing strategy
- Success criteria
- Risk mitigation
- Resource links

---

## Key Statistics

| Metric | Value |
|--------|-------|
| Total TUI Files | 46 |
| Framework | @opentui/solid |
| Context Providers | 11 |
| Tool Renderers | 11 |
| Theme Colors | 40+ |
| Dialog Components | 8 |
| UI Primitives | 5 (box, text, textarea, code, scrollbox) |
| Largest Component | session/index.tsx (1564 lines) |
| Keybind Variants | 20+ |
| Built-in Themes | 23 |
| Command Types | 30+ |

---

## Most Important Files to Study (In Order)

1. **app.tsx** - Provider structure, global state, error handling
2. **routes/session/index.tsx** - Message rendering, tool registry, commands
3. **component/prompt/index.tsx** - Input system, virtual text, extmarks
4. **context/theme.tsx** - Color system, syntax highlighting
5. **context/keybind.tsx** - Keyboard handling, leader key
6. **ui/dialog.tsx** - Modal stack system
7. **context/sync.tsx** - Real-time data sync
8. **routes/session/sidebar.tsx** - Collapsible panels
9. **ui/dialog-select.tsx** - Fuzzy search patterns
10. **component/dialog-command.tsx** - Dynamic command registration

---

## UI Patterns Replicated for Dioxus

### Pattern 1: Nested Providers
Use same provider nesting structure with Dioxus Context API

### Pattern 2: Tool Registry
Dynamic component dispatch via HashMap of render functions

### Pattern 3: Modal Stack
Array-based dialog management with ESC handling

### Pattern 4: Virtual Text Overlays
Absolutely positioned badges over textarea (web equivalent of extmarks)

### Pattern 5: Keybind Matching
Event parsing + fuzzy matching with leader key timeout

### Pattern 6: Async Initialization
Effects + ready flags for async data loading

### Pattern 7: Memoized Derived Values
Computed signals for expensive calculations

### Pattern 8: Command Palette
Array-based command registration with dynamic updates

### Pattern 9: Theme Context
Context providing 40+ colors with dark/light variants

### Pattern 10: Real-time Sync
WebSocket listener updating local state store

---

## Architecture Decisions for Web Implementation

**Styling:**
- Recommend Tailwind CSS for rapid development
- CSS variables for theme switching
- CSS Grid/Flexbox for layouts

**Syntax Highlighting:**
- highlight.js for simplicity, many languages
- Prism as alternative with more control

**State Management:**
- Use Dioxus Signals (built-in)
- Context API for global state
- No external state library needed

**Input Handling:**
- HTML textarea with virtual badges overlay
- MouseDown/Paste event listeners
- Clipboard API for copy/paste

**Scrolling:**
- CSS overflow: auto with sticky positioning
- Intersection Observer for auto-scroll-to-bottom
- Virtual scrolling if >1000 messages

**WebSocket:**
- gloo-net (Dioxus ecosystem) for WebSocket
- Event listeners for real-time updates
- Optimistic UI updates

---

## Implementation Timeline

| Phase | Duration | Focus | Output |
|-------|----------|-------|--------|
| 1 | Weeks 1-2 | Foundation | Provider stack, routing, theme |
| 2 | Weeks 2-3 | State Management | Sync, preferences, dialogs, keybinds |
| 3 | Weeks 3-4 | Basic UI | Toasts, alerts, dialogs |
| 4 | Weeks 4-5 | Main Screens | Home, session layout |
| 5 | Weeks 5-7 | Message Rendering | Tool registry, renderers |
| 6 | Weeks 6-7 | Input System | Prompt, autocomplete, history |
| 7 | Weeks 7-8 | Advanced Dialogs | Search, selection, variants |
| 8 | Weeks 8-10 | Advanced Features | Undo/redo, sidebar, session ops |
| 9 | Weeks 10-11 | Polish | Performance, accessibility |
| 10 | Weeks 11-12 | Launch | Testing, docs, deployment |

---

## Critical Insights

### 1. Virtual Text Overlay Complexity
TUI's "extmarks" are sophisticated - virtual text that doesn't affect text content but can be positioned/styled. Dioxus equivalent: absolutely positioned badges over textarea.

### 2. Streaming Rendering
Code blocks support streaming updates as agent responds. Requires fine-grained reactivity. Use Dioxus Signals for partial updates.

### 3. Permission System
Tools can require user approval before execution. Backend validates, frontend shows permission prompt in message. Plan for this early.

### 4. Message History
The `undo/revert` system is complex - tracks diff, allows redo. Requires careful state management. Implement after core features.

### 5. Tool Registry Pattern
The registry system allows tools to self-register renderers. Highly extensible. Replicate as HashMap of render functions with TypeScript discriminated unions for safety.

### 6. Keybind Leader Key
The 2-second timeout for leader key activation is non-trivial. Requires timeout cleanup in effects.

### 7. Sidebar Collapse Logic
Sidebar visibility depends on terminal width AND user preference (show/hide/auto). Web equivalent: viewport width + persistent setting in localStorage.

### 8. Theme Complexity
40+ colors need to be carefully coordinated. Use CSS variables for easy switching. Consider providing dark/light mode generation.

### 9. Real-time Sync
Data can arrive out of order (messages, tools running). Sync provider handles merging. Plan WebSocket reconnection strategy.

### 10. Performance at Scale
Session view can have 100+ messages × 10+ tools each. Plan virtual scrolling from start.

---

## What Makes OpenCode's TUI Exceptional

1. **Keyboard-First Design** - Every feature accessible without mouse
2. **Rich Text Rendering** - Syntax highlighting, markdown, virtual annotations
3. **Async-Aware UI** - Streaming output, pending states, real-time sync
4. **Tool-Centric Architecture** - Each tool has custom UI, organized via registry
5. **Professional Modal System** - Stack-based with focus restoration
6. **Vim-Style Navigation** - Leader key, keybind customization
7. **Comprehensive Theme System** - 23 themes + custom support
8. **Message Streaming** - Incremental rendering as agent responds
9. **Permission Controls** - User approval for sensitive operations
10. **Session Tracking** - Full undo/revert with diff visualization

---

## Dioxus Advantages Over TUI

1. **Mouse Support** - Click interaction, drag-drop (TUI limited)
2. **Touch Support** - Mobile and tablet friendly
3. **Responsive Design** - Easily adapt to different screen sizes
4. **Rich CSS** - Unlimited styling options
5. **Browser Ecosystem** - Access to npm packages
6. **Accessibility** - Better ARIA support, screen readers
7. **Performance** - Efficient DOM diffing vs ANSI rendering
8. **Developer Experience** - Familiar HTML/CSS
9. **Collaboration** - Share via URL, web-native
10. **Mobile App** - Can wrap in Tauri for desktop, React Native bridge for mobile

---

## Next Steps

1. **Review Documentation** - Read all 4 generated docs in order
2. **Study Key Files** - Use file reference guide to understand architecture
3. **Understand Patterns** - Study code patterns doc with examples
4. **Plan Implementation** - Use migration checklist for phased approach
5. **Prototype** - Start with Phase 1 (foundation)
6. **Iterate** - Weekly checkpoints against progress
7. **Test Thoroughly** - Unit, integration, and E2E tests
8. **Optimize** - Performance tuning in final phase
9. **Deploy** - Launch with monitoring

---

## Documentation Files Created

All files created in `/home/thomas/src/projects/opencode-project/`:

1. **OPENCODE_TUI_ANALYSIS.md** - 11 sections, ~6,000 words
2. **OPENCODE_TUI_FILE_REFERENCE.md** - 46 files catalogued, ~3,000 words
3. **OPENCODE_TUI_CODE_PATTERNS.md** - 8 patterns with code, ~4,000 words
4. **TUI_TO_DIOXUS_MIGRATION_CHECKLIST.md** - 10 phases, ~2,000 words
5. **OPENCODE_TUI_EXPLORATION_SUMMARY.md** - This file

**Total Documentation:** ~15,000 words of analysis, patterns, and implementation guidance

---

## Success Criteria

Your Dioxus web app successfully replicates OpenCode's TUI when:

- [ ] All core features work (messaging, tools, sessions)
- [ ] Keyboard shortcuts fully operational (including leader key)
- [ ] Theme system supports dark/light + 23+ themes
- [ ] Dialog stack manages multiple modals correctly
- [ ] Prompt input supports virtual badges + autocomplete
- [ ] Tool registry renders all 11 tool types
- [ ] Real-time sync with backend works
- [ ] Responsive design works mobile/tablet/desktop
- [ ] Accessible (WCAG AA, keyboard navigation)
- [ ] Performance: <2s load, <100ms render, 60 FPS scroll

---

## Questions Answered

- **What terminal UI framework does OpenCode use?** @opentui/solid
- **How many components/files?** 46 files, 11 context providers
- **What's the main pattern?** Nested contexts for state, tool registry for components
- **How do they handle input?** Virtual text overlays (extmarks), multiline textarea
- **How do they render messages?** Dynamic dispatch via tool registry
- **How do they manage modals?** Stack-based dialog system
- **How do they handle themes?** 40+ colors, 23 themes, dark/light variants
- **How do they handle keyboard?** Keybind matching with leader key
- **What makes it special?** Streaming rendering, permission system, undo/revert
- **How to replicate in Dioxus?** Use same patterns with web equivalents

---

## Conclusion

OpenCode's TUI is a sophisticated, well-architected terminal application that leverages Solid.js reactivity and @opentui's low-level terminal control to create a highly responsive, keyboard-first interface. The implementation demonstrates several advanced patterns (tool registry, modal stack, virtual text) that can be effectively adapted to a modern web framework like Dioxus.

The provided documentation gives you:
1. Complete understanding of TUI architecture
2. Code patterns to replicate in Dioxus
3. 12-week implementation plan
4. File-by-file reference guide
5. Critical insights and gotchas

You're ready to begin the Dioxus implementation!

---

**Exploration Completed:** November 17, 2025
**Documentation Generated:** 4 comprehensive guides
**Total Analysis:** 46 files, ~1,500 LOC in key files
**Implementation Ready:** Yes

