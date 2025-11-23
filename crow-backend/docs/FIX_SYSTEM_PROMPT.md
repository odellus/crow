# Fix System Prompt to Match OpenCode Exactly

## Critical Findings from Analysis

### The Big Mistake: Dynamic Reminders Location
**Current (WRONG)**: We add dynamic reminders to system prompt
**OpenCode (CORRECT)**: Reminders are injected into **user messages** via `insertReminders()`

This is a fundamental architectural difference!

### How OpenCode Actually Works

1. **System Prompt**: Built once with 5 layers, returns `Vec<String>` (max 2 items for caching)
2. **User Messages**: Before sending to LLM, `insertReminders()` adds synthetic text parts to the LAST user message for specific agents

Example from OpenCode:
```typescript
function insertReminders(input: { messages: MessageV2.WithParts[]; agent: Agent.Info }) {
  const userMessage = input.messages.findLast((msg) => msg.info.role === "user")
  if (!userMessage) return input.messages
  
  if (input.agent.name === "plan") {
    userMessage.parts.push({
      id: Identifier.ascending("part"),
      messageID: userMessage.info.id,
      sessionID: userMessage.info.sessionID,
      type: "text",
      text: PROMPT_PLAN,  // Reminder text loaded from file
      synthetic: true,
    })
  }
  
  if (wasPlan && input.agent.name === "build") {
    userMessage.parts.push({
      type: "text",
      text: BUILD_SWITCH,
      synthetic: true,
    })
  }
  
  return input.messages
}
```

This happens in `session/prompt.ts` in the main execution loop!

---

## Implementation Plan

### Step 1: Fix Environment Format
**File**: `crow/packages/api/src/agent/prompt.rs`

**Current**:
```rust
fn environment_context(&self) -> String {
    let mut context = String::from("# Environment\n\n");
    context.push_str(&format!("Working directory: {}\n", self.working_dir.display()));
    // ...
}
```

**Fixed**:
```rust
fn environment_context(&self) -> String {
    let mut env = vec![
        "Here is some useful information about the environment you are running in:",
        "<env>",
    ];
    
    env.push(&format!("  Working directory: {}", self.working_dir.display()));
    
    // Git repo check
    let is_git = std::process::Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .current_dir(&self.working_dir)
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);
    
    env.push(&format!("  Is directory a git repo: {}", if is_git { "yes" } else { "no" }));
    env.push(&format!("  Platform: {}", std::env::consts::OS));
    
    // Add date!
    let date = chrono::Local::now().format("%a %b %d %Y").to_string();
    env.push(&format!("  Today's date: {}", date));
    
    env.push("</env>");
    env.push("<project>");
    env.push(&format!("  {}", self.generate_project_tree()));
    env.push("</project>");
    
    env.join("\n")
}
```

**Dependencies**: Add `chrono = "0.4"` to `Cargo.toml`

### Step 2: Remove Dynamic Reminders from System Prompt
**File**: `crow/packages/api/src/agent/prompt.rs`

**Change**:
```rust
pub fn build(&self) -> String {
    let mut prompt = String::new();
    prompt.push_str(&self.header());
    prompt.push_str("\n\n");
    prompt.push_str(&self.agent_or_provider_prompt());
    prompt.push_str("\n\n");
    prompt.push_str(&self.environment_context());
    prompt.push_str("\n\n");
    if let Some(instructions) = self.load_custom_instructions() {
        prompt.push_str(&instructions);
        prompt.push_str("\n\n");
    }
    // REMOVED: prompt.push_str(&self.dynamic_reminders());
    prompt
}
```

Delete the `dynamic_reminders()` function entirely!

### Step 3: Implement insertReminders() in Executor
**File**: `crow/packages/api/src/agent/executor.rs`

**Add before LLM call**:
```rust
// In execute_turn(), before calling provider.chat()

// Insert reminders into last user message (like OpenCode does)
fn insert_reminders(
    messages: &mut Vec<Message>,
    agent_name: &str,
    session_id: &str,
) {
    // Find last user message
    let last_user_idx = messages.iter().rposition(|m| m.role == Role::User);
    
    if let Some(idx) = last_user_idx {
        let reminder = match agent_name {
            "plan" => Some(include_str!("../prompts/plan_reminder.txt")),
            "build" => {
                // Check if previous message was from plan agent
                let was_plan = messages.iter().any(|m| 
                    m.role == Role::Assistant && 
                    m.content.contains("plan mode") // or track in metadata
                );
                
                if was_plan {
                    Some(include_str!("../prompts/build_switch.txt"))
                } else {
                    None
                }
            }
            _ => None,
        };
        
        if let Some(reminder_text) = reminder {
            // Append to last user message content
            messages[idx].content.push_str("\n\n");
            messages[idx].content.push_str(reminder_text);
            
            tracing::debug!(
                "Inserted {} reminder into last user message for session {}",
                agent_name,
                session_id
            );
        }
    }
}

// Call before LLM:
insert_reminders(&mut messages, &agent.name, session_id);
```

### Step 4: Create Reminder Text Files
**New Files**:
- `crow/packages/api/src/prompts/plan_reminder.txt` - Copy from OpenCode's `session/prompt/plan.txt`
- `crow/packages/api/src/prompts/build_switch.txt` - Copy from OpenCode's `session/prompt/build-switch.txt`

**Directory Structure**:
```
crow/packages/api/src/
├── prompts/
│   ├── plan_reminder.txt
│   ├── build_switch.txt
│   ├── anthropic.txt         (future: copy from OpenCode)
│   ├── beast.txt
│   └── ...
```

### Step 5: Return Vec<String> for Caching
**File**: `crow/packages/api/src/agent/prompt.rs`

**Current**: `pub fn build(&self) -> String`

**Fixed**:
```rust
/// Build system prompt as Vec<String> (max 2 items for caching)
pub fn build(&self) -> Vec<String> {
    let mut layers = Vec::new();
    
    // Layer 1: Header
    let header = self.header();
    
    // Layers 2-4: Everything else
    let mut rest = String::new();
    rest.push_str(&self.agent_or_provider_prompt());
    rest.push_str("\n\n");
    rest.push_str(&self.environment_context());
    rest.push_str("\n\n");
    if let Some(instructions) = self.load_custom_instructions() {
        rest.push_str(&instructions);
    }
    
    // Return max 2 items for caching optimization
    if !header.is_empty() {
        layers.push(header);
        layers.push(rest);
    } else {
        layers.push(rest);
    }
    
    layers
}
```

**Update callers** in `executor.rs`:
```rust
let system_prompt_parts = builder.build();
// Convert to provider format (depends on how you're calling LLM)
```

### Step 6: Extract Provider Prompts to Files
**Create**: `crow/packages/api/src/prompts/`

**Files to create** (shameless copy from OpenCode):
- `anthropic.txt` - From OpenCode's `session/prompt/anthropic.txt`
- `anthropic_spoof.txt` - From OpenCode's `session/prompt/anthropic_spoof.txt`
- `qwen.txt` - From OpenCode's `session/prompt/qwen.txt` (ANTHROPIC_WITHOUT_TODO)
- `beast.txt` - From OpenCode's `session/prompt/beast.txt`
- `gemini.txt` - From OpenCode's `session/prompt/gemini.txt`
- `polaris.txt` - From OpenCode's `session/prompt/polaris.txt`
- `codex.txt` - From OpenCode's `session/prompt/codex.txt`

**Update `provider_default_prompt()`**:
```rust
fn provider_default_prompt(&self) -> String {
    match self.provider_id.as_str() {
        id if id.contains("gpt-5") => include_str!("../prompts/codex.txt").to_string(),
        id if id.contains("gpt-") || id.contains("o1") || id.contains("o3") => 
            include_str!("../prompts/beast.txt").to_string(),
        id if id.contains("gemini-") => 
            include_str!("../prompts/gemini.txt").to_string(),
        id if id.contains("claude") => 
            include_str!("../prompts/anthropic.txt").to_string(),
        id if id.contains("polaris-alpha") => 
            include_str!("../prompts/polaris.txt").to_string(),
        _ => include_str!("../prompts/qwen.txt").to_string(),
    }
}

fn header(&self) -> String {
    match self.provider_id.as_str() {
        id if id.contains("anthropic") => 
            include_str!("../prompts/anthropic_spoof.txt").trim().to_string(),
        _ => String::new(),
    }
}
```

### Step 7: Implement Custom Instructions Search (findUp)
**File**: `crow/packages/api/src/agent/prompt.rs`

**Current**: Only checks working_dir
**Fixed**: Search upward to worktree root

```rust
fn load_custom_instructions(&self) -> Option<String> {
    let local_files = vec!["AGENTS.md", "CLAUDE.md", "CONTEXT.md"];
    let global_files = vec![
        // XDG config
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .or_else(|| std::env::var("HOME").ok().map(|h| format!("{}/.config", h)))
            .map(|d| format!("{}/crow/AGENTS.md", d)),
        // Legacy ~/.claude
        std::env::var("HOME").ok().map(|h| format!("{}/.claude/CLAUDE.md", h)),
    ];
    
    let mut instructions = Vec::new();
    
    // Search local files (upward from working_dir to git root)
    if let Some(root) = self.find_git_root() {
        for filename in local_files {
            if let Some(path) = self.find_up(filename, &self.working_dir, &root) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    instructions.push(format!("Instructions from: {}\n{}", path.display(), content));
                    break; // Only first match per file type
                }
            }
        }
    }
    
    // Search global files
    for maybe_path in global_files.into_iter().flatten() {
        let path = PathBuf::from(maybe_path);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                instructions.push(format!("Instructions from: {}\n{}", path.display(), content));
                break; // Only first global match
            }
        }
    }
    
    if instructions.is_empty() {
        None
    } else {
        Some(instructions.join("\n\n"))
    }
}

fn find_git_root(&self) -> Option<PathBuf> {
    let output = std::process::Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .current_dir(&self.working_dir)
        .output()
        .ok()?;
    
    if output.status.success() {
        Some(PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()))
    } else {
        None
    }
}

fn find_up(&self, filename: &str, start: &Path, root: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    
    loop {
        let candidate = current.join(filename);
        if candidate.exists() {
            return Some(candidate);
        }
        
        if current == root {
            break;
        }
        
        if !current.pop() {
            break;
        }
    }
    
    None
}
```

---

## Testing Checklist

After implementing all changes:

- [ ] System prompt has `<env>` and `<project>` tags
- [ ] Environment includes "Today's date"
- [ ] Git repo detection works
- [ ] No dynamic reminders in system prompt
- [ ] Reminders injected into user messages for plan/build agents
- [ ] System prompt returns Vec<String> with max 2 items
- [ ] Provider prompts loaded from .txt files
- [ ] Custom instructions search works (findUp pattern)
- [ ] Verbose mode logs complete system prompt
- [ ] Side-by-side test with OpenCode produces similar system prompts

---

## Files to Copy from OpenCode

```bash
# In opencode-project directory:
cp opencode/packages/opencode/src/session/prompt/*.txt \
   crow/packages/api/src/prompts/

# Should get:
# - anthropic.txt
# - anthropic_spoof.txt
# - qwen.txt
# - beast.txt
# - gemini.txt
# - polaris.txt
# - codex.txt
# - plan.txt → plan_reminder.txt
# - build-switch.txt → build_switch.txt
# - summarize.txt
# - title.txt
```

---

## Estimated Effort

- Step 1 (Environment format): 30 min
- Step 2 (Remove dynamic reminders): 5 min
- Step 3 (insertReminders): 1 hour
- Step 4 (Reminder files): 10 min
- Step 5 (Vec<String> return): 30 min
- Step 6 (Extract prompts): 1 hour
- Step 7 (findUp search): 1 hour

**Total**: ~4.5 hours to achieve system prompt parity
