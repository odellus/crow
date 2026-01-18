# Future Work

## Context Condenser

### Problem
As conversations grow longer, they consume more tokens and increase latency. The OpenHands SDK sends the full conversation history to the LLM for each prompt, which can become expensive and slow.

### Solution
Implement a context condenser that:
1. **Summarizes old messages** - Compress early conversation turns into summaries
2. **Removes redundant content** - Eliminate duplicate information
3. **Preserves important context** - Keep critical decisions and state
4. **Maintains tool results** - Don't lose important tool outputs

### Implementation Plan

#### 1. Create Condenser Class
```python
class ContextCondenser:
    """Condenses conversation history to reduce token usage."""
    
    def __init__(self, max_tokens: int = 8000):
        self.max_tokens = max_tokens
        self.summarizer = LLM()  # Use cheaper model for summarization
    
    async def condense(self, conversation: Conversation) -> Conversation:
        """Condense conversation if it exceeds max_tokens."""
        current_tokens = self.count_tokens(conversation)
        
        if current_tokens <= self.max_tokens:
            return conversation  # No condensation needed
        
        # Summarize old messages
        summary = await self.summarize_old_messages(conversation)
        
        # Replace old messages with summary
        condensed = self.replace_with_summary(conversation, summary)
        
        return condensed
```

#### 2. Integrate with ACP Server
```python
async def prompt(self, session_id: str, prompt: list, **kwargs):
    session = self.sessions[session_id]
    conversation = session["conversation"]
    
    # Condense before sending message
    condenser = session.get("condenser")
    if condenser:
        conversation = await condenser.condense(conversation)
        session["conversation"] = conversation
    
    # Send message
    conversation.send_message(prompt_text)
    await loop.run_in_executor(executor, conversation.run)
```

#### 3. Summarization Strategy
- **Keep recent messages** (last 10 turns) - Full detail
- **Summarize middle messages** - Compressed representation
- **Keep system messages** - Always preserve
- **Keep tool results** - If they affect current state

Example:
```
[Original]
User: What files are in the current directory?
Agent: [tool: terminal, command: ls -la]
Tool Result: file1.py, file2.py, README.md
User: What's in file1.py?
Agent: [tool: file_editor, read file1.py]
Tool Result: def hello(): print("hi")
User: Create a new file
...

[Condensed]
[Summary: User explored directory structure, found 3 files including file1.py which contains a hello() function]
User: Create a new file
...
```

### Configuration
```python
condenser = ContextCondenser(
    max_tokens=8000,           # Target token count
    keep_recent=10,            # Keep last 10 turns
    summarize_threshold=0.8,   # Summarize when 80% full
    summary_model="gpt-3.5",   # Cheaper model for summaries
)
```

## MCP Integration Improvements

### Current State
- MCP servers can be configured in `session/new`
- Tools are loaded at session creation
- No dynamic tool loading during session

### Future Enhancements

#### 1. Dynamic MCP Server Loading
```python
async def load_mcp_server(self, session_id: str, mcp_config: dict):
    """Load MCP server mid-session."""
    session = self.sessions[session_id]
    
    # Load tools from MCP server
    tools = await load_mcp_server(mcp_config)
    
    # Add to agent
    session["agent"].add_tools(tools)
    
    # Notify client
    await self.send_update(session_id, {
        "@type": "tools_added",
        "tools": [t.name for t in tools],
    })
```

#### 2. MCP Tool Caching
- Cache tool schemas to avoid re-fetching
- Cache tool results for idempotent operations
- Invalidate cache on server restart

#### 3. MCP Server Health Monitoring
- Periodically check MCP server connectivity
- Automatically reconnect on failure
- Gracefully handle server unavailability

## Performance Optimizations

### 1. Streaming Improvements
- Batch multiple updates into single notification
- Use binary encoding for large content
- Implement backpressure for slow clients

### 2. Conversation Caching
- Cache conversation state in memory
- Lazy load from disk only when needed
- Implement LRU cache for multiple sessions

### 3. Parallel Tool Execution
```python
# Current: Sequential tool execution
for tool_call in tool_calls:
    result = await execute_tool(tool_call)

# Future: Parallel tool execution
results = await asyncio.gather(*[
    execute_tool(tc) for tc in tool_calls
])
```

### 4. Token Streaming Optimization
- Stream tokens directly to client without buffering
- Use chunked encoding for large responses
- Implement token compression

## Error Handling Improvements

### 1. Retry Logic
```python
async def prompt(self, session_id: str, prompt: list, **kwargs):
    max_retries = 3
    for attempt in range(max_retries):
        try:
            return await self._prompt_internal(session_id, prompt)
        except LLMError as e:
            if attempt == max_retries - 1:
                raise
            logger.warning(f"LLM error, retrying: {e}")
            await asyncio.sleep(2 ** attempt)  # Exponential backoff
```

### 2. Graceful Degradation
- If MCP server fails, continue without its tools
- If condenser fails, continue with full context
- If streaming fails, fall back to batch response

### 3. User-Friendly Error Messages
```python
try:
    await conversation.run()
except PermissionError:
    await self.send_update(session_id, {
        "@type": "error",
        "message": "You don't have permission to access that file.",
    })
except FileNotFoundError:
    await self.send_update(session_id, {
        "@type": "error",
        "message": f"File not found: {path}",
    })
```

## Security Enhancements

### 1. Permission System
- Fine-grained permissions for file access
- Network access controls
- Tool execution limits

### 2. Sandboxing
- Run tools in isolated environment
- Limit resource usage (CPU, memory, disk)
- Prevent access to sensitive files

### 3. Audit Logging
- Log all tool executions
- Log all file modifications
- Log all LLM calls
- Export logs for compliance

## Observability

### 1. Metrics Collection
```python
from prometheus_client import Counter, Histogram

prompt_counter = Counter("acp_prompts_total", "Total prompts")
prompt_duration = Histogram("acp_prompt_duration_seconds", "Prompt duration")
tool_execution_duration = Histogram("acp_tool_duration_seconds", "Tool execution duration")

async def prompt(self, session_id: str, prompt: list, **kwargs):
    prompt_counter.inc()
    with prompt_duration.time():
        # ... execute prompt ...
```

### 2. Distributed Tracing
```python
from opentelemetry import trace

tracer = trace.get_tracer(__name__)

async def prompt(self, session_id: str, prompt: list, **kwargs):
    with tracer.start_as_current_span("acp.prompt") as span:
        span.set_attribute("session_id", session_id)
        # ... execute prompt ...
```

### 3. Logging Improvements
- Structured logging (JSON format)
- Log levels (DEBUG, INFO, WARNING, ERROR)
- Correlation IDs for request tracking
- Sensitive data redaction

## Multi-Session Support

### 1. Session Sharing
Allow multiple clients to connect to the same session:
```python
async def session_attach(self, session_id: str, client_id: str):
    """Attach a client to an existing session."""
    session = self.sessions[session_id]
    session["clients"].add(client_id)
    
    # Replay conversation to new client
    for update in session["history"]:
        await self.send_to_client(client_id, update)
```

### 2. Session Collaboration
- Multiple users editing same files
- Shared terminal sessions
- Collaborative debugging

## Advanced Features

### 1. Conversation Export/Import
```python
async def export_session(self, session_id: str, format: str):
    """Export conversation to file."""
    session = self.sessions[session_id]
    
    if format == "markdown":
        return to_markdown(session["conversation"])
    elif format == "json":
        return to_json(session["conversation"])

async def import_session(self, session_id: str, data: str, format: str):
    """Import conversation from file."""
    if format == "markdown":
        conversation = from_markdown(data)
    # ... restore conversation ...
```

### 2. Custom Visualizers
Allow clients to provide custom visualization for tool results:
```python
await self.send_update(session_id, {
    "@type": "tool_result",
    "tool_call_id": "...",
    "result": "...",
    "visualization": {
        "type": "chart",
        "data": {...},
    }
})
```

### 3. Tool Composition
Allow tools to call other tools:
```python
@tool
def deploy_app(env: str):
    """Deploy application to environment."""
    # Calls: build_tool, test_tool, push_tool
    build_result = build_tool()
    test_result = test_tool(build_result)
    push_result = push_tool(test_result, env)
    return push_result
```

## Testing Improvements

### 1. Property-Based Testing
Use hypothesis to test edge cases:
```python
@given(st.lists(st.text()))
async def test_long_conversations(messages):
    """Test that condenser works for any conversation."""
    session = await create_session()
    for msg in messages:
        await prompt(session, msg)
    # Verify conversation is condensed
    assert count_tokens(session) < MAX_TOKENS
```

### 2. Load Testing
```python
async def test_concurrent_sessions():
    """Test 100 concurrent sessions."""
    sessions = [create_session() for _ in range(100)]
    
    # Send prompts to all sessions
    await asyncio.gather(*[
        prompt(s, "Hello") for s in sessions
    ])
    
    # Verify all sessions work
    for s in sessions:
        assert s["conversation"].history
```

### 3. Integration Tests
Test with real ACP clients:
```python
async def test_zed_integration():
    """Test with Zed editor."""
    # Start Zed with crow ACP server
    # Send prompts via Zed
    # Verify responses
```

## Documentation

### 1. API Documentation
Generate API docs from type hints:
```bash
sphinx-apidoc -o docs crow/src/crow/agent
```

### 2. Architecture Diagrams
- Sequence diagrams for message flow
- Component diagrams for system architecture
- State diagrams for session lifecycle

### 3. Troubleshooting Guide
Common issues and solutions:
- "Agent not responding" → Check LLM API key
- "Conversation not persisting" → Check disk permissions
- "Tools not working" → Check tool configuration

## Conclusion

This book has covered the implementation of the Crow ACP server, from initial setup to current state. The key insight is that **conversation reuse is critical** - without it, agents lose all context between prompts.

The next priority is implementing a **context condenser** to prevent unbounded conversation growth. This will enable long-running sessions without performance degradation.

The ACP protocol provides a solid foundation for building AI agents that can integrate with any client. The Crow implementation demonstrates how to bridge async protocols with synchronous SDKs while maintaining conversation state.
