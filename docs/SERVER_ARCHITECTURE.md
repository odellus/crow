# Crow Server Architecture

## Current Setup (Phase 1)

### Two Servers Approach

We're running two separate servers that work together:

#### 1. crow-serve (Port 8080) - REST API
```bash
cd crow/packages/api
cargo run --bin crow-serve --features server
```

**Purpose:** OpenCode-compatible REST API
- `GET /session` - List sessions ✅
- `POST /session` - Create session ✅
- `GET /session/:id` - Get session ✅
- `DELETE /session/:id` - Delete session ✅
- `PATCH /session/:id` - Update session ✅

**Technology:** Pure Axum server

#### 2. dx serve (Port 3000) - Dioxus UI
```bash
cd crow/packages/web
dx serve
```

**Purpose:** Frontend UI + Dioxus server functions
- Serves the Dioxus web app
- Server functions like `/api/echo`, `/api/send_message`
- Hot reloading for development

**Technology:** Dioxus fullstack (Axum under the hood)

---

## Why Two Servers?

### Current State
- **Dioxus server functions** are always POST methods
- **OpenCode API** uses RESTful GET/POST/PATCH/DELETE
- Mixing them requires custom Axum configuration

### Benefits of Separation
1. ✅ **Clean REST API** - Matches OpenCode exactly
2. ✅ **Working NOW** - No complex integration needed
3. ✅ **Easy testing** - Can test API independently
4. ✅ **Development speed** - UI and API can evolve separately

### Drawbacks
- Need to run two processes
- Two different ports
- Slight overhead

---

## Usage

### For API Development/Testing
```bash
# Start the REST API
cd crow/packages/api
cargo run --bin crow-serve --features server

# Test it
curl http://localhost:8080/session
curl -X POST http://localhost:8080/session -d '{"title":"test"}'
```

### For UI Development
```bash
# Start the Dioxus app
cd crow/packages/web
dx serve

# Open browser
# http://localhost:3000
```

### For Full Stack Development
```bash
# Terminal 1: API server
cd crow/packages/api && cargo run --bin crow-serve --features server

# Terminal 2: UI server
cd crow/packages/web && dx serve
```

The UI can call the REST API at `http://localhost:8080`.

---

## Future Integration (Phase 2+)

### Option: Merge into Single Server

Using Dioxus 0.7's `LaunchBuilder`, we can create a custom launcher that:

```rust
use dioxus::prelude::*;
use api::create_router;

fn main() {
    LaunchBuilder::custom(custom_server)
        .launch(App);
}

async fn custom_server(cfg: ServeConfig) -> std::io::Result<()> {
    use axum::Router;
    use dioxus::prelude::*;
    
    // Create base Dioxus router
    let mut router = Router::new()
        .serve_dioxus_application(cfg, App);
    
    // Add custom REST routes
    let rest_api = create_router();
    router = router.merge(rest_api);
    
    // Serve on single port
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await
}
```

**Benefits:**
- ✅ Single server on one port
- ✅ Shared state between UI and API
- ✅ Simpler deployment

**When to do this:**
- After core functionality is working
- When deployment becomes priority
- When shared state is needed

---

## Current Progress

### ✅ Working
- REST API endpoints (session CRUD)
- Response format matches OpenCode
- HTTP methods match (GET, POST, etc.)
- Clean separation of concerns

### 🔨 Next Steps
1. Add message storage to SessionStore
2. Implement message endpoints
3. Add tool system
4. Build agent executor

### 🎯 Future
- Merge servers with LaunchBuilder
- Add WebSocket for real-time updates
- Implement SSE for streaming responses

---

## Testing Both Servers

```bash
# Start both
cd crow/packages/api && cargo run --bin crow-serve --features server &
cd crow/packages/web && dx serve &

# Test REST API
curl http://localhost:8080/session

# Test UI
open http://localhost:3000

# Test integration
# UI calls API at localhost:8080
```

---

## Comparison with OpenCode

| Feature | OpenCode | Crow (current) |
|---------|----------|----------------|
| REST API | ✅ Single server | ✅ crow-serve (8080) |
| Web UI | ✅ Built-in | ✅ dx serve (3000) |
| Port | 4096 | 8080 (API) + 3000 (UI) |
| Technology | TypeScript/Hono | Rust/Axum + Dioxus |
| API compatibility | - | ✅ Matches |

---

## Decision: Keep Two Servers for Now

**Rationale:**
1. It works
2. Clean separation
3. Easy to test
4. Can merge later with LaunchBuilder

**Next milestone:** Get full functionality working with two servers, then optimize.
