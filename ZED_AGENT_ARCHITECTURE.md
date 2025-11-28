# Zed Agent Architecture Deep Dive

## Crate Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              agent_ui                                        │
│  UI components: AcpThreadView, MessageEditor, AgentPanel, AgentDiff         │
│  Handles: user input, rendering, tool authorization dialogs                 │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ Entity<AcpThread>
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              acp_thread                                      │
│  AcpThread: conversation state (entries, terminals, diffs)                  │
│  AgentConnection trait: abstraction over agent backends                     │
│  Handles: message accumulation, tool call state, git checkpoints           │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    │                               │
                    ▼                               ▼
┌───────────────────────────────┐   ┌───────────────────────────────┐
│           agent               │   │        agent_servers          │
│  NativeAgent + Thread         │   │  AcpConnection (external)     │
│  Built-in Zed agent           │   │  Claude, Gemini, Codex        │
│  ReACT loop in Rust           │   │  ReACT loop in subprocess     │
└───────────────────────────────┘   └───────────────────────────────┘
                    │
                    ▼
┌───────────────────────────────┐
│        agent_settings         │
│  AgentProfileSettings         │
│  Tools, models, MCP servers   │
└───────────────────────────────┘
```

## Key Types and Their Relationships

### 1. AcpThreadView (agent_ui/src/acp/thread_view.rs)
The main UI component for an agent conversation.

```rust
pub struct AcpThreadView {
    agent: Rc<dyn AgentServer>,           // The backend (native or external)
    thread_state: ThreadState,             // Loading | Ready { thread: Entity<AcpThread> } | ...
    message_editor: Entity<MessageEditor>, // User input area
    entry_view_state: Entity<EntryViewState>, // Renders conversation entries
    // ...
}
```

**Key methods:**
- `send_impl()` → calls `thread.send(contents, cx)` → triggers agent turn
- `handle_thread_event()` → responds to AcpThreadEvent (new entries, errors, etc.)

### 2. AcpThread (acp_thread/src/acp_thread.rs)
The conversation state holder. Doesn't know about LLMs directly - delegates to connection.

```rust
pub struct AcpThread {
    title: SharedString,
    entries: Vec<AgentThreadEntry>,        // The conversation history
    connection: Rc<dyn AgentConnection>,   // Backend that does the actual work
    session_id: acp::SessionId,
    // terminals, diffs, etc.
}

pub enum AgentThreadEntry {
    UserMessage(UserMessage),
    AssistantMessage(AssistantMessage),
    ToolCall(ToolCall),
}
```

**Key methods:**
- `send()` → creates UserMessage entry, calls `connection.prompt()`, handles response
- `push_user_content_block()` / `push_assistant_content_block()` → add to entries
- `upsert_tool_call()` → add/update tool call in entries

### 3. AgentConnection trait (acp_thread/src/connection.rs)
The abstraction that allows different agent backends.

```rust
pub trait AgentConnection {
    fn new_thread(...) -> Task<Result<Entity<AcpThread>>>;
    fn prompt(...) -> Task<Result<acp::PromptResponse>>;
    fn cancel(...);
    fn model_selector(...) -> Option<Rc<dyn AgentModelSelector>>;
    // ...
}
```

**Implementations:**
1. `NativeAgentConnection` (agent/src/agent.rs) - Zed's built-in agent
2. `AcpConnection` (agent_servers/src/acp.rs) - External agents via ACP protocol

### 4. NativeAgent + Thread (agent/src/agent.rs, agent/src/thread.rs)
Zed's built-in agent implementation.

```rust
// agent.rs
pub struct NativeAgent {
    sessions: HashMap<acp::SessionId, Session>,
    templates: Arc<Templates>,              // System prompt templates
    models: LanguageModels,
    // ...
}

struct Session {
    thread: Entity<Thread>,                 // The actual agent logic
    acp_thread: WeakEntity<AcpThread>,      // For streaming events to UI
}

// thread.rs  
pub struct Thread {
    messages: Vec<Message>,                 // Internal message format
    tools: BTreeMap<SharedString, Arc<dyn AnyAgentTool>>,
    profile_id: AgentProfileId,             // Controls which tools enabled
    model: Option<Arc<dyn LanguageModel>>,
    templates: Arc<Templates>,              // For system prompt
    // ...
}

pub enum Message {
    User(UserMessage),
    Agent(AgentMessage),
    Resume,
}
```

**Key methods in Thread:**
- `send()` → adds user message, calls `run_turn()`
- `run_turn()` → spawns async task for `run_turn_internal()`
- `run_turn_internal()` → THE REACT LOOP
- `build_completion_request()` → builds LLM request with system prompt
- `build_request_messages()` → converts Thread messages to LLM format

### 5. The ReACT Loop (agent/src/thread.rs:1218)

```rust
async fn run_turn_internal(
    this: &WeakEntity<Self>,
    model: Arc<dyn LanguageModel>,
    event_stream: &ThreadEventStream,
    cx: &mut AsyncApp,
) -> Result<()> {
    let mut intent = CompletionIntent::UserPrompt;
    loop {
        // 1. Build request (system prompt + messages + tools)
        let request = this.build_completion_request(intent, cx)??;
        
        // 2. Call LLM
        let events = model.stream_completion(request, cx).await?;
        
        // 3. Process streaming events, execute tools
        let mut tool_results = FuturesUnordered::new();
        while let Some(event) = events.next().await {
            tool_results.extend(this.handle_completion_event(event, event_stream, cx)??);
        }
        
        // 4. Wait for tools to complete
        let end_turn = tool_results.is_empty();
        while let Some(tool_result) = tool_results.next().await {
            // Store results
        }
        
        // 5. Decision: continue or return
        if end_turn {
            return Ok(());  // ← TURN ENDS, control returns to user
        } else {
            intent = CompletionIntent::ToolResults;  // Loop with tool results
        }
    }
}
```

### 6. System Prompt (agent/src/templates/system_prompt.hbs)
A Handlebars template that generates the system prompt. Includes:
- Base instructions (communication style, tool use guidelines)
- Project worktrees
- User rules (from .cursorrules, CLAUDE.md, etc.)
- Model-specific info

**Current limitation: ONE template for all agents. No per-agent customization.**

### 7. AgentProfileSettings (agent_settings/src/agent_profile.rs)
Controls agent behavior per-profile.

```rust
pub struct AgentProfileSettings {
    pub name: SharedString,
    pub tools: IndexMap<Arc<str>, bool>,           // Which tools enabled
    pub enable_all_context_servers: bool,
    pub context_servers: IndexMap<Arc<str>, ContextServerPreset>,
    pub default_model: Option<LanguageModelSelection>,
    // NOTE: No custom system prompt field!
}
```

## Data Flow: User Message → LLM Response

```
1. User types in MessageEditor, presses Enter
   └─→ AcpThreadView.send()
   
2. AcpThreadView.send_impl()
   └─→ message_editor.contents() → Vec<acp::ContentBlock>
   └─→ thread.send(contents, cx)
   
3. AcpThread.send() 
   └─→ push UserMessage entry to self.entries
   └─→ connection.prompt(request, cx)
   
4a. [NativeAgentConnection] NativeAgentConnection.run_turn()
    └─→ Thread.send(user_message, cx)
    └─→ Thread.run_turn() → run_turn_internal()
    └─→ ReACT loop executes
    └─→ ThreadEvents streamed back
    └─→ handle_thread_events() → AcpThread.push_*()

4b. [AcpConnection] External agent via ACP protocol
    └─→ conn.prompt(params).await
    └─→ Events streamed from subprocess
    └─→ AcpThread.push_*()

5. AcpThread emits AcpThreadEvent::NewEntry, etc.
   └─→ AcpThreadView.handle_thread_event()
   └─→ UI updates
```

## Extension Points for Multi-Agent

### Option 1: Custom System Prompts per Profile
Add `system_prompt: Option<String>` to `AgentProfileSettings`.
Modify `Thread.build_request_messages()` to use profile's prompt if set.

### Option 2: Multiple Agent Types
Create new "agent type" concept separate from profiles:
- Executor agent (does work)
- Discriminator agent (reviews)

Each type has its own system prompt template.

### Option 3: DualAgentConnection
Wrap two connections/threads, orchestrate between them:

```rust
pub struct DualAgentConnection {
    executor: NativeAgentConnection,
    discriminator: NativeAgentConnection,
    shared_acp_thread: WeakEntity<AcpThread>,  // Both stream to same UI
}

impl AgentConnection for DualAgentConnection {
    fn prompt(&self, ...) -> Task<Result<PromptResponse>> {
        // 1. Run executor turn
        // 2. On EndTurn, render to markdown
        // 3. Send to discriminator as user message
        // 4. Run discriminator turn
        // 5. Check for task_complete or loop
    }
}
```

### Option 4: Orchestrator at Thread level
Add sibling thread concept to Thread itself:

```rust
pub struct Thread {
    // ... existing ...
    sibling_thread: Option<WeakEntity<Thread>>,
    agent_role: Option<AgentRole>,  // Executor | Discriminator
}
```

After `run_turn_internal()` returns, check for sibling and continue.

## Key Files Reference

| File | Purpose |
|------|---------|
| `agent_ui/src/acp/thread_view.rs` | Main UI, handles send/receive |
| `acp_thread/src/acp_thread.rs` | Conversation state |
| `acp_thread/src/connection.rs` | AgentConnection trait |
| `agent/src/agent.rs` | NativeAgent, NativeAgentConnection |
| `agent/src/thread.rs` | Thread, ReACT loop |
| `agent/src/templates.rs` | Template management |
| `agent/src/templates/system_prompt.hbs` | THE system prompt |
| `agent_settings/src/agent_profile.rs` | Profile configuration |
| `agent_servers/src/acp.rs` | External agent connection |

---

## Detailed Extension Point Analysis

### Critical Injection Points

#### 1. `Thread.run_turn_internal()` - The EndTurn Decision (thread.rs:1218)

This is the ReACT loop. Currently:
```rust
if end_turn {
    return Ok(());  // ← Control returns to user
} else {
    intent = CompletionIntent::ToolResults;  // Loop continues with tool results
}
```

**For dual-agent**: Instead of returning on `end_turn`, we could:
1. Check if this is an executor turn
2. Serialize the turn output 
3. Hand off to discriminator
4. Only return when discriminator says `task_complete`

#### 2. `Thread.build_request_messages()` - System Prompt Injection (thread.rs:1968)

```rust
let system_prompt = SystemPromptTemplate {
    project: self.project_context.read(cx),
    available_tools,
    model_name: self.model.as_ref().map(|m| m.name().0.to_string()),
}
.render(&self.templates)
```

**For dual-agent**: Add `agent_role: Option<AgentRole>` to Thread, pass to template:
```rust
let system_prompt = SystemPromptTemplate {
    project: self.project_context.read(cx),
    available_tools,
    model_name: ...,
    agent_role: self.agent_role,  // NEW: Executor | Discriminator | None
    custom_prompt: self.custom_system_prompt.as_deref(),  // NEW
}
```

#### 3. `NativeAgentConnection.prompt()` - Turn Orchestration (agent.rs:1029)

```rust
fn prompt(&self, id, params, cx) -> Task<Result<acp::PromptResponse>> {
    self.run_turn(session_id, cx, move |thread, cx| {
        thread.update(cx, |thread, cx| thread.send(id, content, cx))
    })
}
```

**For dual-agent**: This is where we'd intercept. A `DualAgentConnection` would:
1. Call executor's prompt()
2. Capture the PromptResponse 
3. If stop_reason == EndTurn and not task_complete, send to discriminator
4. Loop until discriminator calls task_complete

#### 4. `NativeAgentConnection.handle_thread_events()` - Event Streaming (agent.rs:765)

All ThreadEvents flow through here to reach AcpThread (UI). For dual-agent, we need to:
- Tag events with agent role (executor vs discriminator)
- UI can then render with appropriate labels

```rust
ThreadEvent::AgentText(text) => {
    acp_thread.update(cx, |thread, cx| {
        thread.push_assistant_content_block(
            acp::ContentBlock::Text(acp::TextContent {
                text,
                annotations: Some(vec![
                    ("agent_role".into(), "executor".into())  // NEW
                ]),
                meta: None,
            }),
            false,
            cx,
        )
    })?;
}
```

### Minimal Implementation Path

The least invasive approach for dual-agent:

1. **Add `custom_system_prompt` to `AgentProfileSettings`** (agent_settings)
   - Profiles can now have different prompts
   - No changes to Thread/ReACT loop needed

2. **Create `DualAgentSession` wrapper** (new file in agent crate)
   ```rust
   pub struct DualAgentSession {
       executor_session: Session,
       discriminator_session: Session,
       shared_acp_thread: WeakEntity<AcpThread>,
   }
   ```

3. **Modify `NativeAgent.new_thread()`** to optionally create dual session
   - Based on profile flag `dual_agent: bool` or separate UI option

4. **Implement orchestration in `DualAgentSession.prompt()`**
   - Run executor turn
   - Convert output to discriminator input (role reversal like in Crow)
   - Run discriminator turn
   - Check for task_complete tool call
   - Loop or return

5. **Add `task_complete` tool** (new tool in acp_tools)
   - Only available to discriminator
   - When called, breaks the dual-agent loop

### Message Role Perspective (from Crow R&D)

Critical insight from Crow's `primary_dual.rs`:

```
FROM DISCRIMINATOR'S PERSPECTIVE:
  USER: "How can I help you?" (init message)
  ASSISTANT: <the original task> (discriminator "asked for" this)
  USER: <executor's work> (executor is USER to discriminator)
  ASSISTANT: <discriminator's review/feedback>
```

This means when forwarding executor output to discriminator:
- Executor's ASSISTANT messages become discriminator's USER messages
- Discriminator's responses are ASSISTANT from its perspective

The Thread needs to know its role to correctly build messages.

### UI Considerations

For interleaved single-pane UI:
- `AssistantMessage` entries need role annotation
- `ContentBlock` already has `annotations: Option<Vec<(String, String)>>`
- UI can read annotation to render "Executor:" vs "Reviewer:" labels
- Tool calls already have `id` - can extend to include agent source
