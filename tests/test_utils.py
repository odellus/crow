"""Unit tests for Crow ACP server utilities."""

import os
import pytest
from crow.agent.acp_server import _map_tool_to_kind
from crow.agent.config import LLMConfig, AgentConfig, ServerConfig


class TestMapToolToKind:
    """Test the _map_tool_to_kind function."""

    def test_terminal_tools(self):
        """Test that terminal-related tools map to 'execute'."""
        assert _map_tool_to_kind("TerminalTool") == "execute"
        assert _map_tool_to_kind("run_command") == "execute"
        assert _map_tool_to_kind("execute_bash") == "execute"
        assert _map_tool_to_kind("ExecuteTool") == "execute"

    def test_file_edit_tools(self):
        """Test that file editing tools map to 'edit'."""
        assert _map_tool_to_kind("FileEditorTool") == "edit"
        assert _map_tool_to_kind("edit_file") == "edit"
        assert _map_tool_to_kind("FileEdit") == "edit"

    def test_read_tools(self):
        """Test that read tools map to 'read'."""
        assert _map_tool_to_kind("read_file") == "read"
        assert _map_tool_to_kind("ReadTool") == "read"
        assert _map_tool_to_kind("file_reader") == "read"

    def test_search_tools(self):
        """Test that search tools map to 'search'."""
        assert _map_tool_to_kind("search_files") == "search"
        assert _map_tool_to_kind("SearchTool") == "search"
        # grep is a search tool but doesn't have "search" in the name
        # This is expected - it would map to 'other' or we could add special handling

    def test_delete_tools(self):
        """Test that delete tools map to 'delete'."""
        assert _map_tool_to_kind("delete_file") == "delete"
        assert _map_tool_to_kind("DeleteTool") == "delete"
        assert _map_tool_to_kind("remove") == "delete"

    def test_move_tools(self):
        """Test that move/rename tools map to 'move'."""
        assert _map_tool_to_kind("move_file") == "move"
        assert _map_tool_to_kind("rename_file") == "move"
        assert _map_tool_to_kind("MoveTool") == "move"

    def test_other_tools(self):
        """Test that unrecognized tools map to 'other'."""
        assert _map_tool_to_kind("SomeRandomTool") == "other"
        assert _map_tool_to_kind("UnknownTool") == "other"
        assert _map_tool_to_kind("CustomAction") == "other"

    def test_case_insensitive(self):
        """Test that mapping is case-insensitive."""
        assert _map_tool_to_kind("TERMINALTOOL") == "execute"
        assert _map_tool_to_kind("FileEditorTool") == "edit"
        assert _map_tool_to_kind("ReadFile") == "read"

    def test_partial_matches(self):
        """Test that partial name matches work correctly."""
        # Should match 'terminal' in the name
        assert _map_tool_to_kind("my_terminal_tool") == "execute"
        # Should match 'file' in the name
        assert _map_tool_to_kind("my_file_tool") == "edit"


class TestLLMConfig:
    """Test LLMConfig loading."""

    def test_from_env_default_values(self):
        """Test loading config with default values."""
        # Clear environment variables
        for key in ["LLM_MODEL", "ZAI_API_KEY", "ZAI_BASE_URL", "LLM_TEMPERATURE", "LLM_MAX_TOKENS"]:
            os.environ.pop(key, None)
        
        config = LLMConfig.from_env()
        
        assert config.model == "anthropic/glm-4.7"
        assert config.api_key == ""
        assert config.base_url is None
        assert config.temperature == 0.0
        assert config.max_tokens == 4096
        assert config.stream is True

    def test_from_env_custom_values(self):
        """Test loading config with custom values."""
        os.environ["LLM_MODEL"] = "custom-model"
        os.environ["ZAI_API_KEY"] = "test-key"
        os.environ["ZAI_BASE_URL"] = "https://test.example.com"
        os.environ["LLM_TEMPERATURE"] = "0.5"
        os.environ["LLM_MAX_TOKENS"] = "2048"
        
        config = LLMConfig.from_env()
        
        assert config.model == "custom-model"
        assert config.api_key == "test-key"
        assert config.base_url == "https://test.example.com"
        assert config.temperature == 0.5
        assert config.max_tokens == 2048


class TestAgentConfig:
    """Test AgentConfig loading."""

    def test_from_env_default_values(self):
        """Test loading agent config with defaults."""
        os.environ.pop("MAX_ITERATIONS", None)
        os.environ.pop("AGENT_TIMEOUT", None)
        
        config = AgentConfig.from_env(cwd="/test/dir")
        
        assert config.cwd == "/test/dir"
        assert config.max_iterations == 500
        assert config.timeout == 300
        assert config.mcp_servers is None

    def test_from_env_custom_values(self):
        """Test loading agent config with custom values."""
        os.environ["MAX_ITERATIONS"] = "1000"
        os.environ["AGENT_TIMEOUT"] = "600"
        
        config = AgentConfig.from_env(cwd="/custom/dir")
        
        assert config.cwd == "/custom/dir"
        assert config.max_iterations == 1000
        assert config.timeout == 600


class TestServerConfig:
    """Test ServerConfig loading."""

    def test_from_env_default_values(self):
        """Test loading server config with defaults."""
        os.environ.pop("SERVER_NAME", None)
        os.environ.pop("SERVER_VERSION", None)
        os.environ.pop("SERVER_TITLE", None)
        
        config = ServerConfig.from_env()
        
        assert config.name == "crow-acp-server"
        assert config.version == "0.1.0"
        assert config.title == "Crow ACP Server"

    def test_from_env_custom_values(self):
        """Test loading server config with custom values."""
        os.environ["SERVER_NAME"] = "custom-server"
        os.environ["SERVER_VERSION"] = "2.0.0"
        os.environ["SERVER_TITLE"] = "Custom Server"
        
        config = ServerConfig.from_env()
        
        assert config.name == "custom-server"
        assert config.version == "2.0.0"
        assert config.title == "Custom Server"
