# Crow Agent Guidelines

Essential commands and guidelines for agents working on the Crow project.

## Project Commands

### Running Commands

**ALWAYS use `uv --project` for running commands:**

```bash
# Run Python scripts
uv --project crow run python script.py

# Run tests
uv --project crow run python -m pytest tests/

# Run linting/formatting
uv --project crow run ruff check .
uv --project crow run ruff format .

# Run the ACP server
uv --project crow run crow acp
```

**Why `--project`?**
- Ensures commands run in the correct project context
- Avoids "No such file or directory" errors
- Prevents accidentally running commands in the wrong directory
- More reliable than `cd`-ing into directories

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

### Development Workflow

```bash
# Install dependencies
uv --project crow sync

# Add a dependency
uv --project crow add requests

# Run the main script
uv --project crow run python main.py

# Format code
uv --project crow run ruff format .

# Check code
uv --project crow run ruff check .
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
- Default branch is `main`
- Write clear, focused commit messages
- Never commit secrets (API keys, tokens)
- Run tests before committing

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
