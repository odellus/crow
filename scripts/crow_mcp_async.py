"""
Async OpenHands SDK with MCP Integration and Streaming

This example demonstrates:
1. Running OpenHands Conversation in an async context (example 11)
2. Streaming tokens via token_callbacks (example 29)
3. MCP server integration
"""

import asyncio
import os
import sys
from typing import Literal

from dotenv import load_dotenv

load_dotenv()

from openhands.sdk import (
    LLM,
    Agent,
    Conversation,
    Event,
    LLMConvertibleEvent,
    Tool,
    get_logger,
)
from openhands.sdk.conversation.types import ConversationCallbackType
from openhands.sdk.llm.streaming import ModelResponseStream
from openhands.sdk.utils.async_utils import AsyncCallbackWrapper
from openhands.tools.file_editor import FileEditorTool
from openhands.tools.terminal import TerminalTool
from pydantic import SecretStr

logger = get_logger(__name__)

# Configure LLM
api_key = os.getenv("ZAI_API_KEY") or os.getenv("LLM_API_KEY")
if not api_key:
    raise RuntimeError("Set ZAI_API_KEY or LLM_API_KEY in your environment.")

model = os.getenv("LLM_MODEL", "anthropic/glm-4.7")
base_url = os.getenv("ZAI_BASE_URL") or os.getenv("LLM_BASE_URL")

llm = LLM(
    usage_id="crow-mcp-async",
    model=model,
    base_url=base_url,
    api_key=SecretStr(api_key),
    stream=True,  # Enable streaming
)

# Tools
cwd = os.getcwd()
tools = [
    Tool(name=TerminalTool.name),
    Tool(name=FileEditorTool.name),
]

# MCP Configuration
mcp_config = {
    "mcpServers": {
        "fetch": {"command": "uvx", "args": ["mcp-server-fetch"]},
        "web_search": {
            "command": "uv",
            "args": [
                "run",
                "--project",
                "/home/thomas/src/smolagents-example",
                "/home/thomas/src/smolagents-example/search.py",
            ],
        },
    }
}

# Agent
agent = Agent(
    llm=llm,
    tools=tools,
    mcp_config=mcp_config,
)

# Collect LLM messages for analysis
llm_messages = []


# ============================================================================
# STREAMING CALLBACK
# ============================================================================

StreamingState = Literal["thinking", "content", "tool_name", "tool_args"]
_current_state: StreamingState | None = None


def on_token(chunk: ModelResponseStream) -> None:
    """
    Handle streaming tokens with state tracking.

    This is called synchronously from the background thread during
    conversation.run(), so we write directly to stdout/stderr.
    """
    global _current_state

    choices = chunk.choices
    for choice in choices:
        delta = choice.delta
        if delta is None:
            continue

        # Handle thinking blocks (reasoning content)
        reasoning_content = getattr(delta, "reasoning_content", None)
        if isinstance(reasoning_content, str) and reasoning_content:
            if _current_state != "thinking":
                if _current_state is not None:
                    sys.stderr.write("\n")
                sys.stderr.write("🧠 THINKING: ")
                _current_state = "thinking"
            sys.stderr.write(reasoning_content)
            sys.stderr.flush()

        # Handle regular content
        content = getattr(delta, "content", None)
        if isinstance(content, str) and content:
            if _current_state != "content":
                if _current_state is not None:
                    sys.stderr.write("\n")
                sys.stderr.write("💬 CONTENT: ")
                _current_state = "content"
            sys.stderr.write(content)
            sys.stderr.flush()

        # Handle tool calls
        tool_calls = getattr(delta, "tool_calls", None)
        if tool_calls:
            for tool_call in tool_calls:
                tool_name = tool_call.function.name if tool_call.function.name else ""
                tool_args = (
                    tool_call.function.arguments if tool_call.function.arguments else ""
                )

                if tool_name:
                    if _current_state != "tool_name":
                        if _current_state is not None:
                            sys.stderr.write("\n")
                        sys.stderr.write(f"🔧 TOOL: {tool_name}")
                        _current_state = "tool_name"
                    sys.stderr.flush()

                if tool_args:
                    if _current_state != "tool_args":
                        if _current_state is not None:
                            sys.stderr.write("\n")
                        sys.stderr.write("   → ")
                        _current_state = "tool_args"
                    sys.stderr.write(tool_args)
                    sys.stderr.flush()


# ============================================================================
# ASYNC CALLBACK
# ============================================================================


async def callback_coro(event: Event):
    """
    Async callback that runs in the main event loop.

    This is called via AsyncCallbackWrapper from the background thread.
    """
    if isinstance(event, LLMConvertibleEvent):
        llm_messages.append(event.to_llm_message())


# ============================================================================
# SYNCHRONOUS RUNNER
# ============================================================================


def run_conversation(callback: ConversationCallbackType):
    """
    Synchronous function that runs the conversation.

    This runs in a background thread via loop.run_in_executor().
    """
    conversation = Conversation(
        agent=agent,
        callbacks=[callback],
        token_callbacks=[on_token],  # Streaming!
        workspace=cwd,
    )

    logger.info("Starting async conversation with MCP integration...")

    conversation.send_message(
        "Based on what you see in AGENTS.md please fetch three of the URLs listed "
        "in the llms.txt section and write 3 facts about agent client protocol "
        "and OpenHands into FACTS.txt."
    )
    conversation.run()

    conversation.send_message("Great! Now delete that file.")
    conversation.run()

    logger.info("Conversation finished.")


# ============================================================================
# ASYNC MAIN
# ============================================================================


async def main():
    """Main async entry point."""
    loop = asyncio.get_running_loop()

    # Wrap the async callback for use from sync context
    callback = AsyncCallbackWrapper(callback_coro, loop)

    print("=" * 100)
    print("Async OpenHands SDK with MCP + Streaming")
    print("=" * 100)
    print(f"\nModel: {model}")
    print(f"Base URL: {base_url}")
    print(f"Workspace: {cwd}")
    print(f"\nMCP Servers:")
    for name, config in mcp_config["mcpServers"].items():
        print(f"  - {name}: {config['command']} {' '.join(config['args'])}")
    print("\n" + "=" * 100 + "\n")

    # Run the conversation in a background thread
    # This blocks until conversation.run() completes
    await loop.run_in_executor(None, run_conversation, callback)

    print("\n" + "=" * 100)
    print("RESULTS")
    print("=" * 100)
    print(f"\nTotal LLM messages: {len(llm_messages)}")
    for i, message in enumerate(llm_messages):
        preview = str(message)[:200]
        print(f"  Message {i}: {preview}...")

    # Report cost
    cost = llm.metrics.accumulated_cost
    print(f"\n💰 Total Cost: {cost}")
    print("=" * 100)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n\n⚠️  Interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n\n❌ Error: {e}")
        import traceback

        traceback.print_exc()
        sys.exit(1)
