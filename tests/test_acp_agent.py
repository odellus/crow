#!/usr/bin/env python3
"""
Test script for Crow ACP Agent.

This script starts the Crow ACP server as a subprocess and sends it a prompt
to verify that the agent is working correctly.

Usage:
    uv run python test_acp_agent.py
"""

import asyncio
import asyncio.subprocess as aio_subprocess
import contextlib
import logging
import os
import sys
import time
from pathlib import Path
from typing import Any

# Add parent directory to path so we can import from acp-python-sdk
sys.path.insert(0, str(Path(__file__).parent.parent / "acp-python-sdk"))

from acp import (
    PROTOCOL_VERSION,
    Client,
    RequestError,
    connect_to_agent,
    text_block,
)
from acp.core import ClientSideConnection
from acp.schema import (
    AgentMessageChunk,
    AgentPlanUpdate,
    AgentThoughtChunk,
    AudioContentBlock,
    AvailableCommandsUpdate,
    ClientCapabilities,
    CreateTerminalResponse,
    CurrentModeUpdate,
    EmbeddedResourceContentBlock,
    EnvVariable,
    ImageContentBlock,
    Implementation,
    KillTerminalCommandResponse,
    PermissionOption,
    ReadTextFileResponse,
    ReleaseTerminalResponse,
    RequestPermissionResponse,
    ResourceContentBlock,
    TerminalOutputResponse,
    TextContentBlock,
    ToolCall,
    ToolCallProgress,
    ToolCallStart,
    UserMessageChunk,
    WaitForTerminalExitResponse,
    WriteTextFileResponse,
)


class TestClient(Client):
    """Test client that prints all agent responses."""

    def __init__(self):
        super().__init__()
        self.start_time = time.time()
        self.chunk_count = 0
        self.thought_count = 0
        self.tool_calls = []

    async def request_permission(
        self,
        options: list[PermissionOption],
        session_id: str,
        tool_call: ToolCall,
        **kwargs: Any,
    ) -> RequestPermissionResponse:
        """Auto-approve all tool calls for testing."""
        print(f"\n[PERMISSION] Tool call: {tool_call.title}")
        print(f"[PERMISSION] Auto-approving...")

        from acp.schema import PermissionOutcome

        return RequestPermissionResponse(
            outcome=PermissionOutcome(option_id="allow_once")
        )

    async def write_text_file(
        self, content: str, path: str, session_id: str, **kwargs: Any
    ) -> WriteTextFileResponse | None:
        raise RequestError.method_not_found("fs/write_text_file")

    async def read_text_file(
        self,
        path: str,
        session_id: str,
        limit: int | None = None,
        line: int | None = None,
        **kwargs: Any,
    ) -> ReadTextFileResponse:
        raise RequestError.method_not_found("fs/read_text_file")

    async def create_terminal(
        self,
        command: str,
        session_id: str,
        args: list[str] | None = None,
        cwd: str | None = None,
        env: list[EnvVariable] | None = None,
        output_byte_limit: int | None = None,
        **kwargs: Any,
    ) -> CreateTerminalResponse:
        raise RequestError.method_not_found("terminal/create")

    async def terminal_output(
        self, session_id: str, terminal_id: str, **kwargs: Any
    ) -> TerminalOutputResponse:
        raise RequestError.method_not_found("terminal/output")

    async def release_terminal(
        self, session_id: str, terminal_id: str, **kwargs: Any
    ) -> ReleaseTerminalResponse | None:
        raise RequestError.method_not_found("terminal/release")

    async def wait_for_terminal_exit(
        self, session_id: str, terminal_id: str, **kwargs: Any
    ) -> WaitForTerminalExitResponse:
        raise RequestError.method_not_found("terminal/wait_for_exit")

    async def kill_terminal(
        self, session_id: str, terminal_id: str, **kwargs: Any
    ) -> KillTerminalCommandResponse | None:
        raise RequestError.method_not_found("terminal/kill")

    async def session_update(
        self,
        session_id: str,
        update: UserMessageChunk
        | AgentMessageChunk
        | AgentThoughtChunk
        | ToolCallStart
        | ToolCallProgress
        | AgentPlanUpdate
        | AvailableCommandsUpdate
        | CurrentModeUpdate,
        **kwargs: Any,
    ) -> None:
        """Handle all session updates from the agent."""
        elapsed = time.time() - self.start_time

        # Handle agent message chunks (text response)
        if isinstance(update, AgentMessageChunk):
            self.chunk_count += 1
            content = update.content

            text: str
            if isinstance(content, TextContentBlock):
                text = content.text
            elif isinstance(content, ImageContentBlock):
                text = "<image>"
            elif isinstance(content, AudioContentBlock):
                text = "<audio>"
            elif isinstance(content, ResourceContentBlock):
                text = content.uri or "<resource>"
            elif isinstance(content, EmbeddedResourceContentBlock):
                text = "<resource>"
            else:
                text = "<content>"

            # Print text without newline to allow streaming
            print(text, end="", flush=True)

        # Handle thought chunks (reasoning)
        elif isinstance(update, AgentThoughtChunk):
            self.thought_count += 1
            if hasattr(update, "content") and update.content:
                print(f"\n[THOUGHT] {update.content}", flush=True)

        # Handle tool call starts
        elif isinstance(update, ToolCallStart):
            self.tool_calls.append(update.tool_call_id)
            print(f"\n[TOOL] {update.title or update.tool_call_id}", flush=True)
            if update.content:
                for block in update.content:
                    if hasattr(block, "text"):
                        print(f"  Args: {block.text[:100]}...", flush=True)

        # Handle tool call progress
        elif isinstance(update, ToolCallProgress):
            if update.content:
                for block in update.content:
                    if hasattr(block, "text"):
                        print(f"\n[TOOL RESULT]\n{block.text}", flush=True)

        # Handle plan updates
        elif isinstance(update, AgentPlanUpdate):
            print(f"\n[PLAN UPDATE]", flush=True)
            if hasattr(update, "plan_entries"):
                for entry in update.plan_entries:
                    status_icon = {
                        "pending": "⏳",
                        "in_progress": "🔄",
                        "completed": "✅",
                        "failed": "❌",
                    }.get(getattr(entry, "status", "pending"), "❓")
                    print(
                        f"  {status_icon} {getattr(entry, 'content', 'Unknown')}",
                        flush=True,
                    )

    async def ext_method(self, method: str, params: dict) -> dict:
        raise RequestError.method_not_found(method)

    async def ext_notification(self, method: str, params: dict) -> None:
        raise RequestError.method_not_found(method)


async def main() -> int:
    """Main test function."""
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    )

    print("=" * 80)
    print("Crow ACP Agent Test")
    print("=" * 80)

    # Start the Crow ACP server as a subprocess
    print("\n[1/4] Starting Crow ACP server...")

    # Find the project root (where src/ directory lives)
    # We're in tests/, so go up one level
    project_root = Path(__file__).parent.parent
    crow_dir = project_root / "src"

    proc = await asyncio.create_subprocess_exec(
        sys.executable,
        "-m",
        "crow.cli.main",
        "acp",
        stdin=aio_subprocess.PIPE,
        stdout=aio_subprocess.PIPE,
        stderr=None,  # Send stderr to console so we can see logs
        cwd=str(crow_dir),
    )

    if proc.stdin is None or proc.stdout is None:
        print("[ERROR] Agent process does not expose stdio pipes", file=sys.stderr)
        return 1

    print(f"[INFO] Server started with PID: {proc.pid}")

    # Create client and connect
    print("\n[2/4] Creating client and connecting...")

    client_impl = TestClient()
    conn = connect_to_agent(client_impl, proc.stdin, proc.stdout)

    # Initialize connection
    print("[INFO] Initializing connection...")
    await conn.initialize(
        protocol_version=PROTOCOL_VERSION,
        client_capabilities=ClientCapabilities(),
        client_info=Implementation(
            name="crow-test-client", title="Crow Test Client", version="0.1.0"
        ),
    )
    print("[INFO] Connection initialized successfully!")

    # Create new session
    print("\n[3/4] Creating new session...")
    session = await conn.new_session(mcp_servers=[], cwd=str(Path.cwd()))
    print(f"[INFO] Session created: {session.session_id}")

    # Send first test prompt
    print("\n[4/6] Sending first test prompt...")
    print("-" * 80)
    print("PROMPT 1: What is 2+2? Please respond with just the number.")
    print("-" * 80)
    print("\nRESPONSE:")

    try:
        await conn.prompt(
            session_id=session.session_id,
            prompt=[text_block("What is 2+2? Please respond with just the number.")],
        )
    except Exception as exc:
        logging.error("Prompt failed: %s", exc)
        return 1

    print("\n")

    # Send SECOND prompt in the SAME session to test persistence
    print("\n[5/6] Sending second test prompt (SAME SESSION)...")
    print("-" * 80)
    print("PROMPT 2: What was my previous question?")
    print("-" * 80)
    print("\nRESPONSE:")

    try:
        await conn.prompt(
            session_id=session.session_id,
            prompt=[text_block("What was my previous question?")],
        )
    except Exception as exc:
        logging.error("Second prompt failed: %s", exc)
        return 1

    print("\n")
    print("-" * 80)
    print("[SUMMARY]")
    print(f"  Message chunks: {client_impl.chunk_count}")
    print(f"  Thought chunks: {client_impl.thought_count}")
    print(f"  Tool calls: {len(client_impl.tool_calls)}")
    print(f"  Elapsed time: {time.time() - client_impl.start_time:.2f}s")
    print("-" * 80)

    # Cleanup
    print("\n[CLEANUP] Shutting down server...")
    if proc.returncode is None:
        proc.terminate()
        with contextlib.suppress(ProcessLookupError):
            await asyncio.wait_for(proc.wait(), timeout=5.0)

    print("\n[DONE] Test completed successfully!")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(asyncio.run(main()))
    except KeyboardInterrupt:
        print("\n[INTERRUPTED] Test cancelled by user")
        sys.exit(130)
