# Crow Visibility & Storage Guide

How data is stored and how to access it. Critical for debugging and building CLI tools.

## Storage Structure

```
~/.local/share/crow/storage/
├── session/
│   └── {projectID}/
│       └── {sessionID}.json          # Session metadata only
├── message/
│   └── {sessionID}/
│       └── {messageID}.json          # Message metadata (no content!)
├── part/
│   └── {messageID}/
│       ├── part-text-{uuid}.json     # Text content
│       ├── part-tool-{uuid}.json     # Tool call + result
│       ├── part-thinking-{uuid}.json # LLM reasoning
│       └── part-patch-{uuid}.json    # File changes
└── todo/
    └── {sessionID}.json              # Todo list state
```

## Key Insight: Content is in PARTS, not Messages

Messages are just metadata. The actual content (text, tool calls, etc.) is in the `part/` directory.

```
# WRONG - this only shows metadata
cat ~/.local/share/crow/storage/message/{sessionID}/{messageID}.json

# RIGHT - this shows actual content
cat ~/.local/share/crow/storage/part/{messageID}/part-*.json
```

## Session JSON Structure

```json
{
  "id": "ses_xxx",
  "projectID": "abc123",
  "directory": "/path/to/working/dir",
  "parentID": "ses_parent",           // null for top-level sessions
  "title": "Session Title",
  "version": "1.0.0",
  "time": {
    "created": 1234567890,
    "updated": 1234567890
  },
  "metadata": {
    // For verified tasks:
    "dual_agent": {
      "role": "executor" | "arbiter",
      "pair_id": "pair-uuid",
      "sibling_id": "ses_sibling"     // Links executor ↔ arbiter
    },
    "dualPairComplete": true,
    "dualPairFinalStep": 1,
    "completionSummary": "...",
    "completionVerification": "..."
  }
}
```

## Message JSON Structure

```json
{
  "role": "user" | "assistant",
  "id": "msg-xxx",
  "session_id": "ses_xxx",
  "time": { "created": ..., "completed": ... },
  
  // Assistant-only fields:
  "model_id": "kimi-k2-thinking",
  "provider_id": "moonshotai",
  "mode": "build" | "arbiter" | "general",
  "cost": 0.003,
  "tokens": {
    "input": 18866,
    "output": 375,
    "reasoning": 0,
    "cache": { "read": 0, "write": 0 }
  }
}
```

## Part JSON Structures

### Text Part
```json
{
  "type": "text",
  "id": "part-text-xxx",
  "session_id": "ses_xxx",
  "message_id": "msg-xxx",
  "text": "The actual response text..."
}
```

### Tool Part
```json
{
  "type": "tool",
  "id": "part-tool-xxx",
  "session_id": "ses_xxx",
  "message_id": "msg-xxx",
  "call_id": "call_xxx",
  "tool": "write",
  "state": {
    "type": "completed",
    "input": { "file_path": "/tmp/hello.txt", "content": "Hello" },
    "output": "File written successfully",
    "title": "write",
    "time": { "start": 123, "end": 456 }
  }
}
```

### Thinking Part
```json
{
  "type": "thinking",
  "id": "part-thinking-xxx",
  "session_id": "ses_xxx", 
  "message_id": "msg-xxx",
  "text": "LLM's reasoning/chain of thought..."
}
```

## CLI Commands Needed (TODO)

### View Session Tree
```bash
# Show parent → child relationships
crow-cli session tree <session-id>

# Example output:
# ses_ABC123 (Verified Test)
# ├── ses_DEF456 (Verified: Executor)
# └── ses_GHI789 (Verified: Arbiter)
```

### View Full Message with Parts
```bash
# Show message with all parts inline
crow-cli message <message-id>

# Example output:
# === Message: msg-xxx (assistant) ===
# Model: kimi-k2-thinking | Cost: $0.0037 | Tokens: 18866→375
# 
# [THINKING]
# I need to create the file...
#
# [TOOL: write]
# Input: {"file_path": "/tmp/hello.txt", "content": "Hello"}
# Output: File written successfully (3ms)
#
# [TOOL: bash]
# Input: {"command": "cat /tmp/hello.txt"}
# Output: Hello (2ms)
#
# [TEXT]
# I've created the file hello.txt with the content "Hello".
```

### View Verified Task Details
```bash
# Show executor + arbiter sessions with their tool calls
crow-cli verified <parent-session-id>

# Example output:
# === Verified Task: ses_ABC123 ===
# Status: ✅ Completed in 1 step
# Summary: Created hello.txt with content 'Hello World'
# Verification: File exists with correct content
#
# --- Executor (ses_DEF456) ---
# Step 1:
#   [write] /tmp/hello.txt → success
#   [bash] ls -la → success
#   [read] /tmp/hello.txt → success
#
# --- Arbiter (ses_GHI789) ---  
# Step 1:
#   [bash] cat /tmp/hello.txt → success
#   [read] /tmp/hello.txt → success
#   [task_complete] ✅
```

### List Parts for a Message
```bash
crow-cli parts <message-id>

# Example output:
# Parts for msg-xxx:
#   part-tool-aaa (write) - completed
#   part-tool-bbb (bash) - completed
#   part-tool-ccc (read) - completed
#   part-text-ddd - 45 chars
```

## Quick Debug Commands

```bash
# Find all sessions for a directory
ls ~/.local/share/crow/storage/session/*/  | xargs grep -l "/tmp/mydir"

# List messages in a session
ls ~/.local/share/crow/storage/message/{sessionID}/

# List parts for a message
ls ~/.local/share/crow/storage/part/{messageID}/

# See what tools were called
for f in ~/.local/share/crow/storage/part/{messageID}/part-tool-*.json; do
  cat "$f" | jq -r '.tool'
done

# See tool inputs/outputs
cat ~/.local/share/crow/storage/part/{messageID}/part-tool-*.json | jq '{tool: .tool, input: .state.input, output: .state.output}'

# Find verified task pairs
grep -r "dual_agent" ~/.local/share/crow/storage/session/

# Find sessions by title
grep -r "Verified:" ~/.local/share/crow/storage/session/
```

## Verified Task Flow Visibility

When a verified task runs:

1. **Parent session** calls Task tool with `subagent_type: "verified"`
2. **Creates two child sessions:**
   - `Verified: Executor` (uses "build" agent)
   - `Verified: Arbiter` (uses "arbiter" agent)
3. **Links them via metadata:**
   - Both have `metadata.dual_agent.pair_id` (same value)
   - Both have `metadata.dual_agent.sibling_id` (points to each other)
4. **Loop runs:**
   - Executor turn → messages/parts stored in executor session
   - Arbiter turn → messages/parts stored in arbiter session
5. **On completion:**
   - Both sessions get `dualPairComplete: true`
   - `completionSummary` and `completionVerification` stored

## Priority CLI Features

1. **`crow-cli message <id>`** - View message with inline parts (CRITICAL)
2. **`crow-cli session tree <id>`** - Show parent/child relationships
3. **`crow-cli verified <id>`** - Show executor/arbiter breakdown
4. **`crow-cli parts <message-id>`** - List parts for a message

## Subagent Output in Parent CLI

**PROBLEM:** When a subagent (verified/general) completes, the parent CLI only shows the final output string. We don't see:
- What tools the subagent called
- How many LLM calls it made
- Token usage breakdown
- Errors/retries

**SOLUTION:** The Task tool result should include structured metadata that the CLI can render:

```json
{
  "output": "✅ Task completed...",
  "metadata": {
    "subagent": "verified",
    "executor_session_id": "ses_xxx",
    "arbiter_session_id": "ses_yyy",
    "steps": 1,
    "total_cost": 0.0115,
    "total_tokens": { "input": 54000, "output": 1200 },
    "tool_calls": [
      { "agent": "executor", "tool": "write", "duration_ms": 50 },
      { "agent": "executor", "tool": "bash", "duration_ms": 30 },
      { "agent": "arbiter", "tool": "read", "duration_ms": 20 },
      { "agent": "arbiter", "tool": "task_complete", "duration_ms": 5 }
    ]
  }
}
```

Then the CLI's `render_task()` can show:
```
🤖 task [verified] (76488ms)
   → { "description": "...", "prompt": "...", "subagent_type": "verified" }
   ← ✅ Task completed and verified in 1 step(s)
   
   Executor (ses_xxx):
     write → bash → read
   Arbiter (ses_yyy):  
     read → task_complete ✅
   
   Cost: $0.0115 | Tokens: 54k→1.2k
```
