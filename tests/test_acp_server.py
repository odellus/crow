"""Test ACP server with simple client."""

import asyncio
import json


async def test_acp_server():
    """Test the ACP server by spawning it as a subprocess."""

    # Start the server
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
            line_str = line.decode().strip()
            if not line_str:
                print("[EMPTY LINE]")
                continue
            try:
                resp = json.loads(line)
                if resp.get("id") == target_id:
                    return resp
                if "method" in resp:
                    print(f"[NOTIFICATION] {resp.get('method')}")
                    # Print update details
                    if resp.get("method") == "session/update":
                        update = resp.get("params", {}).get("update", {})
                        update_type = update.get("sessionUpdate", "unknown")
                        if update_type == "agent_message_chunk":
                            content = update.get("content", {})
                            text = content.get("text", "")
                            print(f"  -> Content: {text}")
                        elif update_type == "agent_thought_chunk":
                            content = update.get("content", {})
                            text = content.get("text", "")
                            print(f"  -> Thought: {text}")
            except json.JSONDecodeError as e:
                print(f"[JSON ERROR] {e}")
                print(f"[RAW LINE] {line_str}")

    try:
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

    finally:
        proc.terminate()
        await proc.wait()
        stderr_task.cancel()


if __name__ == "__main__":
    asyncio.run(test_acp_server())
