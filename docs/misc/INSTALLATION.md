# Crow CLI Installation and Configuration

## Overview

The Crow CLI is now properly installed as a global tool using `uv tool install`, with configuration stored in `~/.crow/`. This allows the agent to be used from any directory without needing to manage virtual environments or local configuration files.

## Installation

The Crow CLI is installed as a uv tool:

```bash
# From the crow directory
cd /home/thomas/src/projects/orchestrator-project/crow
uv build
uv tool install dist/crow_ai-0.1.2-py3-none-any.whl --python 3.12
```

This makes the `crow` command available globally from any directory.

## Usage

### Starting the ACP Server

```bash
crow acp
```

This starts the ACP server, which can then be used with ACP clients:

```bash
uv --project /path/to/acp-python-sdk run python acp-python-sdk/examples/client.py crow acp
```

### Other Commands

```bash
crow              # Show usage information
crow --version    # Show version (0.1.2)
```

## Configuration

Configuration is stored in `~/.crow/`:

### Directory Structure

```
~/.crow/
├── .env           # API keys and secrets
├── config.yaml    # User settings
├── sessions/      # Session persistence
└── logs/          # Log files
```

### ~/.crow/.env

Contains API keys and sensitive configuration:

```bash
ZAI_API_KEY=your_api_key_here
ZAI_BASE_URL=https://api.z.ai/api/anthropic
LANGFUSE_PUBLIC_KEY=...
LANGFUSE_SECRET_KEY=...
LANGFUSE_HOST=http://localhost:3044
LMNR_PROJECT_API_KEY=...
```

### ~/.crow/config.yaml

Contains user settings (non-sensitive):

```yaml
llm:
  model: "anthropic/glm-4.7"
  temperature: 0.0
  max_tokens: 4096
  stream: true

agent:
  max_iterations: 500
  timeout: 300

server:
  name: "crow-acp-server"
  version: "0.1.2"
  title: "Crow ACP Server"

session:
  sessions_dir: "~/.crow/sessions"
  logs_dir: "~/.crow/logs"
```

## Configuration Loading Priority

The configuration system loads in this order:

1. `~/.crow/.env` (global environment variables)
2. `~/.crow/config.yaml` (global settings)
3. Local `.env` file (for development)
4. Environment variables (can override above)

This allows for:
- Global configuration for everyday use
- Local overrides for development/testing
- Environment-specific settings via env vars

## Implementation Details

### Key Files Modified

1. **src/crow/agent/config.py**
   - Added `get_crow_config_dir()` to get `~/.crow` path
   - Added `load_crow_env()` to load from `~/.crow/.env`
   - Added `load_crow_config()` to load from `~/.crow/config.yaml`
   - Updated `LLMConfig`, `AgentConfig`, and `ServerConfig` to use these functions

2. **src/crow/agent/cli.py**
   - Added argument parsing for `crow acp` command
   - Added `--version` flag
   - Added usage information

3. **src/crow/__init__.py**
   - Added `__version__ = "0.1.2"`

4. **pyproject.toml**
   - Added `pyyaml>=6.0.0` dependency

## Benefits

1. **No Virtual Environment Management**: Use `crow` from any directory without worrying about venvs
2. **Global Configuration**: Set up API keys once in `~/.crow/.env`
3. **Consistent Behavior**: Same configuration across all projects
4. **Easy Updates**: Rebuild and reinstall with `uv build && uv tool install dist/...`
5. **ACP Standard**: Uses the Agent Client Protocol for interoperability

## Testing

Tested from `/tmp` directory - confirmed that:
- Configuration loads from `~/.crow/.env` (API key and base URL)
- ACP server starts successfully
- Can be used with ACP clients
- Works from any directory

## Next Steps

Potential improvements:
1. Add `crow config init` command for interactive setup
2. Add `crow config edit` command to open config in editor
3. Add `crow session list` to show saved sessions
4. Add `crow session clear` to clean up old sessions
5. Add more configuration options to config.yaml
