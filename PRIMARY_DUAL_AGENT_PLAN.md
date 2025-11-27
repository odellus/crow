# Primary Dual Agent Plan

**STATUS: IMPLEMENTED** ✅

## Overview

Implement dual-agent loop at the primary level (not subagent). User chats with Planner, Architect reviews and can call `task_complete`. User can interrupt and "be" the Architect.

## Usage

```bash
# Run in dual-agent mode
crow-cli chat --auto "Create a file hello.txt with content Hello World"

# The loop will:
# 1. Planner executes the task
# 2. Architect reviews (or user types to interrupt and become Architect)
# 3. If task_complete called → done
# 4. Otherwise, Planner gets feedback and continues
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  PRIMARY DUAL LOOP                                          │
│                                                             │
│  User Request (role: user)                                  │
│       │                                                     │
│       ▼                                                     │
│  ┌─────────────────┐                                        │
│  │ PLANNER         │  ← Does work, full ReACT loop          │
│  │ (builder - task)│  ← No task tool (no subagents)         │
│  └─────────────────┘                                        │
│       │                                                     │
│       │ response stored as role: assistant                  │
│       ▼                                                     │
│  [INTERRUPT CHECK] ──── User pressed key? ────┐             │
│       │                                       │             │
│       │ no                                    │ yes         │
│       ▼                                       ▼             │
│  ┌─────────────────┐                    ┌──────────────┐    │
│  │ ARCHITECT (AI)  │                    │ USER INPUT   │    │
│  │ has task_complete                    │ (as Architect)│   │
│  └─────────────────┘                    └──────────────┘    │
│       │                                       │             │
│       │ stored as role: user                  │             │
│       ├───────────────────────────────────────┘             │
│       │                                                     │
│       ├── task_complete? ──► DONE                           │
│       │                                                     │
│       ▼                                                     │
│  Back to PLANNER                                            │
└─────────────────────────────────────────────────────────────┘
```

## Session Structure

Single session. Messages alternate:
- `role: user` - Architect messages (AI or human interrupt)
- `role: assistant` - Planner messages (ReACT loop output)

```
Message History:
────────────────────────────────────────
user:      "Create a todo app"           ← Original request
assistant: [Planner turn 1]              ← Planner works
user:      [Architect feedback]          ← AI or human
assistant: [Planner turn 2]              ← Planner responds
user:      [Architect: task_complete]    ← Done
────────────────────────────────────────
```

## Agents

### Planner Agent
- Based on `builder` agent
- All standard tools (bash, read, write, edit, grep, glob, list, patch, etc.)
- NO `task` tool (no subagents for now)
- NO `task_complete` tool

### Architect Agent  
- Based on `builder` agent
- All standard tools
- NO `task` tool
- HAS `task_complete` tool
- System prompt includes: "You are reviewing another agent's work. Call task_complete when the task is fully done."

## Implementation Steps

### 1. Add agents to builtins.rs

```rust
// In agent/builtins.rs

pub fn planner_agent() -> AgentInfo {
    AgentInfo {
        name: "planner".to_string(),
        description: "Primary agent that executes tasks".to_string(),
        mode: AgentMode::Build,
        tools: standard_tools_without_task(),
        // ... 
    }
}

pub fn architect_agent() -> AgentInfo {
    AgentInfo {
        name: "architect".to_string(),
        description: "Reviews planner's work, calls task_complete when done".to_string(),
        mode: AgentMode::Build,  // or new AgentMode::Architect
        tools: standard_tools_without_task_plus_task_complete(),
        // ...
    }
}
```

### 2. Create task_complete tool (if not exists)

`tools/task_complete.rs` - signals completion of dual loop.

### 3. Add PrimaryDualRuntime

New file: `agent/primary_dual.rs`

```rust
pub struct PrimaryDualRuntime {
    session: Session,
    planner: AgentInfo,
    architect: AgentInfo,
    interrupt_rx: mpsc::Receiver<String>,  // For user interrupts
}

impl PrimaryDualRuntime {
    pub async fn run(&mut self, initial_message: String) -> Result<()> {
        // Store initial message as user
        self.session.add_user_message(&initial_message);
        
        loop {
            // 1. Run Planner turn
            let planner_response = execute_turn(&self.planner, &self.session).await?;
            self.session.add_assistant_message(planner_response);
            
            // 2. Check for interrupt
            let architect_message = if let Ok(user_input) = self.interrupt_rx.try_recv() {
                // User interrupted - their input becomes Architect message
                user_input
            } else {
                // Run AI Architect
                let response = execute_turn(&self.architect, &self.session).await?;
                
                // Check for task_complete
                if response.has_task_complete() {
                    return Ok(());
                }
                
                response.to_string()
            };
            
            // 3. Store Architect message as user (from Planner's POV)
            self.session.add_user_message(&architect_message);
        }
    }
}
```

### 4. CLI Integration

Add `--auto` flag to `crow-cli chat`:

```rust
#[derive(Parser)]
struct ChatArgs {
    message: String,
    #[arg(long)]
    auto: bool,  // Enable dual-agent auto mode
    #[arg(long)]
    session: Option<String>,
}
```

When `--auto`:
- Spawn interrupt listener thread (watches for key press)
- Use PrimaryDualRuntime instead of single-agent execute_turn
- On interrupt: pause, prompt for input, resume

### 5. Interrupt Mechanism (CLI)

```rust
// Spawn in background
let (interrupt_tx, interrupt_rx) = mpsc::channel();

thread::spawn(move || {
    // Listen for specific key (e.g., Escape or Ctrl+I)
    loop {
        if let Ok(key) = read_key() {
            if key == Key::Escape {
                // Signal interrupt, then read user input
                let input = prompt_user("Architect> ");
                interrupt_tx.send(input).ok();
            }
        }
    }
});
```

### 6. Backfill for Mid-Session Activation

When user enables auto mode mid-session:
- Existing messages don't change
- The framing just shifts:
  - Past `user` messages = "human was being Architect"
  - Past `assistant` messages = "Planner was responding"
- Next turn: AI Architect takes over (unless interrupted)

No actual data migration needed - it's a perspective shift.

## UI Display (CLI)

```
═══════════════════════════════════════════════════════════════
[PLANNER] Creating todo app...
  → write src/todo.rs
  → bash cargo build
  
[ARCHITECT] Reviewing...
  → read src/todo.rs
  → bash cargo test
  "Tests pass. But you forgot to add error handling for..."
  
[PLANNER] Adding error handling...
  → edit src/todo.rs
  
[ARCHITECT] ✓ task_complete
  "All requirements met. Tests pass."
═══════════════════════════════════════════════════════════════

# User interrupt would show:
[PLANNER] Creating todo app...
  → write src/todo.rs
  
[INTERRUPT] Press Enter to type as Architect, or wait for AI...
Architect> Actually, use a different file structure...

[PLANNER] Restructuring...
```

## Files Created/Modified

| File | Status | Description |
|------|--------|-------------|
| `agent/builtins.rs` | ✅ DONE | Added `planner` and `architect` agents with ARCHITECT_PROMPT |
| `agent/primary_dual.rs` | ✅ DONE | Created PrimaryDualRuntime with streaming events |
| `agent/mod.rs` | ✅ DONE | Exported primary_dual module and types |
| `tools/task_complete.rs` | ✅ EXISTS | Already existed from subagent dual work |
| `bin/crow-cli.rs` | ✅ DONE | Added `--auto` flag, `chat_dual_agent()`, `DualAgentRenderer` |

## Testing

1. Basic auto mode:
```bash
crow-cli chat --auto "Create a file hello.txt with 'Hello World'"
# Should see Planner create file, Architect verify, task_complete
```

2. Interrupt test:
```bash
crow-cli chat --auto "Build a complex feature"
# Press Escape during Planner turn
# Type custom Architect feedback
# Watch Planner respond to your feedback
```

3. Mid-session activation:
```bash
crow-cli chat "Start building something"  # Normal mode
crow-cli chat --session $SES --auto "Continue"  # Now in auto mode
```

## Future Work (Not Now)

- **Streaming dual agents** - Real-time streaming of both Planner and Architect output to UI
- **REPL interrupt-as-architect** - Port the interrupt handling to REPL mode where user can type during execution to become Architect
- Task tool re-enabled for subagent spawning
- Tauri UI with proper interrupt button
- Session infrastructure for agent↔agent conversations
- Configurable agent selection (not just planner/architect)
