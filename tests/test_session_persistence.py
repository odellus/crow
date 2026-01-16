"""Test session persistence functionality."""

import json
import pytest
from pathlib import Path
from unittest.mock import AsyncMock, patch

from crow.agent.acp_server import CrowAcpAgent


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


@pytest.fixture
def temp_sessions_dir(tmp_path):
    """Create a temporary sessions directory."""
    sessions_dir = tmp_path / "sessions"
    sessions_dir.mkdir(parents=True, exist_ok=True)
    return sessions_dir


@pytest.mark.asyncio
async def test_save_session_creates_file(agent, temp_sessions_dir):
    """Test that saving a session creates a file."""
    # Patch the sessions directory
    with patch.object(agent, "_sessions_dir", temp_sessions_dir):
        # Create a session
        response = await agent.new_session(cwd="/tmp")
        session_id = response.session_id
        
        # Verify session file was created
        session_file = temp_sessions_dir / f"{session_id}.json"
        assert session_file.exists()
        
        # Verify file contains valid JSON
        with open(session_file) as f:
            session_data = json.load(f)
        
        assert session_data["session_id"] == session_id
        assert session_data["cwd"] == "/tmp"
        assert session_data["mode"] == "default"


@pytest.mark.asyncio
async def test_save_session_includes_conversation_history(agent, temp_sessions_dir, mock_connection):
    """Test that saving a session includes conversation history."""
    agent.on_connect(mock_connection)
    
    # Patch the sessions directory
    with patch.object(agent, "_sessions_dir", temp_sessions_dir):
        # Create a session
        response = await agent.new_session(cwd="/tmp")
        session_id = response.session_id
        
        # Add conversation history
        agent._sessions[session_id]["conversation_history"] = [
            {"role": "user", "content": "Hello"},
            {"role": "assistant", "parts": [{"type": "text", "text": "Hi there!"}]},
        ]
        
        # Save session
        agent._save_session(session_id)
        
        # Verify conversation history was saved
        session_file = temp_sessions_dir / f"{session_id}.json"
        with open(session_file) as f:
            session_data = json.load(f)
        
        assert "conversation_history" in session_data
        assert len(session_data["conversation_history"]) == 2
        assert session_data["conversation_history"][0]["role"] == "user"
        assert session_data["conversation_history"][0]["content"] == "Hello"


@pytest.mark.asyncio
async def test_load_session_restores_metadata(agent, temp_sessions_dir):
    """Test that loading a session restores metadata."""
    # Create a session file manually
    session_id = "test-session-123"
    session_file = temp_sessions_dir / f"{session_id}.json"
    
    session_data = {
        "session_id": session_id,
        "cwd": "/test/dir",
        "mode": "code",
        "conversation_history": [],
    }
    
    with open(session_file, "w") as f:
        json.dump(session_data, f)
    
    # Patch the sessions directory and load the session
    with patch.object(agent, "_sessions_dir", temp_sessions_dir):
        response = await agent.load_session(
            session_id=session_id,
            cwd="/tmp",  # This should be overridden by saved cwd
            mcp_servers=[],
        )
        
        # Verify session was restored
        assert session_id in agent._sessions
        assert agent._sessions[session_id]["cwd"] == "/test/dir"
        assert agent._sessions[session_id]["mode"] == "code"


@pytest.mark.asyncio
async def test_load_session_restores_conversation_history(agent, temp_sessions_dir, mock_connection):
    """Test that loading a session restores and replays conversation history."""
    agent.on_connect(mock_connection)
    
    # Create a session file with conversation history
    session_id = "test-session-456"
    session_file = temp_sessions_dir / f"{session_id}.json"
    
    session_data = {
        "session_id": session_id,
        "cwd": "/tmp",
        "mode": "default",
        "conversation_history": [
            {
                "role": "user",
                "content": "What is the capital of France?",
            },
            {
                "role": "assistant",
                "parts": [
                    {"type": "text", "text": "The capital of France is Paris."},
                ],
            },
        ],
    }
    
    with open(session_file, "w") as f:
        json.dump(session_data, f)
    
    # Patch the sessions directory and load the session
    with patch.object(agent, "_sessions_dir", temp_sessions_dir):
        response = await agent.load_session(
            session_id=session_id,
            cwd="/tmp",
            mcp_servers=[],
        )
        
        # Verify conversation history was restored
        assert session_id in agent._sessions
        assert "conversation_history" in agent._sessions[session_id]
        assert len(agent._sessions[session_id]["conversation_history"]) == 2
        
        # Verify session_update was called to replay history
        assert mock_connection.session_update.called
        # Should have been called multiple times (once per message part)
        assert mock_connection.session_update.call_count >= 2


@pytest.mark.asyncio
async def test_load_session_unknown_session(agent, temp_sessions_dir):
    """Test that loading an unknown session raises an error."""
    # Patch the sessions directory
    with patch.object(agent, "_sessions_dir", temp_sessions_dir):
        with pytest.raises(ValueError, match="Session not found"):
            await agent.load_session(
                session_id="unknown-session-id",
                cwd="/tmp",
                mcp_servers=[],
            )


@pytest.mark.asyncio
async def test_load_session_invalid_json(agent, temp_sessions_dir):
    """Test that loading a session with invalid JSON raises an error."""
    # Create a session file with invalid JSON
    session_id = "test-session-invalid"
    session_file = temp_sessions_dir / f"{session_id}.json"
    
    with open(session_file, "w") as f:
        f.write("{ invalid json }")
    
    # Patch the sessions directory and try to load
    with patch.object(agent, "_sessions_dir", temp_sessions_dir):
        with pytest.raises(ValueError, match="Failed to load session"):
            await agent.load_session(
                session_id=session_id,
                cwd="/tmp",
                mcp_servers=[],
            )


@pytest.mark.asyncio
async def test_load_session_id_mismatch(agent, temp_sessions_dir):
    """Test that loading a session with ID mismatch raises an error."""
    # Create a session file with different ID
    session_file = temp_sessions_dir / "session-abc.json"
    
    session_data = {
        "session_id": "session-xyz",  # Different from filename
        "cwd": "/tmp",
        "mode": "default",
        "conversation_history": [],
    }
    
    with open(session_file, "w") as f:
        json.dump(session_data, f)
    
    # Patch the sessions directory and try to load
    with patch.object(agent, "_sessions_dir", temp_sessions_dir):
        with pytest.raises(ValueError, match="Session ID mismatch"):
            await agent.load_session(
                session_id="session-abc",
                cwd="/tmp",
                mcp_servers=[],
            )


@pytest.mark.asyncio
async def test_save_session_handles_errors_gracefully(agent, temp_sessions_dir):
    """Test that save_session handles errors gracefully."""
    # Create a session
    response = await agent.new_session(cwd="/tmp")
    session_id = response.session_id
    
    # Patch the sessions directory to a read-only location (should cause error)
    readonly_dir = temp_sessions_dir / "readonly"
    readonly_dir.mkdir()
    
    # Make directory read-only
    import stat
    readonly_dir.chmod(stat.S_IRUSR | stat.S_IXUSR)
    
    with patch.object(agent, "_sessions_dir", readonly_dir):
        # Should not raise an exception, just print a warning
        agent._save_session(session_id)
    
    # Restore permissions for cleanup
    readonly_dir.chmod(stat.S_IRWXU | stat.S_IRWXG | stat.S_IRWXO)


@pytest.mark.asyncio
async def test_conversation_replay_includes_all_parts(agent, temp_sessions_dir, mock_connection):
    """Test that conversation replay includes all message parts (text, thoughts, tools)."""
    agent.on_connect(mock_connection)
    
    # Create a session file with complex conversation history
    session_id = "test-session-complex"
    session_file = temp_sessions_dir / f"{session_id}.json"
    
    session_data = {
        "session_id": session_id,
        "cwd": "/tmp",
        "mode": "default",
        "conversation_history": [
            {
                "role": "user",
                "content": "List files in /tmp",
            },
            {
                "role": "assistant",
                "parts": [
                    {"type": "thought", "text": "I need to list files"},
                    {"type": "tool_call", "id": "call_123", "name": "terminal", "arguments": 'ls /tmp'},
                    {"type": "text", "text": "I'll list the files for you."},
                ],
            },
        ],
    }
    
    with open(session_file, "w") as f:
        json.dump(session_data, f)
    
    # Patch the sessions directory and load the session
    with patch.object(agent, "_sessions_dir", temp_sessions_dir):
        response = await agent.load_session(
            session_id=session_id,
            cwd="/tmp",
            mcp_servers=[],
        )
        
        # Verify all parts were replayed
        # Should have calls for: user message, thought, tool_call (start + end), text
        assert mock_connection.session_update.call_count >= 5
