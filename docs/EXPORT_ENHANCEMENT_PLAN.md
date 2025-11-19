# Crow Export Enhancement Plan

## Current Status

### What Works ✅
- Crow exports sessions to `.crow/sessions/{session_id}.md` after every message
- Export includes: title, session ID, timestamps, user messages
- Storage structure matches OpenCode (session/message/part JSON files)
- API compatibility with OpenCode confirmed

### What's Missing ❌
- **Full LLM prompts are not logged**
- System prompts not visible
- Conversation context sent to LLM not shown
- Tool definitions not included in export

## Enhancement: Full Prompt Logging

### Goal
Make Crow log the COMPLETE prompt sent to the LLM, including:
1. System prompt (agent-specific instructions)
2. Tool definitions
3. Full conversation history
4. All context that the LLM actually sees

### Implementation Plan

#### 1. Capture Prompts in Executor
In `packages/api/src/agent/executor.rs`:
- After building LLM context in `build_llm_context()`, save it
- Store in a new `llm_prompt` field or separate structure
- Associate with the assistant message

#### 2. Enhanced Export Format
Example markdown format:

```markdown
# Session Title

**Session ID:** `ses-xxx`
**Created:** 2025-11-17T01:17:44.702Z
**Project:** /path/to/project

---

## 👤 User (user)
*2025-11-17T01:17:44.729Z*

What is 2+2?

---

## 🤖 Assistant (build agent)
*2025-11-17T01:17:45.123Z - 01:17:47.456Z*

<details>
<summary>📋 Full LLM Prompt (Click to expand)</summary>

### System Prompt
```
You are a BUILD agent for OpenCode/Crow.
Your role is to write code and make changes to files.
... (full system prompt)
```

### Tools Available
- Edit
- Read  
- Bash
... (with full definitions)

### Conversation History
1. **User**: What is 2+2?

### Total Tokens
- Input: 1234
- Output: 567

</details>

The answer is 4.

---
```

#### 3. Benefits

**For Debugging:**
- See exactly what the LLM sees
- Understand why certain responses were generated
- Compare prompts across sessions

**For Development:**
- Test prompt changes
- Optimize system prompts
- Understand token usage

**For Transparency:**
- Full visibility into AI behavior
- Reproducible results
- Better than OpenCode!

### Files to Modify

1. `packages/api/src/agent/executor.rs`
   - Save built context somewhere accessible
   
2. `packages/api/src/session/export.rs`
   - Add full prompt section to exports
   - Format nicely with collapsible sections

3. `packages/api/src/types.rs` (optional)
   - Add `llm_context` field to Message or create new type

### Status
🔄 **Ready to implement**

OpenCode doesn't do this - Crow will be BETTER!
