#!/usr/bin/env python3
"""Simple test script to verify crow ACP server works."""

import json
import subprocess
import sys
import time


def test_acp_server():
    """Test the Crow ACP server with a simple initialize -> new session flow."""

    # Server startup command
    cmd = """cd /home/thomas/src/projects/orchestrator-project/crow && . .venv/bin/activate && exec crow-acp"""

    print("🚀 Starting Crow ACP server...")
    server = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        shell=True,
        bufsize=0,
    )

    def send(msg):
        """Send a JSON-RPC message."""
        data = json.dumps(msg) + "\n"
        server.stdin.write(data.encode())
        server.stdin.flush()
        print(f"⬆️  Sent: {msg['method']}")

    def read_response(timeout=5):
        """Read a JSON-RPC response."""
        import select

        ready, _, _ = select.select([server.stdout], [], [], timeout)
        if ready:
            line = server.stdout.readline().decode().rstrip()
            if line:
                resp = json.loads(line)
                print(f"⬇️  Received: {resp.get('method', 'response')}")
                return resp
        return None

    # Give server time to start
    time.sleep(1)

    try:
        # Initialize
        send(
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": 1,
                    "clientCapabilities": {
                        "fs": {"readTextFile": True, "writeTextFile": True},
                        "terminal": True,
                    },
                    "clientInfo": {"name": "test", "version": "1.0"},
                },
            }
        )

        resp = read_response(5)
        if resp and "result" in resp:
            print("✅ Initialize successful")
        else:
            print(f"❌ Initialize failed: {resp}")
            return False

        # Create new session
        send(
            {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "session/new",
                "params": {"cwd": "/tmp", "mcpServers": []},
            }
        )

        resp = read_response(10)
        if resp and "result" in resp:
            print("✅ New session successful")
            session_id = resp["result"].get("sessionId", "N/A")
            print(f"📋 Session ID: {session_id}")
        else:
            print(f"❌ New session failed: {resp}")
            return False

        print("\n🎉 All tests passed!")
        return True

    finally:
        # Cleanup
        print("\n🧹 Cleaning up...")
        if server.poll() is None:
            server.terminate()
            server.wait(timeout=2)

        # Print stderr if there was any
        stderr = server.stderr.read().decode()
        if stderr:
            print(f"\n📛 STDERR:\n{stderr}")


if __name__ == "__main__":
    success = test_acp_server()
    sys.exit(0 if success else 1)
