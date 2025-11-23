# Crow - AI Coding Agent Platform

## Overview

Crow is an AI-powered coding assistant platform with a Rust backend and React frontend. It's based on [OpenCode](https://github.com/opencode-ai/opencode) - use OpenCode as the primary reference implementation for understanding architecture decisions and tool implementations.

## Quick Start

### Backend (crow-backend)
```bash
cd crow-backend
cargo build --bin crow-serve --features server
cd ../test-dummy  # or your project directory
../crow-backend/target/debug/crow-serve
# Runs on http://localhost:7070
```

### Frontend (crow-frontend)
```bash
cd crow-frontend
npm install
npm run dev
# Runs on http://localhost:5173
```

## Architecture

### Backend (Rust)
- **Framework**: Axum for HTTP/SSE
- **LLM Providers**: OpenAI-compatible APIs (configurable via `.crow/config.jsonc`)
- **Storage**: File-based in `.crow/` directory (sessions, messages, parts)

Key directories:
- `packages/api/src/tools/` - Tool implementations (read, write, edit, bash, todowrite, etc.)
- `packages/api/src/agent/` - Agent executor and streaming logic
- `packages/api/src/providers/` - LLM client implementations
- `packages/api/src/prompts/` - System prompts (codex.txt)
- `packages/api/src/bin/crow-serve.rs` - Server entry point

### Frontend (React/TypeScript)
- **Framework**: React + Vite
- **Styling**: Inline styles (dark theme)
- **State**: React hooks, SSE for real-time updates

Key files:
- `src/App.tsx` - Main app, routing, message state
- `src/components/ChatView.tsx` - Message display, markdown rendering
- `src/hooks/useEventStream.ts` - API calls and SSE handling
- `src/types.ts` - TypeScript interfaces

## Data Flow

1. User sends message via frontend
2. Frontend POSTs to `/session/{id}/message` with streaming
3. Backend creates user message, invokes LLM
4. LLM streams response with tool calls
5. Backend executes tools, streams deltas via SSE
6. Frontend displays streaming text and tool results
7. Messages persisted to `.crow/storage/`

## Tool System

Tools are defined in `crow-backend/packages/api/src/tools/`:

| Tool | File | Purpose |
|------|------|---------|
| read | read.rs | Read file contents |
| write | write.rs | Write files |
| edit | edit.rs | String replacement in files |
| bash | bash.rs | Execute shell commands |
| glob | glob.rs | Find files by pattern |
| grep | grep.rs | Search file contents |
| todowrite | todowrite.rs | Manage task lists |
| todoread | todoread.rs | Read task lists |
| websearch | websearch.rs | Internet search |
| webfetch | webfetch.rs | Fetch URL content |
| task | task.rs | Spawn sub-agents |

Tools receive `ToolContext` with session_id, working directory, etc.

## Configuration

Project config: `.crow/config.jsonc`
```jsonc
{
  "provider": {
    "name": "openai-compatible",
    "model": "your-model",
    "api_key": "your-key",
    "base_url": "http://localhost:8080/v1"
  }
}
```

## Storage Structure

```
.crow/
├── config.jsonc          # Project configuration
├── sessions/             # Session markdown logs
│   └── {session_id}.md
└── storage/
    ├── session/          # Session metadata JSON
    ├── message/          # Message metadata JSON
    └── part/             # Message parts (text, tools) JSON
```

## Development Notes

### Building
```bash
# Backend with server features
cd crow-backend && cargo build --bin crow-serve --features server

# Frontend
cd crow-frontend && npm run dev
```

### Key Patterns from OpenCode

1. **Message Parts**: Messages contain parts (text, tool calls, thinking)
2. **Tool State**: pending → running → completed
3. **Session Scoping**: Tools use `ctx.session_id` not LLM-provided IDs
4. **Streaming**: SSE for real-time updates, accumulate deltas

### Current Issues / TODOs

- Thinking tokens display as blockquote (simplified from collapsible)
- Todo persistence uses XDG paths (`~/.local/share/crow/storage/todo/`)
- Sub-agent (task tool) needs testing

## Reference

- **OpenCode**: https://github.com/opencode-ai/opencode - The reference implementation
- Check `crow-backend/docs/` for planning documents and analysis
