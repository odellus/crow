# Critical Gaps - Implementation Checklist

## Reality Check ❌

### We do NOT have carbon copy agents!

**Missing:**
- ❌ Full OpenCode agent prompts (BUILD, PLAN, DOCS, etc.)
- ❌ Task tool for spawning subagents
- ❌ Agent prompt management from `.crow/agent/*.md` files
- ❌ Proper agent system prompts

**What we have:**
- ✅ Basic agent structure (AgentInfo, AgentMode, permissions)
- ✅ Six agent types defined (general, build, plan, supervisor, architect, discriminator)
- ✅ BUT they don't have the actual OpenCode prompts!

---

## HOTL Critical Gaps (No Permission Ask)

### 1. Session Locking & Abort ⏰ 2-3 hours

**Create: `crow/packages/api/src/session/lock.rs`**

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

/// Session lock with abort signal
pub struct SessionLock {
    pub session_id: String,
    pub locked_at: u64,
    pub abort_signal: Arc<AtomicBool>,
}

impl SessionLock {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            locked_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            abort_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_locked(&self) -> bool {
        !self.abort_signal.load(Ordering::Relaxed)
    }

    pub fn abort(&self) {
        self.abort_signal.store(true, Ordering::Relaxed);
    }

    pub fn should_abort(&self) -> bool {
        self.abort_signal.load(Ordering::Relaxed)
    }
}

/// Global lock manager
pub struct SessionLockManager {
    locks: Arc<RwLock<HashMap<String, Arc<SessionLock>>>>,
}

impl SessionLockManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn acquire(&self, session_id: &str) -> Result<Arc<SessionLock>, String> {
        let mut locks = self.locks.write().unwrap();
        
        if locks.contains_key(session_id) {
            return Err(format!("Session {} is already locked", session_id));
        }

        let lock = Arc::new(SessionLock::new(session_id.to_string()));
        locks.insert(session_id.to_string(), lock.clone());
        Ok(lock)
    }

    pub fn release(&self, session_id: &str) {
        let mut locks = self.locks.write().unwrap();
        locks.remove(session_id);
    }

    pub fn get(&self, session_id: &str) -> Option<Arc<SessionLock>> {
        let locks = self.locks.read().unwrap();
        locks.get(session_id).cloned()
    }

    pub fn abort(&self, session_id: &str) -> Result<(), String> {
        let locks = self.locks.read().unwrap();
        if let Some(lock) = locks.get(session_id) {
            lock.abort();
            Ok(())
        } else {
            Err(format!("Session {} is not locked", session_id))
        }
    }
}
```

**Update: `crow/packages/api/src/session/mod.rs`**
```rust
pub mod lock;
pub use lock::{SessionLock, SessionLockManager};
```

**Update: `crow/packages/api/src/server.rs`**
```rust
// Add to AppState
pub struct AppState {
    // ... existing fields
    pub lock_manager: Arc<SessionLockManager>,
}

// Add abort endpoint
async fn abort_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state.lock_manager.abort(&session_id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    
    Ok(Json(serde_json::json!({"aborted": true})))
}

// Add to router
.route("/session/:id/abort", post(abort_session))
```

**Update: `crow/packages/api/src/agent/executor.rs`**
```rust
// Check for abort in ReACT loop
pub async fn execute_turn(
    &self,
    session_id: &str,
    agent_name: &str,
    working_dir: &Path,
    user_parts: Vec<Part>,
) -> Result<MessageWithParts, String> {
    // Acquire lock
    let lock = self.lock_manager.acquire(session_id)?;
    
    // In loop, check abort
    loop {
        if lock.should_abort() {
            return Err("Session aborted by user".to_string());
        }
        
        // ... rest of ReACT loop
    }
    
    // Release lock when done
    self.lock_manager.release(session_id);
}
```

---

### 2. Retry Logic with Exponential Backoff ⏰ 2 hours

**Create: `crow/packages/api/src/session/retry.rs`**

```rust
use std::time::Duration;
use tokio::time::sleep;

pub struct SessionRetry;

impl SessionRetry {
    const MAX_RETRIES: u32 = 10;
    const BASE_DELAY_MS: u64 = 1000;
    const MAX_DELAY_MS: u64 = 30000;

    /// Calculate bounded exponential backoff delay
    pub fn get_bounded_delay(retry_count: u32) -> Duration {
        let delay_ms = Self::BASE_DELAY_MS * 2u64.pow(retry_count);
        let bounded = delay_ms.min(Self::MAX_DELAY_MS);
        Duration::from_millis(bounded)
    }

    /// Retry async operation with exponential backoff
    pub async fn with_retry<F, Fut, T>(mut f: F) -> Result<T, String>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        let mut retry_count = 0;

        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if retry_count >= Self::MAX_RETRIES {
                        return Err(format!(
                            "Max retries ({}) exceeded. Last error: {}",
                            Self::MAX_RETRIES,
                            e
                        ));
                    }

                    // Check if error is retryable
                    if !Self::is_retryable(&e) {
                        return Err(e);
                    }

                    let delay = Self::get_bounded_delay(retry_count);
                    eprintln!(
                        "[RETRY] Attempt {} failed: {}. Retrying in {:?}",
                        retry_count + 1,
                        e,
                        delay
                    );

                    sleep(delay).await;
                    retry_count += 1;
                }
            }
        }
    }

    /// Determine if error is retryable
    fn is_retryable(error: &str) -> bool {
        let error_lower = error.to_lowercase();
        
        // Retryable: rate limits, timeouts, network errors
        error_lower.contains("rate limit")
            || error_lower.contains("timeout")
            || error_lower.contains("connection")
            || error_lower.contains("503")
            || error_lower.contains("429")
            || error_lower.contains("temporary")
    }
}
```

**Update: `crow/packages/api/src/session/mod.rs`**
```rust
pub mod retry;
pub use retry::SessionRetry;
```

**Update: `crow/packages/api/src/agent/executor.rs`**
```rust
use crate::session::SessionRetry;

// Wrap LLM calls with retry
let response = SessionRetry::with_retry(|| async {
    self.provider
        .chat_completion(request.clone())
        .await
        .map_err(|e| e.to_string())
})
.await?;
```

---

### 3. Doom Loop Detection ⏰ 1 hour

**Update: `crow/packages/api/src/agent/executor.rs`**

```rust
use std::collections::{VecDeque, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};

struct DoomLoopDetector {
    recent_calls: VecDeque<(String, u64)>, // (tool_name, args_hash)
    max_history: usize,
}

impl DoomLoopDetector {
    fn new() -> Self {
        Self {
            recent_calls: VecDeque::new(),
            max_history: 10, // Track last 10 calls
        }
    }

    fn hash_args(args: &serde_json::Value) -> u64 {
        let mut hasher = DefaultHasher::new();
        format!("{:?}", args).hash(&mut hasher);
        hasher.finish()
    }

    fn check_doom_loop(&mut self, tool: &str, args: &serde_json::Value) -> bool {
        let args_hash = Self::hash_args(args);

        // Count identical calls (same tool + same args)
        let identical_count = self
            .recent_calls
            .iter()
            .filter(|(t, h)| t == tool && h == &args_hash)
            .count();

        if identical_count >= 3 {
            eprintln!(
                "[DOOM LOOP] Detected: {} called 3+ times with same args",
                tool
            );
            return true;
        }

        // Add to history
        self.recent_calls.push_back((tool.to_string(), args_hash));
        if self.recent_calls.len() > self.max_history {
            self.recent_calls.pop_front();
        }

        false
    }
}

// In AgentExecutor struct
pub struct AgentExecutor {
    // ... existing fields
    doom_detector: Arc<RwLock<HashMap<String, DoomLoopDetector>>>, // Per-session detectors
}

// In execute_turn, before executing tool:
let mut doom_detectors = self.doom_detector.write().unwrap();
let detector = doom_detectors
    .entry(session_id.to_string())
    .or_insert_with(DoomLoopDetector::new);

if detector.check_doom_loop(&tool_name, &tool_input) {
    return Err(format!(
        "Doom loop detected: {} called repeatedly with same arguments. Stopping execution.",
        tool_name
    ));
}
```

---

### 4. Task Tool for Subagents ⏰ 4-6 hours

**Create: `crow/packages/api/src/tools/task.rs`**

```rust
use async_openai::types::{
    ChatCompletionTool, ChatCompletionToolType, FunctionObject,
};
use serde_json::json;

pub struct TaskTool;

impl TaskTool {
    pub fn definition() -> ChatCompletionTool {
        ChatCompletionTool {
            r#type: ChatCompletionToolType::Function,
            function: FunctionObject {
                name: "Task".to_string(),
                description: Some("Launch a specialized agent to handle complex, multi-step tasks autonomously. Use this when you need to delegate work to a subagent.".to_string()),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "description": {
                            "type": "string",
                            "description": "A short (3-5 word) description of the task"
                        },
                        "prompt": {
                            "type": "string",
                            "description": "The detailed task for the agent to perform"
                        },
                        "subagent_type": {
                            "type": "string",
                            "description": "The type of specialized agent to use",
                            "enum": ["general", "plan", "docs"]
                        },
                        "model": {
                            "type": "string",
                            "description": "Optional model to use (haiku/sonnet/opus)",
                            "enum": ["haiku", "sonnet", "opus"]
                        }
                    },
                    "required": ["description", "prompt", "subagent_type"]
                })),
                strict: Some(false),
            },
        }
    }

    pub async fn execute(
        input: serde_json::Value,
        agent_executor: Arc<AgentExecutor>,
        session_id: String,
        working_dir: PathBuf,
    ) -> Result<String, String> {
        let description = input["description"]
            .as_str()
            .ok_or("Missing description")?;
        let prompt = input["prompt"].as_str().ok_or("Missing prompt")?;
        let subagent_type = input["subagent_type"]
            .as_str()
            .ok_or("Missing subagent_type")?;

        eprintln!("[TASK] Spawning {} subagent: {}", subagent_type, description);

        // Create user parts for subagent
        let user_parts = vec![Part::Text {
            id: format!("prt-{}", uuid::Uuid::new_v4()),
            session_id: session_id.clone(),
            message_id: format!("msg-task-{}", uuid::Uuid::new_v4()),
            text: prompt.to_string(),
        }];

        // Execute subagent
        let result = agent_executor
            .execute_turn(&session_id, subagent_type, &working_dir, user_parts)
            .await?;

        // Extract text from result parts
        let mut output = String::new();
        for part in &result.parts {
            if let Part::Text { text, .. } = part {
                output.push_str(text);
                output.push('\n');
            }
        }

        Ok(output)
    }
}
```

**Update: `crow/packages/api/src/tools/mod.rs`**
```rust
mod task;
pub use task::TaskTool;

// In register_tools()
tools.insert("Task".to_string(), Box::new(TaskTool));
```

---

### 5. Copy OpenCode Agent Prompts ⏰ 1-2 hours

**Check OpenCode prompts:**
```bash
cat /home/thomas/src/projects/opencode-project/opencode/.opencode/agent/*.md
```

**Copy to Crow:**
```bash
mkdir -p /home/thomas/src/projects/opencode-project/.crow/agent
cp /home/thomas/src/projects/opencode-project/opencode/.opencode/agent/*.md \
   /home/thomas/src/projects/opencode-project/.crow/agent/
```

**Update builtins.rs to load prompts from files:**
```rust
// In get_builtin_agents()
let build_prompt = std::fs::read_to_string(".crow/agent/build.md")
    .ok();

let build = AgentInfo {
    name: "build".to_string(),
    // ...
    prompt: build_prompt,
    // ...
};
```

---

## Estimated Total Time: 10-14 hours

### Priority Order:
1. **Session Locking & Abort** (2-3h) - Critical for safety
2. **Retry Logic** (2h) - Critical for reliability  
3. **Doom Loop Detection** (1h) - Prevents token waste
4. **Task Tool** (4-6h) - Needed for subagent spawning
5. **Agent Prompts** (1-2h) - Makes agents actually useful

## Testing Checklist

After implementing:
- [ ] Can abort a running session
- [ ] Sessions retry on rate limit errors
- [ ] Doom loops are detected and stopped
- [ ] Task tool can spawn subagents
- [ ] Agents have proper OpenCode prompts
