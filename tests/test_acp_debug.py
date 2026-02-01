"""Debug test to see raw ACP server output."""

import asyncio
import json


async def test_acp_server():
    """Test the ACP server by spawning it as a subprocess."""

    # Start the server
    proc = await asyncio.create_subprocess_exec(
        "uv",
        "--project",
        ".",
        "run",
        "python",
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
            print(f"[STDERR] {line.decode().rstrip()}", file=__import__('sys').stderr)

    stderr_task = asyncio.create_task(read_stderr())
    
    # Wait for server to start
    await asyncio.sleep(2)

    async def send(msg):
        line = json.dumps(msg) + "\n"
        print(f"[SEND] {line.rstrip()}")
        proc.stdin.write(line.encode())
        await proc.stdin.drain()

    async def read_raw():
        """Read raw output."""
        for _ in range(100):
            line = await proc.stdout.readline()
            if not line:
                print("[EOF]")
                break
            line_str = line.decode()
            print(f"[RECV RAW] {repr(line_str)}")

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
        
        await read_raw()

    finally:
        proc.terminate()
        await proc.wait()
        stderr_task.cancel()


if __name__ == "__main__":
    asyncio.run(test_acp_server())
