#!/usr/bin/env python3
"""Test that tool results are properly sent to ACP client."""

import asyncio
import json
import sys


async def test_tool_results():
    """Test that tool execution results are sent via ACP."""
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
    
    # Write all stdout to a file for jq parsing
    with open("test_output.ndjson", "w") as output_file:
        
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
        
        # Helper to send request
        async def send_request(request):
            message = json.dumps(request) + "\n"
            process.stdin.write(message.encode())
            await process.stdin.drain()
        
        # Helper to read all responses
        async def read_all_responses():
            """Read all responses until we get a final result."""
            responses = []
            while True:
                line = await process.stdout.readline()
                if not line:
                    break
                
                try:
                    response = json.loads(line.decode())
                    responses.append(response)
                    
                    # Write to file for jq
                    output_file.write(line.decode())
                    output_file.flush()
                    
                    # Check if this is a final response (has "id" and "result")
                    if "id" in response and "result" in response:
                        return responses
                        
                except json.JSONDecodeError as e:
                    print(f"[ERROR] Failed to parse JSON: {e}", flush=True)
                    print(f"[ERROR] Line: {line}", flush=True)
            
            return responses
        
        try:
            # Initialize
            print("=== Initialize ===", flush=True)
            await send_request({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocol_version": 1,
                    "client_capabilities": {},
                    "client_info": {"name": "test", "version": "1.0"}
                }
            })
            responses = await read_all_responses()
            print(f"✓ Got {len(responses)} responses", flush=True)
            
            # Create session
            print("\n=== Create Session ===", flush=True)
            await send_request({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "session/new",
                "params": {
                    "cwd": "/home/thomas/src/projects/orchestrator-project/crow",
                    "mcpServers": []
                }
            })
            responses = await read_all_responses()
            session_id = responses[-1]["result"]["sessionId"]
            print(f"✓ Session: {session_id}", flush=True)
            
            # Send a prompt that will trigger a tool call
            print("\n=== Test: List files in current directory ===", flush=True)
            await send_request({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "session/prompt",
                "params": {
                    "session_id": session_id,
                    "prompt": [{"type": "text", "text": "List the files in the current directory using the file_editor tool"}]
                }
            })
            
            print("Reading all responses...", flush=True)
            responses = await read_all_responses()
            
            # Analyze responses
            print(f"\n=== Analysis ===", flush=True)
            print(f"Total responses: {len(responses)}", flush=True)
            
            # Count different types
            updates = [r for r in responses if "method" in r and r["method"] == "session/update"]
            final = [r for r in responses if "id" in r and "result" in r]
            
            print(f"Session updates: {len(updates)}", flush=True)
            print(f"Final responses: {len(final)}", flush=True)
            
            # Look for tool calls
            tool_calls = []
            tool_results = []
            
            for update in updates:
                update_data = update.get("params", {}).get("update", {})
                update_type = update_data.get("sessionUpdate", "")
                
                if update_type == "tool_call_start":
                    tool_calls.append(update_data)
                    print(f"\n✓ Tool call started: {update_data.get('title', 'unknown')}", flush=True)
                    print(f"  Tool Call ID: {update_data.get('toolCallId', 'unknown')}", flush=True)
                    
                elif update_type == "tool_call_update":
                    status = update_data.get("status", "unknown")
                    tool_call_id = update_data.get("toolCallId", "unknown")
                    print(f"\n✓ Tool call update: {status}", flush=True)
                    print(f"  Tool Call ID: {tool_call_id}", flush=True)
                    
                    # Check for content/output
                    if "content" in update_data:
                        content = update_data["content"]
                        print(f"  Has content: {type(content)}", flush=True)
                        if isinstance(content, str):
                            print(f"  Content preview: {content[:200]}...", flush=True)
                            tool_results.append(content)
                    
                    if "rawOutput" in update_data:
                        raw_output = update_data["rawOutput"]
                        print(f"  Has rawOutput: {type(raw_output)}", flush=True)
                        if isinstance(raw_output, dict):
                            print(f"  RawOutput keys: {list(raw_output.keys())}", flush=True)
            
            print(f"\n=== Summary ===", flush=True)
            print(f"Tool calls started: {len(tool_calls)}", flush=True)
            print(f"Tool results found: {len(tool_results)}", flush=True)
            
            if tool_results:
                print("\n✓ SUCCESS: Tool results are being sent!", flush=True)
                print("\nFirst tool result:", flush=True)
                print(tool_results[0][:500] if len(tool_results[0]) > 500 else tool_results[0], flush=True)
            else:
                print("\n✗ FAILURE: No tool results found!", flush=True)
                print("\nDumping all updates for debugging:", flush=True)
                for i, update in enumerate(updates[:5]):  # First 5 updates
                    print(f"\n--- Update {i+1} ---", flush=True)
                    print(json.dumps(update, indent=2)[:500], flush=True)
            
        finally:
            # Cleanup
            print("\n=== Cleaning up ===", flush=True)
            process.terminate()
            await asyncio.sleep(1)
            if process.returncode is None:
                process.kill()
            print("✓ Server stopped", flush=True)
            print(f"\n✓ Output saved to test_output.ndjson", flush=True)
            print("You can parse it with:", flush=True)
            print("  jq 'select(.params.update.sessionUpdate == \"tool_call_update\")' test_output.ndjson", flush=True)


if __name__ == "__main__":
    asyncio.run(test_tool_results())
