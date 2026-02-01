"""
Minimal ACP Server wrapping OpenHands SDK with proper streaming.

This server extends the ACP Agent base class and uses OpenHands SDK
to run an AI agent with MCP tool support.
"""

import asyncio
import json
import logging
import os
import uuid
from pathlib import Path
from typing import Any, Literal

from dotenv import load_dotenv

load_dotenv()

# Configure logging to stderr to avoid interfering with ACP protocol
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    stream=os.sys.stderr,
)
logger = logging.getLogger(__name__)

# ACP imports
from acp import run_agent, stdio_streams, text_block, tool_content
from acp.core import AgentSideConnection
from acp.helpers import (
    plan_entry,
    start_tool_call,
    update_agent_message_text,
    update_agent_thought_text,
    update_plan,
    update_tool_call,
)
from acp.interfaces import Agent
from acp.schema import (
    AgentCapabilities,
    AgentPlanUpdate,
    AvailableCommand,
    AvailableCommandsUpdate,
    ContentToolCallContent,
    Diff,
    EmbeddedResource,
    FileEditToolCallContent,
    ImageContent,
    Implementation,
    InitializeResponse,
    LoadSessionResponse,
    McpCapabilities,
    McpServerStdio,
    ModelInfo,
    NewSessionResponse,
    PermissionOption,
    PlanEntry,
    PromptCapabilities,
    PromptResponse,
    SessionCapabilities,
    SessionMode,
    SetSessionModeResponse,
    StopReason,
    ToolCallStatus,
    ToolCallUpdate,
    ToolKind,
)

# OpenHands imports
from openhands.sdk import LLM, Conversation, Tool
from openhands.sdk import Agent as OpenHandsAgent
from openhands.sdk.conversation.base import BaseConversation
from openhands.sdk.event import ActionEvent, Event, ObservationEvent
from openhands.sdk.hooks import HookConfig
from openhands.sdk.llm.streaming import ModelResponseStream
from openhands.tools.file_editor import FileEditorObservation, FileEditorTool
from openhands.tools.task_tracker import TaskTrackerTool
from openhands.tools.terminal import TerminalTool

from crow.agent.config import LLMConfig, ServerConfig

# Streaming state for boundary detection
StreamingState = Literal["thinking", "content", "tool_name", "tool_args"]

# Diff context window
N_CONTEXT_LINES = 3


def _build_diff_blocks(
    path: str,
    old_text: str,
    new_text: str,
) -> list[FileEditToolCallContent]:
    """Build diff blocks grouped with small context windows.

    Uses SequenceMatcher to find actual changed regions and creates
    multiple focused blocks with surrounding context, similar to kimi-cli's approach.
    """
    from difflib import SequenceMatcher

    old_lines = old_text.splitlines(keepends=True)
    new_lines = new_text.splitlines(keepends=True)
    matcher = SequenceMatcher(None, old_lines, new_lines, autojunk=False)
    blocks: list[FileEditToolCallContent] = []

    for group in matcher.get_grouped_opcodes(n=N_CONTEXT_LINES):
        if not group:
            continue

        # Extract the changed region with context
        i1, i2 = group[0][1], group[-1][2]
        j1, j2 = group[0][3], group[-1][4]

        blocks.append(
            FileEditToolCallContent(
                type="diff",
                path=path,
                old_text="".join(old_lines[i1:i2]),
                new_text="".join(new_lines[j1:j2]),
            )
        )

    return blocks


async def _send_tool_result_to_acp(
    session_id: str,
    conn: Any,
    event: ObservationEvent,
    tool_call_id_map: dict[str, str],
) -> None:
    """Send tool result to ACP client.

    This function is called from the event callback when a tool completes.
    It sends the tool output to the ACP client via session_update.

    Args:
        session_id: The ACP session ID
        conn: The ACP connection
        event: The ObservationEvent from OpenHands SDK
        tool_call_id_map: Mapping from OpenHands SDK tool_call_id to ACP tool_call_id
    """
    try:
        obs = event.observation
        content_blocks = None

        # Translate OpenHands SDK tool_call_id to ACP tool_call_id
        acp_tool_call_id = tool_call_id_map.get(event.tool_call_id)
        logger.debug(
            f"Looking up tool call ID: {event.tool_call_id} -> {acp_tool_call_id}, "
            f"available IDs: {list(tool_call_id_map.keys())}"
        )
        if not acp_tool_call_id:
            logger.warning(
                f"No ACP tool call ID found for OpenHands SDK tool call ID: {event.tool_call_id}"
            )
            return

        # Check if this is a file_editor observation with diff information
        if isinstance(obs, FileEditorObservation):
            # Only use FileEditToolCallContent for actual edits (when we have content)
            if (
                hasattr(obs, "path")
                and hasattr(obs, "old_content")
                and hasattr(obs, "new_content")
                and (obs.old_content or obs.new_content)
            ):
                # Build intelligent diff blocks with context windows
                content_blocks = _build_diff_blocks(
                    path=obs.path,
                    old_text=obs.old_content or "",
                    new_text=obs.new_content or "",
                )

        # Fallback to plain text if no structured content
        if not content_blocks:
            output = str(event.visualize.plain)
            if output and output.strip():
                content_blocks = [tool_content(block=text_block(text=output))]

        # Send tool completion update with result
        await conn.session_update(
            session_id=session_id,
            update=update_tool_call(
                tool_call_id=acp_tool_call_id,  # Use the ACP tool call ID
                status="completed",
                content=content_blocks,
                raw_output={"output": str(event.visualize.plain)},
            ),
        )
    except Exception as e:
        logger.error(f"Error sending tool result to ACP: {e}", exc_info=True)


# TODO - REFACTOR this is pathetic
def _map_tool_to_kind(tool_name: str) -> str:
    """Map OpenHands tool name to ACP tool kind."""
    tool_name_lower = tool_name.lower()

    # Check in order of specificity - more specific patterns first
    if (
        "terminal" in tool_name_lower
        or "run" in tool_name_lower
        or "execute" in tool_name_lower
    ):
        return "execute"
    elif "read" in tool_name_lower:
        return "read"
    elif "search" in tool_name_lower:
        return "search"
    elif "delete" in tool_name_lower or "remove" in tool_name_lower:
        return "delete"
    elif "move" in tool_name_lower or "rename" in tool_name_lower:
        return "move"
    elif "file" in tool_name_lower or "edit" in tool_name_lower:
        return "edit"
    else:
        return "other"


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

        # Disable streaming for custom APIs that don't support it properly
        # ZAI API and other custom providers may not return CustomStreamWrapper
        stream = self._llm_config.stream
        if self._llm_config.base_url and "zai" in str(self._llm_config.base_url).lower():
            logger.warning("Detected ZAI API - disabling streaming due to compatibility issues")
            stream = False

        self._llm = LLM(
            model=self._llm_config.model,
            api_key=self._llm_config.api_key,
            base_url=self._llm_config.base_url,
            stream=stream,
        )

        # Define available session modes
        self._available_modes = [
            SessionMode(
                id="default",
                name="Default Mode",
                description="Standard agent behavior with full tool access",
            ),
            SessionMode(
                id="code",
                name="Code Mode",
                description="Focused on code generation and editing tasks",
            ),
            SessionMode(
                id="chat",
                name="Chat Mode",
                description="Conversational mode with minimal tool usage",
            ),
        ]

        # Define available slash commands
        self._available_commands = [
            AvailableCommand(
                name="/help",
                description="Show help information and available commands",
            ),
            AvailableCommand(
                name="/clear",
                description="Clear the current conversation context",
            ),
            AvailableCommand(
                name="/status",
                description="Show current session status and configuration",
            ),
        ]

        # Session persistence directory
        self._sessions_dir = Path.home() / ".crow" / "sessions"
        self._sessions_dir.mkdir(parents=True, exist_ok=True)

    def _enhance_tool_call_title(self, tool_name: str, tool_args: str) -> str:
        """Enhance tool call title with key argument for better UX.

        This extracts a meaningful argument from the tool args to create
        a more descriptive title, similar to kimi-cli's approach.

        Examples:
            - "terminal" with command "ls -la" -> "terminal: ls -la"
            - "file_editor" with path "test.py" -> "file_editor: test.py"
        """
        import json
        from typing import Any

        try:
            args: dict[str, Any] = json.loads(tool_args) if tool_args else {}
        except json.JSONDecodeError:
            # Args are incomplete or invalid, just return tool name
            return tool_name

        # Extract key argument based on tool name
        key_arg = None
        tool_name_lower = tool_name.lower()

        # Terminal/execute tools - show the command
        if (
            "terminal" in tool_name_lower
            or "run" in tool_name_lower
            or "execute" in tool_name_lower
        ):
            key_arg = args.get("command") or args.get("cmd")

        # File operations - show the path
        elif "file" in tool_name_lower or "edit" in tool_name_lower:
            key_arg = args.get("path")
            if key_arg:
                # Shorten path to just the filename if it's long
                if len(key_arg) > 50:
                    parts = key_arg.split("/")
                    if len(parts) > 1:
                        key_arg = f".../{parts[-1]}"

        # Search tools - show the query/pattern
        elif "search" in tool_name_lower or "grep" in tool_name_lower:
            key_arg = args.get("query") or args.get("pattern") or args.get("text")

        # Read tools - show the path
        elif "read" in tool_name_lower:
            key_arg = args.get("path")

        # Browser tools - show the URL or action
        elif "browser" in tool_name_lower or "browse" in tool_name_lower:
            key_arg = args.get("url") or args.get("action")

        # If we found a key argument, format it nicely
        if key_arg:
            key_str = str(key_arg)
            # Truncate if too long
            if len(key_str) > 40:
                key_str = key_str[:37] + "..."
            return f"{tool_name}: {key_str}"

        # No key argument found, return just the tool name
        return tool_name

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
            agent_capabilities=AgentCapabilities(
                prompt_capabilities={
                    "text": True,
                    "image": True,  # Support for image content
                    "resource": True,  # Support for embedded resources
                },
                mcp_capabilities=McpCapabilities(http=False, sse=False),
                load_session=True,  # Enable session persistence
            ),
            agent_info={
                "name": self._server_config.name,
                "version": self._server_config.version,
                "title": self._server_config.title,
            },
        )

    async def new_session(
        self,
        cwd: str,
        mcp_servers: list[Any] | None = None,
        **kwargs: Any,
    ) -> NewSessionResponse:
        """Create a new OpenHands conversation session."""

        try:
            # Validate cwd
            if not cwd or not isinstance(cwd, str):
                raise ValueError(f"Invalid working directory: {cwd}")

            # Build MCP config from ACP mcp_servers
            mcp_config = None
            if mcp_servers:
                mcp_config = {"mcpServers": {}}
                for server in mcp_servers:
                    if isinstance(server, McpServerStdio):
                        mcp_config["mcpServers"][server.name] = {
                            "command": server.command,
                            "args": server.args,
                        }

            # Create OpenHands agent with security disabled (auto-approve all)
            tools = [
                Tool(name=TerminalTool.name),
                Tool(name=FileEditorTool.name),
                Tool(name=TaskTrackerTool.name),
            ]

            # Only pass mcp_config if it's not None
            agent_kwargs = {
                "llm": self._llm,
                "tools": tools,
            }
            if mcp_config:
                agent_kwargs["mcp_config"] = mcp_config

            oh_agent = OpenHandsAgent(**agent_kwargs)

            # Store session
            session_id = str(uuid.uuid4())
            self._sessions[session_id] = {
                "agent": oh_agent,
                "cwd": cwd,
                "mode": "default",  # Default mode
                "conversation_history": [],  # Track history for ACP replay
            }

            # Save session to disk
            self._save_session(session_id)

            return NewSessionResponse(
                session_id=session_id,
            )
        except Exception as e:
            raise ValueError(f"Failed to create session: {e}")

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

        # Initialize conversation history if not present
        if "conversation_history" not in session:
            session["conversation_history"] = []

        # Extract content from prompt (support multiple content types)
        user_message = ""
        has_content = False

        try:
            for block in prompt:
                # Handle text content
                if hasattr(block, "text"):
                    user_message += block.text
                    has_content = True

                # Handle image content (for future vision support)
                elif hasattr(block, "data") and hasattr(block, "mime_type"):
                    # For now, we'll note that images are present but not process them
                    # This would require vision capabilities from the LLM
                    if block.mime_type.startswith("image/"):
                        user_message += f"[Image: {block.mime_type}]"
                        has_content = True

                # Handle embedded resources
                elif hasattr(block, "resource"):
                    # Extract text from text resources
                    resource = block.resource
                    if hasattr(resource, "text"):
                        user_message += resource.text
                        has_content = True
                    elif hasattr(resource, "blob"):
                        # For binary resources, note their presence
                        user_message += f"[Binary resource: {resource.mime_type}]"
                        has_content = True

        except Exception as e:
            raise ValueError(f"Invalid prompt format: {e}")

        if not has_content:
            raise ValueError("Prompt must contain at least one content block")

        # Handle slash commands
        if user_message.strip().startswith("/"):
            return await self._handle_slash_command(session_id, user_message.strip())

        # Record user message in conversation history
        session["conversation_history"].append(
            {
                "role": "user",
                "content": user_message,
            }
        )

        # Create initial plan based on user prompt
        # NOTE: This implementation creates plan entries for each step of execution
        # initial_plan_entries = [
        #     plan_entry(
        #         content=f"Process request: {user_message[:100]}{'...' if len(user_message) > 100 else ''}",
        #         priority="high",
        #         status="in_progress",
        #     ),
        # ]

        # # Track plan entries for this session
        # plan_entries = initial_plan_entries.copy()

        # # Send initial plan update
        # try:
        #     await self._conn.session_update(
        #         session_id=session_id,
        #         update=update_plan(initial_plan_entries),
        #     )
        # except Exception as e:
        #     # Plan updates are optional, don't fail if they don't work
        #     logger.warning(f"Failed to send plan update: {e}")

        # Create queue for streaming updates from sync callback
        update_queue: asyncio.Queue = asyncio.Queue()

        # Cancellation flag for this prompt
        cancelled_flag = {"cancelled": False}

        # Streaming state for boundary detection (EXACT pattern from crow_mcp_integration.py)
        current_state = [None]

        # Tool call tracking
        current_tool_call = {"id": None, "name": None, "args": ""}

        # Track tool call IDs: map OpenHands SDK tool_call_id -> ACP tool_call_id
        # This is needed because OpenHands SDK uses its own tool call IDs,
        # but we generate new UUIDs for ACP. We need to map them when sending results.
        # Store in session so event callback can access it
        tool_call_id_map: dict[str, str] = {}
        session["tool_call_id_map"] = tool_call_id_map

        # Track assistant response for conversation history
        assistant_parts = []

        def on_token(chunk: ModelResponseStream) -> None:
            """
            Handle all types of streaming tokens including content,
            tool calls, and thinking blocks with dynamic boundary detection.

            CRITICAL: Send updates directly via run_coroutine_threadsafe, NOT through queue!
            The queue adds latency and prevents true streaming.
            
            CRITICAL: Check cancelled_flag before sending any updates!
            """
            # Check for cancellation before processing any tokens
            if cancelled_flag["cancelled"]:
                logger.debug("Cancellation detected in on_token, ignoring further tokens")
                return
            
            for choice in chunk.choices:
                delta = choice.delta
                if delta is None:
                    continue

                # Handle thinking blocks (reasoning content)
                reasoning_content = getattr(delta, "reasoning_content", None)
                if isinstance(reasoning_content, str) and reasoning_content:
                    if current_state[0] != "thinking":
                        current_state[0] = "thinking"
                    # Send directly to ACP - NO QUEUE!
                    asyncio.run_coroutine_threadsafe(
                        self._conn.session_update(
                            session_id=session_id,
                            update=update_agent_thought_text(reasoning_content),
                        ),
                        loop,
                    )

                # Handle regular content
                content = getattr(delta, "content", None)
                if isinstance(content, str) and content:
                    if current_state[0] != "content":
                        current_state[0] = "content"
                    # Send directly to ACP - NO QUEUE!
                    asyncio.run_coroutine_threadsafe(
                        self._conn.session_update(
                            session_id=session_id,
                            update=update_agent_message_text(content),
                        ),
                        loop,
                    )

                # Handle tool calls
                tool_calls = getattr(delta, "tool_calls", None)
                if tool_calls:
                    for tool_call in tool_calls:
                        # Get the OpenHands SDK tool call ID (from LLM)
                        oh_tool_call_id = getattr(tool_call, "id", None)

                        tool_name = (
                            tool_call.function.name if tool_call.function.name else ""
                        )
                        tool_args = (
                            tool_call.function.arguments
                            if tool_call.function.arguments
                            else ""
                        )

                        logger.debug(
                            f"Tool call chunk: id={oh_tool_call_id}, name={tool_name}, "
                            f"args_len={len(tool_args)}, current_name={current_tool_call['name']}"
                        )

                        # Store tool call ID mapping as soon as we see it
                        # This must happen BEFORE tool execution starts to avoid race conditions
                        if oh_tool_call_id and oh_tool_call_id not in tool_call_id_map:
                            # Generate ACP tool call ID (new UUID for ACP)
                            acp_tool_call_id = str(uuid.uuid4())
                            tool_call_id_map[oh_tool_call_id] = acp_tool_call_id
                            logger.debug(
                                f"Stored tool call ID mapping: {oh_tool_call_id} -> {acp_tool_call_id}"
                            )

                        # New tool call starting (different name)
                        if tool_name and current_tool_call["name"] != tool_name:
                            # Finish previous tool call if exists
                            if current_tool_call["id"]:
                                update_queue.put_nowait(
                                    ("tool_end", current_tool_call.copy())
                                )

                            # Get the ACP tool call ID (already generated above)
                            acp_tool_call_id = tool_call_id_map.get(oh_tool_call_id) if oh_tool_call_id else str(uuid.uuid4())

                            current_tool_call["id"] = acp_tool_call_id
                            current_tool_call["name"] = tool_name
                            current_tool_call["args"] = tool_args
                            current_state[0] = "tool_name"
                            update_queue.put_nowait(
                                (
                                    "tool_start",
                                    (
                                        acp_tool_call_id,
                                        tool_name,
                                        tool_args,
                                        oh_tool_call_id,
                                    ),
                                )
                            )

                        # Accumulate args for current tool
                        elif tool_args and current_tool_call["name"]:
                            current_tool_call["args"] += tool_args
                            current_state[0] = "tool_args"
                            update_queue.put_nowait(
                                (
                                    "tool_args",
                                    (
                                        current_tool_call["id"],
                                        current_tool_call["args"],
                                    ),
                                )
                            )

        # Start background task to send updates from queue
        async def sender_task():
            import time

            start_time = time.time()
            try:
                while True:
                    try:
                        update_type, data = await update_queue.get()
                        elapsed = time.time() - start_time
                        logger.debug(f"[{elapsed:.3f}] Sender processing: {update_type}")
                    except Exception as e:
                        logger.error(f"Error getting update from queue: {e}")
                        break
                    
                    # Check for cancellation FIRST, before processing any updates
                    if cancelled_flag["cancelled"]:
                        logger.info(f"Cancellation detected, breaking event loop for session {session_id}")
                        # Drain any remaining events in the queue
                        while not update_queue.empty():
                            try:
                                update_queue.get_nowait()
                            except asyncio.QueueEmpty:
                                break
                        # Finish any pending tool call
                        if current_tool_call["id"]:
                            await self._conn.session_update(
                                session_id=session_id,
                                update=update_tool_call(
                                    tool_call_id=current_tool_call["id"],
                                    status="failed",
                                ),
                            )
                        break
                    
                    if update_type == "done":
                        # Finish any pending tool call
                        if current_tool_call["id"]:
                            await self._conn.session_update(
                                session_id=session_id,
                                update=update_tool_call(
                                    tool_call_id=current_tool_call["id"],
                                    status="completed",
                                ),
                            )
                        break
                    elif update_type == "thought":
                        assistant_parts.append({"type": "thought", "text": data})
                        # Fire-and-forget - don't await to avoid blocking the queue drain
                        asyncio.create_task(
                            self._conn.session_update(
                                session_id=session_id,
                                update=update_agent_thought_text(data),
                            )
                        )
                    elif update_type == "content":
                        assistant_parts.append({"type": "text", "text": data})
                        # Fire-and-forget - don't await to avoid blocking the queue drain
                        asyncio.create_task(
                            self._conn.session_update(
                                session_id=session_id,
                                update=update_agent_message_text(data),
                            )
                        )
                    elif update_type == "tool_start":
                        tool_call_id, tool_name, tool_args, oh_tool_call_id = data

                        # Track tool call for conversation history
                        assistant_parts.append(
                            {
                                "type": "tool_call",
                                "id": tool_call_id,
                                "name": tool_name,
                                "arguments": tool_args,
                            }
                        )

                        # Request permission before executing tool
                        permission_granted = await self._request_tool_permission(
                            session_id=session_id,
                            tool_call_id=tool_call_id,
                            tool_name=tool_name,
                            tool_args=tool_args,
                        )

                        # Add plan entry for this tool
                        # if permission_granted:
                        #     tool_plan_entry = plan_entry(
                        #         content=f"Execute tool: {tool_name}",
                        #         priority="medium",
                        #         status="in_progress",
                        #     )
                        #     plan_entries.append(tool_plan_entry)

                        #     # Send updated plan
                        #     try:
                        #         await self._conn.session_update(
                        #             session_id=session_id,
                        #             update=update_plan(plan_entries),
                        #         )
                        #     except Exception as e:
                        #         logger.warning(f"Failed to send plan update: {e}")

                        # Map tool name to ACP tool kind
                        tool_kind = _map_tool_to_kind(tool_name)

                        # Create enhanced title with key argument if available
                        title = self._enhance_tool_call_title(tool_name, tool_args)

                        # Send tool call update with appropriate status and content
                        # Include the arguments in content so users can see what's being executed
                        status = "in_progress" if permission_granted else "failed"
                        await self._conn.session_update(
                            session_id=session_id,
                            update=start_tool_call(
                                tool_call_id=tool_call_id,
                                title=title,
                                kind=tool_kind,
                                status=status,
                                content=[
                                    tool_content(
                                        block=text_block(
                                            text=tool_args if tool_args else "{}"
                                        )
                                    )
                                ],
                                raw_input=tool_args,
                            ),
                        )
                    elif update_type == "tool_args":
                        tool_call_id, tool_args = data
                        # Update tool call with progress and stream the accumulating arguments
                        # This shows users the arguments being built up in real-time
                        await self._conn.session_update(
                            session_id=session_id,
                            update=update_tool_call(
                                tool_call_id=tool_call_id,
                                status="in_progress",
                                content=[
                                    tool_content(
                                        block=text_block(
                                            text=tool_args if tool_args else "{}"
                                        )
                                    )
                                ],
                            ),
                        )
                    elif update_type == "tool_end":
                        tool_info = data

                        # Mark tool as completed
                        await self._conn.session_update(
                            session_id=session_id,
                            update=update_tool_call(
                                tool_call_id=tool_info["id"],
                                status="completed",
                            ),
                        )

                        # Update plan entry for this tool
                        # # Find the most recent tool plan entry and mark it completed
                        # for i in range(len(plan_entries) - 1, -1, -1):
                        #     entry = plan_entries[i]
                        #     if (
                        #         entry.status == "in_progress"
                        #         and "Execute tool:" in entry.content
                        #     ):
                        #         # Update this entry to completed
                        #         plan_entries[i] = plan_entry(
                        #             content=entry.content,
                        #             priority=entry.priority,
                        #             status="completed",
                        #         )
                        #         break

                        # # Send updated plan
                        # try:
                        #     await self._conn.session_update(
                        #         session_id=session_id,
                        #         update=update_plan(plan_entries),
                        #     )
                        # except Exception as e:
                        #     logger.warning(f"Failed to send plan update: {e}")

                        # Reset current tool
                        current_tool_call["id"] = None
                        current_tool_call["name"] = None
                        current_tool_call["args"] = ""
            finally:
                logger.info(f"Sender task finished for session {session_id}")

        sender = asyncio.create_task(sender_task())

        # Create or reuse conversation for this session
        # The Conversation object MUST be created once and reused for all prompts
        # to maintain conversation state across multiple messages
        if "conversation" not in session:
            # First prompt in this session - create the Conversation
            # Note: token_callbacks will be attached per-prompt below
            logger.info(f"Creating NEW Conversation for session {session_id}")

            # Create event callback to handle tool results
            # This callback runs synchronously from the SDK's worker thread
            # and schedules async ACP updates on the event loop
            loop = asyncio.get_running_loop()

            def event_callback(event: Event) -> None:
                """Handle OpenHands SDK events and send tool results to ACP client.

                This callback is invoked synchronously from the SDK's worker thread,
                so we schedule async ACP updates on the event loop.
                """
                if isinstance(event, ObservationEvent):
                    # Tool completed - send result to ACP client
                    # Get the tool_call_id_map from the session
                    id_map = session.get("tool_call_id_map", {})
                    asyncio.run_coroutine_threadsafe(
                        _send_tool_result_to_acp(
                            session_id=session_id,
                            conn=self._conn,
                            event=event,
                            tool_call_id_map=id_map,
                        ),
                        loop,
                    )

            conversation = Conversation(
                agent=session["agent"],
                workspace=session["cwd"],
                visualizer=None,  # Disable UI output to stdout
                callbacks=[event_callback],  # Add event callback for tool results
            )
            session["conversation"] = conversation
            logger.info(f"Conversation created: {id(conversation)}")
        else:
            # Reuse existing conversation
            conversation = session["conversation"]
            logger.info(
                f"Reusing existing Conversation: {id(conversation)} for session {session_id}"
            )

        # Get the conversation (created once per session)
        conversation = session["conversation"]

        # Attach token callbacks for THIS prompt only
        # The callbacks have access to the per-prompt update_queue
        # We need to replace the token_callbacks for each prompt
        conversation._on_token = BaseConversation.compose_callbacks([on_token])

        # Store conversation in session for cancellation
        session["cancelled_flag"] = cancelled_flag

        # Run conversation in background thread WITHOUT awaiting
        # This allows the event loop to process run_coroutine_threadsafe calls immediately
        import threading

        conversation_error = [None]  # Use list to allow assignment in nested function
        conversation_done = threading.Event()

        def run_conversation():
            try:
                conversation.send_message(user_message)
                conversation.run()
            except Exception as e:
                logger.error(f"Error running conversation: {e}", exc_info=True)
                conversation_error[0] = str(e)
                update_queue.put_nowait(("error", str(e)))
            finally:
                conversation_done.set()

        # Start conversation in background thread - DON'T AWAIT!
        thread = threading.Thread(target=run_conversation, daemon=True)
        thread.start()

        # Wait for conversation to finish while processing updates
        # The sender_task is already running and draining the queue
        # Use async wait to avoid blocking the event loop!
        loop = asyncio.get_running_loop()
        await loop.run_in_executor(None, conversation_done.wait)

        # Check if conversation failed
        if conversation_error:
            # Signal done and wait for sender
            await update_queue.put(("done", None))
            await sender

            # Return error response - use "end_turn" for errors
            return PromptResponse(
                stop_reason="end_turn",
            )

        # Clean up cancelled_flag only - keep the conversation for reuse!
        session.pop("cancelled_flag", None)

        # Signal done and wait for sender
        await update_queue.put(("done", None))
        await sender

        # Return appropriate stop reason
        stop_reason = "cancelled" if cancelled_flag["cancelled"] else "end_turn"
        logger.info(f"Conversation finished with stop_reason: {stop_reason}, cancelled_flag: {cancelled_flag['cancelled']}")

        # Save assistant response to conversation history
        if assistant_parts:
            session["conversation_history"].append(
                {
                    "role": "assistant",
                    "parts": assistant_parts,
                }
            )

            # Save session to disk with updated conversation history
            self._save_session(session_id)

        return PromptResponse(
            stop_reason=stop_reason,
        )

    async def cancel(self, session_id: str, **kwargs: Any) -> None:
        """Cancel ongoing generation."""
        try:
            if session_id in self._sessions:
                session = self._sessions[session_id]
                logger.info(f"Cancel requested for session {session_id}")

                # Set cancellation flag
                if "cancelled_flag" in session:
                    session["cancelled_flag"]["cancelled"] = True
                    logger.info(f"Set cancelled_flag to True for session {session_id}")
                else:
                    logger.warning(f"No cancelled_flag found in session {session_id}")

                # Pause the conversation if it exists
                if "conversation" in session:
                    conversation = session["conversation"]
                    # Note: pause() waits for the current LLM call to complete
                    # This is a temporary solution until hard cancellation is added
                    logger.info(f"Pausing conversation for session {session_id}")
                    conversation.pause()
                else:
                    logger.warning(f"No conversation found in session {session_id}")
        except Exception as e:
            logger.error(f"Error during cancellation: {e}", exc_info=True)

    async def set_session_mode(
        self,
        session_id: str,
        mode_id: str,
        **kwargs: Any,
    ) -> SetSessionModeResponse:
        """Set the mode for a session."""
        if session_id not in self._sessions:
            raise ValueError(f"Unknown session: {session_id}")

        # Validate mode_id
        valid_mode_ids = [mode.id for mode in self._available_modes]
        if mode_id not in valid_mode_ids:
            raise ValueError(
                f"Invalid mode_id: {mode_id}. Valid modes: {valid_mode_ids}"
            )

        # Update session mode
        self._sessions[session_id]["mode"] = mode_id

        # Save session to disk
        self._save_session(session_id)

        # Send mode update notification
        try:
            await self._conn.session_update(
                session_id=session_id,
                update={
                    "type": "current_mode_update",
                    "modeId": mode_id,
                },
            )
        except Exception as e:
            # Mode updates are optional, don't fail if they don't work
            logger.warning(f"Failed to send mode update: {e}")

        return SetSessionModeResponse()

    def _save_session(self, session_id: str) -> None:
        """Save session state to disk."""
        if session_id not in self._sessions:
            return

        session = self._sessions[session_id]
        session_file = self._sessions_dir / f"{session_id}.json"

        # Prepare session data for serialization
        session_data = {
            "session_id": session_id,
            "cwd": session.get("cwd", ""),
            "mode": session.get("mode", "default"),
            "conversation_history": session.get("conversation_history", []),
            # Note: We can't serialize the agent or conversation objects,
            # so we'll recreate them on load
        }

        try:
            with open(session_file, "w") as f:
                json.dump(session_data, f, indent=2)
        except Exception as e:
            logger.warning(f"Failed to save session {session_id}: {e}")

    async def load_session(
        self,
        session_id: str,
        cwd: str,
        mcp_servers: list[Any] | None = None,
        **kwargs: Any,
    ) -> LoadSessionResponse:
        """Load an existing session."""

        # Check if session file exists
        session_file = self._sessions_dir / f"{session_id}.json"
        if not session_file.exists():
            raise ValueError(f"Session not found: {session_id}")

        # Load session data
        try:
            with open(session_file, "r") as f:
                session_data = json.load(f)
        except Exception as e:
            raise ValueError(f"Failed to load session {session_id}: {e}")

        # Validate session_id matches
        if session_data.get("session_id") != session_id:
            raise ValueError(
                f"Session ID mismatch: expected {session_id}, got {session_data.get('session_id')}"
            )

        # Build MCP config from ACP mcp_servers
        mcp_config = None
        if mcp_servers:
            mcp_config = {"mcpServers": {}}
            for server in mcp_servers:
                if isinstance(server, McpServerStdio):
                    mcp_config["mcpServers"][server.name] = {
                        "command": server.command,
                        "args": server.args,
                    }

        # Create OpenHands agent with security disabled (auto-approve all)
        tools = [
            Tool(name=TerminalTool.name),
            Tool(name=FileEditorTool.name),
            Tool(name=TaskTrackerTool.name),
        ]

        # Only pass mcp_config if it's not None
        agent_kwargs = {
            "llm": self._llm,
            "tools": tools,
            # "security_policy_filename": None,  # Disable security checks
        }
        if mcp_config:
            agent_kwargs["mcp_config"] = mcp_config

        oh_agent = OpenHandsAgent(**agent_kwargs)

        # Restore session
        conversation_history = session_data.get("conversation_history", [])
        self._sessions[session_id] = {
            "agent": oh_agent,
            "cwd": session_data.get("cwd", cwd),
            "mode": session_data.get("mode", "default"),
            "conversation_history": conversation_history,
        }

        # Replay conversation history via session/update notifications
        # This is required by ACP spec: "The Agent MUST replay the entire conversation
        # to the Client in the form of `session/update` notifications"
        for msg in conversation_history:
            try:
                if msg["role"] == "user":
                    # Replay user message
                    await self._conn.session_update(
                        session_id=session_id,
                        update={
                            "type": "user_message",
                            "content": msg.get("content", ""),
                        },
                    )
                elif msg["role"] == "assistant":
                    # Replay assistant message (may have multiple parts)
                    for part in msg.get("parts", []):
                        if part["type"] == "text":
                            await self._conn.session_update(
                                session_id=session_id,
                                update=update_agent_message_text(
                                    text=part.get("text", ""),
                                ),
                            )
                        elif part["type"] == "thought":
                            await self._conn.session_update(
                                session_id=session_id,
                                update=update_agent_thought_text(
                                    text=part.get("text", ""),
                                ),
                            )
                        elif part["type"] == "tool_call":
                            await self._conn.session_update(
                                session_id=session_id,
                                update=start_tool_call(
                                    tool_call_id=part.get("id", ""),
                                    title=part.get("name", ""),
                                ),
                            )
                            # Mark tool as completed
                            await self._conn.session_update(
                                session_id=session_id,
                                update=update_tool_call(
                                    tool_call_id=part.get("id", ""),
                                    status="completed",
                                ),
                            )
            except Exception as e:
                # Don't fail the entire load if one message fails to replay
                logger.warning(f"Failed to replay message: {e}")

        return LoadSessionResponse()

    async def _handle_slash_command(
        self,
        session_id: str,
        command: str,
    ) -> PromptResponse:
        """Handle slash commands."""
        session = self._sessions[session_id]

        # Parse command and arguments
        parts = command.split()
        cmd = parts[0].lower()
        args = parts[1:] if len(parts) > 1 else []

        # Send command response as agent message
        response_text = ""

        if cmd == "/help":
            response_text = """Available Commands:
/help - Show this help message
/clear - Clear the current conversation context
/status - Show current session status and configuration

Available Modes:
- default: Standard agent behavior with full tool access
- code: Focused on code generation and editing tasks
- chat: Conversational mode with minimal tool usage

Use session/set_mode to change modes."""

        elif cmd == "/clear":
            # Clear conversation context if it exists
            if "conversation" in session:
                # Create a new conversation to clear context

                session["conversation"] = Conversation(
                    agent=session["agent"],
                    llm=self._llm,
                )
            # Also clear conversation history
            session["conversation_history"] = []
            response_text = "Conversation context cleared."

        elif cmd == "/status":
            mode = session.get("mode", "default")
            cwd = session.get("cwd", "unknown")
            response_text = f"""Session Status:
- Session ID: {session_id}
- Mode: {mode}
- Working Directory: {cwd}
- Model: {self._llm_config.model}
- Server: {self._server_config.name} v{self._server_config.version}"""

        else:
            response_text = (
                f"Unknown command: {cmd}. Type /help for available commands."
            )

        # Send response as agent message
        try:
            await self._conn.session_update(
                session_id=session_id,
                update=update_agent_message_text(response_text),
            )
        except Exception as e:
            logger.warning(f"Failed to send command response: {e}")

        return PromptResponse(
            stop_reason="end_turn",
        )

    async def _request_tool_permission(
        self,
        session_id: str,
        tool_call_id: str,
        tool_name: str,
        tool_args: str,
    ) -> bool:
        """
        Request permission from the user before executing a tool.

        Returns True if permission was granted, False otherwise.

        Note: This is a basic implementation that sends permission requests
        but always allows execution. Full permission enforcement would require
        deeper integration with OpenHands SDK's security policy system.
        """
        try:
            # Create permission options
            options = [
                PermissionOption(
                    option_id="allow_once",
                    name="Allow Once",
                    kind="allow_once",
                ),
                PermissionOption(
                    option_id="allow_always",
                    name="Allow Always",
                    kind="allow_always",
                ),
                PermissionOption(
                    option_id="reject_once",
                    name="Reject Once",
                    kind="reject_once",
                ),
                PermissionOption(
                    option_id="reject_always",
                    name="Reject Always",
                    kind="reject_always",
                ),
            ]

            # Create tool call update for permission request
            tool_call_update = ToolCallUpdate(
                tool_call_id=tool_call_id,
                title=tool_name,
                kind=_map_tool_to_kind(tool_name),
                status="pending",
                raw_input=tool_args,
            )

            # Request permission from client
            response = await self._conn.request_permission(
                options=options,
                session_id=session_id,
                tool_call=tool_call_update,
            )

            # Check the outcome
            if response.outcome and hasattr(response.outcome, "option_id"):
                option_id = response.outcome.option_id
                logger.info(f"Permission response: {option_id}")

                # Allow if user selected an allow option
                if option_id in ("allow_once", "allow_always"):
                    return True
                else:
                    logger.info(f"Tool execution rejected by user: {option_id}")
                    return False
            else:
                # No outcome or cancelled - default to allow for now
                # (full enforcement would require OpenHands SDK integration)
                logger.info("No permission outcome, allowing execution")
                return True

        except Exception as e:
            # If permission request fails, log and allow execution
            # (fail-open for better UX, could be configurable)
            logger.warning(f"Permission request failed: {e}, allowing execution")
            return True


async def main():
    """Entry point for the ACP server."""

    # Get proper stdin/stdout streams for ACP communication
    reader, writer = await stdio_streams()

    # Create agent instance
    agent = CrowAcpAgent()

    # Create ACP connection
    conn = AgentSideConnection(agent, writer, reader, use_unstable_protocol=True)

    # Keep the server running
    await asyncio.Event().wait()


def sync_main():
    """Synchronous entry point for CLI."""
    asyncio.run(main())


if __name__ == "__main__":
    asyncio.run(main())
