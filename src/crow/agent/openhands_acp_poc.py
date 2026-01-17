"""
Minimal POC: OpenHands SDK → ACP Streaming Adapter

This demonstrates the core concept of wrapping OpenHands SDK's synchronous
Conversation.run() in an async ACP handler with streaming support.
"""

import asyncio
import os
import sys
from pathlib import Path
from typing import Any, Literal

from pydantic import SecretStr

# OpenHands SDK imports
from openhands.sdk import Agent, Conversation, LLM, Tool
from openhands.sdk.llm.streaming import ModelResponseStream
from openhands.tools.file_editor import FileEditorTool
from openhands.tools.terminal import TerminalTool

# ACP imports (simplified for POC)
# In real implementation, these would be from acp package


# ============================================================================
# STREAMING STATE TRACKING
# ============================================================================

StreamingState = Literal["thinking", "content", "tool_name", "tool_args"]


class StreamingTracker:
    """Track streaming state for boundary detection."""
    
    def __init__(self):
        self.current_state: StreamingState | None = None
        self.current_tool_call_id: str | None = None
        self.current_args: str = ""
    
    def transition_to(self, new_state: StreamingState) -> bool:
        """Transition to new state, return True if state changed."""
        if self.current_state != new_state:
            print(f"\n[STATE: {self.current_state} → {new_state}]", end="", file=sys.stderr)
            self.current_state = new_state
            return True
        return False


# ============================================================================
# ACP ADAPTER
# ============================================================================

class OpenHandsACPAdapter:
    """
    Adapter that wraps OpenHands SDK for ACP streaming.
    
    Key responsibilities:
    1. Run synchronous Conversation.run() in async context
    2. Stream tokens via token_callback
    3. Map OpenHands events to ACP updates
    """
    
    def __init__(self, workspace: Path):
        self.workspace = workspace
        self.tracker = StreamingTracker()
        
        # Initialize OpenHands components
        self.llm = LLM(
            model=os.getenv("LLM_MODEL", "anthropic/claude-sonnet-4-5-20250929"),
            api_key=SecretStr(os.getenv("LLM_API_KEY", "")),
            base_url=os.getenv("LLM_BASE_URL"),
            stream=True,  # Enable streaming
        )
        
        self.agent = Agent(
            llm=self.llm,
            tools=[
                Tool(name=TerminalTool.name),
                Tool(name=FileEditorTool.name),
            ],
        )
        
        self.conversation = Conversation(
            agent=self.agent,
            workspace=str(self.workspace),
        )
    
    def _token_callback(self, chunk: ModelResponseStream) -> None:
        """
        Handle streaming tokens from OpenHands SDK.
        
        This is where we map OpenHands events to ACP updates.
        In real implementation, this would call conn.session_update().
        """
        choices = chunk.choices
        for choice in choices:
            delta = choice.delta
            if delta is None:
                continue
            
            # Handle thinking blocks (reasoning)
            reasoning_content = getattr(delta, "reasoning_content", None)
            if isinstance(reasoning_content, str) and reasoning_content:
                if self.tracker.transition_to("thinking"):
                    print("\n🧠 THINKING:", end="", file=sys.stderr)
                print(reasoning_content, end="", file=sys.stderr)
                sys.stderr.flush()
            
            # Handle regular content
            content = getattr(delta, "content", None)
            if isinstance(content, str) and content:
                if self.tracker.transition_to("content"):
                    print("\n💬 CONTENT:", end="", file=sys.stderr)
                print(content, end="", file=sys.stderr)
                sys.stderr.flush()
            
            # Handle tool calls
            tool_calls = getattr(delta, "tool_calls", None)
            if tool_calls:
                for tool_call in tool_calls:
                    # Tool name
                    tool_name = tool_call.function.name if tool_call.function.name else ""
                    if tool_name:
                        if self.tracker.transition_to("tool_name"):
                            print(f"\n🔧 TOOL: {tool_name}", end="", file=sys.stderr)
                            self.tracker.current_tool_call_id = tool_call.id
                    
                    # Tool arguments (streaming)
                    tool_args = tool_call.function.arguments if tool_call.function.arguments else ""
                    if tool_args:
                        if self.tracker.transition_to("tool_args"):
                            print(" → ", end="", file=sys.stderr)
                        print(tool_args, end="", file=sys.stderr)
                        sys.stderr.flush()
    
    async def prompt(self, user_message: str) -> dict[str, Any]:
        """
        Send prompt to OpenHands with streaming.
        
        In real ACP implementation, this would:
        1. Send user message via conversation.send_message()
        2. Run conversation.run() with token_callback
        3. Stream updates via conn.session_update()
        4. Return PromptResponse with stop_reason
        """
        print(f"\n{'='*80}", file=sys.stderr)
        print(f"PROMPT: {user_message}", file=sys.stderr)
        print(f"{'='*80}\n", file=sys.stderr)
        
        # Send message to conversation
        self.conversation.send_message(user_message)
        
        # Run conversation in thread pool (blocking → async)
        # In real implementation, use asyncio.to_thread()
        loop = asyncio.get_event_loop()
        await loop.run_in_executor(
            None,
            lambda: self.conversation.run()
        )
        
        # Return response (simplified for POC)
        return {
            "stop_reason": "end_turn",
            "metrics": {
                "total_cost": str(self.llm.metrics.accumulated_cost),
            }
        }


# ============================================================================
# POC MAIN
# ============================================================================

async def main():
    """Run the POC."""
    print("\n" + "="*80, file=sys.stderr)
    print("OpenHands SDK → ACP Streaming Adapter POC", file=sys.stderr)
    print("="*80 + "\n", file=sys.stderr)
    
    # Check environment
    if not os.getenv("LLM_API_KEY"):
        print("❌ Error: LLM_API_KEY not set", file=sys.stderr)
        print("   Set it with: export LLM_API_KEY=sk-ant-xxx", file=sys.stderr)
        return
    
    # Create adapter
    workspace = Path.cwd()
    adapter = OpenHandsACPAdapter(workspace)
    
    # Test prompts
    prompts = [
        "Write a haiku about AI agents.",
        "List 3 files in the current directory.",
    ]
    
    for prompt in prompts:
        print(f"\n{'─'*80}\n", file=sys.stderr)
        result = await adapter.prompt(prompt)
        print(f"\n✓ Complete: {result}", file=sys.stderr)
    
    print(f"\n{'='*80}\n", file=sys.stderr)


if __name__ == "__main__":
    asyncio.run(main())
