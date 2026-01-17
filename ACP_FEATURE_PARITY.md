# ACP Server Feature Parity Comparison

## Overview

Comparing **Crow ACP Server** (`src/crow/agent/acp_server.py`) vs **Kimi-CLI ACP Server** (`kimi-cli/src/kimi_cli/acp/`)

---

## Architecture Comparison

### Crow ACP Server (OpenHands SDK-based)
```
ACP Protocol Handler (async)
    ↓
Thread Pool Executor
    ↓
OpenHands SDK Conversation (sync/blocking)
    ↓
Token Callback → asyncio.Queue
    ↓
ACP Session Updates (async)
```

### Kimi-CLI ACP Server (Custom Soul-based)
```
ACP Protocol Handler (async)
    ↓
Custom Agent Loop (async)
    ↓
Streaming Event Generator (async)
    ↓
ACP Session Updates (async)
```

**Key Difference**: Crow uses thread-based bridging for blocking OpenHands SDK, while Kimi-CLI is natively async throughout.

---

## Feature Comparison Matrix

| Feature | Crow ACP Server | Kimi-CLI ACP Server | Status |
|---------|----------------|-------------------|--------|
| **Core ACP Protocol** |
| `initialize` | ✅ Full support | ✅ Full support | 🟰 Parity |
| `new_session` | ✅ Basic support | ✅ Full support + metadata | 🔴 Missing metadata |
| `load_session` | ✅ JSON persistence | ✅ KaosPath-based | 🟡 Different approach |
| `list_sessions` | ❌ Not implemented | ✅ Full implementation | 🔴 Missing |
| `prompt` | ✅ Streaming support | ✅ Streaming support | 🟰 Parity |
| `cancel` | ⚠️ `conversation.pause()` only | ✅ Async event cancellation | 🔴 Weaker cancellation |
| `set_session_mode` | ✅ Implemented | ⚠️ Only "default" mode | 🟢 Better |
| `set_session_model` | ❌ Not implemented | ✅ Full implementation | 🔴 Missing |
| **Streaming** |
| Token streaming | ✅ via `token_callbacks` | ✅ via async event loop | 🟰 Parity |
| Thinking blocks | ✅ `reasoning_content` | ✅ `ThinkPart` | 🟰 Parity |
| Tool call streaming | ✅ Incremental args | ✅ `ToolCallPart` | 🟰 Parity |
| **Tool Integration** |
| Terminal tool | ✅ OpenHands TerminalTool | ✅ Custom ACP Terminal | 🔴 Using client terminal |
| File editor | ✅ OpenHands FileEditorTool | ❌ Uses diff blocks | 🟢 Better UX |
| MCP servers | ✅ Full support | ✅ Full support | 🟰 Parity |
| Tool approval | ✅ ACP permission requests | ✅ ACP permission requests | 🟰 Parity |
| **Error Handling** |
| Try/except wrapper | ❌ None (fails fast) | ✅ Full error categorization | 🔴 Missing error handling |
| Error recovery | ❌ None | ✅ Graceful degradation | 🔴 Missing |
| Error context | ⚠️ Basic logging | ✅ Rich error context | 🔴 Missing |
| **Session Management** |
| Persistence | ✅ JSON files | ✅ KaosPath-based | 🟡 Different |
| Session replay | ✅ On load | ⚠️ "TODO: replay" comment | 🟢 Better |
| Multiple sessions | ✅ Supported | ✅ Supported | 🟰 Parity |
| **Model Management** |
| Model switching | ❌ Not implemented | ✅ Runtime model switching | 🔴 Missing |
| Thinking mode toggle | ❌ Not implemented | ✅ `,thinking` suffix | 🔴 Missing |
| Model capabilities | ❌ Not queried | ✅ `derive_model_capabilities()` | 🔴 Missing |
| **Advanced Features** |
| Slash commands | ✅ Basic (/help, /clear, /status) | ✅ Registry-based | 🟡 Different approach |
| Plan updates | ✅ Basic plan entries | ✅ Todo list integration | 🟡 Different approach |
| Auth methods | ❌ Not implemented | ✅ `terminal-auth` setup | 🔴 Missing |
| Client capabilities | ⚠️ Stored but unused | ✅ Used for tool replacement | 🔴 Underutilized |

---

## Detailed Feature Analysis

### 🔴 Critical Missing Features

#### 1. **Error Handling & Recovery**
**Kimi-CLI:**
```python
except LLMNotSet as e:
    raise acp.RequestError.auth_required() from e
except LLMNotSupported as e:
    raise acp.RequestError.internal_error({"error": str(e)}) from e
except ChatProviderError as e:
    raise acp.RequestError.internal_error({"error": str(e)}) from e
except MaxStepsReached as e:
    return acp.PromptResponse(stop_reason="max_turn_requests")
except RunCancelled:
    return acp.PromptResponse(stop_reason="cancelled")
```

**Crow:**
```python
# No try/except - errors propagate as exceptions
# Conversation just stops on error
```

**Impact**: Poor UX, crashes instead of graceful errors.

---

#### 2. **Session Listing**
**Kimi-CLI:**
```python
async def list_sessions(self, cursor: str | None = None, cwd: str | None = None):
    sessions = await Session.list(work_dir)
    return acp.schema.ListSessionsResponse(
        sessions=[acp.schema.SessionInfo(
            cwd=cwd,
            session_id=s.id,
            title=s.title,
            updated_at=datetime.fromtimestamp(s.updated_at).isoformat(),
        ) for s in sessions],
        next_cursor=None,
    )
```

**Crow:**
```python
# Not implemented
```

**Impact**: Users can't see or manage previous sessions.

---

#### 3. **Model Switching**
**Kimi-CLI:**
```python
async def set_session_model(self, model_id: str, session_id: str):
    model_id_conv = _ModelIDConv.from_acp_model_id(model_id)
    new_model = config.models.get(model_id_conv.model_key)
    new_llm = create_llm(new_provider, new_model, thinking=model_id_conv.thinking)
    cli_instance.soul.runtime.llm = new_llm
```

**Crow:**
```python
# Not implemented
# Model is fixed at agent creation
```

**Impact**: Can't switch between fast/thinking models mid-session.

---

#### 4. **Native Cancellation**
**Kimi-CLI:**
```python
self._turn_state.cancel_event = asyncio.Event()
# Async loop checks event and breaks gracefully
async for msg in self._cli.run(user_input, self._turn_state.cancel_event):
    if self._turn_state.cancel_event.is_set():
        break
```

**Crow:**
```python
conversation.pause()  # Only waits for current LLM call
# No hard cancellation
```

**Impact**: Long-running operations can't be stopped immediately.

---

#### 5. **Terminal Tool Replacement**
**Kimi-CLI:**
```python
class Terminal(CallableTool2[ShellParams]):
    async def __call__(self, params: ShellParams):
        terminal = await self._acp_conn.create_terminal(...)
        await self._acp_conn.session_update(
            update=acp.schema.ToolCallProgress(
                content=[acp.schema.TerminalToolCallContent(
                    type="terminal",
                    terminal_id=terminal.id,
                )]
            ),
        )
```

**Crow:**
```python
# Uses OpenHands TerminalTool which outputs to stdout/stderr
# Doesn't use ACP terminal streaming
```

**Impact**: Terminal output doesn't stream nicely in ACP clients.

---

### 🟡 Partial Implementation

#### 1. **Session Modes**
**Crow:**
```python
self._available_modes = [
    SessionMode(id="default", name="Default Mode", ...),
    SessionMode(id="code", name="Code Mode", ...),
    SessionMode(id="chat", name="Chat Mode", ...),
]
```

**Kimi-CLI:**
```python
modes=acp.schema.SessionModeState(
    available_modes=[
        acp.schema.SessionMode(id="default", name="The default mode.", ...),
    ],
    current_mode_id="default",
)
```

**Issue**: Crow defines modes but doesn't actually change behavior based on mode.

---

#### 2. **Tool Call State Management**
**Kimi-CLI:**
```python
class _TurnState:
    def __init__(self):
        self.id = str(uuid.uuid4())
        self.tool_calls: dict[str, _ToolCallState] = {}
        self.last_tool_call: _ToolCallState | None = None
        self.cancel_event = asyncio.Event()
```

**Crow:**
```python
current_tool_call = {"id": None, "name": None, "args": ""}
# Basic dict tracking
```

**Issue**: Crow's state management is simpler and less robust.

---

### 🟢 Crow Advantages

#### 1. **Simpler Architecture**
- Direct OpenHands SDK usage
- Less custom code to maintain
- Leverages well-tested SDK

#### 2. **File Editor Tool**
OpenHands FileEditorTool provides richer editing:
- `str_replace` operations
- Multi-file edits
- View, create, edit commands

#### 3. **Security Disabled by Default**
```python
"security_policy_filename": "",  # Disable security checks
```
Better for automated agent workflows.

---

## Implementation Priority

### Phase 1: Critical Error Handling (HIGH PRIORITY)
```python
async def prompt(self, prompt: list, session_id: str):
    try:
        # ... existing code ...
    except OpenHandsError as e:
        logger.exception("OpenHands SDK error:")
        raise acp.RequestError.internal_error({"error": str(e)}) from e
    except Exception as e:
        logger.exception("Unexpected error:")
        raise acp.RequestError.internal_error({"error": str(e)}) from e
```

### Phase 2: Session Listing (HIGH PRIORITY)
```python
async def list_sessions(self, cwd: str | None = None):
    if not cwd:
        return acp.schema.ListSessionsResponse(sessions=[], next_cursor=None)
    
    sessions_dir = Path.home() / ".crow" / "sessions"
    sessions = []
    for session_file in sessions_dir.glob("*.json"):
        with open(session_file) as f:
            data = json.load(f)
            sessions.append(acp.schema.SessionInfo(
                cwd=data.get("cwd", cwd),
                session_id=data["session_id"],
                title=data.get("title", "Untitled"),
                updated_at=datetime.fromtimestamp(session_file.stat().st_mtime).isoformat(),
            ))
    
    return acp.schema.ListSessionsResponse(sessions=sessions, next_cursor=None)
```

### Phase 3: Model Switching (MEDIUM PRIORITY)
```python
async def set_session_model(self, model_id: str, session_id: str):
    # Parse model_id (may have ",thinking" suffix)
    # Update session["agent"]._llm with new model
    # Send session/update notification
```

### Phase 4: Terminal Tool Replacement (MEDIUM PRIORITY)
```python
# Replace OpenHands TerminalTool with ACP Terminal tool
# Use acp_conn.create_terminal() for streaming output
```

### Phase 5: Better Cancellation (LOW PRIORITY)
```python
# Implement hard cancellation beyond just pause()
# May require OpenHands SDK changes
```

---

## Code Quality Comparison

### Kimi-CLI Strengths
- ✅ Comprehensive error handling
- ✅ Rich type hints
- ✅ Well-documented classes
- ✅ Modular architecture (separate files for session, tools, mcp)
- ✅ Context variables for scoping
- ✅ Streaming JSON lexer for tool args
- ✅ Display block system for rich output

### Crow Strengths
- ✅ Simpler, more direct code
- ✅ Leverages OpenHands SDK (less custom code)
- ✅ Clear separation of concerns
- ✅ Good logging
- ⚠️ Could benefit from better error handling
- ⚠️ Could use more type hints
- ⚠️ Could use better state management

---

## Recommendations

### Immediate Actions (Week 1)
1. **Add comprehensive error handling** to `prompt()` method
2. **Implement `list_sessions()`** for session management
3. **Add full stack traces** to error responses

### Short-term (Weeks 2-3)
4. **Implement model switching** via `set_session_model()`
5. **Replace Terminal tool** with ACP-native terminal
6. **Add session metadata** (title, timestamps)
7. **Improve cancellation** beyond just `pause()`

### Long-term (Month 1+)
8. **Consider async-first architecture** if OpenHands SDK adds native async support
9. **Add more ACP features** (auth methods, slash command registry)
10. **Improve tool call state management** with proper classes

---

## Conclusion

**Crow ACP Server Status**: 60% feature parity with Kimi-CLI

**Core functionality works**, but missing:
- Error handling (critical)
- Session listing (important)
- Model switching (important)
- Native terminal tool (nice-to-have)

**Recommendation**: Implement Phase 1-3 features to reach 90% parity. The OpenHands SDK foundation is solid - just need to add the missing ACP protocol features.
