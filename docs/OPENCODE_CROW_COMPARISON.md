# OpenCode vs Crow Implementation Comparison

**Analysis Date:** 2025-11-16  
**OpenCode Location:** `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/`  
**Crow Location:** `/home/thomas/src/projects/opencode-project/crow/packages/api/src/`

---

## Executive Summary

Crow has implemented ~60% of OpenCode's core functionality. We have the basic ReACT loop, session management, and tool execution working. However, we're missing critical production features around:
- Session lifecycle management (compaction, revert, retry, summarization)
- Advanced permission system
- Initialization/bootstrap sequence
- Background processing and optimization
- File watching and LSP integration

---

## 1. SESSION MANAGEMENT

### ✅ What We HAVE (Crow)
- **Basic CRUD**: Create, read, update, delete sessions
- **File persistence**: JSON storage in `.crow/storage/session/`, `.crow/storage/message/`, `.crow/storage/part/`
- **Session relationships**: Parent-child session tracking
- **Message storage**: Messages with parts (text, tool, thinking, file)
- **Real-time export**: Automatic markdown export to `.crow/sessions/{id}.md`
- **Dual-agent support**: Special session types for executor/discriminator pairs

**Files:**
- `crow/packages/api/src/session/store.rs` - SessionStore with in-memory + file persistence
- `crow/packages/api/src/storage/crow.rs` - CrowStorage for .crow directory management
- `crow/packages/api/src/session/export.rs` - Markdown export

### ❌ What We're MISSING (OpenCode has)

#### 1.1 Session Compaction (`SessionCompaction`)
**Location:** `opencode/packages/opencode/src/session/compaction.ts`

**Purpose:**
- Auto-summarizes long sessions when context overflow occurs
- Prunes old tool call outputs (keeps last 40k tokens worth, removes older)
- Prevents hitting model context limits

**Key Features:**
- `isOverflow()` - Detects when session exceeds model context
- `prune()` - Removes old tool outputs while preserving recent ones
- `run()` - Generates AI summary of compacted messages
- Configurable thresholds: `PRUNE_MINIMUM=20000`, `PRUNE_PROTECT=40000`

**Impact:** Without this, long sessions will fail when hitting context limits.

---

#### 1.2 Session Revert (`SessionRevert`)
**Location:** `opencode/packages/opencode/src/session/revert.ts`

**Purpose:**
- Time-travel: Revert session to any previous message/part
- Uses git-like snapshots to track filesystem changes
- Can undo file edits made by assistant

**Key Features:**
- `revert()` - Marks messages after a point as reverted
- `unrevert()` - Restores reverted messages
- `cleanup()` - Removes reverted messages from storage
- Integrates with `Snapshot` system for filesystem diffs

**Impact:** Users can't undo mistakes or explore alternative conversation branches.

---

#### 1.3 Session Summary (`SessionSummary`)
**Location:** `opencode/packages/opencode/src/session/summary.ts`

**Purpose:**
- Auto-generates titles for sessions
- Computes diff summaries (additions/deletions per file)
- Tracks which files were modified in session

**Key Features:**
- `summarize()` - Called after each message
- `summarizeSession()` - Updates session-level diff stats
- `summarizeMessage()` - Generates concise title using small model
- Stores diffs in `storage/session_diff/{sessionID}.json`

**Impact:** No automatic session titles, no file change tracking.

---

#### 1.4 Session Retry (`SessionRetry`)
**Location:** `opencode/packages/opencode/src/session/retry.ts`

**Purpose:**
- Automatic retry with exponential backoff for API errors
- Handles rate limits, timeouts, transient failures
- Configurable max retries (default: 10)

**Key Features:**
- `getBoundedDelay()` - Calculates backoff delay
- `sleep()` - Async sleep with abort support
- Integrates with `SessionPrompt.process()` retry loop

**Impact:** Sessions fail on temporary API errors instead of retrying.

---

#### 1.5 Session Locking (`SessionLock`)
**Location:** `opencode/packages/opencode/src/session/lock.ts`

**What We Have:** ✅ Basic concept in `AgentExecutor`  
**What We're Missing:**
- Centralized lock registry (prevents concurrent execution)
- `abort()` method to cancel running sessions
- `isLocked()` query method
- Used throughout: `/session/:id/abort` endpoint, `SessionPrompt`, etc.

**Impact:** Race conditions if multiple clients try to prompt same session.

---

## 2. AGENT EXECUTION

### ✅ What We HAVE (Crow)
- **ReACT loop**: Reasoning + Acting with tool calls
- **Agent registry**: Multiple agents (build, discriminator, supervisor, etc.)
- **System prompts**: Agent-specific prompts with environment context
- **Tool filtering**: Agents can enable/disable specific tools
- **Permission checks**: Basic edit/bash permission enforcement
- **Dual-agent runtime**: Executor + Discriminator collaboration

**Files:**
- `crow/packages/api/src/agent/executor.rs` - AgentExecutor (ReACT loop)
- `crow/packages/api/src/agent/runtime.rs` - DualAgentRuntime
- `crow/packages/api/src/agent/registry.rs` - AgentRegistry
- `crow/packages/api/src/agent/prompt.rs` - SystemPromptBuilder

### ❌ What We're MISSING (OpenCode has)

#### 2.1 Advanced System Prompt Building
**Location:** `opencode/packages/opencode/src/session/system.ts`

**OpenCode builds system prompts from:**
- Header (provider-specific)
- Agent prompt (custom or built-in)
- Environment context (platform, OS, working directory, git status)
- Custom prompts from config
- Provider-specific formatting

**We have:** Basic system prompt with agent description  
**We're missing:** Environment context, git integration, provider customization

---

#### 2.2 Message Processing Pipeline
**Location:** `opencode/packages/opencode/src/session/prompt.ts` - `createProcessor()`

**OpenCode's Processor:**
- Tracks tool call state transitions (pending → running → completed/error)
- Handles streaming text/reasoning/tool deltas
- Manages snapshots for filesystem tracking
- Computes token usage and costs
- Detects "doom loops" (same tool called 3x with same args)
- Emits granular events for UI streaming

**We have:** Basic tool execution  
**We're missing:** State tracking, streaming, doom loop detection, cost tracking

---

#### 2.3 Context Injection
**Location:** `opencode/packages/opencode/src/session/prompt.ts` - `getMessages()`

**OpenCode supports:**
- Parent context: Include messages from parent session
- Sibling context: Include messages from sibling sessions
- Conversation threading: Link related sessions
- Configured via `session.metadata.includeParentContext`, etc.

**We have:** Simple message history  
**We're missing:** Cross-session context sharing

---

#### 2.4 Reminder System
**Location:** `opencode/packages/opencode/src/session/prompt.ts` - `insertReminders()`

**OpenCode injects:**
- Agent-specific instructions (e.g., "plan" agent gets read-only reminder)
- Mode transition hints (e.g., switching from "plan" to "build")
- Synthetic parts added to last user message

**We have:** None  
**We're missing:** Dynamic instruction injection

---

## 3. TOOL SYSTEM

### ✅ What We HAVE (Crow)
- **Tool registry**: Register and execute tools
- **OpenAI format**: Convert to OpenAI tool schemas
- **Basic tools**: Read, Write, Edit, Bash, Grep, Glob, List, TodoRead/Write, WorkCompleted
- **Agent filtering**: Tools filtered by agent configuration
- **Permission checks**: Basic wildcard-based permission matching

**Files:**
- `crow/packages/api/src/tools/mod.rs` - ToolRegistry
- `crow/packages/api/src/tools/*.rs` - Individual tool implementations

### ❌ What We're MISSING (OpenCode has)

#### 3.1 Dynamic Tool Registry
**Location:** `opencode/packages/opencode/src/tool/registry.ts`

**OpenCode features:**
- Runtime tool registration (not just static)
- MCP (Model Context Protocol) integration - external tool servers
- Provider-specific tool filtering
- Tool schema transformation per provider
- `experimental_repairToolCall` - fixes common tool name typos

**We have:** Static tool registration  
**We're missing:** MCP, dynamic registration, schema transformation

---

#### 3.2 Permission System
**Location:** `opencode/packages/opencode/src/permission/index.ts`

**OpenCode's Permission:**
- Three levels: `allow`, `deny`, `ask`
- Interactive permission requests with UI
- Per-agent permission overrides
- Pattern-based matching (wildcards)
- Special permissions: `doom_loop`, `external_directory`, `webfetch`
- `RejectedError` with metadata
- Plugin hooks for custom permission logic

**We have:** Basic allow/deny checking  
**We're missing:** `ask` mode, interactive requests, plugin hooks

---

#### 3.3 Tool Metadata Callbacks
**Location:** `opencode/packages/opencode/src/session/prompt.ts` - tool execute metadata callback

**OpenCode tools can:**
- Update their status mid-execution (`metadata()` callback)
- Show progress (e.g., "Downloading file... 50%")
- Store execution metadata

**We have:** None  
**We're missing:** Progress tracking during tool execution

---

## 4. STORAGE & PERSISTENCE

### ✅ What We HAVE (Crow)
- **File-based storage**: JSON files in `.crow/storage/`
- **Three-tier structure**: session/ , message/, part/
- **Async I/O**: Tokio async file operations
- **Auto-initialization**: Creates .crow directory structure
- **Markdown export**: Real-time session export

**Files:**
- `crow/packages/api/src/storage/crow.rs` - CrowStorage

### ❌ What We're MISSING (OpenCode has)

#### 4.1 Storage Abstraction Layer
**Location:** `opencode/packages/opencode/src/storage/storage.ts`

**OpenCode's Storage namespace:**
- Key-path based access: `Storage.read(["session", projectID, sessionID])`
- Atomic operations: `Storage.update()` with callback
- File locking: Read/write locks for concurrent access
- Migrations: Versioned schema migrations
- Error handling: Custom `NotFoundError`
- Glob-based listing: `Storage.list(["session", projectID])`

**We have:** Direct file access  
**We're missing:** Abstraction layer, locking, migrations

---

#### 4.2 Data Migrations
**Location:** `opencode/packages/opencode/src/storage/storage.ts` - `MIGRATIONS`

**OpenCode has:**
- Migration framework with version tracking
- Automatic execution on startup
- Stored migration index in `storage/migration`
- Examples: Project ID migration, diff extraction

**We have:** None  
**We're missing:** Schema evolution strategy

---

#### 4.3 Snapshot System
**Location:** `opencode/packages/opencode/src/snapshot/` (inferred from imports)

**OpenCode tracks:**
- Filesystem state before/after each tool execution
- Git-like patches for file changes
- Revert capability
- Diff computation

**We have:** None  
**We're missing:** Filesystem change tracking

---

## 5. SERVER INITIALIZATION & LIFECYCLE

### ✅ What We HAVE (Crow)
- **Basic router**: Axum router with REST endpoints
- **CORS**: Permissive CORS for development
- **State management**: AppState with Arc for thread safety

**Files:**
- `crow/packages/api/src/server.rs` - create_router()

### ❌ What We're MISSING (OpenCode has)

#### 5.1 Instance Bootstrap
**Location:** `opencode/packages/opencode/src/project/bootstrap.ts` - `InstanceBootstrap()`

**OpenCode initializes:**
1. Plugin system (`Plugin.init()`)
2. Share service (`Share.init()`)
3. Formatters (`Format.init()`)
4. LSP servers (`LSP.init()`)
5. File watcher (`FileWatcher.init()`)
6. File indexer (`File.init()`)
7. Event bus subscriptions

**We have:** None  
**We're missing:** Entire initialization sequence

---

#### 5.2 Instance.provide() Pattern
**Location:** `opencode/packages/opencode/src/project/instance.ts`

**OpenCode uses:**
- Async context for directory-scoped operations
- `Instance.provide({ directory, init, fn })` wraps all requests
- Automatic cleanup on shutdown
- Scoped state management

**We have:** Global state  
**We're missing:** Directory-scoped instances

---

#### 5.3 Event Bus System
**Location:** `opencode/packages/opencode/src/bus/`

**OpenCode publishes events:**
- `Session.Event.Created`, `Updated`, `Deleted`
- `MessageV2.Event.PartUpdated`, `Removed`
- `TuiEvent.*` for UI synchronization
- `GlobalBus` for cross-instance events
- SSE endpoint: `/global/event`

**We have:** None  
**We're missing:** Event-driven architecture

---

#### 5.4 Background Services

**OpenCode runs:**
- File watcher: Auto-detects file changes
- LSP client: Code intelligence
- Format service: Auto-formatting
- MCP servers: External tool providers
- Share service: Session sharing to cloud

**We have:** None  
**We're missing:** All background services

---

## 6. API ENDPOINTS

### ✅ What We HAVE (Crow)

**Session Management:**
- ✅ `GET /session` - List sessions
- ✅ `POST /session` - Create session
- ✅ `GET /session/:id` - Get session
- ✅ `DELETE /session/:id` - Delete session
- ✅ `PATCH /session/:id` - Update session
- ✅ `POST /session/:id/fork` - Fork session
- ✅ `GET /session/:id/children` - Get child sessions

**Messages:**
- ✅ `GET /session/:id/message` - List messages
- ✅ `POST /session/:id/message` - Send message (non-streaming)
- ✅ `POST /session/:id/message/stream` - Send message (SSE streaming)
- ✅ `GET /session/:id/message/:messageID` - Get specific message

**Other:**
- ✅ `GET /config` - Get config
- ✅ `GET /config/providers` - List providers
- ✅ `GET /agent` - List agents
- ✅ `GET /experimental/tool/ids` - List tool IDs
- ✅ `GET /experimental/tool` - List tools with schemas

### ❌ What We're MISSING (OpenCode has)

**Session Operations:**
- ❌ `POST /session/:id/abort` - Abort running session
- ❌ `POST /session/:id/share` - Share session
- ❌ `DELETE /session/:id/share` - Unshare session
- ❌ `POST /session/:id/init` - Initialize project analysis
- ❌ `POST /session/:id/summarize` - Trigger summarization
- ❌ `POST /session/:id/revert` - Revert to message
- ❌ `POST /session/:id/unrevert` - Restore reverted messages
- ❌ `GET /session/:id/diff` - Get session diffs
- ❌ `GET /session/:id/todo` - Get todo list

**Messages:**
- ❌ `POST /session/:id/command` - Execute slash command
- ❌ `POST /session/:id/shell` - Run shell command
- ❌ `POST /session/:id/permissions/:permissionID` - Respond to permission request

**Files:**
- ❌ `GET /file` - List files
- ❌ `GET /file/content` - Read file
- ❌ `GET /file/status` - Get git status
- ❌ `GET /find` - Text search (ripgrep)
- ❌ `GET /find/file` - File search
- ❌ `GET /find/symbol` - Symbol search (LSP)

**Services:**
- ❌ `GET /mcp` - MCP server status
- ❌ `POST /mcp` - Add MCP server
- ❌ `GET /lsp` - LSP server status
- ❌ `GET /formatter` - Formatter status

**Project:**
- ❌ `GET /project` - List projects
- ❌ `GET /project/current` - Get current project

**Global:**
- ❌ `GET /global/event` - SSE event stream
- ❌ `GET /path` - Get paths
- ❌ `POST /log` - Send log to server

**TUI Control:**
- ❌ `/tui/*` - All TUI control endpoints

---

## 7. ARCHITECTURAL DIFFERENCES

### OpenCode Architecture
- **Namespace-based**: Heavy use of TypeScript namespaces (e.g., `Session.`, `Agent.`, `Storage.`)
- **Lazy initialization**: Many components use `lazy(() => ...)` pattern
- **Event-driven**: Pub/sub via `Bus.publish()` / `Bus.subscribe()`
- **Instance-scoped**: State scoped to `Instance.directory`
- **Middleware-heavy**: Hono middleware for validation, error handling, etc.
- **Provider abstraction**: `ai` SDK for multi-provider support
- **Plugin system**: Extensible via plugins

### Crow Architecture
- **Module-based**: Rust modules with explicit exports
- **Eager initialization**: Most components initialized upfront
- **Direct calls**: Function calls instead of events
- **Global state**: Arc<> for shared state
- **Minimal middleware**: Basic CORS only
- **Provider-specific**: OpenAI client directly used
- **No plugin system**: Static configuration

---

## 8. CRITICAL GAPS REQUIRING IMMEDIATE ATTENTION

### Priority 1: Session Lifecycle (Blocks Production)
1. **Session Locking** - Prevents concurrent execution crashes
2. **Session Abort** - Users can't stop runaway sessions
3. **Retry Logic** - Sessions fail on transient API errors

### Priority 2: Tool Execution (User Experience)
4. **Permission Ask Mode** - Users can't review dangerous operations
5. **Tool Progress** - No feedback during long operations
6. **Doom Loop Detection** - Infinite loops waste tokens

### Priority 3: Context Management (Scalability)
7. **Session Compaction** - Long sessions hit context limits
8. **Message Pruning** - Old tool outputs bloat context

### Priority 4: Data Integrity (Reliability)
9. **File Locking** - Concurrent access could corrupt storage
10. **Migrations** - No upgrade path for schema changes

### Priority 5: Observability (Debugging)
11. **Event Bus** - No visibility into system state
12. **Streaming** - UI can't show real-time progress
13. **Logging** - Basic logging only

---

## 9. FEATURE COMPARISON MATRIX

| Feature | OpenCode | Crow | Priority |
|---------|----------|------|----------|
| **Session CRUD** | ✅ | ✅ | - |
| **Message Storage** | ✅ | ✅ | - |
| **Tool Execution** | ✅ | ✅ | - |
| **Agent System** | ✅ | ✅ | - |
| **Dual Agents** | ❌ | ✅ | - |
| **Session Locking** | ✅ | ❌ | P1 |
| **Session Abort** | ✅ | ❌ | P1 |
| **Retry Logic** | ✅ | ❌ | P1 |
| **Compaction** | ✅ | ❌ | P3 |
| **Revert** | ✅ | ❌ | P4 |
| **Summarization** | ✅ | ❌ | P4 |
| **Permission Ask** | ✅ | ❌ | P2 |
| **Tool Progress** | ✅ | ❌ | P2 |
| **Doom Loop Detection** | ✅ | ❌ | P2 |
| **Snapshots** | ✅ | ❌ | P4 |
| **File Locking** | ✅ | ❌ | P4 |
| **Migrations** | ✅ | ❌ | P4 |
| **Event Bus** | ✅ | ❌ | P5 |
| **SSE Streaming** | ✅ | ⚠️ (Partial) | P5 |
| **MCP** | ✅ | ❌ | P5 |
| **LSP** | ✅ | ❌ | P5 |
| **File Watcher** | ✅ | ❌ | P5 |
| **Session Sharing** | ✅ | ❌ | P5 |
| **Markdown Export** | ❌ | ✅ | - |

---

## 10. RECOMMENDATIONS

### Immediate Actions (Week 1)
1. **Implement SessionLock** - Copy OpenCode's lock.ts logic to prevent race conditions
2. **Add abort endpoint** - Allow canceling running sessions
3. **Basic retry logic** - Exponential backoff for API errors

### Short-term (Month 1)
4. **Permission ask mode** - Interactive permission requests
5. **Doom loop detection** - Track repeated tool calls
6. **File locking** - Prevent concurrent storage writes

### Medium-term (Month 2-3)
7. **Session compaction** - Auto-summarize long sessions
8. **Message pruning** - Remove old tool outputs
9. **Event bus** - Enable real-time UI updates

### Long-term (Month 4+)
10. **Snapshot system** - Track filesystem changes
11. **Revert capability** - Time-travel in sessions
12. **MCP integration** - External tool providers
13. **LSP integration** - Code intelligence

### Consider NOT Implementing
- **Instance.provide pattern** - Rust doesn't need this (scoped state via Arc)
- **TUI endpoints** - If building web UI instead
- **Session sharing** - Unless building SaaS

---

## 11. CODE EXAMPLES

### Missing: Session Locking

**OpenCode has:**
```typescript
// opencode/packages/opencode/src/session/lock.ts
export namespace SessionLock {
  export function acquire(input: { sessionID: string }) {
    const controller = new AbortController()
    state().locks.set(input.sessionID, { controller, created: Date.now() })
    return {
      signal: controller.signal,
      abort() { controller.abort() },
      [Symbol.dispose]() { unset() }
    }
  }
  
  export function abort(sessionID: string) {
    const lock = get(sessionID)
    if (lock) lock.controller.abort()
  }
}
```

**Crow needs:**
```rust
// crow/packages/api/src/session/lock.rs
pub struct SessionLock {
    locks: Arc<RwLock<HashMap<String, CancellationToken>>>
}

impl SessionLock {
    pub fn acquire(&self, session_id: &str) -> Result<LockGuard, String> {
        let token = CancellationToken::new();
        // ... implementation
    }
    
    pub fn abort(&self, session_id: &str) -> bool {
        // Cancel the token
    }
}
```

---

### Missing: Retry Logic

**OpenCode has:**
```typescript
// opencode/packages/opencode/src/session/prompt.ts
for (let retry = 1; retry < maxRetries; retry++) {
  const delayMs = SessionRetry.getBoundedDelay({
    error: lastRetryPart.error,
    attempt: retry,
    startTime: start,
  })
  if (!delayMs) break
  
  await SessionRetry.sleep(delayMs, abort.signal)
  stream = doStream()
  result = await processor.process(stream, { count: retry, max: maxRetries })
  if (!result.shouldRetry) break
}
```

**Crow needs:**
```rust
// crow/packages/api/src/session/retry.rs
pub async fn execute_with_retry<F, T>(
    f: F,
    max_retries: usize,
) -> Result<T, String>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T, String>>>>,
{
    let mut attempt = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries && is_retryable(&e) => {
                let delay = calculate_backoff(attempt);
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## 12. SUMMARY

**What we have:** A functional agent execution system with basic session management and tool execution.

**What we're missing:** Production-ready features for reliability (locking, retry, error recovery), user experience (permissions, progress, abort), and scalability (compaction, pruning, optimization).

**Estimated completion:** 60% of core features, 30% of production features.

**Next steps:** Focus on Priority 1 & 2 items to make Crow production-ready.

