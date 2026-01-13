#!/usr/bin/env python3
"""Test crow ACP server directly in same process."""

import asyncio
import json
import sys
import threading
import time

from acp import run_agent

from crow.agent.acp_server import CrowAcpAgent


async def test_crow_acp():
    """Test the ACP server with a simple prompt."""

    # Start server in background thread
    def run_server():
        try:
            asyncio.run(run_agent(CrowAcpAgent()))
        except Exception as e:
            print(f"[SERVER ERROR] {e}")

    server_thread = threading.Thread(target=run_server, daemon=True)
    server_thread.start()

    # Give server time to start
    await asyncio.sleep(1)

    # For now, just test that we can import and instantiate
    agent = CrowAcpAgent()
    print(f"✅ Agent created: {agent._server_config.name}")
    print(f"✅ LLM model: {agent._llm_config.model}")

    print("\nServer is running. Use debug_client.py to interact with it.")
    print("Press Ctrl+C to exit.")

    # Keep running
    try:
        while True:
            await asyncio.sleep(1)
    except KeyboardInterrupt:
        print("\nShutting down...")


if __name__ == "__main__":
    asyncio.run(test_crow_acp())
