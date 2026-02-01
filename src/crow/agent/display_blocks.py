"""
DisplayBlock system for rich ACP tool call content.

This module provides a type-safe, extensible system for creating rich
display content in ACP tool calls, inspired by kimi-cli's approach.

DisplayBlocks are converted to ACP content types (FileEditToolCallContent,
TerminalToolCallContent, etc.) when sending tool results to the client.
"""

from __future__ import annotations

from typing import Literal


class DisplayBlock:
    """Base class for all display blocks.

    DisplayBlocks represent rich content that should be displayed to the user
    when a tool completes. They are converted to ACP content types before
    being sent to the client.
    """

    type: str

    def to_acp_content(self):
        """Convert this display block to an ACP content type.

        This method should be overridden by subclasses to return the appropriate
        ACP content type (FileEditToolCallContent, TerminalToolCallContent, etc.)
        """
        raise NotImplementedError(
            f"{self.__class__.__name__} must implement to_acp_content()"
        )


class DiffDisplayBlock(DisplayBlock):
    """Display block describing a file diff.

    This block is used to show file changes in a structured diff format,
    which the ACP client can render with syntax highlighting and line numbers.

    Attributes:
        type: Always "diff" for this block type
        path: The file path being modified
        old_text: The original content (with context)
        new_text: The new content (with context)
    """

    type: Literal["diff"] = "diff"
    path: str
    old_text: str
    new_text: str

    def __init__(
        self,
        path: str,
        old_text: str,
        new_text: str,
    ):
        """Initialize a DiffDisplayBlock.

        Args:
            path: The file path being modified
            old_text: The original content (typically with context lines)
            new_text: The new content (typically with context lines)
        """
        self.path = path
        self.old_text = old_text
        self.new_text = new_text

    def to_acp_content(self):
        """Convert to FileEditToolCallContent for ACP."""
        from acp.schema import FileEditToolCallContent

        return FileEditToolCallContent(
            type="diff",
            path=self.path,
            old_text=self.old_text,
            new_text=self.new_text,
        )


class TerminalDisplayBlock(DisplayBlock):
    """Display block for terminal output.

    This block indicates that terminal output should be streamed directly
    to the ACP client rather than being included in the tool result text.

    Attributes:
        type: Always "terminal" for this block type
        terminal_id: The ID of the terminal session to stream from
    """

    type: Literal["terminal"] = "terminal"
    terminal_id: str

    def __init__(self, terminal_id: str):
        """Initialize a TerminalDisplayBlock.

        Args:
            terminal_id: The ID of the terminal session to stream from
        """
        self.terminal_id = terminal_id

    def to_acp_content(self):
        """Convert to TerminalToolCallContent for ACP."""
        from acp.schema import TerminalToolCallContent

        return TerminalToolCallContent(
            type="terminal",
            terminal_id=self.terminal_id,
        )


class ContentDisplayBlock(DisplayBlock):
    """Display block for generic text content.

    This block is used for plain text output that doesn't fit into
    specialized categories like diffs or terminal output.

    Attributes:
        type: Always "content" for this block type
        text: The text content to display
    """

    type: Literal["content"] = "content"
    text: str

    def __init__(self, text: str):
        """Initialize a ContentDisplayBlock.

        Args:
            text: The text content to display
        """
        self.text = text

    def to_acp_content(self):
        """Convert to ContentToolCallContent for ACP."""
        from acp import text_block, tool_content
        from acp.schema import ContentToolCallContent

        return ContentToolCallContent(
            type="content",
            content=tool_content(block=text_block(text=self.text)),
        )


class HideOutputDisplayBlock(DisplayBlock):
    """Display block that hides output from the tool result.

    This is used when the output is already being streamed via another
    mechanism (e.g., terminal streaming) and shouldn't be duplicated
    in the tool result text.

    Attributes:
        type: Always "hide_output" for this block type
    """

    type: Literal["hide_output"] = "hide_output"

    def __init__(self):
        """Initialize a HideOutputDisplayBlock."""

    def to_acp_content(self):
        """This block doesn't produce any ACP content.

        Returns:
            None to indicate no content should be added
        """
        return None


def display_blocks_to_acp_content(
    blocks: list[DisplayBlock],
) -> list:
    """Convert a list of DisplayBlocks to ACP content types.

    This function filters out None values (from HideOutputDisplayBlock)
    and converts each DisplayBlock to its corresponding ACP content type.

    Args:
        blocks: List of DisplayBlock objects

    Returns:
        List of ACP content objects (FileEditToolCallContent, etc.)
    """
    content = []
    for block in blocks:
        acp_content = block.to_acp_content()
        if acp_content is not None:
            content.append(acp_content)
    return content
