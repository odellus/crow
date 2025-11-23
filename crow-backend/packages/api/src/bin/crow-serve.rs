//! Crow server binary - mirrors `opencode serve`
//! Runs an HTTP server with OpenCode-compatible REST API

use api::create_router_with_storage;
use std::env;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mut port = 7070u16;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--port" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().unwrap_or(7070);
                    i += 2;
                } else {
                    eprintln!("Error: --port requires a value");
                    std::process::exit(1);
                }
            }
            _ => i += 1,
        }
    }

    println!("Initializing storage...");

    // Create the router with storage initialization
    let app = match create_router_with_storage().await {
        Ok(app) => {
            println!("Storage initialized successfully");
            app
        }
        Err(e) => {
            eprintln!("Failed to initialize storage: {}", e);
            std::process::exit(1);
        }
    };

    let addr = format!("127.0.0.1:{}", port);
    println!("Binding to port {}...", port);

    // Bind to specified port
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect(&format!("Failed to bind to port {}", port));

    tracing::info!("crow server listening on http://{}", addr);
    println!("Server started on http://{}", addr);

    // Run the server
    axum::serve(listener, app).await.expect("Server error");
}
