# Dual-Agent Implementation Plan

## Current State (What's Already Done)

### 1. `DualAgentOrchestrator` (dual.rs) - COMPLETE
- Orchestrates executor ↔ discriminator turns
- `run()` method spawns async loop
- `run_loop()` does the actual orchestration:
  1. Send input to executor via `thread.send()`
  2. Forward executor events to shared AcpThread
  3. Export executor output via `export_last_turn()`
  4. Send to discriminator as USER message (role flip)
  5. Watch for `task_complete` tool call
  6. If complete → return, else loop with feedback

### 2. `TaskCompleteTool` (tools/task_complete_tool.rs) - COMPLETE
- Takes `summary: String` parameter
- Registered in `tools!` macro
- Discriminator calls this when satisfied

### 3. `export_last_turn()` on Thread (thread.rs) - COMPLETE
- Exports agent messages since last user message to markdown
- Used for handoff between agents

### 4. `DualAgentState` struct (agent.rs) - COMPLETE
```rust
pub struct DualAgentState {
    pub executor_session_id: acp::SessionId,
    pub discriminator_session_id: acp::SessionId,
    pub enabled: bool,
}
```

### 5. `dual_sessions` HashMap in NativeAgent - COMPLETE
- Maps executor session ID → DualAgentState

### 6. `enable_dual_agent_mode()` method - COMPLETE
- Creates discriminator Thread (shares AcpThread)
- Adds TaskCompleteTool to discriminator
- Registers in `dual_sessions`

---

## What's Left To Do

### Step 1: Modify `prompt()` in NativeAgentConnection

**File:** `agent.rs` line ~1150

**Current behavior:**
```rust
fn prompt(&self, id, params, cx) -> Task<Result<acp::PromptResponse>> {
    self.run_turn(session_id, cx, |thread, cx| {
        thread.update(cx, |thread, cx| thread.send(id, content, cx))
    })
}
```

**New behavior:**
```rust
fn prompt(&self, id, params, cx) -> Task<Result<acp::PromptResponse>> {
    // Check if dual-agent mode is enabled for this session
    let dual_state = self.0.read(cx).dual_agent_state(&session_id).cloned();
    
    if let Some(state) = dual_state && state.enabled {
        // Route through orchestrator
        let acp_thread = self.0.read(cx).sessions.get(&session_id)
            .map(|s| s.acp_thread.clone());
        
        let orchestrator = DualAgentOrchestrator::new(
            state.executor_session_id,
            state.discriminator_session_id,
            self.clone(),
            acp_thread,
        );
        
        return orchestrator.run(content, cx);
    }
    
    // Normal single-agent path
    self.run_turn(session_id, cx, |thread, cx| {
        thread.update(cx, |thread, cx| thread.send(id, content, cx))
    })
}
```

**Key considerations:**
- Need to convert `params.prompt` to `Vec<UserMessageContent>` BEFORE the branch
- Need to handle the case where acp_thread is None
- The orchestrator takes ownership of the content

---

### Step 2: Fix task_complete Detection in Orchestrator - VERIFIED CORRECT

**File:** `dual.rs` line ~188

**Current code:**
```rust
if let ThreadEvent::ToolCall(ref tool_call) = event {
    let is_task_complete = tool_call
        .meta
        .as_ref()
        .and_then(|m| m.get("tool_name"))
        .and_then(|v| v.as_str())
        .map(|name| name == TASK_COMPLETE_TOOL)
        .unwrap_or(false);
}
```

**VERIFIED:** This is correct! Looking at `thread.rs:2404-2416`, the `ThreadEventStream::initial_tool_call()` method sets:
```rust
acp::ToolCall {
    meta: Some(serde_json::json!({
        "tool_name": tool_name   // <-- Tool name IS in meta
    })),
    ...
}
```

So detection logic is already correct.

---

### Step 3: ThreadEvent Structure - VERIFIED

**File:** `thread.rs`

```rust
pub enum ThreadEvent {
    UserMessage(UserMessage),
    AgentText(String),
    AgentThinking(String),
    ToolCall(acp::ToolCall),      // Contains meta.tool_name
    ToolCallUpdate(acp_thread::ToolCallUpdate),
    ToolCallAuthorization(ToolCallAuthorization),
    Retry(acp_thread::RetryStatus),
    Stop(acp::StopReason),
}
```

The `acp::ToolCall` struct has a `meta: Option<serde_json::Value>` field that contains `{"tool_name": "..."}` when set by `initial_tool_call()`.

---

### Step 4: Handle User Message in Orchestrator

**Current code in dual.rs:**
```rust
ThreadEvent::UserMessage(message) => {
    acp_thread.update(cx, |thread, cx| {
        for content in message.content {
            thread.push_user_content_block(
                Some(message.id.clone()),
                content.into(),
                cx,
            );
        }
    })?;
}
```

**Problem:** The discriminator's internal "user messages" (which are the executor's output) should NOT be shown as user messages in the UI. We need to:
- NOT forward discriminator's UserMessage events to AcpThread
- Only forward the discriminator's AssistantMessage and ToolCall events

**Solution:** Add an `AgentRole` enum and track which agent's events we're forwarding:
```rust
enum AgentRole {
    Executor,
    Discriminator,
}

// In forward_events methods, skip UserMessage for discriminator
```

---

### Step 5: Add Agent Labels to Output (Optional for V1)

To distinguish executor vs discriminator output in the UI:

**Option A (Simple):** Add to system prompts
- Executor prompt: "Always prefix your responses with [Executor]:"
- Discriminator prompt: "Always prefix your responses with [Reviewer]:"

**Option B (Better):** Use annotations
```rust
ThreadEvent::AgentText(text) => {
    acp_thread.update(cx, |thread, cx| {
        thread.push_assistant_content_block(
            acp::ContentBlock::Text(acp::TextContent {
                text,
                annotations: Some(vec![("agent_role".into(), role.to_string().into())]),
                meta: None,
            }),
            false,
            cx,
        )
    })?;
}
```

---

### Step 6: Initialize Discriminator with Context

**Current code:**
```rust
let discriminator_thread = cx.new(|cx| {
    let mut thread = Thread::new(/* ... */);
    thread.add_tool(crate::tools::TaskCompleteTool);
    thread
});
```

**What's missing:**
1. **Discriminator system prompt** - needs a different persona
2. **Initial context** - discriminator should know:
   - What the original user request was
   - That it's reviewing another agent's work
   - That it should call task_complete when satisfied

**Options:**

**Option A:** Use a different profile for discriminator
```rust
thread.set_profile(AgentProfileId("discriminator".into()), cx);
```
This requires creating a "discriminator" profile in settings.

**Option B:** Set custom system prompt directly
```rust
// If we added system_prompt field to Thread
thread.set_system_prompt(DISCRIMINATOR_SYSTEM_PROMPT, cx);
```

**Option C:** Send an initial setup message
- When enabling dual-agent, immediately send a "priming" message to discriminator
- This establishes context before the loop starts

---

### Step 7: Test Harness (For Initial Testing)

For testing without UI toggle:

**Option A:** Add a test in `agent/src/tests/mod.rs`
```rust
#[gpui::test]
async fn test_dual_agent_loop(cx: &mut TestAppContext) {
    // 1. Create NativeAgent
    // 2. Create executor session via new_thread()
    // 3. Enable dual-agent mode
    // 4. Send a prompt
    // 5. Verify orchestrator runs
    // 6. Verify task_complete breaks the loop
}
```

**Option B:** Add a debug flag
```rust
// In enable_dual_agent_mode or somewhere accessible
const FORCE_DUAL_AGENT: bool = true;

// In prompt()
if FORCE_DUAL_AGENT || dual_state.enabled {
    // Use orchestrator
}
```

---

## Implementation Order

1. **Step 1: Modify prompt()** - Route through orchestrator when dual mode enabled
2. **Step 3: Verify ThreadEvent** - Make sure task_complete detection works
3. **Step 2: Fix detection** - Update based on Step 3 findings
4. **Step 4: Handle UserMessage** - Don't show internal messages
5. **Step 7: Test** - Write test or add debug flag
6. **Step 6: Context** - Add discriminator system prompt
7. **Step 5: Labels** - Optional, for better UX

---

## Data Flow Summary

```
User sends message
        │
        ▼
┌─────────────────────────────────────────────────────────────┐
│  NativeAgentConnection.prompt()                              │
│    │                                                         │
│    ├─ if dual_mode:                                          │
│    │      └─► DualAgentOrchestrator.run()                    │
│    │              │                                          │
│    │              ▼                                          │
│    │     ┌─────────────────────────────────────────────┐    │
│    │     │  LOOP:                                       │    │
│    │     │   1. executor.send(input)                    │    │
│    │     │   2. forward executor events → AcpThread     │    │
│    │     │   3. executor.export_last_turn()             │    │
│    │     │   4. discriminator.send(executor_output)     │    │
│    │     │   5. forward discrim events → AcpThread      │    │
│    │     │   6. if task_complete → break                │    │
│    │     │      else → input = discrim.export_last_turn │    │
│    │     │      goto 1                                  │    │
│    │     └─────────────────────────────────────────────┘    │
│    │                                                         │
│    └─ else:                                                  │
│           └─► run_turn() (normal single-agent)               │
└─────────────────────────────────────────────────────────────┘
        │
        ▼
   PromptResponse returned
```

---

## Files to Modify

| File | Changes |
|------|---------|
| `agent/src/agent.rs` | Modify `prompt()` to check dual mode |
| `agent/src/dual.rs` | Fix task_complete detection, handle UserMessage |
| `agent/src/thread.rs` | Verify ThreadEvent structure |
| `agent/src/tests/mod.rs` | Add test for dual-agent |

---

## Potential Issues

1. **Tool authorization** - Currently skipped in dual mode. May need to handle.
2. **Cancellation** - Need to cancel both threads when user cancels.
3. **Max iterations** - Should add a max loop count to prevent infinite loops.
4. **Token counting** - Both agents contribute tokens, need combined display.
5. **Error handling** - If executor errors, should we still run discriminator?

---

## Next Immediate Action

1. Read `ThreadEvent` enum definition to understand tool call structure
2. Modify `prompt()` method
3. Test with a simple case
