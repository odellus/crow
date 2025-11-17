//! Crow server binary - mirrors `opencode serve`
//! Runs an HTTP server with OpenCode-compatible REST API

use api::create_router_with_storage;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

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

    println!("Binding to port 7070...");

    // Bind to port 7070
    let listener = tokio::net::TcpListener::bind("127.0.0.1:7070")
        .await
        .expect("Failed to bind to port 7070");

    tracing::info!("crow server listening on http://127.0.0.1:7070");
    println!("Server started on http://127.0.0.1:7070");

    // Run the server
    axum::serve(listener, app).await.expect("Server error");
}
