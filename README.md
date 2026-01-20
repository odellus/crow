<p align="center">
    <img src="https://github.com/odellus/crow/raw/v0.1.0/assets/crow-logo-crop.png" description="crow logo"width=500/>
</p>

# Crow

**An Agent Development Environment (ADE) for building, running, and improving autonomous coding agents.**

> **Status**: In active development. Phase 0 complete (ACP server, iterative refinement, task pipeline). See [ROADMAP.md](ROADMAP.md) for what's next.

## What is Crow?

Crow is NOT just an ACP server, NOT just an IDE. It's a complete environment where:

- **Humans plan** in a journal (Logseq-inspired knowledge base)
- **Humans + agents prime** the environment together (pair programming)
- **Autonomous agents work** in the primed environment (read journal → write code → document decisions)
- **Humans review** in the journal and provide feedback
- **Knowledge accumulates** and agents get better over time

## Quick Start

### Installation

```bash
cd crow
uv sync
```

### Run the ACP Server

```bash
python -m crow.agent.acp_server
```

The server will listen on stdin/stdout for JSON-RPC messages following the ACP protocol.

### Run Iterative Refinement

```bash
python task_pipeline.py --plan-file PLAN.md
```

This will:
1. Split PLAN.md into tasks
2. Run each task through iterative refinement (planning → implementation → critique)
3. Track progress and results

## Documentation

- **[DESIGN.md](DESIGN.md)** - Vision, architecture, and design decisions
- **[CURRENT_STATE.md](CURRENT_STATE.md)** - Analysis of current code and what needs fixing
- **[ROADMAP.md](ROADMAP.md)** - Development phases and timeline
- **[AGENTS.md](AGENTS.md)** - Project-specific knowledge for agents
- **[REFACTOR_PLAN.md](REFACTOR_PLAN.md)** - Original refactor plan (superseded by ROADMAP.md)

## Features

### Current (Phase 0 - Complete)

- ✅ **ACP Server** - Streaming ACP server wrapping OpenHands SDK
- ✅ **Iterative Refinement** - Planning → Implementation → Critique → Documentation loop
- ✅ **Task Pipeline** - Split PLAN.md into tasks, run sequentially
- ✅ **MCP Integration** - playwright, zai-vision, fetch, web_search
- ✅ **Session Management** - Multiple concurrent sessions with persistence
- ✅ **Slash Commands** - /help, /clear, /status

### In Progress (Phase 1-3)

- 🚧 **Restructure** - Moving files from root to `src/crow/`
- 📋 **Jinja2 Templates** - Replace hardcoded prompts with templates
- 📋 **Environment Priming** - Human + agent pair programming before autonomous phase

### Planned (Phase 4-8)

- 📋 **Project Management** - `/projects/` directory, git repos, journals
- 📋 **Journal Page** - Logseq-inspired knowledge base
- 📋 **Web UI** - CodeBlitz/Monaco integration
- 📋 **Feedback Loops** - Capture human feedback, feed to agents
- 📋 **Telemetry** - Self-hosted Laminar/Langfuse

## Architecture

```
Crow
├── ACP Server (src/crow/agent/)
│   └── Streaming ACP protocol implementation
├── Orchestration (src/crow/orchestration/)
│   ├── Environment priming
│   ├── Task splitting
│   ├── Iterative refinement
│   └── Task pipeline
├── Web UI (Future)
│   ├── CodeBlitz/Monaco editor
│   ├── Journal page
│   ├── Project browser
│   └── Terminal
└── Projects (/projects/)
    └── Each project = git repo + journal
```

## The Problem We're Solving

Current AI coding tools:
- ❌ Drop agents into empty workspaces (no context)
- ❌ Lose agent decisions in markdown files ("lost like tears in rain")
- ❌ No feedback loop (human review not captured)
- ❌ No knowledge accumulation

Our solution:
- ✅ **Environment priming** - Human + agent set up context first
- ✅ **Journal** - All decisions documented and linked
- ✅ **Feedback loops** - Human review captured and fed back
- ✅ **Knowledge accumulation** - Agents get better over time

## Contributing

This is a personal project, but feedback and ideas are welcome!

## License

MIT

## Acknowledgments

- [Agent Client Protocol](https://agentclientprotocol.com/)
- [OpenHands SDK](https://docs.openhands.dev/)
- [Model Context Protocol](https://modelcontextprotocol.io/)
- [Trae Solo](https://traesolo.net/) - Autonomous development inspiration
- [Google Antigravity](https://antigravity.google/) - Agent-first IDE inspiration
- [Logseq](https://logseq.com/) - Knowledge management inspiration
- [CodeBlitz](https://github.com/sugarforever/codeblitz) - Web IDE foundation

---

*"The agent is the primary developer, humans are the critics/product managers."*

Modified with Crow ADE
