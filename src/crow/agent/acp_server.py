#!/usr/bin/env python3
"""
Minimal ACP Server wrapping OpenHands SDK with proper streaming.

This server extends the ACP Agent base class and uses OpenHands SDK
to run an AI agent with MCP tool support.
"""

import asyncio
import os
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
    StopReason,
)
from openhands.sdk import LLM, Conversation, Tool
from openhands.sdk import Agent as OpenHandsAgent
from openhands.sdk.llm.streaming import ModelResponseStream
from openhands.tools.file_editor import FileEditorTool
from openhands.tools.terminal import TerminalTool

from .config import LLMConfig, ServerConfig

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
        self._llm_config = LLMConfig.from_env()
        self._server_config = ServerConfig.from_env()

        self._llm = LLM(
            model=self._llm_config.model,
            api_key=self._llm_config.api_key,
            base_url=self._llm_config.base_url,
            stream=self._llm_config.stream,
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
                "name": self._server_config.name,
                "version": self._server_config.version,
                "title": self._server_config.title,
            },
            available_models=[
                ModelInfo(
                    model_id=self._llm_config.model,
                    name=self._llm_config.model,
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

        # Store session
        session_id = f"session_{len(self._sessions) + 1}"
        self._sessions[session_id] = {
            "agent": oh_agent,
            "cwd": cwd,
        }

        return NewSessionResponse(
            session_id=session_id,
        )

    async def prompt(
        self,
        prompt: list[Any],
        session_id: str,
        **kwargs: Any,
    ) -> PromptResponse:
        """Send prompt to session with streaming."""
        if session_id not in self._sessions:
            raise ValueError(f"Unknown session: {session_id}")

        session = self._sessions[session_id]

        # Extract text from prompt
        user_message = ""
        for block in prompt:
            if hasattr(block, "text"):
                user_message += block.text

        # Create queue for streaming updates from sync callback
        update_queue: asyncio.Queue = asyncio.Queue()

        # Streaming state for boundary detection (EXACT pattern from crow_mcp_integration.py)
        current_state = [None]

        def on_token(chunk: ModelResponseStream) -> None:
            """
            Handle all types of streaming tokens including content,
            tool calls, and thinking blocks with dynamic boundary detection.
            EXACT pattern from crow_mcp_integration.py but emitting to queue.
            """
            for choice in chunk.choices:
                delta = choice.delta
                if delta is None:
                    continue

                # Handle thinking blocks (reasoning content)
                reasoning_content = getattr(delta, "reasoning_content", None)
                if isinstance(reasoning_content, str) and reasoning_content:
                    if current_state[0] != "thinking":
                        current_state[0] = "thinking"
                    update_queue.put_nowait(("thought", reasoning_content))

                # Handle regular content
                content = getattr(delta, "content", None)
                if isinstance(content, str) and content:
                    if current_state[0] != "content":
                        current_state[0] = "content"
                    update_queue.put_nowait(("content", content))

                # Handle tool calls
                tool_calls = getattr(delta, "tool_calls", None)
                if tool_calls:
                    for tool_call in tool_calls:
                        tool_name = (
                            tool_call.function.name if tool_call.function.name else ""
                        )
                        tool_args = (
                            tool_call.function.arguments
                            if tool_call.function.arguments
                            else ""
                        )
                        if tool_name:
                            current_state[0] = "tool_name"
                            update_queue.put_nowait(("tool_name", tool_name))
                        if tool_args:
                            current_state[0] = "tool_args"
                            update_queue.put_nowait(("tool_args", tool_args))

        # Start background task to send updates from queue
        async def sender_task():
            while True:
                try:
                    update_type, data = await update_queue.get()
                    if update_type == "done":
                        break
                    elif update_type == "thought":
                        await self._conn.session_update(
                            session_id=session_id,
                            update=update_agent_thought_text(data),
                        )
                    elif update_type == "content":
                        await self._conn.session_update(
                            session_id=session_id,
                            update=update_agent_message_text(data),
                        )
                    elif update_type in ("tool_name", "tool_args"):
                        # TODO: Send tool call updates
                        pass
                except Exception as e:
                    print(f"Error sending update: {e}")

        sender = asyncio.create_task(sender_task())

        # Create conversation for this prompt
        conversation = Conversation(
            agent=session["agent"],
            token_callbacks=[on_token],
            workspace=session["cwd"],
        )

        # Run in thread pool to avoid blocking event loop
        def run_conversation():
            conversation.send_message(user_message)
            conversation.run()

        await asyncio.to_thread(run_conversation)

        # Signal done and wait for sender
        await update_queue.put(("done", None))
        await sender

        return PromptResponse(
            stop_reason="end_turn",
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
