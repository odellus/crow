# ACP Server Import Fix Task

## Problem
The ACP server at `crow/src/crow/agent/acp_server.py` cannot import `LLMConfig` from `config.py` in the same directory.

Current import attempt:
```python
from .config import LLMConfig, ServerConfig
```

Error when running:
```
ModuleNotFoundError: No module named 'crow'
```

## Files
- `crow/src/crow/agent/acp_server.py` - Main ACP server
- `crow/src/crow/agent/config.py` - Config dataclasses (LLMConfig, ServerConfig)
- `crow/src/crow/__init__.py` - Package init (currently empty)
- `crow/src/crow/agent/__init__.py` - Agent package init (currently empty)
- `crow/pyproject.toml` - Project config

## What Needs to Happen

1. **Fix the package structure** so imports work
2. **Make it runnable** as a module: `python -m crow.agent.acp_server`
3. **NO SHEBANGS** in `__init__.py` files
4. Use proper relative imports: `from .config import LLMConfig`

## Context from Research
- Python 3.3+ has implicit namespace packages
- `__init__.py` is optional but still useful
- Need to ensure `src/crow` is recognized as a package
- `pyproject.toml` may need proper package configuration

## Current State
- `src/crow/__init__.py` exists but empty
- `src/crow/agent/__init__.py` exists but empty
- `pyproject.toml` has `[project.scripts]` but no package path config
- Running `.venv/bin/python src/crow/agent/acp_server.py` works directly
- But imports fail when run as module

## Expected Result
```bash
cd crow && .venv/bin/python -m crow.agent.acp_server
```
Should start the ACP server without import errors.

## Key Constraint
DO NOT use sys.path hacks. Use proper Python package structure.
