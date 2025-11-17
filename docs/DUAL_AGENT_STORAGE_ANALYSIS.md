# Dual-Agent Storage Analysis

## Current Architecture (As Built)

### What We Have

**Three layers of storage:**

1. **SharedConversation** (in-memory only, runtime.rs)
   - Contains raw dual-agent conversation
   - Has agent attribution (User, Executor, Discriminator)
   - Lives only during `/session/dual` request
   - **NOT persisted anywhere**

2. **SessionStore - Executor Session** (ses-XXX-executor)
   - Contains transformed messages from executor's POV
   - All messages stored as "assistant" role
   - Persisted to `~/.crow/sessions/ses-XXX/`
   - Has executor's tool calls and results

3. **SessionStore - Discriminator Session** (ses-XXX-discriminator)
   - Contains transformed messages from discriminator's POV  
   - All messages stored as "assistant" role
   - Persisted to `~/.crow/sessions/ses-XXX/`
   - Has discriminator's tool calls and work_completed

### The Problem

**SharedConversation is LOST after the dual-agent run completes!**

```rust
// In server.rs POST /session/dual
let mut shared_conversation = SharedConversation::new(...);
let result = runtime.run(&mut shared_conversation, &working_dir).await?;

// shared_conversation is dropped here!
// We only return DualAgentResult with IDs
```

**What gets saved:**
- ✅ Executor session with messages
- ✅ Discriminator session with messages  
- ❌ SharedConversation ground truth
- ❌ Agent attribution (who said what)
- ❌ The actual conversation flow

**What we can reconstruct:**
- Each individual session's messages
- Tool calls from each perspective
- But NOT the actual back-and-forth dialog

### Current Message Storage

Each session stores `MessageWithParts`:

```rust
pub struct MessageWithParts {
    pub info: Message,  // role is always "assistant"
    pub parts: Vec<Part>,
}
```

**Executor session** (`ses-c0e06952-16dc-4e9d-9fcd-e81533d4e193`):
- Message 1: assistant (executor's first response)
- Message 2: assistant (executor after discriminator feedback #1)
- Message 3: assistant (executor after discriminator feedback #2)  
- Message 4: assistant (executor after discriminator feedback #3)

**Discriminator session** (`ses-0265ea83-7d47-47b6-9f07-b46d2ce1b4e6`):
- Message 1: assistant (discriminator review #1)
- Message 2: assistant (discriminator review #2)
- Message 3: assistant (discriminator review #3)
- Message 4: assistant (discriminator review #4 with work_completed)

**Problem:** Looking at either session alone, you can't tell:
- Which agent said what
- The conversation sequence
- Who the user was vs executor vs discriminator

## What You Want (Langfuse-Style Telemetry)

A markdown file showing the COMPLETE conversation:

```markdown
# Conversation: conv-518d6151-b05e-4ed4-952d-5b66c0caab80

**Task:** Create a simple hello.txt file with the message Hello from dual-agent
**Started:** 2025-11-16 17:03:45
**Completed:** true (4 steps)
**Verdict:** [discriminator's summary]

---

## System Context

**Executor Session:** ses-c0e06952-16dc-4e9d-9fcd-e81533d4e193
**Discriminator Session:** ses-0265ea83-7d47-47b6-9f07-b46d2ce1b4e6
**Working Directory:** /home/thomas/src/projects/opencode-project/test-dummy

---

## Step 1: Executor Turn

**Agent:** build
**System Prompt:**
```
You are a build agent...
[full 5-layer prompt]
```

**Available Tools:**
- bash
- edit
- write
- read
- grep
- glob
- todowrite
- todoread
(NOT work_completed)

**Response:**

I'll create the hello.txt file...

🔧 bash: echo "Hello from dual-agent" > hello.txt

**Tool Result:**
[success]

---

## Step 2: Discriminator Turn

**Agent:** discriminator
**System Prompt:**
```
You are the DISCRIMINATOR...
[full discriminator prompt]
```

**Available Tools:**
- bash
- edit
- write
- read  
- grep
- glob
- todowrite
- todoread
- work_completed ✅

**Response:**

Let me verify the work...

🔧 bash: cat hello.txt

**Tool Result:**
Hello from dual-agent

The file looks incorrect. It should say "Hello from dual-agent" but...

---

[etc for all 4 steps]

---

## Final Summary

**Completed:** true
**Total Steps:** 4
**Files Modified:**
- hello.rs (created)
- test_dual.sh (created)

**Discriminator Verdict:**
[full comprehensive summary]
```

## Storage Design: Two Options

### Option A: Save SharedConversation (Simple)

**Where:** `{server_dir}/.crow/conversations/conv-XXX/`

```
.crow/
└── conversations/
    └── conv-518d6151.../
        ├── conversation.json     # SharedConversation serialized
        ├── metadata.json         # task, times, result
        └── conversation.md       # rendered markdown
```

**Pros:**
- Simple - just serialize SharedConversation
- Ground truth is preserved
- Easy to reconstruct

**Cons:**
- Duplicate data (also in sessions)
- Need to link conv ↔ sessions

### Option B: Enhance Sessions with Dual Metadata (Complex)

**Where:** `{server_dir}/.crow/sessions/ses-XXX/`

```
.crow/
└── sessions/
    ├── ses-executor-XXX/
    │   ├── messages/
    │   ├── metadata.json
    │   └── dual.json  ← { conversation_id, peer_session_id, role: "executor" }
    └── ses-discriminator-XXX/
        ├── messages/
        ├── metadata.json
        └── dual.json  ← { conversation_id, peer_session_id, role: "discriminator" }
```

Then reconstruct SharedConversation by:
1. Load executor session
2. Load discriminator session  
3. Interleave messages by timestamp
4. Rebuild conversation flow

**Pros:**
- No duplicate storage
- Sessions are self-contained

**Cons:**
- Complex reconstruction
- Lossy (can't perfectly rebuild SharedConversation)
- More error-prone

## Recommendation: Option A + Enhancement

1. **Save SharedConversation** to `.crow/conversations/`
2. **Link sessions** to conversation via metadata
3. **Generate markdown** on save

### File Structure

```
{server_cwd}/.crow/
├── conversations/
│   └── conv-{uuid}/
│       ├── conversation.json      # SharedConversation
│       ├── result.json            # DualAgentResult
│       ├── conversation.md        # Human-readable
│       └── telemetry.md           # Full Langfuse-style trace
├── sessions/
│   ├── ses-{executor-uuid}/
│   │   ├── messages/
│   │   ├── metadata.json
│   │   └── link.json → { conversation_id: "conv-..." }
│   └── ses-{discriminator-uuid}/
│       ├── messages/
│       ├── metadata.json
│       └── link.json → { conversation_id: "conv-..." }
└── config.json
```

## Implementation Plan

### Phase 1: Persist SharedConversation
- [ ] Create `ConversationStore` struct
- [ ] Save to `.crow/conversations/{id}/conversation.json`
- [ ] Link from executor/discriminator sessions

### Phase 2: Generate Telemetry Markdown
- [ ] Create markdown renderer for SharedConversation
- [ ] Include system prompts, tool lists, full messages
- [ ] Save as `.crow/conversations/{id}/telemetry.md`

### Phase 3: Storage Location (.crow directory)
- [ ] Detect server start directory
- [ ] Create `.crow/` there (not in ~/.crow)
- [ ] Add to .gitignore automatically

## Questions to Answer

1. **Server storage location:**
   - You're right: crow is a SERVER sitting on top of a directory
   - Should be `{server_cwd}/.crow/` not `~/.crow/`
   - Like JupyterLab model

2. **What goes in telemetry.md:**
   - Full system prompts for each turn
   - Tool availability for each agent
   - All messages with agent attribution
   - All tool calls with arguments and results
   - Timing information
   - Token usage and costs

3. **Do we need the sessions at all if we have SharedConversation?**
   - YES for single-agent mode (still need SessionStore)
   - MAYBE for dual-agent (could just use conversation)
   - But keeping both allows:
     - Individual agent inspection
     - Resume from either session
     - Fork from specific point

Let me know which approach you prefer and I'll implement it!
