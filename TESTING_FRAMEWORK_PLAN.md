# Crow Testing Framework Plan

## Overview

This document outlines a comprehensive testing strategy for crow-tauri, comparing each tool/component against OpenCode's implementation to identify gaps and define test cases at unit, integration, and E2E levels.

**Testing Philosophy:**
1. Look at crow code to understand current implementation
2. Compare against OpenCode to identify missing features
3. Unit test the shit out of each tool
4. Integration test with other components
5. E2E test via CLI with fresh sessions
6. Verify XDG persistence is correct

---

## Table of Contents

1. [Read Tool](#1-read-tool)
2. [Write Tool](#2-write-tool)
3. [Edit Tool](#3-edit-tool)
4. [Bash Tool](#4-bash-tool)
5. [Grep Tool](#5-grep-tool)
6. [Glob Tool](#6-glob-tool)
7. [List Tool](#7-list-tool)
8. [TodoWrite Tool](#8-todowrite-tool)
9. [WebFetch & WebSearch Tools](#9-webfetch--websearch-tools)
10. [Task, Batch, Patch Tools](#10-task-batch-patch-tools)
11. [Snapshot Mechanism](#11-snapshot-mechanism)
12. [Session Storage](#12-session-storage)
13. [XDG Path Summary](#13-xdg-path-summary)
14. [Test Execution Strategy](#14-test-execution-strategy)

---

## 1. Read Tool

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
- 4 existing unit tests

### OpenCode Features (gaps to implement)
- ❌ Image file support (JPEG, PNG, GIF, BMP, WebP with base64)
- ❌ Binary file detection (magic bytes + heuristic analysis)
- ❌ File not found with fuzzy filename suggestions
- ❌ Security blocking for .env files
- ❌ Permission system for external directory access
- ❌ FileTime session-based read tracking
- ❌ Relative path resolution to CWD
- ❌ Zero-padded line numbers (5 digits)

### Unit Test Cases
```
test_basic_file_read_default_params
test_line_numbers_1_indexed
test_offset_and_limit_parameters
test_empty_file_handling
test_line_truncation_at_2000_chars
test_non_existent_file_error
test_large_file_limit_applied
test_various_line_endings (LF, CRLF, mixed)
test_offset_beyond_file_length
test_image_file_detection (future)
test_binary_file_detection (future)
test_env_file_blocking (future)
```

### Integration Test Cases
```
test_read_write_sequence
test_read_edit_integration
test_multi_file_read_session
test_concurrent_reads
test_large_file_performance
```

### E2E Test Cases
```
CLI: crow-cli chat "read /path/to/file.txt"
CLI: crow-cli chat "read file with offset 100 limit 50"
CLI: crow-cli chat "read non-existent file" → helpful error
CLI: crow-cli chat "read empty file" → warning message
CLI: crow-cli chat "read very large file" → limited output
```

---

## 2. Write Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/write.rs`
- OpenCode: `opencode/packages/opencode/src/tool/write.ts`

### Crow Features
- Basic file write with async I/O (tokio::fs)
- Automatic parent directory creation
- File existence detection
- Metrics tracking (bytes_written, existed flag)
- JSON output with filepath and stats

### OpenCode Features (gaps to implement)
- ❌ Permission system (external_directory, edit permissions)
- ❌ FileTime tracking for concurrent modification detection
- ❌ LSP integration with diagnostics on write
- ❌ Event bus for file change notifications
- ❌ Diagnostic output in tool response

### Unit Test Cases
```
test_write_new_file_empty_path
test_write_overwrite_existing
test_write_creates_parent_directories
test_write_empty_file
test_write_large_file_1mb
test_write_special_characters_in_path
test_write_relative_path_normalization
test_write_unicode_content
test_write_various_line_endings
test_invalid_json_input_handling
```

### Integration Test Cases
```
test_write_read_roundtrip
test_write_triggers_snapshot_tracking
test_write_session_persistence
test_concurrent_writes_different_files
```

### E2E Test Cases
```
CLI: crow-cli chat "create file test.txt with content hello"
CLI: crow-cli chat "write file and verify content"
CLI: Write outside working directory → permission check (future)
```

---

## 3. Edit Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/edit.rs`
- OpenCode: `opencode/packages/opencode/src/tool/edit.ts`

### Crow Features
- 9 cascading fuzzy replacer strategies:
  1. Simple exact match
  2. Line-trimmed matching
  3. Block anchor matching
  4. Whitespace-normalized matching
  5. Indentation-flexible matching
  6. Escape-normalized matching
  7. Trimmed boundary matching
  8. Context-aware matching
  9. Multi-occurrence matching
- Levenshtein distance for similarity
- Unified diff generation
- replaceAll parameter support
- 70+ existing unit tests

### OpenCode Features (gaps to implement)
- ❌ Permission system (external_directory, edit)
- ❌ LSP diagnostics reporting after edit
- ❌ FileTime tracking (prevent editing without reading)
- ❌ Snapshot/FileDiff tracking for session history
- ❌ Bus event publishing for file edits

### Unit Test Cases
```
test_exact_string_replacement
test_replace_all_occurrences
test_error_when_old_equals_new
test_error_when_not_found
test_error_multiple_matches_no_replaceall
test_multiline_replacement
test_special_characters_escaping
test_empty_old_string_handling
test_fuzzy_whitespace_normalization
test_fuzzy_indentation_handling
test_block_anchor_matching
test_levenshtein_distance_calculations
test_diff_generation_accuracy
```

### Integration Test Cases
```
test_edit_triggers_snapshot_patch
test_edit_session_persistence
test_edit_after_read_filetime (future)
test_edit_lsp_diagnostics (future)
```

### E2E Test Cases
```
CLI: crow-cli chat "edit file.rs change foo to bar"
CLI: crow-cli chat "edit with fuzzy matching"
CLI: crow-cli chat "replace all occurrences"
CLI: Full workflow: read → edit → verify
```

---

## 4. Bash Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/bash.rs`
- OpenCode: `opencode/packages/opencode/src/tool/bash.ts`

### Crow Features
- Command execution via bash -c with tokio
- Timeout handling (default 120s, max 600s)
- Output truncation at 30,000 characters
- Abort/cancellation via CancellationToken
- Process tree killing (SIGTERM → grace → SIGKILL)
- Stderr/stdout capture combined
- Exit code reporting
- 20+ existing unit tests

### OpenCode Features (gaps to implement)
- ❌ Permission system with AST-based command analysis
- ❌ Tree-sitter parsing for bash commands
- ❌ Stream-based metadata updates (real-time output)
- ❌ Windows-specific path normalization (Git Bash)
- ❌ Detached process mode
- ❌ Wildcard-based permission matching

### Unit Test Cases
```
test_basic_echo_command
test_error_nonzero_exit_code
test_stderr_capture
test_combined_stdout_stderr
test_working_directory_pwd
test_multiline_output
test_specific_exit_codes
test_environment_variable_access
test_command_chaining_and
test_pipe_operations
test_command_substitution
test_arithmetic_expansion
test_quote_handling
test_custom_timeout
test_output_truncation_boundary
test_timeout_enforcement
test_cancellation_abort
```

### Integration Test Cases
```
test_bash_with_session_context
test_bash_triggers_snapshot_tracking
test_bash_abort_signal_integration
test_concurrent_bash_commands
```

### E2E Test Cases
```
CLI: crow-cli chat "run ls -la"
CLI: crow-cli chat "run command with timeout"
CLI: crow-cli chat "run long command" → abort
CLI: crow-cli chat "run failing command" → error handling
```

---

## 5. Grep Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/grep.rs`
- OpenCode: `opencode/packages/opencode/src/tool/grep.ts`

### Crow Features
- Ripgrep via Command::new("rg")
- Full regex pattern support
- File filtering via --glob
- Results sorted by modification time
- Groups output by file path
- 100 match limit with truncation metadata

### OpenCode Features (gaps to implement)
- ❌ Ripgrep binary auto-download/management
- ❌ JSON output parsing support
- ❌ File tree generation from results
- ❌ Better Instance.directory context

### Unit Test Cases
```
test_basic_regex_pattern
test_complex_regex_capture_groups
test_case_insensitive_search
test_multiline_patterns
test_empty_pattern_validation
test_invalid_regex_error
test_glob_filtering
test_no_matches_result
test_match_count_accuracy
test_truncation_at_100_matches
test_file_grouping_format
test_unicode_patterns
test_hidden_files
```

### Integration Test Cases
```
test_grep_with_glob_pipeline
test_grep_with_session_context
test_concurrent_grep_operations
test_grep_respects_gitignore
```

### E2E Test Cases
```
CLI: crow-cli chat "search for pattern in codebase"
CLI: crow-cli chat "grep with glob filter *.rs"
CLI: crow-cli chat "search returns sorted by mtime"
```

---

## 6. Glob Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/glob.rs`
- OpenCode: `opencode/packages/opencode/src/tool/glob.ts`

### Crow Features
- Ripgrep via rg --files --glob
- Optional path parameter (defaults to ".")
- 100 result truncation with warning
- Count and truncated status metadata

### OpenCode Features (gaps to implement)
- ❌ Modification time sorting of results
- ❌ Result title field (relative path)
- ❌ Instance context management
- ❌ Path resolution against Instance.directory

### Unit Test Cases
```
test_simple_glob_pattern
test_no_matches_message
test_optional_path_parameter
test_nested_directory_patterns
test_file_extension_patterns
test_result_truncation_at_100
test_metadata_count_truncated
test_mtime_sorting (future)
test_gitignore_respect
test_hidden_files
```

### Integration Test Cases
```
test_glob_grep_pipeline
test_glob_with_tool_registry
test_project_root_detection
test_concurrent_glob_operations
```

### E2E Test Cases
```
CLI: crow-cli chat "find all rust files"
CLI: crow-cli chat "glob **/*.ts in path"
CLI: Large codebase performance test
```

---

## 7. List Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/list.rs`
- OpenCode: `opencode/packages/opencode/src/tool/ls.ts`

### Crow Features
- Lists files and directories
- Glob pattern filtering
- Recursive directory traversal
- FileEntry objects (name, path, is_dir, size)
- Directories first, alphabetical sorting
- Skips hidden files (starting with '.')

### OpenCode Features (gaps to implement)
- ❌ Built-in ignore patterns (node_modules, .git, etc.)
- ❌ Ripgrep-backed enumeration for performance
- ❌ Customizable ignore patterns as array
- ❌ Directory tree ASCII rendering
- ❌ Includes hidden files by default
- ❌ 100 file output limit

### Unit Test Cases
```
test_basic_list_absolute_path
test_empty_directory
test_pattern_filtering_glob
test_directory_detection_is_dir
test_file_size_metadata
test_hidden_file_handling
test_ignore_pattern_application (future)
test_recursive_vs_non_recursive
test_absolute_vs_relative_paths
test_invalid_path_error
test_sorting_order
test_tree_rendering (future)
```

### Integration Test Cases
```
test_list_with_read_tool
test_list_with_session_context
test_large_directory_handling
test_symlink_handling
```

### E2E Test Cases
```
CLI: crow-cli chat "list files in directory"
CLI: crow-cli chat "list with pattern *.rs"
CLI: crow-cli chat "list recursively"
```

---

## 8. TodoWrite Tool

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/todowrite.rs`
- OpenCode: `opencode/packages/opencode/src/tool/todo.ts`

### Crow Features
- In-memory storage per session (HashMap)
- Persistent XDG storage: `~/.local/share/crow/storage/todo/{sessionID}.json`
- Three status states: Pending, InProgress, Completed
- TodoItem: content, status, activeForm
- Session-scoped todos
- Paired TodoReadTool

### OpenCode Features (gaps to implement)
- ❌ Four status states (+cancelled)
- ❌ Priority field (high/medium/low)
- ❌ Unique ID for each todo item
- ❌ Event bus integration
- ❌ Storage abstraction with locking

### XDG Path
```
~/.local/share/crow/storage/todo/{sessionID}.json
```

### Unit Test Cases
```
test_valid_todo_creation
test_invalid_input_handling
test_status_enum_validation
test_empty_todo_list
test_todo_count_calculation
test_activeform_presence
test_priority_field (future)
test_unique_id_generation (future)
test_cancelled_status (future)
test_multiple_todos_batch
test_special_characters_content
test_concurrent_writes
```

### Integration Test Cases
```
test_todowrite_todoread_roundtrip
test_session_isolation
test_persistence_restart
test_event_bus_notification (future)
test_xdg_path_creation
```

### E2E Test Cases
```
CLI: crow-cli chat "create todo list for task"
CLI: crow-cli session todo <id>
CLI: Verify persistence across sessions
```

---

## 9. WebFetch & WebSearch Tools

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/webfetch.rs`, `websearch.rs`
- OpenCode: `opencode/packages/opencode/src/tool/webfetch.ts`, `websearch.ts`

### Crow WebFetch Features
- HTTP/HTTPS fetching with 30s timeout
- HTTP to HTTPS auto-upgrade
- Simple HTML tag stripper
- 50KB truncation
- Custom User-Agent

### OpenCode WebFetch Features (gaps)
- ❌ Multiple output formats (text, markdown, html)
- ❌ TurndownService for HTML-to-Markdown
- ❌ 5MB limit (vs 50KB)
- ❌ 15-minute self-cleaning cache
- ❌ Configurable timeout up to 120s

### Crow WebSearch Features
- SearXNG backend (localhost:8082)
- JSON response parsing
- Infobox support
- Result limiting (default 5)

### OpenCode WebSearch Features (gaps)
- ❌ Exa AI backend
- ❌ Live crawl modes (fallback/preferred)
- ❌ Search types (auto/fast/deep)
- ❌ Context length optimization
- ❌ SSE response parsing

### Unit Test Cases
```
# WebFetch
test_basic_fetch
test_http_to_https_upgrade
test_html_stripping
test_timeout_configuration
test_size_limit_enforcement
test_format_parameter (future)
test_markdown_conversion (future)

# WebSearch
test_basic_search
test_result_limit
test_empty_results
test_timeout_handling
test_livecrawl_modes (future)
test_search_types (future)
```

### E2E Test Cases
```
CLI: crow-cli chat "fetch https://example.com"
CLI: crow-cli chat "search for rust programming"
```

---

## 10. Task, Batch, Patch Tools

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/tools/task.rs`, `batch.rs`, `patch.rs`
- OpenCode: `opencode/packages/opencode/src/tool/task.ts`, `batch.ts`, `patch.ts`

### Task Tool
**Crow Features:**
- Launches subagents for complex tasks
- Creates child sessions with parent-child relationships
- Validates subagent type

**Gaps:**
- ❌ Session continuation/reuse
- ❌ Event bus integration
- ❌ Model selection per subagent
- ❌ Tool access control per subagent
- ❌ Cancellation support

### Batch Tool
**Crow Features:**
- Up to 10 parallel tool calls
- Disallows: batch, edit, todoread
- Concurrent execution with tokio::spawn
- Success/failure tracking

**Gaps:**
- ❌ Tool state persistence in session
- ❌ Timing tracking (start/end)
- ❌ Attachments handling

### Patch Tool
**Crow Features:**
- Unified diff format parsing
- Add/update/delete operations
- Hunk-based parsing
- Context line matching

**Gaps:**
- ❌ Custom patch format (BEGIN/END markers)
- ❌ FileTime tracking
- ❌ Permission system
- ❌ File move support
- ❌ Chunk-based matching algorithm

### Unit Test Cases
```
# Task
test_task_invalid_agent
test_task_missing_parameters
test_task_non_subagent_error
test_task_working_dir_inheritance
test_task_session_creation

# Batch
test_batch_empty_tool_calls
test_batch_max_size_10
test_batch_disallowed_tools
test_batch_parallel_execution
test_batch_partial_success

# Patch
test_patch_empty_error
test_patch_invalid_format
test_patch_add_new_file
test_patch_update_existing
test_patch_delete_file
test_patch_multiple_hunks
test_patch_context_matching
```

---

## 11. Snapshot Mechanism

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/snapshot/mod.rs`
- OpenCode: `opencode/packages/opencode/src/snapshot/index.ts`

### Crow Features
- Shadow git repository per project
- `track()` - Creates git tree hash snapshot
- `patch(hash)` - Returns changed files since snapshot
- `revert(patches)` - Reverts files to snapshot state
- `restore(hash)` - Full restoration
- `diff(hash)` - Unified diff
- `diff_full(from, to)` - Detailed per-file diffs
- 10+ unit tests

### OpenCode Features (gaps)
- ❌ Configuration toggle (snapshot: boolean)
- ❌ Session-level revert/unrevert integration
- ❌ Git worktree support
- ❌ 40+ comprehensive tests

### XDG Path
```
~/.local/share/crow/snapshots/{project_id}/
```

### Unit Test Cases
```
test_snapshot_track_and_revert
test_snapshot_new_file_delete_on_revert
test_snapshot_diff
test_snapshot_multiple_files
test_snapshot_nested_directories
test_snapshot_restore_full
test_snapshot_diff_full_stats
test_snapshot_xdg_path_structure
test_snapshot_incremental_tracking
test_snapshot_empty_directory
test_binary_file_handling
test_symlink_handling
test_unicode_filenames
test_hidden_files
test_gitignore_changes
test_concurrent_operations
test_project_isolation
```

### E2E Test Cases
```
CLI: crow-cli snapshot list
CLI: crow-cli snapshot show
CLI: crow-cli snapshot diff
CLI: Full workflow: track → modify → patch → revert
```

---

## 12. Session Storage

**Files:**
- Crow: `crow-tauri/src-tauri/core/src/session/store.rs`
- OpenCode: `opencode/packages/opencode/src/session/index.ts`

### Crow Features
- In-memory HashMap with Arc<RwLock<>>
- Async/sync initialization
- Session CRUD operations
- Message management
- Child session support
- Real-time markdown export
- Event publishing

### OpenCode Features (gaps)
- ❌ Session forking with message cloning
- ❌ Auto-sharing functionality
- ❌ Session compaction for context overflow
- ❌ Revert/unrevert functionality
- ❌ Cost calculation and token analysis
- ❌ File diff tracking in session.summary

### XDG Paths
```
~/.local/share/crow/storage/session/{projectID}/{sessionID}.json
~/.local/share/crow/storage/message/{sessionID}/{messageID}.json
~/.local/share/crow/storage/part/{messageID}/{partID}.json
.crow/sessions/{sessionID}.md (project-relative export)
```

### Unit Test Cases
```
test_session_creation
test_session_crud_operations
test_message_management
test_session_metadata
test_storage_serialization
test_xdg_path_resolution
test_timestamp_handling
test_project_id_computation
test_child_session_relationships
test_event_publishing
test_export_functionality
test_token_tracking
```

### E2E Test Cases
```
CLI: crow-cli session create --title "Test"
CLI: crow-cli sessions
CLI: crow-cli session info <id>
CLI: crow-cli session history <id>
CLI: Verify .crow/sessions/{id}.md export
```

---

## 13. XDG Path Summary

All crow-tauri XDG paths:

| Component | Path |
|-----------|------|
| Data Home | `~/.local/share/crow/` |
| Config Home | `~/.config/crow/` |
| State Home | `~/.local/state/crow/` |
| Cache Home | `~/.cache/crow/` |
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

## 14. Test Execution Strategy

### Phase 1: Unit Tests (3-4 days)
**Priority Order:**
1. Edit tool (core fuzzy matching)
2. Read/Write tools (file I/O foundation)
3. Bash tool (command execution)
4. Grep/Glob/List tools (search)
5. TodoWrite tool (state management)
6. Snapshot mechanism
7. Session storage

**Test Framework:** `#[cfg(test)]` with tokio::test for async

### Phase 2: Integration Tests (2-3 days)
**Focus Areas:**
1. Tool + Executor pipeline
2. Tool + Session persistence
3. Tool + Snapshot tracking
4. Cross-tool workflows (read → edit → verify)
5. Concurrent operations

### Phase 3: E2E Tests (2-3 days)
**Strategy:**
- Fresh session per test
- Use `crow-cli chat` command
- Verify file system state
- Check XDG paths for persistence
- Parse JSON output mode for assertions

**Test Harness:**
```rust
#[tokio::test]
async fn test_e2e_read_file() {
    let session_id = create_fresh_session();
    let output = run_cli(&["chat", "--json", "--session", &session_id, "read test.txt"]);
    let result: JsonOutput = parse_output(&output);
    assert!(result.tools.iter().any(|t| t.name == "read"));
    cleanup_session(&session_id);
}
```

### Phase 4: Gap Implementation (ongoing)
As tests reveal gaps vs OpenCode, prioritize implementation:

**HIGH Priority:**
1. FileTime tracking (edit safety)
2. Permission system (security)
3. Session compaction (context management)

**MEDIUM Priority:**
4. Image/binary file detection (read tool)
5. LSP diagnostics (edit tool)
6. Event bus integration

**LOW Priority:**
7. Exa AI search backend
8. Session forking
9. Advanced patch format

---

## Appendix: Test File Locations

```
crow-tauri/src-tauri/core/src/tools/
├── read.rs          # 4 existing tests
├── write.rs         # 2 existing tests
├── edit.rs          # 70+ existing tests
├── bash.rs          # 20+ existing tests
├── grep.rs          # 1 existing test
├── glob.rs          # 0 tests
├── list.rs          # 0 tests
├── todowrite.rs     # 1 existing test
├── todoread.rs      # 0 tests
├── webfetch.rs      # 3 existing tests
├── websearch.rs     # 1 existing test
├── task.rs          # 0 tests
├── batch.rs         # 0 tests
├── patch.rs         # 0 tests
└── mod.rs           # Tool registry

crow-tauri/src-tauri/core/src/
├── snapshot/mod.rs  # 10+ existing tests
├── session/store.rs # 0 tests
└── logging.rs       # 0 tests
```

---

*Generated: 2025-11-26*
*Based on analysis of crow-tauri vs opencode implementations*
