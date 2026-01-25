#!/usr/bin/env python3
"""Full flow test of Crow ACP server including prompt."""

import json
import sys
import subprocess
import time


def run_full_test():
    # Server startup command
    cmd = '''cd /home/thomas/src/projects/orchestrator-project/crow && . .venv/bin/activate && exec crow-acp'''
    
    print("🚀 Starting Crow ACP server...")
    server = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        shell=True,
        bufsize=0
    )
    
    def send(msg):
        """Send a JSON-RPC message."""
        data = json.dumps(msg) + '\n'
        server.stdin.write(data.encode())
        server.stdin.flush()
        print(f"\n⬆️  Sent: {msg['method']}")
    
    def read_response(timeout=10):
        """Read a JSON-RPC response."""
        import select
        ready, _, _ = select.select([server.stdout], [], [], timeout)
        if ready:
            line = server.stdout.readline().decode().rstrip()
            if line:
                resp = json.loads(line)
                print(f"\n⬇️  Received: response")
                if 'error' in resp:
                    print(f"   ❌ Error: {resp['error']}")
                    sys.exit(1)
                return resp
        return None
    
    # Give server time to start
    time.sleep(1)
    
    try:
        # Initialize
        print("\n🔧 Step 1: Initialize connection")
        send({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": 1,
                "clientCapabilities": {
                    "fs": {"readTextFile": True, "writeTextFile": True},
                    "terminal": True,
                },
                "clientInfo": {"name": "test", "version": "1.0"},
            }
        })
        
        resp = read_response(5)
        if not resp:
            print("❌ No response to initialize!")
            return False
            
        print("   ✅ Initialize successful")
        
        # Create new session
        print("\n🔧 Step 2: Create new session")
        send({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "session/new",
            "params": {"cwd": "/tmp", "mcpServers": []}
        })
        
        resp = read_response(10)
        if resp and 'result' in resp:
            session_id = resp['result'].get('sessionId', 'N/A')
            print(f"   ✅ New session successful")
            print(f"   📝 Session ID: {session_id}")
        else:
            print(f"❌ New session failed: {resp}")
            return False
        
        # Send a prompt
        print("\n🔧 Step 3: Send test prompt")
        send({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "session/prompt",
            "params": {
                "sessionId": session_id,
                "agentRequest": {
                    "role": "user",
                    "content": [{"type": "text", "text": "Hello! Can you help me?"}],
                },
            },
        })
        
        # Read responses (might be multiple streaming responses)
        print("\n📨 Reading responses (streaming)...")
        for i in range(5):
            resp = read_response(5)
            if resp:
                print(f"   Response {i+1}: {resp.get('method', 'response')}")
        
        print("\n🎉 Full flow test completed!")
        return True
        
    finally:
        # Cleanup
        print("\n🧹 Shutting down server...")
        if server.poll() is None:
            server.terminate()
            try:
                server.wait(timeout=5)
            except subprocess.TimeoutExpired:
                server.kill()
        
        # Print stderr for debugging
        stderr = server.stderr.read().decode()
        if stderr:
            print(f"\n📛 STDERR for debugging:\n{stderr[-2000:]}")  # Last 2000 chars


if __name__ == "__main__":
    success = run_full_test()
    sys.exit(0 if success else 1)
