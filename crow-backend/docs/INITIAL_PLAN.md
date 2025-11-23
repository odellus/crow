# OpenCode-Lab: Initial Implementation Plan

## Executive Summary

**OpenCode-Lab is a Rust rewrite of OpenCode with DualAgent as a first-class primitive.**

This is NOT a reimagination. This is NOT design theater. This IS:
- OpenCode's proven architecture, ported to Rust
- DualAgent pattern built into the core (not a hack)
- Orchestrator layer for multi-project coordination
- Built for long-running local LLMs
- Server-first with Dioxus frontend (no TUI)
- API-compatible with OpenCode
- ACP protocol support (Zed integration)

**Timeline: 3-4 months to MVP**

---

## What We're Building

### Core Concept

OpenCode works. It has great architecture. We're taking that and:

1. **Rewriting in Rust** - Performance, safety, single binary
2. **DualAgent first-class** - Every agent can have a discriminator (not just a hack function)
3. **MyST-MD rendering** - Structured text for tool output (reproducible, scientific)
4. **Orchestration layer** - Multi-project coordination, repo cloning, deep research
5. **Git automation** - Fork, clone, PR via Tangled/GitHub/GitLab
6. **Web search built-in** - SearxNG integration for research
7. **Dioxus frontend** - Render conversations, sessions, live updates

### What We're NOT Building

❌ Complete reimagination of agent systems  
❌ TUI interface (fuck that)  
❌ JupyterLab clone  
❌ Over-engineered directory structures  
❌ Novel research (we're implementing proven patterns)  

### What We're Mirroring from OpenCode

✅ Agent system (build, supervisor, architect, discriminator)  
✅ Tool system (bash, read, write, edit, grep, glob, task, etc.)  
✅ Session management (filesystem-based storage)  
✅ Message structure (MessageV2 with parts)  
✅ Permission system (per-agent, per-tool)  
✅ Config loading (opencode.jsonc hierarchy)  
✅ MCP integration (context servers)  
✅ ACP protocol (for Zed integration)  
✅ API routes (`/session/prompt`, `/session/create`, etc.)  

---

## The DualAgent Pattern (First-Class)

### Current State (OpenCode TypeScript)

```typescript
// dual-pair.ts - a hack function
export async function run(config: Config): Promise<Result> {
  // Hardcoded "build" and "discriminator" agents
  // Manual conversation inversion
  // Bolted on top of existing architecture
}
```

### Our State (OpenCode-Lab Rust)

```rust
// Every agent can optionally have a discriminator

pub struct Agent {
    pub name: String,
    pub tools: HashMap<String, bool>,
    pub permission: Permission,
    
    // NEW: First-class discriminator
    pub discriminator: Option<DiscriminatorConfig>,
}

pub struct DiscriminatorConfig {
    pub agent_name: String,      // Which agent acts as discriminator
    pub max_steps: usize,         // Max executor/discriminator rounds
    pub exit_condition: ExitCondition,
}

// Agent definition (TOML)
[discriminator]
agent = "discriminator-strict"
max_steps = 50
exit_condition = "task_done"
```

### How It Works

```
User/Orchestrator: "Implement authentication"
    ↓
DualAgent Runtime spawns:
    - Executor session (agent: build)
    - Discriminator session (agent: discriminator-strict)
    ↓
Loop (max 50 iterations):
    Executor Turn:
        - Execute tools (write, edit, bash, etc.)
        - No tool calls? → Hand off to discriminator
        
    Invert:
        - Render executor's tool calls as MyST-MD
        - Inject into discriminator session as user message
        
    Discriminator Turn:
        - Read executor's work
        - Run tests (bash, lsp_diagnostics)
        - Call task_done if good
        - Give feedback if issues
        
    Feedback Loop:
        - Discriminator feedback → Executor as user message
        - Executor refines based on feedback
    ↓
Exit when:
    - Discriminator calls task_done (success)
    - Max steps reached (timeout)
    ↓
Generate summary from discriminator's context
    (it has the best view - saw work + ran tests)
```

---

## Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────┐
│  User (Browser/CLI/Zed)                         │
└────────────┬────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────┐
│  OpenCode-Lab Server (Axum)                     │
│  ┌───────────────────────────────────────────┐ │
│  │  HTTP API  │  WebSocket  │  ACP Protocol  │ │
│  └───────────────────────────────────────────┘ │
│                                                  │
│  ┌───────────────────────────────────────────┐ │
│  │  DualAgent Runtime                        │ │
│  │  ├─ Executor Session                      │ │
│  │  └─ Discriminator Session                 │ │
│  └───────────────────────────────────────────┘ │
│                                                  │
│  ┌───────────────────────────────────────────┐ │
│  │  Orchestrator                             │ │
│  │  ├─ Multi-project coordination            │ │
│  │  ├─ Repo cloning (submodules)             │ │
│  │  └─ Task delegation                       │ │
│  └───────────────────────────────────────────┘ │
│                                                  │
│  ┌───────────────────────────────────────────┐ │
│  │  Tools                                    │ │
│  │  ├─ bash, read, write, edit               │ │
│  │  ├─ grep, glob, lsp                       │ │
│  │  ├─ task, task_done                       │ │
│  │  ├─ git (fork, clone, PR)                 │ │
│  │  └─ web_search (SearxNG)                  │ │
│  └───────────────────────────────────────────┘ │
│                                                  │
│  ┌───────────────────────────────────────────┐ │
│  │  Providers (LLM)                          │ │
│  │  ├─ Anthropic                             │ │
│  │  ├─ OpenAI                                │ │
│  │  ├─ LM Studio (local)                     │ │
│  │  └─ Ollama (local)                        │ │
│  └───────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────┐
│  Storage                                        │
│  ├─ Sessions (~/.opencode-lab/sessions/)        │
│  ├─ Training data (~/.opencode-lab/training/)   │
│  └─ Config (~/.opencode-lab/config.toml)        │
└─────────────────────────────────────────────────┘
```

### Project Structure

```
opencode-lab/
├── Cargo.toml                      # Workspace root
├── crates/
│   ├── opencode-lab/               # Main binary
│   │   └── src/
│   │       ├── main.rs             # CLI: serve, init, etc.
│   │       └── server.rs           # Axum server setup
│   │
│   ├── core/                       # Core types (mirrors OpenCode)
│   │   └── src/
│   │       ├── agent/
│   │       │   ├── agent.rs        # Agent definition
│   │       │   ├── registry.rs     # Agent registry
│   │       │   └── builtin.rs      # Built-in agents
│   │       ├── session/
│   │       │   ├── session.rs      # Session type
│   │       │   ├── message.rs      # MessageV2 equivalent
│   │       │   ├── storage.rs      # Session storage
│   │       │   └── dual_agent.rs   # ⭐ DualAgent runtime
│   │       ├── tool/
│   │       │   ├── tool.rs         # Tool trait
│   │       │   ├── registry.rs     # Tool registry
│   │       │   └── context.rs      # Execution context
│   │       ├── config/
│   │       │   ├── config.rs       # Config loading
│   │       │   └── permission.rs   # Permission system
│   │       └── project/
│   │           └── project.rs      # Project management
│   │
│   ├── runtime/                    # Execution runtime
│   │   └── src/
│   │       ├── prompt.rs           # SessionPrompt equivalent
│   │       ├── dual_agent.rs       # DualAgent execution
│   │       ├── single_agent.rs     # Normal ReACT loop
│   │       └── renderer.rs         # MyST-MD tool renderer
│   │
│   ├── tools/                      # Built-in tools
│   │   └── src/
│   │       ├── bash.rs
│   │       ├── read.rs
│   │       ├── write.rs
│   │       ├── edit.rs
│   │       ├── grep.rs
│   │       ├── glob.rs
│   │       ├── lsp/
│   │       │   ├── mod.rs
│   │       │   └── diagnostics.rs
│   │       ├── task.rs             # Task delegation
│   │       ├── task_done.rs        # Discriminator exit
│   │       ├── todo.rs             # Todo management
│   │       ├── git.rs              # Git operations
│   │       └── web_search.rs       # SearxNG integration
│   │
│   ├── provider/                   # LLM providers
│   │   └── src/
│   │       ├── provider.rs         # Provider trait
│   │       ├── anthropic.rs
│   │       ├── openai.rs
│   │       ├── lmstudio.rs
│   │       └── ollama.rs
│   │
│   ├── server/                     # HTTP server
│   │   └── src/
│   │       ├── api/                # REST API routes
│   │       │   ├── session.rs
│   │       │   ├── message.rs
│   │       │   ├── dual_agent.rs
│   │       │   └── orchestrator.rs
│   │       └── ws/                 # WebSocket
│   │           └── stream.rs
│   │
│   ├── acp/                        # ACP protocol server
│   │   └── src/
│   │       ├── server.rs
│   │       └── protocol.rs
│   │
│   ├── git/                        # Git automation
│   │   └── src/
│   │       ├── tangled.rs          # Tangled.sh API
│   │       ├── github.rs           # GitHub API
│   │       ├── gitlab.rs           # GitLab API
│   │       └── operations.rs       # Git ops (clone, fork, PR)
│   │
│   └── frontend/                   # Dioxus web UI
│       └── src/
│           ├── app.rs
│           ├── views/
│           │   ├── sessions.rs
│           │   ├── agents.rs
│           │   └── orchestrator.rs
│           └── components/
│               ├── session_viewer.rs
│               └── myst_renderer.rs
│
├── agents/                         # Agent definitions (TOML)
│   ├── build.toml
│   ├── discriminator-strict.toml
│   ├── discriminator-balanced.toml
│   ├── discriminator-permissive.toml
│   ├── supervisor.toml
│   ├── orchestrator.toml
│   └── architect.toml
│
├── docker/
│   └── searxng/                    # SearxNG Docker Compose
│       ├── docker-compose.yml
│       └── settings.yml
│
└── README.md
```

---

## Implementation Phases

### Phase 0: Core Foundation (4-6 weeks)

**Goal:** Basic Rust OpenCode that works

**Tasks:**
1. ✅ Create Cargo workspace
2. ✅ Implement core types:
   - `Agent` struct with discriminator field
   - `Session` type (Primary, Executor, Discriminator)
   - `Message` (MessageV2 equivalent with parts)
   - `Tool` trait and context
3. ✅ Config system:
   - TOML loading (opencode.jsonc → opencode.toml)
   - Permission merging
   - Agent registry
4. ✅ Basic tools:
   - bash, read, write, edit
   - grep, glob
5. ✅ LLM provider (Anthropic first)
6. ✅ Session storage (filesystem)
7. ✅ Single-agent ReACT loop

**Deliverable:**
```bash
cargo run --bin opencode-lab -- serve -p 4096

# Test single-agent execution
curl -X POST http://localhost:4096/session/prompt \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "build",
    "parts": [{"type": "text", "text": "write hello world in rust"}]
  }'
```

**Reference:**
- OpenCode TypeScript: `/packages/opencode/src/`
- Zed Rust: `/crates/assistant/src/` (when stuck)

### Phase 1: DualAgent Runtime (3-4 weeks)

**Goal:** DualAgent pattern working end-to-end

**Tasks:**
1. ✅ `DualAgentSession` type
2. ✅ Executor turn logic:
   - Execute tools until no tool calls
   - Track all tool calls/results
3. ✅ MyST-MD renderer:
   - Convert tool calls to MyST-MD directives
   - Structured output (not just markdown)
4. ✅ Conversation inversion:
   - Render executor work for discriminator
   - Inject as user message
5. ✅ Discriminator turn logic:
   - Read executor work
   - Run verification tools
   - Call `task_done` or give feedback
6. ✅ Feedback loop:
   - Discriminator feedback → executor user message
7. ✅ Summary generation:
   - Use discriminator's context
   - Structured output

**Deliverable:**
```bash
# Test dual-agent execution
curl -X POST http://localhost:4096/dual-agent/run \
  -H "Content-Type: application/json" \
  -d '{
    "task": "implement fibonacci function with tests in rust",
    "max_steps": 20
  }'

# Returns:
{
  "completed": true,
  "steps": 3,
  "executor_session_id": "session-abc123",
  "discriminator_session_id": "session-def456",
  "summary": {
    "text": "Implemented fibonacci function with comprehensive tests...",
    "files_modified": [
      {"path": "src/fibonacci.rs", "operations": ["created"]},
      {"path": "tests/fibonacci_test.rs", "operations": ["created"]}
    ],
    "test_results": {
      "passed": 5,
      "failed": 0
    }
  }
}
```

**Reference:**
- OpenCode: `/packages/opencode/src/session/dual-pair.ts`
- DUAL_AGENT_CORE.md (our design doc)

### Phase 2: Orchestrator Agent (2-3 weeks)

**Goal:** Multi-project coordination

**Tasks:**
1. ✅ Orchestrator agent definition
2. ✅ Multi-repo operations:
   - Clone repos as submodules
   - Add to project context
3. ✅ Task delegation:
   - Break complex tasks into supervisor tasks
   - Track task dependencies
4. ✅ Todo-driven planning:
   - Orchestrator ALWAYS uses todowrite
   - Sticks to plan
5. ✅ Summary-based message passing:
   - Markdown with pointers to filesystem
   - Stateless communication

**Deliverable:**
```bash
# Orchestrator coordinates multi-repo project
curl -X POST http://localhost:4096/orchestrator/start \
  -H "Content-Type: application/json" \
  -d '{
    "task": "Research and implement quantum computing library",
    "repos_to_clone": [
      "github.com/qiskit/qiskit",
      "github.com/quantumlib/cirq"
    ]
  }'

# Orchestrator:
# 1. Clones repos into examples/
# 2. Creates todo list
# 3. Delegates to supervisor agents
# 4. Coordinates completion
```

**Reference:**
- OpenCode: `/packages/opencode/src/agent/supervisor.txt`
- AGENTS.md (our orchestration design)

### Phase 3: Git Automation (2-3 weeks)

**Goal:** Fork, clone, PR automation

**Tasks:**
1. ✅ GitHub API client:
   - Fork repos
   - Create PRs
   - Clone repos
2. ✅ GitLab API client (same operations)
3. ✅ Tangled.sh API client
4. ✅ Git tool:
   - `git_fork`, `git_clone`, `git_pr`
   - Available to orchestrator/supervisor
5. ✅ Submodule management

**Deliverable:**
```bash
# Agent automatically forks and clones
curl -X POST http://localhost:4096/git/fork \
  -H "Content-Type: application/json" \
  -d '{
    "repo": "github.com/some/library",
    "local_path": "dependencies/library"
  }'

# Creates fork at github.com/your-username/library
# Clones to project directory
# Returns fork URL and local path
```

**Reference:**
- OpenCode: `/packages/opencode/src/git/` (if exists)
- Zed: `/crates/project/src/` (git integration)

### Phase 4: Web Search Integration (1-2 weeks)

**Goal:** SearxNG for research

**Tasks:**
1. ✅ SearxNG Docker Compose setup
2. ✅ SearxNG HTTP client (Rust)
3. ✅ Web search tool:
   - General web search
   - ArXiv search
   - GitHub code search
4. ✅ MCP server wrapper (optional)

**Deliverable:**
```bash
# Start SearxNG
docker-compose -f docker/searxng/docker-compose.yml up -d

# Agent uses web search
curl -X POST http://localhost:4096/session/prompt \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "architect",
    "parts": [{"type": "text", "text": "research rust async runtimes"}]
  }'

# Architect searches web, arxiv, github
# Returns summary with sources
```

**Reference:**
- Your Zed config: `~/.config/zed/settings.json` (context_servers.searxng)

### Phase 5: Dioxus Frontend (3-4 weeks)

**Goal:** Web UI for viewing sessions

**Tasks:**
1. ✅ Dioxus app setup
2. ✅ Views:
   - Session list
   - Session detail (executor + discriminator)
   - Orchestrator dashboard
3. ✅ MyST-MD renderer component
4. ✅ Live updates (WebSocket)
5. ✅ Agent control:
   - Start/stop agents
   - View progress

**Deliverable:**
```bash
opencode-lab serve
# Opens http://localhost:3000

# UI shows:
# - Running agents
# - Executor/discriminator conversations
# - Todo lists
# - Session history
```

**Reference:**
- Zed: `/crates/assistant/src/` (UI patterns)
- Dioxus docs: https://dioxuslabs.com/

### Phase 6: ACP Server (1-2 weeks)

**Goal:** Zed integration

**Tasks:**
1. ✅ ACP protocol implementation
2. ✅ stdio server (like OpenCode)
3. ✅ Tool execution via ACP
4. ✅ Compatible with Zed agent_servers

**Deliverable:**
```bash
# In Zed settings.json
{
  "agent_servers": {
    "OpenCodeLab": {
      "command": "opencode-lab",
      "args": ["acp"]
    }
  }
}

# Zed can now use OpenCode-Lab as agent backend
```

**Reference:**
- OpenCode: `/packages/opencode/src/acp/`
- Zed: `/crates/agent/src/` (ACP client)

---

## Key Design Decisions

### 1. MyST-MD Tool Rendering

**Why:** Structured text for reproducibility

**Example:**
```markdown
## Executor Turn 1

Created file `src/lib.rs`:

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Ran tests:

```{bash-output}
$ cargo test
running 1 test
test tests::test_add ... ok

test result: ok. 1 passed; 0 failed
```

```{tool-call}
:tool: write
:path: src/lib.rs
:status: success
```
```

### 2. Stateless Message Passing

**Why:** RST-style, filesystem as database

**Orchestrator → Supervisor:**
```markdown
# Task: Implement Authentication

See project structure in `PROJECT_STRUCTURE.md`

Dependencies cloned to:
- `examples/oauth2-rs/` - OAuth2 library reference
- `examples/actix-identity/` - Session management example

Your tasks (see `TODO.md`):
1. Implement user model
2. Add password hashing
3. Create login endpoint
4. Write tests

Report back via `PROGRESS.md`
```

### 3. Todo-Driven Planning

**Why:** Agents stay on track

**Every orchestrator/supervisor:**
```rust
// First action: Create todos
todowrite([
    "Search arxiv for quantum computing papers",
    "Clone top 3 repos as examples",
    "Create project structure",
    "Implement core algorithm"
]);

// Track progress
todoread(); // Check what's done
// Update todos as needed
```

### 4. Submodules First-Class

**Why:** Examples raise the ocean

**Orchestrator workflow:**
```bash
# 1. Research phase
web_search("rust quantum computing libraries")
  → Finds: qiskit, cirq, forest-sdk

# 2. Clone as examples
git_clone("github.com/qiskit/qiskit", "examples/qiskit")
git_clone("github.com/rigetti/forest-sdk", "examples/forest-sdk")

# 3. Now executor has full context
# Can read examples/, learn patterns
# Implements with reference to examples
```

### 5. Long-Running Sessions

**Why:** Local LLMs, complex tasks

**Design:**
- No timeouts on sessions
- Checkpoint/resume capability
- WebSocket for live updates
- Pause/cancel via API

```bash
# Start long task
POST /dual-agent/run
  → Returns session_id immediately

# Stream progress
WS /session/{id}/stream
  → Live updates of executor/discriminator conversation

# Pause
POST /session/{id}/pause

# Resume
POST /session/{id}/resume
```

---

## When Stuck: Reference Guide

### Mirror OpenCode Architecture?
**→ Read:** `/packages/opencode/src/` (TypeScript)
**→ Focus on:** Concepts, not syntax

### Implement in Rust?
**→ Read:** Zed codebase (similar patterns)
**→ `/crates/assistant/src/` for agent patterns
**→ `/crates/rpc/src/` for protocol implementations

### Validate Approach?
**→ Read:** DUAL_AGENT_CORE.md (our design)
**→ Read:** RESEARCH_SUMMARY.md (academic validation)

### Agent Orchestration?
**→ Read:** AGENTS.md (our orchestration design)
**→ Papers:** Multi-Agent Collaboration via Evolving Orchestration

### Prompt Optimization?
**→ Read:** RESEARCH_SUMMARY.md (GEPA section)
**→ Later phase, not MVP

---

## MVP Scope (3-4 Months)

### In Scope

✅ Rust rewrite of OpenCode core  
✅ DualAgent pattern built-in  
✅ Basic orchestrator (repo cloning, task delegation)  
✅ MyST-MD rendering  
✅ Essential tools (bash, read, write, edit, grep, glob, git, web_search)  
✅ Dioxus frontend (session viewer)  
✅ ACP server (Zed integration)  
✅ SearxNG integration (existing Docker)  
✅ GitHub API (fork, clone, PR)  

### Out of Scope (Post-MVP)

❌ GEPA prompt optimization (collect data first)  
❌ Tangled.sh integration (GitHub sufficient for MVP)  
❌ GitLab API (GitHub first)  
❌ Advanced UI features (agent control dashboard)  
❌ SLURM/network compute  
❌ Rust SearxNG port (use existing)  
❌ Training pipeline (need data first)  
❌ Multiple discriminator profiles (one strict is enough)  

---

## Success Criteria

### Phase 0 Success
```bash
# Single-agent execution works
opencode-lab serve -p 4096
curl -X POST http://localhost:4096/session/prompt \
  -d '{"agent":"build","parts":[{"type":"text","text":"hello world"}]}'
# → Returns valid response
```

### Phase 1 Success
```bash
# Dual-agent execution works
curl -X POST http://localhost:4096/dual-agent/run \
  -d '{"task":"implement fibonacci with tests","max_steps":20}'
# → Executor writes code
# → Discriminator verifies with tests
# → Returns success with summary
```

### Phase 2 Success
```bash
# Orchestrator delegates tasks
curl -X POST http://localhost:4096/orchestrator/start \
  -d '{"task":"build web app","repos_to_clone":["actix-web"]}'
# → Creates project structure
# → Clones examples
# → Delegates to supervisors
# → Supervisors delegate to workers
# → Returns completion summary
```

### MVP Success
```bash
# Full workflow works
opencode-lab serve
# → Opens browser to http://localhost:3000
# → Shows orchestrator dashboard
# → Click "Start Research Project"
# → Orchestrator searches web, clones repos
# → Creates plan, assigns tasks
# → Workers execute with dual-agent
# → UI shows live progress
# → All sessions viewable
# → Zed integration works
```

---

## Development Workflow

### Daily
```bash
# 1. Pull latest
git pull

# 2. Run tests
cargo test --all

# 3. Work on current phase
# See GitHub Projects board for tasks

# 4. Commit frequently
git commit -m "feat(core): implement dual-agent session type"

# 5. Push at end of day
git push
```

### Weekly
```bash
# 1. Review phase progress
# Are we on track for 3-4 month MVP?

# 2. Update GitHub Projects
# Move completed tasks, add blockers

# 3. Demo progress
# Show working features

# 4. Adjust plan if needed
# Are we blocked? Need to defer features?
```

### Per Phase
```bash
# Phase complete when:
# 1. All deliverables working
# 2. Tests passing
# 3. Documentation updated
# 4. Next phase can start

# Tag releases
git tag -a v0.1.0-phase0 -m "Phase 0: Core Foundation"
git push --tags
```

---

## Dependencies

```toml
# Cargo.toml (workspace)
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"

# HTTP server
axum = "0.7"
tower = "0.5"
tower-http = { version = "0.5", features = ["cors", "trace", "fs"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# LLM providers
reqwest = { version = "0.12", features = ["json", "stream"] }
eventsource-stream = "0.2"  # SSE for streaming

# Tools
ignore = "0.4"              # .gitignore handling
grep-searcher = "0.1"       # ripgrep library
skim = "0.10"               # fuzzy finding

# Git
git2 = "0.19"               # libgit2 bindings
octocrab = "0.40"           # GitHub API

# LSP
tower-lsp = "0.20"          # LSP client

# Storage
sled = "0.34"               # Embedded database (optional)

# CLI
clap = { version = "4", features = ["derive"] }
console = "0.15"

# Frontend
dioxus = "0.6"
dioxus-router = "0.6"

# Utils
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1", features = ["v7"] }
```

---

## Next Steps

### Immediate (This Week)
1. ✅ Read this plan thoroughly
2. ✅ Set up Cargo workspace structure
3. ✅ Create basic crate structure (core, runtime, tools, etc.)
4. ✅ Start Phase 0: Core types

### Short Term (Month 1)
1. ✅ Complete Phase 0 (Core Foundation)
2. ✅ Start Phase 1 (DualAgent Runtime)
3. ✅ Test dual-agent with simple tasks

### Medium Term (Months 2-3)
1. ✅ Complete Phase 1-4
2. ✅ Start Dioxus frontend
3. ✅ ACP integration
4. ✅ MVP testing

### Long Term (Month 4+)
1. ✅ MVP complete, production-ready
2. ✅ Start data collection
3. ✅ Begin GEPA optimization
4. ✅ Post-MVP features

---

## Compact Session Starter

**For next session, start with:**

```
Here's the plan for OpenCode-Lab:

A Rust rewrite of OpenCode with DualAgent as a first-class primitive.

Key points:
1. Mirror OpenCode's architecture (proven design)
2. DualAgent built into core (not a hack)
3. Orchestrator layer for multi-project coordination
4. MyST-MD rendering for structured output
5. Dioxus frontend (no TUI)
6. 3-4 month MVP timeline

See INITIAL_PLAN.md for complete details.

Current phase: Phase 0 - Core Foundation
Next task: Set up Cargo workspace and implement core types

References:
- OpenCode TypeScript: /packages/opencode/src/
- Zed Rust: When stuck on implementation
- DUAL_AGENT_CORE.md: Pattern design
- RESEARCH_SUMMARY.md: Academic validation
- AGENTS.md: Orchestration strategy

Let's build this properly. Start with Cargo workspace setup?
```

---

## The Vision

**3 months from now:**

```bash
# User installs
curl https://opencode-lab.dev/install.sh | sh

# Starts project
cd ~/projects/quantum-research
opencode-lab init

# Starts server
opencode-lab serve
# → Opens http://localhost:3000

# Orchestrator agent:
# 1. Searches arxiv for quantum papers
# 2. Clones top 3 repos as examples
# 3. Creates project plan (todos)
# 4. Delegates to supervisor agents
# 5. Supervisors delegate to worker agents
# 6. Workers use dual-agent (executor/discriminator)
# 7. Code written, tests run, verification done
# 8. All sessions viewable in UI
# 9. Zed integration for interactive coding

# Human reviews:
# - Session transcripts (MyST-MD)
# - Dual-agent conversations
# - Final code + tests

# Human feedback improves discriminator
# GEPA optimizes prompts
# System gets better over time
```

**This is the future of AI-assisted development.**

**Let's fucking build it.** 🚀
