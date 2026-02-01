# Kimi-cli ACP Implementation Analysis

This document analyzes how kimi-cli implements advanced ACP features for rich tool call display.

## Overview

Kimi-cli provides gorgeous ACP responses with:
- **File diffs** for write/edit operations
- **Terminal output** displayed inline
- **Streaming tool call updates** with progressive argument display
- **Rich display blocks** for different content types

## Key Architecture Components

### 1. Display Block System

Kimi-cli uses a `DisplayBlock` system to represent rich content:

```python
class DiffDisplayBlock(DisplayBlock):
    """Display block describing a file diff."""
    type: str = "diff"
    path: str
    old_text: str
    new_text: str

class TodoDisplayBlock(DisplayBlock):
    """Display block describing a todo list update."""
    type: str = "todo"
    items: list[TodoDisplayItem]
```

### 2. Tool Result Structure

Tool results return `ToolReturnValue` with:
- `output`: The actual result (string or ContentPart list)
- `message`: Human-readable summary
- `display`: List of DisplayBlock objects for rich UI
- `is_error`: Whether the tool failed

```python
return ToolReturnValue(
    is_error=False,
    output="",
    message=f"File successfully overwritten. Current size: {file_size} bytes.",
    display=diff_blocks,  # List of DiffDisplayBlock
)
```

## ACP Content Types

Kimi-cli uses three ACP content types for tool calls:

### 1. ContentToolCallContent

Generic text content:

```python
acp.schema.ContentToolCallContent(
    type="content",
    content=acp.schema.TextContentBlock(type="text", text="output here")
)
```

### 2. FileEditToolCallContent

File diff display:

```python
acp.schema.FileEditToolCallContent(
    type="diff",
    path="/path/to/file.py",
    old_text="old content",
    new_text="new content"
)
```

### 3. TerminalToolCallContent

Terminal output streaming:

```python
acp.schema.TerminalToolCallContent(
    type="terminal",
    terminal_id="term_123",
)
```

## Tool Call Lifecycle

### 1. Tool Call Start

When a tool is called, kimi-cli sends:

```python
await self._conn.session_update(
    session_id=session_id,
    update=acp.schema.ToolCallStart(
        session_update="tool_call",
        tool_call_id=acp_tool_call_id,
        title="WriteFile: /path/to/file.py",  # Enhanced with key argument
        status="in_progress",
        content=[
            acp.schema.ContentToolCallContent(
                type="content",
                content=acp.schema.TextContentBlock(
                    type="text", 
                    text='{"path": "/path/to/file.py", "content": "..."}'
                )
            )
        ],
    ),
)
```

**Key features:**
- Unique `tool_call_id` prefixed with turn ID: `"{turn_id}/{tool_call_id}"`
- Enhanced title with key argument extracted from JSON
- Full arguments shown in content

### 2. Tool Call Progress (Streaming Arguments)

As arguments stream in, updates are sent:

```python
await self._conn.session_update(
    session_id=session_id,
    update=acp.schema.ToolCallProgress(
        session_update="tool_call_update",
        tool_call_id=acp_tool_call_id,
        title="WriteFile: /path/to/file.py",  # May update as args stream
        status="in_progress",
        content=[
            acp.schema.ContentToolCallContent(
                type="content",
                content=acp.schema.TextContentBlock(
                    type="text", 
                    text=accumulated_args  # Accumulated JSON
                )
            )
        ],
    ),
)
```

**Key features:**
- Arguments accumulate as they stream from LLM
- Title updates when key argument becomes available
- Shows progressive JSON building

### 3. Tool Call Completion

When tool finishes, result is sent:

```python
update = acp.schema.ToolCallProgress(
    session_update="tool_call_update",
    tool_call_id=acp_tool_call_id,
    status="completed",  # or "failed"
    content=[
        # Rich content from tool result
        acp.schema.FileEditToolCallContent(
            type="diff",
            path="/path/to/file.py",
            old_text="old line\n",
            new_text="new line\n"
        ),
        acp.schema.ContentToolCallContent(
            type="content",
            content=acp.schema.TextContentBlock(
                type="text", 
                text="File successfully overwritten."
            )
        )
    ],
)

await self._conn.session_update(session_id=session_id, update=update)
```

## File Operations with Diffs

### WriteFile Tool

The WriteFile tool generates diffs:

```python
# 1. Read old content if file exists
old_text = await p.read_text(errors="replace") if file_existed else ""

# 2. Build diff blocks
diff_blocks = list(build_diff_blocks(
    params.path,
    old_text or "",
    new_text,
))

# 3. Return with display
return ToolReturnValue(
    is_error=False,
    output="",
    message=f"File successfully overwritten. Current size: {file_size} bytes.",
    display=diff_blocks,  # DiffDisplayBlock objects
)
```

### Diff Generation

The `build_diff_blocks` function:

```python
def build_diff_blocks(
    path: str,
    old_text: str,
    new_text: str,
) -> list[DiffDisplayBlock]:
    """Build diff display blocks grouped with small context windows."""
    old_lines = old_text.splitlines()
    new_lines = new_text.splitlines()
    matcher = SequenceMatcher(None, old_lines, new_lines, autojunk=False)
    blocks: list[DiffDisplayBlock] = []
    
    # Group changes with 3 lines of context
    for group in matcher.get_grouped_opcodes(n=3):
        if not group:
            continue
        i1 = group[0][1]
        i2 = group[-1][2]
        j1 = group[0][3]
        j2 = group[-1][4]
        blocks.append(
            DiffDisplayBlock(
                path=path,
                old_text="\n".join(old_lines[i1:i2]),
                new_text="\n".join(new_lines[j1:j2]),
            )
        )
    return blocks
```

**Key features:**
- Uses `SequenceMatcher` for diff generation
- Groups changes with context windows (3 lines)
- Multiple blocks for large files
- Only shows changed regions + context

## Terminal Operations

### Terminal Tool

The Terminal tool uses ACP's terminal streaming:

```python
# 1. Create terminal
term = await self._acp_conn.create_terminal(
    command=params.command,
    session_id=self._acp_session_id,
    output_byte_limit=builder.max_chars,
)

# 2. Send terminal content to client
await self._acp_conn.session_update(
    session_id=self._acp_session_id,
    update=acp.schema.ToolCallProgress(
        session_update="tool_call_update",
        tool_call_id=acp_tool_call_id,
        status="in_progress",
        content=[
            acp.schema.TerminalToolCallContent(
                type="terminal",
                terminal_id=terminal.id,
            )
        ],
    ),
)

# 3. Wait for completion
exit_status = await terminal.wait_for_exit()

# 4. Get final output
output_response = await terminal.current_output()

# 5. Return result (output hidden because terminal streams it)
return builder.ok(f"Command executed successfully.")
```

**Key features:**
- Uses `HideOutputDisplayBlock()` to prevent double display
- Terminal streams output directly to client via ACP
- Only shows summary message in tool result

## Conversion Functions

### DisplayBlock to ACP Content

```python
def display_block_to_acp_content(
    block: DisplayBlock,
) -> acp.schema.FileEditToolCallContent | None:
    if isinstance(block, DiffDisplayBlock):
        return acp.schema.FileEditToolCallContent(
            type="diff",
            path=block.path,
            old_text=block.old_text,
            new_text=block.new_text,
        )
    return None
```

### ToolResult to ACP Content

```python
def tool_result_to_acp_content(
    tool_ret: ToolReturnValue,
) -> list[
    acp.schema.ContentToolCallContent
    | acp.schema.FileEditToolCallContent
    | acp.schema.TerminalToolCallContent
]:
    contents: list[...] = []
    
    # Process display blocks
    for block in tool_ret.display:
        if isinstance(block, HideOutputDisplayBlock):
            return []  # Don't show output
        
        content = display_block_to_acp_content(block)
        if content is not None:
            contents.append(content)
    
    # Add output text
    if tool_ret.output:
        contents.append(
            acp.schema.ContentToolCallContent(
                type="content",
                content=acp.schema.TextContentBlock(
                    type="text", 
                    text=tool_ret.output
                )
            )
        )
    
    # Fallback to message
    if not contents and tool_ret.message:
        contents.append(
            acp.schema.ContentToolCallContent(
                type="content",
                content=acp.schema.TextContentBlock(
                    type="text", 
                    text=tool_ret.message
                )
            )
        )
    
    return contents
```

## Tool Call ID Management

Kimi-cli uses unique tool call IDs to avoid conflicts:

```python
class _ToolCallState:
    @property
    def acp_tool_call_id(self) -> str:
        # Prefix with turn ID for uniqueness
        turn_id = _current_turn_id.get()
        assert turn_id is not None
        return f"{turn_id}/{self.tool_call.id}"
```

**Why this matters:**
- Tool calls can be rejected/cancelled and retried
- LLM may reuse tool call IDs
- ACP client needs unique IDs per actual tool invocation
- Turn ID ensures uniqueness across retries

## Enhanced Titles

Kimi-cli extracts key arguments for better titles:

```python
def get_title(self) -> str:
    """Get the current title with subtitle if available."""
    tool_name = self.tool_call.function.name
    subtitle = extract_key_argument(self.lexer, tool_name)
    if subtitle:
        return f"{tool_name}: {subtitle}"
    return tool_name
```

**Examples:**
- `WriteFile` → `WriteFile: /path/to/file.py`
- `Shell` → `Shell: ls -la`
- `ReadFile` → `ReadFile: README.md`

## Implementation Checklist for Crow

To match kimi-cli's ACP features:

### 1. Add DisplayBlock System
- [ ] Create `DiffDisplayBlock` class
- [ ] Create `TodoDisplayBlock` class
- [ ] Create `HideOutputDisplayBlock` class

### 2. Modify Tool Return Values
- [ ] Add `display` field to tool results
- [ ] Generate diffs for file operations
- [ ] Return rich display blocks

### 3. Implement ACP Content Conversion
- [ ] Convert DisplayBlocks to ACP content types
- [ ] Handle FileEditToolCallContent for diffs
- [ ] Handle TerminalToolCallContent for terminal

### 4. Enhance Tool Call Updates
- [ ] Add enhanced titles with key arguments
- [ ] Stream tool call arguments progressively
- [ ] Send rich content on completion

### 5. Terminal Integration
- [ ] Use ACP's create_terminal for shell commands
- [ ] Send TerminalToolCallContent during execution
- [ ] Hide duplicate output with HideOutputDisplayBlock

### 6. Tool Call ID Management
- [ ] Prefix tool call IDs with turn/session ID
- [ ] Ensure uniqueness across retries

### 7. Diff Generation
- [ ] Implement build_diff_blocks function
- [ ] Use SequenceMatcher for diff generation
- [ ] Group changes with context windows

## References

- kimi-cli source: `/home/thomas/src/projects/orchestrator-project/kimi-cli/src/kimi_cli/`
- ACP tools: `/home/thomas/src/projects/orchestrator-project/kimi-cli/src/kimi_cli/acp/tools.py`
- File utils: `/home/thomas/src/projects/orchestrator-project/kimi-cli/src/kimi_cli/tools/file/utils.py`
- Convert module: `/home/thomas/src/projects/orchestrator-project/kimi-cli/src/kimi_cli/acp/convert.py`
- Session module: `/home/thomas/src/projects/orchestrator-project/kimi-cli/src/kimi_cli/acp/session.py`
