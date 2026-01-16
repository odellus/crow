"""Test content type handling functionality."""

import pytest
from unittest.mock import AsyncMock, MagicMock, patch

from crow.agent.acp_server import CrowAcpAgent
from acp.schema import TextContent, ImageContent, EmbeddedResource


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
async def test_prompt_with_text_content(agent, mock_connection):
    """Test handling text content in prompts."""
    agent.on_connect(mock_connection)
    
    # Create a session
    new_response = await agent.new_session(cwd="/tmp")
    session_id = new_response.session_id
    
    # Mock the conversation run to avoid actual LLM call
    with patch("crow.agent.acp_server.Conversation") as mock_conv_class:
        mock_conv = MagicMock()
        mock_conv_class.return_value = mock_conv
        
        # Send text prompt
        text_content = TextContent(text="Hello, agent!")
        response = await agent.prompt(
            prompt=[text_content],
            session_id=session_id,
        )
        
        # Verify the text was extracted
        # (We can't easily verify the full flow without mocking more, but we can check no errors)


@pytest.mark.asyncio
async def test_prompt_with_multiple_text_blocks(agent, mock_connection):
    """Test handling multiple text content blocks."""
    agent.on_connect(mock_connection)
    
    # Create a session
    new_response = await agent.new_session(cwd="/tmp")
    session_id = new_response.session_id
    
    # Initialize conversation history
    agent._sessions[session_id]["conversation_history"] = []
    
    # Send multiple text blocks
    text_content1 = TextContent(text="First part. ")
    text_content2 = TextContent(text="Second part.")
    
    # This should concatenate the text
    with patch("crow.agent.acp_server.Conversation") as mock_conv_class:
        mock_conv = MagicMock()
        mock_conv_class.return_value = mock_conv
        
        response = await agent.prompt(
            prompt=[text_content1, text_content2],
            session_id=session_id,
        )
        
        # Verify conversation history has the concatenated message
        history = agent._sessions[session_id]["conversation_history"]
        assert len(history) == 1
        assert history[0]["role"] == "user"
        assert "First part." in history[0]["content"]
        assert "Second part." in history[0]["content"]


@pytest.mark.asyncio
async def test_prompt_with_image_content(agent, mock_connection):
    """Test handling image content in prompts."""
    agent.on_connect(mock_connection)
    
    # Create a session
    new_response = await agent.new_session(cwd="/tmp")
    session_id = new_response.session_id
    
    # Initialize conversation history
    agent._sessions[session_id]["conversation_history"] = []
    
    # Send image content
    image_content = ImageContent(
        data=b"fake_image_data",
        mime_type="image/png",
    )
    
    with patch("crow.agent.acp_server.Conversation") as mock_conv_class:
        mock_conv = MagicMock()
        mock_conv_class.return_value = mock_conv
        
        response = await agent.prompt(
            prompt=[image_content],
            session_id=session_id,
        )
        
        # Verify image was noted in conversation history
        history = agent._sessions[session_id]["conversation_history"]
        assert len(history) == 1
        assert "[Image: image/png]" in history[0]["content"]


@pytest.mark.asyncio
async def test_prompt_with_text_resource(agent, mock_connection):
    """Test handling embedded text resource in prompts."""
    agent.on_connect(mock_connection)
    
    # Create a session
    new_response = await agent.new_session(cwd="/tmp")
    session_id = new_response.session_id
    
    # Initialize conversation history
    agent._sessions[session_id]["conversation_history"] = []
    
    # Create embedded text resource
    from acp.schema import TextResourceContents
    resource = TextResourceContents(
        text="This is embedded text content",
        uri="file:///example.txt",
    )
    embedded_resource = EmbeddedResource(resource=resource)
    
    with patch("crow.agent.acp_server.Conversation") as mock_conv_class:
        mock_conv = MagicMock()
        mock_conv_class.return_value = mock_conv
        
        response = await agent.prompt(
            prompt=[embedded_resource],
            session_id=session_id,
        )
        
        # Verify resource text was extracted
        history = agent._sessions[session_id]["conversation_history"]
        assert len(history) == 1
        assert "This is embedded text content" in history[0]["content"]


@pytest.mark.asyncio
async def test_prompt_with_blob_resource(agent, mock_connection):
    """Test handling embedded blob resource in prompts."""
    agent.on_connect(mock_connection)
    
    # Create a session
    new_response = await agent.new_session(cwd="/tmp")
    session_id = new_response.session_id
    
    # Initialize conversation history
    agent._sessions[session_id]["conversation_history"] = []
    
    # Create embedded blob resource
    from acp.schema import BlobResourceContents
    resource = BlobResourceContents(
        blob=b"binary data",
        uri="file:///example.bin",
        mime_type="application/octet-stream",
    )
    embedded_resource = EmbeddedResource(resource=resource)
    
    with patch("crow.agent.acp_server.Conversation") as mock_conv_class:
        mock_conv = MagicMock()
        mock_conv_class.return_value = mock_conv
        
        response = await agent.prompt(
            prompt=[embedded_resource],
            session_id=session_id,
        )
        
        # Verify blob was noted
        history = agent._sessions[session_id]["conversation_history"]
        assert len(history) == 1
        assert "[Binary resource:" in history[0]["content"]


@pytest.mark.asyncio
async def test_prompt_with_mixed_content_types(agent, mock_connection):
    """Test handling mixed content types in a single prompt."""
    agent.on_connect(mock_connection)
    
    # Create a session
    new_response = await agent.new_session(cwd="/tmp")
    session_id = new_response.session_id
    
    # Initialize conversation history
    agent._sessions[session_id]["conversation_history"] = []
    
    # Create mixed content
    text_content = TextContent(text="Look at this image: ")
    image_content = ImageContent(
        data=b"fake_image_data",
        mime_type="image/jpeg",
    )
    text_content2 = TextContent(text=" What do you think?")
    
    with patch("crow.agent.acp_server.Conversation") as mock_conv_class:
        mock_conv = MagicMock()
        mock_conv_class.return_value = mock_conv
        
        response = await agent.prompt(
            prompt=[text_content, image_content, text_content2],
            session_id=session_id,
        )
        
        # Verify all content was combined
        history = agent._sessions[session_id]["conversation_history"]
        assert len(history) == 1
        content = history[0]["content"]
        assert "Look at this image:" in content
        assert "[Image: image/jpeg]" in content
        assert "What do you think?" in content


@pytest.mark.asyncio
async def test_prompt_with_empty_content_list(agent, mock_connection):
    """Test that empty prompt list raises an error."""
    agent.on_connect(mock_connection)
    
    # Create a session
    new_response = await agent.new_session(cwd="/tmp")
    session_id = new_response.session_id
    
    # Send empty prompt list
    with pytest.raises(ValueError, match="must contain at least one content block"):
        await agent.prompt(
            prompt=[],
            session_id=session_id,
        )


@pytest.mark.asyncio
async def test_prompt_with_invalid_content_type(agent, mock_connection):
    """Test handling of invalid content type."""
    agent.on_connect(mock_connection)
    
    # Create a session
    new_response = await agent.new_session(cwd="/tmp")
    session_id = new_response.session_id
    
    # Create invalid content object (missing required attributes)
    invalid_content = MagicMock()
    invalid_content.text = None
    invalid_content.data = None
    invalid_content.resource = None
    
    # This should raise an error or handle gracefully
    with pytest.raises(ValueError):
        await agent.prompt(
            prompt=[invalid_content],
            session_id=session_id,
        )


@pytest.mark.asyncio
async def test_initialize_advertises_content_capabilities(agent):
    """Test that initialize response advertises content type support."""
    response = await agent.initialize(
        protocol_version=1,
        client_capabilities={},
        client_info={"name": "test", "version": "1.0"},
    )
    
    # Check that prompt_capabilities includes content types
    # Note: InitializeResponse uses snake_case attributes
    assert hasattr(response, "agent_capabilities")
    prompt_caps = response.agent_capabilities.prompt_capabilities
    
    # Check the actual structure of prompt_capabilities
    # It's a PromptCapabilities object, not a dict
    assert hasattr(prompt_caps, "model_dump")
    caps_dict = prompt_caps.model_dump()
    
    # Current ACP schema has: audio, embedded_context, image
    # Text support is assumed to always be available
    assert "image" in caps_dict
    assert "audio" in caps_dict
    assert "embedded_context" in caps_dict


@pytest.mark.asyncio
async def test_image_content_different_mime_types(agent, mock_connection):
    """Test handling different image mime types."""
    agent.on_connect(mock_connection)
    
    # Create a session
    new_response = await agent.new_session(cwd="/tmp")
    session_id = new_response.session_id
    
    # Initialize conversation history
    agent._sessions[session_id]["conversation_history"] = []
    
    # Test different image formats
    mime_types = ["image/png", "image/jpeg", "image/gif", "image/webp"]
    
    for mime_type in mime_types:
        agent._sessions[session_id]["conversation_history"] = []
        
        image_content = ImageContent(
            data=b"fake_data",
            mime_type=mime_type,
        )
        
        with patch("crow.agent.acp_server.Conversation") as mock_conv_class:
            mock_conv = MagicMock()
            mock_conv_class.return_value = mock_conv
            
            response = await agent.prompt(
                prompt=[image_content],
                session_id=session_id,
            )
            
            history = agent._sessions[session_id]["conversation_history"]
            assert f"[Image: {mime_type}]" in history[0]["content"]
