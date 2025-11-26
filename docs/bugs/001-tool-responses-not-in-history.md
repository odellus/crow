# Bug 001: Tool Responses Not Added to Conversation History

**Date:** 2025-11-25
**Status:** Fixed
**Severity:** Critical

## Symptom

Agent would call the same tool 2-3 times in a row with identical arguments (doom loop), then fail with API error:

```
Error: LLM call failed: API call failed: invalid_request_error: Invalid request: 
an assistant message with 'tool_calls' must be followed by tool messages responding 
to each 'tool_call_id'. The following tool_call_ids did not have response messages: list:2
```

## Root Cause

In `executor.rs`, the `build_llm_context()` function was loading previous messages from the database but **only extracting text content**, completely ignoring tool calls and tool responses.

```rust
// BROKEN CODE - Only extracted text
Message::Assistant { .. } => {
    let text = msg.parts.iter()
        .filter_map(|p| match p {
            Part::Text { text, .. } => Some(text.as_str()),
            _ => None,  // ← IGNORED Part::Tool entries!
        })
        .join("\n");
    
    messages.push(ChatCompletionRequestMessage::Assistant(text));
}
```

When the agent made a second turn, it would:
1. Load previous messages (but lose all tool context)
2. Send to LLM without tool call/response history
3. LLM would re-request the same tools (no memory of having done them)
4. API would reject because tool_call_ids didn't have responses

## The Fix

Modified `build_llm_context()` to properly reconstruct the full message protocol:

```rust
Message::Assistant { .. } => {
    // Check if this message has tool calls
    let tool_parts: Vec<_> = msg.parts.iter()
        .filter_map(|p| {
            if let Part::Tool { call_id, tool, state, .. } = p {
                Some((call_id, tool, state))
            } else {
                None
            }
        })
        .collect();

    if !tool_parts.is_empty() {
        // Reconstruct assistant message WITH tool_calls field
        let openai_tool_calls = tool_parts.iter().map(|(call_id, tool_name, state)| {
            ChatCompletionMessageToolCall {
                id: call_id.clone(),
                function: FunctionCall {
                    name: tool_name.clone(),
                    arguments: serde_json::to_string(&state.input).unwrap(),
                },
            }
        }).collect();

        messages.push(ChatCompletionRequestMessage::Assistant(
            AssistantMessageArgs::default()
                .tool_calls(openai_tool_calls)
                .build()?
        ));

        // Add tool response messages for each completed tool
        for (call_id, _, state) in &tool_parts {
            if let ToolState::Completed { output, .. } = state {
                messages.push(ChatCompletionRequestMessage::Tool(
                    ToolMessageArgs::default()
                        .tool_call_id(call_id.clone())
                        .content(output.clone())
                        .build()?
                ));
            }
        }
    }
}
```

## Key Insight

The OpenAI API (and all compatible APIs) enforce this protocol:
- Every `assistant` message with `tool_calls` MUST be followed by `tool` messages for each call_id
- The order matters
- When reconstructing from database, you must rebuild this exact sequence

## Files Changed

- `crow-tauri/src-tauri/core/src/agent/executor.rs` (lines 954-1044)

## How to Verify Fix

1. Start a chat session
2. Ask something that triggers a tool call (e.g., "list files")
3. Ask a follow-up question about the results (e.g., "what files were in that list?")
4. Agent should answer from memory WITHOUT calling tools again

## Comparison with OpenCode

OpenCode uses the `ai` library which handles this automatically via `streamText()`. 
We implement the ReACT loop manually, so we must handle message protocol ourselves.

## Lesson Learned

When manually implementing a ReACT loop with persistent storage:
1. Store the full structure - not just extracted text
2. When loading from storage, RECONSTRUCT the exact message protocol
3. Don't lose context by filtering only text parts
