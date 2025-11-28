# Dual Agent Implementation Plan for Zed

## Overview

Wire two Threads/Sessions to the same AcpThread, orchestrating executor ↔ discriminator turns with role inversion.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         DualNativeAgent                                      │
│  implements AgentConnection                                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────────┐         ┌─────────────────────┐                   │
│   │ Session (executor)  │         │ Session (discrim)   │                   │
│   │   Thread            │ ──────► │   Thread            │                   │
│   │   - own messages    │ export  │   - own messages    │                   │
│   │   - executor prompt │ + flip  │   - discrim prompt  │                   │
│   └─────────────────────┘         └─────────────────────┘                   │
│            │                               │                                 │
│            │ ThreadEvents                  │ ThreadEvents                    │
│            └───────────────┬───────────────┘                                │
│                            ▼                                                 │
│                   handle_thread_events()                                     │
│                   (both pipe to same acp_thread)                            │
└─────────────────────────────────────────────────────────────────────────────┘
                            │
                            ▼
              ┌─────────────────────────────┐
              │  AcpThread (shared UI)      │
              │  - interleaved entries      │
              │  - both agents visible      │
              └─────────────────────────────┘
```

## Implementation Steps

### Phase 1: task_complete Tool

**File:** `zed/crates/acp_tools/src/task_complete.rs` (new)

```rust
pub struct TaskCompleteTool;

#[derive(Deserialize)]
pub struct TaskCompleteInput {
    pub summary: String,
}

impl AgentTool for TaskCompleteTool {
    fn name(&self) -> &'static str { "task_complete" }
    
    fn description(&self) -> &'static str {
        "Call this when the executor's work is complete and correct. 
         Provide a summary for the user. This ends the review loop."
    }
    
    fn run(&self, input: TaskCompleteInput, cx: &mut Context) -> Task<ToolResult> {
        // The tool result is the summary
        // Orchestrator pattern-matches on tool name to break loop immediately
        Task::ready(ToolResult {
            output: input.summary,
            is_error: false,
        })
    }
}
```

**Registration:** Only enable for discriminator profile/agent.

### Phase 2: Discriminator System Prompt

**File:** `zed/crates/agent/src/templates/discriminator_prompt.hbs` (new)

```handlebars
You are a code review agent. Your role is to verify that the executor agent 
completed the user's request correctly.

Review the executor's work and either:
1. Call task_complete with a summary if the work is correct
2. Provide specific feedback on what needs to be fixed

You see the executor's responses as USER messages. Your responses guide the executor.

{{> common_rules}}
```

### Phase 3: DualNativeAgent

**File:** `zed/crates/agent/src/dual.rs` (new)

```rust
pub struct DualNativeAgent {
    executor: NativeAgent,
    discriminator: NativeAgent,
}

struct DualSession {
    executor_session: Session,
    discriminator_session: Session,
    acp_thread: WeakEntity<AcpThread>,  // shared
}

impl DualNativeAgent {
    pub fn new(
        executor_profile: AgentProfileId,
        discriminator_profile: AgentProfileId,
        // ... other params
    ) -> Self { ... }
}
```

### Phase 4: DualNativeAgentConnection

**File:** `zed/crates/agent/src/dual.rs`

```rust
impl AgentConnection for DualNativeAgentConnection {
    fn new_thread(&self, project, cwd, cx) -> Task<Result<Entity<AcpThread>>> {
        // Create ONE AcpThread
        // Create executor Session pointing to it
        // Create discriminator Session pointing to same AcpThread
        // Initialize discriminator with "How can I help you?" + original task
    }
    
    fn prompt(&self, id, params, cx) -> Task<Result<PromptResponse>> {
        // THE ORCHESTRATION LOOP
        cx.spawn(async move |cx| {
            loop {
                // 1. Run executor turn
                let executor_events = executor_thread.send(user_content, cx)?;
                let executor_response = handle_thread_events(executor_events, acp_thread, cx).await?;
                
                // 2. Export executor turn to markdown
                let executor_output = export_turn_to_markdown(&executor_thread);
                
                // 3. Feed to discriminator as USER (role flip)
                let discrim_events = discriminator_thread.send(executor_output, cx)?;
                
                // 4. Handle discriminator events, watch for task_complete
                while let Some(event) = discrim_events.next().await {
                    match &event {
                        Ok(ThreadEvent::ToolCall(tc)) if tc.name == "task_complete" => {
                            // Stream this last tool call to UI
                            acp_thread.upsert_tool_call(tc, cx);
                            // BREAK IMMEDIATELY - don't wait for EndTurn
                            return Ok(PromptResponse { stop_reason: StopReason::EndTurn, meta: None });
                        }
                        _ => {
                            // Normal event handling - stream to shared acp_thread
                            forward_event_to_acp_thread(event, acp_thread, cx);
                        }
                    }
                }
                
                // 5. Discriminator gave feedback, not task_complete
                // Export discriminator turn, feed back to executor as USER
                let feedback = export_turn_to_markdown(&discriminator_thread);
                user_content = feedback;  // loop continues
            }
        })
    }
}
```

### Phase 5: Message Role Perspective

When feeding executor output to discriminator:

```rust
fn init_discriminator_session(task: &str, discriminator_thread: &mut Thread) {
    // From discriminator's perspective:
    // USER: "How can I help you?"
    // ASSISTANT: <the original task> (discriminator "asked for" verification)
    discriminator_thread.messages.push(Message::User(UserMessage {
        id: UserMessageId::new(),
        content: vec!["How can I help you?".into()],
    }));
    discriminator_thread.messages.push(Message::Agent(AgentMessage {
        content: vec![AgentMessageContent::Text(task.to_string())],
        ..Default::default()
    }));
}

fn feed_executor_output_to_discriminator(
    executor_output: String,
    discriminator_thread: &mut Thread,
) {
    // Executor's work becomes USER message to discriminator
    discriminator_thread.messages.push(Message::User(UserMessage {
        id: UserMessageId::new(),
        content: vec![executor_output.into()],
    }));
}
```

### Phase 6: Export Function

**File:** `zed/crates/agent/src/thread.rs` (add method)

```rust
impl Thread {
    /// Export the last agent turn (since last user message) to markdown
    pub fn export_last_turn(&self) -> String {
        let mut output = String::new();
        
        // Find last user message index
        let last_user_idx = self.messages.iter().rposition(|m| matches!(m, Message::User(_)));
        let start = last_user_idx.map(|i| i + 1).unwrap_or(0);
        
        for message in &self.messages[start..] {
            if let Message::Agent(agent_msg) = message {
                output.push_str(&agent_msg.to_markdown());
            }
        }
        
        if let Some(pending) = &self.pending_message {
            output.push_str(&pending.to_markdown());
        }
        
        output
    }
}
```

### Phase 7: UI Toggle

**File:** `zed/crates/agent_ui/src/acp/thread_view.rs`

Add UI element to toggle dual-agent mode. When enabled:
- Use `DualNativeAgentConnection` instead of `NativeAgentConnection`
- Or switch mid-session by spawning discriminator sibling

## Key Files to Modify

| File | Changes |
|------|---------|
| `acp_tools/src/lib.rs` | Register task_complete tool |
| `acp_tools/src/task_complete.rs` | NEW - the tool |
| `agent/src/dual.rs` | NEW - DualNativeAgent, DualSession, orchestration |
| `agent/src/agent.rs` | Export DualNativeAgent, possibly modify Session |
| `agent/src/thread.rs` | Add `export_last_turn()` method |
| `agent/src/templates/discriminator_prompt.hbs` | NEW - discriminator system prompt |
| `agent_settings/src/agent_profile.rs` | Add `system_prompt: Option<String>` field |
| `agent_ui/src/acp/thread_view.rs` | UI toggle for dual-agent mode |

## Design Decisions (Already Settled)

1. **Mid-session activation**: Spawn discriminator with existing context. User clicks "Auto" mid-chat, discriminator gets the full conversation history (role-flipped).

2. **Agent customization**: Need to extend Zed's agent framework to support custom system prompts per agent. This is required infrastructure - not just for discriminator, but for any multi-agent pattern.

3. **Model selection**: Different models per agent. Part of agent customization. Executor might use a reasoning model, discriminator might use a vision model to review screenshots/diffs.

4. **Cancellation**: User cancels → executor's partial response gets converted to markdown and shown as final agent response. Discriminator never sees it. No revert needed - user pre-empted before arbiter review.

5. **Token display**: Combined total (both agents contribute to the work).
