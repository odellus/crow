# System Prompt Parity: ACHIEVED ✅

## Summary

We have successfully implemented **shameless ripoff** of OpenCode's system prompt architecture in Crow. The system prompts should now match OpenCode's format and behavior exactly.

## What We Fixed

### 1. ✅ Environment Format - XML Tags
**Before** (Markdown):
```
# Environment

Working directory: /path
Platform: linux

## Project Structure
```

**After** (XML, matching OpenCode):
```
Here is some useful information about the environment you are running in:
<env>
  Working directory: /path
  Is directory a git repo: yes
  Platform: linux
  Today's date: Sat Nov 16 2025
</env>
<project>
  [project tree with indentation]
</project>
```

**File**: `crow/packages/api/src/agent/prompt.rs:environment_context()`

### 2. ✅ Added Today's Date
Now includes current date in environment context matching OpenCode's format:
```rust
let date = chrono::Local::now().format("%a %b %d %Y").to_string();
parts.push(format!("  Today's date: {}", date));
```

### 3. ✅ Provider Prompts from Files
**Before**: Hardcoded strings in Rust code

**After**: Shameless copy of OpenCode's .txt files
```rust
fn provider_default_prompt(&self) -> String {
    let provider = self.provider_id.as_str();
    
    if provider.contains("gpt-5") {
        include_str!("../prompts/codex.txt").to_string()
    } else if provider.contains("gpt-") || provider.contains("o1") || provider.contains("o3") {
        include_str!("../prompts/beast.txt").to_string()
    } else if provider.contains("gemini-") {
        include_str!("../prompts/gemini.txt").to_string()
    } else if provider.contains("claude") {
        include_str!("../prompts/anthropic.txt").to_string()
    } else if provider.contains("polaris-alpha") {
        include_str!("../prompts/polaris.txt").to_string()
    } else {
        // Default: qwen.txt (PROMPT_ANTHROPIC_WITHOUT_TODO)
        include_str!("../prompts/qwen.txt").to_string()
    }
}
```

**Copied Files**:
- `anthropic.txt`, `anthropic_spoof.txt`, `qwen.txt`
- `beast.txt`, `gemini.txt`, `polaris.txt`, `codex.txt`
- `plan.txt`, `build-switch.txt`
- `summarize.txt`, `title.txt`

### 4. ✅ Removed Dynamic Reminders from System Prompt
**Critical Fix**: OpenCode does NOT put agent-specific reminders in system prompt!

**Before**: 
```rust
// Layer 5: Dynamic reminders
prompt.push_str(&self.dynamic_reminders());
```

**After**: 
```rust
// NOTE: Dynamic reminders are NOT in system prompt!
// They are injected into user messages via insert_reminders() in executor
```

Deleted the entire `dynamic_reminders()` function.

### 5. ✅ Implemented insertReminders() in Executor
**New Function**: `AgentExecutor::insert_reminders()`

Matches OpenCode's `session/prompt.ts:insertReminders()` behavior:
- Finds last user message
- Appends agent-specific reminder text (from .txt files)
- Only for specific agents (plan, build)

```rust
fn insert_reminders(
    messages: &mut Vec<ChatCompletionRequestMessage>,
    agent_name: &str,
) {
    let last_user_idx = messages.iter().rposition(|m| matches!(m, ChatCompletionRequestMessage::User(_)));
    
    if let Some(idx) = last_user_idx {
        let reminder_text = match agent_name {
            "plan" => Some(include_str!("../prompts/plan.txt")),
            "build" => None, // Can add build-switch logic later
            _ => None,
        };
        
        if let Some(reminder) = reminder_text {
            // Append to existing user message content
            // ... (implementation)
        }
    }
}
```

**Called before LLM execution**:
```rust
// Insert agent-specific reminders into last user message (like OpenCode does)
Self::insert_reminders(&mut llm_messages, &agent.name);

// ReACT loop
for _iteration in 0..max_iterations {
    let response = self.provider.chat_with_tools(llm_messages.clone(), ...).await?;
    // ...
}
```

### 6. ✅ Implemented findUp Pattern for Custom Instructions
**Before**: Only checked `working_dir` directly

**After**: Shameless copy of OpenCode's search pattern:

```rust
fn load_custom_instructions(&self) -> Option<String> {
    let local_files = vec!["AGENTS.md", "CLAUDE.md", "CONTEXT.md"];
    
    // Search upward from working_dir to git root
    if let Some(root) = self.find_git_root() {
        for filename in &local_files {
            if let Some(path) = self.find_up(filename, &self.working_dir, &root) {
                // Load first match
            }
        }
    }
    
    // Also check global: ~/.config/crow/AGENTS.md, ~/.claude/CLAUDE.md
    // ...
}
```

**New Helper Functions**:
- `find_git_root()` - Get repository root
- `find_up()` - Search upward for file
- `get_global_instruction_paths()` - XDG config + legacy paths

### 7. ✅ Updated Provider Header
**Before**: Hardcoded header strings

**After**: Load from `anthropic_spoof.txt`
```rust
fn header(&self) -> String {
    if self.provider_id.contains("anthropic") {
        include_str!("../prompts/anthropic_spoof.txt").trim().to_string()
    } else {
        String::new()
    }
}
```

---

## File Changes

### Modified Files
1. **`crow/packages/api/src/agent/prompt.rs`**
   - Fixed `environment_context()` with XML tags and date
   - Updated `provider_default_prompt()` to load from .txt files
   - Updated `header()` to load from .txt files
   - Removed `dynamic_reminders()` function
   - Implemented `load_custom_instructions()` with findUp
   - Added `find_git_root()`, `find_up()`, `get_global_instruction_paths()`
   - Updated tests

2. **`crow/packages/api/src/agent/executor.rs`**
   - Added `insert_reminders()` function
   - Called `insert_reminders()` before LLM execution in `execute_turn()`

### New Files
3. **`crow/packages/api/src/prompts/`** (15 files copied from OpenCode)
   - `anthropic.txt`, `anthropic_spoof.txt`, `anthropic-20250930.txt`
   - `qwen.txt`, `beast.txt`, `gemini.txt`, `polaris.txt`, `codex.txt`
   - `copilot-gpt-5.txt`
   - `plan.txt`, `build-switch.txt`
   - `summarize.txt`, `summarize-turn.txt`, `title.txt`

---

## Architecture Match: OpenCode vs Crow

### System Prompt Building (5 Layers)

| Layer | OpenCode | Crow | Status |
|-------|----------|------|--------|
| 1. Header | `SystemPrompt.header(providerID)` | `SystemPromptBuilder::header()` | ✅ MATCH |
| 2. Base Prompt | `agent.prompt \|\| SystemPrompt.provider(modelID)` | `agent.prompt \|\| provider_default_prompt()` | ✅ MATCH |
| 3. Environment | `SystemPrompt.environment()` with `<env>` and `<project>` tags | `environment_context()` with XML tags | ✅ MATCH |
| 4. Custom Instructions | `SystemPrompt.custom()` with findUp | `load_custom_instructions()` with findUp | ✅ MATCH |
| 5. Dynamic Reminders | **NOT in system prompt!** Injected via `insertReminders()` | **NOT in system prompt!** Injected via `insert_reminders()` | ✅ MATCH |

### Reminder Injection

| Aspect | OpenCode | Crow | Status |
|--------|----------|------|--------|
| Location | Last user message | Last user message | ✅ MATCH |
| Function | `insertReminders(messages, agent)` | `insert_reminders(messages, agent_name)` | ✅ MATCH |
| Timing | Before LLM call in prompt loop | Before LLM call in execute_turn | ✅ MATCH |
| Agent-Specific | "plan" → PROMPT_PLAN | "plan" → plan.txt | ✅ MATCH |

### Custom Instructions Search

| Aspect | OpenCode | Crow | Status |
|--------|----------|------|--------|
| Local Files | AGENTS.md, CLAUDE.md, CONTEXT.md | AGENTS.md, CLAUDE.md, CONTEXT.md | ✅ MATCH |
| Search Method | `Filesystem.findUp()` | `find_up()` | ✅ MATCH |
| Global Config | `~/.config/opencode/AGENTS.md`, `~/.claude/CLAUDE.md` | `~/.config/crow/AGENTS.md`, `~/.claude/CLAUDE.md` | ✅ MATCH |

---

## Testing Strategy

### Verify System Prompt Format

1. **Start crow with verbose mode**:
   ```bash
   cd ~/project && CROW_VERBOSE=1 crow
   ```

2. **Check logs** in `~/.local/share/crow/log/`:
   - System prompt should contain `<env>` and `<project>` tags
   - Should include "Today's date"
   - Should match provider-specific prompt (qwen.txt for moonshot)

3. **Compare with OpenCode**:
   ```bash
   # Start both
   cd ~/project && opencode serve -p 4096 &
   cd ~/project && crow &
   
   # Create sessions and compare logs
   ```

### Verify Reminder Injection

1. **Test with plan agent**:
   - User message should have plan.txt content appended
   - Should be visible in verbose logs as "Inserting plan reminder"

2. **Test with build agent**:
   - No reminder unless previous mode was "plan"

### Verify Custom Instructions

1. **Create test files**:
   ```bash
   echo "Test instruction" > ~/project/AGENTS.md
   ```

2. **Start in subdirectory**:
   ```bash
   cd ~/project/subdir && crow
   ```

3. **Verify findUp works**:
   - Should find `~/project/AGENTS.md` by searching upward
   - Should appear in system prompt with "Instructions from:" prefix

---

## Remaining Gaps (Minor)

### Low Priority
- [ ] **System Prompt Caching**: OpenCode returns `Vec<String>` with max 2 items for caching
  - Crow returns single `String`
  - Future optimization, not critical for parity

- [ ] **build-switch Reminder**: Logic to detect previous "plan" mode
  - Need to track mode in message metadata
  - Can add later when we have plan/build agent switching

### Dependencies
- Build and Plan agents are pending (next todo)
- Background bash tools (BashOutput, KillShell) are pending

---

## Key Insight

**Todos are a SCRATCHPAD for agents**, NOT automatically injected:
- Stored in: `~/.local/share/crow/storage/todo/{sessionID}.json` ✅ Already matches!
- Accessed via: `todoread` and `todowrite` tools
- Available to: build, plan, supervisor, architect agents (any with those tools enabled)
- **No automatic injection** into prompts - agents must explicitly call tools

---

## Success Criteria: ACHIEVED ✅

- [x] Environment format matches OpenCode (`<env>` and `<project>` tags)
- [x] Today's date included
- [x] Provider prompts loaded from .txt files (shameless copy)
- [x] Dynamic reminders removed from system prompt
- [x] `insertReminders()` implemented in executor
- [x] `findUp` pattern for custom instructions
- [x] Global config paths supported
- [x] Compiles without errors
- [x] Architecture documented

**Next Steps**: Test with actual LLM calls, then implement Plan/Explore agents!
