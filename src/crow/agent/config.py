"""Configuration for Crow ACP server."""

import os
from pathlib import Path
from dataclasses import dataclass, field
from typing import Any

import yaml
from dotenv import load_dotenv


def get_crow_config_dir() -> Path:
    """Get the Crow configuration directory (~/.crow)."""
    return Path.home() / ".crow"


def load_crow_env() -> None:
    """Load environment variables from Crow config directory.
    
    Loads from ~/.crow/.env if it exists, then from local .env.
    """
    crow_env = get_crow_config_dir() / ".env"
    if crow_env.exists():
        load_dotenv(crow_env, override=True)
    # Also load local .env for development
    load_dotenv(".env", override=False)


def load_crow_config() -> dict[str, Any]:
    """Load configuration from ~/.crow/config.yaml."""
    config_file = get_crow_config_dir() / "config.yaml"
    if config_file.exists():
        with open(config_file) as f:
            return yaml.safe_load(f)
    return {}


# Load environment on import
load_crow_env()
# Load YAML config
_crow_config = load_crow_config()


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
        """Load LLM config from environment variables and config.yaml."""
        llm_config = _crow_config.get("llm", {})
        return cls(
            model=llm_config.get("model", os.getenv("LLM_MODEL", "anthropic/glm-4.7")),
            api_key=os.getenv("ZAI_API_KEY", ""),
            base_url=os.getenv("ZAI_BASE_URL"),
            temperature=float(llm_config.get("temperature", os.getenv("LLM_TEMPERATURE", "0.0"))),
            max_tokens=int(llm_config.get("max_tokens", os.getenv("LLM_MAX_TOKENS", "4096"))),
            stream=llm_config.get("stream", True),
        )


@dataclass
class AgentConfig:
    """Agent configuration."""

    cwd: str
    mcp_servers: list[Any] | None = None
    max_iterations: int = 500
    timeout: int = 300

    @classmethod
    def from_env(cls, cwd: str) -> "AgentConfig":
        """Load agent config from environment variables and config.yaml."""
        agent_config = _crow_config.get("agent", {})
        return cls(
            cwd=cwd,
            max_iterations=int(agent_config.get("max_iterations", os.getenv("MAX_ITERATIONS", "500"))),
            timeout=int(agent_config.get("timeout", os.getenv("AGENT_TIMEOUT", "300"))),
        )


@dataclass
class ServerConfig:
    """ACP server configuration."""

    name: str = "crow-acp-server"
    version: str = "0.1.0"
    title: str = "Crow ACP Server"

    @classmethod
    def from_env(cls) -> "ServerConfig":
        """Load server config from environment variables and config.yaml."""
        server_config = _crow_config.get("server", {})
        return cls(
            name=server_config.get("name", os.getenv("SERVER_NAME", "crow-acp-server")),
            version=server_config.get("version", os.getenv("SERVER_VERSION", "0.1.2")),
            title=server_config.get("title", os.getenv("SERVER_TITLE", "Crow ACP Server")),
        )
