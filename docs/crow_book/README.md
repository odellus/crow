# Crow ACP Server Book

A comprehensive guide to the Crow ACP Server implementation.

## Table of Contents

1. [Introduction](01_introduction.md)
   - What is Crow?
   - What is ACP?
   - Architecture overview
   - Key design decisions

2. [ACP Protocol Deep Dive](02_acp_protocol.md)
   - Message flow
   - Update types
   - Session management
   - Implementation notes

3. [Implementation Details](03_implementation.md)
   - Core components
   - Conversation reuse pattern
   - Async to sync bridge
   - Streaming architecture
   - Session persistence
   - Cancellation support

4. [Steps Taken to Date](04_steps_taken.md)
   - Project setup
   - ACP server implementation
   - Critical bug fix
   - Testing infrastructure
   - Frontend integration
   - Current status
   - Lessons learned

5. [Future Work](05_future_work.md)
   - Context condenser
   - MCP integration improvements
   - Performance optimizations
   - Error handling improvements
   - Security enhancements
   - Observability
   - Multi-session support
   - Advanced features

## Quick Start

### Running the Server

```bash
# Install dependencies
cd crow
uv sync

# Start ACP server
crow acp
```

### Testing

```bash
# Manual test
python test_acp_manual.py

# Conversation memory test
python test_conversation_memory.py
```

### Integration

```bash
# Start with stdio-to-ws for browser clients
npx stdio-to-ws "uv run --project crow crow acp" --port 3027
```

## Key Takeaways

### 1. Conversation Reuse is Critical
The most important design pattern is reusing the Conversation object across prompts. Without this, the agent has no memory.

### 2. Async/Sync Bridge
ACP is async, OpenHands SDK is sync. Use `run_in_executor` to bridge them without blocking.

### 3. Streaming via Queues
Token callbacks happen in worker threads. Use async queues to bridge to the main event loop.

### 4. Session State Management
Be careful about what you store in sessions. Don't pop the conversation!

### 5. Test with Real Clients
ACP servers are complex. Test with subprocess and real clients, not just unit tests.

## References

- [ACP Specification](https://agentclientprotocol.com)
- [OpenHands SDK](https://github.com/OpenHands/software-agent-sdk)
- [OpenHands-CLI](https://github.com/OpenHands/OpenHands-CLI)
- [Crow IDE](../crow_ide/)

## Contributing

Contributions welcome! Please read the implementation details first to understand the architecture.

## License

MIT
