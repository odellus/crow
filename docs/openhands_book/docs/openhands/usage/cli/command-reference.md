# Command Reference

> Complete reference for all OpenHands CLI commands and options

## Basic Usage

```bash  theme={null}
openhands [OPTIONS] [COMMAND]
```

## Global Options

| Option                        | Description                                                          |
| ----------------------------- | -------------------------------------------------------------------- |
| `-v, --version`               | Show version number and exit                                         |
| `-t, --task TEXT`             | Initial task to seed the conversation                                |
| `-f, --file PATH`             | Path to a file whose contents seed the conversation                  |
| `--resume [ID]`               | Resume a conversation. If no ID provided, lists recent conversations |
| `--last`                      | Resume the most recent conversation (use with `--resume`)            |
| `--exp`                       | Use textual-based UI (now default, kept for compatibility)           |
| `--headless`                  | Run in headless mode (no UI, requires `--task` or `--file`)          |
| `--json`                      | Enable JSONL output (requires `--headless`)                          |
| `--always-approve`            | Auto-approve all actions without confirmation                        |
| `--llm-approve`               | Use LLM-based security analyzer for action approval                  |
| `--exit-without-confirmation` | Exit without showing confirmation dialog                             |

## Subcommands

### serve

Launch the OpenHands GUI server using Docker.

```bash  theme={null}
openhands serve [OPTIONS]
```

| Option        | Description                                            |
| ------------- | ------------------------------------------------------ |
| `--mount-cwd` | Mount the current working directory into the container |
| `--gpu`       | Enable GPU support via nvidia-docker                   |

**Examples:**

```bash  theme={null}
openhands serve
openhands serve --mount-cwd
openhands serve --gpu
openhands serve --mount-cwd --gpu
```

### web

Launch the CLI as a web application accessible via browser.

```bash  theme={null}
openhands web [OPTIONS]
```

| Option    | Default   | Description                    |
| --------- | --------- | ------------------------------ |
| `--host`  | `0.0.0.0` | Host to bind the web server to |
| `--port`  | `12000`   | Port to bind the web server to |
| `--debug` | `false`   | Enable debug mode              |

**Examples:**

```bash  theme={null}
openhands web
openhands web --port 8080
openhands web --host 127.0.0.1 --port 3000
openhands web --debug
```

### cloud

Create a new conversation in OpenHands Cloud.

```bash  theme={null}
openhands cloud [OPTIONS]
```

| Option             | Description                                                                            |
| ------------------ | -------------------------------------------------------------------------------------- |
| `-t, --task TEXT`  | Initial task to seed the conversation                                                  |
| `-f, --file PATH`  | Path to a file whose contents seed the conversation                                    |
| `--server-url URL` | OpenHands server URL (default: [https://app.all-hands.dev](https://app.all-hands.dev)) |

**Examples:**

```bash  theme={null}
openhands cloud -t "Fix the bug"
openhands cloud -f task.txt
openhands cloud --server-url https://custom.server.com -t "Task"
```

### acp

Start the Agent Client Protocol server for IDE integrations.

```bash  theme={null}
openhands acp [OPTIONS]
```

| Option             | Description                         |
| ------------------ | ----------------------------------- |
| `--resume [ID]`    | Resume a conversation by ID         |
| `--last`           | Resume the most recent conversation |
| `--always-approve` | Auto-approve all actions            |
| `--llm-approve`    | Use LLM-based security analyzer     |
| `--streaming`      | Enable token-by-token streaming     |

**Examples:**

```bash  theme={null}
openhands acp
openhands acp --llm-approve
openhands acp --resume abc123def456
openhands acp --resume --last
```

### mcp

Manage Model Context Protocol server configurations.

```bash  theme={null}
openhands mcp <command> [OPTIONS]
```

#### mcp add

Add a new MCP server.

```bash  theme={null}
openhands mcp add <name> --transport <type> [OPTIONS] <target> [-- args...]
```

| Option        | Description                                                      |
| ------------- | ---------------------------------------------------------------- |
| `--transport` | Transport type: `http`, `sse`, or `stdio` (required)             |
| `--header`    | HTTP header for http/sse (format: `"Key: Value"`, repeatable)    |
| `--env`       | Environment variable for stdio (format: `KEY=value`, repeatable) |
| `--auth`      | Authentication method (e.g., `oauth`)                            |
| `--enabled`   | Enable immediately (default)                                     |
| `--disabled`  | Add in disabled state                                            |

**Examples:**

```bash  theme={null}
openhands mcp add my-api --transport http https://api.example.com/mcp
openhands mcp add my-api --transport http --header "Authorization: Bearer token" https://api.example.com
openhands mcp add local --transport stdio python -- -m my_server
openhands mcp add local --transport stdio --env "API_KEY=secret" python -- -m server
```

#### mcp list

List all configured MCP servers.

```bash  theme={null}
openhands mcp list
```

#### mcp get

Get details for a specific MCP server.

```bash  theme={null}
openhands mcp get <name>
```

#### mcp remove

Remove an MCP server configuration.

```bash  theme={null}
openhands mcp remove <name>
```

#### mcp enable

Enable an MCP server.

```bash  theme={null}
openhands mcp enable <name>
```

#### mcp disable

Disable an MCP server.

```bash  theme={null}
openhands mcp disable <name>
```

### login

Authenticate with OpenHands Cloud.

```bash  theme={null}
openhands login [OPTIONS]
```

| Option             | Description                                                                            |
| ------------------ | -------------------------------------------------------------------------------------- |
| `--server-url URL` | OpenHands server URL (default: [https://app.all-hands.dev](https://app.all-hands.dev)) |

**Examples:**

```bash  theme={null}
openhands login
openhands login --server-url https://enterprise.openhands.dev
```

### logout

Log out from OpenHands Cloud.

```bash  theme={null}
openhands logout [OPTIONS]
```

| Option             | Description                                                      |
| ------------------ | ---------------------------------------------------------------- |
| `--server-url URL` | Server URL to log out from (if not specified, logs out from all) |

**Examples:**

```bash  theme={null}
openhands logout
openhands logout --server-url https://app.all-hands.dev
```

## Interactive Commands

Commands available inside the CLI (prefix with `/`):

| Command     | Description                          |
| ----------- | ------------------------------------ |
| `/settings` | Open the settings configuration menu |
| `/new`      | Start a new conversation             |
| `/help`     | Show all available commands          |
| `/mcp`      | View MCP server status               |

## Environment Variables

| Variable              | Description                                |
| --------------------- | ------------------------------------------ |
| `LLM_API_KEY`         | API key for your LLM provider              |
| `LLM_MODEL`           | Model to use                               |
| `OPENHANDS_CLOUD_URL` | Default cloud server URL                   |
| `OPENHANDS_VERSION`   | Docker image version for `openhands serve` |

## Exit Codes

| Code | Meaning              |
| ---- | -------------------- |
| `0`  | Success              |
| `1`  | Error or task failed |
| `2`  | Invalid arguments    |

## Configuration Files

| File                          | Purpose                   |
| ----------------------------- | ------------------------- |
| `~/.openhands/settings.json`  | LLM and general settings  |
| `~/.openhands/mcp.json`       | MCP server configurations |
| `~/.openhands/conversations/` | Conversation history      |

## See Also

* [Installation](/openhands/usage/cli/installation) - Install the CLI
* [Quick Start](/openhands/usage/cli/quick-start) - Get started
* [MCP Servers](/openhands/usage/cli/mcp-servers) - Configure MCP servers


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt