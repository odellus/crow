# ACP Server Implementation Status

This document tracks the implementation status of the Agent Client Protocol (ACP) server in the Crow project.

## ✅ What We Have Implemented

| Feature | Status | Notes |
|---------|--------|-------|
| **Initialization** | ✅ Complete | `initialize()` method with protocol version & capabilities negotiation |
| **Session Management** | ✅ Complete | `session/new` creates OpenHands conversations with unique session IDs |
| **Prompt Sending** | ✅ Complete | `session/prompt` accepts user messages (text content blocks) |
| **Streaming Updates** | ✅ Complete | `session/update` with `agent_message_chunk` and `agent_thought_chunk` for token-level streaming |
| **Tool Call Reporting** | ✅ Complete | `tool_call` and `tool_call_update` notifications with proper status tracking |
| **MCP Tool Support** | ✅ Complete | Loads MCP servers via stdio transport (fetch, terminal, file_editor) |
| **Async/Sync Bridge** | ✅ Complete | Uses `loop.run_in_executor()` to run blocking OpenHands code in thread pool |
| **Clean Output** | ✅ Complete | `visualizer=None` disables OpenHands UI pollution on stdout |
| **Cancellation** | ✅ Complete | `cancel()` method maps to OpenHands `pause()`, returns `stopReason: "cancelled"` |
| **Permission Requests** | ✅ Complete | `_request_tool_permission()` calls `session/request_permission` before tool execution |
| **Agent Plans** | ✅ Complete | Plan updates sent via `session/update` with status tracking |

## ❌ What We're Missing

### Core ACP Features (High Priority)

All high-priority features are implemented! ✅

### Core ACP Features (Medium Priority)

All medium-priority features are implemented! ✅

### Core ACP Features (Low Priority)

| Feature | Priority | Description |
|---------|----------|-------------|
| **Session Modes** | 🟢 LOW | Support `session/set_mode` and return `available_modes` in `session/new` |
| **Slash Commands** | 🟢 LOW | Advertise commands via `available_commands_update` notification |
| **Session Load** | 🟢 LOW | Support `session/load` for conversation persistence |

### Client Methods (Agent calls these, Client implements)

These are methods defined in the ACP spec that the **agent** may call on the **client**. Our server (as the agent) might need to call these:

| Method | Priority | Description |
|--------|----------|-------------|
| `session/request_permission` | 🔴 HIGH | Ask user for tool execution approval |
| `fs/read_text_file` | 🟡 MEDIUM | Read files from client (including unsaved edits) |
| `fs/write_text_file` | 🟡 MEDIUM | Write files to client filesystem |
| `terminal/create` | 🟢 LOW | Create terminal for command execution |
| `terminal/output` | 🟢 LOW | Get terminal output |
| `terminal/wait_for_exit` | 🟢 LOW | Wait for command completion |
| `terminal/kill` | 🟢 LOW | Kill running command |
| `terminal/release` | 🟢 LOW | Release terminal resources |

**Note:** These are client-side capabilities. OpenHands might try to use them, so we need to handle them gracefully even if the client doesn't support them.

## Technical Implementation Details

### Cancellation Implementation

**Current State:** ✅ Implemented

**Implementation:**
- `session/cancel` → OpenHands `pause()`
- Returns `stopReason: "cancelled"` from `prompt()`
- Cancellation flag tracked per session
- Graceful shutdown of streaming updates
- Limitation: `pause()` waits for current LLM call to complete; not a hard stop

**Future Solution:**
- Add hard cancellation to OpenHands SDK
- Interrupt in-progress LLM calls
- New `ConversationExecutionStatus.CANCELLED` status
- Proper `cancel()` method (distinct from `pause()`)

### Tool Call Integration

**Current State:** ✅ Implemented

**Implementation:**
- Detects tool calls from LLM streaming tokens
- Sends `tool_call` notification when tool starts (includes `toolCallId`, `title`, `kind`, `status`)
- Sends `tool_call_update` with status progression (`in_progress` → `completed`)
- Maps OpenHands tools to ACP tool kinds:
  - `execute` - terminal commands
  - `edit` - file modifications
  - `read` - file reading operations
  - `search` - search operations
  - `delete` - file deletion
  - `move` - move/rename operations
  - `other` - default
- Handles tool completion and errors properly

### Permission System

**Current State:** ✅ Implemented

**Implementation:**
- `_request_tool_permission()` method calls `session/request_permission` before tool execution
- Permission options presented: `allow_once`, `allow_always`, `reject_once`, `reject_always`
- Tool status set to `failed` if permission rejected
- Plan entries only added for permitted tools
- Integrated into tool_start event handler in streaming callback
- Fail-open design: if permission request fails, tool execution proceeds (configurable)

### Content Types

**Current State:** Only text content blocks

**Needed:**
- Image support (`agent_message_chunk` with images)
- Resource links (file references)
- Diff content (for file modifications)

### Agent Plan System

**Current State:** ✅ Implemented

**Implementation:**
- Initial plan entry created when prompt starts
- Plan entries added for each tool execution
- Plan status updates: `in_progress` → `completed`
- Plan updates sent via `session/update` with `update_plan()` helper
- Plan entries tracked per session
- Final plan update marks all entries as completed on turn end

## Next Steps - Prioritized

### Phase 1: Permission Requests (HIGH PRIORITY)
- [x] Implement permission interception
- [x] Call `session/request_permission` before tool execution
- [x] Handle permission responses
- [x] Integrate with OpenHands confirmation policies

### Phase 2: Agent Plans (MEDIUM PRIORITY)
- [x] Detect when LLM creates execution plans
- [x] Send `plan` updates via `session/update`
- [x] Update plan entry statuses as work progresses

### Phase 3: Advanced Features (LOW PRIORITY)
- [ ] Implement session modes
- [ ] Add slash command advertising
- [ ] Support session persistence with `session/load`

## ACP Protocol References

- **Overview**: https://agentclientprotocol.com/protocol/overview.md
- **Initialization**: https://agentclientprotocol.com/protocol/initialization.md
- **Session Setup**: https://agentclientprotocol.com/protocol/session-setup.md
- **Prompt Turn**: https://agentclientprotocol.com/protocol/prompt-turn.md
- **Tool Calls**: https://agentclientprotocol.com/protocol/tool-calls.md
- **Cancellation**: https://agentclientprotocol.com/protocol/draft/cancellation.md
- **File System**: https://agentclientprotocol.com/protocol/file-system.md
- **Terminals**: https://agentclientprotocol.com/protocol/terminals.md
- **Session Modes**: https://agentclientprotocol.com/protocol/session-modes.md
- **Agent Plans**: https://agentclientprotocol.com/protocol/agent-plan.md
- **Slash Commands**: https://agentclientprotocol.com/protocol/slash-commands.md
- **Extensibility**: https://agentclientprotocol.com/protocol/extensibility.md

## OpenHands SDK References

- **Pause and Resume**: https://docs.openhands.dev/sdk/guides/convo-pause-and-resume.md
- **SDK Guides**: https://docs.openhands.dev/sdk/guides/
- **API Reference**: https://docs.openhands.dev/sdk/api-reference/
