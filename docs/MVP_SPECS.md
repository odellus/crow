# OpenCode-Lab MVP Specifications (CORRECTED)

## What is the MVP?

**An orchestrator agent system with DualAgent at its core and Dioxus frontend for visualization.**

This is NOT a TUI. This is NOT a CLI-first tool. This IS:
- **DualAgent-first** - Executor + Discriminator is the PRIMARY pattern
- **Orchestration-focused** - Multi-agent coordination, not single-agent execution
- **Dioxus frontend** - Web UI to visualize sessions, todos, review workflow
- **Server architecture** - HTTP API + WebSocket for live updates
- **Single-agent as fallback** - Corner case for simple tasks

**Timeline:** 3-4 months to MVP

## Core Philosophy Correction

> **"This is an orchestration agent, not a TUI!"**

**What I got wrong:**
- ❌ Treated single-agent as primary, dual-agent as advanced feature
- ❌ Put Dioxus frontend in "post-MVP"
- ❌ Focused on CLI execution
- ❌ Missed the orchestration angle entirely

**What it actually is:**
- ✅ DualAgent is THE core pattern
- ✅ Dioxus frontend IS the MVP interface
- ✅ Orchestration = coordinating multiple dual-agent sessions
- ✅ Single-agent is just dual-agent with discriminator=None (corner case)
- ✅ Server-first, not CLI-first

## MVP Features (In Scope) - REVISED

### 1. Core: DualAgent Pattern (PRIMARY)

**This is the foundation, not an addon:**

```rust
// Every execution is dual-agent by default
pub struct Agent {
    pub name: String,
    pub discriminator: Option<DiscriminatorConfig>,  // None = single-agent (rare)
}

// DualAgent is the DEFAULT execution path
impl Agent {
    pub async fn execute(&self, task: String) -> Result<ExecutionResult> {
        match &self.discriminator {
            Some(disc_config) => {
                // PRIMARY PATH: Dual-agent with review workflow
                DualAgentRuntime::run(self, disc_config, task).await
            }
            None => {
                // FALLBACK: Single-agent (corner case)
                SingleAgentRuntime::run(self, task).await
            }
        }
    }
}
```

**Built-in Agents (All with discriminators by default):**
- `build` - Executor agent (HAS discriminator: "discriminator-strict")
- `supervisor` - Delegates to build agents (HAS discriminator: "architect")
- `orchestrator` - Multi-project coordination (HAS discriminator: "supervisor")

**Single-agent mode only for:**
- Quick read-only queries
- User explicitly disables discriminator
- Testing/debugging

### 2. Reflection Sessions (SharedConversation)

**Two coupled sessions viewing one conversation:**

```rust
pub struct SharedConversation {
    pub id: String,
    pub messages: Vec<RawMessage>,     // Ground truth
    pub todos: Vec<SharedTodo>,        // Shared todo list
    pub executor_session_id: String,   // Executor's view
    pub discriminator_session_id: String,  // Discriminator's view
    pub status: ConversationStatus,
}

pub enum SessionType {
    Executor { discriminator_id, shared_conversation_id },
    Discriminator { executor_id, shared_conversation_id },
    Primary { id },  // Rare - only for single-agent fallback
}
```

**Automatic perspective transformation** - sessions project different views of same conversation

### 3. Shared Todo List with Review Workflow

**The coordination mechanism between executor and discriminator:**

```rust
pub enum TodoStatus {
    Proposed,      // Executor proposes
    Pending,       // Discriminator approves proposal
    InProgress,    // Executor working
    UnderReview,   // Executor sends for review
    Completed,     // Discriminator approves
    Rejected,      // Discriminator rejects with feedback
}

// Executor tools
send_for_review(todo_id, summary, artifacts)

// Discriminator tools
approve_review(todo_id, approved, feedback)
task_done(ready, summary)  // Only when all todos complete
```

**This is not optional - this IS the workflow**

### 4. Dioxus Frontend (IN MVP, NOT POST-MVP)

**Why it's essential:**
- Visualize executor vs discriminator perspectives side-by-side
- Show todo list with statuses and review workflow
- See real-time updates as agents work
- Approve/reject reviews via UI
- Monitor orchestration (multiple dual-agent sessions)

**MVP UI Features:**

#### Main View: Dual-Session Dashboard
```
┌─────────────────────────────────────────────────────────────┐
│  OpenCode-Lab - Session: "Implement JWT Auth"              │
├─────────────────────┬───────────────────────────────────────┤
│  EXECUTOR           │  DISCRIMINATOR                        │
├─────────────────────┼───────────────────────────────────────┤
│ [Agent: build]      │  [Agent: discriminator-strict]        │
│                     │                                       │
│ > I'll implement    │  USER: Review executor's work         │
│   JWT auth          │                                       │
│                     │  ASSISTANT: Let me check the code     │
│ Tool: write         │                                       │
│   file: auth.rs     │  Tool: read                           │
│   output: Created   │    file: auth.rs                      │
│                     │                                       │
│ Tool: send_for_     │  Tool: bash                           │
│   review            │    cmd: cargo test auth               │
│   todo: auth-impl   │    output: 3 passed                   │
│                     │                                       │
│ [Waiting for        │  Tool: approve_review                 │
│  review...]         │    todo: auth-impl                    │
│                     │    approved: true                     │
└─────────────────────┴───────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  TODO LIST (Shared)                                         │
├─────────────────────────────────────────────────────────────┤
│  ✓ Implement JWT authentication        [Completed]          │
│  ● Add tests for auth                  [In Progress]        │
│  ○ Update documentation                [Pending]            │
└─────────────────────────────────────────────────────────────┘
```

#### Session List View
```
┌─────────────────────────────────────────────────────────────┐
│  Active Sessions                                            │
├─────────────────────────────────────────────────────────────┤
│  📝 Implement JWT Auth                    [Dual]  Active    │
│     └─ Executor: build                                      │
│     └─ Discriminator: discriminator-strict                  │
│     └─ Todos: 2/3 complete                                  │
│                                                             │
│  📝 Add rate limiting                     [Dual]  Review    │
│     └─ Executor: build                                      │
│     └─ Discriminator: discriminator-strict                  │
│     └─ Awaiting review on 1 todo                            │
│                                                             │
│  📝 Update README                         [Single]  Done    │
│     └─ Agent: build                                         │
└─────────────────────────────────────────────────────────────┘
```

#### Todo Review Interface
```
┌─────────────────────────────────────────────────────────────┐
│  Todo Under Review                                          │
├─────────────────────────────────────────────────────────────┤
│  Task: Implement JWT authentication                         │
│  Status: Under Review (sent 2 minutes ago)                  │
│                                                             │
│  Executor Summary:                                          │
│  "Implemented JWT auth with HS256, expiration checking,    │
│   role-based claims. Added comprehensive tests."           │
│                                                             │
│  Artifacts:                                                 │
│  📄 src/auth.rs         [View Diff]                         │
│  📄 src/jwt.rs          [View Diff]                         │
│  📄 tests/auth_test.rs  [View Diff]                         │
│                                                             │
│  Test Results:                                              │
│  ✓ test_valid_token ... ok                                 │
│  ✓ test_expired_token ... ok                               │
│  ✓ test_invalid_signature ... ok                           │
│                                                             │
│  [Approve] [Reject with Feedback]                          │
└─────────────────────────────────────────────────────────────┘
```

**Dioxus Tech:**
- Server-side rendering with live updates
- WebSocket for real-time session updates
- Component-based architecture
- Responsive design (works in browser)

### 5. Server Architecture (HTTP + WebSocket)

**Not a CLI tool - it's a server with web UI:**

```
┌─────────────────────────────────────────────┐
│  opencode-lab server                        │
│                                             │
│  HTTP API:                                  │
│  - POST /session/create                     │
│  - POST /session/{id}/execute               │
│  - GET  /session/{id}                       │
│  - GET  /session/{id}/messages              │
│  - GET  /conversation/{id}/todos            │
│  - POST /todo/{id}/approve_review           │
│                                             │
│  WebSocket:                                 │
│  - /ws/session/{id}  (live updates)         │
│                                             │
│  Dioxus:                                    │
│  - GET  /  (main UI)                        │
│  - GET  /session/{id}  (session view)       │
└─────────────────────────────────────────────┘
```

**Also exposes ACP for Zed integration**, but Dioxus UI is primary interface.

### 6. Orchestration (Multi-Session Coordination)

**The whole point of this project:**

```rust
// Orchestrator agent coordinates MULTIPLE dual-agent sessions
pub struct Orchestrator {
    pub sessions: Vec<DualAgentSession>,
}

impl Orchestrator {
    pub async fn coordinate(&mut self, project: ProjectSpec) {
        // Break project into tasks
        let tasks = self.plan(project);
        
        // Spawn dual-agent session for each task
        for task in tasks {
            let session = DualAgentRuntime::create(task).await;
            self.sessions.push(session);
        }
        
        // Monitor progress across all sessions
        while !self.all_complete() {
            self.check_progress().await;
            self.handle_dependencies().await;
        }
    }
}
```

**Orchestrator monitors:**
- Multiple executor/discriminator pairs working in parallel
- Todo lists across all sessions
- Dependencies between tasks
- Overall project progress

**UI shows all sessions** - not just one

### 7. Tool System (Same as Before)

**Core Tools:**
- read, write, edit, bash, grep, glob
- todowrite, todoread (permission-aware)
- send_for_review (executor-only)
- approve_review (discriminator-only)
- task_done (discriminator-only)
- task (spawn subagent - also dual by default)

### 8. LLM Provider System (Same)

- OpenRouter, LM Studio, Anthropic, OpenAI
- Streaming support
- TOML config

### 9. Storage (Same)

- Filesystem-based
- SharedConversation storage
- Session metadata
- Todo lists

## MVP Execution Flow (CORRECTED)

### Default: Dual-Agent Orchestration

```bash
# Start server
opencode-lab serve --port 3000

# Open browser to http://localhost:3000
# UI shows Dioxus dashboard

# Create new project via UI
# "Implement user authentication system"

# Orchestrator breaks into tasks:
# - Task 1: Implement JWT auth (dual-agent session)
# - Task 2: Add password hashing (dual-agent session)
# - Task 3: Create login endpoint (dual-agent session)

# For each task:
#   - Executor (build) implements
#   - Discriminator (discriminator-strict) reviews
#   - Todo list coordinates workflow
#   - UI shows both perspectives side-by-side

# User monitors via Dioxus UI
# User can intervene on reviews
# When all sessions complete, orchestrator summarizes
```

### Fallback: Single-Agent (Rare)

```bash
# Via API or UI, disable discriminator
POST /session/create
{
  "agent": "build",
  "task": "Read the README",
  "discriminator": null  // Explicitly disabled
}

# Runs as single session, no review workflow
# Used only for simple queries
```

### Also: Zed Integration (ACP)

```bash
# User configures Zed to use opencode-lab as agent server
# Zed sends requests via ACP
# opencode-lab executes dual-agent sessions
# Streams results back to Zed
# But Dioxus UI is still available for monitoring
```

## Success Criteria (REVISED)

### Phase 0: Core Types & DualAgent Runtime (4-6 weeks)
✅ Cargo workspace builds  
✅ SharedConversation storage works  
✅ Executor/Discriminator sessions created  
✅ Perspective transformation works  
✅ Tool rendering works  
✅ DualAgent loop executes  

**Demo:** Dual-agent session completes a task with feedback loop (logged to console)

### Phase 1: Shared Todo + Review Workflow (2-3 weeks)
✅ Shared todo list storage  
✅ Permission-based transitions  
✅ send_for_review tool  
✅ approve_review tool  
✅ task_done exits loop  

**Demo:** Executor sends for review, discriminator approves/rejects (logged)

### Phase 2: Tool System (3-4 weeks)
✅ All core tools work  
✅ Bash pattern matching  
✅ Permission enforcement  
✅ Tool context  

**Demo:** Dual-agent session edits files, runs tests, completes task

### Phase 3: HTTP Server + WebSocket (2 weeks)
✅ Axum server starts  
✅ HTTP API endpoints work  
✅ WebSocket broadcasts session updates  
✅ Can create/monitor sessions via API  

**Demo:** Create dual-agent session via API, watch live updates via WebSocket

### Phase 4: Dioxus Frontend (4-5 weeks)
✅ Dual-session dashboard view  
✅ Side-by-side executor/discriminator  
✅ Shared todo list display  
✅ Review approval interface  
✅ Session list view  
✅ Real-time updates via WebSocket  

**Demo:** Watch dual-agent session execute in browser UI, approve review via UI

### Phase 5: Orchestration (2-3 weeks)
✅ Orchestrator agent  
✅ Multi-session coordination  
✅ Task delegation  
✅ UI shows all sessions  

**Demo:** Orchestrator breaks project into 3 tasks, spawns 3 dual-agent sessions, monitors progress

### Phase 6: ACP Integration (1-2 weeks)
✅ ACP protocol server  
✅ Zed connects  
✅ Can use via Zed  

**Demo:** Execute dual-agent session from Zed, monitor in Dioxus UI

### MVP Complete (18-20 weeks)
✅ DualAgent is primary execution mode  
✅ Dioxus UI visualizes dual sessions  
✅ Todo list + review workflow works  
✅ Orchestrator coordinates multiple sessions  
✅ Can build real features end-to-end  
✅ ACP integration for Zed  

**Final Demo:** 
1. Give orchestrator a complex task ("Build a REST API with auth")
2. Orchestrator spawns 5 dual-agent sessions
3. Watch in Dioxus UI as executors implement, discriminators review
4. Approve/reject reviews via UI
5. All sessions complete, project is done
6. Works with local LLM (LM Studio)

## Out of Scope (Actual Post-MVP)

❌ Git automation (fork, clone, PR)  
❌ Web search integration  
❌ LSP integration  
❌ MyST-MD rendering (simple markdown is fine)  
❌ Langfuse/OTEL (basic logging only)  
❌ Advanced plugin system  
❌ Session compaction  
❌ Multi-project workspaces  

## Technical Stack (REVISED)

### Additional Crates for Dioxus
```toml
[workspace.dependencies]
# Everything from before, PLUS:

# Dioxus
dioxus = "0.6"
dioxus-router = "0.6"

# WebSocket
tokio-tungstenite = "0.21"

# Additional web stuff
axum-extra = "0.9"
```

### Project Structure (REVISED)
```
opencode-lab/
├── Cargo.toml
├── crates/
│   ├── opencode-lab/         # Main binary - SERVER not CLI
│   ├── core/                 # DualAgent-first types
│   ├── runtime/              # DualAgent runtime (primary)
│   ├── tools/                # All tools
│   ├── provider/             # LLM providers
│   ├── config/               # Config loading
│   ├── storage/              # Filesystem storage
│   ├── server/               # HTTP + WebSocket
│   ├── acp/                  # ACP protocol
│   └── frontend/             # ⭐ Dioxus UI (IN MVP)
└── .opencode-lab/
    └── agent/
        ├── build.toml              # HAS discriminator
        ├── supervisor.toml         # HAS discriminator
        ├── orchestrator.toml       # HAS discriminator
        └── discriminator-strict.toml
```

## What Makes This MVP Actually Correct?

### 1. **DualAgent is Default, Not Feature**
- Single-agent is the fallback/corner case
- Everything designed around executor/discriminator pattern
- No "adding dual-agent support" - it IS the core

### 2. **Orchestration Focus**
- Multi-session coordination from day 1
- Not a single-agent task executor
- Built for complex projects with multiple dual-agent pairs

### 3. **Dioxus UI is Essential**
- Can't orchestrate without visualization
- Need to see executor vs discriminator perspectives
- Review workflow requires UI for approval
- Real-time updates critical for monitoring

### 4. **Server-First Architecture**
- HTTP API + WebSocket, not CLI
- Dioxus frontend, not terminal
- ACP for Zed is bonus, not primary

## Timeline (REVISED)

**Week 1-6:** DualAgent Core + Shared Todo  
**Week 7-10:** Tool System  
**Week 11-12:** HTTP Server + WebSocket  
**Week 13-17:** Dioxus Frontend ⭐  
**Week 18-20:** Orchestration + Polish  

**Total: 20 weeks (5 months)**

## Summary

**MVP = Orchestrator with DualAgent + Dioxus UI**

NOT a CLI agent executor  
NOT OpenCode with better dual-agent  
IS an orchestration system with:
- DualAgent as the core pattern
- Reflection sessions with shared todos
- Review workflow built-in
- Dioxus web UI for visualization
- Multi-session coordination
- Server architecture, not terminal

**Success = Orchestrator coordinates 5 dual-agent sessions to build a complex feature, monitored in real-time via Dioxus UI, with local LLM**
