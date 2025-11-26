# Crow Testing Framework Plan

## Overview

**EVERY TOOL GETS TESTS. NO EXCEPTIONS.**

This document outlines a comprehensive testing strategy for crow-tauri. Every single tool/component MUST have:
- Unit tests
- Integration tests  
- E2E tests via CLI

**Testing Philosophy:**
1. Look at crow code to understand current implementation
2. Compare against OpenCode to identify missing features (IGNORE LSP and permissions)
3. Unit test the shit out of each tool
4. Integration test with other components
5. E2E test via CLI with **FRESH SESSIONS ALWAYS**
6. Verify XDG persistence is correct

**CRITICAL RULES:**
- ⚠️ **ALWAYS create new session for each test** - reusing sessions causes weird model behavior
- ⚠️ **Verify XDG paths** - check files appear in correct locations
- ⚠️ **Compare with OpenCode** but IGNORE LSP and permissions features

---

## 🚨 CRITICAL PRIORITY: Task Tool

**THE MOST IMPORTANT TOOL TO TEST. THIS IS THE CORE OF THE AGENT SYSTEM.**

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/task.rs`
- OpenCode: `opencode/packages/opencode/src/tool/task.ts`

### Why Task is Critical
The Task tool is THE mechanism for:
- Launching subagents for complex multi-step tasks
- Creating child sessions with parent-child relationships
- Enabling the dual-agent architecture (executor/discriminator)
- Breaking down complex work into manageable pieces

**If Task doesn't work perfectly, the entire agent system fails.**

### Crow Features
- Launches subagents for complex tasks
- Creates child sessions with parent-child relationships
- Validates subagent type (is_subagent check)
- Dynamic description generation with agent capabilities
- Extracts text output from response parts
- Returns metadata with session ID and subagent type
- Uses SessionStore, AgentRegistry, ToolRegistry, SessionLockManager

### OpenCode Features (gaps to implement - IGNORE LSP/PERMISSIONS)
- ❌ Session continuation/reuse (session_id parameter to resume)
- ❌ Event bus integration for real-time progress tracking
- ❌ Model selection per subagent
- ❌ Tool access control per subagent
- ❌ Cancellation support (AbortController/AbortSignal)
- ❌ Metadata subscription during execution
- ❌ Tool parts collection from subagent
- ❌ task_metadata XML block in output

### Unit Test Cases (MUST HAVE ALL)
```rust
// Core functionality
test_task_creates_child_session
test_task_inherits_working_dir_from_parent
test_task_validates_subagent_type
test_task_rejects_non_subagent_agents
test_task_generates_dynamic_description
test_task_parses_input_correctly
test_task_returns_metadata_with_session_id
test_task_returns_metadata_with_subagent_type

// Error handling
test_task_error_invalid_agent_name
test_task_error_missing_prompt_parameter
test_task_error_missing_subagent_type
test_task_error_agent_registry_failure
test_task_error_session_creation_failure
test_task_error_executor_failure

// Agent registry integration
test_task_agent_registry_lookup_success
test_task_agent_registry_lookup_failure
test_task_agent_list_subagents_only
test_task_agent_description_includes_capabilities

// Session management
test_task_session_has_correct_parent_id
test_task_session_has_correct_project_id
test_task_session_message_added_before_execution
test_task_child_session_persisted_to_xdg

// Output handling
test_task_extracts_text_from_response_parts
test_task_handles_empty_response
test_task_handles_multi_part_response
test_task_handles_tool_parts_in_response
```

### Integration Test Cases (MUST HAVE ALL)
```rust
test_task_full_executor_pipeline
test_task_with_session_store_persistence
test_task_with_tool_registry_access
test_task_child_can_use_read_tool
test_task_child_can_use_write_tool
test_task_child_can_use_edit_tool
test_task_child_can_use_bash_tool
test_task_child_can_use_grep_tool
test_task_child_inherits_snapshot_context
test_task_parent_receives_child_output
test_task_multiple_sequential_subagents
test_task_nested_task_calls (task within task)
test_task_concurrent_subagent_isolation
test_task_xdg_session_file_created
test_task_xdg_message_file_created
```

### E2E Test Cases (MUST HAVE ALL)
```bash
# Basic subagent launch
crow-cli chat "use a subagent to list files in current directory"
# Verify: child session created, output returned

# Complex multi-step task
crow-cli chat "use a subagent to analyze the codebase structure"
# Verify: subagent executes multiple tools, returns summary

# Task with specific agent type
crow-cli chat "launch explore subagent to find all test files"
# Verify: correct agent used, results accurate

# Task failure handling
crow-cli chat "launch nonexistent-agent to do something"
# Verify: helpful error message, no crash

# Session isolation
crow-cli chat "create subagent session A" 
crow-cli chat "create subagent session B"
# Verify: sessions are isolated, no cross-contamination

# XDG verification
crow-cli chat "use subagent for task"
ls ~/.local/share/crow/storage/session/*/
# Verify: child session JSON file exists

# Output completeness
crow-cli chat --json "use subagent to read README.md"
# Verify: JSON output contains full subagent response
```

---

## 🚨 HIGH PRIORITY: TodoWrite & TodoRead Tools

**CRITICAL FOR TASK TRACKING AND AGENT COORDINATION**

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/todowrite.rs`, `todoread.rs`
- OpenCode: `opencode/packages/opencode/src/tool/todo.ts`

### Why Todo is Critical
- Tracks task progress across agent turns
- Enables agents to break down complex work
- Provides visibility into what the agent is doing
- Essential for multi-turn conversations

### Crow Features
- In-memory storage per session (HashMap)
- Persistent XDG storage: `~/.local/share/crow/storage/todo/{sessionID}.json`
- Three status states: Pending, InProgress, Completed
- TodoItem: content, status, activeForm
- Session-scoped todos
- Paired TodoReadTool

### OpenCode Features (gaps - IGNORE LSP/PERMISSIONS)
- ❌ Four status states (+cancelled)
- ❌ Priority field (high/medium/low)
- ❌ Unique ID for each todo item
- ❌ Event bus integration
- ❌ Storage abstraction with file locking

### XDG Path
```
~/.local/share/crow/storage/todo/{sessionID}.json
```

### Unit Test Cases (MUST HAVE ALL)
```rust
// TodoWrite core
test_todowrite_creates_new_todo
test_todowrite_updates_existing_todo
test_todowrite_replaces_entire_list
test_todowrite_empty_list_clears_todos
test_todowrite_validates_status_enum
test_todowrite_validates_content_required
test_todowrite_validates_activeform_required
test_todowrite_handles_special_characters
test_todowrite_handles_unicode_content
test_todowrite_handles_very_long_content
test_todowrite_handles_multiple_todos
test_todowrite_preserves_order

// TodoWrite status transitions
test_todowrite_pending_to_in_progress
test_todowrite_in_progress_to_completed
test_todowrite_pending_to_completed
test_todowrite_completed_cannot_change (or can it?)

// TodoWrite persistence
test_todowrite_persists_to_xdg_path
test_todowrite_creates_parent_directories
test_todowrite_overwrites_existing_file
test_todowrite_file_format_is_valid_json
test_todowrite_survives_process_restart

// TodoRead core
test_todoread_returns_current_todos
test_todoread_returns_empty_when_none
test_todoread_reads_from_xdg_path
test_todoread_handles_missing_file
test_todoread_handles_corrupted_file

// Session isolation
test_todo_session_a_isolated_from_session_b
test_todo_different_sessions_different_files
test_todo_session_id_in_filename

// Error handling
test_todowrite_invalid_json_input
test_todowrite_missing_required_fields
test_todoread_permission_error_graceful
```

### Integration Test Cases (MUST HAVE ALL)
```rust
test_todowrite_then_todoread_roundtrip
test_todowrite_in_executor_pipeline
test_todoread_in_executor_pipeline
test_todo_with_session_store
test_todo_persists_across_multiple_turns
test_todo_xdg_file_contains_correct_data
test_todo_concurrent_writes_same_session
test_todo_concurrent_reads_same_session
test_todo_with_task_tool_subagent
test_todo_visible_in_session_export
```

### E2E Test Cases (MUST HAVE ALL)
```bash
# Basic todo creation
crow-cli chat "create a todo list with 3 items"
# Verify: todos created, visible in output

# Todo status updates
crow-cli chat "mark the first todo as in progress"
crow-cli chat "mark it as completed"
# Verify: status changes reflected

# Todo persistence
crow-cli chat "create todos"
crow-cli session todo <session-id>
# Verify: todos visible via CLI command

# XDG file verification
crow-cli chat "create todo list"
cat ~/.local/share/crow/storage/todo/<session-id>.json
# Verify: file exists, valid JSON, correct content

# Session isolation
crow-cli chat --session A "create todo: task A"
crow-cli chat --session B "create todo: task B"
crow-cli session todo A
crow-cli session todo B
# Verify: each session has only its own todos

# Complex workflow
crow-cli chat "plan a refactoring task with 5 steps"
crow-cli chat "start working on step 1"
crow-cli chat "complete step 1, start step 2"
# Verify: todo states update correctly
```

---

## Table of Contents (Full Test Coverage Required)

| # | Tool/Component | Current Tests | Required | Priority |
|---|---------------|---------------|----------|----------|
| 1 | **Task** | 0 | 30+ | 🚨 CRITICAL |
| 2 | **TodoWrite/Read** | 1 | 25+ | 🚨 CRITICAL |
| 3 | Read | 4 | 15+ | HIGH |
| 4 | Write | 2 | 15+ | HIGH |
| 5 | Edit | 70+ | ✅ Good | MEDIUM |
| 6 | Bash | 20+ | ✅ Good | MEDIUM |
| 7 | Grep | 1 | 15+ | HIGH |
| 8 | Glob | 0 | 15+ | HIGH |
| 9 | List | 0 | 15+ | HIGH |
| 10 | Batch | 0 | 15+ | HIGH |
| 11 | Patch | 0 | 15+ | HIGH |
| 12 | WebFetch | 3 | 10+ | MEDIUM |
| 13 | WebSearch | 1 | 10+ | MEDIUM |
| 14 | Snapshot | 10+ | 20+ | HIGH |
| 15 | Session Store | 0 | 20+ | HIGH |

---

## 3. Read Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/read.rs`
- OpenCode: `opencode/packages/opencode/src/tool/read.ts`

### Crow Features
- Line numbering (1-indexed, cat -n style)
- Offset and limit parameters (default 2000 lines)
- Max line length truncation (2000 chars)
- Empty file detection with warning
- File size metadata in response
- Async file I/O with tokio

### OpenCode Features (gaps - IGNORE LSP/PERMISSIONS)
- ❌ Image file support (JPEG, PNG, GIF, BMP, WebP with base64)
- ❌ Binary file detection (magic bytes + heuristic analysis)
- ❌ File not found with fuzzy filename suggestions
- ❌ Relative path resolution to CWD
- ❌ Zero-padded line numbers (5 digits)

### Unit Test Cases
```rust
test_read_basic_file_default_params
test_read_line_numbers_1_indexed
test_read_offset_parameter
test_read_limit_parameter
test_read_offset_and_limit_combined
test_read_empty_file_warning
test_read_line_truncation_at_2000_chars
test_read_non_existent_file_error
test_read_large_file_limit_applied
test_read_lf_line_endings
test_read_crlf_line_endings
test_read_mixed_line_endings
test_read_offset_beyond_file_length
test_read_unicode_content
test_read_binary_file_handling
```

### Integration Test Cases
```rust
test_read_with_session_context
test_read_write_roundtrip
test_read_edit_verify_workflow
test_read_multiple_files_same_session
test_read_concurrent_different_files
test_read_large_file_performance
```

### E2E Test Cases
```bash
crow-cli chat "read file.txt"
crow-cli chat "read file.txt starting at line 50"
crow-cli chat "read first 10 lines of file.txt"
crow-cli chat "read nonexistent.txt"  # error handling
crow-cli chat "read empty.txt"  # warning message
```

---

## 4. Write Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/write.rs`
- OpenCode: `opencode/packages/opencode/src/tool/write.ts`

### Crow Features
- Basic file write with async I/O (tokio::fs)
- Automatic parent directory creation
- File existence detection
- Metrics tracking (bytes_written, existed flag)

### OpenCode Features (gaps - IGNORE LSP/PERMISSIONS)
- ❌ FileTime tracking for concurrent modification detection
- ❌ Event bus for file change notifications

### Unit Test Cases
```rust
test_write_new_file
test_write_overwrite_existing
test_write_creates_parent_directories
test_write_empty_file
test_write_large_file_1mb
test_write_special_characters_in_path
test_write_relative_path
test_write_absolute_path
test_write_unicode_content
test_write_various_line_endings
test_write_metrics_bytes_written
test_write_metrics_existed_flag
test_write_invalid_path_error
```

### Integration Test Cases
```rust
test_write_read_roundtrip
test_write_triggers_snapshot
test_write_session_persistence
test_write_concurrent_different_files
```

### E2E Test Cases
```bash
crow-cli chat "create file test.txt with content hello"
crow-cli chat "write to existing file"
crow-cli chat "create file in nested/dir/path.txt"
```

---

## 5. Edit Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/edit.rs`
- OpenCode: `opencode/packages/opencode/src/tool/edit.ts`

### Status: ✅ Well Tested (70+ existing tests)

### Crow Features
- 9 cascading fuzzy replacer strategies
- Levenshtein distance for similarity
- Unified diff generation
- replaceAll parameter support

### OpenCode Features (gaps - IGNORE LSP/PERMISSIONS)
- ❌ FileTime tracking (prevent editing without reading)
- ❌ Snapshot/FileDiff tracking for session history
- ❌ Bus event publishing for file edits

### Additional Tests Needed
```rust
test_edit_triggers_snapshot_patch
test_edit_session_persistence
test_edit_xdg_session_export_shows_diff
```

---

## 6. Bash Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/bash.rs`
- OpenCode: `opencode/packages/opencode/src/tool/bash.ts`

### Status: ✅ Well Tested (20+ existing tests)

### Crow Features
- Command execution via bash -c with tokio
- Timeout handling (default 120s, max 600s)
- Output truncation at 30,000 characters
- Abort/cancellation via CancellationToken
- Process tree killing

### OpenCode Features (gaps - IGNORE LSP/PERMISSIONS)
- ❌ Stream-based metadata updates (real-time output)
- ❌ Detached process mode

### Additional Tests Needed
```rust
test_bash_triggers_snapshot
test_bash_session_persistence
test_bash_with_task_subagent
```

---

## 7. Grep Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/grep.rs`
- OpenCode: `opencode/packages/opencode/src/tool/grep.ts`

### Crow Features
- Ripgrep via Command::new("rg")
- Full regex pattern support
- File filtering via --glob
- Results sorted by modification time
- 100 match limit with truncation

### Unit Test Cases
```rust
test_grep_basic_pattern
test_grep_regex_pattern
test_grep_case_insensitive
test_grep_glob_filter
test_grep_no_matches
test_grep_truncation_at_100
test_grep_file_grouping
test_grep_unicode_pattern
test_grep_special_regex_chars
test_grep_multiline_pattern
test_grep_hidden_files
test_grep_respects_gitignore
```

### Integration Test Cases
```rust
test_grep_with_session_context
test_grep_with_glob_pipeline
test_grep_concurrent_searches
```

### E2E Test Cases
```bash
crow-cli chat "search for TODO in codebase"
crow-cli chat "grep pattern in *.rs files"
crow-cli chat "find all function definitions"
```

---

## 8. Glob Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/glob.rs`
- OpenCode: `opencode/packages/opencode/src/tool/glob.ts`

### Crow Features
- Ripgrep via rg --files --glob
- Optional path parameter
- 100 result truncation

### Unit Test Cases
```rust
test_glob_simple_pattern
test_glob_recursive_pattern
test_glob_extension_filter
test_glob_no_matches
test_glob_truncation_at_100
test_glob_with_path_parameter
test_glob_default_path
test_glob_hidden_files
test_glob_gitignore_respect
```

### Integration Test Cases
```rust
test_glob_with_session_context
test_glob_grep_pipeline
test_glob_read_pipeline
```

### E2E Test Cases
```bash
crow-cli chat "find all rust files"
crow-cli chat "glob **/*.ts in src/"
crow-cli chat "find files matching *.md"
```

---

## 9. List Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/list.rs`
- OpenCode: `opencode/packages/opencode/src/tool/ls.ts`

### Crow Features
- Lists files and directories
- Glob pattern filtering
- Recursive traversal option
- FileEntry objects with metadata

### Unit Test Cases
```rust
test_list_basic_directory
test_list_empty_directory
test_list_with_pattern
test_list_recursive
test_list_non_recursive
test_list_directories_first
test_list_alphabetical_sort
test_list_hidden_files_excluded
test_list_file_size_metadata
test_list_is_dir_flag
test_list_invalid_path_error
```

### Integration Test Cases
```rust
test_list_with_session_context
test_list_read_pipeline
test_list_large_directory
```

### E2E Test Cases
```bash
crow-cli chat "list files in current directory"
crow-cli chat "list only rust files"
crow-cli chat "list recursively"
```

---

## 10. Batch Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/batch.rs`
- OpenCode: `opencode/packages/opencode/src/tool/batch.ts`

### Crow Features
- Up to 10 parallel tool calls
- Disallows: batch, edit, todoread
- Concurrent execution with tokio::spawn
- Success/failure tracking

### Unit Test Cases
```rust
test_batch_single_tool
test_batch_multiple_tools
test_batch_max_10_tools
test_batch_rejects_over_10
test_batch_disallows_batch
test_batch_disallows_edit
test_batch_disallows_todoread
test_batch_parallel_execution
test_batch_partial_success
test_batch_all_success
test_batch_all_failure
test_batch_mixed_results
test_batch_invalid_tool_name
test_batch_empty_tool_calls_error
```

### Integration Test Cases
```rust
test_batch_with_session_context
test_batch_read_multiple_files
test_batch_grep_multiple_patterns
test_batch_concurrent_isolation
```

### E2E Test Cases
```bash
crow-cli chat "read 5 files in parallel"
crow-cli chat "search for multiple patterns"
```

---

## 11. Patch Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/patch.rs`
- OpenCode: `opencode/packages/opencode/src/tool/patch.ts`

### Crow Features
- Unified diff format parsing
- Add/update/delete operations
- Hunk-based parsing
- Context line matching

### Unit Test Cases
```rust
test_patch_add_new_file
test_patch_update_existing_file
test_patch_delete_file
test_patch_multiple_files
test_patch_multiple_hunks
test_patch_context_matching
test_patch_creates_parent_dirs
test_patch_empty_patch_error
test_patch_invalid_format_error
test_patch_absolute_path
test_patch_relative_path
test_patch_partial_success
```

### Integration Test Cases
```rust
test_patch_triggers_snapshot
test_patch_session_persistence
test_patch_with_edit_comparison
```

### E2E Test Cases
```bash
crow-cli chat "apply this diff to refactor code"
crow-cli chat "create multiple files via patch"
```

---

## 12. Snapshot Mechanism

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/snapshot/mod.rs`
- OpenCode: `opencode/packages/opencode/src/snapshot/index.ts`

### Crow Features
- Shadow git repository per project
- track(), patch(), revert(), restore(), diff(), diff_full()

### XDG Path
```
~/.local/share/crow/snapshots/{project_id}/
```

### Unit Test Cases
```rust
test_snapshot_track_creates_hash
test_snapshot_track_same_content_same_hash
test_snapshot_patch_detects_changes
test_snapshot_patch_empty_when_no_changes
test_snapshot_revert_restores_files
test_snapshot_revert_deletes_new_files
test_snapshot_diff_unified_format
test_snapshot_diff_full_per_file
test_snapshot_xdg_path_structure
test_snapshot_project_isolation
test_snapshot_nested_directories
test_snapshot_binary_files
test_snapshot_symlinks
test_snapshot_unicode_filenames
test_snapshot_hidden_files
test_snapshot_concurrent_operations
```

### Integration Test Cases
```rust
test_snapshot_with_edit_tool
test_snapshot_with_write_tool
test_snapshot_with_bash_tool
test_snapshot_with_session_store
```

### E2E Test Cases
```bash
crow-cli snapshot list
crow-cli snapshot show
crow-cli snapshot diff
crow-cli chat "make changes then revert"
```

---

## 13. Session Storage

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/session/store.rs`
- OpenCode: `opencode/packages/opencode/src/session/index.ts`

### Crow Features
- In-memory HashMap with Arc<RwLock<>>
- Session CRUD, message management
- Real-time markdown export
- Event publishing

### XDG Paths
```
~/.local/share/crow/storage/session/{projectID}/{sessionID}.json
~/.local/share/crow/storage/message/{sessionID}/{messageID}.json
.crow/sessions/{sessionID}.md (project-relative export)
```

### Unit Test Cases
```rust
test_session_create
test_session_get
test_session_list
test_session_update
test_session_delete
test_session_add_message
test_session_get_messages
test_session_child_relationships
test_session_xdg_path_creation
test_session_json_serialization
test_session_markdown_export
test_session_event_publishing
test_session_concurrent_access
test_session_project_id_computation
```

### Integration Test Cases
```rust
test_session_with_tool_execution
test_session_with_task_tool
test_session_with_todo_persistence
test_session_persistence_across_restart
```

### E2E Test Cases
```bash
crow-cli sessions
crow-cli session info <id>
crow-cli session history <id>
crow-cli session todo <id>
ls ~/.local/share/crow/storage/session/
```

---

## 14. XDG Path Summary

| Component | Path |
|-----------|------|
| Data Home | `~/.local/share/crow/` |
| Config Home | `~/.config/crow/` |
| State Home | `~/.local/state/crow/` |
| Sessions | `~/.local/share/crow/storage/session/{projectID}/{sessionID}.json` |
| Messages | `~/.local/share/crow/storage/message/{sessionID}/{messageID}.json` |
| Parts | `~/.local/share/crow/storage/part/{messageID}/{partID}.json` |
| Todos | `~/.local/share/crow/storage/todo/{sessionID}.json` |
| Snapshots | `~/.local/share/crow/snapshots/{project_id}/` |
| Logs | `~/.local/state/crow/logs/` |
| Agent Log | `~/.local/state/crow/logs/agent.log` |
| Tool Calls | `~/.local/state/crow/logs/tool-calls.jsonl` |
| AGENTS.md | `~/.config/crow/AGENTS.md` |
| Session Export | `.crow/sessions/{sessionID}.md` (project-relative) |

---

## 15. Test Execution Strategy

### Phase 1: CRITICAL (Do First)
1. **Task Tool** - 30+ tests across all levels
2. **TodoWrite/TodoRead** - 25+ tests across all levels

### Phase 2: HIGH Priority
3. Glob (0 → 15+ tests)
4. List (0 → 15+ tests)
5. Batch (0 → 15+ tests)
6. Patch (0 → 15+ tests)
7. Session Store (0 → 20+ tests)
8. Grep (1 → 15+ tests)

### Phase 3: MEDIUM Priority
9. Read (4 → 15+ tests)
10. Write (2 → 15+ tests)
11. Snapshot (10+ → 20+ tests)
12. WebFetch (3 → 10+ tests)
13. WebSearch (1 → 10+ tests)

### Phase 4: Already Good (Verify)
14. Edit (70+ tests) - verify integration/E2E
15. Bash (20+ tests) - verify integration/E2E

---

## Test Harness Pattern

**ALWAYS create fresh session for each E2E test:**

```rust
#[tokio::test]
async fn test_e2e_example() {
    // 1. Create fresh session
    let session_id = create_fresh_session().await;
    
    // 2. Run CLI command
    let output = run_cli(&[
        "chat", "--json", 
        "--session", &session_id, 
        "your test prompt"
    ]).await;
    
    // 3. Parse and verify
    let result: JsonOutput = parse_output(&output);
    assert!(result.tools.iter().any(|t| t.name == "expected_tool"));
    
    // 4. Verify XDG paths
    assert!(xdg_file_exists(&format!(
        "~/.local/share/crow/storage/session/*/{}.json", 
        session_id
    )));
    
    // 5. Cleanup
    cleanup_session(&session_id).await;
}
```

---

*Generated: 2025-11-26*
*IGNORE: LSP features, Permission system*
*FOCUS: Task tool, Todo tools, XDG persistence, Fresh sessions*
