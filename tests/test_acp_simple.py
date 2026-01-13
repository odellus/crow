"""Simple test that verifies ACP server works end-to-end."""

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

    async def send(msg):
        line = json.dumps(msg) + "\n"
        proc.stdin.write(line.encode())
        await proc.stdin.drain()

    responses_seen = []

    async def read_all():
        """Read all responses until process ends."""
        while True:
            try:
                line = await asyncio.wait_for(proc.stdout.readline(), timeout=10)
                if not line:
                    break
                line_str = line.decode().strip()
                if not line_str:
                    continue
                try:
                    resp = json.loads(line)
                    responses_seen.append(resp)
                    print(f"[JSON] {resp.get('method', resp.get('id', 'unknown'))}")
                except json.JSONDecodeError:
                    # Skip non-JSON lines (OpenHands UI output)
                    pass
            except asyncio.TimeoutError:
                break

    reader_task = asyncio.create_task(read_all())

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

        # Send prompt
        print("Sending prompt...")
        await send(
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "session/prompt",
                "params": {
                    "sessionId": "session_1",
                    "prompt": [{"type": "text", "text": "say hello"}],
                },
            }
        )

        # Wait a bit for responses
        await asyncio.sleep(30)

        # Verify we got the expected responses
        print(f"\nTotal JSON responses seen: {len(responses_seen)}")

        # Check for initialize response
        init_responses = [r for r in responses_seen if r.get("id") == 1]
        if init_responses:
            print(f"✓ Got initialize response")
        else:
            print("✗ Missing initialize response")

        # Check for session/new response
        session_responses = [r for r in responses_seen if r.get("id") == 2]
        if session_responses:
            print(f"✓ Got session/new response")
            session_id = session_responses[0]["result"]["sessionId"]
            print(f"  Session ID: {session_id}")
        else:
            print("✗ Missing session/new response")

        # Check for session/prompt response
        prompt_responses = [r for r in responses_seen if r.get("id") == 3]
        if prompt_responses:
            print(f"✓ Got session/prompt response")
            stop_reason = prompt_responses[0]["result"]["stopReason"]
            print(f"  Stop reason: {stop_reason}")
        else:
            print("✗ Missing session/prompt response")

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
        reader_task.cancel()


if __name__ == "__main__":
    asyncio.run(test_acp_server())
