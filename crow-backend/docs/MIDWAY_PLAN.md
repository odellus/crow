# Crow Midway Plan - From Chaos to Clarity

## Where We Started

We began with ambitions to build a sophisticated dual-agent system with a custom architecture. We explored bionic-gpt, researched different approaches, and tried to be clever. We wanted to be different, to innovate.

Then we hit TypeScript. And something clicked.

## The Revelation

**OpenCode is fucking beautiful.** Their architecture is clean, their API is well-designed, and their agent system just works. When we dove into their codebase, we realized:

1. They've already solved the hard problems
2. Their patterns are elegant and testable
3. The API-first design makes everything modular
4. The TUI is just one consumer of a solid backend

So we made a decision: **Stop being clever. Start being smart. Clone it in Rust.**

## What We've Accomplished So Far

### Core Infrastructure ✅
- **REST API Server** - Complete OpenCode-compatible endpoints
- **Session Management** - Sessions, messages, parts, full storage
- **Agent System** - AgentRegistry, AgentExecutor with ReACT loop
- **Tool System** - 11 tools with ToolContext for execution awareness

### Critical Features ✅
- **Doom Loop Detection** - Prevents infinite tool call cycles (3x threshold)
- **Tool Context** - Tools know their session_id, message_id, agent, working_dir
- **Task Tool** - Full subagent spawning with parent-child session relationships
- **Cost Tracking** - Kimi K2 pricing ($0.15/M input, $2.50/M output)
- **Agent Prompts** - Exact copies from OpenCode (Supervisor, Architect)

### What Actually Works Right Now
```bash
# Create session
curl -X POST http://localhost:7070/session -d '{"directory": "/some/path"}'

# Send message, agent executes tools
curl -X POST http://localhost:7070/session/ses-123/message \
  -d '{"agent": "build", "parts": [{"type": "text", "text": "list files"}]}'

# Agent spawns subagent via Task tool
# Parent and child sessions both track tokens/cost independently
# Child session has parent_id set correctly
```

## Implementation Philosophy: The Carbon Copy Approach

### Why We're Cloning OpenCode Exactly

1. **They figured it out** - Don't reinvent solved problems
2. **Rust benefits** - Get their architecture + type safety, performance, memory safety
3. **Future refactor** - Get it working first, optimize later
4. **Learn by doing** - Understanding comes from implementation

### Our Guiding Principles

**DO:**
- Copy OpenCode's architecture exactly
- Match their API responses precisely
- Use their agent prompts verbatim
- Test against actual OpenCode behavior
- Build the TUI last (Dioxus web UI, not terminal)

**DON'T:**
- Try to be clever with "improvements"
- Deviate from their patterns
- Skip features thinking we won't need them
- Worry about perfection - iterate later

## What We've Learned

### Technical Insights

1. **Circular Dependencies Are Real** - TaskTool needs ToolRegistry, ToolRegistry creates TaskTool
   - Solution: `Arc<RwLock<Option<Arc<ToolRegistry>>>>`
   - Works, not pretty, we'll refactor later

2. **Tool Context Matters** - Tools need to know where they're executing
   - session_id, message_id, agent, working_dir
   - Critical for Task tool to spawn children in correct location

3. **ReACT Loop Is The Heart** - Everything flows through executor.rs
   - Build system prompt → Call LLM → Parse tool calls → Execute → Loop
   - Track tokens/cost across all iterations

4. **OpenCode's Type System Maps Well** - Their TypeScript types → Rust structs cleanly
   - Session, Message, Part, Tool, Agent all translate nicely
   - Serde makes JSON compatibility easy

### Architectural Insights

1. **API-First Is Correct** - Everything is an HTTP endpoint
   - Makes testing trivial
   - UI becomes just another client
   - Can swap implementations easily

2. **Session Model Is Elegant** - Sessions contain messages, messages contain parts
   - Parent-child relationships for subagents
   - Storage is simple key-value with session_id

3. **Agent Registry Pattern Works** - Agents defined declaratively
   - Name, prompt, tools, permissions
   - Easy to add new agents
   - Built-in agents vs. custom agents

## The Directory/Working Directory Question

### Current State of Confusion

- **OpenCode**: `cd /your/project && opencode` - runs in that directory, uses it for context
- **Crow (current)**: Server runs in crow/, uses that as base directory
- **Problem**: Sessions all show crow's directory, not the working directory we want

### The Two Paths Forward

#### Option 1: Match OpenCode Exactly (RECOMMENDED)
```bash
# User does:
cd /home/thomas/my-project
crow serve

# Crow server:
# - Runs in /home/thomas/my-project (cwd)
# - Default session directory: /home/thomas/my-project
# - Can still create sessions in other directories via API
# - Just like OpenCode does it
```

**Pros:**
- Exact parity with OpenCode
- User mental model matches
- Simple to reason about
- Easy to test against OpenCode

**Cons:**
- Need one server per project (or handle multi-project differently)

#### Option 2: Centralized Server
```bash
# Crow always runs from /home/thomas/crow
# But tracks working_dir per session properly
# Sessions can be in any directory
```

**Pros:**
- One server for all projects
- More "server-like"

**Cons:**
- Different from OpenCode
- Harder to compare behavior
- What's the "default" directory?

### Decision: Go With Option 1

**Match OpenCode exactly.** Here's why:

1. We're doing a carbon copy - this is part of the copy
2. User expectation is `cd project && crow serve`
3. Makes testing against OpenCode trivial
4. We can change later if we want
5. The Dioxus web UI will make multi-project management easy anyway

### What This Means Practically

- User runs `crow serve` from their project directory
- That becomes the default session directory
- API still allows creating sessions in other directories
- Session.directory field is meaningful and correct
- Tools execute in the right context

## Current Gaps vs. OpenCode Parity

### What We Have
- ✅ Core agent execution (ReACT loop)
- ✅ Tool system with 11 tools
- ✅ Task tool for subagents
- ✅ Doom loop detection
- ✅ Cost tracking
- ✅ Session storage
- ✅ Basic agents (build, supervisor, architect)

### What We're Missing

#### 1. Agent Completeness
**OpenCode has:**
- build, supervisor, architect, discriminator, plan, explore
- Each with specific tool access patterns
- Specific delegation rules (supervisor → build only, architect → supervisor/orchestrator)

**We have:**
- build, supervisor, architect, discriminator (partial)
- Need: plan, explore agents
- Need: Proper tool filtering per agent
- Need: Test delegation rules

#### 2. Tool Completeness
**OpenCode has ~20 tools:**
- bash, edit, write, read, glob, grep, list (we have these)
- task (we have this)
- WebFetch, WebSearch, Skill, SlashCommand
- NotebookEdit, TodoRead, TodoWrite (we have TodoRead/Write)
- ExitPlanMode, AskUserQuestion
- BashOutput, KillShell (for background processes)

**Missing critical tools:**
- Background bash execution (BashOutput, KillShell)
- Web tools (WebFetch, WebSearch)
- Plan mode tools (ExitPlanMode)

#### 3. System Prompt Building
**Need to verify:**
- Environment context (git status, cwd, platform)
- Agent-specific reminders
- Tool descriptions in prompt
- Model-specific formatting

#### 4. Permission System
**OpenCode has:**
- bash permission patterns (allow/deny/ask)
- edit permission levels
- Tool filtering per agent

**We have:**
- Basic tool filtering (is_tool_enabled)
- Bash pattern matching (partially implemented)
- Need: Full permission checking
- Need: Interactive permission requests (for later, skip for HOTL)

#### 5. Streaming & Real-time Updates
**OpenCode has:**
- SSE streaming for tool execution
- Real-time part updates
- Progress indicators

**We have:**
- Basic streaming endpoint (not fully wired)
- Need: Tool execution streaming
- Need: Part state updates

#### 6. Dual-Agent System
**OpenCode has:**
- Discriminator agent reviews executor work
- Shared conversation context
- Verdict system (approved/rejected)

**We have:**
- Basic dual-agent structure
- Need: Full discriminator logic
- Need: Test discriminator behavior

## The Plan Forward

### Phase 1: Achieve Core Parity (NEXT)

**Goal:** Match OpenCode's core agent execution exactly

#### 1.1 Fix Working Directory Behavior
- [ ] Update server to use cwd as default directory
- [ ] Test: `cd test-dummy && crow serve` uses test-dummy
- [ ] Verify sessions created in correct directory
- [ ] Tools execute with correct working_dir context

#### 1.2 Complete Agent Set
- [ ] Add Plan agent with proper prompt
- [ ] Add Explore agent with proper prompt
- [ ] Verify all agents have correct tool access
- [ ] Test delegation rules:
  - Supervisor can only delegate to build
  - Architect can delegate to supervisor/orchestrator
  - Build cannot delegate

#### 1.3 Complete Critical Tools
- [ ] Implement BashOutput (get output from background bash)
- [ ] Implement KillShell (kill background bash)
- [ ] Update Bash tool to support background execution
- [ ] Implement ExitPlanMode (for plan agent)
- [ ] Test background bash execution flow

#### 1.4 System Prompt Parity
- [ ] Extract OpenCode's system prompt builder exactly
- [ ] Include environment context (git, platform, cwd)
- [ ] Add agent-specific reminders
- [ ] Test: Compare prompts sent to LLM with OpenCode

#### 1.5 Verification Testing
- [ ] Create identical test scenarios in OpenCode and Crow
- [ ] Compare: Session structure, messages, parts
- [ ] Compare: Tool calls and arguments
- [ ] Compare: Agent responses
- [ ] Document any differences

### Phase 2: Advanced Features

#### 2.1 Streaming Implementation
- [ ] Wire up SSE streaming properly
- [ ] Stream tool execution in real-time
- [ ] Part state updates during execution
- [ ] Test with long-running tasks

#### 2.2 Permission System
- [ ] Full bash permission pattern matching
- [ ] Tool permission checking
- [ ] Permission denied error handling
- [ ] (Skip interactive permission for now - HOTL not HITL)

#### 2.3 Discriminator & Dual-Agent
- [ ] Complete discriminator agent logic
- [ ] Shared conversation management
- [ ] Verdict system implementation
- [ ] Test discriminator approval/rejection

#### 2.4 Web Tools (If Needed)
- [ ] WebFetch implementation
- [ ] WebSearch implementation
- [ ] Test with real web queries

### Phase 3: Complex Testing

#### 3.1 Real-World Scenarios
```bash
# Scenario 1: Multi-file edit
"Refactor the authentication system to use JWT tokens instead of session cookies"

# Scenario 2: Debugging
"The tests are failing in test_user_login - fix the bug"

# Scenario 3: Feature implementation
"Add a dark mode toggle to the settings page"

# Scenario 4: Research task
"Find all files that import the deprecated API and create a migration plan"
```

#### 3.2 Subagent Delegation
```bash
# Test supervisor delegating to build
"supervisor: Review the codebase and fix any TypeScript errors"
# Should: supervisor analyzes → delegates to build → build fixes → supervisor reports

# Test architect coordinating
"architect: Implement user authentication across frontend and backend"
# Should: architect plans → delegates to supervisor(s) → coordinates → reports
```

#### 3.3 Doom Loop & Error Handling
```bash
# Test doom loop detection
"Read a file that doesn't exist" 
# Should: Try 3 times, detect loop, stop with warning

# Test error recovery
"Fix the syntax error in broken_file.js"
# Should: Read file, attempt fix, verify, iterate
```

#### 3.4 Cost Tracking Accuracy
```bash
# Verify costs match expected pricing
# Track costs across parent + all child sessions
# Ensure no double-counting
```

### Phase 4: Dioxus Web UI

**Only after core parity is achieved**

- [ ] Build web-based UI (not terminal TUI)
- [ ] Session management interface
- [ ] Real-time message/part rendering
- [ ] Multi-project support
- [ ] Cost tracking display

## Testing Strategy

### Comparative Testing (Primary)
```bash
# Run same task in OpenCode and Crow
# Compare outputs at each step

# OpenCode:
cd test-project && opencode
> "list all rust files"
# Save session JSON

# Crow:
cd test-project && crow serve
curl -X POST .../message -d '{"text": "list all rust files"}'
# Save session JSON

# Diff the results
diff opencode-session.json crow-session.json
```

### Integration Testing
- Real LLM calls (not mocked)
- Real file system operations
- Real git operations
- Measure actual costs

### Unit Testing
- Tool execution with context
- Agent prompt building
- Cost calculation
- Doom loop detection

## Success Criteria

### Phase 1 Complete When:
1. Same prompt in OpenCode and Crow produces equivalent behavior
2. Session structures match (sessions, messages, parts)
3. Tool calls are identical
4. Costs are tracked correctly
5. Subagent spawning works consistently
6. All core agents implemented and tested

### Full Parity When:
1. Can replace OpenCode with Crow transparently
2. All agents behave identically
3. All tools work the same
4. Streaming updates work
5. Discriminator validates correctly
6. Complex multi-step tasks complete successfully

## Why This Approach Will Work

1. **Clear Target** - We're not guessing, we're copying something proven
2. **Testable** - Every step can be compared against OpenCode
3. **Incremental** - Each phase builds on the last
4. **Pragmatic** - Working code > perfect architecture
5. **Refactorable** - Once it works, we can optimize

## The Mindset

**Stop trying to outsmart OpenCode. Start trying to match it.**

They've done the hard work. They've tested it in production. They've iterated on the design. Our job is to:

1. Understand what they built
2. Recreate it in Rust
3. Get the benefits of Rust (safety, speed, memory)
4. Then, and only then, consider improvements

Think of it like translating a book. You don't rewrite the story - you translate it faithfully. The artistry is in the translation, not in changing the plot.

## Next Immediate Steps

1. **Fix working directory behavior** - Make `cd project && crow serve` work like OpenCode
2. **Add Plan and Explore agents** - Copy their prompts exactly
3. **Implement background bash** - BashOutput + KillShell tools
4. **Comparative testing** - Run same tasks in both, diff the results
5. **Document gaps** - Every difference we find, we fix

## The Vision

Imagine:
```bash
cd /home/thomas/my-rust-project
crow serve

# Beautiful Dioxus web UI opens
# Agent executes tasks
# Costs tracked
# Sessions managed
# Everything just works
# All the power of OpenCode
# All the safety of Rust
```

That's what we're building. Not because we're being clever. Because we're being smart enough to copy excellence.

---

**Let's stop overthinking and start shipping. Carbon copy first. Refinement later. Rust benefits throughout.**

Now let's get to work on Phase 1.
