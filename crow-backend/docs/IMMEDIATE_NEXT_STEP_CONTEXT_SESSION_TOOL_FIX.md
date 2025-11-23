# Crow Development Status & TodoWrite Session Context Fix

**Date**: 2025-11-17  
**Status**: ~75-80% Core Functionality Complete  
**Critical Fix**: TodoWrite now uses session context correctly

---

## Where We Are: Tool Execution Parity with OpenCode

### ✅ **Working Core Components**

The **most important part** of OpenCode - the tool execution loop - is now fully functional in Crow:

1. **LLM Integration** 
   - Using Moonshot's `kimi-k2-thinking` (262k context)
   - Auth via `~/.local/share/crow/auth.json` (matching OpenCode)
   - Reads API key from auth.json like OpenCode does

2. **Tool Execution Loop**
   - ✅ Read, Write, Edit - file operations
   - ✅ Bash - command execution  
   - ✅ Grep, Glob - code search
   - ✅ TodoWrite, TodoRead - planning/tracking
   - ✅ Task - subagent spawning with dynamic agent registry

3. **Session Management**
   - ✅ Session creation with XDG storage (`~/.local/share/crow/storage/`)
   - ✅ Message persistence and retrieval
   - ✅ Parent/child session linking (for subagents)
   - ✅ Session export

4. **Agent System**
   - ✅ Agent registry with dynamic tool permissions
   - ✅ Subagent spawning via Task tool
   - ✅ Agent-specific system prompts
   - ✅ Build, General, Plan agents (need to add Explore)

5. **Storage Architecture**
   - ✅ XDG base directories matching OpenCode
   - ✅ Per-session message storage
   - ✅ Todo persistence per session
   - ✅ Session metadata and exports

---

## The Problem We Just Fixed: TodoWrite Session Context

### **Bug Discovered**

TodoWrite tool was saving todos to `default.json` instead of `{session_id}.json` because:

1. The `TodoWriteInput` struct had a `session_id` field with `#[serde(default = "default_session_id")]`
2. When the LLM didn't provide `session_id` in the tool call, it defaulted to `"default"`
3. This caused all todos to be written to `~/.local/share/crow/storage/todo/default.json`

**Expected behavior** (from OpenCode):
- Todos should be stored per-session: `~/.local/share/crow/storage/todo/ses-{uuid}.json`
- The session ID should come from the execution context, not the LLM

### **Evidence of the Bug**

```bash
$ ls ~/.local/share/crow/storage/todo/
default.json  # ❌ Wrong - all sessions writing here
ses-5785c848-d314-4adb-b789-a1947b1c7135.json  # ✅ Correct format
```

OpenCode stores todos correctly:
```bash
$ ls ~/.local/share/opencode/storage/todo/
ses_57a498407ffeYjL68CbDIw3jP6.json
ses_57a877156ffeGmDziIoKsJT67A.json
# ... all per-session
```

---

## How It Was Resolved

### **Code Changes in `crow/packages/api/src/tools/todowrite.rs`**

**1. Changed execute signature to use context:**
```rust
// Before:
async fn execute(&self, input: Value, _ctx: &ToolContext) -> ToolResult {

// After:  
async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
```

**2. Used session_id from context:**
```rust
// Added this line:
let session_id = &ctx.session_id;

// Changed all uses of todo_input.session_id to session_id
todos.insert(session_id.clone(), todo_input.todos.clone());
let todo_file = global_paths.join(format!("{}.json", session_id));
metadata: json!({"session_id": session_id, ...})
```

**3. Removed session_id from TodoWriteInput:**
```rust
// Before:
#[derive(Deserialize)]
struct TodoWriteInput {
    #[serde(default = "default_session_id")]
    session_id: String,  // ❌ Don't ask LLM for this
    todos: Vec<TodoItem>,
}

// After:
#[derive(Deserialize)]
struct TodoWriteInput {
    todos: Vec<TodoItem>,  // ✅ Only todos needed from LLM
}
```

**Why This Is Correct:**

The `ToolContext` struct already contains the session ID:
```rust
pub struct ToolContext {
    pub session_id: String,  // ✅ Use this!
    pub message_id: String,
    pub agent: String,
    pub working_dir: std::path::PathBuf,
}
```

The LLM shouldn't need to know or provide the session ID - it's environmental context that the executor provides.

---

## What To Do Going Forward & Why

### **Immediate Next Steps (In Order of Priority)**

#### **1. Build the Dioxus TUI (HIGHEST PRIORITY)** 🎯

**Why This Is Critical:**

Right now we're testing Crow by:
- Manually curling endpoints
- Reading JSON dumps
- Checking file contents
- Grepping logs

This is **blind debugging**. We can't see:
- ❌ The agent thinking in real-time
- ❌ Tools being executed as they happen
- ❌ The todo list updating
- ❌ Session tree (parent/child relationships)
- ❌ Token usage and costs
- ❌ Reasoning traces (kimi-k2-thinking supports this!)

**What OpenCode's TUI Provides:**

Check `opencode/packages/opencode/src/cli/cmd/tui/`:
- Real-time message stream with tool execution
- Todo list panel (updating as agent works)
- Session tree view
- Token/cost tracking
- Keyboard navigation
- Theme support
- Model switching
- Session management

**The TUI makes development 10x faster** because you can:
1. Watch the agent work in real-time
2. See exactly where it's getting stuck
3. Observe tool execution patterns
4. Debug system prompts by seeing what the agent does
5. Catch bugs immediately (like the session_id issue)

**Implementation Plan:**

```
crow/packages/tui/  (new package)
├── src/
│   ├── main.rs         # Dioxus desktop app
│   ├── routes/
│   │   ├── session.rs  # Session list view
│   │   ├── messages.rs # Message stream view
│   │   └── todos.rs    # Todo panel
│   ├── components/
│   │   ├── tool_execution.rs  # Tool call renderer
│   │   ├── message.rs         # Message bubble
│   │   └── session_tree.rs    # Parent/child sessions
│   └── state/
│       └── app.rs      # Global app state
```

Study OpenCode's TUI components and port them to Dioxus patterns.

---

#### **2. Test the TodoWrite Fix**

After rebuilding:
```bash
# Start crow
cd test-dummy && crow serve -p 7070

# Create session and ask for plan
curl -X POST http://localhost:7070/session -d '{"working_directory": "..."}'
# Get session ID: ses-abc123...

curl -X POST http://localhost:7070/session/ses-abc123.../message \
  -d '{"agent": "build", "parts": [{"text": "Make a plan with TodoWrite to..."}]}'

# Verify todos saved to correct file
cat ~/.local/share/crow/storage/todo/ses-abc123....json
# Should see the todos, NOT in default.json
```

---

#### **3. Implement Missing Tools**

**Background Bash Execution:**
- BashOutput - retrieve output from long-running commands
- KillShell - terminate background processes

Study `opencode/packages/opencode/src/tool/bash.ts` for the background execution pattern.

**Why Important:**
Long-running commands (builds, tests, servers) need to run async while the agent does other work.

---

#### **4. Add Plan and Explore Agents**

These are critical for the full OpenCode experience:

**Plan Agent:**
- Uses Task tool to spawn subagents
- Breaks down complex tasks
- Coordinates multiple agents

**Explore Agent:**
- Fast codebase navigation
- Pattern matching
- Question answering about code structure

Copy their system prompts from OpenCode and add to `crow/packages/api/src/agent/builtins.rs`.

---

#### **5. Implement Streaming (SSE)**

The `/session/{id}/message` endpoint should support Server-Sent Events for real-time streaming.

Check `crow/packages/api/src/server.rs` - there's already a `send_message_stream` function stub.

**Why Important:**
The TUI needs this to show messages as they're generated, not after completion.

---

### **Lower Priority (But Still Important)**

6. **Model Switching Without Rebuild**
   - Currently model is hardcoded in 4 places
   - Should read from session config or user preference
   - OpenCode uses `Provider.defaultModel()` pattern

7. **System Prompt Verification**
   - Compare Crow's prompts to OpenCode's exactly
   - Use verbose mode to log full prompts
   - Ensure tool descriptions match

8. **Comparative Testing**
   - Run same tasks in OpenCode and Crow
   - Compare tool execution patterns
   - Verify outputs match

9. **MCP Support**
   - Model Context Protocol integration
   - See `opencode/packages/opencode/src/mcp/`

10. **LSP Integration**
    - Language Server Protocol for code intelligence
    - Hover, diagnostics, etc.

---

## Project Structure Comparison

### **What Crow Has:**
```
crow/packages/api/src/
├── agent/          # Agent system with registry, executor, prompts
├── auth.rs         # Auth.json support (NEW)
├── providers/      # LLM provider clients
├── session/        # Session store, locks, exports
├── storage/        # XDG storage management
├── tools/          # All tool implementations
└── server.rs       # REST API
```

### **What OpenCode Has (That We're Missing):**
```
opencode/packages/opencode/src/
├── cli/cmd/tui/    # 🎯 Terminal UI (PRIORITY #1)
├── config/         # User configuration system
├── mcp/            # Model Context Protocol
├── lsp/            # Language Server Protocol
├── observability/  # Langfuse telemetry
├── file/           # fzf, ripgrep, watcher
├── format/         # Code formatters
├── util/           # Tons of utilities
└── ...
```

**The TUI is the missing piece that makes everything else visible and usable.**

---

## Success Metrics

We'll know we're at 100% feature parity when:

✅ Can run `crow tui` and see a beautiful Dioxus interface  
✅ Watch agents execute tools in real-time  
✅ See todos update as agents work  
✅ Navigate session trees  
✅ Switch models without rebuilding  
✅ Run same task in OpenCode and Crow - get same result  
✅ Stream responses via SSE  
✅ Background bash commands work  
✅ Plan/Explore agents coordinate complex tasks  

---

## Current Test Results

**Last successful test** (2025-11-17):

```bash
# Crow successfully:
✅ Created 7-step plan using TodoWrite
✅ Saved todos to ~/.local/share/crow/storage/todo/ses-f1e2753a-d0e4-4c1d-90ca-b5f38eab25b0.json
✅ Spawned subagent via Task tool  
✅ Created child session: ses-369a9dea-e0bd-4d18-af21-1a19f83efd75
✅ Used kimi-k2-thinking (262k context)
✅ Read API key from auth.json
```

**The core loop works.** Now we need visibility.

---

## Build the TUI Next 🚀

The path forward is clear:
1. Study OpenCode's TUI structure
2. Port to Dioxus components
3. Connect to Crow's REST API
4. Watch the magic happen in real-time

Then the rest becomes easy because we can SEE what's happening.
