# Crow Project Whiteboard Notes

## Core Architecture

### CLI Foundation
- **Python CLI** (NOT a TUI! CLI interface)
- **OpenHands agents** with memory tools and skill bots
- **~/crow** for auth and configuration
- **ACP First!** - Building on AEP for immediate deployment capability

## Development Strategy

### OpenHands Fork Strategy
- Work on local fork of OpenHands (OH)
- Turn changes into PRs
- **Key PR goal**: "Make security tools optional"
- Develop sync mechanism for upstream OH changes without breaking our fork

### Code Generation Approach
- **Old school template shift**
- Multiple options available
- Heavy use of old school codegen for first pass prototype
- Rapid prototyping → hands off to optimization

## Skillbot Concept

### What is Skillbot?
- Agent that researches through sessions
- Does deep analysis of agent behavior
- Builds shareable skills for OpenHands
- **Philosophy**: Rapid prototyping then hands off optimization

## Core Features

### Must-Have Features
- ✅ Streaming support
- ✅ Fix outputs of tasks
- ✅ Add more tools
- ✅ Customize OpenHands system prompt
- ✅ Prompt management
- ✅ Agent coordination

## Cloud Infrastructure Strategy

### VSCodium + Code-Server Setup
- Use **VSCodium build pipeline** with **code-server** setup
- Operationalize local desktop app as website
- Connect to PDS (Personal Data Server)
- Already set up as website (not a jerk, have a job, this is for fun)

### Kubernetes Architecture
- **Persistent filesystem** powered by K8s
- PV/PVC setup for persistent storage
- **Same pattern** as local development, just swap in persistent volume
- **Auto-scale to zero** when inactive to save costs
- **Long-running agents** by design

### Cost Optimization Strategy
- Spin down inactive instances
- But these are meant to be long-running agents
- Design to **maxx the fuck out of subscription**

## Use Case: Local Slow Machine

### Origin Story
- Very slow local machine that needs to always be running
- Want continuous background processing
- **Endpoint for slow async processes** that don't need real-time completion
- Always keep endpoint working with bookkeeping tasks

### Training Loop Vision (The Real Goal)
- Endpoint working in continuous loop
- Generate turns with **local model vs cloud models**
- **Constantly train local model** on good turns of conversation
- Self-improvement cycle
- **Baby steps** to get there

## Technical Stack

### Frontend
- **VSCodium** (open source VS Code binaries)
- **code-server** for web-based IDE
- Custom extensions and themes
- Transform VS Code into completely different experience

### Backend
- **atproto PDS** as backbone
  - Decentralized identity
  - DNS for CDN
  - Freeze requests in time
- **Kubernetes** cluster
  - Auto-scaling
  - Persistent volumes
  - Container orchestration

### Agent Platform
- **OpenHands SDK** foundation
- **ACP (Agent Client Protocol)** first
- **MCP (Model Context Protocol)** integration
- Custom tools and skills

## Business Model

### Open Source + Cloud
- **Open source core** (transparent documentation)
- **Cloud version** using same exact technology
- **Tiered subscriptions** based on:
  - Number of agents
  - Compute resources
  - Storage limits
- **Break even** at minimum
- **Profit** as we scale

### Value Proposition
- "Claude Code for web that doesn't suck"
- Full desktop experience through web
- Never worry about resource constraints
- Same tech, different packaging

## Development Phases

### Phase 0: Foundation ✅
- ACP server with streaming
- Iterative refinement pipeline
- Task splitting and execution
- MCP integration
- Session management

### Phase 1: Restructure 🚧
- Clean up codebase
- Proper package structure
- Move files to src/crow/
- Make maintainable and testable

### Phase 2: Templates
- Jinja2 templates for prompts
- Versionable prompts
- A/B testing framework

### Phase 3: Environment Priming
- Human + agent pair programming
- Setup before autonomous work
- End-to-end tests FIRST
- Validate environments

### Phase 4: Project Management
- /projects/ directory structure
- Each project = independent git repo
- Project journals for knowledge

### Phase 5: Journal (MVP)
- Logseq-inspired knowledge base
- Wiki links, backlinks, tags
- Agent read/write API
- Web UI

### Phase 6: Web UI
- CodeBlitz/Monaco integration
- /editor, /journal, /projects, /terminal
- ACP server integration
- Deploy to advanced-eschatonics.com

### Phase 7: Feedback Loops
- Capture human feedback
- Feed back to agents
- Track improvement over time

### Phase 8: Telemetry (Optional)
- Self-hosted Laminar/Langfuse
- Track agent performance
- Prompt optimization
- Genetic algorithm optimization

## Key Principles

### Agent Driven Development (ADD)
- **Agent is primary developer**
- **Humans are critics/product managers**
- Human-out-of-the-loop autonomous coding
- Build primed environments → turn agents loose → capture feedback → iterate

### Knowledge Management
- **Journal system** (Logseq-inspired)
- Wiki links connect concepts
- Backlinks show relationships
- Tags for organization (#todo, #decision, #agent)
- **No more "lost like tears in rain"**

### Environment Priming
- Agents need context, constraints, rubrics
- Human + agent pair programming first
- Document everything in journal
- End-to-end tests validate decisions
- **Works from day one**

## The Meta Moment

> "The fact that you're talking to me through Zed right now, working on this very project? That's the meta moment right there." 🚀

We're building the very tool we're using to have this conversation.

---

**Last Updated**: 2025-01-19
**Status**: Whiteboard transcription complete
**Next Steps**: Start Phase 1 restructuring
