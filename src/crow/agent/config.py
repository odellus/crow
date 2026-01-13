"""Configuration for Crow ACP server."""

import os
from dataclasses import dataclass
from typing import Any

from dotenv import load_dotenv

load_dotenv()


@dataclass
class LLMConfig:
    """LLM configuration."""

    model: str
    api_key: str
    base_url: str | None = None
    temperature: float = 0.0
    max_tokens: int = 4096
    stream: bool = True

    @classmethod
    def from_env(cls) -> "LLMConfig":
        """Load LLM config from environment variables."""
        return cls(
            model=os.getenv("LLM_MODEL", "anthropic/glm-4.7"),
            api_key=os.getenv("ZAI_API_KEY", ""),
            base_url=os.getenv("ZAI_BASE_URL"),
            temperature=float(os.getenv("LLM_TEMPERATURE", "0.0")),
            max_tokens=int(os.getenv("LLM_MAX_TOKENS", "4096")),
            stream=True,
        )
