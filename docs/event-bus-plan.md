# Crow Global Event Bus Implementation Plan

## Overview

Implement a global event bus with SSE `/event` endpoint to match OpenCode's architecture. This replaces the current per-request streaming with a global event stream that all clients can subscribe to.

## Current State

**What crow has:**
- Per-request SSE streaming in `/session/{id}/message/stream`
- Events only flow within a single request/response cycle
- No global event broadcasting

**What OpenCode has:**
- Global Bus with publish/subscribe pattern
- `/event` SSE endpoint that streams all events
- `/global/event` for cross-directory events
- Type-safe events with Zod schemas

## Implementation Plan

### Phase 1: Event Bus Core (2-3 hours)

#### 1.1 Create Event Bus Module
**File:** `crow/packages/api/src/bus/mod.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::broadcast;

/// Event payload that gets broadcast
#[derive(Clone, Debug, serde::Serialize)]
pub struct Event {
    #[serde(rename = "type")]
    pub event_type: String,
    pub properties: serde_json::Value,
}

/// Global event bus with broadcast channel
pub struct EventBus {
    /// Broadcast channel for events (1000 event buffer)
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event_type: &str, properties: serde_json::Value) {
        let event = Event {
            event_type: event_type.to_string(),
            properties,
        };
        // Ignore send errors (no subscribers)
        let _ = self.sender.send(event);
    }

    /// Subscribe to events, returns a receiver
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}

// Global singleton
lazy_static::lazy_static! {
    pub static ref BUS: EventBus = EventBus::new();
}

/// Publish helper function
pub fn publish(event_type: &str, properties: serde_json::Value) {
    BUS.publish(event_type, properties);
}

/// Subscribe helper function
pub fn subscribe() -> broadcast::Receiver<Event> {
    BUS.subscribe()
}
```

#### 1.2 Define Event Types
**File:** `crow/packages/api/src/bus/events.rs`

```rust
/// Event type constants matching OpenCode
pub mod events {
    // Session events
    pub const SESSION_CREATED: &str = "session.created";
    pub const SESSION_UPDATED: &str = "session.updated";
    pub const SESSION_DELETED: &str = "session.deleted";
    pub const SESSION_ERROR: &str = "session.error";
    
    // Session status
    pub const SESSION_STATUS: &str = "session.status";
    pub const SESSION_IDLE: &str = "session.idle";
    
    // Message events
    pub const MESSAGE_UPDATED: &str = "message.updated";
    pub const MESSAGE_REMOVED: &str = "message.removed";
    pub const MESSAGE_PART_UPDATED: &str = "message.part.updated";
    pub const MESSAGE_PART_REMOVED: &str = "message.part.removed";
    
    // File events
    pub const FILE_EDITED: &str = "file.edited";
    
    // Server events
    pub const SERVER_CONNECTED: &str = "server.connected";
}
```

### Phase 2: SSE Endpoint (1-2 hours)

#### 2.1 Add /event Endpoint
**File:** `crow/packages/api/src/server.rs`

```rust
/// GET /event - Global SSE event stream
async fn event_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = crate::bus::subscribe();
    
    let stream = async_stream::stream! {
        // Send connected event
        yield Ok(Event::default()
            .event("message")
            .data(serde_json::json!({
                "type": "server.connected",
                "properties": {}
            }).to_string()));
        
        // Stream all events
        loop {
            match rx.recv().await {
                Ok(event) => {
                    yield Ok(Event::default()
                        .event("message")
                        .data(serde_json::to_string(&event).unwrap_or_default()));
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // Client fell behind, skip missed events
                    tracing::warn!("Event stream lagged by {} events", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    };
    
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
```

#### 2.2 Add Route
```rust
.route("/event", get(event_stream))
```

### Phase 3: Integrate Event Publishing (2-3 hours)

#### 3.1 Session Events
**File:** `crow/packages/api/src/session/store.rs`

Add event publishing to session operations:

```rust
// In create()
crate::bus::publish("session.created", serde_json::json!({
    "info": session
}));

// In update_with()
crate::bus::publish("session.updated", serde_json::json!({
    "info": updated_session
}));

// In delete()
crate::bus::publish("session.deleted", serde_json::json!({
    "info": session
}));
```

#### 3.2 Message Events
**File:** `crow/packages/api/src/session/store.rs`

```rust
// In add_message()
crate::bus::publish("message.updated", serde_json::json!({
    "info": message.info
}));
```

#### 3.3 Executor Events
**File:** `crow/packages/api/src/agent/executor.rs`

Replace direct event_tx sending with bus publishing:

```rust
// Text delta
crate::bus::publish("message.part.updated", serde_json::json!({
    "part": {
        "type": "text",
        "id": text_part_id,
        "session_id": session_id,
        "message_id": message_id,
        "text": accumulated_text,
    },
    "delta": text_delta,
}));

// Tool part
crate::bus::publish("message.part.updated", serde_json::json!({
    "part": tool_part,
}));

// Session status
crate::bus::publish("session.status", serde_json::json!({
    "sessionID": session_id,
    "status": { "type": "busy" }
}));
```

#### 3.4 Session Lock Events
**File:** `crow/packages/api/src/session/lock.rs`

```rust
// When lock acquired
crate::bus::publish("session.status", serde_json::json!({
    "sessionID": session_id,
    "status": { "type": "busy" }
}));

// When lock released
crate::bus::publish("session.idle", serde_json::json!({
    "sessionID": session_id,
}));
```

### Phase 4: Update Streaming Endpoint (1 hour)

The existing `/session/{id}/message/stream` endpoint should continue to work for backwards compatibility, but now it can also publish to the global bus.

**Option A:** Keep both - per-request streaming AND global bus
**Option B:** Deprecate per-request, use only global bus

Recommendation: **Option A** for backwards compatibility during transition.

### Phase 5: Testing (1-2 hours)

#### 5.1 Unit Tests
```rust
#[tokio::test]
async fn test_event_bus_publish_subscribe() {
    let mut rx = crate::bus::subscribe();
    
    crate::bus::publish("test.event", serde_json::json!({"foo": "bar"}));
    
    let event = rx.recv().await.unwrap();
    assert_eq!(event.event_type, "test.event");
}
```

#### 5.2 Integration Test
Create `test-event-bus.sh`:
- Start server
- Connect to /event endpoint with curl
- Create session, verify event received
- Send message, verify message.part.updated events
- Test multiple concurrent clients

## Event Types Reference

| Event Type | Properties | When Published |
|------------|------------|----------------|
| `server.connected` | `{}` | SSE connection established |
| `session.created` | `{ info: Session }` | Session created |
| `session.updated` | `{ info: Session }` | Session modified |
| `session.deleted` | `{ info: Session }` | Session deleted |
| `session.status` | `{ sessionID, status }` | Lock acquired/processing |
| `session.idle` | `{ sessionID }` | Lock released |
| `message.updated` | `{ info: Message }` | Message added/modified |
| `message.part.updated` | `{ part, delta? }` | Part created/streaming |
| `file.edited` | `{ file: string }` | File modified by tool |

## Dependencies to Add

```toml
# Cargo.toml
lazy_static = "1.4"
async-stream = "0.3"
```

## Migration Path

1. Implement bus without breaking existing streaming
2. Add /event endpoint
3. Integrate publishing in key locations
4. Test with frontend
5. Optionally deprecate per-request streaming

## Timeline

| Phase | Time | Description |
|-------|------|-------------|
| 1 | 2-3h | Event bus core module |
| 2 | 1-2h | SSE endpoint |
| 3 | 2-3h | Integrate publishing |
| 4 | 1h | Update streaming |
| 5 | 1-2h | Testing |
| **Total** | **7-11h** | Complete implementation |

## Success Criteria

- [ ] `/event` endpoint returns SSE stream
- [ ] `server.connected` sent on connect
- [ ] Session CRUD emits events
- [ ] Message streaming emits `message.part.updated`
- [ ] Multiple clients receive same events
- [ ] Events include proper payloads matching OpenCode schema
- [ ] Integration test passes
