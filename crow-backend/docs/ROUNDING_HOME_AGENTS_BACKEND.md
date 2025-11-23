# Rounding Home: Agents & Backend (~40% → 80%)

## Reality Check

We're at maybe **40% actual parity**, not 70%. We've done the foundation work (system prompts, storage, config), but haven't tested with real LLM calls or built the critical agent ecosystem.

## Critical Question: ARE WE STREAMING? 🤔

**MUST VERIFY**: Does Crow actually stream responses like OpenCode?

### What OpenCode Does
```typescript
// From session/prompt.ts
const stream = streamText({
  // ... config
})

for await (const value of stream.fullStream) {
  // Streams: text-delta, tool-input-delta, reasoning-delta, etc.
  // Updates UI in real-time
}
```

### What Crow Might Be Doing
```rust
// In executor.rs - are we streaming or blocking?
let response = self.provider.chat_with_tools(...).await?;
// ^ Is this blocking until complete? Or streaming?
```

**ACTION NEEDED**: Check if `ProviderClient::chat_with_tools()` streams or blocks!

---

## The Frontend Problem

You said it: **"I'm worried I'm spending way too much time on the backend and interacting with the agent through you [an agent] instead of running myself"**

### Current State
- **Backend**: System prompts ✅, Storage ✅, Config ✅
- **Frontend**: Dioxus shell exists, but... does it work?
- **Testing**: Via Claude Code (meta!), not by actually running Crow

### The Risk
We're building a carbon copy of OpenCode's backend without:
1. Actually running it ourselves
2. Seeing if responses stream
3. Having a working UI to interact with
4. Testing basic workflows (create session, send message, see response)

---

## Proposed Plan: Backend-First Validation

### Phase 1: Basic Backend Works (Week 1)
**Goal**: Prove Crow can handle a simple task end-to-end

1. **Verify Streaming Works**
   - [ ] Check `ProviderClient` implementation
   - [ ] Does it use SSE (Server-Sent Events) like OpenCode?
   - [ ] Can we see incremental text responses?
   - [ ] Test: `curl http://localhost:3000/sessions/{id}/messages -d '{"text":"hello"}'`

2. **Test Basic LLM Call**
   - [ ] Start crow: `cd ~/test-project && crow`
   - [ ] Create session via API
   - [ ] Send message
   - [ ] Verify response comes back (streaming or not)
   - [ ] Check system prompt in logs with `CROW_VERBOSE=1`

3. **Verify Tool Execution**
   - [ ] Send message that requires tool use: "read package.json"
   - [ ] Does Read tool execute?
   - [ ] Does response include file contents?
   - [ ] Check logs for tool execution

**Success Criteria**: Can create session, send message, get response with tool execution. Even if UI is broken, backend proves it works.

### Phase 2: Agent Ecosystem (Week 2)
**Goal**: Build, Plan, Explore agents working

1. **Add Plan Agent**
   - [ ] Copy from OpenCode's `agent.ts` 
   - [ ] Read-only tools (read, grep, list, web_search)
   - [ ] Custom prompt from `plan.txt` (already copied)
   - [ ] Test: Plan agent can explore codebase without editing

2. **Add Explore Agent**
   - [ ] Copy from OpenCode
   - [ ] Specialized for codebase search
   - [ ] Test: Can find functions, classes, understand structure

3. **Build Agent Polish**
   - [ ] Verify has all tools (read, write, edit, bash, grep, glob)
   - [ ] Test: Can complete simple tasks (add function, fix bug)

4. **Background Bash**
   - [ ] Implement BashOutput tool
   - [ ] Implement KillShell tool
   - [ ] Test: Long-running command (npm install, cargo build)

**Success Criteria**: Can switch agents, each has correct tools, background bash works.

### Phase 3: Comparative Testing (Week 3)
**Goal**: Same task on OpenCode vs Crow, compare results

**Test Suite**:
1. **Simple Task**: "Create hello.txt with 'Hello World'"
   - OpenCode: Time, steps, cost
   - Crow: Time, steps, cost
   - Compare: Tool calls, response quality

2. **Medium Task**: "Add a new route to this Express app"
   - OpenCode: Plan → Execute → Verify
   - Crow: Plan → Execute → Verify
   - Compare: Accuracy, tool usage

3. **Complex Task**: "Implement user authentication with JWT"
   - OpenCode: Multi-step, multiple files
   - Crow: Multi-step, multiple files
   - Compare: Code quality, completeness

**Metrics**:
- Task completion rate
- Number of tool calls
- Time to completion
- Cost (tokens used)
- Code quality (does it work?)

### Phase 4: Frontend Revival (Week 4)
**Goal**: Working UI that doesn't suck

**Option A: Fix Dioxus UI**
- [ ] Does the web UI render?
- [ ] Can you create a session from UI?
- [ ] Does streaming work in browser?
- [ ] Chat interface usable?

**Option B: Minimal CLI**
- [ ] Simple REPL: `crow chat`
- [ ] Shows streaming responses
- [ ] Can switch agents
- [ ] Good enough for dogfooding

**Option C: Steal OpenCode's Frontend**
- [ ] OpenCode is TypeScript + React
- [ ] Can we just point it at Crow's backend?
- [ ] If APIs match, should "just work"
- [ ] Fastest path to working UI

**Decision Point**: Which is fastest to get a usable interface?

---

## The Streaming Investigation (DO THIS FIRST!)

### Check Provider Client

```rust
// packages/api/src/providers/client.rs
// Look for:
pub async fn chat_with_tools(...) -> Result<...> {
    // Is this:
    // A) Blocking: waits for full response
    // B) Streaming: returns iterator/stream
    // C) Hybrid: streams but we're not using it
}
```

### Check Server Endpoints

```rust
// packages/api/src/server.rs
// Look for message creation endpoint
// Does it:
// A) Return SSE (Server-Sent Events)
// B) Return JSON (blocking)
// C) Use WebSocket
```

### OpenCode Comparison

```typescript
// OpenCode does this:
export async function* streamText() {
  for await (const chunk of stream) {
    yield chunk  // Streams to frontend
  }
}
```

**Question**: Does Crow do equivalent?

---

## Honest Assessment: What's Actually Done?

### ✅ Solid Foundation (40%)
- System prompt building ✅
- Todo storage ✅
- XDG config ✅
- Verbose logging ✅
- Provider prompt files ✅
- findUp pattern ✅

### 🤔 Probably Works But Untested (20%)
- Tool execution (have tools, haven't tested)
- Session management (have code, haven't used)
- LLM integration (have provider, haven't called)
- Basic agent (build agent exists, never ran it)

### ❌ Definitely Missing (40%)
- Streaming responses (unknown if implemented)
- Plan agent (not added)
- Explore agent (not added)
- Background bash (not implemented)
- Working frontend (unknown state)
- Actual testing (only meta-testing via Claude Code)
- Comparative validation (never run OpenCode vs Crow side-by-side)

---

## The Meta Problem

We're using **Claude Code** (an agent) to build **Crow** (an agent system), but not actually **using Crow** to verify it works.

It's like building a car and asking someone else to describe what it's like to drive instead of test driving it yourself.

### Solution: Dogfood Early

**Week 1 Goal**: Get Crow to a state where you can actually interact with it (CLI or minimal UI), even if it's janky.

**Test**: Try to use Crow to complete a simple task for real. Not through me, through the actual system.

---

## Proposed Next Session Tasks

### Immediate (Session 1)
1. **Investigate streaming** - Does it work? Yes/No
2. **Basic backend test** - Start crow, create session, send message, get response
3. **Add Plan agent** - Copy from OpenCode, test read-only mode

### Short-term (Sessions 2-3)
4. **Add Explore agent** - Codebase search specialist
5. **Background bash** - BashOutput, KillShell
6. **Frontend triage** - Fix Dioxus OR build minimal CLI OR steal OpenCode's UI

### Mid-term (Sessions 4-6)
7. **Comparative testing** - Run same tasks on both, document gaps
8. **Close gaps** - Fix whatever breaks in testing
9. **Dogfooding** - Use Crow for a real project task

---

## Success Metrics (Real Ones)

Not "did we copy the code?" but "does it actually work?"

**Minimum Viable Crow**:
- [ ] Can start crow in a project directory
- [ ] Can create a session (via API or UI)
- [ ] Can send a message
- [ ] Get a streaming response back
- [ ] Agent can use tools (read file, run command)
- [ ] Can see the output in real-time
- [ ] Can complete a simple task end-to-end

**That's 80%**. Everything else is polish.

---

## The Question You Asked

> "Does that make any sense?"

**YES.** You're absolutely right.

We've been building the backend engine while ignoring:
1. Does it turn over?
2. Can you steer it?
3. Does it go forward?

Time to test drive the car instead of just building it.

---

## Recommended Approach: Streaming-First Investigation

**Next session, start here**:

```bash
# 1. Start crow with verbose
cd ~/test-project
CROW_VERBOSE=1 cargo run --bin crow

# 2. In another terminal, test API
curl -X POST http://localhost:3000/sessions \
  -H "Content-Type: application/json" \
  -d '{"working_directory": "/home/thomas/test-project"}'

# 3. Send a message
curl -X POST http://localhost:3000/sessions/{session_id}/messages \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello, can you read package.json?"}'

# 4. Does it stream? Block? Error?
```

**If it works**: Great! Add Plan agent, test more.

**If it doesn't work**: Fix the basics before adding more complexity.

**If streaming doesn't work**: That's the #1 priority to fix - OpenCode's UX depends on streaming.

---

## Bottom Line

You're at **~40% real completion**, and you're right to pump the brakes on backend work until we verify:

1. **Streaming works**
2. **Basic LLM calls work**
3. **You can actually use it** (not just build it)

Then we can confidently push toward 80% with agents, tools, and real comparative testing.

**Let's build less, test more, and actually use the thing.** 🚗
