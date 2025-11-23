//! Crow main binary - serves web UI + API from anywhere
//! Like `jupyter lab` or `opencode serve -p 4096`

use api::create_router_with_storage;
use axum::{
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    Router,
};
use rust_embed::RustEmbed;
use tower_http::cors::CorsLayer;

#[derive(RustEmbed)]
#[folder = "../../target/dx/web/release/web/public/"]
struct Assets;

struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Assets::get(path.as_str()) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        }
    }
}

async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == "index.html" {
        return index_html().await;
    }

    StaticFile(path.to_string()).into_response()
}

async fn index_html() -> Response {
    match Assets::get("index.html") {
        Some(content) => Html(content.data).into_response(),
        None => {
            // Fallback: serve a basic HTML that loads the WASM
            Html(
                r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Crow</title>
    <link rel="stylesheet" href="/assets/main.css">
</head>
<body>
    <div id="main"></div>
    <script type="module">
        import init from '/assets/web.js';
        init();
    </script>
</body>
</html>"#,
            )
            .into_response()
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse port from args
    let port = std::env::args()
        .nth(1)
        .and_then(|arg| {
            if arg == "-p" || arg == "--port" {
                std::env::args().nth(2).and_then(|p| p.parse().ok())
            } else {
                arg.parse().ok()
            }
        })
        .unwrap_or(8080);

    println!("Initializing storage...");

    // Create the API router
    let api_router = match create_router_with_storage().await {
        Ok(router) => {
            println!("Storage initialized successfully");
            router
        }
        Err(e) => {
            eprintln!("Failed to initialize storage: {}", e);
            std::process::exit(1);
        }
    };

    // Combine API routes with static file serving
    let app = Router::new()
        .merge(api_router)
        .fallback(static_handler)
        .layer(CorsLayer::permissive());

    let addr = format!("127.0.0.1:{}", port);
    println!("Binding to {}...", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", addr));

    tracing::info!("🚀 Crow server listening on http://{}", addr);
    println!("🚀 Crow server started!");
    println!("   Web UI:  http://{}", addr);
    println!("   API:     http://{}/session", addr);
    println!();
    println!("Press Ctrl+C to stop");

    // Run the server
    axum::serve(listener, app).await.expect("Server error");
}
