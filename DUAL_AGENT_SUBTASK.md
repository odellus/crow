# Dual Agent Architecture: Executor-Discriminator Pattern

## Overview

Two agents sharing one session, with inverted perspectives. One builds, one validates. They take turns in alternating ReACT loops until the discriminator calls `work_completed`.

```
┌─────────────────────────────────────────────────────────────┐
│                      SHARED SESSION                         │
│                   (single todo state)                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────────┐                    ┌─────────────────┐   │
│   │  EXECUTOR   │  ←── inverted ───→ │  DISCRIMINATOR  │   │
│   │  (builder)  │      messages      │   (validator)   │   │
│   └─────────────┘                    └─────────────────┘   │
│         │                                    │              │
│   qwen3-coder-30B                    qwen3-VL-30B          │
│   (code generation)                  (vision + review)     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Core Principles

### 1. Shared Session, Inverted Perspectives

Both agents see the same conversation but with inverted roles:
- What Executor sees as its own assistant messages → Discriminator sees as user messages
- What Discriminator sees as its own assistant messages → Executor sees as user messages
- Tool calls are rendered to markdown so the other agent can read them as natural text

### 2. Shared Todo State (Critical)

This is the most important constraint. Both agents MUST operate on the exact same todo list:
- Executor works through tasks
- Discriminator validates completion, can reject/reopen tasks
- Todo state is the ground truth for progress
- No separate todo lists, no divergence

### 3. No Truncation

Tool responses must be complete:
- No "+22 others" bullshit
- No summarization of outputs
- Full tool call inputs and outputs
- This is how agents maintain shared context

### 4. Alternating ReACT Loops

```
1. Executor does full ReACT loop (tools until no more tool calls)
2. Executor's turn rendered as "user message" to Discriminator
3. Discriminator does full ReACT loop
4. Discriminator's turn rendered as "user message" to Executor
5. Repeat until Discriminator calls work_completed
```

## Agent Configurations

### Executor (Build Agent)
- **Model**: qwen3-coder-30B (via llama.cpp)
- **Role**: Implementation, code generation, file operations
- **Tools**: bash, edit, read, write, grep, glob, todoread, todowrite
- **Behavior**: 
  - Follows todo list strictly
  - Updates docs after completing tasks
  - Creates pre/post task planning docs
  - Uses response as pointer to filesystem state

### Discriminator (Plan/Review Agent)
- **Model**: qwen3-VL-30B (via llama.cpp) - HAS VISION
- **Role**: Validation, code review, UI verification, dense reward signal
- **Tools**: bash, read, grep, glob, todoread, todowrite, **screenshot**, **click/keyboard**
- **Behavior**:
  - Reviews executor's work
  - Can reject and reopen tasks
  - Provides corrective feedback (dense reward)
  - Catches reward hacking / going off rails
  - Calls work_completed when satisfied
  - Can visually verify UI with screenshots

## Message Inversion

The key transformation for passing messages between agents:

```rust
// When rendering Executor's turn for Discriminator:
fn invert_for_discriminator(executor_messages: Vec<Message>) -> Vec<Message> {
    // Executor's assistant messages → role: user (markdown-rendered tool calls)
    // Executor's tool results → included in the user message as markdown
}

// When rendering Discriminator's turn for Executor:
fn invert_for_executor(discriminator_messages: Vec<Message>) -> Vec<Message> {
    // Same inversion logic
}
```

Tool calls rendered as markdown:
```markdown
**Tool Call: edit**
- file: src/main.rs
- changes: [full diff here]

**Result:**
[full output, no truncation]
```

## Session Schema Changes

Current session assumes single agent. Need to support:

```rust
struct DualSession {
    id: String,
    project_id: String,
    
    // Shared state
    todos: Vec<Todo>,
    
    // Separate message streams that reference same underlying data
    executor_perspective: Vec<MessageRef>,
    discriminator_perspective: Vec<MessageRef>,
    
    // Track whose turn it is
    current_agent: AgentRole, // Executor | Discriminator
    
    // Completion state
    completed: bool,
    completed_by: Option<AgentRole>,
}
```

## Task Input Schema

Tasks should be structured, not vague:

```rust
struct DualAgentTask {
    // Required
    todo_list: Vec<Todo>,           // Pre-defined tasks
    explainer: String,              // Short description
    
    // Context pointers
    context_files: Vec<PathBuf>,    // Files with more info
    
    // Limits
    max_steps: Option<u32>,         // Prevent runaway
    max_turns: Option<u32>,         // Max executor-discriminator exchanges
}
```

## Provider Configuration

Two llama.cpp instances on different ports:

```json
{
  "executor": {
    "api_url": "http://192.168.1.175:1234/v1",
    "model": "qwen3-coder-30b-a3b-instruct",
    "max_tokens": 262144
  },
  "discriminator": {
    "api_url": "http://192.168.1.175:1235/v1",
    "model": "qwen3-VL-30B-A3B-Instruct",
    "max_tokens": 262144,
    "capabilities": {
      "vision": true
    }
  }
}
```

## New Tools Needed

### For Discriminator (Vision Agent)

```rust
// Screenshot tool - capture current screen/window
struct ScreenshotTool;
// Returns: base64 image that vision model can process

// UI interaction tools (like Playwright but for desktop)
struct ClickTool;      // click at coordinates
struct TypeTextTool;   // type text / send keys
struct ScrollTool;     // scroll in window
```

### Shared

```rust
// Modified work_completed - only Discriminator can call this
struct WorkCompletedTool {
    // Only callable by discriminator role
    // Signals task completion
    // Includes summary of what was accomplished
}
```

## Implementation Steps

### Phase 1: Message Inversion
- [ ] Remove all truncation from tool output rendering
- [ ] Create `render_tool_call_as_markdown()` function
- [ ] Create `render_tool_result_as_markdown()` function
- [ ] Test: same tool call renders identically from both perspectives

### Phase 2: Dual Session
- [ ] Modify session schema to support dual agents
- [ ] Implement perspective switching (who sees what as user/assistant)
- [ ] Shared todo state with atomic updates
- [ ] Turn tracking (whose turn is it)

### Phase 3: Alternating Loop
- [ ] Modify executor to support "run until no tool calls, then yield"
- [ ] Implement turn handoff between agents
- [ ] work_completed only callable by discriminator
- [ ] Add max_turns limit

### Phase 4: Vision Tools
- [ ] Screenshot capture tool
- [ ] Basic click/type tools for UI testing
- [ ] Test with qwen3-VL model

### Phase 5: Provider Wiring
- [ ] Configure dual llama.cpp providers
- [ ] Test both models independently
- [ ] Test alternating loop with both

## Open Questions

1. **Subagent sessions**: Keep separate? Or same dual pattern?
2. **Error handling**: What if one agent crashes mid-turn?
3. **Context limits**: How to handle when combined context exceeds limits?
4. **Persistence**: Save after each turn? Or only on completion?

## Why This Architecture?

- **Dense reward signal**: Discriminator catches mistakes early, provides immediate feedback
- **Reward hacking prevention**: Builder can't mark its own homework as done
- **Vision verification**: Can actually SEE if UI looks right, not just check code
- **Shared state**: Todo list as single source of truth prevents divergence
- **Senior/junior dynamic**: Discriminator is the senior reviewer, Executor does the work
