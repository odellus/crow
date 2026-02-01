# Crow Monoextension Strategy: Purple TRAE Solo for VSCode

**Date**: 2025-01-31
**Status**: Strategic Planning
**Goal**: Create a monoextension that transforms VSCode into a purple TRAE Solo-like experience

## Vision Statement

**What We're Building**: A single VSCode extension ("monoextension") that completely transforms the VSCode experience into a purple-themed, TRAE Solo-inspired AI coding environment.

**Key Insight**: We're not forking VSCode. We're building a **comprehensive extension** that:
- Reskins the UI with purple TRAE Solo theme
- Provides full ACP agent integration
- Works with standard Python backend tools
- Can be installed by anyone (open source)
- Offers a CLI alternative (`uv tool install crow`)

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    VSCode (User Interface)                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         Crow Monoextension (Purple TRAE Solo)        │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐      │  │
│  │  │ Purple     │  │ Agent      │  │ Visual     │      │  │
│  │  │ Theme      │  │ Panel      │  │ Planning   │      │  │
│  │  └────────────┘  └────────────┘  └────────────┘      │  │
│  └──────────────────────────────────────────────────────┘  │
│                         ↕ VSCode Extension API              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         Extension Backend (TypeScript)               │  │
│  │  - Spawns Python backend as subprocess               │  │
│  │  - Manages stdio/WebSocket bridge                    │  │
│  │  - Handles VSCode API integration                    │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                         ↕ WebSocket/stdio
┌─────────────────────────────────────────────────────────────┐
│              Python Backend (Separate Process)             │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐          │
│  │ Crow       │  │ OpenHands  │  │ Standard   │          │
│  │ Agent      │  │ SDK        │  │ Tools      │          │
│  └────────────┘  └────────────┘  └────────────┘          │
└─────────────────────────────────────────────────────────────┘
```

## Component Breakdown

### 1. Crow Monoextension (VSCode Extension)

**Location**: `~/src/projects/orchestrator-project/vscode-acp/`
**Type**: VSCode Extension (TypeScript)
**Purpose**: Transform VSCode into purple TRAE Solo

**Key Features**:
- **Purple TRAE Solo Theme**: Custom color scheme based on TRAE SOLO design
  - Primary: `#9333EA` (purple)
  - Accent: `#00FF9F` (neon green, from TRAE SOLO)
  - Background: `#121212` (near-black)
- **Agent Panel**: Reuses crow-ide React components
- **Visual Planning Interface**: TRAE Solo-style plan-first workflow
- **Full ACP Integration**: Supports all ACP agents
- **VSCode Native**: Uses VSCode webview API

**How It Works**:
```typescript
// Extension spawns Python backend
const pythonBackend = spawn('python', [
  '-m', 'crow.backend',
  '--port', '3030'
]);

// Webview communicates with backend
const acpClient = useAcpClient({
  url: 'ws://localhost:3030/message'
});
```

### 2. Python Backend (Separate Process)

**Location**: `~/src/projects/orchestrator-project/crow/backend/`
**Type**: Python server
**Purpose**: Handle agent logic, file operations, tool execution

**Key Features**:
- **ACP Server**: Implements ACP protocol
- **Agent Orchestration**: Manages Crow, OpenHands, etc.
- **File Operations**: Standard Python tools (os, pathlib, etc.)
- **Tool Execution**: Can run any command-line tool
- **WebSocket Server**: Real-time communication with extension

**Why Separate Backend?**
- **Flexibility**: Can use any Python tools/libraries
- **Security**: Runs in separate process (isolated from VSCode)
- **Compatibility**: Works with any VSCode installation
- **Upgradability**: Can update backend without updating extension

### 3. Crow CLI (Alternative Interface)

**Location**: `~/src/projects/orchestrator-project/crow/cli/`
**Type**: Python CLI tool
**Purpose**: Programmatic way to interact with Crow agent

**Installation**:
```bash
uv tool install crow
```

**Usage**:
```bash
# Start agent
crow agent start

# Send prompt
crow agent prompt "Create a hello world app"

# List sessions
crow session list

# Run in headless mode
crow agent run --headless
```

**Why CLI?**
- **Scripting**: Automate agent workflows
- **Headless Operation**: No UI needed
- **CI/CD Integration**: Use in pipelines
- **Power Users**: Programmatic control

## Design System: Purple TRAE Solo

### Color Palette

Based on TRAE SOLO design with purple twist:

```css
/* Primary Colors */
--crow-primary: #9333EA;      /* Purple (main brand) */
--crow-accent: #00FF9F;       /* Neon green (from TRAE SOLO) */
--crow-bg: #121212;           /* Near-black (background) */
--crow-bg-secondary: #1E1E1E; /* Dark gray (panels) */

/* Text Colors */
--crow-text-primary: #FFFFFF;  /* White */
--crow-text-secondary: #B0B0B0; /* Light gray */

/* Status Colors */
--crow-success: #00FF9F;      /* Neon green */
--crow-warning: #F59E0B;      /* Amber */
--crow-error: #EF4444;        /* Red */
--crow-info: #3B82F6;         /* Blue */
```

### Typography

```css
/* Headers */
--font-heading: 'Inter', system-ui, sans-serif;
--font-heading-weight: 700;

/* Body */
--font-body: 'Inter', system-ui, sans-serif;
--font-body-weight: 400;

/* Code */
--font-code: 'JetBrains Mono', 'Fira Code', monospace;
```

### Components

**Agent Panel**:
- Purple gradient header
- Neon green accent buttons
- Dark background
- Rounded corners (8px)

**Tool Calls**:
- Purple icons
- Status indicators (neon green for success)
- Expandable details

**Planning Interface**:
- Purple cards for plan items
- Neon green for completed items
- Drag-and-drop reordering

## Implementation Strategy

### Phase 1: Component Extraction (Week 1-2)

**Goal**: Extract shared React components from crow-ide

```bash
# Create shared component library
mkdir -p crow/packages/acp-ui

# Extract components
cp -r crow_ide/frontend/src/components/acp/* \
   crow/packages/acp-ui/src/

# Build as npm package
cd crow/packages/acp-ui
npm run build
npm publish
```

**Deliverables**:
- `@crow/acp-ui` package
- Storybook for visual testing
- Documentation

### Phase 2: VSCode Extension Development (Week 3-4)

**Goal**: Build monoextension using shared components

```bash
# Create extension
cd vscode-acp

# Install shared components
npm install @crow/acp-ui

# Build extension
npm run build
```

**Key Tasks**:
1. Implement purple TRAE Solo theme
2. Integrate @crow/acp-ui components
3. Create Python backend bridge
4. Add visual planning interface
5. Test with Playwright

**Deliverables**:
- Working VSCode extension
- Purple TRAE Solo theme
- Python backend server
- E2E tests passing

### Phase 3: Python Backend (Week 5-6)

**Goal**: Build robust Python backend server

```bash
# Create backend
cd crow/backend

# Implement ACP server
python -m crow.backend --port 3030
```

**Key Features**:
- ACP protocol implementation
- Agent orchestration
- File operations
- Tool execution
- WebSocket server

**Deliverables**:
- Python backend server
- ACP protocol compliance
- Agent integration tests

### Phase 4: CLI Tool (Week 7-8)

**Goal**: Build CLI tool for power users

```bash
# Create CLI
cd crow/cli

# Install as uv tool
uv tool install crow
```

**Key Features**:
- Agent start/stop
- Prompt submission
- Session management
- Headless mode
- Configuration

**Deliverables**:
- CLI tool
- Documentation
- Examples

## Testing Strategy

### Playwright + zai-vision Testing

**Critical Advantage**: Everything runs in browser (code-server)

```bash
# 1. Start code-server with extension
code-server --install-extension crow-monoextension.vsix

# 2. Navigate with Playwright
playwright-mcp_browser_navigate --url http://localhost:8942/

# 3. Take screenshot
playwright-mcp_browser_take_screenshot --filename purple-trae-solo.png

# 4. Analyze with zai-vision
zai-vision-server_analyze_image \
  --image-source purple-trae-solo.png \
  --prompt "Verify purple TRAE Solo theme, agent panel, and planning interface"
```

### E2E Testing

```typescript
// Test agent interaction
test('agent creates file', async () => {
  await page.click('[data-testid="agent-input"]');
  await page.fill('Create hello.py');
  await page.click('[data-testid="send-button"]');
  
  // Verify tool call
  await expect(page.locator('[data-testid="tool-call"]')).toBeVisible();
  
  // Verify file created
  const content = await fs.readFile('hello.py', 'utf-8');
  expect(content).toContain('print("Hello World")');
});
```

## Deployment Strategy

### Option 1: VSCode Marketplace (Open Source)

**Pros**:
- Easy discovery
- Auto-updates
- Large audience

**Cons**:
- Microsoft review process
- "Kiss of death for a startup" (your concern)

### Option 2: Direct Distribution (Open Source)

**Pros**:
- No Microsoft gatekeeping
- Full control
- Can update anytime

**Cons**:
- Manual installation
- No auto-updates

**Installation**:
```bash
# Install from VSIX
code --install-extension crow-monoextension.vsix

# Or from GitHub
code --install-extension https://github.com/crow/crow-monoextension/releases/latest/download/crow-monoextension.vsix
```

### Option 3: Hybrid Approach

**Strategy**: Offer both options

```bash
# For casual users (VSCode Marketplace)
ext install crow-monoextension

# For power users (direct download)
code --install-extension crow-monoextension.vsix

# For developers (build from source)
git clone https://github.com/crow/crow-monoextension
cd crow-monoextension
npm run build
code --install-extension crow-monoextension.vsix
```

## Monorepo Structure

```
crow/
├── packages/
│   ├── acp-ui/              # Shared React components
│   ├── vscode-extension/    # VSCode monoextension
│   ├── python-backend/      # Python backend server
│   └── cli-tool/           # CLI tool
├── apps/
│   ├── crow-ide/           # Standalone web app
│   └── docs/               # Documentation
├── tools/
│   ├── playwright-tests/   # E2E tests
│   └── build-scripts/      # Build automation
└── pnpm-workspace.yaml
```

## Success Criteria

### Phase 1 Success
- [ ] @crow/acp-ui package published
- [ ] Storybook working
- [ ] Components tested in isolation

### Phase 2 Success
- [ ] VSCode extension installs without errors
- [ ] Purple TRAE Solo theme applied
- [ ] Agent panel displays correctly
- [ ] Can send prompts and receive responses
- [ ] Tool calls display properly

### Phase 3 Success
- [ ] Python backend starts without errors
- [ ] ACP protocol working
- [ ] Can spawn Crow agent
- [ ] File operations work
- [ ] WebSocket communication stable

### Phase 4 Success
- [ ] CLI tool installs with `uv tool install crow`
- [ ] Can start/stop agent
- [ ] Can send prompts
- [ ] Headless mode works
- [ ] Documentation complete

## Risks & Mitigations

### Risk 1: VSCode Extension API Limitations

**Problem**: VSCode webview API has limitations

**Mitigation**:
- Use webview for complex UI
- Use VSCode native API for file operations
- Test thoroughly with Playwright

### Risk 2: Python Backend Complexity

**Problem**: Managing separate process adds complexity

**Mitigation**:
- Auto-spawn backend on extension activation
- Robust error handling
- Clear logging

### Risk 3: Performance

**Problem**: Webview can be slow

**Mitigation**:
- Lazy loading
- Code splitting
- Optimize React rendering

### Risk 4: User Adoption

**Problem**: Users might not want to install extension

**Mitigation**:
- Offer standalone web app (crow-ide)
- Offer CLI tool
- Make installation dead simple

## Next Steps

1. **Extract Components**: Start with @crow/acp-ui
2. **Design Purple Theme**: Create color palette and components
3. **Build Extension Prototype**: Get basic extension working
4. **Test with Playwright**: Verify in code-server
5. **Iterate**: Refine based on testing

## Conclusion

**The Vision**: A purple TRAE Solo for VSCode that's open source, flexible, and powerful.

**The Strategy**: Reuse crow-ide components, build monoextension, separate Python backend, offer CLI alternative.

**The Timeline**: 8 weeks to full working system.

**The Goal**: Transcend code with visual interfaces, real-time feedback, and powerful AI agents.

---

**Date**: 2025-01-31
**Status**: Ready for Implementation
**Next Review**: 2025-02-07
