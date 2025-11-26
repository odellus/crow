# Crow-Tauri Migration Plan

## Critical Insight

**The agent can't test/debug GUI code.** We need CLI + observability BEFORE frontend work.

The workflow must be:
1. Write code for a command
2. Test it via CLI: `crow-cli chat "test"`
3. Observe logs: `tail -f ~/.local/state/crow/logs/agent.log`
4. Write bash script to test complex flows
5. Iterate until working
6. THEN wire up frontend

---

## XDG Directory Structure (Copied from OpenCode)

```
~/.config/crow/           # Config files
├── config.json           # User settings
└── providers.json        # Provider configs

~/.local/share/crow/      # Persistent data
├── storage/
│   ├── session/{projectID}/{sessionID}.json
│   ├── message/{sessionID}/{messageID}.json
│   ├── part/{messageID}/{partID}.json
│   └── project/{projectID}.json
├── bin/                  # Downloaded binaries
└── log/                  # Legacy log location

~/.local/state/crow/      # Runtime state & logs
├── logs/
│   ├── agent.log         # Main agent log (human readable)
│   ├── tool-calls.jsonl  # Every tool call (JSONL)
│   └── messages.jsonl    # Every message (JSONL)
└── telemetry/
    └── sessions/
        └── {session-id}/
            ├── timeline.json
            └── summary.json

~/.cache/crow/            # Temp/cache files
├── version               # Cache version marker
└── ...
```

---

## Target Architecture

```
crow-tauri/
├── src/                          # React frontend (later)
├── docs/
│   ├── bugs/                     # Bug documentation
│   │   └── 001-tool-responses-not-in-history.md
│   └── decisions/                # Architecture decisions
└── src-tauri/
    ├── Cargo.toml                # Workspace root
    │
    ├── core/                     # Core logic crate
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── bin/
    │       │   └── crow-cli.rs   # CLI binary for testing
    │       ├── global.rs         # XDG paths
    │       ├── logging.rs        # Structured logging
    │       ├── agent/
    │       ├── tools/
    │       ├── providers/
    │       ├── session/
    │       ├── storage/
    │       └── ...
    │
    └── app/                      # Tauri application crate
        ├── Cargo.toml
        └── src/
            ├── main.rs           # Entry, CLI args
            ├── lib.rs            # Tauri setup
            └── commands/         # Tauri commands
```

---

## Phase 1: Project Structure Setup ✅ COMPLETE

- [x] Create workspace in src-tauri/Cargo.toml
- [x] Create core crate from crow-backend/api
- [x] Create app crate from existing src-tauri/src
- [x] Strip dioxus/axum dependencies

---

## Phase 2: Core Crate Cleanup ✅ COMPLETE

- [x] Remove HTTP/Axum dependencies
- [x] Remove Dioxus dependencies  
- [x] Clean up feature flags
- [x] Fix duplicate code from sed cleanup
- [x] Rename core -> crow_core (avoid rust prelude conflict)
- [x] Verify core builds

---

## Phase 3: Backend Testing Infrastructure ✅ COMPLETE

### 3.1 XDG Directory Setup ✅ DONE
- [x] Copy XDG patterns from opencode (global.rs)
- [x] Implement `GlobalPaths` struct (config/data/state/cache)
- [x] Update storage to use XDG paths
- [x] Storage now at ~/.local/share/crow/storage/

### 3.2 CLI Binary for Testing ✅ DONE
- [x] Create crow-cli binary in core crate
- [x] `crow-cli chat "message"` - send message to agent with streaming
- [x] `crow-cli sessions` - list sessions
- [x] `crow-cli new [title]` - create session
- [x] `crow-cli messages <id>` - show full history with colored parts
- [x] `crow-cli paths` - show XDG paths
- [x] `crow-cli logs [count]` - show recent agent logs
- [x] `crow-cli prompt [agent]` - dump full system prompt
- [x] Provider detection (ANTHROPIC_API_KEY, OPENAI_API_KEY)
- [x] RUST_LOG support for debug output

### 3.3 Full Observability Streaming ✅ DONE
- [x] Verbose mode (default) - shows EVERYTHING:
  - 🟦 Blue: Agent thinking/reasoning (streamed token by token)
  - 🟩 Green: Tool calls starting (name, call_id, input JSON)
  - 🟨 Yellow: Tool results (output, duration)
  - 🟥 Red: Errors
  - ⬜ White: Final response text (streamed token by token)
- [x] JSON mode (`--json`) - machine-readable output with all data
- [x] Quiet mode (`--quiet`) - just response text
- [x] Session header with ID, provider, model, working dir
- [x] Stats footer with token counts, tool count, duration

### 3.4 Structured Logging ✅ DONE
- [x] Created logging module with file appenders
- [x] Log agent executions to ~/.local/state/crow/logs/agent.log
- [x] Timestamps, session_id, duration on all entries
- [x] Converted debug eprintln! to tracing::debug! for clean CLI output

### 3.5 Bug Fix: Tool Response History ✅ DONE
- [x] Fixed critical bug: tool responses weren't being added to conversation history
- [x] Agent was calling same tools repeatedly (doom loop)
- [x] Now properly reconstructs tool_calls and tool responses when loading from DB
- [x] Documented in docs/bugs/001-tool-responses-not-in-history.md

---

## Phase 3.5: Config & Storage Verification ⏳ IN PROGRESS

Before proceeding to Tauri, we need HIGH confidence that:
1. XDG directories are created and used correctly
2. Agent configs load from `~/.config/crow/agent/*.md`
3. Custom prompts in agent configs are actually applied
4. Sessions persist and restore correctly
5. Snapshots track file changes properly
6. Logging captures all executions

### 3.5.1 XDG Directory Verification
- [ ] `crow-cli paths` shows correct XDG paths
- [ ] All 4 directories created on first run (config, data, state, cache)
- [ ] Sessions stored in `~/.local/share/crow/sessions/`
- [ ] Snapshots stored in `~/.local/share/crow/snapshots/{project_id}/`
- [ ] Logs written to `~/.local/state/crow/logs/`

### 3.5.2 Agent Config Loading
- [ ] Built-in agents load (build, plan, general)
- [ ] Custom agents load from `~/.config/crow/agent/*.md`
- [ ] Custom agents load from `.crow/agent/*.md` (project-level)
- [ ] Project-level configs override global configs
- [ ] YAML frontmatter parsed correctly (description, mode, tools)
- [ ] Custom prompt (markdown body) applied to agent
- [ ] `crow-cli prompt <agent>` shows custom prompt when configured

### 3.5.3 Session Persistence
- [ ] New sessions created with `crow-cli new`
- [ ] `crow-cli sessions` lists all sessions
- [ ] `crow-cli chat --session <id>` continues existing session
- [ ] Messages persist across CLI invocations
- [ ] Agent remembers context from previous messages in session
- [ ] `crow-cli messages <id>` shows full history with tool calls

### 3.5.4 Snapshot System
- [ ] Snapshot manager auto-initializes from working directory
- [ ] Shadow git created in `~/.local/share/crow/snapshots/{project_id}/`
- [ ] File modifications tracked during agent execution
- [ ] Patch parts created for edit/write/bash tools
- [ ] Can verify file state before/after agent modification

### 3.5.5 Logging & Observability  
- [ ] Agent executions logged to `~/.local/state/crow/logs/agent.log`
- [ ] Log entries include: timestamp, session_id, agent, model, tokens, cost
- [ ] RUST_LOG=debug shows internal decisions (model selection, config loading)
- [ ] Tool calls visible in verbose CLI output

### Verification Commands
See AGENTS.md "Verifying XDG Storage" section for detailed test scripts.

---

## Phase 4: Tauri Commands Layer ⏳ TODO

### 4.1 Create command modules
- [ ] app/src/commands/mod.rs
- [ ] app/src/commands/session.rs
- [ ] app/src/commands/message.rs
- [ ] app/src/commands/file.rs

### 4.2 Session commands
- [ ] list_sessions()
- [ ] create_session(title)
- [ ] get_session(id)
- [ ] delete_session(id)

### 4.3 Message commands with streaming
- [ ] send_message(session_id, text, on_event: Channel)
- [ ] list_messages(session_id)
- [ ] StreamEvent enum (TextDelta, ToolCall, Complete, Error)

### 4.4 File commands
- [ ] list_files(path)
- [ ] read_file(path)

### 4.5 CLI integration
- [ ] All Tauri commands callable via crow-cli
- [ ] Test each command before wiring to frontend

---

## Phase 5: Frontend Migration ⏳ TODO

### 5.1 API abstraction
- [ ] src/api/index.ts
- [ ] src/api/session.ts (invoke wrappers)
- [ ] src/api/message.ts (channel handling)

### 5.2 Replace fetch with invoke
- [ ] Update useEventStream.ts
- [ ] Replace all fetch() calls
- [ ] Remove SSE/EventSource

### 5.3 Streaming via channels
- [ ] Import Channel from @tauri-apps/api/core
- [ ] Handle StreamEvent messages
- [ ] Update UI state

---

## Phase 6: Testing & Polish ⏳ TODO

- [ ] cargo tauri build produces executable
- [ ] App launches, can chat
- [ ] Tool calls work
- [ ] Logs are written correctly
- [ ] --project-dir flag works
- [ ] Clean up unused code

---

## Current Status

**Active:** Phase 3.5 - Config & Storage Verification

**Completed:**
- Phase 1: Crate structure set up
- Phase 2: Core compiles without dioxus/axum
- Phase 3: Full CLI with observability
  - XDG directories working
  - Streaming CLI with colored output
  - JSON and quiet modes
  - Agent execution confirmed working (Moonshot provider)
  - Tool response history bug fixed
  - Config-driven agents (load from `~/.config/crow/agent/*.md`)
  - TaskTool for subagent spawning
  - Agents match OpenCode: build (primary), plan (primary), general (subagent)

**In Progress:**
- Phase 3.5: Verifying all configs and storage work correctly
  - Need to confirm agent config prompts are actually applied
  - Need to test snapshot system thoroughly
  - Need to verify session continuity

**Next Steps:**
1. Run verification scripts from AGENTS.md
2. Fix any issues found with config loading
3. Then proceed to Phase 4 (Tauri commands) OR
4. Consider CLI hardening (--repl, interrupt handling) first

---

## CLI Usage (Current)

```bash
# Build release
cd crow-tauri/src-tauri
cargo build --release

# Full verbose streaming (default) - shows EVERYTHING
./target/release/crow-cli chat "list files in this directory"

# JSON output (for scripting)
./target/release/crow-cli chat --json "what is 2+2"

# Quiet mode (just response)
./target/release/crow-cli chat --quiet "hello"

# Specific session
./target/release/crow-cli chat --session ses_abc123 "hello"

# Session management
./target/release/crow-cli sessions              # List all
./target/release/crow-cli new "My Session"       # Create new
./target/release/crow-cli messages <session-id>  # Full history

# Debug/observe
./target/release/crow-cli paths                  # Show storage paths
./target/release/crow-cli logs 20                # Recent agent logs
./target/release/crow-cli prompt build           # Dump system prompt

# Different providers
ANTHROPIC_API_KEY=xxx ./target/release/crow-cli chat "test"
OPENAI_API_KEY=xxx ./target/release/crow-cli chat "test"

# Debug mode
RUST_LOG=debug ./target/release/crow-cli chat "test"
```

---

## Output Examples

### Verbose Mode (default)
```
═══════════════════════════════════════════════════════════════
Session: ses_VzayPg0dRodkDvgAqoTfFR (Streaming Test)
Provider: moonshot (kimi-k2-thinking)
Working dir: /home/thomas/src/projects/opencode-project/test-dummy
═══════════════════════════════════════════════════════════════

▶ USER
list files in this directory

🟦 THINKING
The user is asking me to list files. Let me check...

🟩 TOOL CALL: bash
   Call ID: bash:0
   Input: {"command": "ls -la"}

🟨 TOOL RESULT: bash (6ms)
   total 6112
   drwxrwxr-x  6 thomas thomas    4096 Nov 25 17:24 .
   -rw-rw-r--  1 thomas thomas     154 Nov 22 07:10 Cargo.lock
   ...

⬜ RESPONSE
Here are the files:
- Cargo.lock
- Cargo.toml
...

═══════════════════════════════════════════════════════════════
✓ ~194 thinking, ~63 response | 1 tool calls | 22.0s
Session: ses_VzayPg0dRodkDvgAqoTfFR
═══════════════════════════════════════════════════════════════
```

### JSON Mode
```json
{
  "session_id": "ses_VzayPg0dRodkDvgAqoTfFR",
  "message_id": "msg-769a5ade-4743-413f-b866-910e60dc573d",
  "thinking": "The user is asking...",
  "response": "Here are the files...",
  "tools": [
    {
      "name": "bash",
      "call_id": "bash:0",
      "input": {"command": "ls -la"},
      "output": "total 6112...",
      "duration_ms": 6
    }
  ],
  "stats": {
    "thinking_tokens": 194,
    "response_tokens": 63,
    "tool_calls": 1,
    "duration_ms": 22000
  }
}
```

---

## Log File Locations

```
~/.local/share/crow/storage/     # Session/message data
~/.local/state/crow/logs/        # Runtime logs
  ├── agent.log                  # Human-readable agent log
  ├── tool-calls.jsonl           # JSONL tool call log
  └── messages.jsonl             # JSONL message log
```

---

## Reference

- [OpenCode Global Paths](../opencode/packages/opencode/src/global/index.ts)
- [OpenCode Storage](../opencode/packages/opencode/src/storage/storage.ts)
- [Tauri Commands](https://v2.tauri.app/develop/calling-rust/)
- [Tauri Channels](https://v2.tauri.app/develop/calling-frontend/)
