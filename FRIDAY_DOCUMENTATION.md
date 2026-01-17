# Friday Documentation Session - Summary

**Date**: 2025-01-17
**Goal**: Document where Crow is, where it's going, and what needs to happen

## What We Did

Created comprehensive documentation for the Crow project:

1. **DESIGN.md** - The vision and architecture
2. **CURRENT_STATE.md** - Analysis of existing code and problems
3. **ROADMAP.md** - Development phases and timeline
4. **Updated README.md** - Clear project overview with links to docs

## Key Insights

### The Problem

Current AI coding tools drop agents into empty workspaces with no context. Agent decisions are "lost like tears in rain" in markdown files. There's no feedback loop for human review.

### The Solution

**Crow is an Agent Development Environment (ADE)** where:

1. **Humans plan** in a journal (Logseq-inspired knowledge base)
2. **Humans + agents prime** the environment together (pair programming)
3. **Autonomous agents work** in the primed environment (read journal → write code → document)
4. **Humans review** in the journal and provide feedback
5. **Knowledge accumulates** and agents get better over time

### The Architecture

```
Crow
├── ACP Server (src/crow/agent/) ✅ DONE
├── Orchestration (src/crow/orchestration/) ✅ DONE (needs cleanup)
│   ├── Environment priming 📋 NEXT
│   ├── Task splitting ✅
│   ├── Iterative refinement ✅
│   └── Task pipeline ✅
├── Web UI (Future) 📋
│   ├── CodeBlitz/Monaco editor
│   ├── Journal page
│   ├── Project browser
│   └── Terminal
└── Projects (/projects/) 📋
    └── Each project = git repo + journal
```

## Current State

### What Works (Phase 0 - Complete)

- ✅ ACP server with streaming support
- ✅ Iterative refinement pipeline (planning → implementation → critique → documentation)
- ✅ Task splitting and pipeline execution
- ✅ MCP integration (playwright, zai-vision, fetch, web_search)
- ✅ Session management and persistence

### What's Broken

- ❌ Empty workspace problem - agents dropped into empty directories with no context
- ❌ Lost knowledge - markdown files not connected or queryable
- ❌ No feedback loop - human review not captured
- ❌ Messy code structure - files in root directory

### The Big Problem

```python
# From task_pipeline.py line 177
task_output_dir = self.workspace_dir / f"task_{task_id}"
task_output_dir.mkdir(exist_ok=True)  # ← EMPTY DIRECTORY
```

Agents have NO:
- Existing codebase
- Config files
- Project structure
- Dependencies
- Test framework
- NOTHING

## The Solution: Environment Priming

### Phase 0: Environment Priming (NEW)

**NOT autonomous.** This is human + agent pair programming.

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

4. **Validate**
   ```bash
   python -c "import crow.cli.acp"
   ```
   Works from day one.

### The Journal (Logseq-Inspired)

**NOT** an iframe. A page that works like Logseq:

- `[[wiki links]]` - Connect concepts
- Backlinks - See what links here
- Tags - #todo, #decision, #agent
- Time stamps
- Search
- Graph view (eventually)

**Structure**:
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

## Roadmap

### Phase 1: Restructure 🚧 NEXT
- Move files from root to `src/crow/orchestration/`
- Clean up the mess
- Make code maintainable

### Phase 2: Jinja2 Templates 📋
- Replace hardcoded prompts with templates
- Make prompts versionable

### Phase 3: Environment Priming 📋
- Create Phase 0 workflow
- Human + agent pair programming
- Validate environments

### Phase 4: Project Management 📋
- `/projects/` directory
- Each project = git repo + journal

### Phase 5: Journal Page (MVP) 📋
- Basic journal for projects
- Wiki links, backlinks, tags
- Agent read/write API

### Phase 6: Web UI (MVP) 📋
- CodeBlitz/Monaco integration
- Project browser, journal, terminal

### Phase 7: Feedback Loops 📋
- Capture human feedback
- Feed back to agents

### Phase 8: Telemetry (Optional) 📋
- Self-hosted Laminar/Langfuse
- Prompt optimization

## Timeline

**Full-time** (8h/day): 4 weeks
**Part-time** (2-3h/day): 4 months

## Success Criteria

### Phase 1-3
- ✅ Clean code structure
- ✅ Prompts are templates
- ✅ Environment priming works
- ✅ `python -c "import crow.cli.acp"` works

### Phase 4-5
- ✅ Can create projects
- ✅ Projects have journals
- ✅ Agents can read/write journals
- ✅ Knowledge is connected

### Phase 6-7
- ✅ Web UI works
- ✅ Can edit code in browser
- ✅ Can view/edit journal
- ✅ Human feedback captured

## Key Decisions

### Why OpenHands SDK?
> "Vibe coding a production quality coding agent is actually more challenging than you might think. Because of the millions of hours of use catching bugs and corner cases."

Standing on giants' shoulders, fixing what we disagree with (e.g., Laminar telemetry requirement).

### Why /projects/ Directory?
Filesystems are fractal. Each project is an independent git repo with its own journal.

### Why Journal (Not Just Markdown)?
Knowledge needs to be connected, queryable, and linked. Agents need context. Humans need to see reasoning.

### Why Environment Priming?
Agents need context, constraints, rubrics, and alignment. They can't create from nothing.

## Next Steps

When coding resumes (after Friday):

1. **Phase 1**: Restructure into `src/crow/orchestration/`
2. **Phase 2**: Convert prompts to Jinja2 templates
3. **Phase 3**: Build environment priming workflow
4. **Phase 4**: Add project management
5. **Phase 5**: Create journal page

## References

- [DESIGN.md](DESIGN.md) - Full design document
- [CURRENT_STATE.md](CURRENT_STATE.md) - Detailed code analysis
- [ROADMAP.md](ROADMAP.md) - Full roadmap with phases
- [README.md](README.md) - Project overview

## Notes

- **Hosted at**: advanced-eschatonics.com (immanentizer)
- **Philosophy**: Agent Driven Development (ADD)
- **Goal**: Human-out-of-the-loop autonomous coding
- **Method**: Build primed environments, turn agents loose, capture feedback, iterate

---

*"Friday is for documentation, not code. We've documented where we are, where we're going, and how to get there. Now we can rest."*
