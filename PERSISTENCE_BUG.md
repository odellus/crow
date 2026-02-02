# PERSISTENCE BUG

**Date**: 2026-02-01
**Status**: CRITICAL
**Priority**: HIGH

## Problem Description

The ACP server's session persistence system has a critical bug where:

1. **New sessions are created but not populated** - The most recent sessions by modification time are empty (0 messages)
2. **Actual conversations exist in older sessions** - The largest sessions (by file size) contain real conversations but are older
3. **Sessions are being lost** - Content is not being properly saved to the most recent sessions

## Evidence

### Most Recent Sessions (by timestamp)
```bash
# Current time: 2026-02-01 17:04:07 EST
# Most recent session: c87bc091-457f-44d8-9f7c-92ba82cc814c.json
# Modified: 2026-02-01 16:40:51 (24 minutes ago)
# Size: 173 bytes
# Conversation length: 0 messages (empty)
```

### Largest Sessions (by size)
```bash
# Session 1: fe2ffd4e-c2f1-4111-b522-9c414671e4e0.json
# Size: 690,133 bytes (690KB)
# Conversation length: 36 messages
# Working directory: /home/thomas/src/projects/orchestrator-project
# First messages: "hey" → "" → "oustanding. what tools do you have?"

# Session 2: a20c3325-0e6f-45f4-9b34-0e9e6160ad31.json
# Size: 428,456 bytes (428KB)
# Conversation length: 61 messages
# Working directory: /home/thomas/src/projects/orchestrator-project/vscode
# First messages: "hey" → "" → "wow. write a sonnet about ordoliberalism and why it might be better than full blown neoliberalism it's still not quite ideal"
```

## Root Cause Analysis

### Sessions Directory
- Location: `~/.crow/sessions/`
- Format: JSON files named by session ID
- Structure:
  ```json
  {
    "session_id": "uuid",
    "cwd": "/path/to/workspace",
    "mode": "default",
    "conversation_history": [...]
  }
  ```

### Suspected Issues

1. **Session Creation vs Usage**
   - ACP server creates new sessions on each connection
   - But conversations might not be properly saved to the most recent session
   - Old sessions accumulate while new ones remain empty

2. **Filesystem Overwrites**
   - Possible that sessions are being overwritten instead of appended
   - Or new sessions are created but old ones are being cleaned up

3. **Streaming State Issues**
   - The streaming might be working (as evidenced by the large session with the ordoliberalism sonnet)
   - But the most recent sessions aren't capturing the output

## Impact

- **CRITICAL**: Agents cannot rely on session persistence
- **CRITICAL**: Agent-to-agent communication via filesystem breadcrumbs is unreliable
- **CRITICAL**: Conversation history is being lost
- **HIGH**: User cannot review agent work from recent sessions

## Next Steps

1. **Investigate ACP server session management**
   - Check how sessions are created and saved
   - Verify conversation history is being written to disk
   - Check for race conditions or file locking issues

2. **Add logging**
   - Log when sessions are created
   - Log when conversations are saved
   - Log when sessions are loaded

3. **Test session persistence**
   - Create a new session and verify it's populated
   - Check if multiple agents can share the same session file
   - Test with concurrent connections

4. **Consider alternative persistence**
   - If filesystem is unreliable, consider:
     - SQLite database
     - Redis for session storage
     - In-memory state with periodic snapshots

## Related Issues

- Streaming appears to be working (evidenced by large session with sonnet)
- Tool calls are being recorded (evidenced by tool_call in conversation history)
- But the most recent sessions are empty

## Files to Investigate

- `src/crow/acp/server.py` - ACP server implementation
- Session creation and saving logic
- Conversation history management
