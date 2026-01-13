#!/usr/bin/env python3
"""Quick test of crow ACP server."""

import asyncio
import json


async def test_crow_acp():
    """Test the ACP server with a simple prompt."""
    proc = await asyncio.create_subprocess_exec(
        ".venv/bin/python",
        "-m",
        "crow.agent.acp_server",
        stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )

    async def read_stderr():
        while True:
            line = await proc.stderr.readline()
            if not line:
                break
            print(f"[STDERR] {line.decode().rstrip()}")

    stderr_task = asyncio.create_task(read_stderr())

    async def send(msg):
        line = json.dumps(msg) + "\n"
        proc.stdin.write(line.encode())
        await proc.stdin.drain()

    async def read_until_id(target_id, timeout=30):
        while True:
            line = await asyncio.wait_for(proc.stdout.readline(), timeout=timeout)
            if not line:
                print("[EOF]")
                return None
            resp = json.loads(line)
            if resp.get("id") == target_id:
                return resp
            if "method" in resp:
                print(f"[NOTIFICATION] {resp.get('method')}")

    # Initialize
    print("Sending initialize...")
    await send(
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": 1,
                "clientCapabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"},
            },
        }
    )
    resp = await read_until_id(1)
    print(f"Initialize response: {resp}")

    # New session
    print("\nSending new_session...")
    await send(
        {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "session/new",
            "params": {"cwd": "/tmp", "mcpServers": []},
        }
    )
    resp = await read_until_id(2, timeout=60)
    print(f"Session response: {resp}")

    session_id = resp["result"]["sessionId"]
    print(f"\nSession ID: {session_id}")

    # Send prompt
    print("\nSending prompt...")
    await send(
        {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "session/prompt",
            "params": {
                "sessionId": session_id,
                "prompt": [{"type": "text", "text": "say hello"}],
            },
        }
    )
    resp = await read_until_id(3, timeout=120)
    print(f"Prompt response: {resp}")

    proc.terminate()
    await proc.wait()
    stderr_task.cancel()


if __name__ == "__main__":
    asyncio.run(test_crow_acp())
