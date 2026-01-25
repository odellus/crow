# ACP Server Test Results

## Summary

Successfully tested the Crow ACP server by sending NDJSON messages to initialize a session and send a prompt. The server is working correctly!

## What Was Fixed

### Issue
The ACP server was failing with an internal error when trying to process prompts:
```
[Errno 21] Is a directory: '/home/thomas/src/projects/orchestrator-project/software-agent-sdk/openhands-sdk/openhands/sdk/agent/prompts/'
```

### Root Cause
We were passing `security_policy_filename: ""` (empty string) when creating the OpenHands Agent. This caused the SDK to try to read the prompts directory as a file.

### Solution
Removed the `security_policy_filename` parameter entirely from the agent initialization, allowing the OpenHands SDK to use its defaults.

## Test Results

### ✅ Initialize
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "agentCapabilities": {
      "loadSession": true,
      "mcpCapabilities": {
        "http": false,
        "sse": false
      },
      "promptCapabilities": {
        "image": true
      }
    },
    "agentInfo": {
      "name": "crow-acp-server",
      "title": "Crow ACP Server",
      "version": "0.1.0"
    },
    "protocolVersion": 1
  }
}
```

### ✅ Session/New
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "sessionId": "5aa4e798-a6c6-4571-ac23-97172c3c1ad7"
  }
}
```

### ✅ Session/Prompt with Streaming
The server successfully:
1. Sent a plan update with the task
2. Streamed agent response token-by-token via `session/update` notifications
3. Responded to the question "What is 2+2?" with "Hello! 2+2 equals 4. Is there anything else I can help you with today?..."

## How to Test

Run the manual test script:
```bash
cd /home/thomas/src/projects/orchestrator-project
uv run --project crow crow/test_acp_manual.py
```

## Key Learnings

1. **ACP Protocol Flow**: The protocol follows this sequence:
   - `initialize` → negotiate capabilities
   - `session/new` → create session with cwd and mcp_servers
   - `session/prompt` → send prompt with content blocks
   - `session/update` notifications → streaming updates

2. **Content Format**: Prompts use content blocks directly, not wrapped in role/parts:
   ```json
   {
     "prompt": [
       {"type": "text", "text": "Hello!"}
     ]
   }
   ```

3. **Response Format**: Responses use camelCase (e.g., `sessionId`) not snake_case

4. **Error Handling**: Added proper error handling in the prompt method to catch conversation failures and return appropriate error responses

## Next Steps

The ACP server is now functional and ready for integration with ACP clients like Zed or VS Code!
