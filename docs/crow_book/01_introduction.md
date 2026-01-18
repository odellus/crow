# Crow ACP Server - Introduction

## What is Crow?

Crow is an AI agent framework built on top of the OpenHands SDK. It provides a flexible, extensible platform for building AI-powered agents that can interact with systems through tools, maintain conversation state, and leverage Model Context Protocol (MCP) for dynamic tool integration.

## What is ACP?

The Agent Client Protocol (ACP) is a standardized protocol for communication between AI agents and client applications. It defines:

- **JSON-RPC 2.0** based messaging over stdio
- **Session management** - create, load, resume sessions
- **Streaming responses** - real-time updates as the agent thinks and acts
- **Tool calls** - structured execution of operations with permission handling
- **State persistence** - save and restore conversation state

ACP enables agents to be integrated into various clients (Zed, VS Code, custom IDEs) through a common protocol.

## The Crow ACP Server

The Crow ACP Server (`crow/src/crow/agent/acp_server.py`) is an ACP-compliant server that:

1. **Wraps the OpenHands SDK** - Uses OpenHands Conversation, Agent, and tool system
2. **Implements ACP protocol** - Handles initialize, session/new, session/prompt, etc.
3. **Streams responses** - Sends real-time updates for thinking, content, and tool calls
4. **Maintains conversation state** - Reuses Conversation objects across prompts
5. **Supports MCP** - Can load external tools from MCP servers

## Architecture

```
┌─────────────────┐
│   ACP Client    │ (Zed, VS Code, Crow IDE, etc.)
└────────┬────────┘
         │ stdio (JSON-RPC)
         ▼
┌─────────────────────────────────┐
│   Crow ACP Server               │
│   - acp_server.py               │
│   - Implements ACP interface     │
│   - Manages sessions             │
└────────┬────────────────────────┘
         │
         ▼
┌─────────────────────────────────┐
│   OpenHands SDK                 │
│   - Conversation (state mgmt)    │
│   - Agent (LLM + tools)         │
│   - Tool execution              │
└────────┬────────────────────────┘
         │
         ▼
┌─────────────────────────────────┐
│   Tools                         │
│   - Terminal                    │
│   - File Editor                 │
│   - MCP Tools (dynamic)         │
└─────────────────────────────────┘
```

## Key Design Decisions

### 1. Conversation Reuse
The most critical design decision is **Conversation object reuse**. Each ACP session creates ONE Conversation object that persists across all prompts. This maintains conversation history and context.

**Wrong approach** (what we initially did):
```python
async def prompt():
    # Create new Conversation for each prompt - loses context!
    conversation = Conversation(agent=agent, workspace=cwd)
    conversation.send_message(message)
    conversation.run()
```

**Correct approach** (current implementation):
```python
async def new_session():
    # Create Conversation once
    session["conversation"] = Conversation(agent=agent, workspace=cwd)

async def prompt():
    # Reuse the same Conversation
    conversation = session["conversation"]
    conversation.send_message(message)
    conversation.run()
```

### 2. Async to Sync Bridge
ACP is async (asyncio), but OpenHands Conversation.run() is synchronous. We use `loop.run_in_executor()` to bridge this gap without blocking the event loop.

### 3. Streaming via Queues
The OpenHands SDK uses token callbacks for streaming. We bridge these to ACP's async notification system using an `asyncio.Queue`:
- Token callback → queue.put()
- Background task → queue.get() → session/update notification

### 4. Session Persistence
Sessions are persisted to disk (`~/.crow/sessions/{session_id}.json`) for:
- Resume after server restart
- Multi-client scenarios
- Debugging and audit trails

## What This Book Covers

1. **ACP Protocol Deep Dive** - How ACP works, message flow, state management
2. **Implementation Details** - How we built the Crow ACP server
3. **Conversation State Management** - The critical reuse pattern
4. **Streaming Architecture** - How we bridge sync SDK to async ACP
5. **Tool Execution** - How tools are called and permissions handled
6. **MCP Integration** - Dynamic tool loading from MCP servers
7. **Testing and Debugging** - How to test ACP servers
8. **Future Work** - Condensers, improvements, optimizations
