# Crow CLI - Setup Complete! 🎉

## What We Accomplished

Successfully set up Crow as a global CLI tool with centralized configuration in `~/.crow/`. The agent can now be used from any directory without needing virtual environments or local configuration files.

## Key Changes

### 1. Configuration System (`~/.crow/`)
- **Created**: `~/.crow/.env` - API keys and secrets
- **Created**: `~/.crow/config.yaml` - User settings
- **Updated**: `src/crow/agent/config.py` - Loads from `~/.crow` first
- **Added**: `pyyaml` dependency for YAML config support

### 2. CLI Improvements
- **Updated**: `src/crow/agent/cli.py` - Handles `crow acp` command
- **Added**: `crow --version` flag
- **Added**: Usage information
- **Added**: `__version__` to `src/crow/__init__.py`

### 3. Documentation Updates
- **Updated**: `AGENTS.md` - New installation and usage instructions
- **Updated**: `README.md` - Quick start guide with global installation
- **Created**: `INSTALLATION.md` - Detailed installation and configuration guide

## Usage

```bash
# Start ACP server (works from any directory!)
crow acp

# Use with ACP client
uv --project /path/to/acp-python-sdk run python acp-python-sdk/examples/client.py crow acp

# Show version
crow --version

# Show help
crow
```

## Testing Results

✅ Tested from `/tmp`, `/tmp/test_crow`, and `/tmp/final_test`
✅ Configuration loads from `~/.crow/.env` (API key and base URL)
✅ ACP server starts correctly
✅ Works with ACP clients
✅ No virtual environment management needed

## Benefits

1. **No Virtual Environment Management** - Use `crow acp` from anywhere
2. **Global Configuration** - Set up API keys once in `~/.crow/.env`
3. **Consistent Behavior** - Same configuration across all projects
4. **Easy Updates** - Rebuild and reinstall with two commands
5. **ACP Standard** - Uses Agent Client Protocol for interoperability
6. **Works in Zed** - Can be used as an ACP server in Zed editor!

## Reinstalling After Changes

```bash
cd /home/thomas/src/projects/orchestrator-project/crow
uv build
uv tool install dist/crow_ai-0.1.2-py3-none-any.whl --python 3.12
```

## Configuration Files

### ~/.crow/.env
```bash
ZAI_API_KEY=your_api_key_here
ZAI_BASE_URL=https://api.z.ai/api/anthropic
LANGFUSE_PUBLIC_KEY=...
LANGFUSE_SECRET_KEY=...
LANGFUSE_HOST=http://localhost:3044
LMNR_PROJECT_API_KEY=...
```

### ~/.crow/config.yaml
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

## Next Steps

Potential improvements:
1. Add `crow config init` command for interactive setup
2. Add `crow config edit` command to open config in editor
3. Add `crow session list` to show saved sessions
4. Add `crow session clear` to clean up old sessions
5. Add more configuration options to config.yaml

## Dogfooding Ready! 🐕

You can now start dogfooding Crow in your daily workflow. The authentication issues are resolved, and the agent works seamlessly from any directory including Zed!

---

**Status**: ✅ Production Ready
**Version**: 0.1.2
**Date**: 2025-01-24
