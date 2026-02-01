"""
Enhanced observations with display block support.

This module provides wrapper classes that add display block support to
OpenHands SDK observations, enabling rich ACP tool call content.
"""

from __future__ import annotations

from typing import Any

from crow.agent.display_blocks import DisplayBlock


class EnhancedObservation:
    """Wrapper that adds display blocks to OpenHands SDK observations.

    This wrapper allows us to attach rich display content (diffs, terminal
    output, etc.) to observations without modifying the OpenHands SDK itself.

    Attributes:
        observation: The original OpenHands SDK Observation object
        display: List of DisplayBlock objects for rich ACP content
        message: Optional human-readable summary message
    """

    def __init__(
        self,
        observation: Any,
        display: list[DisplayBlock] | None = None,
        message: str | None = None,
    ):
        """Initialize an EnhancedObservation.

        Args:
            observation: The original OpenHands SDK Observation object
            display: List of DisplayBlock objects for rich ACP content
            message: Optional human-readable summary message
        """
        self.observation = observation
        self.display = display or []
        self.message = message

    @property
    def is_error(self) -> bool:
        """Check if the underlying observation is an error."""
        return getattr(self.observation, "is_error", False)

    @property
    def content(self) -> list:
        """Get the content from the underlying observation."""
        return getattr(self.observation, "content", [])

    def to_llm_content(self) -> list:
        """Convert to LLM content format."""
        # Use the observation's to_llm_content if available
        if hasattr(self.observation, "to_llm_content"):
            return self.observation.to_llm_content()
        # Otherwise, return the content field
        return self.content

    def __getattr__(self, name: str) -> Any:
        """Proxy any other attributes to the underlying observation."""
        return getattr(self.observation, name)


def enhance_observation(
    observation: Any,
    display: list[DisplayBlock] | None = None,
    message: str | None = None,
) -> EnhancedObservation:
    """Enhance an observation with display blocks.

    This is a convenience function that creates an EnhancedObservation
    if display blocks or message are provided, otherwise returns the
    original observation unchanged.

    Args:
        observation: The original OpenHands SDK Observation object
        display: List of DisplayBlock objects for rich ACP content
        message: Optional human-readable summary message

    Returns:
        EnhancedObservation if display/message provided, else original observation
    """
    if display or message:
        return EnhancedObservation(
            observation=observation,
            display=display,
            message=message,
        )
    return observation
