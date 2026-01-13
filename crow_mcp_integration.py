import os
import sys
from typing import Literal

from dotenv import load_dotenv

load_dotenv()
from openhands.sdk import LLM, Agent, Conversation, Tool
from openhands.tools.file_editor import FileEditorTool
from openhands.tools.task_tracker import TaskTrackerTool
from openhands.tools.terminal import TerminalTool

llm = LLM(
    model=os.getenv("LLM_MODEL", "anthropic/glm-4.7"),
    api_key=os.getenv("ZAI_API_KEY"),
    base_url=os.getenv("ZAI_BASE_URL", None),
    stream=True,
)


from openhands.sdk import (
    Event,
    LLMConvertibleEvent,
    get_logger,
)
from openhands.sdk.llm.streaming import ModelResponseStream
from openhands.sdk.security.llm_analyzer import LLMSecurityAnalyzer
from openhands.sdk.tool import Tool
from openhands.tools.file_editor import FileEditorTool
from openhands.tools.terminal import TerminalTool

logger = get_logger(__name__)


cwd = os.getcwd()
tools = [
    Tool(name=TerminalTool.name),
    Tool(name=FileEditorTool.name),
]

# Add MCP Tools
mcp_config = {
    "mcpServers": {
        "fetch": {"command": "uvx", "args": ["mcp-server-fetch"]},
        # "repomix": {"command": "npx", "args": ["-y", "repomix@1.4.2", "--mcp"]},
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


# Define streaming states
StreamingState = Literal["thinking", "content", "tool_name", "tool_args"]
# Track state across on_token calls for boundary detection
_current_state: StreamingState | None = None


def on_token(chunk: ModelResponseStream) -> None:
    """
    Handle all types of streaming tokens including content,
    tool calls, and thinking blocks with dynamic boundary detection.
    """
    global _current_state

    choices = chunk.choices
    for choice in choices:
        delta = choice.delta
        if delta is not None:
            # Handle thinking blocks (reasoning content)
            reasoning_content = getattr(delta, "reasoning_content", None)
            if isinstance(reasoning_content, str) and reasoning_content:
                if _current_state != "thinking":
                    if _current_state is not None:
                        sys.stdout.write("\n")
                    sys.stdout.write("THINKING: ")
                    _current_state = "thinking"
                sys.stdout.write(reasoning_content)
                sys.stdout.flush()

            # Handle regular content
            content = getattr(delta, "content", None)
            if isinstance(content, str) and content:
                if _current_state != "content":
                    if _current_state is not None:
                        sys.stdout.write("\n")
                    sys.stdout.write("CONTENT: ")
                    _current_state = "content"
                sys.stdout.write(content)
                sys.stdout.flush()

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
                        if _current_state != "tool_name":
                            if _current_state is not None:
                                sys.stdout.write("\n")
                            sys.stdout.write("TOOL NAME: ")
                            _current_state = "tool_name"
                        sys.stdout.write(tool_name)
                        sys.stdout.flush()
                    if tool_args:
                        if _current_state != "tool_args":
                            if _current_state is not None:
                                sys.stdout.write("\n")
                            sys.stdout.write("TOOL ARGS: ")
                            _current_state = "tool_args"
                        sys.stdout.write(tool_args)
                        sys.stdout.flush()


# Agent
agent = Agent(
    llm=llm,
    tools=tools,
    mcp_config=mcp_config,
    # This regex filters out all repomix tools except pack_codebase
    filter_tools_regex="^(?!repomix)(.*)|^repomix.*pack_codebase.*$",
)

llm_messages = []  # collect raw LLM messages


def conversation_callback(event: Event):
    if isinstance(event, LLMConvertibleEvent):
        llm_messages.append(event.to_llm_message())


# Conversation
conversation = Conversation(
    agent=agent,
    callbacks=[conversation_callback],
    token_callbacks=[on_token],
    workspace=cwd,
)
# conversation.set_security_analyzer(LLMSecurityAnalyzer())

logger.info("Starting conversation with MCP integration...")
conversation.send_message(
    "Based on what you see in AGENTS.md please fetch three of the URLs listed in the llms.txt section and write 3 facts "
    "about agent client protocol and OpenHands into FACTS.txt."
)
conversation.run()

conversation.send_message("Great! Now delete that file.")
conversation.run()

print("=" * 100)
print("Conversation finished. Got the following LLM messages:")
for i, message in enumerate(llm_messages):
    print(f"Message {i}: {str(message)[:200]}")

# Report cost
cost = llm.metrics.accumulated_cost
print(f"EXAMPLE_COST: {cost}")
