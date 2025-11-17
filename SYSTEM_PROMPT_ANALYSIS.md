# System Prompt Analysis: OpenCode vs Crow

## How OpenCode Builds System Prompts

### The 5-Layer Architecture (session/system.ts)

OpenCode constructs system prompts in **5 distinct layers**:

```typescript
// From session/prompt.ts:resolveSystemPrompt()
let system = SystemPrompt.header(input.providerID)           // Layer 1
system.push(...(input.system || input.agent.prompt || 
                 SystemPrompt.provider(input.modelID)))      // Layer 2
system.push(...(await SystemPrompt.environment()))           // Layer 3
system.push(...(await SystemPrompt.custom()))                // Layer 4
// Layer 5 handled by insertReminders() on user messages
```

#### Layer 1: Provider Header
- **Purpose**: Provider-specific branding/identity
- **Examples**:
  - Anthropic: `PROMPT_ANTHROPIC_SPOOF` ("You are Claude...")
  - Others: Empty array `[]`

#### Layer 2: Base Agent Prompt
- **Priority Order**:
  1. Custom system prompt from API call (`input.system`)
  2. Agent's custom prompt (`agent.prompt`)
  3. Provider default (`SystemPrompt.provider(modelID)`)

- **Provider Defaults**:
  - `gpt-5*` → `PROMPT_CODEX`
  - `gpt-*` / `o1` / `o3` → `PROMPT_BEAST`
  - `gemini-*` → `PROMPT_GEMINI`
  - `claude*` → `PROMPT_ANTHROPIC`
  - `polaris-alpha` → `PROMPT_POLARIS`
  - **Everything else** → `PROMPT_ANTHROPIC_WITHOUT_TODO`

#### Layer 3: Environment Context
```typescript
// From session/system.ts:environment()
return [
  `Here is some useful information about the environment you are running in:`,
  `<env>`,
  `  Working directory: ${Instance.directory}`,
  `  Is directory a git repo: ${project.vcs === "git" ? "yes" : "no"}`,
  `  Platform: ${process.platform}`,
  `  Today's date: ${new Date().toDateString()}`,
  `</env>`,
  `<project>`,
  `  ${await Ripgrep.tree({ cwd: Instance.directory, limit: 200 })}`,
  `</project>`,
].join("\n")
```

**Key Point**: Project tree uses `Ripgrep.tree()` with 200 item limit

#### Layer 4: Custom Instructions
- **Search Locations** (in priority order):
  1. Project-local: `AGENTS.md`, `CLAUDE.md`, `CONTEXT.md` (deprecated)
  2. Global: `~/.config/opencode/AGENTS.md`, `~/.claude/CLAUDE.md`
  3. Config-specified: `config.instructions` array (supports globs)

- **Format**: `"Instructions from: <path>\n" + file_content`

#### Layer 5: Dynamic Reminders
- **CRITICAL**: This is NOT in the system prompt!
- **Location**: Injected into **last user message** via `insertReminders()`
- **Examples**:
  ```typescript
  if (input.agent.name === "plan") {
    userMessage.parts.push({
      type: "text",
      text: PROMPT_PLAN,
      synthetic: true,
    })
  }
  ```

### How Todos Work in OpenCode

#### Storage Location
```typescript
// From session/todo.ts
await Storage.write(["todo", input.sessionID], input.todos)
// Saves to: ~/.local/share/opencode/storage/todo/{sessionID}.json
```

#### Tool Availability
Only specific agents have todo tools:
- **supervisor**: `todowrite: true, todoread: true`
- **architect**: `todowrite: true, todoread: true, task: true`

#### Todo Schema
```typescript
{
  content: string,           // "Brief description of the task"
  status: string,            // "pending" | "in_progress" | "completed" | "cancelled"
  priority?: string,         // "high" | "medium" | "low"
  id?: string,               // Unique identifier
  activeForm?: string,       // Present continuous form
}
```

#### How Agents "See" Todos
**They don't automatically!** Agents only access todos by:
1. **Explicitly calling `todoread` tool** to fetch from storage
2. **Writing with `todowrite` tool** which updates storage
3. **No automatic injection** into prompts or context

---

## How Crow Currently Builds System Prompts

### Current Implementation (crow/packages/api/src/agent/prompt.rs)

```rust
pub fn build(&self) -> String {
    let mut prompt = String::new();
    
    // Layer 1: Header (provider-specific)
    prompt.push_str(&self.header());
    
    // Layer 2: Agent prompt or provider default
    prompt.push_str(&self.agent_or_provider_prompt());
    
    // Layer 3: Environment context
    prompt.push_str(&self.environment_context());
    
    // Layer 4: Custom instructions (if any)
    if let Some(instructions) = self.load_custom_instructions() {
        prompt.push_str(&instructions);
    }
    
    // Layer 5: Dynamic reminders
    prompt.push_str(&self.dynamic_reminders());
    
    prompt
}
```

### Current Environment Context
```rust
fn environment_context(&self) -> String {
    let mut context = String::from("# Environment\n\n");
    context.push_str(&format!("Working directory: {}\n", self.working_dir.display()));
    context.push_str(&format!("Platform: {}\n", std::env::consts::OS));
    
    // Git information (if in git repo)
    if let Some(git_info) = self.get_git_info() {
        context.push_str(&git_info);
    }
    
    // Project structure
    context.push_str("\n## Project Structure\n\n");
    context.push_str(&self.generate_project_tree());
    
    context
}
```

### Current Todo Implementation
```rust
// From crow/packages/api/src/tools/todowrite.rs
// Saves to: ~/.local/share/crow/storage/todo/{sessionID}.json
// Exact same location as OpenCode!
```

---

## Critical Gaps: Crow vs OpenCode

### ❌ WRONG: Layer 5 Implementation
**Problem**: Crow adds dynamic reminders to **system prompt**
**OpenCode**: Injects reminders into **user messages** via `insertReminders()`

**Fix Required**:
- Remove `dynamic_reminders()` from system prompt building
- Implement `insertReminders()` mechanism in executor
- Inject synthetic text parts into last user message before LLM call

### ❌ WRONG: Environment Section Format
**Crow**:
```
# Environment

Working directory: /path
Platform: linux

## Project Structure
```

**OpenCode**:
```
Here is some useful information about the environment you are running in:
<env>
  Working directory: /path
  Is directory a git repo: yes
  Platform: linux
  Today's date: Mon Jan 13 2025
</env>
<project>
  [project tree here]
</project>
```

**Fix Required**: Match OpenCode's exact XML-style format with `<env>` and `<project>` tags

### ❌ MISSING: Date in Environment
**Crow**: No date
**OpenCode**: `Today's date: ${new Date().toDateString()}`

**Fix Required**: Add current date to environment context

### ❌ WRONG: Project Tree Implementation
**Crow**: Uses `std::fs::read_dir()` recursion
**OpenCode**: Uses `Ripgrep.tree()` with 200 limit

**Fix Required**: 
- Either implement proper ripgrep-based tree
- Or improve current implementation to match OpenCode's output format

### ❌ MISSING: Custom Instructions Search Patterns
**Crow**: Only checks `AGENTS.md`, `CLAUDE.md` in working_dir
**OpenCode**: 
- Searches upward from working_dir to worktree root
- Checks global config locations
- Supports glob patterns from `config.instructions`

**Fix Required**: Implement `findUp()` pattern searching like OpenCode

### ❌ MISSING: Provider-Specific Prompt Files
**Crow**: Hardcoded strings in `provider_default_prompt()`
**OpenCode**: Loads from text files (`PROMPT_ANTHROPIC`, `PROMPT_BEAST`, etc.)

**Fix Required**: Extract prompts to files, shameless copy from OpenCode

### ❌ MISSING: System Prompt Caching Structure
**OpenCode**:
```typescript
const [first, ...rest] = system
system = [first, rest.join("\n")]  // Max 2 messages for caching
```

**Crow**: Returns single string

**Fix Required**: Return `Vec<String>` with max 2 items for caching optimization

---

## Action Items for Parity

### High Priority
1. ✅ **Todo Storage Location** - Already matches! `~/.local/share/crow/storage/todo/`
2. ❌ **Environment Format** - Must use `<env>` and `<project>` XML tags
3. ❌ **Add Date** - Include `Today's date` in environment
4. ❌ **insertReminders()** - Move dynamic reminders from system prompt to user message injection

### Medium Priority
5. ❌ **Provider Prompt Files** - Extract to .txt files like OpenCode
6. ❌ **Custom Instructions Search** - Implement findUp() and glob patterns
7. ❌ **System Prompt Caching** - Return Vec<String> with 2 items max

### Low Priority
8. ❌ **Project Tree** - Consider ripgrep-based implementation
9. ❌ **Git Status Details** - More comprehensive git info

---

## Testing Strategy

Once fixed, we can verify parity by:

1. **Same Input Test**:
   ```bash
   # Start both servers
   cd ~/project && opencode serve -p 4096 &
   cd ~/project && crow &
   
   # Send identical requests
   curl -X POST http://localhost:4096/sessions
   curl -X POST http://localhost:3000/sessions
   ```

2. **Compare System Prompts**:
   ```bash
   # With CROW_VERBOSE=1, compare logged system prompts
   diff \
     ~/.local/share/opencode/log/session_xyz_prompt.txt \
     ~/.local/share/crow/log/session_abc_prompt.txt
   ```

3. **Compare Todo Storage**:
   ```bash
   # After same task execution
   diff \
     ~/.local/share/opencode/storage/todo/{sessionID}.json \
     ~/.local/share/crow/storage/todo/{sessionID}.json
   ```

---

## Summary

**Current Status**: Crow has the RIGHT architecture but WRONG details

**Critical Insight**: Todos are NOT automatically visible to agents in OpenCode - they must explicitly use todoread/todowrite tools. There's no "todo injection" into prompts except for the tool descriptions themselves.

**Next Step**: Fix environment format, add date, implement insertReminders() for user message injection (not system prompt), then test side-by-side.
