# Agent Guidelines

This file contains build commands, testing instructions, and code style guidelines for agents working in this monorepo.

## Agent Workflow Guidelines

### Always Make a Plan First

**IMPORTANT**: Always use the `task_tracker` tool to create and manage a plan before starting implementation. This is non-negotiable.

```python
# Example: Create a plan before coding
task_tracker("plan", task_list=[
    {"title": "Explore codebase and understand requirements", "status": "todo"},
    {"title": "Design solution approach", "status": "todo"},
    {"title": "Implement core functionality", "status": "todo"},
    {"title": "Test and verify", "status": "todo"},
])
```

**Why?**
- Forces structured thinking before coding
- Provides visibility into progress
- Makes it easy to resume work later
- Prevents "I forgot what I was doing" syndrome

### Tool Usage Notes

**file_editor security_risk parameter**:
- You're running in **always-approve mode**
- Always set `security_risk="LOW"` for file operations
- Don't spend time thinking about it - just use LOW
- Version control exists for a reason

```python
# Good - minimal thought, just get it done
file_editor("view", path="/path/to/file", security_risk="LOW")
file_editor("create", path="/path/to/new", file_text="...", security_risk="LOW")
```

## Project Structure

This is a monorepo containing multiple projects:
- **opencode** - AI-powered development tool (TypeScript/Bun)
- **marimo** - Reactive notebooks and apps (Python/TypeScript)
- **karla** - Python coding agent on Crow/Letta (Python)
- **crow** - Python agent framework (Python)
- **crow_ide** - Crow IDE with frontend (Python/React)

---

## Build/Lint/Test Commands


### Crow & Crow_IDE (Python)

```bash
cd crow  # or cd crow_ide

# Setup
cd path/to/dir && . .venv/bin/activate && uv sync      # Install dependencies
cd path/to/dir && . .venv/bin/activate && uv add ../acp-python-sdk/  # Add dependencies

# Development
cd crow && .venv/bin/python main.py           # Run main script

# Checks
cd path/to/dir && . .venv/bin/activate && ruff check .                    # Lint
cd path/to/dir && . .venv/bin/activate && ruff format .                   # Format
```


---

## Code Style Guidelines

### Python

#### Imports
- Use `from __future__ import annotations` at the top of all Python files (marimo convention)
- Group imports in this order: standard library, third-party, local imports
- Use absolute imports over relative imports (marimo bans relative imports)
- Type imports should be in `TYPE_CHECKING` blocks for performance

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
- **Line length**: 79 characters (marimo), 100 characters (karla, trae-agent)
- **Indentation**: 4 spaces
- **Quotes**: Double quotes for strings
- No trailing whitespace
- Use `ruff format .` to auto-format

#### Type Hints
- Use modern type annotations: `dict[str, int]` not `Dict[str, int]`
- Use `|` for union types: `str | None` not `Optional[str]`
- Return types are required for all public functions (marimo strict mode)
- Use `typing.Any` sparingly, prefer specific types

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
- Use Google-style docstrings (marimo convention)
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

### TypeScript

#### Imports
- Group imports: external libraries, then internal modules
- Use `import type` for type-only imports
- Avoid default exports, prefer named exports

```typescript
import { useState, useEffect } from 'react'
import type { ToolDefinition } from './tool'
import { executeTool } from './utils'
```

#### Formatting (Prettier/Biome)
- **Line width**: 80-120 characters (varies by project)
- **Indentation**: 2 spaces
- **Semicolons**: Always (marimo), Never (opencode - semi: false)
- **Quotes**: Single quotes (preferred) or double
- **Trailing commas**: All (marimo), none (opencode)

```bash
# Marimo frontend (Biome)
biome check --write .

# OpenCode (Prettier)
prettier --write .
```

#### Type Definitions
- Use `interface` for object shapes, `type` for unions/utility types
- Avoid `any`, use `unknown` or specific types
- Use readonly for immutable arrays

```typescript
interface PluginConfig {
  readonly name: string
  readonly tools: ToolDefinition[]
}

type Result<T> = { success: true; data: T } | { success: false; error: Error }
```

#### Naming Conventions
- **Components**: `PascalCase`
- **Functions/Variables**: `camelCase`
- **Types/Interfaces**: `PascalCase`
- **Constants**: `UPPER_SNAKE_CASE`

```typescript
const MAX_RETRIES = 3

interface DataLoader {
  fetchData(): Promise<Data>
}

function processData(items: Item[]): ProcessedItem[] {
  return items.map(item => transform(item))
}
```

#### Error Handling
- Use try-catch for async operations
- Type errors properly

```typescript
try {
  const result = await apiCall()
} catch (error) {
  if (error instanceof Error) {
    logger.error('API call failed', error)
  }
  throw new ProcessingError('Failed to process', { cause: error })
}
```

---

## General Guidelines

### Testing
- Write tests alongside implementation
- Use descriptive test names
- Test edge cases, not just happy paths
- Use fixtures for test data

### Git Workflow
- Default branch is `dev` (opencode)
- Write clear, focused commit messages
- Never commit secrets (API keys, tokens)
- Run tests before committing

### Documentation
- Keep docs up-to-date with code changes
- Use README files for project-level info
- Inline comments for complex logic only

### Security
- Never hardcode credentials
- Use environment variables for configuration
- Validate all user inputs
- Sanitize file paths before operations

---

## Documentation References

### ACP Documentation
- **ACP llms.txt**: https://agentclientprotocol.com/llms.txt
  - Agent Client Protocol (ACP) official documentation
  - Use when working on ACP-related tasks or integrating with ACP clients

### OpenHands Documentation
- **OpenHands llms.txt**: https://docs.openhands.dev/llms.txt
  - OpenHands SDK reference and platform documentation
  - Use when working on OpenHands, software-agent-sdk, or agent configuration
  - Contains all API references, configuration options, runtime details, and integration guides

### Skills Documentation
- **Skills**: https://docs.openhands.dev/overview/skills/keyword.md | https://docs.openhands.dev/overview/skills/org.md | https://docs.openhands.dev/overview/skills/repo.md
  - Skills system in OpenHands provides context and specialized instructions
  - Three types: Global (registry), Organization/User (`.openhands` repo), General (always loaded)

---

> **Note:** When working on OpenHands or ACP-related tasks, having access to these documentation links in AGENTS.md provides immediate context about available protocols, APIs, and capabilities without needing to search separately. This is similar to how OpenHands Skills system works - providing context that the agent can reference during conversations.

When running tests or debugging:
- **ALWAYS view the FULL raw output** without any filtering
- Grep/tail/head filters out critical information needed to diagnose problems
- If you filter output, you WILL miss important error messages, stack traces, or context
- The user will get angry if you grep test output (and they should)

**Wrong:**
```bash
pytest test_file.py | grep -E "PASS|FAIL"
pytest test_file.py | tail -20
```

**Right:**
```bash
pytest test_file.py  # View full output
pytest test_file.py -xvs  # View full verbose output
```

**When investigating test failures:**
1. Run the test without any filtering
2. Read the ENTIRE output from start to finish
3. Look for stack traces, error messages, and unexpected behavior
4. Only after understanding the full output should you make changes

**This rule applies to:**
- All test runs (pytest, unittest, etc.)
- All debugging output
- All build output
- All server logs
- ANY diagnostic information

**Reason:** Filtering output destroys context and makes debugging impossible. You've been warned repeatedly about this.


# AGENT CLIENT FUCKING PROTOCOL

FETCH THIS!!!

READ THIS!!!

https://agentclientprotocol.com/llms.txt

FUCK IT HERE'S THE WHOLE THING


# Agent Client Protocol

## Docs

- [Brand](https://agentclientprotocol.com/brand.md): Assets for the Agent Client Protocol brand.
- [Code of Conduct](https://agentclientprotocol.com/community/code-of-conduct.md)
- [Contributor Communication](https://agentclientprotocol.com/community/communication.md): Communication methods for Agent Client Protocol contributors
- [Contributing](https://agentclientprotocol.com/community/contributing.md): How to participate in the development of ACP
- [Governance](https://agentclientprotocol.com/community/governance.md): How the ACP project is governed
- [Working and Interest Groups](https://agentclientprotocol.com/community/working-interest-groups.md): Learn about the two forms of collaborative groups within the Agent Client Protocol's governance structure - Working Groups and Interest Groups.
- [Community](https://agentclientprotocol.com/libraries/community.md): Community managed libraries for the Agent Client Protocol
- [Kotlin](https://agentclientprotocol.com/libraries/kotlin.md): Kotlin library for the Agent Client Protocol
- [Python](https://agentclientprotocol.com/libraries/python.md): Python library for the Agent Client Protocol
- [Rust](https://agentclientprotocol.com/libraries/rust.md): Rust library for the Agent Client Protocol
- [TypeScript](https://agentclientprotocol.com/libraries/typescript.md): TypeScript library for the Agent Client Protocol
- [Agents](https://agentclientprotocol.com/overview/agents.md): Agents implementing the Agent Client Protocol
- [Architecture](https://agentclientprotocol.com/overview/architecture.md): Overview of the Agent Client Protocol architecture
- [Clients](https://agentclientprotocol.com/overview/clients.md): Clients implementing the Agent Client Protocol
- [Introduction](https://agentclientprotocol.com/overview/introduction.md): Get started with the Agent Client Protocol (ACP)
- [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan.md): How Agents communicate their execution plans
- [Content](https://agentclientprotocol.com/protocol/content.md): Understanding content blocks in the Agent Client Protocol
- [Cancellation](https://agentclientprotocol.com/protocol/draft/cancellation.md): Mechanisms for request cancellation
- [Schema](https://agentclientprotocol.com/protocol/draft/schema.md): Schema definitions for the Agent Client Protocol
- [Extensibility](https://agentclientprotocol.com/protocol/extensibility.md): Adding custom data and capabilities
- [File System](https://agentclientprotocol.com/protocol/file-system.md): Client filesystem access methods
- [Initialization](https://agentclientprotocol.com/protocol/initialization.md): How all Agent Client Protocol connections begin
- [Overview](https://agentclientprotocol.com/protocol/overview.md): How the Agent Client Protocol works
- [Prompt Turn](https://agentclientprotocol.com/protocol/prompt-turn.md): Understanding the core conversation flow
- [Schema](https://agentclientprotocol.com/protocol/schema.md): Schema definitions for the Agent Client Protocol
- [Session Modes](https://agentclientprotocol.com/protocol/session-modes.md): Switch between different agent operating modes
- [Session Setup](https://agentclientprotocol.com/protocol/session-setup.md): Creating and loading sessions
- [Slash Commands](https://agentclientprotocol.com/protocol/slash-commands.md): Advertise available slash commands to clients
- [Terminals](https://agentclientprotocol.com/protocol/terminals.md): Executing and managing terminal commands
- [Tool Calls](https://agentclientprotocol.com/protocol/tool-calls.md): How Agents report tool call execution
- [Transports](https://agentclientprotocol.com/protocol/transports.md): Mechanisms for agents and clients to communicate with each other
- [Requests for Dialog (RFDs)](https://agentclientprotocol.com/rfds/about.md): Our process for introducing changes to the protocol
- [ACP Agent Registry](https://agentclientprotocol.com/rfds/acp-agent-registry.md)
- [Agent Telemetry Export](https://agentclientprotocol.com/rfds/agent-telemetry-export.md)
- [Introduce RFD Process](https://agentclientprotocol.com/rfds/introduce-rfd-process.md)
- [MCP-over-ACP: MCP Transport via ACP Channels](https://agentclientprotocol.com/rfds/mcp-over-acp.md)
- [Meta Field Propagation Conventions](https://agentclientprotocol.com/rfds/meta-propagation.md)
- [Agent Extensions via ACP Proxies](https://agentclientprotocol.com/rfds/proxy-chains.md)
- [Request Cancellation Mechanism](https://agentclientprotocol.com/rfds/request-cancellation.md)
- [Session Config Options](https://agentclientprotocol.com/rfds/session-config-options.md)
- [Forking of existing sessions](https://agentclientprotocol.com/rfds/session-fork.md)
- [Session Info Update](https://agentclientprotocol.com/rfds/session-info-update.md)
- [Session List](https://agentclientprotocol.com/rfds/session-list.md)
- [Resuming of existing sessions](https://agentclientprotocol.com/rfds/session-resume.md)
- [Session Usage and Context Status](https://agentclientprotocol.com/rfds/session-usage.md)
- [Updates](https://agentclientprotocol.com/updates.md): Updates and announcements about the Agent Client Protocol


FETCH THESE WHEN IN DOUBT. THESE ARE YOU SKILLS SO YOU AREN'T A STUPID BITCH ABOUT AGENT CLIENT FUCKING PROTOCOL



# OPENHANDS 

FETCH THIS SO YOU DON'T LOOK LIKE A STUPID BITCH WHEN TALKING ABOUT  OPEN FUCKING HANDS

https://docs.openhands.dev/llms.txt

# OpenHands Docs

## Docs

- [Add Event](https://docs.openhands.dev/api-reference/add-event.md)
- [Alive](https://docs.openhands.dev/api-reference/alive.md)
- [Create Custom Secret](https://docs.openhands.dev/api-reference/create-custom-secret.md)
- [Delete Conversation](https://docs.openhands.dev/api-reference/delete-conversation.md)
- [Delete Custom Secret](https://docs.openhands.dev/api-reference/delete-custom-secret.md)
- [Download Workspace Archive](https://docs.openhands.dev/api-reference/download-workspace-archive.md): Return a ZIP archive of the current conversation workspace.
- [Get Config](https://docs.openhands.dev/api-reference/get-config.md): Get current config.
- [Get Conversation](https://docs.openhands.dev/api-reference/get-conversation.md)
- [Get File Content](https://docs.openhands.dev/api-reference/get-file-content.md): Return the content of the given file from the conversation workspace.
- [Get Microagents](https://docs.openhands.dev/api-reference/get-microagents.md): Get all microagents associated with the conversation.

This endpoint returns all repository and knowledge microagents that are loaded for the conversation.
- [Get Prompt](https://docs.openhands.dev/api-reference/get-prompt.md)
- [Get Remote Runtime Config](https://docs.openhands.dev/api-reference/get-remote-runtime-config.md): Retrieve the runtime configuration.

Currently, this is the session ID and runtime ID (if available).
- [Get Repository Branches](https://docs.openhands.dev/api-reference/get-repository-branches.md): Get branches for a repository.
- [Get Repository Microagent Content](https://docs.openhands.dev/api-reference/get-repository-microagent-content.md): Fetch the content of a specific microagent file from a repository.
- [Get Repository Microagents](https://docs.openhands.dev/api-reference/get-repository-microagents.md): Scan the microagents directory of a repository and return the list of microagents.

The microagents directory location depends on the git provider and actual repository name:
- If git provider is not GitLab and actual repository name is ".openhands": scans "microagents" folder
- If git provider is GitLab and actual repository name is "openhands-config": scans "microagents" folder
- Otherwise: scans ".openhands/microagents" folder
- [Get Suggested Tasks](https://docs.openhands.dev/api-reference/get-suggested-tasks.md): Get suggested tasks for the authenticated user across their most recently pushed repositories.
- [Get Trajectory](https://docs.openhands.dev/api-reference/get-trajectory.md): Get trajectory.

This function retrieves the current trajectory and returns it.
- [Get User](https://docs.openhands.dev/api-reference/get-user.md)
- [Get User Installations](https://docs.openhands.dev/api-reference/get-user-installations.md)
- [Get User Repositories](https://docs.openhands.dev/api-reference/get-user-repositories.md)
- [Git Changes](https://docs.openhands.dev/api-reference/git-changes.md)
- [Git Diff](https://docs.openhands.dev/api-reference/git-diff.md)
- [Health](https://docs.openhands.dev/api-reference/health.md)
- [List Agents](https://docs.openhands.dev/api-reference/list-agents.md): List available agent types supported by this server.
- [List Security Analyzers](https://docs.openhands.dev/api-reference/list-security-analyzers.md): List supported security analyzers.
- [List Supported Models](https://docs.openhands.dev/api-reference/list-supported-models.md): List model identifiers available on this server based on configured providers.
- [List Workspace Files](https://docs.openhands.dev/api-reference/list-workspace-files.md): List workspace files visible to the conversation runtime. Applies .gitignore and internal ignore rules.
- [Load Custom Secrets Names](https://docs.openhands.dev/api-reference/load-custom-secrets-names.md)
- [Load Settings](https://docs.openhands.dev/api-reference/load-settings.md)
- [New Conversation](https://docs.openhands.dev/api-reference/new-conversation.md): Initialize a new session or join an existing one.

After successful initialization, the client should connect to the WebSocket
using the returned conversation ID.
- [Reset Settings](https://docs.openhands.dev/api-reference/reset-settings.md): Resets user settings. (Deprecated)
- [Search Conversations](https://docs.openhands.dev/api-reference/search-conversations.md)
- [Search Events](https://docs.openhands.dev/api-reference/search-events.md): Search through the event stream with filtering and pagination.
- [Search Repositories](https://docs.openhands.dev/api-reference/search-repositories.md)
- [Start Conversation](https://docs.openhands.dev/api-reference/start-conversation.md): Start an agent loop for a conversation.

This endpoint calls the conversation_manager's maybe_start_agent_loop method
to start a conversation. If the conversation is already running, it will
return the existing agent loop info.
- [Stop Conversation](https://docs.openhands.dev/api-reference/stop-conversation.md): Stop an agent loop for a conversation.

This endpoint calls the conversation_manager's close_session method
to stop a conversation.
- [Store Provider Tokens](https://docs.openhands.dev/api-reference/store-provider-tokens.md)
- [Store Settings](https://docs.openhands.dev/api-reference/store-settings.md)
- [Submit Feedback](https://docs.openhands.dev/api-reference/submit-feedback.md): Submit user feedback.

This function stores the provided feedback data.

To submit feedback:
- [Unset Provider Tokens](https://docs.openhands.dev/api-reference/unset-provider-tokens.md)
- [Update Conversation](https://docs.openhands.dev/api-reference/update-conversation.md): Update conversation metadata.

This endpoint allows updating conversation details like title.
Only the conversation owner can update the conversation.
- [Update Custom Secret](https://docs.openhands.dev/api-reference/update-custom-secret.md)
- [Upload Files](https://docs.openhands.dev/api-reference/upload-files.md)
- [Configuration Options](https://docs.openhands.dev/openhands/usage/advanced/configuration-options.md): This page outlines all available configuration options for OpenHands, allowing you to customize its behavior and integrate it with other services.
- [Custom Sandbox](https://docs.openhands.dev/openhands/usage/advanced/custom-sandbox-guide.md): This guide is for users that would like to use their own custom Docker image for the runtime. For example, with certain tools or programming languages pre-installed.
- [Search Engine Setup](https://docs.openhands.dev/openhands/usage/advanced/search-engine-setup.md): Configure OpenHands to use Tavily as a search engine.
- [OpenHands Cloud](https://docs.openhands.dev/openhands/usage/cli/cloud.md): Create and manage OpenHands Cloud conversations from the CLI
- [Command Reference](https://docs.openhands.dev/openhands/usage/cli/command-reference.md): Complete reference for all OpenHands CLI commands and options
- [GUI Server](https://docs.openhands.dev/openhands/usage/cli/gui-server.md): Launch the full OpenHands web GUI using Docker
- [Headless Mode](https://docs.openhands.dev/openhands/usage/cli/headless.md): Run OpenHands without UI for scripting, automation, and CI/CD pipelines
- [JetBrains IDEs](https://docs.openhands.dev/openhands/usage/cli/ide/jetbrains.md): Configure OpenHands with IntelliJ IDEA, PyCharm, WebStorm, and other JetBrains IDEs
- [IDE Integration Overview](https://docs.openhands.dev/openhands/usage/cli/ide/overview.md): Use OpenHands directly in your favorite code editor through the Agent Client Protocol
- [Toad Terminal](https://docs.openhands.dev/openhands/usage/cli/ide/toad.md): Use OpenHands with the Toad universal terminal interface for AI agents
- [VS Code](https://docs.openhands.dev/openhands/usage/cli/ide/vscode.md): Use OpenHands in Visual Studio Code with the VSCode ACP community extension
- [Zed IDE](https://docs.openhands.dev/openhands/usage/cli/ide/zed.md): Configure OpenHands with the Zed code editor through the Agent Client Protocol
- [Installation](https://docs.openhands.dev/openhands/usage/cli/installation.md): Install the OpenHands CLI on your system
- [MCP Servers](https://docs.openhands.dev/openhands/usage/cli/mcp-servers.md): Manage Model Context Protocol servers to extend OpenHands capabilities
- [Quick Start](https://docs.openhands.dev/openhands/usage/cli/quick-start.md): Get started with OpenHands CLI in minutes
- [Resume Conversations](https://docs.openhands.dev/openhands/usage/cli/resume.md): How to resume previous conversations in the OpenHands CLI
- [Terminal (TUI)](https://docs.openhands.dev/openhands/usage/cli/terminal.md): Use OpenHands interactively in your terminal with the text-based user interface
- [Web Interface](https://docs.openhands.dev/openhands/usage/cli/web-interface.md): Access the OpenHands CLI through your web browser
- [Bitbucket Integration](https://docs.openhands.dev/openhands/usage/cloud/bitbucket-installation.md): This guide walks you through the process of installing OpenHands Cloud for your Bitbucket repositories. Once set up, it will allow OpenHands to work with your Bitbucket repository.
- [Cloud API](https://docs.openhands.dev/openhands/usage/cloud/cloud-api.md): OpenHands Cloud provides a REST API that allows you to programmatically interact with OpenHands. This guide explains how to obtain an API key and use the API to start conversations and retrieve their status.
- [Cloud UI](https://docs.openhands.dev/openhands/usage/cloud/cloud-ui.md): The Cloud UI provides a web interface for interacting with OpenHands. This page provides references on how to use the OpenHands Cloud UI.
- [GitHub Integration](https://docs.openhands.dev/openhands/usage/cloud/github-installation.md): This guide walks you through the process of installing OpenHands Cloud for your GitHub repositories. Once set up, it will allow OpenHands to work with your GitHub repository through the Cloud UI or straight from GitHub!
- [GitLab Integration](https://docs.openhands.dev/openhands/usage/cloud/gitlab-installation.md): This guide walks you through the process of installing OpenHands Cloud for your GitLab repositories. Once set up, it will allow OpenHands to work with your GitLab repository through the Cloud UI or straight from GitLab!.
- [Getting Started](https://docs.openhands.dev/openhands/usage/cloud/openhands-cloud.md): Getting started with OpenHands Cloud.
- [Jira Cloud Integration](https://docs.openhands.dev/openhands/usage/cloud/project-management/jira-integration.md): Complete guide for setting up Jira Cloud integration with OpenHands Cloud, including service account creation, API token generation, webhook configuration, and workspace integration setup.
- [Slack Integration](https://docs.openhands.dev/openhands/usage/cloud/slack-installation.md): This guide walks you through installing the OpenHands Slack app.
- [Repository Customization](https://docs.openhands.dev/openhands/usage/customization/repository.md): You can customize how OpenHands interacts with your repository by creating a `.openhands` directory at the root level.
- [Key Features](https://docs.openhands.dev/openhands/usage/key-features.md)
- [Azure](https://docs.openhands.dev/openhands/usage/llms/azure-llms.md): OpenHands uses LiteLLM to make calls to Azure's chat models. You can find their documentation on using Azure as a provider [here](https://docs.litellm.ai/docs/providers/azure).
- [Google Gemini/Vertex](https://docs.openhands.dev/openhands/usage/llms/google-llms.md): OpenHands uses LiteLLM to make calls to Google's chat models. You can find their documentation on using Google as a provider -> [Gemini - Google AI Studio](https://docs.litellm.ai/docs/providers/gemini), [VertexAI - Google Cloud Platform](https://docs.litellm.ai/docs/providers/vertex)
- [Groq](https://docs.openhands.dev/openhands/usage/llms/groq.md): OpenHands uses LiteLLM to make calls to chat models on Groq. You can find their documentation on using Groq as a provider [here](https://docs.litellm.ai/docs/providers/groq).
- [LiteLLM Proxy](https://docs.openhands.dev/openhands/usage/llms/litellm-proxy.md): OpenHands supports using the [LiteLLM proxy](https://docs.litellm.ai/docs/proxy/quick_start) to access various LLM providers.
- [Overview](https://docs.openhands.dev/openhands/usage/llms/llms.md): OpenHands can connect to any LLM supported by LiteLLM. However, it requires a powerful model to work.
- [Local LLMs](https://docs.openhands.dev/openhands/usage/llms/local-llms.md): When using a Local LLM, OpenHands may have limited functionality. It is highly recommended that you use GPUs to serve local models for optimal experience.
- [Moonshot AI](https://docs.openhands.dev/openhands/usage/llms/moonshot.md): How to use Moonshot AI models with OpenHands
- [OpenAI](https://docs.openhands.dev/openhands/usage/llms/openai-llms.md): OpenHands uses LiteLLM to make calls to OpenAI's chat models. You can find their documentation on using OpenAI as a provider [here](https://docs.litellm.ai/docs/providers/openai).
- [OpenHands](https://docs.openhands.dev/openhands/usage/llms/openhands-llms.md): OpenHands LLM provider with access to state-of-the-art (SOTA) agentic coding models.
- [OpenRouter](https://docs.openhands.dev/openhands/usage/llms/openrouter.md): OpenHands uses LiteLLM to make calls to chat models on OpenRouter. You can find their documentation on using OpenRouter as a provider [here](https://docs.litellm.ai/docs/providers/openrouter).
- [Configure](https://docs.openhands.dev/openhands/usage/run-openhands/gui-mode.md): High level overview of configuring the OpenHands Web interface.
- [Setup](https://docs.openhands.dev/openhands/usage/run-openhands/local-setup.md): Getting started with running OpenHands on your own.
- [Daytona Runtime](https://docs.openhands.dev/openhands/usage/runtimes/daytona.md): You can use [Daytona](https://www.daytona.io/) as a runtime provider.
- [Docker Runtime](https://docs.openhands.dev/openhands/usage/runtimes/docker.md): This is the default Runtime that's used when you start OpenHands.
- [E2B Runtime](https://docs.openhands.dev/openhands/usage/runtimes/e2b.md): E2B is an open-source secure cloud environment (sandbox) made for running AI-generated code and agents.
- [Local Runtime](https://docs.openhands.dev/openhands/usage/runtimes/local.md): The Local Runtime allows the OpenHands agent to execute actions directly on your local machine without using Docker. This runtime is primarily intended for controlled environments like CI pipelines or testing scenarios where Docker is not available.
- [Modal Runtime](https://docs.openhands.dev/openhands/usage/runtimes/modal.md)
- [Overview](https://docs.openhands.dev/openhands/usage/runtimes/overview.md): This section is for users that would like to use a runtime other than Docker for OpenHands.
- [Remote Runtime](https://docs.openhands.dev/openhands/usage/runtimes/remote.md): This runtime is specifically designed for agent evaluation purposes only through the [OpenHands evaluation harness](https://github.com/OpenHands/OpenHands/tree/main/evaluation). It should not be used to launch production OpenHands applications.
- [Runloop Runtime](https://docs.openhands.dev/openhands/usage/runtimes/runloop.md): Runloop provides a fast, secure and scalable AI sandbox (Devbox). Check out the [runloop docs](https://docs.runloop.ai/overview/what-is-runloop) for more detail.
- [API Keys Settings](https://docs.openhands.dev/openhands/usage/settings/api-keys-settings.md): View your OpenHands LLM key and create API keys to work with OpenHands programmatically.
- [Application Settings](https://docs.openhands.dev/openhands/usage/settings/application-settings.md): Configure application-level settings for OpenHands.
- [Integrations Settings](https://docs.openhands.dev/openhands/usage/settings/integrations-settings.md): How to setup and modify the various integrations in OpenHands.
- [Language Model (LLM) Settings](https://docs.openhands.dev/openhands/usage/settings/llm-settings.md): This page goes over how to set the LLM to use in OpenHands. As well as some additional LLM settings.
- [Model Context Protocol (MCP)](https://docs.openhands.dev/openhands/usage/settings/mcp-settings.md): This page outlines how to configure and use the Model Context Protocol (MCP) in OpenHands, allowing you to extend the agent's capabilities with custom tools.
- [Secrets Management](https://docs.openhands.dev/openhands/usage/settings/secrets-settings.md): How to manage secrets in OpenHands.
- [Prompting Best Practices](https://docs.openhands.dev/openhands/usage/tips/prompting-best-practices.md): When working with OpenHands AI software developer, providing clear and effective prompts is key to getting accurate and useful responses. This guide outlines best practices for crafting effective prompts.
- [null](https://docs.openhands.dev/openhands/usage/troubleshooting/feedback.md)
- [Troubleshooting](https://docs.openhands.dev/openhands/usage/troubleshooting/troubleshooting.md)
- [Community](https://docs.openhands.dev/overview/community.md): Learn about the OpenHands community, mission, and values
- [Contributing](https://docs.openhands.dev/overview/contributing.md): Join us in building OpenHands and the future of AI. Learn how to contribute to make a meaningful impact.
- [FAQs](https://docs.openhands.dev/overview/faqs.md): Frequently asked questions about OpenHands.
- [First Projects](https://docs.openhands.dev/overview/first-projects.md): So you've [run OpenHands](/overview/quickstart). Now what?
- [Introduction](https://docs.openhands.dev/overview/introduction.md): Welcome to OpenHands, a community focused on AI-driven development
- [Model Context Protocol (MCP)](https://docs.openhands.dev/overview/model-context-protocol.md): Model Context Protocol support across OpenHands platforms
- [Quick Start](https://docs.openhands.dev/overview/quickstart.md): Running OpenHands Cloud or running on your own.
- [Overview](https://docs.openhands.dev/overview/skills.md): Skills are specialized prompts that enhance OpenHands with domain-specific knowledge, expert guidance, and automated task handling.
- [Keyword-Triggered Skills](https://docs.openhands.dev/overview/skills/keyword.md): Keyword-triggered skills provide OpenHands with specific instructions that are activated when certain keywords appear in the prompt. This is useful for tailoring behavior based on particular tools, languages, or frameworks.
- [Organization and User Skills](https://docs.openhands.dev/overview/skills/org.md): Organizations and users can define skills that apply to all repositories belonging to the organization or user.
- [Global Skills](https://docs.openhands.dev/overview/skills/public.md): Global skills are [keyword-triggered skills](/overview/skills/keyword) that apply to all OpenHands users. The official global skill registry is maintained at [github.com/OpenHands/skills](https://github.com/OpenHands/skills).
- [General Skills](https://docs.openhands.dev/overview/skills/repo.md): General guidelines for OpenHands to work more effectively with the repository.
- [openhands.sdk.agent](https://docs.openhands.dev/sdk/api-reference/openhands.sdk.agent.md): API reference for openhands.sdk.agent module
- [openhands.sdk.conversation](https://docs.openhands.dev/sdk/api-reference/openhands.sdk.conversation.md): API reference for openhands.sdk.conversation module
- [openhands.sdk.event](https://docs.openhands.dev/sdk/api-reference/openhands.sdk.event.md): API reference for openhands.sdk.event module
- [openhands.sdk.llm](https://docs.openhands.dev/sdk/api-reference/openhands.sdk.llm.md): API reference for openhands.sdk.llm module
- [openhands.sdk.security](https://docs.openhands.dev/sdk/api-reference/openhands.sdk.security.md): API reference for openhands.sdk.security module
- [openhands.sdk.tool](https://docs.openhands.dev/sdk/api-reference/openhands.sdk.tool.md): API reference for openhands.sdk.tool module
- [openhands.sdk.utils](https://docs.openhands.dev/sdk/api-reference/openhands.sdk.utils.md): API reference for openhands.sdk.utils module
- [openhands.sdk.workspace](https://docs.openhands.dev/sdk/api-reference/openhands.sdk.workspace.md): API reference for openhands.sdk.workspace module
- [Agent](https://docs.openhands.dev/sdk/arch/agent.md): High-level architecture of the reasoning-action loop
- [Condenser](https://docs.openhands.dev/sdk/arch/condenser.md): High-level architecture of the conversation history compression system
- [Conversation](https://docs.openhands.dev/sdk/arch/conversation.md): High-level architecture of the conversation orchestration system
- [Design Principles](https://docs.openhands.dev/sdk/arch/design.md): Core architectural principles guiding the OpenHands Software Agent SDK's development.
- [Events](https://docs.openhands.dev/sdk/arch/events.md): High-level architecture of the typed event framework
- [LLM](https://docs.openhands.dev/sdk/arch/llm.md): High-level architecture of the provider-agnostic language model interface
- [Overview](https://docs.openhands.dev/sdk/arch/overview.md): Understanding the OpenHands Software Agent SDK's package structure, component interactions, and execution models.
- [Security](https://docs.openhands.dev/sdk/arch/security.md): High-level architecture of action security analysis and validation
- [Skill](https://docs.openhands.dev/sdk/arch/skill.md): High-level architecture of the reusable prompt system
- [Tool System & MCP](https://docs.openhands.dev/sdk/arch/tool-system.md): High-level architecture of the action-observation tool framework
- [Workspace](https://docs.openhands.dev/sdk/arch/workspace.md): High-level architecture of the execution environment abstraction
- [FAQ](https://docs.openhands.dev/sdk/faq.md): Frequently asked questions about the OpenHands SDK
- [Getting Started](https://docs.openhands.dev/sdk/getting-started.md): Install the OpenHands SDK and build AI agents that write software.
- [Browser Use](https://docs.openhands.dev/sdk/guides/agent-browser-use.md): Enable web browsing and interaction capabilities for your agent.
- [Creating Custom Agent](https://docs.openhands.dev/sdk/guides/agent-custom.md): Learn how to design specialized agents with custom tool sets
- [Sub-Agent Delegation](https://docs.openhands.dev/sdk/guides/agent-delegation.md): Enable parallel task execution by delegating work to multiple sub-agents that run independently and return consolidated results.
- [Interactive Terminal](https://docs.openhands.dev/sdk/guides/agent-interactive-terminal.md): Enable agents to interact with terminal applications like ipython, python REPL, and other interactive CLI tools.
- [Batch Get Bash Events](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/bash/batch-get-bash-events.md): Get a batch of bash event events given their ids, returning null for any
missing item.
- [Clear All Bash Events](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/bash/clear-all-bash-events.md): Clear all bash events from storage
- [Execute Bash Command](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/bash/execute-bash-command.md): Execute a bash command and wait for a result
- [Get Bash Event](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/bash/get-bash-event.md): Get a bash event event given an id
- [Search Bash Events](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/bash/search-bash-events.md): Search / List bash event events
- [Start Bash Command](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/bash/start-bash-command.md): Execute a bash command in the background
- [Ask Agent](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/ask-agent.md): Ask the agent a simple question without affecting conversation state.
- [Batch Get Conversations](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/batch-get-conversations.md): Get a batch of conversations given their ids, returning null for
any missing item
- [Condense Conversation](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/condense-conversation.md): Force condensation of the conversation history.
- [Count Conversations](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/count-conversations.md): Count conversations matching the given filters
- [Delete Conversation](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/delete-conversation.md): Permanently delete a conversation.
- [Generate Conversation Title](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/generate-conversation-title.md): Generate a title for the conversation using LLM.
- [Get Conversation](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/get-conversation.md): Given an id, get a conversation
- [Pause Conversation](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/pause-conversation.md): Pause a conversation, allowing it to be resumed later.
- [Run Conversation](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/run-conversation.md): Start running the conversation in the background.
- [Search Conversations](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/search-conversations.md): Search / List conversations
- [Set Conversation Confirmation Policy](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/set-conversation-confirmation-policy.md): Set the confirmation policy for a conversation.
- [Set Conversation Security Analyzer](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/set-conversation-security-analyzer.md): Set the security analyzer for a conversation.
- [Start Conversation](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/start-conversation.md): Start a conversation in the local environment.
- [Update Conversation](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/update-conversation.md): Update conversation metadata.

This endpoint allows updating conversation details like title.
- [Update Conversation Secrets](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/conversations/update-conversation-secrets.md): Update secrets for a conversation.
- [Get Desktop Url](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/desktop/get-desktop-url.md): Get the noVNC URL for desktop access.

Args:
    base_url: Base URL for the noVNC server (default: http://localhost:8002)

Returns:
    noVNC URL if available, None otherwise
- [Batch Get Conversation Events](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/events/batch-get-conversation-events.md): Get a batch of local events given their ids, returning null for any
missing item.
- [Count Conversation Events](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/events/count-conversation-events.md): Count local events matching the given filters
- [Get Conversation Event](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/events/get-conversation-event.md): Get a local event given an id
- [Respond To Confirmation](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/events/respond-to-confirmation.md): Accept or reject a pending action in confirmation mode.
- [Search Conversation Events](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/events/search-conversation-events.md): Search / List local events
- [Send Message](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/events/send-message.md): Send a message to a conversation
- [Download File](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/files/download-file.md): Download a file from the workspace.
- [Download Trajectory](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/files/download-trajectory.md): Download a file from the workspace.
- [Upload File](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/files/upload-file.md): Upload a file to the workspace.
- [Get Server Info](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/get-server-info.md)
- [Git Changes](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/git/git-changes.md)

---

## UV Cheatsheet for AGENTS

**CRITICAL: For `uv` commands, prefix with `cd path/to/dir && . .venv/bin/activate &&`**

**NEVER use #! shebang lines for project scripts - always use .venv/bin/python**

**Exception: For installable scripts in ~/.local/bin, use full venv path:**
```bash
#!/path/to/crow/.venv/bin/python
```

### Essential Commands

```bash
# Setup project
cd path/to/dir && . .venv/bin/activate && uv sync
cd path/to/dir && . .venv/bin/activate && uv add ../acp-python-sdk/  # Add local dependency

# Install dependencies
cd path/to/dir && . .venv/bin/activate && uv add requests
cd path/to/dir && . .venv/bin/activate && uv remove requests
cd path/to/dir && . .venv/bin/activate && uv sync

# Run Python commands - ALWAYS use .venv/bin/python
.venv/bin/python script.py
.venv/bin/python -m module.cli
.venv/bin/pytest tests/
.venv/bin/ruff check .
.venv/bin/ruff format .
```

--- 
SUPERCRITICAL INFO BELOW:

# **AGENTS.MD LESSONS FROM CROW-AGENT STREAMING WORK:**

1. **DO NOT use try/except blocks to hide errors** - Let all errors propagate visibly so root causes can be found and fixed
2. **READ the actual code, don't guess API** - Check the actual imports and method signatures in the codebase before making changes
3. **Build on OpenHands SDK directly** - Don't depend on openhands_cli or external setup scripts
4. **ALL imports at TOP of file** - Never import modules inside functions/methods to avoid scope issues
5. **Use Edit tool to modify files** - Never use Write to completely rewrite files (loses context, history, and breaks git tracking)
6. **ALWAYS use .venv/bin/python** - Never rely on system Python or bare `python` command
7. **NEVER use `git checkout` to revert** - It's destructive and wipes uncommitted work; always use Edit to fix specific issues

### Essential Commands

```bash
# Setup project
cd path/to/dir && . .venv/bin/activate && uv sync
cd path/to/dir && . .venv/bin/activate && uv add ../acp-python-sdk/  # Add local dependency

# Install dependencies
cd path/to/dir && . .venv/bin/activate && uv add requests
cd path/to/dir && . .venv/bin/activate && uv remove requests
cd path/to/dir && . .venv/bin/activate && uv sync

# Run Python commands
.venv/bin/python script.py
.venv/bin/python -m module.cli
.venv/bin/pytest tests/
.venv/bin/ruff check .
.venv/bin/ruff format .
```

- [Git Diff](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/git/git-diff.md)
- [Alive](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/server-details/alive.md)
- [Get Server Info](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/server-details/get-server-info.md)
- [Health](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/server-details/health.md)
- [List Available Tools](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/tools/list-available-tools.md): List all available tools.
- [Get Vscode Status](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/vscode/get-vscode-status.md): Get the VSCode server status.

Returns:
    Dictionary with running status and enabled status
- [Get Vscode Url](https://docs.openhands.dev/sdk/guides/agent-server/api-reference/vscode/get-vscode-url.md): Get the VSCode URL with authentication token.

Args:
    base_url: Base URL for the VSCode server (default: http://localhost:8001)
    workspace_dir: Path to workspace directory

Returns:
    VSCode URL with token if available, None otherwise
- [API-based Sandbox](https://docs.openhands.dev/sdk/guides/agent-server/api-sandbox.md): Connect to hosted API-based agent server for fully managed infrastructure.
- [Apptainer Sandbox](https://docs.openhands.dev/sdk/guides/agent-server/apptainer-sandbox.md): Run agent server in rootless Apptainer containers for HPC and shared computing environments.
- [OpenHands Cloud Workspace](https://docs.openhands.dev/sdk/guides/agent-server/cloud-workspace.md): Connect to OpenHands Cloud for fully managed sandbox environments.
- [Custom Tools with Remote Agent Server](https://docs.openhands.dev/sdk/guides/agent-server/custom-tools.md): Learn how to use custom tools with a remote agent server by building a custom base image that includes your tool implementations.
- [Docker Sandbox](https://docs.openhands.dev/sdk/guides/agent-server/docker-sandbox.md): Run agent server in isolated Docker containers for security and reproducibility.
- [Local Agent Server](https://docs.openhands.dev/sdk/guides/agent-server/local-server.md): Run agents through a local HTTP server with RemoteConversation for client-server architecture.
- [Overview](https://docs.openhands.dev/sdk/guides/agent-server/overview.md): Run agents on remote servers with isolated workspaces for production deployments.
- [Stuck Detector](https://docs.openhands.dev/sdk/guides/agent-stuck-detector.md): Detect and handle stuck agents automatically with timeout mechanisms.
- [Theory of Mind (TOM) Agent](https://docs.openhands.dev/sdk/guides/agent-tom-agent.md): Enable your agent to understand user intent and preferences through Theory of Mind capabilities, providing personalized guidance based on user modeling.
- [Context Condenser](https://docs.openhands.dev/sdk/guides/context-condenser.md): Manage agent memory by condensing conversation history to save tokens.
- [Ask Agent Questions](https://docs.openhands.dev/sdk/guides/convo-ask-agent.md): Get sidebar replies from the agent during conversation execution without interrupting the main flow.
- [Conversation with Async](https://docs.openhands.dev/sdk/guides/convo-async.md): Use async/await for concurrent agent operations and non-blocking execution.
- [Custom Visualizer](https://docs.openhands.dev/sdk/guides/convo-custom-visualizer.md): Customize conversation visualization by creating custom visualizers or configuring the default visualizer.
- [Pause and Resume](https://docs.openhands.dev/sdk/guides/convo-pause-and-resume.md): Pause agent execution, perform operations, and resume without losing state.
- [Persistence](https://docs.openhands.dev/sdk/guides/convo-persistence.md): Save and restore conversation state for multi-session workflows.
- [Send Message While Running](https://docs.openhands.dev/sdk/guides/convo-send-message-while-running.md): Interrupt running agents to provide additional context or corrections.
- [Custom Tools](https://docs.openhands.dev/sdk/guides/custom-tools.md): Tools define what agents can do. The SDK includes built-in tools for common operations and supports creating custom tools for specialized needs.
- [Assign Reviews](https://docs.openhands.dev/sdk/guides/github-workflows/assign-reviews.md): Automate PR management with intelligent reviewer assignment and workflow notifications using OpenHands Agent
- [PR Review](https://docs.openhands.dev/sdk/guides/github-workflows/pr-review.md): Use OpenHands Agent to generate meaningful pull request review
- [TODO Management](https://docs.openhands.dev/sdk/guides/github-workflows/todo-management.md): Implement TODOs using OpenHands Agent
- [Hello World](https://docs.openhands.dev/sdk/guides/hello-world.md): The simplest possible OpenHands agent - configure an LLM, create an agent, and complete a task.
- [Hooks](https://docs.openhands.dev/sdk/guides/hooks.md): Use lifecycle hooks to observe, log, and customize agent execution.
- [Iterative Refinement](https://docs.openhands.dev/sdk/guides/iterative-refinement.md): Implement iterative refinement workflows where agents refine their work based on critique feedback until quality thresholds are met.
- [Exception Handling](https://docs.openhands.dev/sdk/guides/llm-error-handling.md): Provider‑agnostic exceptions raised by the SDK and recommended patterns for handling them.
- [Image Input](https://docs.openhands.dev/sdk/guides/llm-image-input.md): Send images to multimodal agents for vision-based tasks and analysis.
- [Reasoning](https://docs.openhands.dev/sdk/guides/llm-reasoning.md): Access model reasoning traces from Anthropic extended thinking and OpenAI responses API.
- [LLM Registry](https://docs.openhands.dev/sdk/guides/llm-registry.md): Dynamically select and configure language models using the LLM registry.
- [Model Routing](https://docs.openhands.dev/sdk/guides/llm-routing.md): Route agent's LLM requests to different models.
- [LLM Streaming](https://docs.openhands.dev/sdk/guides/llm-streaming.md): Stream LLM responses token-by-token for real-time display and interactive user experiences.
- [Model Context Protocol](https://docs.openhands.dev/sdk/guides/mcp.md): Model Context Protocol (MCP) enables dynamic tool integration from external servers. Agents can discover and use MCP-provided tools automatically.
- [Metrics Tracking](https://docs.openhands.dev/sdk/guides/metrics.md): Track token usage, costs, and latency metrics for your agents.
- [Observability & Tracing](https://docs.openhands.dev/sdk/guides/observability.md): Enable OpenTelemetry tracing to monitor and debug your agent's execution with tools like Laminar, Honeycomb, or any OTLP-compatible backend.
- [Secret Registry](https://docs.openhands.dev/sdk/guides/secrets.md): Provide environment variables and secrets to agent workspace securely.
- [Security & Action Confirmation](https://docs.openhands.dev/sdk/guides/security.md): Control agent action execution through confirmation policy and security analyzer.
- [Agent Skills & Context](https://docs.openhands.dev/sdk/guides/skill.md): Skills add specialized behaviors, domain knowledge, and context-aware triggers to your agent through structured prompts.
- [Software Agent SDK](https://docs.openhands.dev/sdk/index.md): Build AI agents that write software. A clean, modular SDK with production-ready tools.


## Optional

- [Company](https://openhands.dev/)
- [Blog](https://openhands.dev/blog)
- [OpenHands Cloud](https://app.all-hands.dev)
