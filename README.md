<p align="center">
    <img src="assets/crow-logo.png" description="crow logo"width=500/>
</p>

A minimal [Agent Client Protocol (ACP)](https://agentclientprotocol.com/) server that wraps the OpenHands SDK with streaming support, MCP tool integration, and proper ACP protocol compliance.

## Features

- ✅ **ACP Protocol Compliance**: Implements the ACP specification for agent-client communication
- ✅ **Streaming Responses**: Real-time streaming of agent thoughts, content, and tool calls
- ✅ **Tool Call Reporting**: Full visibility into tool execution with status updates
- ✅ **MCP Integration**: Support for Model Context Protocol (MCP) servers
- ✅ **Cancellation**: Support for cancelling ongoing operations
- ✅ **Session Management**: Multiple concurrent sessions with unique IDs
- ✅ **Session Modes**: Different operational modes (default, code, chat)
- ✅ **Slash Commands**: Built-in commands for common operations (/help, /clear, /status)
- ✅ **Session Persistence**: Save and restore sessions with conversation history
- ✅ **Content Types**: Support for text, images, and embedded resources
- ✅ **OpenHands SDK**: Leverages the power of OpenHands for AI agent capabilities

## Installation

### Prerequisites

- Python 3.12 or higher
- uv (recommended) or pip for package management
- An LLM API key (e.g., Anthropic, OpenAI, or compatible)

### Setup

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd crow
   ```

2. **Install dependencies**:
   ```bash
   uv sync
   # or
   pip install -e .
   ```

3. **Configure environment variables**:
   Create a `.env` file in the project root:
   ```env
   # LLM Configuration
   LLM_MODEL=anthropic/claude-3.5-sonnet
   ZAI_API_KEY=your-api-key-here
   ZAI_BASE_URL=https://api.anthropic.com  # Optional
   
   # Server Configuration (optional)
   SERVER_NAME=crow-acp-server
   SERVER_VERSION=0.1.0
   SERVER_TITLE="Crow ACP Server"
   
   # Agent Configuration (optional)
   MAX_ITERATIONS=500
   AGENT_TIMEOUT=300
   ```

## Usage

### Starting the Server

Run the ACP server using Python:

```bash
python -m crow.agent.acp_server
```

The server will start listening for JSON-RPC messages on stdin and write responses to stdout, following the ACP protocol.

### ACP Protocol Methods

#### `initialize`

Initialize the ACP server and negotiate protocol version.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": 1,
    "clientCapabilities": {},
    "clientInfo": {
      "name": "my-client",
      "version": "1.0.0"
    }
  }
}
```

#### `session/new`

Create a new session for a conversation.

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "session/new",
  "params": {
    "cwd": "/path/to/workspace",
    "mcpServers": []
  }
}
```

#### `session/prompt`

Send a prompt to the agent with streaming responses.

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "session/prompt",
  "params": {
    "sessionId": "<session-id>",
    "prompt": [
      {
        "type": "text",
        "text": "Hello, how can you help me?"
      }
    ]
  }
}
```

#### `session/cancel`

Cancel an ongoing prompt execution.

```json
{
  "jsonrpc": "2.0",
  "method": "session/cancel",
  "params": {
    "sessionId": "<session-id>"
  }
}
```

### Streaming Updates

The server sends `session/update` notifications during prompt execution:

- **`agent_thought_chunk`**: Agent's thinking/reasoning
- **`agent_message_chunk`**: Agent's response content
- **`tool_call`**: Tool execution starts
- **`tool_call_update`**: Tool execution progress

Example update notification:
```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "sessionId": "<session-id>",
    "update": {
      "sessionUpdate": "agent_thought_chunk",
      "content": {
        "text": "Let me think about this..."
      }
    }
  }
}
```

## Configuration

### LLM Configuration

Configure the LLM provider using environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `LLM_MODEL` | Model identifier | `anthropic/glm-4.7` |
| `ZAI_API_KEY` | API key for LLM provider | *required* |
| `ZAI_BASE_URL` | Base URL for LLM API | *provider default* |
| `LLM_TEMPERATURE` | Sampling temperature | `0.0` |
| `LLM_MAX_TOKENS` | Maximum tokens per response | `4096` |

### Server Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_NAME` | Server identifier | `crow-acp-server` |
| `SERVER_VERSION` | Server version | `0.1.0` |
| `SERVER_TITLE` | Human-readable title | `Crow ACP Server` |

### Agent Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `MAX_ITERATIONS` | Maximum agent iterations | `500` |
| `AGENT_TIMEOUT` | Agent timeout in seconds | `300` |

## MCP Integration

The server supports MCP (Model Context Protocol) servers for extending tool capabilities. Configure MCP servers when creating a session:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "session/new",
  "params": {
    "cwd": "/workspace",
    "mcpServers": [
      {
        "name": "fetch",
        "command": "uvx",
        "args": ["mcp-server-fetch"]
      }
    ]
  }
}
```

## Session Modes

The server supports different operational modes that can be set per session:

### Available Modes

- **`default`**: Standard agent behavior with full tool access
- **`code`**: Focused on code generation and editing tasks
- **`chat`**: Conversational mode with minimal tool usage

### Setting a Mode

Use the `session/setMode` method to change the session mode:

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "session/setMode",
  "params": {
    "sessionId": "<session-id>",
    "modeId": "code"
  }
}
```

The server will send a `current_mode_update` notification when the mode changes successfully.

## Slash Commands

The server supports built-in slash commands for common operations:

### `/help`

Display help information and available commands.

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "session/prompt",
  "params": {
    "sessionId": "<session-id>",
    "prompt": [
      {
        "type": "text",
        "text": "/help"
      }
    ]
  }
}
```

### `/clear`

Clear the current conversation context for the session.

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "session/prompt",
  "params": {
    "sessionId": "<session-id>",
    "prompt": [
      {
        "type": "text",
        "text": "/clear"
      }
    ]
  }
}
```

### `/status`

Show current session status and configuration.

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "session/prompt",
  "params": {
    "sessionId": "<session-id>",
    "prompt": [
      {
        "type": "text",
        "text": "/status"
      }
    ]
  }
}
```

## Session Persistence

The server supports saving and loading sessions with full conversation history.

### Session Storage

Sessions are automatically saved to disk in the `~/.crow/sessions/` directory. Each session is stored as a JSON file named `<session-id>.json`.

### Loading a Session

Use the `session/load` method to restore a previously saved session:

```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "method": "session/load",
  "params": {
    "sessionId": "<saved-session-id>",
    "cwd": "/workspace",
    "mcpServers": []
  }
}
```

When a session is loaded, the server will:
1. Restore session metadata (cwd, mode)
2. Replay the entire conversation history via `session/update` notifications
3. Restore the conversation context for continued interaction

**Note**: According to the ACP specification, the agent MUST replay the entire conversation to the client when loading a session. This ensures the client can reconstruct the full conversation context.

### Session File Format

Session files are stored in JSON format:

```json
{
  "session_id": "uuid-string",
  "cwd": "/workspace",
  "mode": "default",
  "conversation_history": [
    {
      "role": "user",
      "content": "User's message"
    },
    {
      "role": "assistant",
      "parts": [
        {
          "type": "thought",
          "text": "Agent's thinking"
        },
        {
          "type": "text",
          "text": "Agent's response"
        },
        {
          "type": "tool_call",
          "id": "call-id",
          "name": "tool_name",
          "arguments": "{...}"
        }
      ]
    }
  ]
}
```

## Content Types

The server supports multiple content types in prompts, enabling rich interactions.

### Text Content

Standard text messages:

```json
{
  "type": "text",
  "text": "Hello, agent!"
}
```

### Image Content

Image content (placeholder for future vision support):

```json
{
  "type": "image",
  "data": "base64-encoded-image-data",
  "mime_type": "image/png"
}
```

**Note**: Image content is currently noted but not processed. Vision capabilities will be added in a future update.

### Embedded Resources

Embed external resources directly in prompts:

**Text Resource:**
```json
{
  "type": "resource",
  "resource": {
    "type": "text",
    "text": "Embedded text content",
    "uri": "file:///example.txt"
  }
}
```

**Blob Resource:**
```json
{
  "type": "resource",
  "resource": {
    "type": "blob",
    "blob": "base64-encoded-binary-data",
    "uri": "file:///example.bin",
    "mime_type": "application/octet-stream"
  }
}
```

### Mixed Content

You can combine multiple content types in a single prompt:

```json
{
  "jsonrpc": "2.0",
  "id": 9,
  "method": "session/prompt",
  "params": {
    "sessionId": "<session-id>",
    "prompt": [
      {
        "type": "text",
        "text": "Analyze this image: "
      },
      {
        "type": "image",
        "data": "base64-data",
        "mime_type": "image/jpeg"
      },
      {
        "type": "text",
        "text": " What do you see?"
      }
    ]
  }
}
```

## Tool Call Reporting

The server reports tool execution to the client following the ACP specification:

1. **Tool Start**: `tool_call` notification with status `in_progress`
2. **Tool Progress**: `tool_call_update` notifications during execution
3. **Tool Completion**: `tool_call_update` with status `completed` or `failed`

Tool kinds are mapped as follows:
- `execute`: Terminal, run, execute commands
- `edit`: File editing operations
- `read`: File reading operations
- `search`: Search operations
- `delete`: Delete operations
- `move`: Move/rename operations
- `other`: Other tool types

## Testing

Run the test suite:

```bash
# Run all tests
python -m pytest tests/

# Run specific test
python -m pytest tests/test_acp_simple.py

# Run with verbose output
python -m pytest tests/ -v
```

### Test Files

- `test_acp_simple.py`: Basic ACP flow test
- `test_acp_cancellation.py`: Cancellation functionality test
- `test_acp_server.py`: Server integration test
- `test_session_modes.py`: Session modes functionality tests
- `test_slash_commands.py`: Slash commands functionality tests
- `test_session_persistence.py`: Session persistence and conversation replay tests
- `test_content_types.py`: Content type handling tests
- `test_utils.py`: Utility function tests

## Troubleshooting

### Issue: "Unknown session" error

**Cause**: Invalid session ID or session expired.

**Solution**: Ensure you're using the session ID returned by `session/new`. Session IDs are UUIDs and should not be modified.

### Issue: Cancellation doesn't work immediately

**Cause**: The current implementation uses "soft" cancellation that waits for the current LLM call to complete.

**Solution**: This is a known limitation. Cancellation will take effect after the current LLM API call finishes (typically within a few seconds).

### Issue: Tool calls not visible

**Cause**: Client may not be handling `tool_call` notifications.

**Solution**: Ensure your client listens for `session/update` notifications with `tool_call` and `tool_call_update` types.

### Issue: MCP servers not loading

**Cause**: MCP server command not found or incorrect configuration.

**Solution**: 
- Verify MCP server is installed (e.g., `uvx mcp-server-fetch`)
- Check the command and args in the `mcpServers` configuration
- Ensure the MCP server is accessible from the system PATH

### Issue: Streaming updates not received

**Cause**: Connection issue or client not reading notifications.

**Solution**:
- Ensure stdout is not buffered
- Check that client is reading line-by-line from stdout
- Verify JSON-RPC messages are properly formatted

## Development

### Project Structure

```
crow/
├── src/crow/
│   ├── agent/
│   │   ├── acp_server.py    # Main ACP server implementation
│   │   └── config.py         # Configuration classes
│   └── ...
├── tests/
│   ├── test_acp_simple.py
│   ├── test_acp_cancellation.py
│   └── test_acp_server.py
├── README.md
└── pyproject.toml
```

### Adding New Features

1. Implement the feature in `src/crow/agent/acp_server.py`
2. Add tests in `tests/`
3. Update this README if user-facing changes are made
4. Run the test suite to ensure compatibility

## License

[Specify your license here]

## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## Support

For issues, questions, or contributions, please [open an issue](link-to-issues) on the repository.

## Acknowledgments

- [Agent Client Protocol (ACP)](https://agentclientprotocol.com/)
- [OpenHands SDK](https://docs.openhands.dev/)
- [Model Context Protocol (MCP)](https://modelcontextprotocol.io/)
