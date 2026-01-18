# ACP Protocol Deep Dive

## Overview

The Agent Client Protocol (ACP) is a JSON-RPC 2.0-based protocol for communication between AI agents and clients. All communication happens over stdin/stdout, making it language-agnostic and easy to integrate.

## Message Flow

### 1. Initialize

Client sends:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocol_version": 1,
    "client_capabilities": {},
    "client_info": {
      "name": "crow-ide",
      "version": "0.1.0"
    }
  }
}
```

Server responds:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocol_version": 1,
    "agent_capabilities": {
      "prompt_capabilities": {
        "text": true,
        "image": true,
        "resource": true
      },
      "mcp_capabilities": {
        "http": false,
        "sse": false
      },
      "load_session": true
    },
    "agent_info": {
      "name": "crow-acp-server",
      "title": "Crow ACP Server",
      "version": "0.1.0"
    }
  }
}
```

### 2. Create Session

Client sends:
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

Server responds:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "sessionId": "uuid-string"
  }
}
```

### 3. Send Prompt (with Streaming)

Client sends:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "session/prompt",
  "params": {
    "session_id": "uuid-string",
    "prompt": [
      {
        "type": "text",
        "text": "Hello, how are you?"
      }
    ]
  }
}
```

Server sends streaming updates (notifications, no `id` field):
```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "sessionId": "uuid-string",
    "update": {
      "@type": "agent_thought_text",
      "text": "Let me think about this..."
    }
  }
}
```

```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "sessionId": "uuid-string",
    "update": {
      "@type": "agent_message_text",
      "text": "I'm doing well, thank you!"
    }
  }
}
```

Finally, server sends response (with `id` field):
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "stop_reason": "end_turn"
  }
}
```

## Update Types

ACP defines various update types for streaming:

### Thinking Updates
```json
{
  "@type": "agent_thought_text",
  "text": "The user is asking about..."
}
```

### Content Updates
```json
{
  "@type": "agent_message_text",
  "text": "Here's the answer to your question..."
}
```

### Tool Call Updates
```json
{
  "@type": "tool_call_start",
  "tool_call_id": "uuid",
  "title": "terminal",
  "kind": "execute",
  "status": "in_progress"
}
```

```json
{
  "@type": "tool_call_update",
  "tool_call_id": "uuid",
  "status": "completed"
}
```

### Plan Updates
```json
{
  "@type": "plan_update",
  "entries": [
    {
      "content": "Analyze the request",
      "priority": "high",
      "status": "completed"
    },
    {
      "content": "Execute tool: terminal",
      "priority": "medium",
      "status": "in_progress"
    }
  ]
}
```

## Session Management

### Load Session
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "session/load",
  "params": {
    "session_id": "existing-uuid",
    "cwd": "/path/to/workspace"
  }
}
```

Server replays conversation history via `session/update` notifications before responding.

### Cancel
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "cancel",
  "params": {
    "session_id": "uuid"
  }
}
```

## Key Protocol Features

### 1. Notifications vs Responses
- **Notifications** (no `id` field): Streaming updates during execution
- **Responses** (with `id` field): Final acknowledgment of a request

### 2. Session Replay
When loading a session, the server MUST replay the entire conversation history via `session/update` notifications. This ensures clients can reconstruct the full conversation state.

### 3. Stop Reasons
- `end_turn`: Agent finished naturally
- `cancelled`: User cancelled the operation
- `error`: Agent encountered an error

### 4. Tool Permissions
ACP supports permission requests before tool execution:
```json
{
  "@type": "permission_request",
  "tool_call_id": "uuid",
  "title": "terminal",
  "arguments": "ls -la",
  "permission_options": [
    {
      "id": "approve",
      "title": "Approve",
      "is_primary": true
    },
    {
      "id": "reject",
      "title": "Reject"
    }
  ]
}
```

Client responds with permission decision via `permission_response`.

## Implementation Notes

### Stdio Communication
- All messages are newline-delimited JSON
- Server reads from stdin, writes to stdout
- Logs should go to stderr to avoid interfering with protocol

### Concurrency
- Multiple sessions can be active simultaneously
- Each session has its own Conversation object
- Sessions are identified by UUID

### Error Handling
Errors follow JSON-RPC 2.0 spec:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "errors": [...]
    }
  }
}
```
