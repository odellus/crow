#!/usr/bin/env python3
"""Test prompt method."""

import json
import sys
import subprocess
import time


def test_prompt():
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
        data = json.dumps(msg) + '\n'
        server.stdin.write(data.encode())
        server.stdin.flush()
        print(f"\n⬆️  Sent: {msg['method']}")
    
    def read_response(timeout=10):
        import select
        ready, _, _ = select.select([server.stdout], [], [], timeout)
        if ready:
            line = server.stdout.readline().decode().rstrip()
            if line:
                resp = json.loads(line)
                print(f"\n⬇️  Received: response")
                if 'error' in resp:
                    print(f"   ❌ Error: {resp['error']}")
                elif 'result' in resp:
                    print(f"   ✅ Success")
                    if 'sessionId' in resp.get('result', {}):
                        print(f"   📝 Session ID: {resp['result']['sessionId']}")
                return resp
        return None
    
    # Give server time to start
    time.sleep(1)
    
    try:
        # Initialize
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
        read_response(5)
        
        # Create new session
        send({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "session/new",
            "params": {"cwd": "/tmp", "mcpServers": []}
        })
        resp = read_response(10)
        session_id = resp['result']['sessionId']
        
        # Send a prompt - CORRECT API
        send({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "session/prompt",
            "params": {
                "sessionId": session_id,
                "prompt": [{"type": "text", "text": "Hello! Can you help me?"}]
            }
        })
        
        # Read a few responses
        for i in range(8):
            resp = read_response(5)
            if resp and 'result' in resp and resp.get('id') == 3:
                print(f"\n✅ Prompt completed with stopReason: {resp['result']}")
                break
        
        print("\n🎉 Prompt test completed!")
        return True
        
    finally:
        # Cleanup
        if server.poll() is None:
            server.terminate()
            server.wait(timeout=2)
