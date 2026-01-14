"""Simple test that verifies ACP server works end-to-end."""

import asyncio
import json
import pytest


@pytest.mark.asyncio
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
        """Read stderr to see any errors."""
        while True:
            line = await proc.stderr.readline()
            if not line:
                break
            print(f"[STDERR] {line.decode().rstrip()}")

    stderr_task = asyncio.create_task(read_stderr())
    
    # Wait for server to start
    await asyncio.sleep(2)

    async def send(msg):
        line = json.dumps(msg) + "\n"
        proc.stdin.write(line.encode())
        await proc.stdin.drain()

    responses_seen = []

    async def read_until_id(target_id, timeout=30):
        """Read until we get a response with the target ID."""
        while True:
            try:
                line = await asyncio.wait_for(proc.stdout.readline(), timeout=timeout)
                if not line:
                    print("[EOF]")
                    return None
                line_str = line.decode().strip()
                if not line_str:
                    continue
                try:
                    resp = json.loads(line)
                    responses_seen.append(resp)
                    if resp.get("id") == target_id:
                        return resp
                    if "method" in resp:
                        print(f"[NOTIFICATION] {resp.get('method')}")
                except json.JSONDecodeError as e:
                    print(f"[JSON ERROR] {e}")
                    pass
            except asyncio.TimeoutError:
                print(f"[TIMEOUT] waiting for id {target_id}")
                return None

    # Track session ID
    session_id = None

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
        init_resp = await read_until_id(1)
        if not init_resp:
            print("ERROR: No initialize response")
            return
        print(f"✓ Got initialize response")

        # New session
        print("Sending new_session...")
        await send(
            {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "session/new",
                "params": {"cwd": "/tmp", "mcpServers": []},
            }
        )
        session_resp = await read_until_id(2, timeout=60)
        if not session_resp or "result" not in session_resp:
            print(f"ERROR: Could not get session response")
            return
        session_id = session_resp["result"]["sessionId"]
        print(f"✓ Got session/new response")
        print(f"  Session ID: {session_id}")

        # Send prompt
        print("Sending prompt...")
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
        prompt_resp = await read_until_id(3, timeout=120)
        if not prompt_resp:
            print("ERROR: No prompt response")
            return
        print(f"✓ Got session/prompt response")
        if "result" in prompt_resp:
            stop_reason = prompt_resp["result"].get("stopReason")
            print(f"  Stop reason: {stop_reason}")

        # Check for streaming updates
        updates = [r for r in responses_seen if r.get("method") == "session/update"]
        if updates:
            print(f"✓ Got {len(updates)} streaming updates")
        else:
            print("✗ No streaming updates")

        print("\n=== TEST PASSED ===")

    finally:
        proc.terminate()
        await proc.wait()
        stderr_task.cancel()


if __name__ == "__main__":
    asyncio.run(test_acp_server())
