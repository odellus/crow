# ACP Server Implementation Plan

## Priority 1: Error Handling (CRITICAL - Week 1)

### Current State
```python
# No try/except - errors crash the conversation
await loop.run_in_executor(None, run_conversation)
```

### Target State
```python
async def prompt(self, prompt: list, session_id: str, **kwargs):
    try:
        # ... existing code ...
        await loop.run_in_executor(None, run_conversation)
    except openhands.sdk.errors.MaxIterationsError as e:
        logger.error("Max iterations reached: {e}", e=e)
        return PromptResponse(stop_reason="max_turn_requests")
    except openhands.sdk.errors.SecurityError as e:
        logger.error("Security policy violation: {e}", e=e)
        raise acp.RequestError.forbidden({"error": str(e)})
    except Exception as e:
        logger.exception("Unexpected error during prompt:")
        raise acp.RequestError.internal_error({
            "error": str(e),
            "type": type(e).__name__,
        })
```

### Implementation Steps
1. Identify OpenHands SDK exception types
2. Map OpenHands exceptions to ACP errors
3. Add try/except to `prompt()` method
4. Add full stack traces to error responses
5. Test with various failure scenarios

**Estimated effort**: 2-3 hours

---

## Priority 2: Session Listing (HIGH - Week 1)

### Current State
```python
# Not implemented
```

### Target State
```python
async def list_sessions(
    self,
    cwd: str | None = None,
    cursor: str | None = None,
    **kwargs,
) -> acp.schema.ListSessionsResponse:
    """List all sessions for a working directory."""
    if not cwd:
        return acp.schema.ListSessionsResponse(sessions=[], next_cursor=None)
    
    sessions_dir = Path.home() / ".crow" / "sessions"
    sessions = []
    
    for session_file in sorted(sessions_dir.glob("*.json"), key=lambda p: p.stat().st_mtime, reverse=True):
        try:
            with open(session_file) as f:
                data = json.load(f)
                
            # Only return sessions for this cwd
            if data.get("cwd") != cwd:
                continue
            
            sessions.append(acp.schema.SessionInfo(
                cwd=cwd,
                session_id=data["session_id"],
                title=data.get("title", "Untitled Session"),
                updated_at=datetime.fromtimestamp(session_file.stat().st_mtime).isoformat(),
            ))
        except Exception as e:
            logger.warning("Failed to load session {file}: {e}", file=session_file, e=e)
    
    return acp.schema.ListSessionsResponse(
        sessions=sessions[:50],  # Limit to 50 sessions
        next_cursor=None,
    )
```

### Implementation Steps
1. Add `list_sessions()` method to `CrowAcpAgent`
2. Read session files from `~/.crow/sessions/`
3. Filter by `cwd` if provided
4. Return session metadata (id, title, updated_at)
5. Add session titles (derive from first user message)

**Estimated effort**: 2-3 hours

---

## Priority 3: Model Switching (HIGH - Week 2)

### Current State
```python
# Model fixed at agent creation
self._llm = LLM(model=self._llm_config.model, ...)
```

### Target State
```python
async def set_session_model(
    self,
    model_id: str,
    session_id: str,
    **kwargs,
) -> None:
    """Change the LLM for a session."""
    if session_id not in self._sessions:
        raise acp.RequestError.invalid_params({"session_id": "Session not found"})
    
    session = self._sessions[session_id]
    agent = session["agent"]
    
    # Parse model_id (may have ",thinking" suffix)
    thinking = False
    if model_id.endswith(",thinking"):
        model_id = model_id[:-len(",thinking")]
        thinking = True
    
    # Create new LLM instance
    new_llm = LLM(
        model=model_id,
        api_key=self._llm_config.api_key,
        base_url=self._llm_config.base_url,
        stream=True,
        reasoning_effort="high" if thinking else "medium",
    )
    
    # Replace agent's LLM
    agent._llm = new_llm
    session["model_id"] = model_id
    session["thinking"] = thinking
    
    # Notify client
    await self._conn.session_update(
        session_id=session_id,
        update={
            "type": "model_update",
            "model_id": model_id,
            "thinking": thinking,
        },
    )
    
    # Save to session
    self._save_session(session_id)
```

### Implementation Steps
1. Add `set_session_model()` method
2. Parse model_id (handle `,thinking` suffix)
3. Create new LLM instance with updated config
4. Replace agent's `_llm` attribute
5. Send session update notification
6. Save to session persistence

**Estimated effort**: 3-4 hours

---

## Priority 4: Terminal Tool Replacement (MEDIUM - Week 2)

### Current State
```python
# Uses OpenHands TerminalTool (outputs to stdout/stderr)
tools = [Tool(name=TerminalTool.name), Tool(name=FileEditorTool.name)]
```

### Target State
```python
class ACPTerminalTool:
    """ACP-native terminal tool that streams to client."""
    
    def __init__(self, acp_conn, session_id):
        self._acp_conn = acp_conn
        self._session_id = session_id
    
    async def execute(self, command: str, timeout: float = 30):
        # Create ACP terminal
        terminal = await self._acp_conn.create_terminal(
            command=command,
            session_id=self._session_id,
            output_byte_limit=100_000,
        )
        
        # Send terminal content to client
        await self._acp_conn.session_update(
            session_id=self._session_id,
            update=acp.schema.ToolCallProgress(
                session_update="tool_call_update",
                tool_call_id=tool_call_id,
                status="in_progress",
                content=[acp.schema.TerminalToolCallContent(
                    type="terminal",
                    terminal_id=terminal.id,
                )],
            ),
        )
        
        # Wait for completion
        exit_status = await terminal.wait_for_exit()
        output = await terminal.current_output()
        
        await terminal.release()
        
        return output.output, exit_status.exit_code

# Replace in tool setup
tools = [
    Tool(name=FileEditorTool.name),
    ACPTerminalTool(conn=self._conn, session_id=session_id),
]
```

### Implementation Steps
1. Create `ACPTerminalTool` class
2. Implement `execute()` using ACP terminal API
3. Handle timeouts and truncation
4. Replace TerminalTool with ACPTerminalTool
5. Test with various commands

**Estimated effort**: 4-5 hours

---

## Priority 5: Session Metadata (MEDIUM - Week 2)

### Current State
```python
# Session only stores id, cwd, mode, conversation_history
session_data = {
    "session_id": session_id,
    "cwd": cwd,
    "mode": "default",
    "conversation_history": [],
}
```

### Target State
```python
# Add title and timestamps
session_data = {
    "session_id": session_id,
    "cwd": cwd,
    "mode": "default",
    "title": self._generate_title(user_message),
    "created_at": datetime.now().isoformat(),
    "updated_at": datetime.now().isoformat(),
    "conversation_history": [],
    "model_id": self._llm_config.model,
    "thinking": False,
}

def _generate_title(self, user_message: str) -> str:
    """Generate a session title from the first user message."""
    # Truncate to 50 chars
    title = user_message[:50].strip()
    if len(user_message) > 50:
        title += "..."
    return title or "Untitled Session"
```

### Implementation Steps
1. Add `title`, `created_at`, `updated_at`, `model_id` fields
2. Generate title from first user message
3. Update timestamps on each prompt
4. Save to session persistence
5. Display in session list

**Estimated effort**: 1-2 hours

---

## Priority 6: Better Cancellation (LOW - Week 3)

### Current State
```python
conversation.pause()  # Only waits for current LLM call
```

### Target State
```python
# Store cancellation future in session
session["cancel_future"] = None

def run_conversation():
    try:
        # Store conversation for cancellation
        nonlocal conversation
        session["conversation"] = conversation
        
        conversation.send_message(user_message)
        conversation.run()
    finally:
        session.pop("conversation", None)
        session.pop("cancel_future", None)

async def cancel(self, session_id: str, **kwargs):
    if session_id not in self._sessions:
        raise ValueError(f"Unknown session: {session_id}")
    
    session = self._sessions[session_id]
    
    # Set cancellation flag
    if "cancelled_flag" in session:
        session["cancelled_flag"]["cancelled"] = True
    
    # Pause conversation (waits for current LLM call)
    if "conversation" in session:
        conversation = session["conversation"]
        conversation.pause()
        
        # Wait for pause to complete
        # (OpenHands SDK doesn't support hard cancellation yet)
```

### Implementation Steps
1. Store conversation in session for cancellation
2. Set cancelled_flag before pausing
3. Wait for pause to complete
4. Consider thread termination (risky)
5. Document limitations

**Estimated effort**: 2-3 hours

---

## Testing Strategy

### Unit Tests
```python
# tests/test_acp_server_error_handling.py
@pytest.mark.asyncio
async def test_prompt_handles_max_iterations():
    """Test that MaxIterationsError returns correct stop_reason."""
    agent = CrowAcpAgent()
    session = await agent.new_session(cwd="/tmp")
    
    # Mock conversation to raise MaxIterationsError
    with patch('openhands.sdk.Conversation.run') as mock_run:
        mock_run.side_effect = MaxIterationsError("Max iterations reached")
        
        response = await agent.prompt(
            prompt=[{"type": "text", "text": "test"}],
            session_id=session["session_id"],
        )
        
        assert response.stop_reason == "max_turn_requests"
```

### Integration Tests
```python
# tests/test_acp_session_list.py
@pytest.mark.asyncio
async def test_list_sessions_filters_by_cwd():
    """Test that list_sessions only returns sessions for given cwd."""
    agent = CrowAcpAgent()
    
    # Create sessions in different directories
    session1 = await agent.new_session(cwd="/tmp/test1")
    session2 = await agent.new_session(cwd="/tmp/test2")
    
    # List sessions for /tmp/test1
    response = await agent.list_sessions(cwd="/tmp/test1")
    
    assert len(response.sessions) == 1
    assert response.sessions[0].session_id == session1["session_id"]
```

### Manual Testing
```bash
# Test error handling
echo '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "session/prompt",
  "params": {
    "sessionId": "test",
    "prompt": [{"type": "text", "text": "cause an error"}]
  }
}' | uv run --project . crow-acp

# Test session listing
echo '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "session/list",
  "params": {"cwd": "/tmp"}
}' | uv run --project . crow-acp
```

---

## Success Criteria

### Week 1 (Critical)
- ✅ All errors caught and returned as ACP errors
- ✅ Session listing works
- ✅ Tests pass for error handling
- ✅ Tests pass for session listing

### Week 2 (High Priority)
- ✅ Model switching works
- ✅ Terminal tool replaced with ACP native
- ✅ Session titles and timestamps
- ✅ Integration tests pass

### Week 3 (Low Priority)
- ✅ Better cancellation implemented
- ✅ Full test coverage
- ✅ Documentation updated

---

## Risk Assessment

### High Risk
- **Terminal tool replacement**: May break existing workflows
- **Model switching**: OpenHands SDK may not support runtime LLM replacement

### Medium Risk
- **Error handling**: Need to identify all OpenHands exception types
- **Session listing**: May have performance issues with many sessions

### Low Risk
- **Session metadata**: Simple additions
- **Cancellation**: Limited by OpenHands SDK capabilities

### Mitigation Strategies
1. Add feature flags for new functionality
2. Extensive testing before deployment
3. Gradual rollout with monitoring
4. Fallback to old behavior on errors

---

## Next Steps

1. **Review and prioritize** this plan with the team
2. **Set up branch** for ACP improvements
3. **Start with Priority 1** (error handling)
4. **Add tests** for each feature
5. **Document changes** in AGENTS.md
6. **Release** as v0.2.0 with all improvements

**Total estimated effort**: 15-20 hours over 3 weeks
