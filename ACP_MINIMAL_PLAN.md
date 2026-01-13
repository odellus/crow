# Minimal ACP Server Implementation Plan

## Goal
Build a minimal ACP server in `crow/` that wraps OpenHands SDK with proper streaming.

## What We Know

### OpenHands SDK (working example in `crow_mcp_integration.py`)
- Direct imports: `from openhands.sdk import LLM, Agent, Conversation`
- Streaming: `on_token(chunk: ModelResponseStream)` callback
- Token types: `content`, `reasoning_content`, `tool_calls`
- State tracking: `_current_state` for boundary detection
- MCP integration: `mcp_config` dict with servers

### ACP Python SDK (to be explored)
Located in `/home/thomas/src/projects/orchestrator-project/acp-python-sdk/`
- Need to understand: Agent base class, Content types, JSON-RPC handling
- Key requirement: `session/update` notifications with `TextContent` + `chunkIndex`

### Karla Reference (READ ONLY)
- File: `karla/src/karla/acp_server.py`
- Patterns: `KarlaAgent` extends ACP base class
- Methods: `new_session()`, `load_session()`, `prompt()`, `cancel()`
- Streaming: `on_text_async()`, `on_reasoning_async()`, etc.

## Implementation Plan

### Phase 1: Understand ACP SDK (RESEARCH)
**File to create**: `crow/acp_research.py` (temporary, for exploration)

1. Explore acp-python-sdk structure
2. Find Agent base class
3. Find Content types (TextContent, etc.)
4. Find how to send `session/update` notifications
5. Document key imports and method signatures

**Questions to answer**:
- What base class do we extend? `from acp import Agent`?
- How do we send streaming updates?
- What's the signature for `prompt()`?
- How do we handle chunkIndex?

### Phase 2: Minimal ACP Server (IMPLEMENTATION)
**File to create**: `crow/crow_acp_server.py`

**Structure**:
```python
#!/usr/bin/env python3
"""
Minimal ACP server for OpenHands agent.
Streams responses properly with TextContent + chunkIndex.
"""

from acp import Agent, Content, TextContent
from openhands.sdk import LLM, Agent, Conversation
from openhands.tools.file_editor import FileEditorTool
from openhands.tools.terminal import TerminalTool
import asyncio

class CrowACPAgent(Agent):
    """ACP agent that wraps OpenHands SDK."""

    def __init__(self):
        super().__init__()
        # TODO: Initialize OpenHands LLM, Agent, Conversation
        self.llm = None
        self.agent = None
        self.conversation = None

    async def new_session(self, params):
        """Create a new session."""
        cwd = params.get("cwd", ".")
        mcp_servers = params.get("mcpServers", {})

        # Initialize OpenHands agent
        self.llm = LLM(...)
        self.agent = Agent(...)
        self.conversation = Conversation(...)

        return {"sessionId": "..."}

    async def prompt(self, params):
        """Handle prompt and stream response."""
        session_id = params["sessionId"]
        message = params["agentRequest"]

        # Token streaming callback that sends ACP updates
        def on_token(chunk):
            # Convert token to TextContent with chunkIndex
            text = extract_text(chunk)
            self.send_update(text, chunk_index)

        self.conversation.send_message(...)
        self.conversation.run()

    def send_update(self, text, chunk_index):
        """Send session/update notification."""
        # TODO: How does ACP SDK handle this?
        pass
```

**Key methods to implement**:
- `new_session(params)` - Initialize OpenHands agent
- `prompt(params)` - Send message, stream response
- `cancel(params)` - Cancel current operation
- `load_session(params)` - (optional) Resume existing session

### Phase 3: Proper Streaming (CRITICAL)

**Problem**: OpenHands-CLI sends internal events instead of ACP TextContent.

**Solution**: Intercept tokens with `on_token`, convert to ACP updates.

```python
# Global chunk counter
_chunk_index = 0

def on_token(chunk: ModelResponseStream) -> None:
    global _chunk_index

    for choice in chunk.choices:
        delta = choice.delta

        # Regular content
        if content := getattr(delta, "content", None):
            # Send ACP TextContent with chunkIndex
            self.send_session_update(
                content=TextContent(
                    type="text",
                    text=content,
                    chunk_index=_chunk_index,
                    chunk_total=-1  # Unknown total
                )
            )
            _chunk_index += 1
```

**Questions**:
- How do we send `session/update` via ACP SDK?
- What's the exact API for `TextContent`?
- Do we call a method or return a value?

### Phase 4: Test with Debug Client

**Use existing client**: `OpenHands-CLI/scripts/acp/debug_client.py`

Test sequence:
1. Start server: `./crow_acp_server.py`
2. Send `initialize`
3. Send `session/new`
4. Send `session/prompt` with "Hello"
5. Verify streaming: chunkIndex increments properly

### Phase 5: Polish & Documentation

**File to create**: `crow/README_ACP.md`

- How to run the server
- How to test with debug client
- How streaming works
- Dependencies (update `pyproject.toml` if needed)

## Immediate Next Steps

1. **Right now**: Explore acp-python-sdk to understand API
2. **Then**: Write `crow_acp_server.py` with minimal implementation
3. **Then**: Test with debug_client.py
4. **Then**: Fix any streaming issues

## Dependencies

Already in `crow/pyproject.toml`:
- `openhands-sdk` ✅
- `acp-python-sdk` ✅ (need to verify)

## Questions for Investigation

1. What's the exact import for ACP Agent base class?
2. How do we send `session/update` notifications?
3. What's the TextContent constructor signature?
4. Do we need to handle JSON-RPC manually or does ACP SDK handle it?
5. How do we run the server (what's the `main()` function)?

## Success Criteria

- Server starts and responds to `initialize`
- `session/new` creates OpenHands agent
- `session/prompt` streams responses with proper `chunkIndex`
- Can be tested with `debug_client.py`
- No internal SDK events leaked - only ACP protocol messages
