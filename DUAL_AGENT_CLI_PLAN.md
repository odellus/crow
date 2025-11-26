# Dual Agent CLI Plan

## Agent Config - ALREADY DONE

`AgentRegistry::new_with_config()` in `agent/registry.rs` already loads from:
1. Built-in agents (general, build, plan)
2. Global: `~/.config/crow/agent/*.md`
3. Project: `.crow/agent/*.md`

Project overrides global. Format is markdown with YAML frontmatter.

**Just ensure `new_with_config()` is called at startup instead of `new()`.**

---

## What We Need to Build

### 1. task_complete Tool

```rust
// tools/task_complete.rs

pub struct TaskCompleteTool;

impl Tool for TaskCompleteTool {
    fn name(&self) -> &str { "task_complete" }
    
    fn description(&self) -> &str {
        "Signal that the task is complete and verified. Call only when you have \
         confirmed the work meets requirements."
    }
    
    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        #[derive(Deserialize)]
        struct Input {
            summary: String,
            verification: String,
        }
        
        let input: Input = serde_json::from_value(input)?;
        
        ToolResult {
            output: format!(
                "Task complete.\n\nSummary: {}\n\nVerification: {}",
                input.summary, input.verification
            ),
            metadata: json!({
                "task_complete": true,
                "summary": input.summary,
                "verification": input.verification,
            }),
            status: ToolStatus::Completed,
        }
    }
}
```

### 2. Arbiter Agent

Either add to `builtins.rs` OR create `~/.config/crow/agent/arbiter.md`:

```rust
pub fn get_arbiter_agent() -> AgentInfo {
    AgentInfo {
        name: "arbiter".to_string(),
        description: Some("Verification agent for dual-agent tasks".to_string()),
        mode: AgentMode::Subagent,
        built_in: true,
        temperature: Some(0.3),
        top_p: None,
        color: Some("#10B981".to_string()),
        permission: AgentPermissions::default_allow(),
        model: None,
        prompt: Some(ARBITER_PROMPT.to_string()),
        tools: {
            let mut tools = default_tools();
            tools.insert("task_complete".to_string(), true);
            tools
        },
        options: HashMap::new(),
    }
}

const ARBITER_PROMPT: &str = r#"
You are the Arbiter agent in a dual-agent system.

You receive the Executor's full session showing everything it did - all thinking, 
tool calls, and outputs.

Your job is to VERIFY the work:
1. Read what the Executor did
2. Run tests (cargo test, npm test, etc.)
3. Check the code actually runs
4. Verify requirements are met

If everything works correctly:
- Call task_complete with summary and verification

If there are problems:
- Explain what's wrong and why
- Your full session will be sent back to the Executor
"#;
```

### 3. DualAgentRuntime

```rust
// agent/dual.rs

pub struct DualAgentRuntime {
    executor: AgentExecutor,
    session_store: Arc<SessionStore>,
    agent_registry: Arc<AgentRegistry>,
}

pub struct DualResult {
    pub executor_sessions: Vec<String>,
    pub arbiter_sessions: Vec<String>,
    pub steps: u32,
    pub completed: bool,
    pub summary: Option<String>,
}

impl DualAgentRuntime {
    pub async fn run(
        &self,
        initial_prompt: &str,
        parent_session_id: &str,
        working_dir: &Path,
        executor_agent: &str,  // Usually "build"
        arbiter_agent: &str,   // Usually "arbiter"
        max_steps: u32,
    ) -> Result<DualResult, String> {
        let pair_id = generate_pair_id();
        let mut executor_sessions = Vec::new();
        let mut arbiter_sessions = Vec::new();
        let mut input = initial_prompt.to_string();
        
        for step in 1..=max_steps {
            // === EXECUTOR ===
            let executor_session = self.create_session(
                working_dir,
                parent_session_id,
                &format!("Dual: Executor (Step {})", step),
                "executor",
                &pair_id,
                step,
            )?;
            executor_sessions.push(executor_session.id.clone());
            
            self.add_user_message(&executor_session.id, &input)?;
            
            let _executor_result = self.executor.execute_turn(
                &executor_session.id,
                executor_agent,
                working_dir,
                vec![],
            ).await?;
            
            // Render executor session
            let executor_markdown = self.render_session(&executor_session.id)?;
            
            // === ARBITER ===
            let arbiter_session = self.create_session(
                working_dir,
                parent_session_id,
                &format!("Dual: Arbiter (Step {})", step),
                "arbiter",
                &pair_id,
                step,
            )?;
            arbiter_sessions.push(arbiter_session.id.clone());
            
            // Link siblings
            self.link_siblings(&executor_session.id, &arbiter_session.id)?;
            
            self.add_user_message(&arbiter_session.id, &executor_markdown)?;
            
            let arbiter_result = self.executor.execute_turn(
                &arbiter_session.id,
                arbiter_agent,
                working_dir,
                vec![],
            ).await?;
            
            // Check for task_complete
            if let Some(completion) = self.find_task_complete(&arbiter_result) {
                return Ok(DualResult {
                    executor_sessions,
                    arbiter_sessions,
                    steps: step,
                    completed: true,
                    summary: Some(completion.summary),
                });
            }
            
            // Render arbiter session for next executor iteration
            input = self.render_session(&arbiter_session.id)?;
        }
        
        Ok(DualResult {
            executor_sessions,
            arbiter_sessions,
            steps: max_steps,
            completed: false,
            summary: None,
        })
    }
    
    fn create_session(
        &self,
        working_dir: &Path,
        parent_id: &str,
        title: &str,
        role: &str,
        pair_id: &str,
        step: u32,
    ) -> Result<Session, String> {
        let session = self.session_store.create(
            working_dir.to_string_lossy().to_string(),
            Some(parent_id.to_string()),
            Some(title.to_string()),
        )?;
        
        self.session_store.update_metadata(&session.id, json!({
            "dual_agent": {
                "role": role,
                "pair_id": pair_id,
                "step": step,
            }
        }))?;
        
        Ok(session)
    }
    
    fn link_siblings(&self, executor_id: &str, arbiter_id: &str) -> Result<(), String> {
        // Update executor with arbiter sibling
        let executor = self.session_store.get(executor_id)?;
        let mut meta = executor.metadata.unwrap_or(json!({}));
        meta["dual_agent"]["sibling_id"] = json!(arbiter_id);
        self.session_store.update_metadata(executor_id, meta)?;
        
        // Update arbiter with executor sibling
        let arbiter = self.session_store.get(arbiter_id)?;
        let mut meta = arbiter.metadata.unwrap_or(json!({}));
        meta["dual_agent"]["sibling_id"] = json!(executor_id);
        self.session_store.update_metadata(arbiter_id, meta)?;
        
        Ok(())
    }
    
    fn render_session(&self, session_id: &str) -> Result<String, String> {
        crate::session::export::render_for_dual_agent(
            &self.session_store,
            session_id,
        )
    }
    
    fn find_task_complete(&self, result: &MessageWithParts) -> Option<TaskCompleteInfo> {
        for part in &result.parts {
            if let Part::Tool { tool, state, .. } = part {
                if tool == "task_complete" {
                    if let ToolState::Completed { metadata, .. } = state {
                        if metadata.get("task_complete") == Some(&json!(true)) {
                            return Some(TaskCompleteInfo {
                                summary: metadata.get("summary")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                            });
                        }
                    }
                }
            }
        }
        None
    }
}
```

---

### 4. CLI Integration

#### Display

Both agents render to stdout the same way as normal chat. Different colors distinguish them:

```rust
// Different ANSI colors for executor vs arbiter
const EXECUTOR_COLOR: &str = "\x1b[36m";  // Cyan
const ARBITER_COLOR: &str = "\x1b[32m";   // Green
const RESET: &str = "\x1b[0m";

fn print_agent_header(role: &str, step: u32) {
    let color = if role == "executor" { EXECUTOR_COLOR } else { ARBITER_COLOR };
    println!("{}═══ {} (Step {}) ═══{}", color, role.to_uppercase(), step, RESET);
}
```

#### --dual Flag

```rust
// bin/crow-cli.rs

#[derive(Parser)]
struct ChatArgs {
    message: String,
    
    #[arg(long)]
    session: Option<String>,
    
    /// Run as dual agent (executor + arbiter loop)
    #[arg(long)]
    dual: bool,
    
    /// Max steps for dual agent loop (default: 5)
    #[arg(long, default_value = "5")]
    max_steps: u32,
}
```

---

## Files to Create/Modify

```
tools/task_complete.rs  # NEW
tools/mod.rs            # Register task_complete
agent/dual.rs           # NEW - DualAgentRuntime
agent/builtins.rs       # Add arbiter agent (optional, can use config file)
session/export.rs       # Add render_for_dual_agent()
bin/crow-cli.rs         # Add --dual flag, colors
```

---

## Testing
```bash
# Test task_complete tool
crow-cli chat "Call task_complete with summary 'test' and verification 'manual'"

# Test dual loop directly
crow-cli chat --dual "Create hello.txt with 'Hello' and verify it exists"

# Visual check - colors should alternate
crow-cli chat --dual "Write a function that adds numbers, test it"

# Check sessions created
crow-cli sessions | grep "Dual"
```

---

## Success Criteria

1. Custom agents load from `~/.config/crow/agent/*.md` and `.crow/agent/*.md`
2. Project agents override global agents
3. `task_complete` tool works and terminates dual loop
4. `crow-cli chat --dual` runs executor → arbiter → executor... loop
5. Both agents display with different colors
6. All sessions persisted with dual_agent metadata
7. Sibling sessions linked via sibling_id
