//! Crow unified binary - API server + Web UI
//! Like `opencode` - run from your project directory

use api::create_router_with_storage;
use api::global::GlobalPaths;

#[tokio::main]
async fn main() {
    // Initialize global paths (like OpenCode uses XDG)
    let global_paths = GlobalPaths::new();

    // Create all necessary directories
    if let Err(e) = global_paths.init() {
        eprintln!("Failed to initialize directories: {}", e);
        std::process::exit(1);
    }

    // Load config from XDG config directory
    // Look for ~/.config/crow/config or ~/.config/crow/.env
    let config_file = global_paths.config.join("config");
    let env_file = global_paths.config.join(".env");

    if config_file.exists() {
        let _ = dotenvy::from_path(&config_file);
    } else if env_file.exists() {
        let _ = dotenvy::from_path(&env_file);
    }

    // Also allow environment variables to override
    let _ = dotenvy::dotenv();

    // Set up logging to file
    let log_file = global_paths.log.join(format!(
        "crow-{}.log",
        chrono::Local::now().format("%Y-%m-%d")
    ));

    let log_file_handle = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .expect("Failed to open log file");

    // Check for verbose mode from env or arg
    let verbose = std::env::var("CROW_VERBOSE").is_ok()
        || std::env::args().any(|arg| arg == "--verbose" || arg == "-v");

    // Initialize tracing with file output
    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::sync::Mutex::new(log_file_handle))
        .with_ansi(false);

    if verbose {
        subscriber.with_max_level(tracing::Level::DEBUG).init();
        println!("🔍 Verbose mode enabled - logging everything!");
    } else {
        subscriber.with_max_level(tracing::Level::INFO).init();
    }

    // Get current working directory - this is the project directory
    let cwd = std::env::current_dir().expect("Failed to get current directory");
    println!("🦅 Crow starting in: {}", cwd.display());
    println!("📁 Config: {}", global_paths.config.display());
    println!("📝 Logs: {}", global_paths.log.display());

    println!("Initializing storage...");

    // Create the API router with storage initialization
    let api_router = match create_router_with_storage().await {
        Ok(app) => {
            println!("Storage initialized successfully");
            app
        }
        Err(e) => {
            eprintln!("Failed to initialize storage: {}", e);
            std::process::exit(1);
        }
    };

    // TODO: Integrate Dioxus web UI here
    // For now, just serve the API
    // Will add web UI routes when we build the actual UI

    println!("Starting Crow server...");
    println!("  API: http://127.0.0.1:7070");
    println!("  Web UI: http://127.0.0.1:7070 (coming soon)");

    // Bind to port 7070
    let listener = tokio::net::TcpListener::bind("127.0.0.1:7070")
        .await
        .expect("Failed to bind to port 7070");

    tracing::info!("Crow listening on http://127.0.0.1:7070");

    // Run the server
    axum::serve(listener, api_router)
        .await
        .expect("Server error");
}
