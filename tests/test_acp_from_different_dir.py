#!/usr/bin/env python3
"""Test crow-acp from a different directory like Zed would."""

import json
import subprocess
import sys
import tempfile
from pathlib import Path

def send_request(proc, request):
    """Send a JSON-RPC request to the server."""
    request_str = json.dumps(request) + "\n"
    print(f"→ Sending: {request_str.strip()}", file=sys.stderr)
    proc.stdin.write(request_str)
    proc.stdin.flush()

def read_response(proc, timeout=5):
    """Read a JSON-RPC response from the server."""
    import select
    
    # Wait for data with timeout
    ready, _, _ = select.select([proc.stdout], [], [], timeout)
    if ready:
        line = proc.stdout.readline()
        if line:
            response = json.loads(line)
            print(f"← Received: {json.dumps(response, indent=2)}", file=sys.stderr)
            return response
    
    print(f"✗ No response within {timeout}s", file=sys.stderr)
    return None

def main():
    """Test the ACP server from a different working directory."""
    # Use a temporary directory to simulate Zed's behavior
    with tempfile.TemporaryDirectory() as tmpdir:
        print(f"Testing from temp directory: {tmpdir}", file=sys.stderr)
        print("Starting ACP server...", file=sys.stderr)
        
        # Start the server process from a different directory
        proc = subprocess.Popen(
            ["crow-acp"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,  # Line buffered
            cwd=tmpdir,  # Run from temp directory like Zed might
        )
        
        # Start a thread to read stderr
        import threading
        def read_stderr():
            for line in proc.stderr:
                print(f"[SERVER STDERR] {line}", end='', file=sys.stderr)
        threading.Thread(target=read_stderr, daemon=True).start()
        
        request_id = 0
        
        try:
            # 1. Initialize
            print("\n=== Testing initialize ===", file=sys.stderr)
            request_id += 1
            send_request(proc, {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "initialize",
                "params": {
                    "protocol_version": 1,
                    "client_capabilities": {},
                    "client_info": {
                        "name": "zed-test",
                        "version": "1.0.0"
                    }
                }
            })
            response = read_response(proc)
            if not response or "error" in response:
                print(f"✗ Initialize failed", file=sys.stderr)
                if response and "error" in response:
                    print(f"Error: {response['error']}", file=sys.stderr)
                return
            
            print("✓ Initialize successful", file=sys.stderr)
            
            # 2. Create new session with the temp directory as cwd
            print("\n=== Testing session/new ===", file=sys.stderr)
            request_id += 1
            send_request(proc, {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "session/new",
                "params": {
                    "cwd": tmpdir,
                    "mcp_servers": []
                }
            })
            response = read_response(proc, timeout=10)
            if not response or "error" in response:
                print(f"✗ Session/new failed", file=sys.stderr)
                if response and "error" in response:
                    print(f"Error: {response['error']}", file=sys.stderr)
                return
            
            session_id = response.get("result", {}).get("sessionId")
            if not session_id:
                print(f"✗ No sessionId in response", file=sys.stderr)
                return
            
            print(f"✓ Session created: {session_id}", file=sys.stderr)
            
            # 3. Send a simple prompt
            print("\n=== Testing session/prompt ===", file=sys.stderr)
            request_id += 1
            send_request(proc, {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "session/prompt",
                "params": {
                    "session_id": session_id,
                    "prompt": [
                        {
                            "type": "text",
                            "text": "Say hello"
                        }
                    ]
                }
            })
            
            # Read a few responses
            print("Reading responses...", file=sys.stderr)
            for i in range(5):
                response = read_response(proc, timeout=10)
                if not response:
                    break
                print(f"Got response {i+1}", file=sys.stderr)
            
            print(f"\n✓ Test completed without errors", file=sys.stderr)
            
        finally:
            print("\n=== Cleaning up ===", file=sys.stderr)
            proc.stdin.close()
            proc.terminate()
            proc.wait(timeout=5)
            print("✓ Server stopped", file=sys.stderr)

if __name__ == "__main__":
    main()
