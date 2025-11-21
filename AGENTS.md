# Crow Development Guide

## Quick Start

### Build & Run

```bash
cd /home/thomas/src/projects/opencode-project/crow
cargo build --release --bin crow-serve --features server
./target/release/crow-serve --port 7070
```

### Prerequisites

1. **Auth** at `~/.local/share/crow/auth.json`:
   ```json
   {"moonshotai":{"type":"api","key":"your-api-key"}}
   ```

2. **Config** at `.crow/config.jsonc` in project directory:
   ```json
   {
     "model": "moonshotai/kimi-k2-thinking"
   }
   ```

## API Usage

**Important:** Endpoints use `/session` not `/sessions`

### Create Session
```bash
curl -s http://localhost:7070/session -X POST \
  -H "Content-Type: application/json" \
  -d '{"path":"/your/project/directory"}'
```

### Send Message
```bash
curl -s "http://localhost:7070/session/$SESSION_ID/message" -X POST \
  -H "Content-Type: application/json" \
  -d '{"parts":[{"type":"text","text":"your message"}]}'
```

### Other Endpoints
- `GET /session` - list sessions
- `GET /session/:id` - get session
- `DELETE /session/:id` - delete session
- `GET /session/:id/message` - list messages
- `GET /session/:id/children` - list child sessions

## Comparing with OpenCode

Run both from the same directory to compare prompts:

```bash
cd /home/thomas/src/projects/opencode-project/test-dummy

# Start OpenCode
OPENCODE_VERBOSE_LOG=1 opencode serve -p 4200 &

# Start Crow
CROW_VERBOSE_LOG=1 ../crow/target/release/crow-serve --port 4201 &

# Create sessions and send messages
OC_SID=$(curl -s http://localhost:4200/session -X POST -d '{}' -H "Content-Type: application/json" | jq -r '.id')
CROW_SID=$(curl -s http://localhost:4201/session -X POST -d '{"path":"/home/thomas/src/projects/opencode-project/test-dummy"}' -H "Content-Type: application/json" | jq -r '.id')

curl -s "http://localhost:4200/session/$OC_SID/message" -X POST -d '{"parts":[{"type":"text","text":"hi"}]}' -H "Content-Type: application/json"
curl -s "http://localhost:4201/session/$CROW_SID/message" -X POST -d '{"parts":[{"type":"text","text":"hi"}]}' -H "Content-Type: application/json"
```

### Verbose Log Locations

- **OpenCode:** `~/.local/share/opencode/verbose/`
- **Crow:** `~/.local/share/crow/requests/`

Compare with:
```bash
OC_LOG=$(ls -t ~/.local/share/opencode/verbose/ | head -1)
CROW_LOG=$(ls -t ~/.local/share/crow/requests/ | head -1)

# System prompt lengths
cat ~/.local/share/opencode/verbose/$OC_LOG | jq '[.messages[] | select(.role == "system") | .content | length]'
cat ~/.local/share/crow/requests/$CROW_LOG | jq '[.messages[] | select(.role == "system") | .content | length]'

# Diff prompts
cat ~/.local/share/opencode/verbose/$OC_LOG | jq -r '.messages[1].content' > /tmp/oc.txt
cat ~/.local/share/crow/requests/$CROW_LOG | jq -r '.messages[1].content' > /tmp/crow.txt
diff /tmp/oc.txt /tmp/crow.txt
```

## Project Structure

```
crow/
├── packages/
│   └── api/
│       └── src/
│           ├── agent/
│           │   ├── executor.rs    # Agent execution loop
│           │   ├── prompt.rs      # System prompt building
│           │   ├── registry.rs    # Agent definitions
│           │   └── types.rs       # Agent types
│           ├── providers/
│           │   └── client.rs      # LLM API calls
│           ├── session/
│           │   └── store.rs       # Session persistence
│           ├── tools/
│           │   ├── mod.rs         # Tool registry
│           │   └── *.rs           # Individual tools
│           ├── config/
│           │   └── loader.rs      # Config loading
│           └── server.rs          # HTTP endpoints
└── Cargo.toml
```

### Key Files for Prompt Parity

| Crow | OpenCode | Purpose |
|------|----------|---------|
| `agent/prompt.rs` | `session/system.ts` | System prompt building |
| `agent/executor.rs` | `session/prompt.ts` | Message construction |
| `providers/client.rs` | Provider SDK | LLM API calls |
| `tools/mod.rs` | `tool/registry.ts` | Tool definitions |

## Storage

```
~/.local/share/crow/
├── auth.json           # API keys
├── requests/           # Verbose request logs
├── storage/
│   ├── session/        # Session metadata
│   ├── message/        # Messages per session
│   └── todo/           # Todos per session
└── log/                # Server logs
```

## Current Status

**Working:**
- 2 system messages matching OpenCode structure
- File tree generation from working directory
- 13 tools (bash, edit, write, read, grep, glob, list, todowrite, todoread, task, webfetch, websearch, invalid)
- Config loading from `.crow/config.jsonc`
- Session management with XDG persistence

**Prompt parity:** System message 1 matches exactly. Message 2 is ~400 chars longer due to file tree differences (breadth-first traversal variations).

## Troubleshooting

**Lock poisoning:** Restart server with `pkill -9 crow-serve`

**Empty verbose logs:** Ensure `CROW_VERBOSE_LOG=1` is set

**Wrong model:** Check `.crow/config.jsonc` has correct model string
