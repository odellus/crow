# Project Directory System

**Date**: 2025-11-16  
**Status**: ✅ Working - Agents can spawn in arbitrary directories

## Overview

Crow supports **project-based directory isolation**, allowing agents to operate in specific working directories independent of where the server is running. This enables multi-project support, monorepo operations, and isolated agent contexts.

## How It Works

### Session Creation with Directory

When creating a session, specify the `directory` field to set the agent's working directory:

```bash
curl -X POST http://localhost:7070/session \
  -H "Content-Type: application/json" \
  -d '{
    "title": "My Project",
    "directory": "/path/to/project"
  }'
```

**Response:**
```json
{
  "id": "ses-xxxxx",
  "directory": "/path/to/project",
  "title": "My Project",
  ...
}
```

### Agent Execution in Project Directory

All agent operations (bash, read, write, edit, etc.) happen **in the session's directory**:

```bash
# Agent spawned in /home/user/my-project
pwd                    # → /home/user/my-project
ls                     # → lists files in my-project
cat README.md          # → reads my-project/README.md
```

The agent's system prompt includes the working directory context:
```
Working directory: /home/user/my-project
Platform: linux
...
```

## Implementation Details

### Server-Side

1. **Session Creation** (`server.rs:124-140`):
   ```rust
   let directory = if let Some(dir) = req.directory {
       dir  // Use requested directory
   } else {
       std::env::current_dir()...  // Fall back to server's cwd
   };
   ```

2. **Message Execution** (`server.rs:268-287`):
   ```rust
   // Get working directory from session
   let session = state.session_store.get(&session_id)?;
   let working_dir = PathBuf::from(&session.directory);
   
   executor.execute_turn(&session_id, &agent, &working_dir, req.parts)
   ```

3. **System Prompt** (`agent/prompt.rs`):
   ```rust
   context.push_str(&format!("Working directory: {}\n", 
       self.working_dir.display()));
   ```

### Storage

Sessions persist their directory in the filesystem:
```
~/.crow/sessions/ses-xxxxx/
  ├── session.json      # Contains "directory": "/path/to/project"
  └── messages/
```

## Use Cases

### 1. Multi-Project Support

Run agents in different projects simultaneously:

```javascript
// Project A - Web app
const sessionA = await createSession({
  title: "Web App",
  directory: "/home/user/projects/web-app"
});

// Project B - API server  
const sessionB = await createSession({
  title: "API Server",
  directory: "/home/user/projects/api-server"
});
```

Each agent works in its own project without interference.

### 2. Monorepo Package Operations

Work in specific packages within a monorepo:

```bash
# Agent in workspace root
directory: "/home/user/monorepo"

# Agent in specific package
directory: "/home/user/monorepo/packages/api"
```

### 3. Subagent Delegation with Context

Supervisor delegates to build agent with specific directory:

```typescript
// Supervisor in project root
supervisor.task("Build API package", {
  agent: "build",
  directory: "/project/packages/api"
});
```

### 4. Isolated Testing Environments

Create temporary directories for testing:

```bash
directory: "/tmp/test-${uuid}"
```

Agent operates in isolation without affecting real projects.

## Real-World Example

### Test-Dummy Project

```bash
# Create session in test-dummy
curl -X POST http://localhost:7070/session \
  -d '{"directory": "/home/thomas/src/projects/opencode-project/test-dummy"}'

# Agent explores the project
{
  "path": {
    "cwd": "/home/thomas/src/projects/opencode-project/test-dummy",
    "root": "/home/thomas/src/projects/opencode-project/test-dummy"
  }
}

# Agent finds project files
ls -la
# → hello.rs
```

**Result**: ✅ Agent successfully spawned in test-dummy, confirmed working directory, and explored project contents.

## Integration Test Results

```bash
cd crow/packages/api
cargo run --bin crow-serve --features server

# Run tests
/tmp/test_directory_spawning.sh
```

**Results**:
- ✅ Agents spawn in arbitrary directories via `session.directory`
- ✅ Each agent operates in its specified working directory  
- ✅ Build agents work correctly in different directories
- ✅ Discriminator agents work correctly in different directories
- ✅ File operations happen in the agent's working directory
- ✅ Confirmed with test-dummy project

## API Reference

### Create Session

**POST** `/session`

```json
{
  "title": "Session Title",           // Optional
  "directory": "/path/to/project",    // Optional - defaults to server's cwd
  "parentID": "ses-xxxxx"             // Optional
}
```

**Response**:
```json
{
  "id": "ses-xxxxx",
  "projectID": "default",
  "directory": "/path/to/project",    // ← Agent's working directory
  "title": "Session Title",
  "version": "1.0.0",
  "time": {...}
}
```

### Send Message

**POST** `/session/:id/message`

The agent uses the session's `directory` for all operations:

```json
{
  "agent": "build",
  "parts": [
    {
      "type": "text",
      "text": "List files and read README.md"
    }
  ]
}
```

Agent executes `ls` and `cat README.md` **in the session's directory**.

## Benefits

1. **Project Isolation**: Each session has its own workspace
2. **Context Preservation**: Directory persists across messages
3. **Multi-Project**: Run agents in different repos simultaneously
4. **Monorepo Support**: Target specific packages
5. **Clean Architecture**: Server location independent of project location

## Future Enhancements

- [ ] Directory validation (ensure path exists and is accessible)
- [ ] Relative path resolution (resolve `./` and `../`)
- [ ] Directory permissions checking
- [ ] Project detection (auto-detect git root, package.json, Cargo.toml)
- [ ] Directory change API (change directory mid-session)

## Conclusion

Crow's project directory system enables **flexible, isolated agent execution** across multiple projects. Agents spawn into specific directories and operate as if that's their natural workspace, making Crow suitable for:

- Multi-project development
- Monorepo management  
- Isolated testing
- Context-aware automation

The system is **production-ready** and has been validated with real-world testing.
