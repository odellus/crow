#!/usr/bin/env python3
"""Test that mimics Zed's ACP client requests."""

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


def test():
    """Test the ACP server like Zed would."""
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
                    "client_info": {"name": "zed", "version": "1.0.0"},
                },
            },
        )
        response = read_response(proc)
        if not response or "error" in response:
            print(f"✗ Initialize failed", file=sys.stderr)
            return

        print("✓ Initialize successful", file=sys.stderr)

        # 2. Try load_session first (maybe Zed does this?)
        print("\n=== Testing session/load (like Zed might) ===", file=sys.stderr)
        request_id += 1
        send_request(
            proc,
            {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "session/load",
                "params": {
                    "session_id": "test-session-123",
                    "cwd": str(Path.cwd()),
                    "mcp_servers": [],
                },
            },
        )
        response = read_response(proc, timeout=5)

        if response and "error" in response:
            error = response["error"]
            print(
                f"session/load error: {error.get('message', 'Unknown')}",
                file=sys.stderr,
            )
        elif response:
            print("✓ Session/load succeeded", file=sys.stderr)
        else:
            print("✓ Session/load not supported (expected)", file=sys.stderr)

        # 3. Create new session
        print("\n=== Testing session/new ===", file=sys.stderr)
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
            print(f"✗ Session/new failed: {response}", file=sys.stderr)
            return

        session_id = response.get("result", {}).get("sessionId")
        if not session_id:
            print(f"✗ No sessionId in response", file=sys.stderr)
            return

        print(f"✓ Session created: {session_id}", file=sys.stderr)

    finally:
        print("\n=== Cleaning up ===", file=sys.stderr)
        proc.stdin.close()
        proc.terminate()
        proc.wait(timeout=5)
        print("✓ Server stopped", file=sys.stderr)

        print("✓ Server stopped", file=sys.stderr)

if __name__ == "__main__":
    test()
