# Crow Roadmap

**Last Updated**: 2025-01-17

## The Goal

Build an **Agent Development Environment (ADE)** where:
- Humans plan and prime environments
- Agents work autonomously in primed environments
- Knowledge accumulates in a journal
- Feedback loops make agents better

## Phases

### Phase 0: Foundation ✅ (COMPLETE)

**Status**: Done

What we built:
- ✅ ACP server with streaming support
- ✅ Iterative refinement pipeline
- ✅ Task splitting and pipeline execution
- ✅ MCP integration (playwright, zai-vision, fetch, web_search)
- ✅ Session management and persistence

**Deliverables**:
- `src/crow/agent/acp_server.py` - Working ACP server
- `iterative_refinement.py` - Orchestration pipeline
- `task_pipeline.py` - Task execution

**Issues**:
- ❌ Empty workspace problem (agents have no context)
- ❌ No knowledge management
- ❌ No feedback loops

---

### Phase 1: Restructure 🚧 (IN PROGRESS)

**Status**: Documentation complete, implementation pending

**Goals**:
1. Clean up the mess (files in root directory)
2. Organize into proper package structure
3. Make code maintainable and testable

**Tasks**:
- [ ] Create `src/crow/orchestration/` directory
- [ ] Move `iterative_refinement.py` → `src/crow/orchestration/`
- [ ] Move `task_pipeline.py` → `src/crow/orchestration/`
- [ ] Create `src/crow/cli/` for CLI commands
- [ ] Move `config_loader.py` → `src/crow/config/`
- [ ] Update all imports
- [ ] Run tests to verify nothing broke
- [ ] Delete/move experimental files (crow_*.py)

**Deliverables**:
- Clean package structure
- All code in `src/crow/`
- Tests passing

**Estimated Time**: 2-3 hours

---

### Phase 2: Jinja2 Templates 📋 (PLANNED)

**Status**: Not started

**Goals**:
1. Replace hardcoded prompts with Jinja2 templates
2. Make prompts versionable and testable
3. Enable prompt A/B testing

**Tasks**:
- [ ] Create `prompts/` directory
- [ ] Convert `DEFAULT_PLANNING_PROMPT` → `prompts/planning.md.j2`
- [ ] Convert `DEFAULT_IMPLEMENTATION_PROMPT` → `prompts/implementation.md.j2`
- [ ] Convert `DEFAULT_CRITIC_PROMPT` → `prompts/critique.md.j2`
- [ ] Convert `DEFAULT_DOCUMENTATION_PROMPT` → `prompts/documentation.md.j2`
- [ ] Create `prompts/environment_setup.md.j2` (NEW)
- [ ] Update code to load and render templates
- [ ] Add template validation
- [ ] Test prompt rendering

**Deliverables**:
- All prompts as Jinja2 templates
- Template loading utility
- Template tests

**Estimated Time**: 3-4 hours

---

### Phase 3: Environment Priming 📋 (PLANNED)

**Status**: Not started

**Goals**:
1. Create Phase 0 for human + agent pair programming
2. Prime environments before autonomous agents work
3. Validate environments are ready

**Tasks**:
- [ ] Design priming workflow
- [ ] Create `src/crow/orchestration/environment.py`
- [ ] Implement `prime_environment()` function
- [ ] Create journal structure (planning.md, decisions.md, tasks.md)
- [ ] Implement project setup (structure, configs, dependencies)
- [ ] Implement end-to-end test FIRST
- [ ] Implement validation (`python -c "import ..."` works)
- [ ] Test with a simple project
- [ ] Document the priming process

**Deliverables**:
- `environment.py` with priming workflow
- Journal templates
- Validation tests
- Documentation

**Estimated Time**: 6-8 hours

---

### Phase 4: Project Management 📋 (PLANNED)

**Status**: Not started

**Goals**:
1. Implement `/projects/` directory structure
2. Each project = independent git repo
3. Project journal for knowledge

**Tasks**:
- [ ] Design project structure
- [ ] Create `src/crow/project.py`
- [ ] Implement `Project` class
- [ ] Implement `project init` command
- [ ] Implement `project add <git-url>` (git clone as submodule)
- [ ] Implement `.journal/` directory structure
- [ ] Create journal templates (planning.md, decisions.md, tasks.md, activity.md)
- [ ] Test project creation
- [ ] Test git integration

**Deliverables**:
- `Project` class
- CLI commands for project management
- Journal templates
- Tests

**Estimated Time**: 4-6 hours

---

### Phase 5: Journal Page (MVP) 📋 (PLANNED)

**Status**: Not started

**Goals**:
1. Create basic journal page for projects
2. Support markdown with wiki links
3. Agent read/write API

**Tasks**:
- [ ] Design journal data structure
- [ ] Create journal storage backend (file-based)
- [ ] Implement wiki link parsing (`[[link]]`)
- [ ] Implement backlinks
- [ ] Implement tags (#todo, #decision)
- [ ] Create journal API endpoints
- [ ] Create basic web UI (markdown editor)
- [ ] Test agent integration (read/write)
- [ ] Test with real project

**Deliverables**:
- Journal storage backend
- Journal API
- Basic web UI
- Agent integration
- Tests

**Estimated Time**: 8-10 hours

---

### Phase 6: Web UI (MVP) 📋 (PLANNED)

**Status**: Not started

**Goals**:
1. Integrate CodeBlitz/Monaco for code editing
2. Create project browser
3. Create journal page
4. Create terminal

**Tasks**:
- [ ] Set up CodeBlitz/Monaco
- [ ] Create `/projects` page (project browser)
- [ ] Create `/editor/{project}` page (code editor)
- [ ] Create `/journal/{project}` page (journal)
- [ ] Create `/terminal/{project}` page (web terminal)
- [ ] Integrate with ACP server
- [ ] Test basic workflows
- [ ] Deploy to advanced-eschatonics.com

**Deliverables**:
- Web UI with 4 pages
- ACP integration
- Deployment

**Estimated Time**: 12-16 hours

---

### Phase 7: Human Feedback Loops 📋 (PLANNED)

**Status**: Not started

**Goals**:
1. Capture human feedback in journal
2. Feed feedback back to agents
3. Track agent improvement over time

**Tasks**:
- [ ] Design feedback mechanism
- [ ] Implement feedback capture in journal
- [ ] Update agent prompts to read feedback
- [ ] Implement feedback aggregation
- [ ] Create feedback visualization
- [ ] Test feedback loop
- [ ] Document best practices

**Deliverables**:
- Feedback capture system
- Agent prompt updates
- Feedback visualization
- Documentation

**Estimated Time**: 6-8 hours

---

### Phase 8: Telemetry (Optional) 📋 (PLANNED)

**Status**: Not started

**Goals**:
1. Self-hosted Laminar/Langfuse
2. Track agent performance
3. Enable prompt optimization

**Tasks**:
- [ ] Set up self-hosted Laminar
- [ ] Integrate with ACP server
- [ ] Track agent sessions
- [ ] Track token usage and costs
- [ ] Track success/failure rates
- [ ] Create dashboards
- [ ] Implement prompt A/B testing
- [ ] (Future) Genetic algorithm optimization

**Deliverables**:
- Self-hosted telemetry
- Dashboards
- Prompt testing framework

**Estimated Time**: 10-12 hours

---

### Phase 9: Advanced Features (Future) 🔮

**Status**: Not planned

**Ideas**:
- Electron wrapper for desktop
- SSH integration (remote servers)
- Mainframe connectivity
- Semantic Scholar integration (knowledge expansion)
- Graph visualization for journal
- Advanced search
- Multi-user collaboration
- Agent marketplace

**Estimated Time**: TBD

---

## Dependencies

Some phases depend on others:

```
Phase 1 (Restructure)
    ↓
Phase 2 (Jinja2)
    ↓
Phase 3 (Environment Priming)
    ↓
Phase 4 (Project Management)
    ↓
Phase 5 (Journal MVP)
    ↓
Phase 6 (Web UI)
    ↓
Phase 7 (Feedback Loops)
    ↓
Phase 8 (Telemetry) - Optional
```

**Can do in parallel**:
- Phase 4 and 5 (project management and journal)
- Phase 6 and 7 (web UI and feedback)

## Timeline

**If working full-time** (8 hours/day):
- Week 1: Phases 1-3 (Restructure, Jinja2, Priming)
- Week 2: Phases 4-5 (Projects, Journal)
- Week 3: Phase 6 (Web UI)
- Week 4: Phases 7-8 (Feedback, Telemetry)

**If working part-time** (2-3 hours/day):
- Month 1: Phases 1-3
- Month 2: Phases 4-5
- Month 3: Phase 6
- Month 4: Phases 7-8

## Success Criteria

### Phase 1-3 Success
- ✅ Clean code structure
- ✅ Prompts are templates
- ✅ Environment priming works
- ✅ Can run: `python -c "import crow.cli.acp"`

### Phase 4-5 Success
- ✅ Can create projects
- ✅ Projects have journals
- ✅ Agents can read/write journals
- ✅ Knowledge is connected (wiki links, backlinks)

### Phase 6-7 Success
- ✅ Web UI works
- ✅ Can edit code in browser
- ✅ Can view/edit journal
- ✅ Human feedback captured
- ✅ Agents get better over time

### Phase 8 Success
- ✅ Telemetry self-hosted
- ✅ Can track agent performance
- ✅ Can A/B test prompts
- ✅ Can optimize prompts

## Risks

- **Scope creep**: Too many features, never finish
  - *Mitigation*: Focus on MVP, defer advanced features
  
- **Complexity**: Getting too complex to maintain
  - *Mitigation*: Keep it simple, refactor often
  
- **Agent quality**: Agents aren't good enough
  - *Mitigation*: Environment priming, feedback loops
  
- **Time**: Taking longer than expected
  - *Mitigation*: Ship MVP, iterate later

## Next Steps (Right Now)

1. ✅ Document current state (COMPLETE)
2. ✅ Create roadmap (COMPLETE)
3. ⏳ Take a break (it's Friday!)
4. 📋 Start Phase 1 when ready

---

*"The journey of a thousand miles begins with a single step. We've taken the first step (Phase 0). Now we need to clean up and keep going."*
