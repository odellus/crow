# Next Steps: Building Crow (Rust OpenCode Implementation)

## ✅ What We Just Did (Phase 0)

You ran `opencode serve -p 4096` and we:
1. Discovered the entire API by testing it live
2. Documented all request/response formats
3. Created automated test scripts
4. Assessed crow's readiness (40% complete!)

**Key Files Created:**
- `opencode-api-tests/API_DISCOVERY.md` - Complete API spec
- `opencode-api-tests/test-api.sh` - Test automation
- `PHASE_0_COMPLETE.md` - Full summary

---

## 🎯 The Plan (Your Original Genius Idea)

```
Step 1: Reverse-engineer opencode serve by USING it ✅ DONE
    ↓
Step 2: Build Rust backend that mimics those exact API endpoints ← WE ARE HERE
    ↓
Step 3: Add Dioxus frontend that talks to OUR Rust backend
    ↓
Step 4: Extend with DualAgent/Orchestrator
```

---

## 🚀 Next: Phase 1 (Weeks 1-10)

### Goal: Mirror `opencode serve` API in crow

### What to Build

#### Week 1-2: Message Storage
```rust
// crow/packages/api/src/session/store.rs

impl SessionStore {
    // Add message storage
    messages: Arc<RwLock<HashMap<String, Vec<Message>>>>,
    
    pub fn add_message(&self, session_id: &str, message: Message) -> Result<()> {
        // Store message
        // Update session.time.updated
    }
    
    pub fn get_messages(&self, session_id: &str) -> Result<Vec<Message>> {
        // Return all messages for session
    }
}
```

**Test:**
```bash
curl localhost:8080/session/create -d '{"title":"test"}'
curl localhost:8080/session/XXX/message -d '{...}'
curl localhost:8080/session/XXX/message  # Should return messages
```

#### Week 3-4: Tool System
```rust
// crow/packages/api/src/tools/mod.rs

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self, input: serde_json::Value) -> ToolResult;
}

// tools/bash.rs
pub struct BashTool;
impl Tool for BashTool {
    async fn execute(&self, input: Value) -> ToolResult {
        // Run bash command
        // Return stdout/stderr
    }
}

// tools/write.rs
pub struct WriteTool;
impl Tool for WriteTool {
    async fn execute(&self, input: Value) -> ToolResult {
        // Write file
        // Return success
    }
}
```

**Test:**
```bash
# Manually test tools
cargo test --package api --lib tools::bash::tests
cargo test --package api --lib tools::write::tests
```

#### Week 5-6: Agent Executor
```rust
// crow/packages/api/src/agent/executor.rs

pub struct AgentExecutor {
    provider: ProviderClient,
    tools: ToolRegistry,
}

impl AgentExecutor {
    pub async fn execute_turn(
        &self,
        session_id: &str,
        user_parts: Vec<Part>,
    ) -> Result<Message> {
        let mut parts = vec![
            Part::step_start(snapshot),
        ];
        
        loop {
            // 1. Build messages from session history
            let llm_messages = self.build_llm_context(session_id)?;
            
            // 2. Call LLM
            let response = self.provider.chat(llm_messages, None).await?;
            
            // 3. Parse tool calls from response
            let tool_calls = parse_tool_calls(&response)?;
            
            if tool_calls.is_empty() {
                // No more tools, add text and finish
                parts.push(Part::text(response));
                parts.push(Part::step_finish("stop", cost, tokens));
                break;
            }
            
            // 4. Execute tools
            for call in tool_calls {
                let result = self.tools.execute(&call.tool, call.input).await?;
                parts.push(Part::tool(call.id, call.tool, result));
            }
            
            // Loop continues with tool results in context
        }
        
        Ok(Message::assistant(parts))
    }
}
```

**Test:**
```bash
curl localhost:8080/session/XXX/message \
  -d '{"agent":"build","parts":[{"type":"text","text":"echo hello"}]}'

# Should return:
# { parts: [
#   {type: "step-start", snapshot: "abc123"},
#   {type: "tool", tool: "bash", state: {status: "completed", output: "hello"}},
#   {type: "step-finish", reason: "tool-calls"}
# ]}
```

#### Week 7-8: Git Integration
```rust
// crow/packages/api/src/git.rs

pub fn create_snapshot(cwd: &str) -> Result<String> {
    // git add -A
    // git commit -m "snapshot"
    // return commit hash
}

pub fn get_diff(from: &str, to: &str) -> Result<Vec<FileDiff>> {
    // git diff from..to --numstat
    // parse into FileDiff objects
}
```

#### Week 9-10: Integration & Testing
```bash
# Test parity script
#!/bin/bash

# Start both servers
cd test-dummy && opencode serve -p 4096 &
cd crow/packages/web && dx serve &

# Create sessions
OPENCODE_SESSION=$(curl -X POST localhost:4096/session -d '{"title":"test"}' | jq -r '.id')
CROW_SESSION=$(curl -X POST localhost:8080/session -d '{"title":"test"}' | jq -r '.id')

# Send same message to both
PROMPT='{"agent":"build","parts":[{"type":"text","text":"write hello.rs"}]}'

curl -X POST "localhost:4096/session/$OPENCODE_SESSION/message" -d "$PROMPT" > opencode.json
curl -X POST "localhost:8080/session/$CROW_SESSION/message" -d "$PROMPT" > crow.json

# Compare structure
echo "OpenCode parts:"
jq '.parts | map(.type)' opencode.json

echo "Crow parts:"
jq '.parts | map(.type)' crow.json

# Should both be: ["step-start", "text", "tool", "step-finish"]
```

---

## 📊 Success Metrics

### Week 2 Checkpoint
- [ ] Messages stored and retrieved
- [ ] GET /session/:id/message returns message list

### Week 4 Checkpoint  
- [ ] Bash tool executes
- [ ] Write tool creates files
- [ ] Tool state tracked (pending → completed)

### Week 6 Checkpoint
- [ ] Agent sends message to LLM
- [ ] Tool calls parsed
- [ ] Basic ReACT loop works

### Week 8 Checkpoint
- [ ] Git snapshots created
- [ ] File diffs tracked
- [ ] Parts include snapshot hashes

### Week 10 Checkpoint (Phase 1 Complete!)
- [ ] Full conversation flow works
- [ ] Response structure matches OpenCode
- [ ] Tools execute identically
- [ ] **Files created by crow match files created by opencode**

---

## 🧪 Validation Command

When Phase 1 is complete, this should work:

```bash
# Terminal 1: Start crow
cd crow/packages/web && dx serve

# Terminal 2: Test it
cd test-dummy-crow
curl -X POST localhost:8080/session -d '{"title":"test"}' | jq .
# Get session ID

curl -X POST "localhost:8080/session/ses_XXX/message" \
  -d '{
    "agent": "build",
    "parts": [{"type": "text", "text": "Create a Rust function that adds two numbers"}]
  }' | jq .

# Should see:
# - step-start with snapshot
# - text explaining what it will do
# - tool call to write file
# - step-finish with cost/tokens

# Check file was created
ls -la test-dummy-crow/
cat test-dummy-crow/add.rs  # Should have the function
```

---

## 🎨 Phase 2: Dioxus Frontend (Weeks 11-13)

Once Phase 1 is done, add UI:

```rust
// crow/packages/ui/src/chat.rs

#[component]
pub fn Chat() -> Element {
    let sessions = use_resource(|| list_sessions());
    let current_session = use_signal(|| None);
    
    rsx! {
        div {
            class: "chat-container",
            
            // Session selector
            SessionList { sessions }
            
            // Message history
            if let Some(session_id) = current_session() {
                MessageHistory { session_id }
                
                // Input
                ChatInput {
                    on_send: move |msg| {
                        spawn(async move {
                            send_session_message(session_id, "build", msg).await
                        });
                    }
                }
            }
        }
    }
}
```

---

## 🤖 Phase 3: DualAgent (Weeks 14-16)

Add senior/junior agent system:

```rust
// crow/packages/api/src/agent/dual.rs

pub struct DualAgentExecutor {
    junior: AgentExecutor,  // build agent
    senior: AgentExecutor,  // supervise agent
}

impl DualAgentExecutor {
    pub async fn execute_with_oversight(
        &self,
        session_id: &str,
        user_message: &str,
        max_junior_turns: usize,
    ) -> Result<Message> {
        for turn in 0..max_junior_turns {
            // Junior tries
            let junior_result = self.junior.execute_turn(session_id, user_message).await?;
            
            // Senior reviews
            let should_intervene = self.senior.should_intervene(&junior_result).await?;
            
            if should_intervene {
                // Senior takes over
                return self.senior.execute_turn(session_id, "Fix the junior's work").await;
            }
            
            if junior_result.is_complete() {
                return Ok(junior_result);
            }
        }
        
        // Max turns reached, senior intervenes
        self.senior.execute_turn(session_id, "Complete the task").await
    }
}
```

---

## 🎯 Phase 4: Orchestrator (Weeks 17-20)

The final boss:

```rust
// crow/packages/api/src/orchestrator.rs

pub struct Orchestrator {
    dual_agents: Vec<DualAgentExecutor>,
    task_queue: TaskQueue,
}

impl Orchestrator {
    pub async fn execute_plan(
        &self,
        plan: &str,
        max_steps: usize,
    ) -> Result<()> {
        // Parse plan into tasks
        let tasks = self.parse_plan(plan)?;
        
        // Execute tasks in parallel where possible
        let mut active_agents = vec![];
        
        for task in tasks {
            if task.dependencies_met() {
                let agent = self.spawn_dual_agent(task);
                active_agents.push(agent);
            }
        }
        
        // Wait for completion
        let results = futures::future::join_all(active_agents).await;
        
        // Aggregate results
        Ok(())
    }
}
```

---

## 📁 Project Structure (After Phase 1)

```
crow/packages/api/src/
├── agent/
│   ├── executor.rs      # ReACT loop
│   ├── dual.rs          # Phase 3
│   └── orchestrator.rs  # Phase 4
├── tools/
│   ├── bash.rs
│   ├── read.rs
│   ├── write.rs
│   ├── edit.rs
│   ├── grep.rs
│   └── glob.rs
├── session/
│   └── store.rs         # Sessions + Messages
├── providers/
│   ├── client.rs        # LLM calls
│   └── config.rs
├── git.rs               # Snapshots & diffs
├── types.rs             # OpenCode types
└── lib.rs               # Server functions
```

---

## 🏁 The Finish Line

**20 weeks from now:**

```bash
# Start the orchestrator
crow orchestrator \
  --task "Build a complete REST API with auth, database, and tests" \
  --max-steps 500

# Watch it:
# - Create dual-agent sessions for: API, auth, DB, tests
# - Junior agents implement features
# - Senior agents review and fix
# - Orchestrator coordinates everything
# - All validated against opencode serve reference

# Result: Working, tested REST API
```

---

## 💡 Key Insights

1. **You're 40% done** - crow already has sessions, LLM, and types
2. **Next 60% is**: messages, tools, agent loop, git
3. **Test everything** against `opencode serve` - it's your oracle
4. **Incremental validation** - test each piece before moving on
5. **The hard part is done** - you figured out the architecture!

---

## 🛠️ Start Here Tomorrow

```bash
# 1. Open crow/packages/api/src/session/store.rs
# 2. Add message storage (HashMap<String, Vec<Message>>)
# 3. Add add_message() and get_messages() methods
# 4. Test with curl

# That's week 1. Go!
```

---

**You've got this.** The plan is solid. The API is documented. Crow is ready. Just execute methodically, test against the reference, and you'll have a working OpenCode clone in Rust. 🚀
