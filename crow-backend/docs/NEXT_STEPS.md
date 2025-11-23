# Crow Progress & Next Steps

## Backend (packages/api) - ~80% Complete

### Working
- Session storage (reads/writes to `.crow/storage/` - same structure as OpenCode)
- SessionStore with sync initialization
- Message and Part storage
- Dioxus server functions (`list_sessions`, `create_session`, `get_session`, `send_message`)
- Tool registry (Bash, Glob, Grep, Read, Write, Edit, Task)
- Agent executor with tool loop
- Provider client (async-openai)

### Issues
- `CrowStorage` should be renamed to `Storage` to match OpenCode naming

### Missing
- Streaming responses
- Proper error handling in some places
- Config loading from OpenCode's format

---

## Frontend (packages/web) - ~40% Complete

### Working
- Dioxus fullstack app builds
- Sessions list view with sidebar
- Session detail view stub
- Tailwind CSS styling
- Server functions calling backend storage
- Sessions load from disk

### Current Errors
- WASM runtime error (`unreachable executed`) - likely a hydration mismatch
- Asset preload warning

### Not Implemented
- Message display in chat
- Input box for sending messages
- Navigation (clicking sessions)
- New session creation wired up
- Tool output rendering
- Streaming message display

---

## Deployment - Working

- Single binary `./crow` + `public/` folder
- Run with `IP="0.0.0.0" PORT="8090" ./crow`
- Build with `dx build --release --platform web`

---

## Next Steps (Priority Order)

1. **Fix WASM runtime error** - Debug hydration mismatch causing `unreachable executed`
2. **Rename CrowStorage to Storage** - Match OpenCode naming conventions
3. **Wire up session navigation** - Clicking sessions should load them
4. **Implement message display** - Show chat history in session detail
5. **Add message input** - Text input for sending messages to agent
6. **Tool output rendering** - Display Bash, Read, Edit results properly
7. **Streaming responses** - Show agent responses as they stream in

---

## Build Commands

```bash
# Build web frontend
cd packages/web && dx build --release --platform web

# Copy assets and binary
cp packages/web/assets/tailwind.css target/dx/web/release/web/public/
cp -r target/dx/web/release/web/public .
cp target/dx/web/release/web/web crow

# Run
IP="0.0.0.0" PORT="8090" ./crow
```
