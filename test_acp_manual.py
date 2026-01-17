#!/usr/bin/env python3
"""Manual test script for ACP server using subprocess and pipes."""

import json
import subprocess
import sys
from pathlib import Path


def send_request(proc, request):
    """Send a JSON-RPC request to the server."""
    request_str = json.dumps(request) + "\n"
    print(f"→ Sending: {request_str.strip()}", file=sys.stderr)
    proc.stdin.write(request_str)
    proc.stdin.flush()


def read_response(proc, timeout=5):
    """Read a JSON-RPC response from the server."""
    import select

    # Wait for data with timeout
    ready, _, _ = select.select([proc.stdout], [], [], timeout)
    if ready:
        line = proc.stdout.readline()
        if line:
            response = json.loads(line)
            print(f"← Received: {json.dumps(response, indent=2)}", file=sys.stderr)
            return response

    print(f"✗ No response within {timeout}s", file=sys.stderr)
    return None


def main():
    """Test the ACP server."""
    print("Starting ACP server...", file=sys.stderr)

    # Start the server process using the installed crow-acp executable
    proc = subprocess.Popen(
        ["crow"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,  # Line buffered
    )

    # Start a thread to read stderr
    import threading

    def read_stderr():
        for line in proc.stderr:
            print(f"[SERVER STDERR] {line}", end="", file=sys.stderr)

    threading.Thread(target=read_stderr, daemon=True).start()

    request_id = 0

    try:
        # 1. Initialize
        print("\n=== Testing initialize ===", file=sys.stderr)
        request_id += 1
        send_request(
            proc,
            {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "initialize",
                "params": {
                    "protocol_version": 1,
                    "client_capabilities": {},
                    "client_info": {"name": "test-client", "version": "1.0.0"},
                },
            },
        )
        response = read_response(proc)
        if not response or "error" in response:
            print(f"✗ Initialize failed", file=sys.stderr)
            return

        print("✓ Initialize successful", file=sys.stderr)

        # 2. Create new session
        print("\n=== Testing session/new ===", file=sys.stderr)
        request_id += 1
        send_request(
            proc,
            {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "session/new",
                "params": {
                    "cwd": str(Path.cwd()),
                    "mcp_servers": [],  # Empty list for no MCP servers
                },
            },
        )
        response = read_response(proc, timeout=10)
        if not response or "error" in response:
            print(f"✗ Session/new failed", file=sys.stderr)
            return

        session_id = response.get("result", {}).get("sessionId")
        if not session_id:
            print(f"✗ No sessionId in response", file=sys.stderr)
            return

        print(f"✓ Session created: {session_id}", file=sys.stderr)

        # 3. Send a prompt
        print("\n=== Testing session/prompt ===", file=sys.stderr)
        request_id += 1
        send_request(
            proc,
            {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "session/prompt",
                "params": {
                    "session_id": session_id,
                    "prompt": [
                        {
                            "type": "text",
                            "text": "Hello! Can you tell me what 2+2 equals?",
                        }
                    ],
                },
            },
        )

        # Read session/update notifications until we get the final response
        print("Reading streaming updates...", file=sys.stderr)
        final_response = None
        while True:
            response = read_response(proc, timeout=10)
            if not response:
                break

            # Check if this is a notification or response
            if "method" in response and response["method"] == "session/update":
                # This is a streaming update
                update = response.get("params", {}).get("update", {})
                update_type = update.get("@type", "unknown")
                print(f"  Update: {update_type}", file=sys.stderr)

                # Show some details
                if update_type == "agent_message_text":
                    text = update.get("text", "")[:100]
                    print(f"    Text: {text}...", file=sys.stderr)
                elif update_type == "tool_call_start":
                    print(
                        f"    Tool: {update.get('title', 'unknown')}", file=sys.stderr
                    )

            elif "id" in response:
                # This is the final response
                final_response = response
                break

        if final_response:
            print(f"\n✓ Prompt completed", file=sys.stderr)
            result = final_response.get("result", {})
            stop_reason = result.get("stop_reason", "unknown")
            print(f"Stop reason: {stop_reason}", file=sys.stderr)
        else:
            print(f"\n✗ No final response received", file=sys.stderr)

    finally:
        print("\n=== Cleaning up ===", file=sys.stderr)
        proc.stdin.close()
        proc.terminate()
        proc.wait(timeout=5)
        print("✓ Server stopped", file=sys.stderr)


if __name__ == "__main__":
    main()
