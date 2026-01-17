#!/usr/bin/env python3
"""Test that crow ACP maintains conversation state across multiple prompts."""

import asyncio
import json
import sys


async def test_conversation_memory():
    """Send multiple prompts and verify conversation state is maintained."""
    from acp import stdio_streams
    from acp.core import AgentSideConnection
    
    # Start the server
    process = await asyncio.create_subprocess_exec(
        "uv",
        "run",
        "--project",
        "crow",
        "crow",
        "acp",
        stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    
    # Start a task to read stderr so it doesn't block
    async def read_stderr():
        while True:
            line = await process.stderr.readline()
            if not line:
                break
            print(f"[SERVER] {line.decode()}", end="", flush=True)
    
    asyncio.create_task(read_stderr())
    
    # Wait for server to start
    await asyncio.sleep(2)
    
    # Helper to send request and get response
    async def send_request(request):
        message = json.dumps(request) + "\n"
        process.stdin.write(message.encode())
        await process.stdin.drain()
        
        # Read response line
        line = await process.stdout.readline()
        if not line:
            return None
        return json.loads(line.decode())
    
    # Helper to read streaming updates
    async def read_updates():
        """Read all streaming updates until final response."""
        updates = []
        while True:
            line = await process.stdout.readline()
            if not line:
                break
            
            try:
                response = json.loads(line.decode())
                
                # Check if this is a notification or response
                if "method" in response and response["method"] == "session/update":
                    # This is a streaming update
                    update = response.get("params", {}).get("update", {})
                    update_type = update.get("@type", "unknown")
                    updates.append(update)
                    
                    # Show some details
                    if update_type == "agent_message_text":
                        text = update.get("text", "")
                        if text:
                            print(f"    Agent: {text[:200]}", flush=True)
                    elif update_type == "tool_call_start":
                        print(f"    Tool: {update.get('title', 'unknown')}", flush=True)
                        
                elif "id" in response:
                    # This is the final response
                    return response, updates
                    
            except json.JSONDecodeError:
                pass
        
        return None, updates
    
    try:
        # Initialize
        print("=== Testing Initialize ===")
        init_response = await send_request({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocol_version": 1,
                "client_capabilities": {},
                "client_info": {"name": "test", "version": "1.0"}
            }
        })
        print(f"✓ Initialize: {init_response is not None}")
        
        # Create session
        print("\n=== Creating Session ===")
        session_response = await send_request({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "session/new",
            "params": {
                "cwd": "/home/thomas/src/projects/orchestrator-project",
                "mcpServers": []
            }
        })
        if session_response and "result" in session_response:
            session_id = session_response["result"]["sessionId"]
            print(f"✓ Session created: {session_id}")
        else:
            print(f"✗ Session creation failed: {session_response}")
            return
        
        # First message
        print("\n=== Message 1: What is 2+2? ===")
        await send_request({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "session/prompt",
            "params": {
                "session_id": session_id,
                "prompt": [{"type": "text", "text": "What is 2+2? Give a brief answer."}]
            }
        })
        print("Reading response...")
        response, updates = await read_updates()
        print(f"✓ Message 1 complete (stop_reason: {response.get('result', {}).get('stop_reason') if response else 'none'})")
        
        # Second message
        print("\n=== Message 2: What did I just ask you? ===")
        await send_request({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "session/prompt",
            "params": {
                "session_id": session_id,
                "prompt": [{"type": "text", "text": "What did I just ask you? Be specific."}]
            }
        })
        print("Reading response...")
        response, updates = await read_updates()
        print(f"✓ Message 2 complete (stop_reason: {response.get('result', {}).get('stop_reason') if response else 'none'})")
        
        # Third message - check conversation memory
        print("\n=== Message 3: How many turns have we had? ===")
        await send_request({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "session/prompt",
            "params": {
                "session_id": session_id,
                "prompt": [{"type": "text", "text": "How many user messages have I sent in this conversation? Count them."}]
            }
        })
        print("Reading response...")
        response, updates = await read_updates()
        print(f"✓ Message 3 complete (stop_reason: {response.get('result', {}).get('stop_reason') if response else 'none'})")
        
        print("\n=== Test Complete ===")
        print("Check the agent responses above to see if it remembers:")
        print("1. First question: 'What is 2+2?'")
        print("2. Second question: 'What did I just ask you?'")
        print("3. Third question: 'How many user messages?'")
        print("\nIf it remembers, it should say:")
        print("- 2+2=4")
        print("- You asked 'What is 2+2?'")
        print("- You have sent 3 messages")
        
    finally:
        # Cleanup
        print("\n=== Cleaning up ===")
        process.terminate()
        await asyncio.sleep(1)
        if process.returncode is None:
            process.kill()
        print("✓ Server stopped")


if __name__ == "__main__":
    asyncio.run(test_conversation_memory())
