# Crow - Rust Agent Framework

**OpenCode in Rust**: A fast, embeddable agent framework with web UI.

## Quick Start

### Launch Crow Web UI

From anywhere:

```bash
cd /path/to/crow
./crow          # Starts on port 8080
./crow 3000     # Starts on custom port
```

Or from the web directory:

```bash
cd packages/web
dx serve --port 8080 --platform server
```

Then open `http://localhost:8080` in your browser.

## What's Working ✅

- **Web UI**: Dark-themed interface with Tailwind CSS
- **Session Management**: Create, list, and view sessions
- **Chat Interface**: Send messages and receive agent responses  
- **API Backend**: REST endpoints on same server
- **12 Tools**: Bash, Read, Write, Edit, Glob, Grep, List, Task, Patch, WebFetch, TodoWrite, WorkCompleted
- **LLM Integration**: Moonshot AI (kimi-k2-thinking with 262k context)
- **Storage**: XDG-compliant (`~/.local/share/crow/`)
- **Subagents**: Task tool spawns child sessions
- **Auth**: Reads from `~/.local/share/crow/auth.json`

## What's Next ⏭️

Current state: **UI renders, navigation not wired**

1. **Wire up navigation** - Session clicks and "New Session" button
2. **Message display** - Show conversation history
3. **Tool rendering** - Dynamic tool output display
4. **File references** - @ mentions for attaching files
5. **Streaming** - Real-time message updates via WebSocket
6. **Single binary** - Embed web UI into `crow` executable

See `CROW_WEB_UI_PROGRESS.md` for detailed status.

## Architecture

```
crow/
├── packages/
│   ├── api/          # Backend: REST API, tools, agents, LLM
│   ├── ui/           # Shared components (Chat, Navbar, etc.)
│   └── web/          # Dioxus fullstack web app
├── crow              # Launcher script
└── README.md
```

## Development

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Dioxus CLI
cargo install dioxus-cli --version 0.7.1

# Install Tailwind (handled automatically by Dioxus)
```

### Run Development Server

```bash
cd packages/web
dx serve --port 8080 --platform server
```

Hot reload enabled - edit code and see changes instantly.

### Build for Production

```bash
cd packages/web
dx build --release --platform server
```

Output: `../../target/dx/web/release/web/`

### Project Structure

- **api**: Core agent framework
  - `src/tools/`: 12 tool implementations
  - `src/agent/`: Agent executor and registry
  - `src/providers/`: LLM provider (Moonshot)
  - `src/session/`: Session and message storage
  - `src/server.rs`: REST API endpoints

- **web**: Dioxus fullstack UI
  - `src/views/sessions.rs`: Session list with sidebar
  - `src/views/session_detail.rs`: Chat view with tabs
  - `tailwind.css`: Tailwind input file (auto-compiled)
  - `Dioxus.toml`: Dioxus configuration

- **ui**: Shared components
  - `chat.rs`: Chat interface with message rendering
  - `navbar.rs`, `hero.rs`, etc.

## Configuration

### Auth (Moonshot API Key)

Create `~/.local/share/crow/auth.json`:

```json
{
  "moonshotai": {
    "type": "api",
    "key": "sk-YOUR_API_KEY_HERE"
  }
}
```

Or use environment variable:

```bash
export MOONSHOT_API_KEY="sk-YOUR_API_KEY_HERE"
```

### Dioxus Config

`packages/web/Dioxus.toml`:

```toml
[application]
name = "web"
default_platform = "web"
asset_dir = "assets"

[web.app]
title = "Crow"

[web.resource]
style = ["/tailwind.css"]  # Auto-compiled by Dioxus

[web.resource.dev]
style = []
script = []
```

## API Endpoints

All endpoints on `http://localhost:8080`:

- `GET /session` - List all sessions
- `POST /session/create` - Create new session
- `GET /session/:id` - Get session details
- `POST /session/:id/message` - Send message
- `GET /session/:id/message` - List messages
- `POST /api/send_message` - Quick message (auto-creates session)

## Tech Stack

- **Framework**: Dioxus 0.7 (Rust fullstack)
- **Backend**: Axum web framework
- **Styling**: Tailwind CSS (auto-compiled)
- **State**: Dioxus Signals
- **Storage**: File-based with atomic writes
- **LLM**: Moonshot AI (Kimi models)

## Comparison to OpenCode

| Feature | OpenCode (TypeScript) | Crow (Rust) |
|---------|----------------------|-------------|
| Agent execution | ✅ | ✅ |
| 12 core tools | ✅ | ✅ |
| Web UI | TUI (terminal) | Web (browser) |
| Storage | XDG directories | XDG directories |
| Subagents | ✅ Task tool | ✅ Task tool |
| Streaming | ✅ WebSocket | ⏭️ Planned |
| File refs | ✅ @ mentions | ⏭️ Planned |
| Themes | ✅ 23 themes | ⏭️ Planned |

## Troubleshooting

### "No CSS styling"

- Ensure Tailwind compiled: Check `packages/web/assets/main.css` exists
- Hard refresh: `Ctrl+Shift+R` (or `Cmd+Shift+R`)
- Clear browser cache

### "Build errors"

```bash
cd /path/to/crow
cargo clean
cd packages/web
dx build --release --platform server
```

### "Port already in use"

```bash
# Find process using port 8080
lsof -i :8080
# Kill it
kill -9 <PID>
# Or use different port
./crow 3000
```

## Contributing

Currently in active development. The web UI needs:

1. Navigation wiring (session clicks)
2. Message history display
3. Tool output rendering
4. File attachment UI
5. WebSocket streaming

See `CROW_WEB_UI_PROGRESS.md` for detailed roadmap.

## License

MIT (matching OpenCode)

## Links

- OpenCode: https://github.com/opencode-ai/opencode
- Dioxus: https://dioxuslabs.com
- Documentation: `/path/to/crow/OPENCODE_TUI_*.md`
