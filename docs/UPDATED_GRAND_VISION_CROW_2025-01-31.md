# Updated Grand Vision: Crow - Open Source TRAE Solo

**Date**: 2025-01-31
**Status**: Active Development
**URL**: http://127.0.0.1:8974/

## Executive Summary

Crow is an open-source implementation of a TRAE Solo-like visual, plan-first, multi-agent coding environment. We're building a complete ecosystem that enables users to interact with AI coding agents through visual interfaces, real-time feedback, and browser-based development environments.

**Core Philosophy**: Transcend code - users should interact with visual planning interfaces, real-time agent feedback, and DOM-mapped code elements, not raw code.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    User Interface Layer                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ crow_ide     │  │ code-server  │  │ vscodium     │      │
│  │ (React/TS)   │  │ + ACP ext    │  │ (VSCode fork)│      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                            ↕ ACP Protocol
┌─────────────────────────────────────────────────────────────┐
│                    Agent Layer                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ crow         │  │ OpenHands    │  │ Other ACP    │      │
│  │ (CLI agent)  │  │ (Python SDK) │  │ agents       │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                            ↕ REST API / WebSockets
┌─────────────────────────────────────────────────────────────┐
│                    Testing & Automation Layer                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Playwright   │  │ zai-vision   │  │ MCP servers  │      │
│  │ (Browser)    │  │ (Visual AI)  │  │ (Extensions) │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## Component Definitions

### 1. Crow (CLI Agent)
**Location**: `~/src/projects/orchestrator-project/crow/`
**Type**: Python ACP agent (CLI, not TUI)
**Purpose**: Core AI coding agent using OpenHands SDK

**Key Characteristics**:
- Command-line interface (no TUI)
- ACP (Agent Client Protocol) server
- Built on OpenHands SDK
- Supports iterative refinement loops
- Delegation capabilities
- Custom tool implementations

**How to Run**:
```bash
cd ~/src/projects/orchestrator-project
uv --project crow run crow acp
```

**ACP Connection**:
```bash
npx stdio-to-ws "uv run --project crow crow acp" --port 3027
```

### 2. Crow IDE (Frontend)
**Location**: `~/src/projects/orchestrator-project/crow_ide/`
**Type**: React + TypeScript web application
**Purpose**: Visual IDE interface for ACP agents

**Key Characteristics**:
- React + TypeScript + Vite frontend
- Python Starlette backend
- Full ACP integration using `use-acp` library
- File explorer, code editor, terminal
- Agent chat panel with real-time streaming
- Session history (SQLite persistence)
- 31/31 tests passing (including 4 Playwright E2E tests)

**How to Run**:
```bash
cd ~/src/projects/orchestrator-project/crow_ide

# Install dependencies
uv sync

# Build frontend
cd frontend
pnpm install
pnpm build
cd ..

# Start server
source .venv/bin/activate
uvicorn crow_ide.server:app --port 8974
```

**Access URL**: http://127.0.0.1:8974/

**Testing with Playwright**:
```bash
# Navigate to URL
playwright-mcp_browser_navigate --url http://127.0.0.1:8974/

# Take screenshot
playwright-mcp_browser_take_screenshot --filename crow-ide-test.png

# Analyze with zai-vision
zai-vision-server_analyze_image --image_source crow-ide-test.png --prompt "Analyze UI layout, components, and functionality"
```

### 3. Code-Server + ACP Extension
**Location**: `~/src/projects/orchestrator-project/` (integration point)
**Type**: VSCode web server + ACP extension
**Purpose**: Browser-based VSCode with ACP integration

**Key Characteristics**:
- Open-source VSCode (code-server)
- ACP extension for agent integration
- Runs in browser (no Electron)
- Enables testing with Playwright
- Full VSCode feature set

**How to Run**:
```bash
# Currently running on port 8942
code-server --port 8942
```

**Access URL**: http://localhost:8942/

**Development Mode** (TODO):
```bash
# Build code-server from source
cd /path/to/code-server
npm run build
npm run start:dev

# Install ACP extension
code-server --install-extension path/to/acp-extension
```

### 4. VSCodium (VSCode Fork)
**Purpose**: Open-source VSCode without proprietary Microsoft code
**Role**: Pipeline to wash out proprietary plugins and Electron requirements

**Why VSCodium?**:
- Fully open-source VSCode build
- Removes Microsoft telemetry and branding
- Enables browser-based testing
- Foundation for custom TRAE Solo builds

## Testing & Development Methodology

### Core Principle: Autonomous Development with Visual Feedback

We use Playwright + zai-vision to test and develop Crow autonomously:

1. **Playwright MCP**: Browser automation for testing
2. **zai-vision Server**: Visual AI analysis of UI/UX
3. **Browser-based testing**: Everything runs in browser (no Electron)
4. **Visual verification**: Screenshots analyzed by AI for quality assurance

### Testing Workflow

```bash
# 1. Start Crow IDE
cd ~/src/projects/orchestrator-project/crow_ide
uvicorn crow_ide.server:app --port 8974

# 2. Navigate with Playwright
playwright-mcp_browser_navigate --url http://127.0.0.1:8974/

# 3. Take screenshot
playwright-mcp_browser_take_screenshot --filename test.png --fullPage=true

# 4. Analyze with zai-vision
zai-vision-server_analyze_image \
  --image_source test.png \
  --prompt "Analyze UI layout, components, functionality, and compare to TRAE SOLO design"

# 5. Test agent interaction
playwright-mcp_browser_click --element "New session button" --ref <ref>
playwright-mcp_browser_type --element "Agent input" --ref <ref> --text "Create a hello world app"
```

### URL Convention

**Development URL**: http://127.0.0.1:8974/
**Testing URL**: http://localhost:8974/
**Production URL**: TBD (deployment target)

## Product Vision: Two-Tier Offering

### Tier 1: Self-Hosted (Free & Open Source)

**Target**: Developers who want to run locally
**Features**:
- Complete Crow IDE (React + TypeScript)
- Crow CLI agent (Python + OpenHands SDK)
- Full ACP protocol support
- Browser-based development
- Playwright + zai-vision testing tools
- Documentation and setup guides

**Distribution**:
- GitHub repository (MIT license)
- npm packages for frontend
- PyPI packages for Python components
- Docker images for easy deployment

**Philosophy**: "Give it away if you want to run it"

### Tier 2: Cloud-Hosted (Paid Service)

**Target**: Teams who want hosted infrastructure
**Features**:
- Everything in Tier 1, plus:
- Cloud workspace (like Overleaf or Claude Code Web)
- Managed agent instances (EC2-like isolation)
- Persistent storage and collaboration
- Team management and access control
- Usage analytics and monitoring
- Priority support

**Key Differentiator**: **No API provider restrictions**
- Users can make API calls to any LLM provider
- No "uber restrictive shit" (unlike some competitors)
- Full control over model selection and costs
- Bring your own API keys or use our managed endpoints

**Philosophy**: "Like a little EC2 instance just for your agent"

## Technical Decisions & Rationale

### Why Python + OpenHands for Agent Core?

**Decision**: Keep agent core in Python using OpenHands SDK

**Rationale**:
- Mature, battle-tested framework
- Native ACP support via `acp-python-sdk`
- Superior AI/ML libraries
- We know Python deeply (don't underestimate this advantage)
- Agent reasoning, planning, and tool orchestration are Python strengths

**Trade-off**: 80-90% of value add is in TypeScript/JavaScript (UI layer)

### Why TypeScript for Frontend/Extension Layer?

**Decision**: Build UI layer in TypeScript

**Rationale**:
- VSCode extensions are 100% TypeScript
- Web ecosystem (React, Vue, Svelte) is TypeScript-first
- Real-time communication (WebSockets, SSE) requires JS
- Browser automation (Playwright) is TypeScript-first
- Visual feedback systems need web technologies

**Learning Curve**: 2-3 weeks to get productive, 1-2 months to be comfortable

### Why Hybrid Architecture?

**Decision**: Python core + TypeScript frontend communicating via REST/WebSocket

**Rationale**:
- Leverage strengths of each ecosystem
- Clean separation of concerns
- Python for agent logic, TypeScript for UI/UX
- ACP protocol as integration layer
- Enables independent development and testing

### Why Browser-Based Development?

**Decision**: Everything runs in browser, no Electron

**Rationale**:
- Enables Playwright automation for testing
- zai-vision can analyze UI/UX
- No proprietary Microsoft code (VSCodium)
- Cross-platform without native dependencies
- Easier deployment and updates
- Aligns with "transcend code" philosophy

## Current Status & Next Steps

### Completed ✅

1. **Crow CLI Agent**: Functional ACP agent using OpenHands SDK
2. **Crow IDE**: Complete React + TypeScript frontend with 31/31 tests passing
3. **ACP Integration**: Full ACP protocol support using `use-acp` library
4. **Testing Infrastructure**: Playwright + zai-vision for autonomous testing
5. **Code-Server Integration**: Running on localhost:8942

### In Progress 🚧

1. **OpenHands ACP Integration**: Somewhat janky, doesn't handle pause/interrupt well
2. **Code-Server Dev Mode**: Need to build from source, understand logging
3. **VSCodium Integration**: Pipeline to wash out proprietary code
4. **Visual Planning Interface**: TRAE Solo-like plan-first workflow

### Next Steps 📋

1. **Fix OpenHands ACP Issues**:
   - Handle pause/interrupt gracefully
   - Improve tool call display
   - Better error handling

2. **Build Custom ACP Implementation**:
   - Full ACP spec compliance
   - Custom OpenHands agents
   - Delegation support
   - Iterative refinement loops

3. **Code-Server Development**:
   - Build from source
   - Understand logging infrastructure
   - Install ACP extension manually
   - Create development workflow

4. **Visual Planning Interface**:
   - TRAE Solo-inspired design
   - Dark theme with neon accents (#00FF9F)
   - Real-time agent feedback
   - DOM-mapped code elements

5. **Documentation**:
   - Getting started guides
   - API documentation
   - Architecture diagrams
   - Video tutorials

## Known Issues & Limitations

### OpenHands ACP Agent Issues

**Problem**: OpenHands ACP agent doesn't expect to be paused or interrupted

**Impact**: Janky behavior when user tries to interact during agent execution

**Solution**: Building custom ACP implementation with proper pause/interrupt handling

**Status**: DO NOT attempt to fix current OpenHands - we're building our own ACP

### Tool Call Display

**Problem**: Tool calls in crow-ide have "a bit to be desired"

**Impact**: Harder to understand what agent is doing

**Solution**: Improve tool call visualization in frontend, better status indicators

**Reference**: Zed has most complete ACP integration (except loading old sessions)

### Session Persistence

**Problem**: Some ACP clients don't support loading old sessions (Zed, etc.)

**Impact**: Can't resume previous conversations

**Solution**: Implement full session persistence in our ACP implementation

## Design Inspiration

### TRAE SOLO

**What We're Borrowing**:
- Visual, plan-first workflow
- Dark theme with neon green accents (#00FF9F)
- Real-time agent feedback
- Parallel agent execution
- "Speak Your Requirements" (voice input)
- All-in-one workspace concept

**What We're Doing Differently**:
- Open-source (not proprietary)
- No API provider restrictions
- Browser-based (not Electron)
- Python agent core (not pure TypeScript)

### Pi-Mono

**What We're Borrowing**:
- Extensible architecture
- Primitives, not features
- TypeScript extension system
- Minimal system prompt

**What We're Doing Differently**:
- Native ACP support (pi-acp is MVP with limited features)
- Python agent core (pi is pure TypeScript)
- Visual IDE interface (pi is TUI-only)

### Zed ACP Integration

**What We're Borrowing**:
- Most complete ACP implementation
- Excellent tool call display
- Clean UI/UX

**What We're Doing Differently**:
- Browser-based (Zed is native)
- Open-source (Zed is GPL)
- Python agent core (Zed is Rust)

## References & Resources

### Documentation
- `~/src/projects/orchestrator-project/crow/docs/openhands_book` - OpenHands SDK reference
- `~/src/projects/orchestrator-project/AGENTS.md` - Development rules and workflows
- `~/src/projects/orchestrator-project/crow_ide/README.md` - Crow IDE documentation

### Key Files
- `~/src/projects/orchestrator-project/crow_ide/PROGRESS.md` - Crow IDE development progress
- `~/src/projects/orchestrator-project/crow_ide/CROW_GRAND_VISION.md` - Original vision document
- `~/src/projects/orchestrator-project/crow_ide/DISTILLATION_DESIGN.md` - Agent distillation design

### Testing URLs
- Crow IDE: http://127.0.0.1:8974/
- Code-Server: http://localhost:8942/

### Commands Reference

**Crow Agent**:
```bash
cd ~/src/projects/orchestrator-project
uv --project crow run crow acp
```

**Crow IDE**:
```bash
cd ~/src/projects/orchestrator-project/crow_ide
uv sync
cd frontend && pnpm install && pnpm build && cd ..
source .venv/bin/activate
uvicorn crow_ide.server:app --port 8974
```

**Testing**:
```bash
playwright-mcp_browser_navigate --url http://127.0.0.1:8974/
playwright-mcp_browser_take_screenshot --filename test.png
zai-vision-server_analyze_image --image_source test.png --prompt "Analyze UI"
```

## Conclusion

Crow is not just another coding agent - it's a complete reimagining of how developers interact with AI. By combining the power of Python-based agents with modern web technologies, we're creating an open-source alternative to TRAE Solo that respects user freedom and enables true visual, autonomous development.

**The Goal**: Transcend code. Users should interact with visual interfaces, real-time feedback, and intelligent agents - not raw code.

**The Method**: Build it ourselves, test it autonomously with Playwright + zai-vision, and give it away for free while offering a premium hosted experience for those who want it.

**The Future**: An open-source ecosystem where anyone can run powerful AI coding agents locally, or pay for managed infrastructure that doesn't restrict their choices.

---

**Last Updated**: 2025-01-31
**Next Review**: 2025-02-07
**Status**: Active Development
