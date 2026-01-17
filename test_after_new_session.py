#!/usr/bin/env python3
"""Test prompt after session/new to see if the error is there."""

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

    # Start the server process
    proc = subprocess.Popen(
        ["crow-acp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
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

        # 2. Create new session
        request_id += 1
        send_request(
            proc,
            {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "session/new",
                "params": {"cwd": str(Path.cwd()), "mcp_servers": []},
            },
        )
        response = read_response(proc, timeout=10)
        if not response or "error" in response:
            print(f"✗ Session/new failed", file=sys.stderr)
            return

        session_id = response.get("result", {}).get("sessionId")
        print(f"✓ Session created: {session_id}", file=sys.stderr)

        # 3. Send a prompt - THIS IS WHERE THE ERROR MIGHT BE
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
                    "prompt": [{"type": "text", "text": "Hello"}],
                },
            },
        )

        # Read responses
        while True:
            response = read_response(proc, timeout=10)
            if not response:
                break
            if "id" in response:
                break

        print(f"\n✓ Test completed", file=sys.stderr)

    finally:
        print("\n=== Cleaning up ===", file=sys.stderr)
        proc.stdin.close()
        proc.terminate()
        proc.wait(timeout=5)


if __name__ == "__main__":
    main()
