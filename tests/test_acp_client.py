"""
Simple ACP client for testing Crow ACP server.

This is a standalone test client that connects to the Crow ACP server
and allows interactive testing. Useful for debugging without Zed.

Usage:
    uv --project ./crow run python test_acp_client.py
"""

import asyncio
import logging
import sys

from acp import Client, RequestError, spawn_agent_process, text_block
from acp.schema import (
    AgentMessageChunk,
    TextContentBlock,
)


class TestClient(Client):
    """Minimal test client for Crow ACP server."""

    async def session_update(self, session_id: str, update, **kwargs):
        """Handle session updates from the agent."""
        if isinstance(update, AgentMessageChunk):
            content = update.content
            if isinstance(content, TextContentBlock):
                print(f"Agent: {content.text}")

    async def request_permission(self, options, session_id, tool_call, **kwargs):
        """Auto-approve all permissions for testing."""
        from acp.schema import RequestPermissionResponse, PermissionOutcome
        return RequestPermissionResponse(
            outcome=PermissionOutcome(option_id="allow_once")
        )

    async def ext_method(self, method: str, params: dict):
        raise RequestError.method_not_found(method)

    async def ext_notification(self, method: str, params: dict):
        pass


async def main():
    """Run test client using spawn_agent_process helper."""
    logging.basicConfig(level=logging.INFO)

    # Use spawn_agent_process helper - this is the recommended way!
    # It handles stdio wiring, process management, and cleanup automatically
    async with spawn_agent_process(TestClient(), "crow") as (conn, _proc):
        # Initialize connection
        await conn.initialize(
            protocol_version=1,  # Integer, not string!
        )

        print("Connected to Crow ACP server", file=sys.stderr)

        # Create session
        session = await conn.new_session(mcp_servers=[], cwd=".")
        print(f"Created session: {session.session_id}", file=sys.stderr)

        # Send a test prompt
        prompt = "Hello! Can you tell me what you can do?"
        print(f"Sending prompt: {prompt}", file=sys.stderr)

        await conn.prompt(
            session_id=session.session_id,
            prompt=[text_block(prompt)],
        )

        # Wait for responses
        await asyncio.sleep(2)

        print("Test complete!", file=sys.stderr)


if __name__ == "__main__":
    asyncio.run(main())
