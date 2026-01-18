#!/home/thomas/src/projects/orchestrator-project/crow/.venv/bin/python
"""
PostToolUse hook for Crow ACP Server.
Captures tool results and sends them to the ACP client.

This hook reads tool execution results from stdin (JSON format)
and writes them to a file-based queue that the ACP server reads from.

Environment variables:
- CROW_ACP_SESSION_ID: Session ID for the ACP session
- CROW_ACP_QUEUE_DIR: Directory for queue files
"""

import json
import os
import sys
from pathlib import Path


def main():
    """Read hook event from stdin and write tool result to queue."""
    # Read JSON from stdin
    event_json = sys.stdin.read()
    if not event_json:
        return 0

    try:
        event = json.loads(event_json)

        # Only process PostToolUse events
        if event.get("event_type") != "PostToolUse":
            return 0

        tool_name = event.get("tool_name")
        tool_response = event.get("tool_response")
        session_id = os.getenv("CROW_ACP_SESSION_ID")

        if not tool_name or not tool_response or not session_id:
            return 0

        # Get queue directory from environment
        queue_dir = os.getenv("CROW_ACP_QUEUE_DIR")
        if not queue_dir:
            return 0

        # Write tool result to queue file
        queue_file = Path(queue_dir) / f"{session_id}.jsonl"

        result = {
            "type": "tool_result",
            "tool_name": tool_name,
            "tool_response": tool_response,
            "session_id": session_id,
        }

        # Append to queue file (JSONL format)
        with open(queue_file, "a") as f:
            f.write(json.dumps(result) + "\n")

    except Exception as e:
        # Don't fail the hook if queue write fails
        print(f"[Crow ACP Hook] Error: {e}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
