# Implementation Details

## File Structure

```
crow/src/crow/agent/
├── acp_server.py          # Main ACP server implementation
├── __init__.py
└── ...
```

## Core Components

### 1. CrowAcpAgent Class

The main class that implements the ACP Agent interface:

```python
class CrowAcpAgent(Agent):
    """ACP agent implementation for Crow."""
    
    async def initialize(
        self,
        protocol_version: int,
        client_capabilities: ClientCapabilities,
        client_info: ClientInfo,
    ) -> InitializeResponse:
        """Handle initialize request."""
        
    async def new_session(
        self,
        cwd: str,
        mcp_servers: list[MCPServerConfig],
        **kwargs: Any,
    ) -> NewSessionResponse:
        """Create a new session."""
        
    async def prompt(
        self,
        session_id: str,
        prompt: list[PromptMessage],
        **kwargs: Any,
    ) -> PromptResponse:
        """Send a prompt to the agent."""
        
    async def cancel(self, session_id: str, **kwargs: Any) -> None:
        """Cancel an in-progress prompt."""
        
    async def load_session(
        self,
        session_id: str,
        cwd: str,
        **kwargs: Any,
    ) -> LoadSessionResponse:
        """Load an existing session."""
```

### 2. Session Management

Sessions are stored in a dict:

```python
# Instance variable
self.sessions: dict[str, dict[str, Any]] = {}
```

Each session contains:
```python
{
    "id": str,                    # Session UUID
    "cwd": str,                   # Working directory
    "agent": OpenHandsAgent,      # OpenHands Agent instance
    "conversation": Conversation, # OpenHands Conversation (REUSED!)
    "cancelled_flag": bool,       # For cancellation
    "mcp_servers": list,          # MCP server configs
}
```

### 3. The Conversation Reuse Pattern

This is the **most critical** part of the implementation.

#### new_session() - Create Conversation Once

```python
async def new_session(self, cwd: str, mcp_servers: list, **kwargs):
    session_id = str(uuid.uuid4())
    
    # Create OpenHands Agent
    oh_agent = Agent(llm=llm, tools=[...])
    
    # Create Conversation ONCE per session
    conversation = Conversation(
        agent=oh_agent,
        workspace=cwd,
        visualizer=None,
    )
    
    # Store in session
    self.sessions[session_id] = {
        "id": session_id,
        "cwd": cwd,
        "agent": oh_agent,
        "conversation": conversation,  # Will be reused!
        "cancelled_flag": False,
    }
    
    return NewSessionResponse(session_id=session_id)
```

#### prompt() - Reuse Conversation

```python
async def prompt(self, session_id: str, prompt: list, **kwargs):
    session = self.sessions[session_id]
    
    # CRITICAL: Reuse existing Conversation
    if "conversation" not in session:
        # This should NEVER happen after new_session
        conversation = Conversation(...)
        session["conversation"] = conversation
    else:
        # Reuse the same Conversation object
        conversation = session["conversation"]
    
    # Send message to the SAME conversation
    conversation.send_message(prompt_text)
    
    # Run the conversation (sync in thread pool)
    await loop.run_in_executor(None, conversation.run)
```

**Why this matters:**
- Conversation maintains message history
- LLM sees full context from all previous prompts
- Tools maintain state across prompts
- Agent "remembers" everything in the session

### 4. Async to Sync Bridge

OpenHands `Conversation.run()` is synchronous and blocking. ACP is async. We bridge them:

```python
async def prompt(self, session_id: str, prompt: list, **kwargs):
    # ... setup ...
    
    # Create a thread pool executor
    loop = asyncio.get_event_loop()
    executor = ThreadPoolExecutor(max_workers=1)
    
    # Run the sync conversation.run() in a thread
    await loop.run_in_executor(
        executor,
        conversation.run,
    )
```

This prevents blocking the async event loop while the agent thinks and acts.

### 5. Streaming Architecture

OpenHands uses token callbacks for streaming. ACP uses async notifications. We bridge them:

```python
async def prompt(self, session_id: str, prompt: list, **kwargs):
    # Create a queue for streaming updates
    update_queue = asyncio.Queue()
    
    # Define token callback that puts updates in queue
    def on_token(token: Token):
        asyncio.run_coroutine_threadsafe(
            update_queue.put(("token", token)),
            loop,
        )
    
    # Attach callback to conversation
    conversation._on_token = BaseConversation.compose_callbacks([on_token])
    
    # Start background task to read from queue and send notifications
    async def sender_task():
        while True:
            update_type, data = await update_queue.get()
            
            if update_type == "token":
                # Convert token to ACP update
                await self.send_update(session_id, data)
            
            elif update_type == "done":
                break
    
    sender = asyncio.create_task(sender_task())
    
    # Run conversation (which calls on_token)
    await loop.run_in_executor(executor, conversation.run)
    
    # Wait for sender to finish
    await sender
```

### 6. Token to ACP Update Mapping

We map OpenHands tokens to ACP updates:

```python
def token_to_acp_update(token: Token) -> dict:
    if token.token_type == "thinking":
        return {
            "@type": "agent_thought_text",
            "text": token.content,
        }
    
    elif token.token_type == "message":
        return {
            "@type": "agent_message_text",
            "text": token.content,
        }
    
    elif token.token_type == "tool_start":
        return {
            "@type": "tool_call_start",
            "tool_call_id": token.tool_call_id,
            "title": token.tool_name,
            "kind": "execute",
            "status": "in_progress",
        }
    
    elif token.token_type == "tool_end":
        return {
            "@type": "tool_call_update",
            "tool_call_id": token.tool_call_id,
            "status": "completed",
        }
```

### 7. Session Persistence

Sessions are saved to disk for resume capability:

```python
async def new_session(self, cwd: str, mcp_servers: list, **kwargs):
    # ... create session ...
    
    # Save to disk
    session_path = Path.home() / ".crow" / "sessions" / f"{session_id}.json"
    session_path.parent.mkdir(parents=True, exist_ok=True)
    
    session_data = {
        "id": session_id,
        "cwd": cwd,
        "mcp_servers": mcp_servers,
        "created_at": datetime.now().isoformat(),
    }
    
    session_path.write_text(json.dumps(session_data, indent=2))
```

When loading a session:
```python
async def load_session(self, session_id: str, cwd: str, **kwargs):
    # Load from disk
    session_path = Path.home() / ".crow" / "sessions" / f"{session_id}.json"
    session_data = json.loads(session_path.read_text())
    
    # Recreate session
    # ... (same as new_session, but with existing data)
    
    # Replay conversation history via updates
    for message in conversation.get_history():
        await self.send_update(session_id, {
            "@type": "agent_message_text",
            "text": message.content,
        })
```

### 8. Cancellation Support

Cancellation is handled via a flag:

```python
async def prompt(self, session_id: str, prompt: list, **kwargs):
    session = self.sessions[session_id]
    cancelled_flag = asyncio.Event()
    session["cancelled_flag"] = cancelled_flag
    
    # In the conversation runner:
    def run_with_cancellation():
        while not conversation.done():
            if cancelled_flag.is_set():
                conversation.cancel()
                break
            time.sleep(0.1)
    
    await loop.run_in_executor(executor, run_with_cancellation)

async def cancel(self, session_id: str, **kwargs):
    session = self.sessions[session_id]
    session["cancelled_flag"].set()
```

## Key Implementation Insights

### 1. Don't Pop the Conversation
**Critical bug we fixed:** Line 636 was doing `session.pop("conversation", None)` after each prompt. This removed the Conversation, causing a new one to be created on the next prompt.

**Fix:** Never pop the conversation. Only pop temporary state like `cancelled_flag`.

### 2. Thread Safety
The `run_in_executor` runs in a separate thread. The token callback runs in that thread but needs to put items in an async queue. Use `asyncio.run_coroutine_threadsafe()` to bridge threads.

### 3. Queue Cleanup
Always signal the sender task to finish before returning from `prompt()`:
```python
await update_queue.put(("done", None))
await sender
```

### 4. Error Handling
Wrap the conversation runner in try/except:
```python
try:
    await loop.run_in_executor(executor, conversation.run)
except Exception as e:
    logger.error(f"Conversation failed: {e}")
    return PromptResponse(stop_reason="error")
```

### 5. MCP Integration
MCP servers are loaded during `new_session()`:
```python
async def new_session(self, cwd: str, mcp_servers: list, **kwargs):
    # Load MCP tools
    mcp_tools = []
    for mcp_config in mcp_servers:
        tools = await load_mcp_server(mcp_config)
        mcp_tools.extend(tools)
    
    # Create agent with MCP tools
    oh_agent = Agent(
        llm=llm,
        tools=default_tools + mcp_tools,
    )
```

## Testing

Test ACP servers using subprocess and pipes:

```python
proc = subprocess.Popen(
    ["crow", "acp"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
)

# Send request
request = {"jsonrpc": "2.0", "id": 1, "method": "initialize", ...}
proc.stdin.write(json.dumps(request) + "\n")
proc.stdin.flush()

# Read response
response = json.loads(proc.stdout.readline())
```

See `crow/test_acp_manual.py` for a complete test example.
