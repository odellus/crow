# Headless Mode

> Run OpenHands without UI for scripting, automation, and CI/CD pipelines

## Overview

Headless mode runs OpenHands without the interactive terminal UI, making it ideal for:

* CI/CD pipelines
* Automated scripting
* Integration with other tools
* Batch processing

```bash  theme={null}
openhands --headless -t "Your task here"
```

## Requirements

* Must specify a task with `--task` or `--file`

<Warning>
  **Headless mode always runs in `always-approve` mode.** The agent will execute all actions without any confirmation. This cannot be changed—`--llm-approve` is not available in headless mode.
</Warning>

## Basic Usage

```bash  theme={null}
# Run a task in headless mode
openhands --headless -t "Write a Python script that prints hello world"

# Load task from a file
openhands --headless -f task.txt
```

## JSON Output Mode

The `--json` flag enables structured JSONL (JSON Lines) output, streaming events as they occur:

```bash  theme={null}
openhands --headless --json -t "Create a simple Flask app"
```

Each line is a JSON object representing an agent event:

```json  theme={null}
{"type": "action", "action": "write", "path": "app.py", ...}
{"type": "observation", "content": "File created successfully", ...}
{"type": "action", "action": "run", "command": "python app.py", ...}
```

### Use Cases for JSON Output

* **CI/CD pipelines**: Parse events to determine success/failure
* **Automated processing**: Feed output to other tools
* **Logging**: Capture structured logs for analysis
* **Integration**: Connect OpenHands with other systems

### Example: Capture Output to File

```bash  theme={null}
openhands --headless --json -t "Add unit tests" > output.jsonl
```

## See Also

* [Terminal Mode](/openhands/usage/cli/terminal) - Interactive CLI usage
* [Command Reference](/openhands/usage/cli/command-reference) - All CLI options


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt