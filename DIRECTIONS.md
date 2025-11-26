# Crow + Murder: Autonomous Agent Research Infrastructure

## Overview

Crow is a dual-agent coding system with comprehensive observability, built for autonomous long-horizon tasks. Murder is the orchestration and prompt optimization layer that enables recursive self-improvement through GEPA.

---

## 1. Murder: Prompt Management & Orchestration System

**Like Langfuse, but local and agent-native**

### Core Features:
- **Unified prompt versioning** stored in `~/.config/crow/prompts/`
- **Manifest system** tracks active prompt versions per agent
- **Easy swapping**: `crow-cli prompt activate executor v1.2.5`
- **Evaluation harness**: Run agents on datasets, compare scores
- **Full audit trail**: All sessions persist to XDG directories

### Directory Structure:
```
~/.config/crow/
├── prompts/
│   ├── executor/v1.2.3.txt
│   ├── discriminator/v2.1.0.txt
│   └── architect/v1.0.0.txt
├── manifest.json  # Active versions
└── eval/
    ├── datasets/
    └── results/
```

### Why Critical:
Without unified prompt management, GEPA cannot iterate. Need ability to:
- Test candidate prompts
- Compare performance
- Roll back bad versions
- Track improvements over time

---

## 2. What If Machine: Synthetic Dataset Generation

**Turns messy HITL sessions into clean SWE-bench style examples**

### The Problem:
- Real sessions are long, rambling, full of dead ends
- Hard to use as training data
- Can't evaluate on them effectively

### The Solution:
1. **Extract real tasks** from git history + session logs
2. **Branch to old state** (git checkout before fix)
3. **Agent solves cleanly** with current prompts
4. **Verify solution** (git proves it matches or is better)
5. **Generate training example** (real task + synthetic clean conversation)

### Example Flow:
```python
def generate_training_example(session: Session) -> CleanExample:
    # 1. Parse messy session
    tasks = extract_tasks(session)  # "fixed auth bug"
    
    # 2. Find git states
    broken_state = find_commit_before(task)
    fixed_state = find_commit_after(task)
    
    # 3. Replay cleanly
    git.checkout(broken_state)
    clean_conversation = agent.solve_task_cleanly(task)
    
    # 4. Verify equivalent
    assert git.diff(fixed_state, "HEAD") == "same or better"
    
    # 5. Return training example
    return CleanExample(
        task=task.description,
        broken_code=git.show(broken_state),
        conversation=clean_conversation,  # SYNTHETIC
        fixed_code=git.show(fixed_state),  # REAL
        verified=True  # Git ground truth
    )
```

### Why This Works:
- **Task is REAL** (from your actual work)
- **Solution is REAL** (verified by git)
- **Conversation is SYNTHETIC** (replayed cleanly)
- **Fully verifiable** (git proves it worked)

### Counterfactual Generation:
Can introduce bug variants to generate multiple training examples from one real task.

---

## 3. Evaluator Agent: Numeric Scoring System

**Assigns scores to agent performance on tasks**

### Scoring Criteria:
- Task completion (binary: solved or not)
- Steps taken (efficiency)
- Tests passing (correctness)
- Code quality (maintainability)
- Prompt effectiveness (how well instructions worked)

### Output Format:
```json
{
  "session_id": "ses_abc123",
  "task": "Fix authentication bug",
  "score": 9.2,
  "steps_taken": 15,
  "tests_passed": true,
  "code_quality": 8.5,
  "prompt_effectiveness": 9.8,
  "verified": true
}
```

### Integration:
- Evaluator runs on clean examples from What If Machine
- Scores feed into GEPA optimization
- High-scoring examples used for LoRA fine-tuning

---

## 4. Agent-Driven GEPA Loop

**Recursive self-improvement through prompt evolution**

### The Loop:
```
1. What If Machine → generates clean examples from sessions
2. Evaluator Agent → scores agent performance on examples
3. GEPA → evolves prompts using scores + textual feedback
4. New prompts activated → agents run on eval dataset
5. Scores compared → keep improvements, discard regressions
6. Repeat
```

### Why Agent-Driven:
- **Not just GEPA out of the box** - agents create the datasets
- **Not just scoring** - agents judge the work
- **Agents improve agents** - full recursive loop

### GEPA Specifics:
- Uses reflection on execution traces (tool calls, reasoning, results)
- Maintains Pareto frontier (diverse high-performing candidates)
- Genetic evolution with textual feedback (not just scalar rewards)
- 35x fewer rollouts than RL methods like GRPO

---

## 5. Dual Agent Architecture: Executor + Discriminator

**Adversarial pairing for quality enforcement**

### Structure:
```
Executor (Builder):
- Model: qwen3-coder-30B
- Role: Implementation, code generation
- Tools: bash, edit, read, write, grep, glob, todoread, todowrite
- Full ReACT loop (thinking + tools until no more tool calls)

Discriminator (Reviewer):
- Model: qwen3-VL-30B (VISION)
- Role: Validation, code review, UI verification
- Tools: Same as executor + screenshot + Playwright
- Reviews executor's work, provides dense reward signal
- Can reject/reopen tasks
- Only one who can call work_completed
```

### Message Inversion:
```
Executor's full ReACT cycle → rendered as markdown → User message to Discriminator
Discriminator's full ReACT cycle → rendered as markdown → User message to Executor
```

Both agents see EVERYTHING the other did (no truncation).

### Why It Works:
- **Prevents reward hacking**: Executor can't approve own work
- **Dense feedback**: Discriminator shows exactly what's wrong
- **Vision validation**: Can screenshot UI and verify correctness
- **Test-first enforcement**: Discriminator reviews tests before code

### Loop Structure (Based on Trae-Agent):
```rust
loop {
    // 1. Executor does full ReACT cycle
    executor.run_react_loop();
    
    // 2. When executor stops calling tools
    // 3. Discriminator reviews (full ReACT cycle)
    discriminator_feedback = discriminator.review();
    
    // 4. Check if work_completed
    if discriminator_feedback.contains_work_completed() {
        break;
    }
    
    // 5. Executor continues with feedback
}
```

---

## 6. Meta-Layer: AI Scientist Agents

**Agents that orchestrate and improve other agents**

### Architecture:
```
AI Scientist (Meta-Agent)
    ↓ orchestrates
Murder (Prompt optimization + eval)
    ↓ manages
Crow Agents (Architect, Planner, Executor, Discriminator)
    ↓ build software
```

### AI Scientist Responsibilities:
- Launch crow instances for experiments
- Design evaluation protocols
- Analyze results across runs
- Propose architectural improvements
- Manage provider configurations
- Self-healing inference infrastructure

### The Recursive Loop:
You used Claude Code to build crow (clone of Claude Code).
Now you'll use crow to improve crow.
Then crow uses crow to improve crow.
**Full recursive self-improvement.**

---

## 7. Infrastructure: Local LLM + Rust Everything

### Current State:
- llama.cpp for inference (qwen3-coder-30B, qwen3-VL-30B)
- AMD Ryzen AI Max+ 395 with 128GB unified memory
- Multiple llama.cpp instances on different ports

### Future Direction:
- **candle-vulkan-kernels**: Bring inference into Rust codebase
- Make it "as flossing spectacular as candle-metal-kernels"
- **Self-healing inference**: On-metal k8s-like orchestration
- Fully integrated: agents manage their own inference providers
- No external dependencies

### Why:
- Own the entire stack
- No rate limits
- No API costs
- Full control over deployment
- Research infrastructure you can modify

---

## 8. Filesystem as Database: Verbose Docs + Compression

**Agents write extensive documentation, messages are pointers**

### Philosophy:
```
Traditional: Stuff everything in context → hit limits → lose info

Your approach: Write detailed docs to files → reference in messages
              → compress tool responses → pointers survive compression
```

### Directory Structure:
```
.crow/
├── plans/
│   ├── task-breakdown.md (3000 lines)
│   └── implementation-strategy.md
├── reviews/
│   ├── discriminator-review-turn-1.md
│   └── discriminator-review-turn-2.md
├── analysis/
│   ├── codebase-structure.md
│   └── test-coverage.md
├── test-results/
│   └── auth-tests.log
└── todos/
    └── session-abc123.json
```

### Message Example:
```
Instead of:
"I analyzed the code and found [3000 lines of analysis]"

Write:
"Analysis complete. See .crow/analysis/auth-system.md
Summary: Found 5 critical issues, 3 require immediate attention.
Details in sections 2.1, 2.3, and 4.2."
```

### After Compression:
```
Before: Tool responses (3000 lines)
After: "Ran tests (results: .crow/test-results/run-5.log)"
```

**The pointer survives.** Data is recoverable on-demand.

### Why This Matters:
- ✅ Full audit trail (debugging)
- ✅ Pointers survive compression (data not lost)
- ✅ Agents can retrieve old context
- ✅ Better code (forces clear thinking)
- ✅ HOTL-ready (autonomous operation)

### Trade-offs:
- ❌ More verbose initially (bigger context)
- ❌ Slower execution (test-time scaling)
- ✅ Higher correctness (stays on rails)
- ✅ Better for long-horizon tasks

**"Slow is smooth, smooth is fast."**

---

## 9. Executable Textbooks + Custom Project Agents

**After project completion: Generate documentation + domain expert agent**

### Post-Completion Flow:
```
1. work_completed called by Discriminator
2. Generate interactive textbook (teaches the math, design decisions)
3. Create custom project agent (domain expert)
4. Situate agent in project context
```

### Interactive Textbook Example:
```
.crow/textbook/
├── 01-overview.md
├── 02-architecture-decisions.md
├── 03-why-indexeddb.md
├── 04-linear-algebra-of-crdt.md  ← Teaches the underlying math
├── 05-conflict-resolution.md
├── 06-implementation-details.md
└── 07-testing-strategy.md
```

### Custom Project Agent:
```markdown
# .crow/agents/project-expert.md

System: You are an expert on this [project name].

You know:
- Every design decision and why it was made
- Implementation details and trade-offs
- How to extend and modify the system
- The math and theory behind key components

Context:
- Architecture: .crow/architecture/
- Implementation: .crow/implementation/
- Textbook: .crow/textbook/
- All source code in project
```

### Dual Role:
**1. Explainer:**
```
User: "Why did we use CRDTs?"
Agent: "See .crow/textbook/04-linear-algebra-of-crdt.md
The decision came from conflict resolution requirements..."
```

**2. Implementer:**
```
User: "Add dark mode"
Agent: "I understand the codebase completely.
Let me implement that."
[Launches executor/discriminator subagent]
[Implementation happens]
"Done. Updated textbook with dark-mode-implementation.md"
```

### GEPA Optimization:
Project agent gets optimized via GEPA on the SWE-bench dataset generated from the actual build sessions.
**Result:** Domain-specialist agent that's THE best at working on this specific codebase.

---

## 10. Four-Agent Architecture (Full System)

```
┌─────────────────────────────────────────────────────────┐
│                    USER (You)                            │
│                       ↓                                  │
│              ┌────────────────┐                          │
│              │   ARCHITECT    │ (Vision + Playwright)    │
│              │  (Human-facing)│                          │
│              └────────────────┘                          │
│                       ↓                                  │
│         Astronaut Architecting (~1 hour):               │
│         - Deep research (web searches)                  │
│         - Opinionated pushback on ideas                 │
│         - Spec refinement with user                     │
│         - Full plan generation                          │
│         - Signals: continue | project_complete |        │
│                    new_direction                        │
│                       ↓                                  │
│              ┌────────────────┐                          │
│              │    PLANNER     │                          │
│              │  (Repo selector)│                         │
│              └────────────────┘                          │
│                       ↓                                  │
│         Environment Preparation:                        │
│         - Select repos (fitness-based, genetic alg)     │
│         - Few-shot examples for synthesis               │
│         - Launch subagent pair                          │
│                       ↓                                  │
│         ┌─────────────────────────────┐                 │
│         │   EXECUTOR + DISCRIMINATOR  │                 │
│         │      (Subagent pair)        │                 │
│         │  Discriminator: Vision +    │                 │
│         │  Playwright for validation  │                 │
│         └─────────────────────────────┘                 │
│                       ↓                                  │
│              Implementation Complete                     │
│                       ↓                                  │
│         ┌─────────────────────────────┐                 │
│         │  INTERACTIVE TEXTBOOK       │                 │
│         │  + Custom Project Agent     │                 │
│         └─────────────────────────────┘                 │
└─────────────────────────────────────────────────────────┘
```

### Architect (Product Manager):
- Faces the user
- Has vision (can see your screen)
- Has Playwright (can interact with apps)
- Does deep research
- Opinionated, pushes back
- Spends ~1 hour refining specs
- **Controls project direction**

### Planner (Environment Preparer):
- Receives architecture from Architect
- Selects repos as few-shot examples (genetic algorithm fitness)
- Extracts patterns for dialectical synthesis
- Prepares environment
- Launches Executor/Discriminator

### Executor (Builder):
- Implements according to plan
- Uses example repos as splines
- Dialectical synthesis of patterns
- Full ReACT loops

### Discriminator (Quality Gate):
- Validates correctness
- Has vision for UI verification
- Uses Playwright to test apps
- Provides dense reward signal
- Only one who calls work_completed

---

## 11. Test-Time Scaling & TDD

### Philosophy:
**Spend more compute at test time → get better results**

Like o1's approach, but:
- ✅ Visible reasoning (audit trail)
- ✅ Tool validation (can actually test)
- ✅ Adversarial review (discriminator catches mistakes)
- ✅ Explicit plans/docs (forced clarity)

### Why TDD Makes Sense Now:
**Traditional TDD:** Humans hate writing tests (too slow, boring)
**Agent TDD:** Agents don't get bored, tests are just more text

**The Hook:**
Discriminator cannot call work_completed unless:
1. Tests are comprehensive (reviews tests BEFORE code)
2. Tests actually verify requirements (can't be gamed)
3. Tests pass

**Anti-Reward-Hacking:** Tests are objective ground truth.

### Slow is Smooth:
Taking 2 hours to complete correctly >> taking 30 minutes to fail + 2 hours human debugging

For HOTL (human out of the loop) autonomous tasks, correctness > speed.

---

## 12. Crow CLI Design Philosophy

### Why REPL > TUI:
**IPython/Julia got it right:**
- Conversational
- Iterative
- Full visibility
- Session-based
- Can interrupt/redirect

**TUI adds:**
- ❌ Complex rendering
- ❌ Can't pipe/grep
- ❌ SSH issues
- ❌ More bugs

**REPL with colors:**
- ✅ Same info
- ✅ Scriptable
- ✅ Works anywhere

### Commands:
```bash
crow-cli repl                    # Interactive (for humans)
crow-cli chat "message"          # One-shot (for automation)
crow-cli chat --json "message"   # Machine-readable
crow-cli sessions                # List sessions
crow-cli prompt activate v1.2.5  # Swap prompts
```

### Session Continuity:
`crow-cli chat` automatically continues most recent session in current project directory.
**Both REPL and one-shot use same underlying session system.**

---

## 13. Genetic Algorithm View of Code Evolution

**Repos are candidates for mutation/selection**

### Example:
```
OpenCode (candidate repo)
    → mutate via Rust port
    → fitness: behavioral match to OpenCode
    → evolve: crow-cli

Selected example repos (candidates)
    → mutate via dialectical synthesis
    → fitness: match to architecture spec
    → evolve: your new project
```

### Planner's Role:
- Selection pressure (chooses repos by fitness)
- Genetic recombination (synthesizes patterns)
- Fitness evaluation (discriminator validates)
- Successful mutation (working code)

**You're building an evolutionary system for code generation.**

---

## 14. Current Status & Next Steps

### Completed:
✅ crow-cli REPL with full observability
✅ Comprehensive testing framework plan
✅ Tool implementations (bash, edit, read, write, etc.)
✅ Session persistence to XDG
✅ Streaming output with colors/emojis
✅ Ctrl+C abort handling
✅ Shadow git snapshots

### In Progress:
🔄 Tool testing (task/subagent tool is CRITICAL)
🔄 Fix display truncation (show full output)
🔄 Snapshot diff rendering

### Next:
🕐 Dual agent implementation (Executor + Discriminator)
🕐 Prompt management system (Murder foundation)
🕐 What If Machine (dataset generation)
🕐 Evaluator agent (scoring)
🕐 GEPA integration
🕐 Four-agent architecture (Architect, Planner, Executor, Discriminator)
🕐 Tauri desktop UI (polish, not critical)

---

## 15. Key Insights

1. **Test-time scaling works**: More compute → better results (like o1)
2. **TDD with agents**: Tests catch mistakes, agents don't get bored writing them
3. **Filesystem as database**: Verbose docs + pointers survive compression
4. **Dual agents prevent reward hacking**: Can't mark own homework
5. **Vision is critical**: Can validate UI correctness visually
6. **Git as ground truth**: Training data verified by version control
7. **REPL > TUI**: Simplicity, scriptability, universality
8. **Slow is smooth**: Correctness > speed for long-horizon tasks
9. **Agents improve agents**: Full recursive self-improvement loop
10. **This is science, not engineering**: Building research infrastructure

---

## 16. The Vision

**You're building recursive self-improvement infrastructure:**

- Agents that write code
- Agents that review code  
- Agents that generate training data from their own sessions
- Agents that evaluate their own performance
- Agents that optimize their own prompts
- Agents that improve the agents

**All local. All owned. All open source.**

Not a product. **A laboratory for autonomous agent research.**

The fact that it works in production is a side effect of good science.

---

**Built with:**
- Rust (bulletproof systems programming)
- llama.cpp → candle-vulkan-kernels (local inference)
- GEPA (prompt evolution)
- Git (ground truth verification)
- Vision models (UI validation)
- Your 11-hour coding marathons

**Happy Thanksgiving. Go rest. You've earned it.** 🦃🔥

---

*Document generated from conversation with Claude on 2024-11-27*
*Conversation covered: GEPA, What If Machine, dual agents, prompt management, recursive self-improvement*
