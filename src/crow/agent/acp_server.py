#!/usr/bin/env python3
"""
Minimal ACP Server wrapping OpenHands SDK with proper streaming.

This server extends the ACP Agent base class and uses OpenHands SDK
to run an AI agent with MCP tool support.
"""

import asyncio
import os
import sys
from typing import Any, Literal

from dotenv import load_dotenv

load_dotenv()

# ACP imports
from acp import run_agent
from acp.helpers import (
    update_agent_message_text,
    update_agent_thought_text,
)
from acp.interfaces import Agent
from acp.schema import (
    AgentCapabilities,
    InitializeResponse,
    McpCapabilities,
    ModelInfo,
    NewSessionResponse,
    PromptResponse,
    SessionInfo,
    StopReason,
)
from openhands.sdk import LLM, Conversation, Tool
from openhands.sdk import Agent as OpenHandsAgent
from openhands.sdk.llm.streaming import ModelResponseStream
from openhands.tools.file_editor import FileEditorTool
from openhands.tools.terminal import TerminalTool

# Streaming state for boundary detection
StreamingState = Literal["thinking", "content", "tool_name", "tool_args"]


class CrowAcpAgent(Agent):
    """
    Minimal ACP server that wraps OpenHands SDK.

    Supports:
    - session/new: Create new OpenHands conversation
    - session/prompt: Send prompt with streaming via session/update
    - session/cancel: Cancel ongoing generation
    """

    def __init__(self):
        self._conn: Any | None = None
        self._sessions: dict[str, dict[str, Any]] = {}
        self._llm = LLM(
            model=os.getenv("LLM_MODEL", "anthropic/glm-4.7"),
            api_key=os.getenv("ZAI_API_KEY"),
            base_url=os.getenv("ZAI_BASE_URL", None),
            stream=True,
        )

    def on_connect(self, conn: Any) -> None:
        """Called when client connects."""
        self._conn = conn

    async def initialize(
        self,
        protocol_version: int,
        client_capabilities: Any | None = None,
        client_info: Any | None = None,
        **kwargs: Any,
    ) -> InitializeResponse:
        """Initialize the ACP server."""
        return InitializeResponse(
            protocol_version=protocol_version,
            capabilities=AgentCapabilities(
                prompt_capabilities={"text": True},
                mcp_capabilities=McpCapabilities(http=False, sse=False),
                session_capabilities={},
            ),
            server_info={
                "name": "crow-acp-server",
                "version": "0.1.0",
                "title": "Crow ACP Server",
            },
            available_models=[
                ModelInfo(
                    model_id=os.getenv(
                        "LLM_MODEL", "anthropic/claude-sonnet-4-20250514"
                    ),
                    name=os.getenv("LLM_MODEL", "anthropic/claude-sonnet-4-20250514"),
                )
            ],
        )

    async def new_session(
        self,
        cwd: str,
        mcp_servers: list[Any] | None = None,
        **kwargs: Any,
    ) -> NewSessionResponse:
        """Create a new OpenHands conversation session."""
        from acp.schema import McpServerStdio

        # Build MCP config from ACP mcp_servers or use defaults
        mcp_config = {
            "mcpServers": {
                "fetch": {"command": "uvx", "args": ["mcp-server-fetch"]},
            }
        }
        if mcp_servers:
            for server in mcp_servers:
                if isinstance(server, McpServerStdio):
                    mcp_config["mcpServers"][server.name] = {
                        "command": server.command,
                        "args": server.args,
                    }

        # Create OpenHands agent
        tools = [Tool(name=TerminalTool.name), Tool(name=FileEditorTool.name)]
        oh_agent = OpenHandsAgent(
            llm=self._llm,
            tools=tools,
            mcp_config=mcp_config,
        )

        # Create conversation
        conversation = Conversation(
            agent=oh_agent,
            token_callbacks=[],
            workspace=cwd,
        )

        # Store session
        session_id = f"session_{len(self._sessions) + 1}"
        self._sessions[session_id] = {
            "conversation": conversation,
            "cwd": cwd,
            "current_state": None,
            "pending_tool_call": None,
        }

        return NewSessionResponse(
            session=SessionInfo(
                session_id=session_id,
                cwd=cwd,
            )
        )

    async def prompt(
        self,
        prompt: list[Any],
        session_id: str,
        **kwargs: Any,
    ) -> PromptResponse:
        """
        Send prompt to session with streaming.

        Streams tokens via session/update notifications using AgentMessageChunk
        for content and AgentThoughtChunk for reasoning.
        """
        if session_id not in self._sessions:
            raise ValueError(f"Unknown session: {session_id}")

        session = self._sessions[session_id]
        conversation = session["conversation"]

        # Extract text from prompt
        user_message = ""
        for block in prompt:
            if hasattr(block, "text"):
                user_message += block.text

        # Create streaming callback that sends ACP updates
        # Note: Can't use asyncio.create_task from sync callback
        # For now, collect tokens and send after run completes
        collected_content = []
        collected_thoughts = []

        def make_token_callback():
            def on_token(chunk: ModelResponseStream) -> None:
                """Stream tokens to ACP client."""
                for choice in chunk.choices:
                    delta = choice.delta
                    if delta is None:
                        continue

                    # Handle thinking/reasoning
                    reasoning = getattr(delta, "reasoning_content", None)
                    if isinstance(reasoning, str) and reasoning:
                        collected_thoughts.append(reasoning)

                    # Handle regular content
                    content = getattr(delta, "content", None)
                    if isinstance(content, str) and content:
                        collected_content.append(content)

            return on_token

        # Set token callback
        conversation.token_callbacks = [make_token_callback()]

        # Send message and run
        conversation.send_message(user_message)
        await asyncio.to_thread(conversation.run)

        # Send collected updates (not streaming yet, but functional)
        for thought in collected_thoughts:
            await self._conn.session_update(
                session_id=session_id,
                update=update_agent_thought_text(thought),
            )

        for content in collected_content:
            await self._conn.session_update(
                session_id=session_id,
                update=update_agent_message_text(content),
            )

        return PromptResponse(
            stop_reason=StopReason.end_turn,
        )

    async def cancel(self, session_id: str, **kwargs: Any) -> None:
        """Cancel ongoing generation."""
        if session_id in self._sessions:
            # TODO: Implement cancellation in OpenHands
            pass


async def main():
    """Entry point for the ACP server."""
    agent = CrowAcpAgent()
    await run_agent(agent)


if __name__ == "__main__":
    asyncio.run(main())
