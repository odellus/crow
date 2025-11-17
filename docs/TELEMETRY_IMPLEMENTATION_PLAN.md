# Telemetry & Storage Implementation Plan

Based on OpenCode's architecture analysis.

## How OpenCode Does It

### Storage Model
```
~/.local/share/opencode/storage/
├── session/{projectID}/{sessionID}.json
├── message/{sessionID}/{messageID}.json
└── part/{messageID}/{partID}.json
```

### Dual-Pair Approach (Key Discovery!)

**ONE session with metadata tagging:**

```json
{
  "id": "ses_XXX",
  "title": "Discriminator: Task description",
  "metadata": {
    "dualPairAgent": "discriminator",
    "dualPairStep": 2,
    "dualPairComplete": true,
    "completionSummary": "..."
  }
}
```

**Messages tagged with agent:**
```json
{
  "id": "msg_XXX",
  "role": "assistant",
  "parts": [...],
  "metadata": {
    "dualPairAgent": "executor",  // or "discriminator"
    "dualPairStep": 1
  }
}
```

**Perspective transformation at runtime:**
- `DualPairPerspective.transformForExecutor(messages)` - discriminator's messages become "user" role
- `DualPairPerspective.transformForDiscriminator(messages)` - executor's messages become "user" role

**Markdown export on every step:**
```typescript
await SessionExport.writeToFile(sessionID, exportPath)
```

Writes to: `.opencode/sessions/{sessionID}.md`

## What We Need to Do for Crow

### Phase 1: Fix Storage Architecture ✅ CRITICAL

**Problem:** We currently create TWO separate sessions (executor + discriminator)
**Solution:** Use ONE session with metadata (like OpenCode)

**Changes needed:**

1. **Modify DualAgentRuntime to use ONE session:**
   ```rust
   // Instead of:
   let executor_session = session_store.create(...);
   let discriminator_session = session_store.create(...);
   
   // Do:
   let session = session_store.create(
       directory,
       None,
       Some(format!("Dual-Pair: {}", task))
   );
   
   // Tag it as dual-pair
   session.metadata = {
       "dualPairAgent": "executor",  // starts with executor
       "dualPairStep": 0,
       "dualPairComplete": false
   }
   ```

2. **Add metadata field to Session:**
   ```rust
   pub struct Session {
       pub id: String,
       pub directory: String,
       pub title: String,
       pub metadata: Option<serde_json::Value>,  // ← Add this
       pub time: SessionTime,
       // ...
   }
   ```

3. **Tag messages with agent:**
   ```rust
   // When adding executor's message:
   message.metadata = json!({
       "dualPairAgent": "executor",
       "dualPairStep": steps
   });
   
   // When adding discriminator's message:
   message.metadata = json!({
       "dualPairAgent": "discriminator", 
       "dualPairStep": steps
   });
   ```

4. **Mark completion in session metadata:**
   ```rust
   // When discriminator calls work_completed:
   session_store.update_metadata(&session_id, json!({
       "dualPairComplete": true,
       "dualPairStep": steps,
       "completionSummary": verdict
   }));
   ```

### Phase 2: Implement Markdown Export

**Create `crow/packages/api/src/session/export.rs`:**

```rust
pub struct SessionExport;

impl SessionExport {
    /// Export session to markdown
    pub async fn to_markdown(
        session_store: &SessionStore,
        session_id: &str
    ) -> Result<String, String> {
        let session = session_store.get(session_id)?;
        let messages = session_store.get_messages(session_id)?;
        
        let mut md = format!("# {}\n\n", session.title);
        md.push_str(&format!("**Session ID:** `{}`\n", session.id));
        md.push_str(&format!("**Created:** {}\n", format_timestamp(session.time.created)));
        md.push_str(&format!("**Project:** {}\n\n", session.directory));
        
        // Dual-pair metadata
        if let Some(metadata) = &session.metadata {
            if metadata.get("dualPairAgent").is_some() {
                md.push_str("## Dual-Pair Session\n\n");
                
                if metadata["dualPairComplete"].as_bool().unwrap_or(false) {
                    md.push_str("✅ **Status:** Completed\n");
                    md.push_str(&format!("**Steps:** {}\n", 
                        metadata["dualPairStep"].as_u64().unwrap_or(0)));
                    
                    if let Some(summary) = metadata["completionSummary"].as_str() {
                        md.push_str(&format!("**Summary:** {}\n", summary));
                    }
                } else {
                    md.push_str("🔄 **Status:** In Progress\n");
                }
                md.push_str("\n");
            }
        }
        
        md.push_str("---\n\n");
        
        // Export each message
        for msg in messages {
            md.push_str(&format_message(&msg));
            md.push_str("\n---\n\n");
        }
        
        Ok(md)
    }
    
    /// Write session export to file
    pub async fn write_to_file(
        session_store: &SessionStore,
        session_id: &str,
        path: &Path
    ) -> Result<(), String> {
        let markdown = Self::to_markdown(session_store, session_id).await?;
        std::fs::write(path, markdown)
            .map_err(|e| format!("Failed to write markdown: {}", e))?;
        Ok(())
    }
}

fn format_message(msg: &MessageWithParts) -> String {
    let role = msg.info.role();
    let timestamp = format_timestamp(msg.info.created());
    
    let agent = if let Some(metadata) = msg.info.metadata() {
        metadata.get("dualPairAgent")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
    } else {
        "unknown"
    };
    
    let mut md = format!("## {} ({})\n\n", 
        if role == "user" { "👤 User" } else { "🤖 Assistant" },
        agent
    );
    md.push_str(&format!("*{}*\n\n", timestamp));
    
    // Token info for assistant
    if role == "assistant" {
        if let Some(tokens) = msg.info.tokens() {
            md.push_str(&format!("**Tokens:** {} in / {} out\n\n", 
                tokens.input, tokens.output));
        }
    }
    
    // Render parts
    for part in &msg.parts {
        match part {
            Part::Text { text, .. } => {
                md.push_str(&format!("{}\n\n", text));
            }
            Part::Tool { tool, state, .. } => {
                md.push_str(&format!("🔧 **{}**\n", tool));
                md.push_str(&format!("```json\n{}\n```\n\n", 
                    serde_json::to_string_pretty(&state).unwrap_or_default()));
            }
            _ => {}
        }
    }
    
    md
}
```

### Phase 3: .crow Directory Structure

**Storage location:**
```
{server_cwd}/.crow/           ← Where crow-serve was started
├── config.json               ← Server config
├── sessions/                 ← Session storage
│   └── {session_id}/
│       ├── session.json      ← Session metadata
│       ├── messages.json     ← All messages
│       └── export.md         ← Markdown export
├── logs/                     ← Server logs
└── .gitignore               ← Auto-generated
```

**Implementation:**

```rust
// In server.rs or new storage.rs module

pub struct CrowStorage {
    root: PathBuf,  // {cwd}/.crow
}

impl CrowStorage {
    pub fn new() -> Result<Self, String> {
        // Get server's current working directory
        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get CWD: {}", e))?;
        
        let root = cwd.join(".crow");
        
        // Create directory structure
        std::fs::create_dir_all(&root)
            .map_err(|e| format!("Failed to create .crow: {}", e))?;
        std::fs::create_dir_all(root.join("sessions"))
            .map_err(|e| format!("Failed to create sessions dir: {}", e))?;
        std::fs::create_dir_all(root.join("logs"))
            .map_err(|e| format!("Failed to create logs dir: {}", e))?;
        
        // Create .gitignore
        let gitignore_path = root.join(".gitignore");
        if !gitignore_path.exists() {
            std::fs::write(&gitignore_path, "*\n")
                .map_err(|e| format!("Failed to create .gitignore: {}", e))?;
        }
        
        Ok(Self { root })
    }
    
    pub fn session_dir(&self, session_id: &str) -> PathBuf {
        self.root.join("sessions").join(session_id)
    }
    
    pub fn session_export_path(&self, session_id: &str) -> PathBuf {
        self.session_dir(session_id).join("export.md")
    }
}
```

### Phase 4: Export on Every Step

**In DualAgentRuntime.run():**

```rust
pub async fn run(
    &self,
    session_id: &str,
    task: &str,
    working_dir: &PathBuf,
) -> Result<DualAgentResult, String> {
    let storage = CrowStorage::new()?;
    
    // Create session with dual-pair metadata
    let session = self.sessions.create(
        working_dir.to_string_lossy().to_string(),
        None,
        Some(format!("Dual-Pair: {}", task)),
    )?;
    
    self.sessions.update_metadata(&session.id, json!({
        "dualPairAgent": "executor",
        "dualPairStep": 0,
        "dualPairComplete": false
    }))?;
    
    let mut steps = 0;
    let mut current_agent = "executor";
    
    // EXPORT INITIAL STATE
    let export_path = storage.session_export_path(&session.id);
    SessionExport::write_to_file(&self.sessions, &session.id, &export_path).await?;
    
    while steps < MAX_STEPS {
        if current_agent == "executor" {
            // Execute with build agent
            let response = executor_exec.execute_turn(...).await?;
            
            // Tag message with metadata
            self.sessions.add_message_with_metadata(
                &session.id,
                response,
                json!({
                    "dualPairAgent": "executor",
                    "dualPairStep": steps
                })
            )?;
            
            // EXPORT AFTER EXECUTOR
            SessionExport::write_to_file(&self.sessions, &session.id, &export_path).await?;
            
            current_agent = "discriminator";
            
        } else {
            // Execute with discriminator
            let response = discriminator_exec.execute_turn(...).await?;
            
            // Check for work_completed
            let completed = check_work_completed(&response.parts);
            
            // Tag message
            self.sessions.add_message_with_metadata(
                &session.id,
                response,
                json!({
                    "dualPairAgent": "discriminator",
                    "dualPairStep": steps
                })
            )?;
            
            if completed {
                // Mark session complete
                self.sessions.update_metadata(&session.id, json!({
                    "dualPairComplete": true,
                    "dualPairStep": steps,
                    "completionSummary": extract_summary(&response.parts)
                }))?;
                
                // EXPORT FINAL STATE
                SessionExport::write_to_file(&self.sessions, &session.id, &export_path).await?;
                
                break;
            }
            
            // EXPORT AFTER DISCRIMINATOR
            SessionExport::write_to_file(&self.sessions, &session.id, &export_path).await?;
            
            current_agent = "executor";
            steps += 1;
        }
    }
    
    Ok(DualAgentResult { ... })
}
```

## Implementation Order

1. **Add metadata field to Session** ✅ Start here
2. **Modify DualAgentRuntime to use ONE session**
3. **Implement SessionExport module**
4. **Create CrowStorage for .crow directory**
5. **Add export calls after each turn**
6. **Test with real dual-agent run**

## Expected Result

After running:
```bash
curl -X POST http://localhost:7070/session/dual \
  -d '{"task": "Create hello.txt", "directory": "/tmp/test"}'
```

You get:
```
/tmp/test/.crow/
└── sessions/
    └── ses-abc123/
        ├── session.json
        ├── messages.json
        └── export.md        ← THIS IS YOUR LANGFUSE!
```

`export.md` contains:
```markdown
# Dual-Pair: Create hello.txt

**Session ID:** `ses-abc123`
**Created:** 2025-11-16T17:30:00Z
**Project:** /tmp/test

## Dual-Pair Session

✅ **Status:** Completed
**Steps:** 2
**Summary:** Successfully created hello.txt with correct content...

---

## 🤖 Assistant (executor)

*2025-11-16T17:30:05Z*

**Tokens:** 150 in / 75 out

I'll create the hello.txt file...

🔧 **bash**
```json
{
  "command": "echo 'Hello World' > hello.txt"
}
```

---

## 🤖 Assistant (discriminator)

*2025-11-16T17:30:12Z*

**Tokens:** 120 in / 90 out

Let me verify the file...

🔧 **bash**
```json
{
  "command": "cat hello.txt"
}
```

Perfect! The file is created correctly.

🔧 **work_completed**
```json
{
  "ready": true
}
```

---
```

This is your Langfuse-style telemetry - no external service needed!
