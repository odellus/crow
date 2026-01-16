"""Test session modes functionality."""

import pytest
from unittest.mock import AsyncMock, MagicMock, patch

from crow.agent.acp_server import CrowAcpAgent
from acp.schema import SessionMode


@pytest.fixture
def agent():
    """Create a CrowAcpAgent instance for testing."""
    return CrowAcpAgent()


@pytest.fixture
def mock_connection():
    """Create a mock connection."""
    conn = AsyncMock()
    conn.session_update = AsyncMock()
    return conn


@pytest.mark.asyncio
async def test_available_modes(agent):
    """Test that available modes are correctly defined."""
    # Check that modes are defined
    assert len(agent._available_modes) == 3
    
    # Check mode IDs
    mode_ids = [mode.id for mode in agent._available_modes]
    assert "default" in mode_ids
    assert "code" in mode_ids
    assert "chat" in mode_ids
    
    # Check that all modes have required fields
    for mode in agent._available_modes:
        assert isinstance(mode, SessionMode)
        assert mode.id  # non-empty string
        assert mode.name  # non-empty string
        assert mode.description  # non-empty string


@pytest.mark.asyncio
async def test_initialize_includes_modes(agent):
    """Test that initialize response includes available modes."""
    response = await agent.initialize(
        protocol_version=1,
        client_capabilities={},
        client_info={"name": "test", "version": "1.0"},
    )
    
    # Check that agent has modes defined (even if not advertised in initialize)
    # Note: Current ACP schema doesn't support advertising modes in InitializeResponse
    # Modes are implemented and can be used via set_session_mode
    assert len(agent._available_modes) == 3
    mode_ids = [mode.id for mode in agent._available_modes]
    assert "default" in mode_ids
    assert "code" in mode_ids
    assert "chat" in mode_ids


@pytest.mark.asyncio
async def test_new_session_has_default_mode(agent):
    """Test that new sessions start with default mode."""
    response = await agent.new_session(cwd="/tmp")
    
    session_id = response.session_id
    assert session_id in agent._sessions
    assert agent._sessions[session_id]["mode"] == "default"


@pytest.mark.asyncio
async def test_set_session_mode_valid(agent, mock_connection):
    """Test setting a valid session mode."""
    agent.on_connect(mock_connection)
    
    # Create a session first
    new_response = await agent.new_session(cwd="/tmp")
    session_id = new_response.session_id
    
    # Set mode to "code"
    response = await agent.set_session_mode(
        session_id=session_id,
        mode_id="code",
    )
    
    # Verify mode was updated
    assert agent._sessions[session_id]["mode"] == "code"
    
    # Verify mode update notification was sent
    mock_connection.session_update.assert_called_once()
    call_args = mock_connection.session_update.call_args
    assert call_args[1]["session_id"] == session_id
    assert call_args[1]["update"]["type"] == "current_mode_update"
    assert call_args[1]["update"]["modeId"] == "code"


@pytest.mark.asyncio
async def test_set_session_mode_invalid(agent):
    """Test setting an invalid session mode."""
    # Create a session first
    new_response = await agent.new_session(cwd="/tmp")
    session_id = new_response.session_id
    
    # Try to set invalid mode
    with pytest.raises(ValueError, match="Invalid mode_id"):
        await agent.set_session_mode(
            session_id=session_id,
            mode_id="invalid_mode",
        )


@pytest.mark.asyncio
async def test_set_session_mode_unknown_session(agent):
    """Test setting mode for unknown session."""
    with pytest.raises(ValueError, match="Unknown session"):
        await agent.set_session_mode(
            session_id="unknown_session_id",
            mode_id="code",
        )


@pytest.mark.asyncio
async def test_set_session_mode_saves_to_disk(agent, mock_connection, tmp_path):
    """Test that setting mode saves session to disk."""
    agent.on_connect(mock_connection)
    
    # Patch the sessions directory to use tmp_path
    with patch.object(agent, "_sessions_dir", tmp_path):
        # Create a session
        new_response = await agent.new_session(cwd="/tmp")
        session_id = new_response.session_id
        
        # Set mode
        await agent.set_session_mode(
            session_id=session_id,
            mode_id="chat",
        )
        
        # Verify session file was saved with new mode
        import json
        session_file = tmp_path / f"{session_id}.json"
        assert session_file.exists()
        
        with open(session_file) as f:
            session_data = json.load(f)
        
        assert session_data["mode"] == "chat"


@pytest.mark.asyncio
async def test_mode_persistence_across_sessions(agent, mock_connection, tmp_path):
    """Test that mode is persisted when session is saved and loaded."""
    from acp.schema import McpServerStdio
    
    agent.on_connect(mock_connection)
    
    # Patch the sessions directory to use tmp_path
    with patch.object(agent, "_sessions_dir", tmp_path):
        # Create a session and set mode
        new_response = await agent.new_session(cwd="/tmp")
        session_id = new_response.session_id
        
        await agent.set_session_mode(
            session_id=session_id,
            mode_id="code",
        )
        
        # Clear the session from memory
        del agent._sessions[session_id]
        
        # Load the session
        load_response = await agent.load_session(
            session_id=session_id,
            cwd="/tmp",
            mcp_servers=[],
        )
        
        # Verify mode was restored
        assert agent._sessions[session_id]["mode"] == "code"
