# Dual Agent Architecture Plan

## Overview

The dual agent system consists of two full agents - **Executor** and **Arbiter** - that work in a loop until the Arbiter verifies the task is complete. This is not a lightweight review system. Both agents have full ReACT loops with tool access. The Arbiter has **more** capabilities (vision, compression, termination control) because it must verify work in the real world.

## The Key Insight

Traditional single-agent flow:
```
User → Agent → Tools → Response → User
```

The problem: the agent self-evaluates. It decides when it's done. There's no verification that the work actually functions correctly in the real world.

Dual agent flow:
```
User → Executor → Tools → [rendered to markdown] → Arbiter → Tools → Decision
                                                                        ↓
                                                              task_complete? 
                                                                 ↓      ↓
                                                               YES      NO
                                                                 ↓      ↓
                                                              Done    [arbiter work rendered to markdown]
                                                                        ↓
                                                                   Back to Executor
```

The Arbiter is a **QA engineer with eyes**. It can run the server, visit the URL, take screenshots, interact with the UI, and verify the work actually functions. Only the Arbiter can terminate the loop.

## Agent Capabilities

### Executor Agent
- Full ReACT loop
- All standard tools: bash, read, write, edit, glob, grep, list, webfetch, websearch
- No termination authority
- Receives: user request (step 0) OR arbiter's full session rendered to markdown (step 1+)
- Outputs: thinking + tool calls + responses (entire session)

### Arbiter Agent  
- Full ReACT loop
- All standard tools (same as executor)
- PLUS vision tools: screenshot, browser_navigate, browser_click, browser_type, desktop_capture
- PLUS `compact` tool: compress conversation history
- PLUS `task_complete` tool: **REQUIRED** - the only way to terminate the loop
- Receives: executor's full session rendered to markdown
- Outputs: thinking + tool calls + responses (entire session) OR task_complete

## The Loop in Detail

```
┌─────────────────────────────────────────────────────────────────┐
│                      DUAL AGENT LOOP                            │
│                                                                 │
│  Step 0:                                                        │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ EXECUTOR receives user request                            │  │
│  │ [Full ReACT loop - thinking, tool calls, responses]       │  │
│  │ Executor stops when it believes work is complete          │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│              [Render ENTIRE executor session to markdown]       │
│                              │                                  │
│                              ▼                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ ARBITER receives executor's rendered session              │  │
│  │ [Full ReACT loop - thinking, tool calls, responses]       │  │
│  │ Arbiter VERIFIES: runs tests, starts servers, visits      │  │
│  │ URLs, takes screenshots, interacts with the app           │  │
│  │                                                           │  │
│  │ Decision point:                                           │  │
│  │   → task_complete called? EXIT LOOP, return to user       │  │
│  │   → Otherwise: continue to step 1                         │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│              [Render ENTIRE arbiter session to markdown]        │
│                              │                                  │
│                              ▼                                  │
│  Step 1+:                                                       │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ EXECUTOR receives arbiter's ENTIRE rendered session       │  │
│  │ (NOT just "feedback" - the full ReACT loop with all       │  │
│  │ tool calls, including image descriptions for screenshots) │  │
│  │                                                           │  │
│  │ [Full ReACT loop]                                         │  │
│  │ Executor sees what arbiter tried, what failed, why        │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│              [Render ENTIRE executor session to markdown]       │
│                              │                                  │
│                              ▼                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ ARBITER receives executor's rendered session              │  │
│  │ [Full ReACT loop - verify again]                          │  │
│  │                                                           │  │
│  │   → task_complete? EXIT LOOP                              │  │
│  │   → step < max_steps? Continue to step 2                  │  │
│  │   → step >= max_steps? EXIT LOOP (best effort)            │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│              [Render ENTIRE arbiter session to markdown]        │
│                              │                                  │
│                              ▼                                  │
│                             ...                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Message Rendering

The glue between agents is **markdown rendering of the entire session**. This is one-way (session → markdown) and that's fine. We don't need to reconstruct tool calls from markdown.

### Executor → Arbiter

Executor's entire session rendered:

```markdown
## Executor Session (Step 1)

### Thinking
I need to implement the user authentication feature. Let me start by examining the current code...

### Tool: read
**Input:**
```json
{"file_path": "/src/auth.rs"}
```

**Output:**
```rust
pub fn authenticate(token: &str) -> Result<User, AuthError> {
    // TODO: implement JWT validation
    unimplemented!()
}
```

### Thinking  
I see the auth function is stubbed out. I'll implement JWT validation using the jsonwebtoken crate...

### Tool: edit
**Input:**
```json
{
  "file_path": "/src/auth.rs",
  "old_string": "// TODO: implement JWT validation\n    unimplemented!()",
  "new_string": "let claims = decode::<Claims>(token, &DECODING_KEY, &Validation::default())\n        .map_err(|_| AuthError::InvalidToken)?;\n    Ok(User { id: claims.sub })"
}
```

**Output:**
```
✓ Successfully edited /src/auth.rs
```

### Tool: bash
**Input:**
```json
{"command": "cargo test auth"}
```

**Output:**
```
running 3 tests
test auth::tests::test_valid_token ... ok
test auth::tests::test_expired_token ... ok  
test auth::tests::test_invalid_token ... ok

test result: ok. 3 passed; 0 failed
```

### Response
I've implemented JWT validation in the auth module. The function now:
1. Decodes the JWT token using the configured secret
2. Extracts the user ID from claims
3. Returns appropriate errors for invalid/expired tokens

All auth tests pass.


### Arbiter → Executor

Arbiter's **entire session** rendered (not just feedback - the full ReACT loop):

```markdown
## Arbiter Session (Step 1)

### Thinking
The executor implemented JWT auth and tests pass. But I need to verify this actually works in the running application. Let me start the server and test the login flow...

### Tool: bash
**Input:**
```json
{"command": "cargo run &"}
```

**Output:**
```
[1] 12345
Server starting on http://localhost:3000
```

### Tool: bash  
**Input:**
```json
{"command": "sleep 2"}
```

**Output:**
```
```

### Tool: browser_navigate
**Input:**
```json
{"url": "http://localhost:3000/login"}
```

**Output:**
```
Page loaded: "Login - MyApp"
```

### Tool: screenshot
**Input:**
```json
{"target": "#login-form"}
```

**Output:**
```
Screenshot saved: /tmp/artifacts/pair_abc123/screenshot_001.png
```

**[Image Description]:**
The login form is displayed with email and password fields. However, there is a critical issue: the "Remember Me" checkbox is missing from the form, and the "Forgot Password" link renders as raw HTML (`<a href="/forgot">`) instead of being properly displayed. The submit button appears functional but is positioned incorrectly, overlapping with the password field border.

### Thinking
There are frontend issues. Let me also test the actual authentication API...

### Tool: bash
**Input:**
```json
{"command": "curl -X POST http://localhost:3000/api/auth/login -H 'Content-Type: application/json' -d '{\"email\":\"test@example.com\",\"password\":\"testpass\"}'"}
```

**Output:**
```json
{"error": "Invalid credentials", "code": "AUTH_001"}
```

### Thinking
The API rejects valid test credentials. Let me check if the test user exists in the database...

### Tool: bash
**Input:**
```json
{"command": "sqlite3 data/app.db \"SELECT * FROM users WHERE email='test@example.com'\""}
```

**Output:**
```
1|test@example.com|$2b$10$abcdef...|2024-01-15
```

### Thinking
User exists. The issue might be in password comparison. Let me check the auth handler...

### Tool: read
**Input:**
```json
{"file_path": "/src/handlers/auth.rs"}
```

**Output:**
```rust
pub async fn login(body: LoginRequest) -> Result<Json<AuthResponse>, ApiError> {
    let user = db::find_user_by_email(&body.email).await?;
    
    // BUG: comparing plain text to hash!
    if body.password != user.password_hash {
        return Err(ApiError::InvalidCredentials);
    }
    
    let token = create_jwt(&user)?;
    Ok(Json(AuthResponse { token }))
}
```

### Response
Verification found multiple issues:

**Critical - Authentication Broken:**
1. Password comparison bug in `/src/handlers/auth.rs`: comparing plaintext password directly to bcrypt hash instead of using `bcrypt::verify()`

**Frontend Issues:**
2. "Remember Me" checkbox missing from login form
3. "Forgot Password" link rendering as raw HTML
4. Submit button overlapping password field

The executor's JWT validation implementation is correct, but the login handler that uses it has a critical bug. The backend auth will never succeed because password comparison is wrong.

---

The executor receives this **entire session** - all the thinking, all the tool calls, all the outputs, including the image descriptions. The executor can see exactly what the arbiter tried, what it discovered, and why the verification failed.

## Session Linking

Sessions are linked via metadata fields, **not** ID parsing:

```rust
// Executor session
Session {
    id: "ses_7a8b9c0d1e2f",  // Normal ID format
    parent_id: Some("ses_user_original"),
    title: "Dual: Executor (Step 1)",
    metadata: Some(json!({
        "dual_agent": {
            "role": "executor",
            "pair_id": "pair_x1y2z3",      // Shared identifier for this pair
            "sibling_id": "ses_3f4e5d6c",  // Arbiter's session ID
            "step": 1,
            "complete": false
        }
    })),
    // ...
}

// Arbiter session
Session {
    id: "ses_3f4e5d6c",  // Normal ID format
    parent_id: Some("ses_user_original"),
    title: "Dual: Arbiter (Step 1)",
    metadata: Some(json!({
        "dual_agent": {
            "role": "arbiter",
            "pair_id": "pair_x1y2z3",      // Same pair_id
            "sibling_id": "ses_7a8b9c0d1e2f",  // Executor's session ID
            "step": 1,
            "complete": false
        }
    })),
    // ...
}
```

Benefits of metadata-based linking:
- Session IDs remain opaque identifiers (no parsing required)
- Relationship is explicit and queryable
- Easy to add fields later (step count, completion status, timing)
- Works with existing storage layer unchanged

Query patterns:
```bash
# Find all sessions in a dual pair
crow-cli sessions --filter 'metadata.dual_agent.pair_id == "pair_x1y2z3"'

# Find arbiter session for an executor
crow-cli session info ses_7a8b9c0d1e2f  # Shows sibling_id in metadata

# List all dual agent sessions
crow-cli sessions --filter 'metadata.dual_agent != null'
```

## Configuration

**Agents are configured outside source code.** This is a hard requirement.

### Agent Configs

`.crow/agents/executor.md`:
```yaml
---
name: executor
description: Implementation agent - writes code, runs commands, does the work
mode: all
model: anthropic/claude-sonnet-4-20250514
temperature: 0.7
tools:
  bash: true
  read: true
  write: true
  edit: true
  glob: true
  grep: true
  list: true
  webfetch: true
  websearch: true
  task: true
---

You are the Executor agent in a dual-agent system.

Your job is to implement the requested changes. Write code, edit files, run commands. Do the work.

You will receive either:
- A user request (first iteration)
- The Arbiter's full session showing what it tried and what failed (subsequent iterations)

When you receive the Arbiter's session, pay careful attention to:
- What verification steps it performed
- What issues it discovered
- Any image descriptions (the Arbiter can see screenshots, you cannot)

Work until you believe the task is complete, then stop. The Arbiter will verify your work.
```

`.crow/agents/arbiter.md`:
```yaml
---
name: arbiter
description: Verification agent - tests, screenshots, visual inspection, final approval
mode: all  
model: anthropic/claude-sonnet-4-20250514
vision: true
temperature: 0.3
tools:
  # Standard tools (same as executor)
  bash: true
  read: true
  write: true
  edit: true
  glob: true
  grep: true
  list: true
  webfetch: true
  websearch: true
  # Vision tools
  screenshot: true
  browser_navigate: true
  browser_click: true
  browser_type: true
  browser_screenshot: true
  desktop_capture: true
  # Control tools
  compact: true
  task_complete: true  # REQUIRED
---

You are the Arbiter agent in a dual-agent system.

Your job is to VERIFY that the Executor's work actually functions correctly. Do not just read code - RUN it. Do not trust test output - SEE the result.

You have capabilities the Executor lacks:
- Vision: You can take screenshots and see what users see
- Browser control: You can navigate, click, type, interact
- Desktop capture: You can see running applications

Verification process:
1. Read what the Executor did
2. Run the code / start the server / build the app
3. Actually use it - visit URLs, click buttons, fill forms
4. Take screenshots of what you see
5. Compare actual behavior to expected behavior

When you find issues:
- Be specific about what you observed
- Describe screenshots in detail (the Executor cannot see images)
- Explain what should have happened vs what did happen

Call `task_complete` ONLY when you have verified:
- The implementation meets requirements
- The code runs without errors
- The user-facing behavior is correct
- Edge cases are handled

If verification fails, your full session (including all tool calls and image descriptions) will be sent to the Executor for the next iteration.
```

### Dual Agent Config

`crow.json` or `.crow/config.json`:
```json
{
  "dual_agent": {
    "enabled": true,
    "executor_agent": "executor",
    "arbiter_agent": "arbiter",
    "max_steps": 10,
    "require_task_complete": true
  }
}
```

### Validation

At startup, validate arbiter config:

```rust
fn validate_dual_agent_config(config: &DualAgentConfig, agents: &AgentRegistry) -> Result<(), ConfigError> {
    let arbiter = agents.get(&config.arbiter_agent)
        .ok_or(ConfigError::AgentNotFound(config.arbiter_agent.clone()))?;
    
    // task_complete is REQUIRED for arbiter
    if arbiter.tools.get("task_complete") != Some(&true) {
        return Err(ConfigError::MissingRequiredTool {
            agent: config.arbiter_agent.clone(),
            tool: "task_complete".to_string(),
            reason: "The arbiter agent MUST have task_complete enabled. \
                     This is the only way to terminate the dual agent loop.".to_string(),
        });
    }
    
    // Vision model validation (if vision tools enabled)
    if arbiter.tools.get("screenshot") == Some(&true) 
        || arbiter.tools.get("browser_screenshot") == Some(&true)
        || arbiter.tools.get("desktop_capture") == Some(&true) 
    {
        if !arbiter.vision.unwrap_or(false) {
            return Err(ConfigError::VisionRequired {
                agent: config.arbiter_agent.clone(),
                reason: "Arbiter has vision tools but vision: true not set. \
                         The model must support vision to use screenshot tools.".to_string(),
            });
        }
    }
    
    Ok(())
}
```

## Special Tools

### task_complete

The only way to terminate the dual agent loop.

```rust
pub struct TaskCompleteTool;

#[derive(Deserialize)]
struct TaskCompleteInput {
    /// Summary of what was accomplished
    summary: String,
    /// How the work was verified (what you tested, what you saw)
    verification: String,
    /// List of files created or modified
    artifacts: Option<Vec<String>>,
}

impl Tool for TaskCompleteTool {
    fn name(&self) -> &str { "task_complete" }
    
    fn description(&self) -> &str {
        "Signal that the task is complete and verified. This terminates the dual agent loop \
         and returns control to the user. Only call this when you have VERIFIED the work \
         meets all requirements through actual testing and observation."
    }
    
    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let input: TaskCompleteInput = serde_json::from_value(input)?;
        
        // Signal completion via return value - the DualAgentRuntime checks for this
        ToolResult {
            output: format!(
                "Task completed.\n\nSummary:\n{}\n\nVerification:\n{}\n\nArtifacts:\n{}",
                input.summary,
                input.verification,
                input.artifacts.unwrap_or_default().join("\n")
            ),
            metadata: json!({
                "task_complete": true,
                "summary": input.summary,
                "verification": input.verification,
                "artifacts": input.artifacts,
            }),
            status: ToolStatus::Completed,
        }
    }
}
```

### compact

Compress conversation history to manage context size.

```rust
pub struct CompactTool;

#[derive(Deserialize)]
struct CompactInput {
    /// Message IDs to preserve verbatim (don't summarize these)
    preserve: Option<Vec<String>>,
    /// Custom instructions for summarization
    instructions: Option<String>,
}

impl Tool for CompactTool {
    fn name(&self) -> &str { "compact" }
    
    fn description(&self) -> &str {
        "Compress conversation history to reduce context size while preserving important \
         information. Use when context is getting large. Optionally specify message IDs \
         to preserve verbatim."
    }
    
    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let input: CompactInput = serde_json::from_value(input)?;
        
        // Get current session messages
        let messages = ctx.session_store.get_messages(&ctx.session_id)?;
        
        // Identify messages to summarize vs preserve
        let preserve_ids: HashSet<_> = input.preserve.unwrap_or_default().into_iter().collect();
        
        let to_summarize: Vec<_> = messages.iter()
            .filter(|m| !preserve_ids.contains(&m.info.id()))
            .collect();
        
        // Use small model to generate summary
        let summary = summarize_messages(&to_summarize, input.instructions).await?;
        
        // Mark compacted messages and add summary
        // (Implementation details - update message metadata, add CompactionPart)
        
        ToolResult {
            output: format!("Compacted {} messages into summary.", to_summarize.len()),
            metadata: json!({
                "compacted_count": to_summarize.len(),
                "preserved_count": preserve_ids.len(),
            }),
            status: ToolStatus::Completed,
        }
    }
}
```

### Vision Tools

These integrate with browser automation (playwright-like) and desktop capture.

```rust
pub struct ScreenshotTool {
    browser: Arc<BrowserContext>,
}

#[derive(Deserialize)]
struct ScreenshotInput {
    /// CSS selector, window name, or "fullscreen"
    target: Option<String>,
}

impl Tool for ScreenshotTool {
    fn name(&self) -> &str { "screenshot" }
    
    fn description(&self) -> &str {
        "Take a screenshot. Optionally specify a CSS selector to capture a specific element, \
         a window name, or 'fullscreen' for the entire screen. Returns the image for visual \
         inspection."
    }
    
    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let input: ScreenshotInput = serde_json::from_value(input)?;
        
        let screenshot_path = ctx.artifacts_dir.join(format!(
            "screenshot_{}.png",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        ));
        
        let screenshot = match input.target.as_deref() {
            Some("fullscreen") | None => self.browser.screenshot_fullscreen().await?,
            Some(selector) if selector.starts_with('#') || selector.starts_with('.') => {
                self.browser.screenshot_element(selector).await?
            }
            Some(window) => capture_window(window).await?,
        };
        
        screenshot.save(&screenshot_path)?;
        
        ToolResult {
            output: format!("Screenshot saved: {}", screenshot_path.display()),
            // The image is returned as an attachment for the vision model
            attachments: vec![Attachment::Image(screenshot_path)],
            status: ToolStatus::Completed,
        }
    }
}
```

## Implementation Plan

### Phase 1: Core Infrastructure

1. **Add dual_agent metadata schema** to Session
   - Define `DualAgentMetadata` struct
   - Add to session creation flow
   - Update session queries to filter by metadata

2. **Implement `task_complete` tool**
   - Tool definition with validation
   - Return value signals completion to runtime

3. **Implement `compact` tool**
   - Message summarization logic
   - Compaction markers in session history

4. **Create `DualAgentRuntime`**
   - Orchestrates executor/arbiter loop
   - Handles session creation and linking
   - Manages step counting and max_steps termination
   - Renders sessions to markdown between agents

### Phase 2: Session Rendering

5. **Enhance markdown export for dual agent use**
   - Ensure all tool calls are rendered with inputs/outputs
   - Add thinking sections
   - Add image description placeholders

6. **Image description injection**
   - When arbiter takes screenshot, description stored in tool output
   - When rendering arbiter session, include `[Image Description]` blocks
   - Executor sees rich text description of visual content

### Phase 3: Vision Tools

7. **Browser automation integration**
   - Playwright-rs or chromiumoxide for browser control
   - browser_navigate, browser_click, browser_type, browser_screenshot

8. **Desktop capture**
   - Platform-specific screen capture
   - Window enumeration and targeting

9. **Screenshot tool**
   - Capture to artifacts directory
   - Return as attachment for vision model

### Phase 4: Configuration & Validation

10. **Agent config validation**
    - Require task_complete for arbiter
    - Validate vision model for vision tools
    - Error messages guide users to fix config

11. **Dual agent config in crow.json**
    - enabled, executor_agent, arbiter_agent, max_steps
    - require_task_complete flag

12. **CLI commands**
    - `crow-cli dual status` - show running dual sessions
    - `crow-cli sessions --dual` - filter to dual agent sessions
    - Session info shows sibling linkage

### Phase 5: Polish

13. **Observability**
    - Both sessions fully logged
    - Step transitions logged
    - Completion/timeout clearly marked

14. **Artifacts management**
    - Screenshots stored in `.crow/artifacts/{pair_id}/`
    - Cleanup policy for old artifacts

15. **Documentation**
    - Agent config examples
    - Workflow diagrams
    - Troubleshooting guide

## Open Questions

1. **Image storage format**: Base64 in markdown vs file path reference?
   - File paths are cleaner but require artifacts to persist
   - Base64 is self-contained but bloats markdown

2. **Arbiter model requirement**: Should we validate the model supports vision?
   - Could check model ID against known vision models
   - Or trust the user's config

3. **Compact trigger**: Auto-compact at context threshold or manual only?
   - Auto-compact risks losing important context
   - Manual-only risks context overflow

4. **Step persistence**: New sessions per step or append to existing?
   - Current design: same session, messages appended
   - Alternative: new session per step, linked by step number

5. **Timeout handling**: What if arbiter's ReACT loop runs forever?
   - Tool call limit per turn?
   - Wall-clock timeout?
   - Token budget?

## File Locations

```
crow-tauri/src-tauri/core/src/
  agent/
    dual.rs          # DualAgentRuntime
    executor.rs      # Existing (used by both agents)
  tools/
    task_complete.rs # New
    compact.rs       # New  
    screenshot.rs    # New
    browser.rs       # New (browser_navigate, browser_click, etc.)
    desktop.rs       # New (desktop_capture)
  session/
    export.rs        # Enhanced for dual agent rendering
  config/
    types.rs         # Add DualAgentConfig
    loader.rs        # Load and validate dual config
```

## Success Criteria

The dual agent system is complete when:

1. User can configure executor and arbiter agents via markdown files
2. Dual loop executes: executor works → arbiter verifies → iterate or complete
3. Arbiter's full session (including image descriptions) is sent to executor
4. Only `task_complete` terminates the loop (or max_steps)
5. Both sessions are fully persisted and observable
6. Missing `task_complete` in arbiter config raises clear error
7. Vision tools capture and describe visual state
8. `compact` tool manages context size
