# Crow Design Document

**Last Updated**: 2025-01-17 (Friday - Documentation Day)

## What is Crow?

Crow is an **Agent Development Environment (ADE)** - a workspace for building, running, and improving autonomous coding agents.

It is NOT:
- Just an ACP server
- Just an IDE
- Just a prompt management system

It IS:
- A complete environment where agents develop software
- A knowledge management system integrated with code
- A feedback loop for continuous agent improvement
- A server that produces artifacts/code

## The Vision

### The Problem

Current AI coding tools have issues:
1. **Lost Knowledge** - Agent decisions are "lost like tears in rain" in markdown files
2. **No Context** - Agents dropped into empty workspaces with no grounding
3. **No Feedback** - Human review not captured or fed back into the system
4. **Vibe Code** - Production quality requires millions of hours of bug fixes

### The Solution

```
Human Planning (Journal)
    ↓
Environment Priming (Human + Agent Pair Programming)
    ↓
Autonomous Agents (Read Journal → Write Code → Document in Journal)
    ↓
Human Review (Review in Journal → Provide Feedback)
    ↓
Knowledge Accumulates (Feedback captured → Agents get better)
```

## Architecture

### Foundation

```
OpenHands SDK
├── Millions of hours of production use
├── Battle-tested tools and patterns
├── We're fixing the parts we disagree with
└── Example: Laminar telemetry (should be self-hosted)
```

### Components

```
Crow
├── ACP Server (src/crow/agent/)
│   ├── Streaming support (unlike OpenHands-CLI)
│   ├── Proper ACP protocol compliance
│   ├── Session management
│   └── MCP integration
│
├── Orchestration (src/crow/orchestration/)
│   ├── Environment priming
│   ├── Task splitting
│   ├── Iterative refinement
│   └── Task pipeline
│
├── Web UI (CodeBlitz/Monaco)
│   ├── /editor - Code editor
│   ├── /journal - Knowledge base (Logseq-inspired)
│   ├── /projects - Project browser
│   └── /terminal - Web terminal
│
├── Project Management
│   ├── /projects/ - All projects live here
│   ├── Each project = Independent git repo
│   ├── .journal/ - Project-specific knowledge
│   └── Git clone = Git submodule (alias)
│
└── Telemetry (Future)
    ├── Laminar/Langfuse (self-hosted)
    ├── Prompt management
    └── Genetic algorithm optimization
```

## The Workflow

### Phase 0: Environment Priming (Human + Agent)

**NOT autonomous.** This is a pair programming session.

1. **Planning in Journal**
   - Outline requirements
   - Define architecture
   - Create task hierarchy
   - Document constraints

2. **Set Up Environment**
   - Create project structure
   - Configure (pyproject.toml, .env)
   - Install dependencies
   - Create skeleton code

3. **End-to-End Test FIRST**
   - Define what "done" looks like
   - Implement minimal test
   - Validate key concepts
   - All decisions documented in journal

4. **Validate**
   ```bash
   python -c "import crow.cli.acp"
   ```
   Works from day one.

### Phase 1: Task Splitting

Split PLAN.md into well-scoped tasks:
- 5-15 tasks
- Each 1-3 hours
- Clear acceptance criteria
- Minimal dependencies

### Phase 2: Iterative Refinement

For each task:
1. **Planning Agent** - Creates task list
2. **Implementation Agent** - Executes task
3. **Critic Agent** - Tests and scores (with playwright + zai-vision)
4. **Loop** - Until quality threshold met

### Phase 3: Human Review

Human reviews in journal:
- Sees agent decisions
- Provides feedback
- Feedback captured
- Agents get better

## The Journal (Logseq-Inspired)

**NOT** an iframe. A page that works like Logseq:

### Features
- `[[wiki links]]` - Connect concepts
- Backlinks - See what links here
- Tags - #todo, #decision, #agent
- Time stamps
- Search
- Graph view (eventually)

### Structure
```markdown
# Project Journal

## Planning
- [[Project Vision]]: What are we building?
- [[Requirements]]: What do we need?
- [[Architecture]]: How will it work?

## Tasks
- [[Task 1]]: Set up project structure
- [[Task 2]]: Write end-to-end test

## Decisions
- [[Decision 1]]: Use Monaco for editor
- [[Decision 2]]: Store projects in /projects/

## Agent Activity
- [[Session 1]]: Environment priming
- [[Session 2]]: Task implementation
```

### Agent Integration
```python
# Agent prompts reference journal
"""
**Context**: Read the journal at {journal_path}
- planning.md: Requirements and vision
- decisions.md: Architecture decisions
- tasks.md: Task breakdown

**Documentation**: Document your work in activity.md
"""
```

## Project Structure

### Current State (Messy)

```
crow/
├── *.py files in root (vibe coded, needs restructuring)
├── AGENTS.md
├── README.md
├── REFACTOR_PLAN.md
└── prompts/
```

### Target State

```
crow/
├── src/crow/
│   ├── __init__.py
│   ├── agent/              # ACP server
│   │   ├── acp_server.py
│   │   └── config.py
│   ├── cli/                # CLI commands
│   ├── editor/             # CodeBlitz integration
│   ├── mcp/                # MCP integration
│   ├── orchestration/      # Pipeline orchestration
│   │   ├── environment.py      # Environment priming
│   │   ├── task_splitter.py    # Split PLAN.md
│   │   ├── task_pipeline.py    # Run tasks
│   │   └── iterative_refinement.py
│   └── telemetry/          # Laminar/Langfuse
├── prompts/                # Jinja2 templates
│   ├── environment_setup.md.j2
│   ├── task_splitter.md.j2
│   ├── planning.md.j2
│   ├── implementation.md.j2
│   ├── critique.md.j2
│   └── documentation.md.j2
├── tests/
├── AGENTS.md               # Project-specific knowledge
├── DESIGN.md               # This document
├── PLAN.md                 # Big plan
├── pipeline_config.yaml    # Pipeline configuration
└── pyproject.toml
```

## Current State Analysis

### What Works
- ✅ ACP server with streaming support
- ✅ Iterative refinement pipeline
- ✅ Task splitting
- ✅ Task pipeline execution
- ✅ MCP integration (playwright, zai-vision, fetch, web_search)

### What's Broken
- ❌ Empty workspace problem (agents have no context)
- ❌ Lost knowledge (markdown files not connected)
- ❌ No feedback loop (human review not captured)
- ❌ Vibe code in root (needs restructuring)
- ❌ No environment priming phase
- ❌ No journal/knowledge system

### What's Missing
- ⏳ Project management (/projects/)
- ⏳ Journal page (Logseq-inspired)
- ⏳ Web UI (CodeBlitz integration)
- ⏳ Environment priming workflow
- ⏳ Human feedback capture
- ⏳ Telemetry integration

## Key Decisions

### Why OpenHands SDK?
> "Vibe coding a production quality coding agent is actually more challenging than you might think. Because of the millions of hours of use catching bugs and corner cases."

Standing on giants' shoulders, fixing what we disagree with.

### Why /projects/ Directory?
Filesystems are fractal. Each project:
- Independent git repo
- Has its own journal
- Managed by Crow
- Git clone = git submodule alias

### Why Journal (Not Just Markdown)?
- Knowledge needs to be connected
- Agents need to query context
- Humans need to see reasoning
- Feedback needs to be captured
- "Lost like tears in rain" problem

### Why Environment Priming?
Agents need:
- Context (what are we building?)
- Constraints (what are the rules?)
- Rubrics (what does "done" look like?)
- Alignment (grounded in human specs)

## Future Plans

### Eventually (Not Day One)
- Electron wrapper for desktop
- SSH integration (remote servers)
- Mainframe connectivity
- Semantic Scholar integration (knowledge expansion)
- Genetic algorithm prompt optimization

### Day One Priorities
1. Restructure into src/crow/
2. Jinja2 templates for prompts
3. Environment priming phase
4. Journal page (Logseq-inspired)
5. Project management (/projects/)
6. Human feedback capture

## References

- [Agent Client Protocol](https://agentclientprotocol.com/)
- [OpenHands SDK](https://docs.openhands.dev/)
- [Model Context Protocol](https://modelcontextprotocol.io/)
- [Trae Solo](https://traesolo.net/) - Autonomous development inspiration
- [Google Antigravity](https://antigravity.google/) - Agent-first IDE inspiration
- [Logseq](https://logseq.com/) - Knowledge management inspiration
- [CodeBlitz](https://github.com/sugarforever/codeblitz) - Web IDE foundation

## Notes

- **Hosted at**: advanced-eschatonics.com (immanentizer)
- **Philosophy**: Agent Driven Development (ADD)
- **Goal**: Human-out-of-the-loop autonomous coding
- **Method**: Build primed environments, turn agents loose, capture feedback, iterate

---

*"The agent is the primary developer, humans are the critics/product managers."*
