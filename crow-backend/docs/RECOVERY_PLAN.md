# Crow Recovery & Enhancement Plan

## Current Status: ✅ STORAGE IS WORKING!

**Verified:**
- Sessions ARE being saved to `.crow/storage/session/*.json`
- Messages ARE being saved to `.crow/storage/message/{session_id}/*.json`
- Parts ARE being saved to `.crow/storage/part/{message_id}/*.json`
- Markdown exports ARE being generated to `.crow/sessions/*.md`

**No data loss occurred!**

---

## Implementation Priorities

Based on the comprehensive comparison in `OPENCODE_CROW_COMPARISON.md`:

### 🔴 **PRIORITY 1: Critical for Production** (Week 1)

#### 1.1 Session Locking & Abort
**Problem:** Multiple requests to same session can cause race conditions. No way to cancel runaway agents.

**Implementation:**
```rust
// crow/packages/api/src/session/lock.rs
pub struct SessionLock {
    session_id: String,
    locked_at: u64,
    abort_signal: Arc<AtomicBool>,
}

impl SessionLock {
    pub fn is_locked(&self) -> bool { ... }
    pub fn abort(&self) { ... }
}

// crow/packages/api/src/session/manager.rs
pub struct SessionManager {
    locks: Arc<RwLock<HashMap<String, SessionLock>>>,
}
```

**API Endpoint:**
```
POST /session/:id/abort
```

**Files to create:**
- `crow/packages/api/src/session/lock.rs`
- Update `crow/packages/api/src/session/store.rs` to use locks
- Add abort endpoint in `crow/packages/api/src/server.rs`

---

#### 1.2 Retry Logic with Exponential Backoff
**Problem:** Transient API failures (rate limits, timeouts) kill sessions permanently.

**Implementation:**
```rust
// crow/packages/api/src/session/retry.rs
pub struct SessionRetry;

impl SessionRetry {
    const MAX_RETRIES: u32 = 10;
    const BASE_DELAY_MS: u64 = 1000;
    const MAX_DELAY_MS: u64 = 30000;
    
    pub fn get_bounded_delay(retry_count: u32) -> u64 {
        let delay = Self::BASE_DELAY_MS * 2u64.pow(retry_count);
        delay.min(Self::MAX_DELAY_MS)
    }
    
    pub async fn with_retry<F, T>(f: F) -> Result<T, String>
    where
        F: Fn() -> Future<Output = Result<T, String>>,
    {
        // Retry loop with backoff
    }
}
```

**Files to create:**
- `crow/packages/api/src/session/retry.rs`
- Update `crow/packages/api/src/agent/executor.rs` to use retry

---

### 🟡 **PRIORITY 2: User Experience** (Week 2-3)

#### 2.1 Permission "Ask" Mode
**Problem:** Can only allow/deny tools. Can't review dangerous commands before execution.

**Current state:**
```rust
// We have Permission::Allow and Permission::Deny
// Missing: Permission::Ask with interactive review
```

**Implementation needed:**
- Add `Permission::Ask` variant
- Store pending permission requests in SessionStore
- Add `POST /session/:id/permissions/:permissionID` endpoint
- Update tool executor to pause on Ask and wait for user response

**OpenCode Reference:**
```typescript
// opencode/packages/opencode/src/agent/permissions.ts
export type Permission = "allow" | "deny" | "ask"

// When ask is triggered:
POST /session/:id/permissions/:permissionID
Body: { response: "allow" | "deny" }
```

---

#### 2.2 Doom Loop Detection
**Problem:** Agent can call same tool with same args repeatedly, wasting tokens.

**Implementation:**
```rust
// Track last N tool calls
struct DoomLoopDetector {
    recent_calls: VecDeque<(String, String)>, // (tool_name, args_hash)
    max_history: usize, // 3
}

impl DoomLoopDetector {
    fn check_doom_loop(&mut self, tool: &str, args: &str) -> bool {
        let hash = hash_args(args);
        if self.recent_calls.iter().filter(|(t, h)| t == tool && h == &hash).count() >= 3 {
            return true; // Doom loop detected!
        }
        self.recent_calls.push_back((tool.to_string(), hash));
        if self.recent_calls.len() > self.max_history {
            self.recent_calls.pop_front();
        }
        false
    }
}
```

**Files to modify:**
- `crow/packages/api/src/agent/executor.rs` - Add doom loop detection before tool execution

---

### 🟢 **PRIORITY 3: Scalability** (Month 1-2)

#### 3.1 Session Compaction
**Problem:** Long sessions hit model context limits and fail.

**OpenCode's approach:**
- Auto-detects when session exceeds ~80% of model context
- Summarizes old messages using a fast model
- Prunes old tool outputs (keeps last 40k tokens worth)
- Stores summary as system message

**Implementation:**
- Create `crow/packages/api/src/session/compaction.rs`
- Estimate token count for each message
- When total > threshold, trigger compaction
- Generate summary of old messages
- Replace old messages with summary

---

#### 3.2 Message Pruning
**Problem:** Old tool outputs waste tokens in context.

**Implementation:**
- Before sending to LLM, prune tool outputs older than 40k tokens
- Keep recent tool outputs (they're relevant)
- OpenCode keeps last ~20 messages of tool outputs, removes older

---

### 🔵 **PRIORITY 4: Nice-to-Have** (Month 2-3)

- Session revert (time-travel)
- Auto-summarization for session titles
- File diff tracking
- Event bus for real-time UI updates
- LSP integration
- MCP server support
- File watching

---

## Immediate Action Items

### This Week:

1. **Implement Session Locking**
   - [ ] Create `crow/packages/api/src/session/lock.rs`
   - [ ] Add lock registry to SessionStore
   - [ ] Add abort functionality
   - [ ] Add `POST /session/:id/abort` endpoint

2. **Implement Retry Logic**
   - [ ] Create `crow/packages/api/src/session/retry.rs`
   - [ ] Add retry wrapper for LLM calls
   - [ ] Add exponential backoff

3. **Test thoroughly**
   - [ ] Test abort cancels running sessions
   - [ ] Test retry recovers from transient failures
   - [ ] Test concurrent session access with locks

### Next Week:

4. **Permission Ask Mode**
   - [ ] Add `Permission::Ask` enum variant
   - [ ] Store pending requests
   - [ ] Add permission response endpoint
   - [ ] Update tool execution to pause and wait

5. **Doom Loop Detection**
   - [ ] Add detector to executor
   - [ ] Track last 3 tool calls
   - [ ] Show warning on doom loop

---

## Files That Need Work

### New Files to Create:
1. `crow/packages/api/src/session/lock.rs` - Session locking
2. `crow/packages/api/src/session/retry.rs` - Retry logic
3. `crow/packages/api/src/session/compaction.rs` - Context compaction (later)

### Existing Files to Modify:
1. `crow/packages/api/src/session/store.rs` - Add lock support
2. `crow/packages/api/src/agent/executor.rs` - Add retry + doom loop detection
3. `crow/packages/api/src/agent/permissions.rs` - Add Ask variant
4. `crow/packages/api/src/server.rs` - Add abort endpoint

---

## Success Criteria

### Week 1 Done When:
- ✅ Can abort a running session via API
- ✅ Sessions retry on transient API errors
- ✅ Multiple concurrent requests don't corrupt session state

### Week 2 Done When:
- ✅ Dangerous bash commands require user approval
- ✅ Doom loops are detected and reported
- ✅ All Priority 1 & 2 items complete

### Month 1 Done When:
- ✅ Long sessions don't fail from context limits
- ✅ Old tool outputs are pruned automatically
- ✅ Crow is production-ready

---

## What We Already Have (Don't Need to Rebuild)

✅ **Storage system** - Working perfectly  
✅ **Session CRUD** - Create, read, update sessions  
✅ **Message storage** - Messages and parts persisted  
✅ **Markdown export** - Real-time streaming export  
✅ **Tool execution** - Full tool registry and execution  
✅ **Agent execution** - ReACT loop with tool calls  
✅ **Dual-agent system** - Executor/discriminator collaboration  

**No need to rebuild these!** Focus on the priorities above.

---

## References

- Full comparison: `OPENCODE_CROW_COMPARISON.md`
- OpenCode source: `/home/thomas/src/projects/opencode-project/opencode/`
- Crow source: `/home/thomas/src/projects/opencode-project/crow/`
