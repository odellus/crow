# Crow: Current State Analysis

**Date**: 2025-01-17 (Friday - Documentation Day)

## The Mess

We have 11 Python files in the root directory that were pure vibe coded:

```
crow/
├── config_loader.py (3.9 KB)
├── crow_browser_use.py (1.8 KB)
├── crow_iterative_refinement.py (13.5 KB)
├── crow_mcp_integration.py (5.6 KB)
├── crow_streaming.py (4.2 KB)
├── iterative_refinement.py (20.8 KB) ← Main orchestration
├── iterative_refinement_template.py (15.6 KB)
├── main.py (797 B)
├── status.py (2.4 KB)
├── task_pipeline.py (12.0 KB)
└── update_plan.py (1.3 KB)
```

## File-by-File Analysis

### Core Orchestration

#### `iterative_refinement.py` (20.8 KB)
**Purpose**: Main orchestration pipeline for planning → implementation → critique → documentation

**What it does**:
- Phase 1: Planning agent reads PLAN.md, creates task list
- Phase 2: Implementation agent executes tasks (with iterative refinement)
- Phase 3: Critique agent tests and scores (with playwright + zai-vision MCP)
- Phase 4: Documentation agent documents and cleans up

**Issues**:
- ❌ Uses `safe_format()` hack instead of Jinja2
- ❌ Prompts hardcoded as module constants
- ❌ No error handling (but you disagree this is needed)
- ❌ Cost tracking only for planning LLM
- ❌ Creates empty workspace for each task (THE BIG PROBLEM)

**Good**:
- ✅ Clear separation of concerns (4 agents)
- ✅ Iterative refinement loop
- ✅ Quality threshold
- ✅ MCP integration for critic

#### `task_pipeline.py` (12.0 KB)
**Purpose**: Split PLAN.md into tasks, run through iterative refinement

**What it does**:
- Splits big PLAN.md into 5-15 tasks
- Creates task_manifest.json
- Runs tasks sequentially through iterative refinement
- Tracks progress, can resume

**Issues**:
- ❌ Line 177: `task_output_dir = self.workspace_dir / f"task_{task_id}"`
- ❌ Creates EMPTY directory for each task
- ❌ Agent has NO context (no codebase, no configs, no nothing)
- ❌ This is the "lost in empty room" problem

**Good**:
- ✅ Task dependency checking
- ✅ Progress tracking
- ✅ Can resume from specific task
- ✅ Saves results to JSON

#### `iterative_refinement_template.py` (15.6 KB)
**Purpose**: Example showing COBOL to Java refactoring

**What it does**:
- Demonstrates the pattern
- Shows how prompts should look
- Reference implementation

**Status**: Good reference, keep it

### ACP Server

#### `src/crow/agent/acp_server.py`
**Purpose**: Minimal ACP server wrapping OpenHands SDK

**What it does**:
- Implements ACP protocol (JSON-RPC over stdin/stdout)
- Streaming responses (unlike OpenHands-CLI)
- Session management
- MCP integration
- Slash commands
- Session persistence

**Status**: This is solid, working well

### Other Files (Likely Experiments)

#### `config_loader.py` (3.9 KB)
**Purpose**: Load YAML config

**Status**: Part of refactor plan, keep

#### `crow_browser_use.py` (1.8 KB)
**Purpose**: Browser automation experiments

**Status**: Probably can delete or move to tests/

#### `crow_iterative_refinement.py` (13.5 KB)
**Purpose**: Alternative iterative refinement implementation?

**Status**: Duplicate? Needs investigation

#### `crow_mcp_integration.py` (5.6 KB)
**Purpose**: MCP server configuration

**Status**: Useful reference, keep

#### `crow_streaming.py` (4.2 KB)
**Purpose**: Streaming experiments

**Status**: Probably integrated into acp_server.py now

#### `main.py` (797 B)
**Purpose**: Entry point?

**Status**: Minimal, probably outdated

#### `status.py` (2.4 KB)
**Purpose**: Status checking?

**Status**: Utility, keep

#### `update_plan.py` (1.3 KB)
**Purpose**: Update plan files?

**Status**: Utility, keep

## The Big Problem: Empty Workspace

```python
# From task_pipeline.py line 177
task_output_dir = self.workspace_dir / f"task_{task_id}"
task_output_dir.mkdir(exist_ok=True)

# Then calls iterative refinement
result = run_iterative_refinement(
    plan_file=task_file,
    workspace_dir=task_output_dir,  # ← EMPTY DIRECTORY
    ...
)
```

**What happens**:
1. Task starts
2. Agent dropped into empty directory
3. Agent has NO context:
   - No existing codebase
   - No config files
   - No project structure
   - No dependencies installed
   - No test framework
   - No NOTHING
4. Agent tries to implement task
5. Agent is confused and fails

**The solution**: Environment priming phase BEFORE autonomous agents

## What Needs Restructuring

### Move to src/crow/

```
src/crow/
├── agent/
│   └── acp_server.py (already there)
├── orchestration/
│   ├── environment.py          # NEW: Environment priming
│   ├── task_splitter.py        # From task_pipeline.py
│   ├── task_pipeline.py        # From task_pipeline.py
│   └── iterative_refinement.py # From iterative_refinement.py
├── cli/
│   └── commands.py             # CLI entry points
└── config/
    └── loader.py               # From config_loader.py
```

### Convert to Jinja2

**Current** (hardcoded prompts):
```python
DEFAULT_PLANNING_PROMPT = """You are a PLANNING AGENT..."""
```

**After** (Jinja2 templates):
```python
# prompts/planning.md.j2
You are a PLANNING AGENT for {{ project_name }}

Read the plan at {{ plan_file }} and create a task list.

{% if previous_context %}
Previous context: {{ previous_context }}
{% endif %}
```

### Add Environment Priming

**NEW** - `src/crow/orchestration/environment.py`:

```python
def prime_environment(
    workspace_dir: Path,
    plan_content: str,
    llm: LLM,
) -> PrimedEnvironment:
    """
    Phase 0: Prime the environment with human-in-the-loop.
    
    1. Human + Agent discuss requirements in journal
    2. Create project structure
    3. Set up configs (pyproject.toml, .env)
    4. Write end-to-end test FIRST
    5. Document all decisions in journal
    6. Validate: `python -c "import ..."` works
    """
```

## Documentation Files

### Existing
- ✅ `AGENTS.md` (53 KB) - Project knowledge
- ✅ `README.md` (13.6 KB) - ACP server docs
- ✅ `REFACTOR_PLAN.md` (5.3 KB) - Refactor plan
- ✅ `DESIGN.md` (NEW) - Vision and architecture
- ✅ `CURRENT_STATE.md` (THIS FILE) - Analysis

### Need to Create
- ⏳ `ROADMAP.md` - What we're building and when
- ⏳ `API.md` - ACP server API documentation
- ⏳ `PROMPTS.md` - Prompt documentation
- ⏳ `TESTING.md` - Testing strategy

## Next Steps (When Coding Resumes)

### Priority 1: Restructure
1. Create `src/crow/orchestration/`
2. Move files from root to proper locations
3. Update imports
4. Test everything still works

### Priority 2: Jinja2 Templates
1. Create `prompts/` directory
2. Convert all prompts to `.md.j2` files
3. Update code to use templates
4. Test prompt rendering

### Priority 3: Environment Priming
1. Design the priming workflow
2. Create `environment.py`
3. Implement human-in-the-loop session
4. Test with a simple project

### Priority 4: Journal/Project Management
1. Design journal data structure
2. Implement project management
3. Create journal page (basic)
4. Test agent integration

## What Works Right Now

✅ **ACP Server** - Streaming, session management, MCP integration
✅ **Iterative Refinement** - The pattern works, just needs context
✅ **Task Pipeline** - Execution works, just needs primed environment
✅ **MCP Integration** - playwright, zai-vision, fetch, web_search

## What Doesn't Work Right Now

❌ **Autonomous agents in empty workspaces** - The core problem
❌ **Knowledge management** - No journal, no feedback loop
❌ **Project structure** - Files in root, not organized
❌ **Prompt management** - Hardcoded, not versioned

## The Vision vs Reality

### Vision
- Primed environment with context
- Agents read journal, write code, document decisions
- Human reviews in journal, provides feedback
- Knowledge accumulates, agents get better

### Reality
- Agents dropped into empty rooms
- Decisions lost in markdown files
- No feedback loop
- No knowledge accumulation

### Gap
- Environment priming phase
- Journal/knowledge system
- Human feedback capture
- Project management

## Conclusion

The **patterns** are solid (iterative refinement, task pipeline, ACP server).
The **problem** is lack of context (empty workspaces, lost knowledge).

The **solution** is environment priming + journal + feedback loops.

We're close. Just need to:
1. Clean up the mess (restructure)
2. Add the missing pieces (priming, journal)
3. Connect everything together

---

*"Friday is for documentation, not code. This is where we are."*
