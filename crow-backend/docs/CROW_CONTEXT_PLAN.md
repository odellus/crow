# Crow Context System Plan

**Goal:** Ensure Crow passes complete context to agents, matching OpenCode

---

## Current State in Crow

### What Works Well
- **SystemPromptBuilder** - 4-layer architecture implemented
  - Header (provider-specific)
  - Agent/provider prompt
  - Environment context (PWD, git, platform, date, file tree)
  - Custom instructions (AGENTS.md, CLAUDE.md)
- **ToolContext** - Basic context passed to tools
  - session_id, message_id, agent, working_dir

### What's Missing/Incomplete

| Feature | OpenCode | Crow | Gap |
|---------|----------|------|-----|
| Project root (worktree) | ✅ Instance.worktree | ❌ Not tracked | Need separate from working_dir |
| Provider ID in tool ctx | ✅ ctx.extra.providerID | ❌ Missing | Tools can't check provider |
| Model ID in tool ctx | ✅ ctx.extra.modelID | ❌ Missing | Tools can't check model |
| Abort signal | ✅ ctx.abort | ❌ Missing | Can't cancel tools |
| Call ID | ✅ ctx.callID | ❌ Missing | Tool call tracking |
| Metadata callback | ✅ ctx.metadata() | ❌ Missing | Dynamic title updates |
| Message path tracking | ✅ msg.path.cwd/root | ❌ Missing | Audit trail |
| Dynamic reminders | ✅ insertReminders() | ❌ Missing | Plan/build transitions |

---

## Implementation Plan

### Phase 1: Enhanced ToolContext

**Update `crow/packages/api/src/tools/mod.rs`:**

```rust
/// Tool execution context - provides session and environment info
#[derive(Clone)]
pub struct ToolContext {
    // Required fields
    pub session_id: String,
    pub message_id: String,
    pub agent: String,
    pub working_dir: PathBuf,
    
    // NEW: Additional context matching OpenCode
    pub project_root: PathBuf,        // Git root or working_dir
    pub call_id: Option<String>,      // Unique tool call ID
    pub provider_id: Option<String>,  // LLM provider
    pub model_id: Option<String>,     // LLM model
    
    // NEW: Cancellation support
    pub abort: Option<tokio_util::sync::CancellationToken>,
    
    // NEW: Extra context for tools
    pub extra: HashMap<String, Value>,
}

impl ToolContext {
    /// Check if tool should abort
    pub fn should_abort(&self) -> bool {
        self.abort.as_ref().map(|t| t.is_cancelled()).unwrap_or(false)
    }
    
    /// Get project root, defaulting to working_dir
    pub fn root(&self) -> &Path {
        &self.project_root
    }
}
```

### Phase 2: Project Root Tracking

**Add to session or executor:**

```rust
/// Find project root (git root or current directory)
fn find_project_root(working_dir: &Path) -> PathBuf {
    // Try git root first
    let output = std::process::Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .current_dir(working_dir)
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return PathBuf::from(root);
        }
    }
    
    // Fall back to working directory
    working_dir.to_path_buf()
}
```

### Phase 3: Message Path Tracking

**Update AssistantMessage in types.rs:**

```rust
pub struct MessagePath {
    pub cwd: String,   // Working directory at execution time
    pub root: String,  // Project root at execution time
}

// In Message::Assistant
pub path: MessagePath,
```

This creates an audit trail of where each message was executed.

### Phase 4: Dynamic Reminders

**Add to executor:**

```rust
/// Insert dynamic reminders into user messages
/// Matches OpenCode's insertReminders() from session/prompt.ts
fn insert_reminders(
    messages: &mut Vec<ChatCompletionRequestMessage>,
    agent: &AgentInfo,
    previous_agent: Option<&str>,
) {
    // Find last user message
    let last_user_idx = messages.iter().rposition(|m| {
        matches!(m, ChatCompletionRequestMessage::User(_))
    });
    
    if let Some(idx) = last_user_idx {
        let mut additions = vec![];
        
        // Plan agent reminder
        if agent.name == "plan" {
            additions.push(include_str!("../prompts/plan_reminder.txt").to_string());
        }
        
        // Build-switch reminder (plan → build transition)
        if previous_agent == Some("plan") && agent.name == "build" {
            additions.push(include_str!("../prompts/build_switch.txt").to_string());
        }
        
        // Inject as additional content in user message
        if !additions.is_empty() {
            // Append to user message content
            if let ChatCompletionRequestMessage::User(user_msg) = &mut messages[idx] {
                // Add reminders as synthetic parts
            }
        }
    }
}
```

### Phase 5: Abort Signal Support

**In executor:**

```rust
pub struct AgentExecutor {
    // ... existing fields
    cancellation: tokio_util::sync::CancellationToken,
}

impl AgentExecutor {
    pub async fn execute_turn(&self, ...) -> Result<MessageWithParts, String> {
        // Pass cancellation token to tool context
        let tool_ctx = ToolContext {
            session_id: session_id.to_string(),
            message_id: message_id.clone(),
            agent: agent_id.to_string(),
            working_dir: working_dir.to_path_buf(),
            project_root: find_project_root(working_dir),
            call_id: Some(tool_call.id.clone()),
            provider_id: Some(self.provider.config().name.clone()),
            model_id: Some(model_id.clone()),
            abort: Some(self.cancellation.clone()),
            extra: HashMap::new(),
        };
        
        // Check abort between iterations
        if self.cancellation.is_cancelled() {
            return Err("Session aborted".to_string());
        }
    }
    
    pub fn abort(&self) {
        self.cancellation.cancel();
    }
}
```

### Phase 6: Update Tools to Use New Context

**Example - bash.rs:**

```rust
async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
    // Check abort before execution
    if ctx.should_abort() {
        return ToolResult {
            status: ToolStatus::Error,
            output: String::new(),
            error: Some("Aborted".to_string()),
            metadata: json!({}),
        };
    }
    
    // Use project_root for relative path resolution
    let cwd = input.get("cwd")
        .and_then(|v| v.as_str())
        .map(|s| ctx.root().join(s))
        .unwrap_or_else(|| ctx.working_dir.clone());
    
    // Execute command
    let mut child = Command::new(shell)
        .args(args)
        .current_dir(&cwd)  // Uses resolved path
        .spawn()?;
    
    // Periodically check abort during execution
    loop {
        if ctx.should_abort() {
            child.kill()?;
            return ToolResult { /* ... */ };
        }
        // Check if process completed
    }
}
```

---

## Context Flow Diagram (After Updates)

```
Request arrives at /session/:id/message
    ↓
Server handler:
    - Load session (get working_dir)
    - Find project_root (git root)
    - Create cancellation token
    ↓
AgentExecutor::execute_turn(session_id, agent_id, working_dir, parts)
    ↓
Build system prompt:
    SystemPromptBuilder::new(agent, working_dir, provider_id)
        - Header (provider-specific)
        - Agent/provider prompt
        - Environment (working_dir, project_root, platform, date, tree)
        - Custom instructions (AGENTS.md)
    ↓
Build message history:
    - Convert stored messages to LLM format
    - insert_reminders(messages, agent, previous_agent)
    ↓
ReACT loop:
    - Call LLM with system prompt + messages + tools
    - For each tool call:
        ToolContext {
            session_id,
            message_id,
            agent,
            working_dir,
            project_root,      // NEW
            call_id,           // NEW
            provider_id,       // NEW
            model_id,          // NEW
            abort,             // NEW
            extra,             // NEW
        }
        - Execute tool with full context
        - Check abort between tools
    ↓
Store message:
    MessageWithParts {
        info: Message::Assistant {
            path: MessagePath {   // NEW
                cwd: working_dir,
                root: project_root,
            },
            // ... other fields
        },
        parts: [...]
    }
```

---

## Files to Modify

| File | Changes |
|------|---------|
| `tools/mod.rs` | Expand ToolContext struct |
| `agent/executor.rs` | Add project_root, abort, reminders |
| `types.rs` | Add MessagePath to Assistant message |
| `server.rs` | Pass project_root to executor |
| `tools/bash.rs` | Use project_root, check abort |
| `tools/read.rs` | Use project_root for relative paths |
| `tools/write.rs` | Use project_root for relative paths |
| `tools/edit.rs` | Use project_root for relative paths |

---

## Prompt Files Needed

Create these in `crow/packages/api/src/prompts/`:

1. **plan_reminder.txt** - Instructions for plan agent
   - Read-only commands only
   - Focus on analysis not implementation
   
2. **build_switch.txt** - Transition from plan to build
   - Signal that planning phase is complete
   - Now implement the plan

---

## Testing

1. **Context propagation test:**
   - Verify working_dir reaches tools
   - Verify project_root is git root (or working_dir)
   
2. **Abort test:**
   - Start long-running bash command
   - Call abort endpoint
   - Verify tool terminates
   
3. **Reminder injection test:**
   - Send message with agent=plan
   - Verify plan reminder injected
   - Switch to build agent
   - Verify build-switch reminder injected

4. **Path tracking test:**
   - Execute session in subdirectory
   - Verify message.path.cwd = subdirectory
   - Verify message.path.root = git root

---

## Priority Order

1. **P0: ToolContext expansion** - Add project_root, call_id, provider_id, model_id
2. **P1: Project root detection** - Find git root for all sessions
3. **P1: Message path tracking** - Audit trail in stored messages
4. **P2: Abort signal** - Cancellation support
5. **P3: Dynamic reminders** - Plan/build transitions

---

## Benefits

- **Tools know their context** - Provider, model, project root
- **Cancellation works** - Long-running tools can be aborted
- **Audit trail** - Know where each message was executed
- **Agent transitions** - Smooth handoff between plan and build
- **OpenCode compatibility** - Same context structure
