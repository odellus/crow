# Forking Zed: Dual-Agent Implementation Status

## Goal

Implement a dual-agent architecture in Zed where two agents (executor and discriminator) work together on the same conversation. The executor does the work, the discriminator reviews it, and they iterate until the discriminator is satisfied and calls `task_complete`.

## Key Insight

Both agents stream their events to the **same AcpThread**, which is the UI state. The UI sees interleaved output from both agents. This is possible because `handle_thread_events()` is stateless - it just takes an event stream and an `AcpThread` reference.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         DualAgentOrchestrator                                │
│  (orchestrates executor ↔ discriminator turns)                              │
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
│              forward to same AcpThread                                       │
└─────────────────────────────────────────────────────────────────────────────┘
                            │
                            ▼
              ┌─────────────────────────────┐
              │  AcpThread (shared UI)      │
              │  - interleaved entries      │
              │  - both agents visible      │
              └─────────────────────────────┘
```

## Message Role Perspective (Critical!)

From the discriminator's perspective, the executor is the USER:

```
DISCRIMINATOR'S VIEW:
  USER: "How can I help you?"           <- init message
  ASSISTANT: <the original task>         <- discriminator "asked for" this
  USER: <executor's work as markdown>    <- executor output, role-flipped
  ASSISTANT: <discriminator's review>    <- discriminator's response
```

This is exactly like what we did in Crow's `primary_dual.rs` and `dual.rs`.

## Implementation Status

### COMPLETED

#### 1. Custom System Prompt Support
**Files modified:**
- `zed/crates/settings/src/settings_content/agent.rs` - Added `system_prompt: Option<Arc<str>>` to `AgentProfileContent`
- `zed/crates/agent_settings/src/agent_profile.rs` - Added `system_prompt: Option<Arc<str>>` to `AgentProfileSettings`, updated `From` impl, `save_to_settings()`, and `create()`
- `zed/crates/agent/src/thread.rs` - Modified `build_request_messages()` to check for custom system prompt in profile and use it instead of the default template

This allows different agents to have different system prompts via profiles.

#### 2. task_complete Tool
**Files created/modified:**
- `zed/crates/agent/src/tools/task_complete_tool.rs` (NEW) - The tool that signals discriminator is satisfied
- `zed/crates/agent/src/tools.rs` - Added module declaration, pub use, and registered in `tools!` macro

The tool takes a `summary: String` parameter. When the orchestrator sees this tool call, it immediately breaks the loop and returns to the user. The summary becomes the final message.

#### 3. export_last_turn() Method
**Files modified:**
- `zed/crates/agent/src/thread.rs` - Added `pub fn export_last_turn(&self) -> String` method to Thread

This exports the last agent turn (since last user message) to markdown. Used for handoff between executor and discriminator.

#### 4. DualAgentOrchestrator (Compiles, not wired up)
**Files created/modified:**
- `zed/crates/agent/src/dual.rs` (NEW) - The orchestration logic
- `zed/crates/agent/src/agent.rs` - Added `mod dual;` and `pub use dual::*;`

The orchestrator:
- Takes executor and discriminator session IDs, a NativeAgentConnection, and a shared AcpThread
- Runs the dual-agent loop:
  1. Run executor turn
  2. Export executor output to markdown
  3. Send to discriminator as USER message (role flip)
  4. Run discriminator turn, watching for `task_complete` tool
  5. If task_complete → break loop, return to user
  6. Else → export discriminator feedback, send back to executor, loop

**Key functions:**
- `DualAgentOrchestrator::new()` - Constructor
- `DualAgentOrchestrator::run()` - Entry point, spawns the async loop
- `run_loop()` - The actual orchestration loop
- `forward_events_to_acp_thread()` - Streams ThreadEvents to shared AcpThread
- `forward_events_watching_for_complete()` - Same but watches for task_complete tool call
- `handle_event()` - Handles individual ThreadEvent by forwarding to AcpThread

**Build status:** ✅ Compiles successfully (`cargo check -p agent` passes)

### NOT YET DONE

#### 5. Wire Two Sessions to Same AcpThread
This is the next step. We need to:
- Modify `NativeAgent` to create a dual session when requested
- Both sessions should point to the same `AcpThread`
- Initialize the discriminator session with the role-flipped history

#### 6. Initialize Discriminator History
When creating the discriminator session mid-conversation:
- Copy existing conversation history
- Flip roles: executor ASSISTANT → discriminator USER
- Add initial "How can I help you?" + task setup

#### 7. UI Toggle
Add a way for users to activate dual-agent mode. Options:
- Button in agent panel
- Keyboard shortcut
- Profile setting

#### 8. Agent Labels in UI
Prefix agent responses with "Executor:" or "Reviewer:" so user can distinguish them. Could be done via:
- System prompt instruction ("Always start with 'Executor:'")
- Annotations in ContentBlock
- UI-level detection

## Key Files Reference

| File | Purpose | Status |
|------|---------|--------|
| `agent_settings/src/agent_profile.rs` | Profile with custom system_prompt | ✅ Modified |
| `settings/src/settings_content/agent.rs` | Settings serialization | ✅ Modified |
| `agent/src/thread.rs` | Thread, ReACT loop, export_last_turn | ✅ Modified |
| `agent/src/tools/task_complete_tool.rs` | task_complete tool | ✅ Created |
| `agent/src/tools.rs` | Tool registration | ✅ Modified |
| `agent/src/dual.rs` | DualAgentOrchestrator | ✅ Created |
| `agent/src/agent.rs` | NativeAgent, module exports | ✅ Modified (exports) |
| `agent/src/agent.rs` | Session creation for dual-agent | ❌ Not yet |
| `agent_ui/src/acp/thread_view.rs` | UI toggle | ❌ Not yet |

## Design Decisions (Already Made)

1. **Mid-session activation**: User clicks "Auto" mid-chat → discriminator gets full conversation history (role-flipped)

2. **Agent customization**: Custom system prompts per profile. Different tools per agent (only discriminator gets task_complete)

3. **Model selection**: Different models per agent. Executor might use reasoning model, discriminator might use vision model

4. **Cancellation**: User cancels → executor's partial response gets converted to markdown and shown as final. Discriminator never sees it

5. **Token display**: Combined total (both agents contribute to work)

## Code Locations for Next Steps

### To create dual session in NativeAgent:

Look at `NativeAgent::register_session()` in `agent/src/agent.rs:303`. This creates a single session with an AcpThread. For dual mode, we need to:
1. Create one AcpThread
2. Create two Sessions (executor + discriminator) both pointing to same AcpThread
3. Initialize discriminator with flipped history

### To trigger dual-agent mode:

The entry point is `AgentConnection::prompt()` in `agent/src/agent.rs:1044`. Currently it calls `NativeAgentConnection::run_turn()`. For dual mode, we'd use `DualAgentOrchestrator::run()` instead.

### UI integration:

`AcpThreadView::send_impl()` in `agent_ui/src/acp/thread_view.rs` is where user messages are sent. This would be modified to check if dual-agent mode is enabled.

## Documentation Files Created

- `crow-tauri/ZED_AGENT_ARCHITECTURE.md` - Complete Zed agent architecture deep dive
- `crow-tauri/DUAL_AGENT_ZED_PLAN.md` - Implementation plan with code examples
- `crow-tauri/FORKING_ZED.md` - This file, current status

## Git Status

Modified files in zed/:
- `crates/settings/src/settings_content/agent.rs`
- `crates/agent_settings/src/agent_profile.rs`
- `crates/agent/src/thread.rs`
- `crates/agent/src/tools.rs`
- `crates/agent/src/agent.rs`

New files in zed/:
- `crates/agent/src/tools/task_complete_tool.rs`
- `crates/agent/src/dual.rs`
