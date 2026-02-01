# Crow IDE vs VSCode ACP Extension: Comparative Analysis

**Date**: 2025-01-31
**Purpose**: Understand architectural similarities and differences for integration planning

## Executive Summary

**Key Insight**: Crow IDE and VSCode ACP extensions are **more alike than different**. Both are essentially React-based web UIs that communicate with ACP agents via WebSocket. The main difference is the **container**:
- **Crow IDE**: Standalone web app (React + Vite + Starlette server)
- **VSCode ACP**: VSCode extension using webview (React + VSCode Extension API)

This means we can **share 80-90% of the React components** between both implementations.

## Architecture Comparison

### Crow IDE (Standalone Web App)

```
┌─────────────────────────────────────────────────────────┐
│                    Browser                               │
│  ┌──────────────────────────────────────────────────┐  │
│  │         React Frontend (Vite)                     │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  │  │
│  │  │ Agent      │  │ File       │  │ Terminal    │  │  │
│  │  │ Panel      │  │ Explorer   │  │            │  │  │
│  │  └────────────┘  └────────────┘  └────────────┘  │  │
│  └──────────────────────────────────────────────────┘  │
│                         ↕ WebSocket                    │
│  ┌──────────────────────────────────────────────────┐  │
│  │         Starlette Server (Python)                │  │
│  │  - HTTP API endpoints                            │  │
│  │  - WebSocket proxy to ACP agents                 │  │
│  │  - Static file serving                           │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                         ↕ stdio-to-ws
┌─────────────────────────────────────────────────────────┐
│                    ACP Agent                            │
│  (crow, OpenHands, karla, claude-code, etc.)           │
└─────────────────────────────────────────────────────────┘
```

**Key Characteristics**:
- **Frontend**: React + TypeScript + Vite
- **Backend**: Python Starlette server
- **Communication**: WebSocket (via `use-acp` library)
- **Deployment**: Standalone web server (uvicorn)
- **URL**: http://127.0.0.1:8974/

### VSCode ACP Extension (Webview)

```
┌─────────────────────────────────────────────────────────┐
│                    VSCode                                │
│  ┌──────────────────────────────────────────────────┐  │
│  │         VSCode Extension Host                    │  │
│  │  ┌────────────────────────────────────────────┐  │  │
│  │  │     Webview Panel (React)                   │  │  │
│  │  │  ┌────────────┐  ┌────────────┐           │  │  │
│  │  │  │ Agent      │  │ Session    │           │  │  │
│  │  │  │ Panel      │  │ History    │           │  │  │
│  │  │  └────────────┘  └────────────┘           │  │  │
│  │  └────────────────────────────────────────────┘  │  │
│  │                     ↕ Extension API              │  │
│  │  ┌────────────────────────────────────────────┐  │  │
│  │  │     Extension Backend (TypeScript)          │  │  │
│  │  │  - Spawns ACP agent as subprocess           │  │  │
│  │  │  - Manages stdio/WebSocket bridge           │  │  │
│  │  │  - Handles VSCode API integration           │  │  │
│  │  └────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                         ↕ stdio
┌─────────────────────────────────────────────────────────┐
│                    ACP Agent                            │
│  (claude-code, gemini-cli, codex, etc.)                 │
└─────────────────────────────────────────────────────────┘
```

**Key Characteristics**:
- **Frontend**: React + TypeScript (webview)
- **Backend**: VSCode Extension API (TypeScript)
- **Communication**: stdio or WebSocket (via `use-acp` library)
- **Deployment**: VSCode extension (.vsix)
- **Integration**: VSCode native UI and APIs

## Component-Level Comparison

### Shared React Components (90% overlap)

Both implementations use **nearly identical React components**:

| Component | Crow IDE | VSCode ACP | Reusability |
|-----------|----------|------------|-------------|
| **Agent Panel** | `agent-panel.tsx` | Similar | ✅ 95% reusable |
| **Thread View** | `thread.tsx` | Similar | ✅ 95% reusable |
| **Markdown Renderer** | `markdown-renderer.tsx` | Similar | ✅ 100% reusable |
| **File Attachments** | `file-attachment.tsx` | Similar | ✅ 90% reusable |
| **Session History** | `session-history.tsx` | Similar | ✅ 95% reusable |
| **Tool Call Display** | `blocks.tsx` | Similar | ✅ 85% reusable |
| **Permission Requests** | `common.tsx` | Similar | ✅ 100% reusable |

### Key Dependencies (Identical)

Both use the **same core libraries**:

```json
{
  "@zed-industries/agent-client-protocol": "^0.4.5",
  "use-acp": "^0.2.5",
  "react": "^18.2.0",
  "react-dom": "^18.2.0",
  "react-markdown": "^10.1.0",
  "jotai": "^2.15.1"
}
```

**Critical Insight**: `use-acp` library provides **ACP protocol abstraction** that works in both standalone web apps AND VSCode webviews.

## Architectural Differences

### 1. Container/Hosting

**Crow IDE**:
- Runs in browser as standalone web app
- Full control over layout and styling
- Can use any web framework (Vite, Next.js, etc.)
- Deployed as web server

**VSCode ACP**:
- Runs in VSCode webview (iframe-like container)
- Constrained by VSCode webview API
- Must use VSCode extension build system
- Deployed as VSCode extension

### 2. Backend Communication

**Crow IDE**:
```typescript
// Direct WebSocket connection to agent
const acpClient = useAcpClient({
  url: 'ws://localhost:3000/message'
});
```

**VSCode ACP**:
```typescript
// Communication through VSCode extension backend
const acpClient = useAcpClient({
  // Uses VSCode extension message passing
  extensionHost: vscode
});
```

### 3. File System Access

**Crow IDE**:
- Custom HTTP API endpoints (`/api/files/*`)
- Direct file operations via backend server
- Full control over file system

**VSCode ACP**:
- VSCode Extension API (`vscode.workspace.fs`)
- Delegated to VSCode's file system providers
- Integrated with VSCode's file watchers

### 4. Terminal Integration

**Crow IDE**:
- Custom WebSocket terminal (xterm.js)
- Direct PTY spawning
- Full terminal control

**VSCode ACP**:
- VSCode Terminal API (`vscode.window.createTerminal`)
- Integrated with VSCode's terminal features
- Delegated to VSCode's terminal implementation

## Integration Strategy

### Option 1: Extract Shared Component Library

**Approach**: Create a shared React component library

```
┌─────────────────────────────────────────────────────────┐
│           @crow/acp-ui (Shared Component Library)       │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐        │
│  │ Agent      │  │ Thread     │  │ Markdown   │        │
│  │ Panel      │  │ View       │  │ Renderer   │        │
│  └────────────┘  └────────────┘  └────────────┘        │
└─────────────────────────────────────────────────────────┘
         ↕                           ↕
    ┌─────────────────┐      ┌─────────────────┐
    │ Crow IDE        │      │ VSCode ACP      │
    │ (Standalone)    │      │ (Extension)     │
    └─────────────────┘      └─────────────────┘
```

**Benefits**:
- Single source of truth for UI components
- Consistent UX across both platforms
- Easier maintenance and updates

**Implementation**:
```bash
# Create shared library
mkdir -p crow/acp-ui/packages/acp-ui

# Extract shared components
cp -r crow_ide/frontend/src/components/acp/* \
   crow/acp-ui/packages/acp-ui/src/

# Publish as npm package
cd crow/acp-ui/packages/acp-ui
npm publish
```

**Usage**:
```typescript
// In Crow IDE
import { AgentPanel, ThreadView } from '@crow/acp-ui';

// In VSCode ACP
import { AgentPanel, ThreadView } from '@crow/acp-ui';
```

### Option 2: Adapter Pattern

**Approach**: Keep components separate, use adapters for platform-specific code

```typescript
// Shared interface
interface AcpAdapter {
  connect(): Promise<void>;
  sendPrompt(prompt: string): Promise<void>;
  readFile(path: string): Promise<string>;
  writeFile(path: string, content: string): Promise<void>;
}

// Crow IDE adapter
class WebAdapter implements AcpAdapter {
  async readFile(path: string) {
    return fetch(`/api/files/read?path=${path}`)
      .then(r => r.json())
      .then(data => data.content);
  }
}

// VSCode adapter
class VscodeAdapter implements AcpAdapter {
  async readFile(path: string) {
    return vscode.workspace.fs.readFile(path)
      .then(content => content.toString());
  }
}
```

**Benefits**:
- Components remain platform-agnostic
- Easy to add new platforms (code-server, vscodium)
- Clear separation of concerns

### Option 3: Monorepo with Shared Frontend

**Approach**: Single frontend, multiple deployment targets

```
crow/
├── packages/
│   ├── acp-ui/           # Shared React components
│   ├── crow-ide/         # Standalone web app
│   ├── vscode-acp/       # VSCode extension
│   └── code-server-acp/  # Code-server extension
└── apps/
    ├── web/             # Vite app (Crow IDE)
    └── extension/       # VSCode extension
```

**Benefits**:
- Maximum code reuse
- Consistent testing across platforms
- Single development workflow

## Recommendations

### Short Term (Immediate)

1. **Extract ACP Panel Components**:
   - Create `@crow/acp-panel` package
   - Move `agent-panel.tsx`, `thread.tsx`, `blocks.tsx`
   - Test in both Crow IDE and VSCode

2. **Standardize on `use-acp`**:
   - Both already using it
   - Ensure consistent version
   - Share ACP client configuration

3. **Create Adapter Interface**:
   - Define `AcpAdapter` interface
   - Implement `WebAdapter` (Crow IDE)
   - Implement `VscodeAdapter` (VSCode)

### Medium Term (Next 2-4 weeks)

1. **Build Shared Component Library**:
   - Extract all reusable components
   - Publish as `@crow/acp-ui`
   - Create Storybook for visual testing

2. **Unify State Management**:
   - Both using Jotai
   - Share state atoms and logic
   - Create `@crow/acp-state` package

3. **Standardize Tool Call Display**:
   - Merge best practices from both
   - Create unified `ToolCallRenderer`
   - Improve status indicators

### Long Term (Next 2-3 months)

1. **Monorepo Structure**:
   - Migrate to Turborepo or Nx
   - Single frontend, multiple targets
   - Shared build and test infrastructure

2. **Code-Server Integration**:
   - Use shared components in code-server extension
   - Enable browser-based testing with Playwright
   - Support VSCodium builds

3. **Visual Planning Interface**:
   - Add TRAE Solo-like planning UI
   - Works in both standalone and VSCode
   - Real-time agent feedback

## Testing Strategy

### Shared Testing Approach

Both platforms can use **same testing approach**:

```bash
# 1. Test components in isolation
npm test -- @crow/acp-ui

# 2. Test in Crow IDE
cd crow-ide
uvicorn crow_ide.server:app --port 8974
playwright-mcp_browser_navigate --url http://127.0.0.1:8974/

# 3. Test in VSCode
code --install-extension crow-acp.vsix
# Manual testing in VSCode

# 4. Visual regression testing
zai-vision-server_ui_diff_check \
  --expected-image crow-ide.png \
  --actual-image vscode-acp.png \
  --prompt "Compare ACP panel layout and components"
```

### Playwright + zai-vision Testing

**Critical Advantage**: Both can be tested with Playwright!

```typescript
// Test Crow IDE
await page.goto('http://127.0.0.1:8974/');
await screenshot({ path: 'crow-ide.png' });

// Test VSCode ACP (in code-server)
await page.goto('http://localhost:8942/');
await screenshot({ path: 'vscode-acp.png' });

// Compare with zai-vision
await zaiVisionAnalyze({
  prompt: 'Compare ACP panel implementation',
  images: ['crow-ide.png', 'vscode-acp.png']
});
```

## Conclusion

**Key Takeaway**: Crow IDE and VSCode ACP are **90% identical** at the component level. The main difference is the container (standalone web app vs VSCode extension).

**Strategic Decision**: Extract shared components into `@crow/acp-ui` library. This enables:
- Single codebase for ACP UI
- Consistent UX across platforms
- Faster development
- Easier maintenance
- Better testing with Playwright + zai-vision

**Next Steps**:
1. Extract ACP panel components to shared library
2. Create adapter interface for platform-specific code
3. Test in both Crow IDE and VSCode
4. Plan for code-server integration

**Ultimate Goal**: A single ACP UI component library that works in:
- Standalone web apps (Crow IDE)
- VSCode extensions (claude-code-acp)
- Code-server (vscodium)
- Future platforms (JetBrains, Neovim, etc.)

---

**Date**: 2025-01-31
**Status**: Ready for implementation
**Priority**: High (enables faster development across all platforms)
