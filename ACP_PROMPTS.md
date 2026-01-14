# ACP Development Prompt Collection

A collection of focused prompts for developing and testing ACP features in Crow.

## Phase 1: Cancellation Prompts

### Research & Understanding
```
I need to implement cancellation support for the Crow ACP server. 

Current state:
- ACP server is in src/crow/acp_server.py
- OpenHands SDK has pause() functionality
- ACP spec: https://agentclientprotocol.com/protocol/draft/cancellation.md

Please:
1. Read the current ACP server implementation
2. Read the ACP cancellation spec
3. Explain how to map session/cancel to OpenHands pause()
4. Show me the exact code changes needed
```

### Implementation
```
Implement the cancel() method in the Crow ACP server:

Requirements:
- Map session/cancel → OpenHands pause()
- Return stopReason: "cancelled" from prompt()
- Handle graceful shutdown of streaming updates
- No resource leaks

Files to modify:
- src/crow/acp_server.py

Please implement this and show me the diff.
```

### Testing
```
Write tests for ACP cancellation functionality:

Requirements:
- Test that cancel() pauses the conversation
- Test that prompt() returns stopReason: "cancelled"
- Test graceful shutdown of streaming updates
- Test resource cleanup

Use the existing test structure in tests/test_acp_server.py

Please write the test file.
```

---

## Phase 2: Tool Call Reporting Prompts

### Research & Understanding
```
I need to implement tool call reporting for the Crow ACP server.

Current state:
- ACP server sends agent_message_chunk updates
- Tool executions happen in OpenHands SDK
- ACP spec: https://agentclientprotocol.com/protocol/tool-calls.md

Please:
1. Explain how to hook into OpenHands tool execution lifecycle
2. Show me how to send tool_call notifications
3. Explain how to map OpenHands tools to ACP tool kinds
4. Show me the exact code structure needed
```

### Implementation
```
Implement tool call reporting in the Crow ACP server:

Requirements:
- Send tool_call notification when tools start (include: toolCallId, title, kind, status)
- Send tool_call_update with status progression (pending → in_progress → completed/error)
- Map OpenHands tools to ACP kinds:
  * read - file reading operations
  * edit - file modifications
  * delete - file deletion
  * execute - terminal commands
  * think - internal reasoning
  * fetch - web requests
  * other - default
- Handle tool errors properly

Files to modify:
- src/crow/acp_server.py

Please implement this step by step.
```

### Testing
```
Write tests for tool call reporting:

Requirements:
- Test tool_call notifications are sent
- Test status progression (pending → in_progress → completed)
- Test error handling (tool_call_update with error status)
- Test tool kind mapping for different tool types
- Test with various OpenHands tools (file_editor, terminal, etc.)

Use the existing test structure in tests/test_acp_server.py

Please write comprehensive tests.
```

---

## Phase 3: Permission Requests Prompts

### Status: SKIPPED

We are NOT implementing ACP permission requests. Instead, we're taking a simpler approach:

**Approach**:
- Set OpenHands `security_policy_filename=""` to disable all security checks
- This auto-approves all tool execution
- No need to implement ACP `request_permission` flow
- No need to handle permission state (allow_once, allow_always, reject_once, reject_always)

**Rationale**:
- ACP permission requests require client interaction (request/response pattern)
- We don't want to block execution waiting for user approval
- We don't need actual security enforcement in this context
- Disabling OpenHands security policy achieves the same goal with less code

**Implementation**:
- Already done in `src/crow/agent/acp_server.py` line 161
- Just set `security_policy_filename=""` when creating the OpenHands Agent

**Testing**:
- No tests needed - we're not implementing this feature
- Tools should execute without prompting for permission

---

## Phase 4: Agent Plans Prompts

### Research & Understanding
```
I need to implement agent plan reporting for the Crow ACP server.

Current state:
- OpenHands uses task_tracker for planning
- ACP spec: https://agentclientprotocol.com/protocol/agent-plan.md

Please:
1. Explain how to detect when LLM creates execution plans
2. Show me how to hook into task_tracker
3. Explain how to send plan updates via session/update
4. Show me how to update plan entry statuses as work progresses
```

### Implementation
```
Implement agent plan reporting in the Crow ACP server:

Requirements:
- Detect when LLM creates execution plans (via task_tracker)
- Send plan updates via session/update
- Update plan entry statuses as work progresses
- Handle plan completion

Files to modify:
- src/crow/acp_server.py

Please implement this step by step.
```

### Testing
```
Write tests for agent plan reporting:

Requirements:
- Test plan detection and reporting
- Test plan entry status updates
- Test plan completion signaling
- Test with various plan structures

Use the existing test structure in tests/test_acp_server.py

Please write comprehensive tests.
```

---

## Phase 5: Integration Testing Prompts

### Test with Debug Client
```
I need to test the Crow ACP server with the debug client.

Current state:
- Debug client: OpenHands-CLI/scripts/acp/debug_client.py
- ACP server: src/crow/acp_server.py

Please:
1. Show me how to start the ACP server
2. Show me how to run the debug client
3. Create a test script that verifies:
   - Initialization works
   - Session creation works
   - Prompt with streaming works
   - Tool calls are reported
   - Permissions work
   - Cancellation works
```

### Test with Real Clients
```
I need to test the Crow ACP server with real ACP clients (Zed, VS Code).

Please:
1. Show me how to configure Zed to use the Crow ACP server
2. Show me how to configure VS Code to use the Crow ACP server
3. Create a test plan for verifying compatibility
4. Show me how to debug client-specific issues
```

---

## Debugging Prompts

### Tool Calls Not Working
```
Tool call reporting is not working in my ACP server.

Symptoms:
- Tools execute but no tool_call notifications are sent
- Only agent_message_chunk updates appear

Please help me debug:
1. Check if hooks are properly registered
2. Verify tool execution lifecycle is being intercepted
3. Check if session/update notifications are being sent
4. Show me how to add debug logging to trace the issue
```

### Permissions Not Working
```
Permission requests are not working in my ACP server.

Symptoms:
- Tools execute without asking for permission
- session/request_permission is never called

Please help me debug:
1. Check if permission interception is set up
2. Verify the permission hook is being called
3. Check if client supports session/request_permission
4. Show me how to add debug logging to trace the issue
```

### Cancellation Not Working
```
Cancellation is not working in my ACP server.

Symptoms:
- cancel() method is called but execution continues
- stopReason: "cancelled" is never returned

Please help me debug:
1. Check if cancel() is properly mapped to pause()
2. Verify pause() is being called
3. Check if prompt() checks for cancellation
4. Show me how to add debug logging to trace the issue
```

---

## Documentation Prompts

### Architecture Documentation
```
Write architecture documentation for the Crow ACP server.

Please create docs/ACP_ARCHITECTURE.md with:
- Overview of the ACP server architecture
- How OpenHands SDK is integrated
- How streaming works
- How tool calls are reported
- How permissions work
- How cancellation works
- How agent plans work
- Key design decisions
- Diagram of the architecture
```

### Testing Documentation
```
Write testing documentation for the Crow ACP server.

Please create docs/ACP_TESTING.md with:
- How to run tests
- How to write new tests
- Test structure and conventions
- How to test with debug client
- How to test with real clients
- Coverage requirements
- CI/CD integration
```

### Troubleshooting Documentation
```
Write troubleshooting documentation for the Crow ACP server.

Please create docs/ACP_TROUBLESHOOTING.md with:
- Common issues and solutions
- How to debug tool call issues
- How to debug permission issues
- How to debug cancellation issues
- How to debug streaming issues
- How to enable debug logging
- How to report bugs
```

---

## Quick Reference Prompts

### "What's the status?"
```
Show me the current implementation status of ACP features in Crow.

Please:
1. Read ACP_INTEGRATION_STATUS.md
2. Summarize what's implemented
3. Summarize what's missing
4. Show me the priority order
5. Tell me what to work on next
```

### "How do I test this?"
```
Show me how to test a specific ACP feature.

Feature: [INSERT FEATURE HERE]

Please:
1. Show me the relevant test file
2. Explain how to run the tests
3. Show me how to add new tests
4. Show me how to debug test failures
```

### "How do I implement this?"
```
Show me how to implement a specific ACP feature.

Feature: [INSERT FEATURE HERE]

Please:
1. Read the ACP spec for this feature
2. Read the current implementation
3. Show me what code changes are needed
4. Show me what tests are needed
5. Walk me through the implementation step by step
```

---

## Usage Instructions

### For Each Phase
1. Start with the "Research & Understanding" prompt
2. Use the "Implementation" prompt to build it
3. Use the "Testing" prompt to verify it works
4. Use debugging prompts if you run into issues

### For Quick Tasks
- Use "Quick Reference" prompts for status checks
- Use debugging prompts when things break
- Use documentation prompts to document your work

### For Full Development
- Go through each phase in order (1 → 2 → 3 → 4 → 5)
- Use the prompts as a guide
- Adapt them to your specific needs
- Add your own prompts as needed

---

## Tips

1. **Be specific**: Replace [INSERT FEATURE HERE] with actual feature names
2. **Provide context**: Include error messages, stack traces, or code snippets
3. **Iterate**: Follow up with clarifying questions based on the responses
4. **Document**: Update documentation as you complete each phase
5. **Test early**: Write tests alongside implementation, not after

---

## Notes

- These prompts are starting points - adapt them to your needs
- Some prompts assume knowledge of the codebase - provide context if needed
- Use the ACP spec links in each prompt for reference
- Check ACP_INTEGRATION_STATUS.md for current implementation status
- Use existing tests (tests/test_acp_server.py) as reference
