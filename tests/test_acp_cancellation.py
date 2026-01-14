"""Test ACP server cancellation functionality."""

import asyncio
import json


async def test_acp_cancellation():
    """Test the ACP server cancellation by spawning it as a subprocess."""

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
        assert resp is not None
        assert "result" in resp

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
        assert resp is not None
        assert "result" in resp

        session_id = resp["result"]["sessionId"]
        print(f"\nSession ID: {session_id}")

        # Send prompt that will trigger a long-running tool
        print("\nSending prompt (long task)...")
        await send(
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "session/prompt",
                "params": {
                    "sessionId": session_id,
                    "prompt": [
                        {
                            "type": "text",
                            "text": "run 'sleep 10' in the terminal to test cancellation",
                        }
                    ],
                },
            }
        )

        # Wait for the tool to start executing
        print("Waiting for tool to start...")
        await asyncio.sleep(2)

        # Send cancellation notification (not a request - no id field)
        print("\nSending cancel notification...")
        await send(
            {
                "jsonrpc": "2.0",
                "method": "session/cancel",
                "params": {"sessionId": session_id},
            }
        )

        # Read the prompt response - should have stopReason: "cancelled"
        print("\nWaiting for prompt response...")
        resp = await read_until_id(3, timeout=45)
        print(f"Prompt response: {resp}")
        assert resp is not None
        assert "result" in resp
        assert resp["result"].get("stopReason") == "cancelled"

        print("\n✅ Cancellation test passed!")

    finally:
        proc.terminate()
        await proc.wait()
        stderr_task.cancel()


async def test_acp_normal_completion():
    """Test that normal completion still works (stopReason: end_turn)."""

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
        assert resp is not None

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
        assert resp is not None

        session_id = resp["result"]["sessionId"]
        print(f"\nSession ID: {session_id}")

        # Send a simple prompt that completes normally
        print("\nSending prompt (simple task)...")
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
        assert resp is not None
        assert "result" in resp
        assert resp["result"].get("stopReason") == "end_turn"

        print("\n✅ Normal completion test passed!")

    finally:
        proc.terminate()
        await proc.wait()
        stderr_task.cancel()


if __name__ == "__main__":
    print("=" * 60)
    print("Test 1: Normal Completion")
    print("=" * 60)
    asyncio.run(test_acp_normal_completion())

    print("\n" + "=" * 60)
    print("Test 2: Cancellation")
    print("=" * 60)
    asyncio.run(test_acp_cancellation())
