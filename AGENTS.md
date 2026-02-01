# Crow Agent Guidelines

Essential commands and guidelines for agents working on the Crow project.

## Installation & Setup

### Quick Start (Recommended)

**Install the Crow CLI globally:**
```bash
cd /home/thomas/src/projects/orchestrator-project/crow
uv build
uv tool install dist/crow_ai-0.2.3-py3-none-any.whl --python 3.12
```

This installs the `crow` command globally, available from any directory.

**Configuration:**
- API keys and secrets: `~/.crow/.env`
- User settings: `~/.crow/config.yaml`
- Sessions: `~/.crow/sessions/`
- Logs: `~/.crow/logs/`

**Usage:**
```bash
# Start ACP server (works from any directory)
crow acp

# Show version
crow --version

# Show help
crow
```

**Using with ACP clients:**
```bash
uv --project /path/to/acp-python-sdk run python acp-python-sdk/examples/client.py crow acp
```

### Reinstalling After Changes

```bash
cd /home/thomas/src/projects/orchestrator-project/crow
uv build
uv tool install dist/crow_ai-0.1.2-py3-none-any.whl --python 3.12
```

### Uninstalling

```bash
uv tool uninstall crow-ai
```

## Development Workflow

### Project Commands

**For development work on Crow itself, use `uv --project`:**

```bash
# Install dependencies
uv --project crow sync

# Add a dependency
uv --project crow add requests

# Run tests
uv --project crow run python -m pytest tests/

# Format code
uv --project crow run ruff format .

# Check code
uv --project crow run ruff check .
```

**Why `--project` for development?**
- Ensures commands run in the correct project context
- Avoids "No such file or directory" errors
- Prevents accidentally running commands in the wrong directory
- More reliable than `cd`-ing into directories

**Note:** Once installed globally with `uv tool install`, you can use `crow acp` from any directory without `uv --project`.

### Testing

**CRITICAL: Never grep test output directly**

❌ **BAD:**
```bash
uv --project crow run python -m pytest tests/ | grep -E "PASS|FAIL"
```
This loses all context and makes debugging impossible.

✅ **GOOD:**
```bash
# Run tests and save output
uv --project crow run python -m pytest tests/ > test_output.txt 2>&1

# Then you can repeatedly grep or read the output
grep "FAILED" test_output.txt
cat test_output.txt
jq '.' test_output.txt  # If JSON output
```

**Why?**
- You can grep multiple times on the same output
- You can see the full context when debugging
- You don't lose information by piping to grep
- You can share the output file for debugging

## Configuration System

### Configuration Loading Priority

The configuration system loads in this order:

1. `~/.crow/.env` (global environment variables)
2. `~/.crow/config.yaml` (global settings)
3. Local `.env` file (for development)
4. Environment variables (can override above)

This allows for:
- Global configuration for everyday use
- Local overrides for development/testing
- Environment-specific settings via env vars

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

## Code Style Guidelines

### Python

#### Imports
- Use `from __future__ import annotations` at the top of all Python files
- Group imports: standard library, third-party, local imports
- Use absolute imports over relative imports
- Type imports in `TYPE_CHECKING` blocks for performance

```python
from __future__ import annotations

import os
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from mymodule import MyClass

from third_party import library
from local_package import something
```

#### Formatting (Ruff)
- **Line length**: 100 characters
- **Indentation**: 4 spaces
- **Quotes**: Double quotes
- No trailing whitespace

#### Type Hints
- Use modern type annotations: `dict[str, int]` not `Dict[str, int]`
- Use `|` for union types: `str | None` not `Optional[str]`
- Return types required for all public functions

```python
def process_data(items: list[dict[str, int]]) -> dict[str, str]:
    """Process data items."""
    return {str(i["id"]): str(i["value"]) for i in items}
```

#### Naming Conventions
- **Functions/Variables**: `snake_case`
- **Classes**: `PascalCase`
- **Constants**: `UPPER_SNAKE_CASE`
- **Private members**: `_leading_underscore`

```python
class DataProcessor:
    MAX_RETRIES = 3

    def __init__(self, config: dict[str, Any]) -> None:
        self._config = config

    def process_batch(self, items: list[Item]) -> Result:
        """Process a batch of items."""
        pass
```

#### Error Handling
- Use specific exceptions, not bare `except:` or `Exception`
- Log exceptions before re-raising with context
- Use context managers for resource management

```python
try:
    result = process_data(data)
except ValueError as e:
    logger.error("Invalid data format: %s", e)
    raise ProcessingError("Failed to process data") from e
finally:
    cleanup()
```

#### Docstrings
- Use Google-style docstrings
- Include Args, Returns, Raises sections as needed
- First line should be a concise summary

```python
def calculate_metrics(data: list[float]) -> dict[str, float]:
    """Calculate statistical metrics for the data.

    Args:
        data: List of numeric values.

    Returns:
        Dictionary with mean, median, and std.

    Raises:
        ValueError: If data is empty.
    """
```

## General Guidelines

### Git Workflow
- **DO NOT make commits** - The user handles git, not you
- Focus on writing code and running tests
- The user will commit when ready
- Default branch is `main`

### Security
- Never hardcode credentials
- Use environment variables for configuration
- Validate all user inputs
- Sanitize file paths before operations

## Documentation References

### ACP Documentation
- **ACP Specification**: https://agentclientprotocol.com
  - Agent Client Protocol (ACP) official documentation
  - Use when working on ACP-related tasks or integrating with ACP clients

### OpenHands Documentation
- **OpenHands SDK**: https://docs.openhands.dev/sdk
  - OpenHands SDK reference and platform documentation
  - Use when working on OpenHands SDK integration
  - Contains all API references, configuration options, and integration guides
