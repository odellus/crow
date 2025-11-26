//! Crow CLI - Full observability streaming interface for agent interaction
//!
//! Color scheme (Purple + Green theme):
//!   🟪 Purple  - Tool names, headers, branding
//!   🟩 Green   - Success, completions, positive output
//!   🟦 Blue    - Agent thinking/reasoning
//!   🟨 Yellow  - Warnings, in-progress states
//!   🟥 Red     - Errors
//!   ⬜ White   - Response text, neutral content
//!
//! Usage:
//!   crow-cli chat "your message"           - Full verbose streaming output
//!   crow-cli chat --json "message"         - JSON output (no streaming)
//!   crow-cli chat --quiet "message"        - Minimal output (just response)

use colored::{ColoredString, Colorize};

// Brand colors - consistent purple/green theme
fn purple(s: &str) -> ColoredString {
    s.truecolor(138, 43, 226) // BlueViolet
}

fn purple_bold(s: &str) -> ColoredString {
    s.truecolor(138, 43, 226).bold()
}

fn light_purple(s: &str) -> ColoredString {
    s.truecolor(180, 130, 255) // Lighter purple for secondary elements
}

fn mint_green(s: &str) -> ColoredString {
    s.truecolor(0, 255, 170) // Mint green for success
}

fn soft_green(s: &str) -> ColoredString {
    s.truecolor(130, 220, 130) // Softer green for output
}

fn lime_green(s: &str) -> ColoredString {
    s.truecolor(180, 255, 100) // Lime green for response text
}

fn thinking_purple(s: &str) -> ColoredString {
    s.truecolor(147, 112, 219) // Medium purple for thinking - softer, contemplative
}

fn dim_purple(s: &str) -> ColoredString {
    s.truecolor(120, 100, 160) // Dimmed purple for secondary/meta info
}
use crow_core::{
    agent::{AgentExecutor, ExecutionEvent},
    global::GlobalPaths,
    session::{MessageWithParts, SessionLockManager, SessionStore},
    storage::CrowStorage,
    types::{Message, MessageTime, Part, ToolState},
    AgentRegistry, ProviderClient, ProviderConfig, ToolRegistry,
};
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

#[derive(Clone, Copy, PartialEq)]
enum OutputMode {
    Verbose, // Default - show everything
    Quiet,   // Just final response
    Json,    // JSON output, no streaming
}

#[tokio::main]
async fn main() {
    // Initialize logging - quiet by default for clean streaming output
    let log_filter = env::var("RUST_LOG").unwrap_or_else(|_| "warn".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_writer(io::stderr)
        .init();

    // Ensure XDG directories exist
    let paths = GlobalPaths::new();
    if let Err(e) = paths.init() {
        eprintln!("{}", format!("Failed to init directories: {}", e).red());
    }

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "repl" => {
            let session_id = args.get(2).map(|s| s.as_str());
            run_repl(session_id).await;
        }
        "chat" => {
            let (session_id, message, mode) = parse_chat_args(&args[2..]);
            if message.is_empty() {
                eprintln!(
                    "{}",
                    "Usage: crow-cli chat [--json|--quiet] [--session <id>] \"message\"".yellow()
                );
                return;
            }
            chat_streaming(session_id.as_deref(), &message, mode).await;
        }
        "sessions" | "list" => {
            list_sessions().await;
        }
        "session" => {
            if args.len() < 3 {
                eprintln!(
                    "{}",
                    "Usage: crow-cli session <info|history|todo> <session-id>".yellow()
                );
                return;
            }
            match args[2].as_str() {
                "info" => {
                    if args.len() < 4 {
                        eprintln!("{}", "Usage: crow-cli session info <session-id>".yellow());
                        return;
                    }
                    session_info(&args[3]).await;
                }
                "history" => {
                    if args.len() < 4 {
                        eprintln!(
                            "{}",
                            "Usage: crow-cli session history <session-id>".yellow()
                        );
                        return;
                    }
                    session_history(&args[3]).await;
                }
                "todo" => {
                    if args.len() < 4 {
                        eprintln!("{}", "Usage: crow-cli session todo <session-id>".yellow());
                        return;
                    }
                    session_todo(&args[3]).await;
                }
                _ => {
                    eprintln!(
                        "{}",
                        "Usage: crow-cli session <info|history|todo> <session-id>".yellow()
                    );
                }
            }
        }
        "snapshot" => {
            if args.len() < 3 {
                eprintln!(
                    "{}",
                    "Usage: crow-cli snapshot <list|show|diff> [project-id]".yellow()
                );
                return;
            }
            match args[2].as_str() {
                "list" => {
                    snapshot_list().await;
                }
                "show" => {
                    let project_id = args.get(3).map(|s| s.as_str());
                    snapshot_show(project_id).await;
                }
                "diff" => {
                    let project_id = args.get(3).map(|s| s.as_str());
                    snapshot_diff(project_id).await;
                }
                _ => {
                    eprintln!(
                        "{}",
                        "Usage: crow-cli snapshot <list|show|diff> [project-id]".yellow()
                    );
                }
            }
        }
        "new" => {
            let title = args.get(2).map(|s| s.as_str());
            create_session(title).await;
        }
        "messages" => {
            if args.len() < 3 {
                eprintln!("{}", "Usage: crow-cli messages <session-id>".yellow());
                return;
            }
            get_messages(&args[2]).await;
        }
        "paths" => {
            show_paths();
        }
        "logs" => {
            let count = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(20);
            show_logs(count);
        }
        "prompt" => {
            let agent = args.get(2).map(|s| s.as_str()).unwrap_or("build");
            dump_prompt(agent).await;
        }
        "-h" | "--help" | "help" => {
            print_usage();
        }
        _ => {
            // Treat unknown command as chat message
            let message = args[1..].join(" ");
            chat_streaming(None, &message, OutputMode::Verbose).await;
        }
    }
}

fn print_banner() {
    // ASCII art banner - only shown in REPL mode
    const BANNER: &str = r#"
 ▄████▄   ██▀███   ▒█████   █     █░
▒██▀ ▀█  ▓██ ▒ ██▒▒██▒  ██▒▓█░ █ ░█░
▒▓█    ▄ ▓██ ░▄█ ▒▒██░  ██▒▒█░ █ ░█
▒▓▓▄ ▄██▒▒██▀▀█▄  ▒██   ██░░█░ █ ░█
▒ ▓███▀ ░░██▓ ▒██▒░ ████▓▒░░░██▒██▓
░ ░▒ ▒  ░░ ▒▓ ░▒▓░░ ▒░▒░▒░ ░ ▓░▒ ▒
  ░  ▒     ░▒ ░ ▒░  ░ ▒ ▒░   ▒ ░ ░
░          ░░   ░ ░ ░ ░ ▒    ░   ░
░ ░         ░         ░ ░      ░
░                                   "#;

    for line in BANNER.lines() {
        if !line.is_empty() {
            // Blue-leaning purple (more blue, no pink)
            println!("{}", line.truecolor(100, 60, 180));
        }
    }
}

fn print_usage() {
    let paths = GlobalPaths::new();
    println!(
        "{}",
        "Crow CLI - Full Observability Agent Interface"
            .cyan()
            .bold()
    );
    println!();
    println!("{}", "USAGE:".yellow());
    println!("  crow-cli repl [session-id]           Interactive REPL mode");
    println!("  crow-cli chat \"message\"              Full verbose streaming (default)");
    println!("  crow-cli chat --quiet \"msg\"         Just the final response");
    println!("  crow-cli chat --json \"msg\"          JSON output, no streaming");
    println!("  crow-cli chat --session ID \"msg\"    Send to specific session");
    println!();
    println!("{}", "SESSION COMMANDS:".yellow());
    println!("  crow-cli new [title]                 Create new session");
    println!("  crow-cli sessions                    List all sessions");
    println!("  crow-cli session info <id>           Detailed session info");
    println!("  crow-cli session history <id>        Message timeline with timestamps");
    println!("  crow-cli session todo <id>           Todo list state for session");
    println!("  crow-cli messages <id>               Show messages (legacy, use session history)");
    println!();
    println!("{}", "SNAPSHOT COMMANDS:".yellow());
    println!("  crow-cli snapshot list               List all project snapshots");
    println!("  crow-cli snapshot show [project]     Show snapshot details");
    println!("  crow-cli snapshot diff [project]     Show file changes");
    println!();
    println!("{}", "DEBUG COMMANDS:".yellow());
    println!("  crow-cli logs [count]                Show recent agent logs");
    println!("  crow-cli prompt [agent]              Dump full system prompt");
    println!("  crow-cli paths                       Show storage paths");
    println!();
    println!("{}", "COLOR LEGEND:".yellow());
    println!(
        "  {}  Tool names, headers, branding",
        purple_bold("🟪 Purple")
    );
    println!(
        "  {}  Agent thinking/reasoning",
        thinking_purple("🔮 Thinking")
    );
    println!("  {}  Success, completions, output", mint_green("🟩 Green"));
    println!("  {}  Warnings, in-progress", "🟨 Yellow".yellow());
    println!("  {}  Errors", "🟥 Red".red());
    println!();
    println!("{}", "PATHS:".yellow());
    println!("  Storage: {}", paths.data.display());
    println!("  Logs:    {}", paths.state.join("logs").display());
    println!();
    println!("{}", "ENVIRONMENT:".yellow());
    println!("  ANTHROPIC_API_KEY    Use Anthropic Claude");
    println!("  OPENAI_API_KEY       Use OpenAI");
    println!("  RUST_LOG=debug       Enable debug logging");
}

fn parse_chat_args(args: &[String]) -> (Option<String>, String, OutputMode) {
    let mut session_id = None;
    let mut mode = OutputMode::Verbose;
    let mut message_parts = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--session" | "-s" => {
                if i + 1 < args.len() {
                    session_id = Some(args[i + 1].clone());
                    i += 2;
                    continue;
                }
            }
            "--json" | "-j" => {
                mode = OutputMode::Json;
                i += 1;
                continue;
            }
            "--quiet" | "-q" => {
                mode = OutputMode::Quiet;
                i += 1;
                continue;
            }
            _ => {
                message_parts.push(args[i].clone());
            }
        }
        i += 1;
    }

    (session_id, message_parts.join(" "), mode)
}

fn show_paths() {
    let paths = GlobalPaths::new();
    println!("{}", "Crow Storage Paths:".cyan().bold());
    println!("  {} {}", "Data:".dimmed(), paths.data.display());
    println!("  {} {}", "Config:".dimmed(), paths.config.display());
    println!("  {} {}", "Cache:".dimmed(), paths.cache.display());
    println!("  {} {}", "State:".dimmed(), paths.state.display());

    if let Ok(storage) = CrowStorage::new() {
        println!("  {} {}", "Storage:".dimmed(), storage.root().display());
    }
    println!(
        "  {} {}",
        "Logs:".dimmed(),
        paths.state.join("logs").display()
    );
}

fn show_logs(count: usize) {
    use crow_core::logging::logger;

    let log = logger();
    println!(
        "{}",
        format!("Recent agent executions (last {}):", count)
            .cyan()
            .bold()
    );
    println!();

    for line in log.read_agent_log(count) {
        println!("{}", line);
    }

    println!();
    println!("{} {}", "Log files:".dimmed(), log.log_dir().display());
}

/// Session info - detailed view of a session
async fn session_info(session_id: &str) {
    let store = SessionStore::new();
    if let Err(e) = store.init_sync() {
        eprintln!("{}", format!("Failed to init storage: {}", e).red());
        return;
    }

    match store.get(session_id) {
        Ok(session) => {
            println!("{}", "Session Info".cyan().bold());
            println!("{}", "═".repeat(60).dimmed());
            println!("  {} {}", "ID:".dimmed(), session.id.yellow());
            let title = if session.title.is_empty() {
                "(untitled)"
            } else {
                &session.title
            };
            println!("  {} {}", "Title:".dimmed(), title);
            println!("  {} {}", "Working Dir:".dimmed(), session.directory);
            println!("  {} {}", "Project ID:".dimmed(), session.project_id);

            // Format timestamps
            let created = chrono::DateTime::from_timestamp_millis(session.time.created as i64)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let updated = chrono::DateTime::from_timestamp_millis(session.time.updated as i64)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "never".to_string());

            println!("  {} {}", "Created:".dimmed(), created);
            println!("  {} {}", "Updated:".dimmed(), updated);

            // Get message count
            match store.get_messages(session_id) {
                Ok(messages) => {
                    let user_count = messages
                        .iter()
                        .filter(|m| matches!(m.info, Message::User { .. }))
                        .count();
                    let assistant_count = messages
                        .iter()
                        .filter(|m| matches!(m.info, Message::Assistant { .. }))
                        .count();
                    println!();
                    println!("{}", "Messages".cyan().bold());
                    println!("  {} {}", "Total:".dimmed(), messages.len());
                    println!("  {} {}", "User:".dimmed(), user_count);
                    println!("  {} {}", "Assistant:".dimmed(), assistant_count);

                    // Count tool calls
                    let tool_calls: usize = messages
                        .iter()
                        .flat_map(|m| m.parts.iter())
                        .filter(|p| matches!(p, Part::Tool { .. }))
                        .count();
                    println!("  {} {}", "Tool Calls:".dimmed(), tool_calls);
                }
                Err(e) => {
                    eprintln!("{}", format!("Failed to get messages: {}", e).red());
                }
            }

            // XDG file location
            let paths = GlobalPaths::new();
            let session_file = paths
                .data
                .join("sessions")
                .join(format!("{}.json", session_id));
            println!();
            println!("{}", "Storage".cyan().bold());
            println!("  {} {}", "File:".dimmed(), session_file.display());
            if session_file.exists() {
                if let Ok(meta) = std::fs::metadata(&session_file) {
                    println!("  {} {} bytes", "Size:".dimmed(), meta.len());
                }
            }
        }
        Err(e) => {
            eprintln!("{}", format!("Session not found: {}", e).red());
        }
    }
}

/// Session history - timeline of messages with timestamps
async fn session_history(session_id: &str) {
    let store = SessionStore::new();
    if let Err(e) = store.init_sync() {
        eprintln!("{}", format!("Failed to init storage: {}", e).red());
        return;
    }

    match store.get_messages(session_id) {
        Ok(messages) => {
            println!(
                "{} {}",
                "Session History:".cyan().bold(),
                session_id.yellow()
            );
            println!("{}", "═".repeat(70).dimmed());

            for msg in &messages {
                let (role, id, created) = match &msg.info {
                    Message::User { id, time, .. } => ("USER", id.as_str(), time.created),
                    Message::Assistant { id, time, .. } => ("ASSISTANT", id.as_str(), time.created),
                };

                let timestamp = chrono::DateTime::from_timestamp_millis(created as i64)
                    .map(|dt| dt.format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| "??:??:??".to_string());

                let role_colored = if role == "USER" {
                    mint_green(role).bold()
                } else {
                    purple_bold(role)
                };

                println!();
                println!("{} {} {}", timestamp.dimmed(), role_colored, id.dimmed());

                // Show parts summary
                for part in &msg.parts {
                    match part {
                        Part::Text { text, .. } => {
                            let preview = if text.len() > 100 {
                                format!("{}...", &text[..100])
                            } else {
                                text.clone()
                            };
                            println!("  📝 {}", preview.replace('\n', " "));
                        }
                        Part::Thinking { text, .. } => {
                            let preview = if text.len() > 80 {
                                format!("{}...", &text[..80])
                            } else {
                                text.clone()
                            };
                            println!("  🧠 {}", preview.replace('\n', " ").dimmed());
                        }
                        Part::Tool { tool, state, .. } => {
                            let status = match state {
                                ToolState::Pending { .. } => "⏳",
                                ToolState::Running { .. } => "🔄",
                                ToolState::Completed { .. } => "✅",
                                ToolState::Error { .. } => "❌",
                            };
                            println!("  {} {}", status, tool.cyan(),);
                        }
                        _ => {}
                    }
                }
            }

            println!();
            println!("{}", "═".repeat(70).dimmed());
            println!("{} messages total", messages.len());
        }
        Err(e) => {
            eprintln!("{}", format!("Failed to get messages: {}", e).red());
        }
    }
}

/// Session todo - show todo list state for a session
async fn session_todo(session_id: &str) {
    let store = SessionStore::new();
    if let Err(e) = store.init_sync() {
        eprintln!("{}", format!("Failed to init storage: {}", e).red());
        return;
    }

    match store.get_messages(session_id) {
        Ok(messages) => {
            println!(
                "{} {}",
                "Todo List for Session:".cyan().bold(),
                session_id.yellow()
            );
            println!("{}", "═".repeat(60).dimmed());

            // Find the most recent todowrite/todoread result
            let mut last_todo_state: Option<String> = None;

            for msg in &messages {
                for part in &msg.parts {
                    if let Part::Tool { tool, state, .. } = part {
                        if tool == "todowrite" || tool == "todoread" {
                            if let ToolState::Completed { output, .. } = state {
                                last_todo_state = Some(output.to_string());
                            }
                        }
                    }
                }
            }

            match last_todo_state {
                Some(state) => {
                    // Try to parse as JSON and pretty print
                    if let Ok(todos) = serde_json::from_str::<serde_json::Value>(&state) {
                        if let Some(items) = todos.get("todos").and_then(|t| t.as_array()) {
                            for item in items {
                                let content =
                                    item.get("content").and_then(|c| c.as_str()).unwrap_or("?");
                                let status = item
                                    .get("status")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("pending");

                                let icon = match status {
                                    "completed" => "✅",
                                    "in_progress" => "🔄",
                                    _ => "⬜",
                                };

                                let content_colored = match status {
                                    "completed" => content.dimmed().to_string(),
                                    "in_progress" => content.yellow().to_string(),
                                    _ => content.to_string(),
                                };

                                println!("  {} {}", icon, content_colored);
                            }
                        } else {
                            println!("{}", state);
                        }
                    } else {
                        println!("{}", state);
                    }
                }
                None => {
                    println!("{}", "No todo list found in session".dimmed());
                }
            }
        }
        Err(e) => {
            eprintln!("{}", format!("Failed to get messages: {}", e).red());
        }
    }
}

/// List all project snapshots
async fn snapshot_list() {
    let paths = GlobalPaths::new();
    let snapshots_dir = paths.data.join("snapshots");

    println!("{}", "Project Snapshots".cyan().bold());
    println!("{}", "═".repeat(60).dimmed());
    println!("  {} {}", "Location:".dimmed(), snapshots_dir.display());
    println!();

    if !snapshots_dir.exists() {
        println!("{}", "No snapshots directory found".dimmed());
        return;
    }

    match std::fs::read_dir(&snapshots_dir) {
        Ok(entries) => {
            let mut projects: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            projects.sort_by_key(|e| e.file_name());

            if projects.is_empty() {
                println!("{}", "No project snapshots found".dimmed());
                return;
            }

            for entry in projects {
                let name = entry.file_name();
                let path = entry.path();

                // Check if it's a valid git directory (bare repo - has HEAD file)
                let has_head = path.join("HEAD").exists();
                let has_git_subdir = path.join(".git").exists();
                let status = if has_head || has_git_subdir {
                    "✅"
                } else {
                    "⚠️"
                };

                // Get size (scan the directory itself for bare repos, or .git for normal)
                let scan_dir = if has_head { &path } else { &path.join(".git") };
                let size = if has_head || has_git_subdir {
                    walkdir::WalkDir::new(scan_dir)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter_map(|e| e.metadata().ok())
                        .filter(|m| m.is_file())
                        .map(|m| m.len())
                        .sum::<u64>()
                } else {
                    0
                };

                let size_str = if size > 1_000_000 {
                    format!("{:.1} MB", size as f64 / 1_000_000.0)
                } else if size > 1_000 {
                    format!("{:.1} KB", size as f64 / 1_000.0)
                } else {
                    format!("{} B", size)
                };

                println!(
                    "  {} {} {}",
                    status,
                    name.to_string_lossy().yellow(),
                    size_str.dimmed()
                );
            }
        }
        Err(e) => {
            eprintln!("{}", format!("Failed to read snapshots: {}", e).red());
        }
    }
}

/// Show snapshot details for a project
async fn snapshot_show(project_id: Option<&str>) {
    let paths = GlobalPaths::new();
    let snapshots_dir = paths.data.join("snapshots");

    // If no project specified, use CrowStorage::project_id for current directory
    let project_id = match project_id {
        Some(id) => id.to_string(),
        None => {
            let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            CrowStorage::project_id(&cwd)
        }
    };

    println!(
        "{}",
        format!("Looking for project: {}", project_id).dimmed()
    );

    // Find matching snapshot directory
    let snapshot_dir = if snapshots_dir.join(&project_id).exists() {
        snapshots_dir.join(&project_id)
    } else {
        // Try to find partial match
        match std::fs::read_dir(&snapshots_dir) {
            Ok(entries) => {
                let found = entries
                    .filter_map(|e| e.ok())
                    .find(|e| e.file_name().to_string_lossy().starts_with(&project_id));
                match found {
                    Some(entry) => entry.path(),
                    None => {
                        eprintln!(
                            "{}",
                            format!("No snapshot found for project: {}", project_id).red()
                        );
                        return;
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", format!("Failed to read snapshots: {}", e).red());
                return;
            }
        }
    };

    println!("{}", "Snapshot Details".cyan().bold());
    println!("{}", "═".repeat(60).dimmed());
    println!(
        "  {} {}",
        "Project:".dimmed(),
        snapshot_dir.file_name().unwrap().to_string_lossy().yellow()
    );
    println!("  {} {}", "Path:".dimmed(), snapshot_dir.display());

    // Detect if it's a bare repo (has HEAD) or normal repo (has .git subdir)
    let git_dir = if snapshot_dir.join("HEAD").exists() {
        snapshot_dir.clone() // Bare repo - the dir itself is the git dir
    } else {
        snapshot_dir.join(".git") // Normal repo
    };

    if git_dir.join("HEAD").exists() || git_dir.exists() {
        // Show object count / statistics
        println!();
        println!("{}", "Git Object Storage".cyan().bold());
        match std::process::Command::new("git")
            .args([
                "--git-dir",
                &git_dir.to_string_lossy(),
                "count-objects",
                "-v",
            ])
            .output()
        {
            Ok(output) => {
                let stats = String::from_utf8_lossy(&output.stdout);
                for line in stats.lines() {
                    println!("  {}", line.dimmed());
                }
            }
            Err(_) => {}
        }
    } else {
        println!("{}", "  No .git directory found".red());
    }
}

/// Show diff for a project snapshot
async fn snapshot_diff(project_id: Option<&str>) {
    let paths = GlobalPaths::new();
    let snapshots_dir = paths.data.join("snapshots");

    // If no project specified, use CrowStorage::project_id for current directory
    let project_id = match project_id {
        Some(id) => id.to_string(),
        None => {
            let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            CrowStorage::project_id(&cwd)
        }
    };

    // Find matching snapshot directory
    let snapshot_dir = match std::fs::read_dir(&snapshots_dir) {
        Ok(entries) => {
            let found = entries
                .filter_map(|e| e.ok())
                .find(|e| e.file_name().to_string_lossy().starts_with(&project_id));
            match found {
                Some(entry) => entry.path(),
                None => {
                    eprintln!(
                        "{}",
                        format!("No snapshot found for project: {}", project_id).red()
                    );
                    return;
                }
            }
        }
        Err(e) => {
            eprintln!("{}", format!("Failed to read snapshots: {}", e).red());
            return;
        }
    };

    let git_dir = snapshot_dir.join(".git");
    if !git_dir.exists() {
        eprintln!("{}", "No .git directory in snapshot".red());
        return;
    }

    println!("{}", "Snapshot Diff".cyan().bold());
    println!("{}", "═".repeat(60).dimmed());
    println!(
        "  {} {}",
        "Project:".dimmed(),
        snapshot_dir.file_name().unwrap().to_string_lossy().yellow()
    );
    println!();

    // Show diff between first and last commit (all changes)
    match std::process::Command::new("git")
        .args([
            "--git-dir",
            &git_dir.to_string_lossy(),
            "diff",
            "--stat",
            "--color=always",
            "$(git rev-list --max-parents=0 HEAD)..HEAD",
        ])
        .output()
    {
        Ok(output) => {
            if output.stdout.is_empty() {
                // Try alternative: show all files changed
                match std::process::Command::new("git")
                    .args([
                        "--git-dir",
                        &git_dir.to_string_lossy(),
                        "log",
                        "--stat",
                        "--oneline",
                        "-5",
                    ])
                    .output()
                {
                    Ok(output) => {
                        println!("{}", String::from_utf8_lossy(&output.stdout));
                    }
                    Err(_) => {
                        println!("{}", "No changes to show".dimmed());
                    }
                }
            } else {
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
        }
        Err(e) => {
            eprintln!("{}", format!("Failed to get diff: {}", e).red());
        }
    }
}

/// Interactive REPL mode - the real developer experience
async fn run_repl(session_id: Option<&str>) {
    use rustyline::error::ReadlineError;
    use rustyline::DefaultEditor;

    let working_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Initialize store
    let store = SessionStore::new();
    if let Err(e) = store.init_sync() {
        eprintln!("{}", format!("Failed to init storage: {}", e).red());
        return;
    }

    // Get or create session
    let session = match session_id {
        Some(id) => match store.get(id) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", format!("Session not found: {}", e).red());
                return;
            }
        },
        None => {
            // Create new REPL session
            store
                .create(
                    working_dir.to_string_lossy().to_string(),
                    None,
                    Some("REPL Session".to_string()),
                )
                .expect("Failed to create session")
        }
    };

    // Determine provider
    let (provider_name, provider_config) = if env::var("ANTHROPIC_API_KEY").is_ok() {
        (
            "anthropic",
            ProviderConfig::custom(
                "anthropic".to_string(),
                "https://api.anthropic.com/v1".to_string(),
                "ANTHROPIC_API_KEY".to_string(),
                "claude-sonnet-4-20250514".to_string(),
            ),
        )
    } else if env::var("OPENAI_API_KEY").is_ok() {
        ("openai", ProviderConfig::openai())
    } else {
        ("moonshot", ProviderConfig::moonshot())
    };

    // Print banner
    println!();
    print_banner();
    println!();
    println!("{} {}", "Session:".dimmed(), session.id.yellow());
    println!(
        "{} {} {}",
        "Provider:".dimmed(),
        provider_name.cyan(),
        format!("({})", provider_config.default_model).dimmed()
    );
    println!(
        "{} {}",
        "Working dir:".dimmed(),
        working_dir.display().to_string().dimmed()
    );
    println!();
    println!(
        "{}",
        "Type your message and press Enter. Ctrl+C to abort. Commands: /exit, /new, /session"
            .dimmed()
    );
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════".dimmed()
    );
    println!();

    // Create readline editor
    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("{}", format!("Failed to init readline: {}", e).red());
            return;
        }
    };

    // Try to load history
    let history_path = GlobalPaths::new().state.join("repl_history.txt");
    let _ = rl.load_history(&history_path);

    // Create provider and executor once (reuse across iterations)
    let provider = match ProviderClient::new(provider_config.clone()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", format!("Failed to create provider: {}", e).red());
            return;
        }
    };

    let session_store = Arc::new(store);
    let lock_manager = Arc::new(SessionLockManager::new());
    let agent_registry = Arc::new(AgentRegistry::new_with_config(&working_dir).await);
    let tool_registry = ToolRegistry::new_with_deps(
        session_store.clone(),
        agent_registry.clone(),
        lock_manager.clone(),
        provider_config,
    )
    .await;

    let mut executor = AgentExecutor::new(
        provider,
        tool_registry,
        session_store.clone(),
        agent_registry,
        lock_manager,
    );

    let mut current_session_id = session.id.clone();

    // REPL loop
    loop {
        let prompt = format!("{}", "crow> ".truecolor(138, 43, 226).bold());

        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(line);

                // Handle commands
                if line.starts_with('/') {
                    match line {
                        "/exit" | "/quit" | "/q" => {
                            println!("{}", "Goodbye! 👋".cyan());
                            break;
                        }
                        "/new" => {
                            // Create new session
                            match session_store.create(
                                working_dir.to_string_lossy().to_string(),
                                None,
                                Some("REPL Session".to_string()),
                            ) {
                                Ok(new_session) => {
                                    current_session_id = new_session.id.clone();
                                    println!(
                                        "{} {}",
                                        "New session:".green(),
                                        current_session_id.yellow()
                                    );
                                }
                                Err(e) => {
                                    eprintln!(
                                        "{}",
                                        format!("Failed to create session: {}", e).red()
                                    );
                                }
                            }
                            continue;
                        }
                        "/session" => {
                            println!(
                                "{} {}",
                                "Current session:".dimmed(),
                                current_session_id.yellow()
                            );
                            continue;
                        }
                        "/help" | "/?" => {
                            println!("{}", "Commands:".yellow());
                            println!("  {} - Exit REPL", "/exit".cyan());
                            println!("  {} - Create new session", "/new".cyan());
                            println!("  {} - Show current session ID", "/session".cyan());
                            println!("  {} - Show this help", "/help".cyan());
                            println!();
                            println!("{}", "During execution:".yellow());
                            println!(
                                "  {} - Interrupt and send new message",
                                "Type + Enter".cyan()
                            );
                            println!("  {} - Abort execution", "Ctrl+C".cyan());
                            println!("  {} - Exit (at prompt)", "Ctrl+D".cyan());
                            continue;
                        }
                        _ => {
                            println!("{}", format!("Unknown command: {}", line).yellow());
                            continue;
                        }
                    }
                }

                // Execute the message
                let start_time = Instant::now();

                // Create user message
                let user_msg_id = format!("msg-user-{}", uuid::Uuid::new_v4());
                let user_parts = vec![Part::Text {
                    id: format!("part-{}", uuid::Uuid::new_v4()),
                    session_id: current_session_id.clone(),
                    message_id: user_msg_id.clone(),
                    text: line.to_string(),
                }];

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                let user_message = MessageWithParts {
                    info: Message::User {
                        id: user_msg_id,
                        session_id: current_session_id.clone(),
                        time: MessageTime {
                            created: now,
                            completed: Some(now),
                        },
                        summary: None,
                        metadata: None,
                    },
                    parts: user_parts.clone(),
                };

                if let Err(e) = session_store.add_message(&current_session_id, user_message) {
                    eprintln!("{}", format!("Failed to add user message: {}", e).red());
                    continue;
                }

                // Create streaming channel
                let (tx, mut rx) = mpsc::unbounded_channel::<ExecutionEvent>();

                // Reset cancellation token for this turn (in case previous was cancelled)
                executor.set_cancellation_token(tokio_util::sync::CancellationToken::new());
                let cancel_token = executor.cancellation_token();

                // Spawn the execution in background
                let session_id_clone = current_session_id.clone();
                let working_dir_clone = working_dir.clone();
                let exec_handle = tokio::spawn({
                    let executor = executor.clone();
                    async move {
                        executor
                            .execute_turn_streaming(
                                &session_id_clone,
                                "build",
                                &working_dir_clone,
                                user_parts,
                                tx,
                            )
                            .await
                    }
                });

                // Stream renderer
                let mut renderer = StreamRenderer::new(OutputMode::Verbose);

                // Process streaming events with interrupt handling
                // User can type during execution - pressing Enter interrupts and sends that message
                let mut interrupt_message: Option<String> = None;

                // Spawn a task to read stdin lines during execution
                let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<String>();
                let stdin_handle = tokio::spawn(async move {
                    use tokio::io::{AsyncBufReadExt, BufReader};
                    let stdin = tokio::io::stdin();
                    let mut reader = BufReader::new(stdin);
                    let mut line_buf = String::new();
                    if reader.read_line(&mut line_buf).await.is_ok() && !line_buf.is_empty() {
                        let _ = stdin_tx.send(line_buf.trim().to_string());
                    }
                });

                loop {
                    tokio::select! {
                        // Handle incoming events
                        event = rx.recv() => {
                            match event {
                                Some(ev) => renderer.handle_event(ev),
                                None => break, // Channel closed, execution done
                            }
                        }
                        // Handle user typing during execution - Enter sends interrupt
                        Some(user_input) = stdin_rx.recv() => {
                            if !user_input.is_empty() {
                                eprintln!();
                                eprintln!("{} {}", "⚡ Interrupting with:".yellow().bold(), user_input.cyan());
                                cancel_token.cancel();
                                interrupt_message = Some(user_input);
                                // Drain remaining events quickly
                                while let Ok(ev) = rx.try_recv() {
                                    renderer.handle_event(ev);
                                }
                                break;
                            }
                        }
                        // Handle Ctrl+C - just abort without follow-up
                        _ = tokio::signal::ctrl_c() => {
                            eprintln!();
                            eprintln!("{}", "^C - Aborting...".yellow().bold());
                            cancel_token.cancel();
                            while let Ok(ev) = rx.try_recv() {
                                renderer.handle_event(ev);
                            }
                            break;
                        }
                    }
                }

                // Clean up stdin reader
                stdin_handle.abort();

                // Wait for execution to complete
                let exec_result = exec_handle.await;

                // If user typed an interrupt message, execute it as a follow-up
                if let Some(follow_up) = interrupt_message {
                    let interrupt_msg_id = format!("msg-user-{}", uuid::Uuid::new_v4());
                    let interrupt_text = format!("[INTERRUPTED] {}", follow_up);
                    let interrupt_parts = vec![Part::Text {
                        id: format!("part-{}", uuid::Uuid::new_v4()),
                        session_id: current_session_id.clone(),
                        message_id: interrupt_msg_id.clone(),
                        text: interrupt_text,
                    }];

                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;

                    let interrupt_msg = MessageWithParts {
                        info: Message::User {
                            id: interrupt_msg_id,
                            session_id: current_session_id.clone(),
                            time: MessageTime {
                                created: now,
                                completed: Some(now),
                            },
                            summary: None,
                            metadata: None,
                        },
                        parts: interrupt_parts.clone(),
                    };

                    if let Err(e) = session_store.add_message(&current_session_id, interrupt_msg) {
                        eprintln!(
                            "{}",
                            format!("Failed to add interrupt message: {}", e).red()
                        );
                    } else {
                        // Execute follow-up turn
                        let (tx2, mut rx2) = mpsc::unbounded_channel::<ExecutionEvent>();
                        executor.set_cancellation_token(tokio_util::sync::CancellationToken::new());

                        let session_id_clone2 = current_session_id.clone();
                        let working_dir_clone2 = working_dir.clone();
                        let exec_handle2 = tokio::spawn({
                            let executor = executor.clone();
                            async move {
                                executor
                                    .execute_turn_streaming(
                                        &session_id_clone2,
                                        "build",
                                        &working_dir_clone2,
                                        interrupt_parts,
                                        tx2,
                                    )
                                    .await
                            }
                        });

                        let mut renderer2 = StreamRenderer::new(OutputMode::Verbose);
                        while let Some(event) = rx2.recv().await {
                            renderer2.handle_event(event);
                        }

                        match exec_handle2.await {
                            Ok(Ok(response)) => {
                                renderer2.finish(
                                    start_time.elapsed(),
                                    &current_session_id,
                                    &response,
                                );
                            }
                            Ok(Err(e)) => {
                                eprintln!("{} {}", "🟥 ERROR:".red().bold(), e.red());
                            }
                            Err(e) => {
                                eprintln!(
                                    "{} {}",
                                    "🟥 TASK ERROR:".red().bold(),
                                    e.to_string().red()
                                );
                            }
                        }
                    }
                } else {
                    // Normal completion - show final stats
                    match exec_result {
                        Ok(Ok(response)) => {
                            renderer.finish(start_time.elapsed(), &current_session_id, &response);
                        }
                        Ok(Err(e)) => {
                            eprintln!();
                            eprintln!("{} {}", "🟥 ERROR:".red().bold(), e.red());
                        }
                        Err(e) => {
                            eprintln!();
                            eprintln!("{} {}", "🟥 TASK ERROR:".red().bold(), e.to_string().red());
                        }
                    }
                }

                println!(); // Blank line between interactions
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "^C - Use /exit to quit".yellow());
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "Goodbye! 👋".cyan());
                break;
            }
            Err(e) => {
                eprintln!("{}", format!("Error: {}", e).red());
                break;
            }
        }
    }

    // Save history
    let _ = rl.save_history(&history_path);
}

/// Streaming chat with full observability
async fn chat_streaming(session_id: Option<&str>, message: &str, mode: OutputMode) {
    let working_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let start_time = Instant::now();

    // Initialize store
    let store = SessionStore::new();
    if let Err(e) = store.init_sync() {
        eprintln!("{}", format!("Failed to init storage: {}", e).red());
        return;
    }

    // Get or create session
    let session = match session_id {
        Some(id) => match store.get(id) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", format!("Session not found: {}", e).red());
                return;
            }
        },
        None => {
            // Use most recent session or create new one
            match store.list(None).ok().and_then(|s| s.first().cloned()) {
                Some(s) => s,
                None => store
                    .create(
                        working_dir.to_string_lossy().to_string(),
                        None,
                        Some("CLI Chat".to_string()),
                    )
                    .expect("Failed to create session"),
            }
        }
    };

    // Create user message
    let user_msg_id = format!("msg-user-{}", uuid::Uuid::new_v4());
    let user_parts = vec![Part::Text {
        id: format!("part-{}", uuid::Uuid::new_v4()),
        session_id: session.id.clone(),
        message_id: user_msg_id.clone(),
        text: message.to_string(),
    }];

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let user_message = MessageWithParts {
        info: Message::User {
            id: user_msg_id,
            session_id: session.id.clone(),
            time: MessageTime {
                created: now,
                completed: Some(now),
            },
            summary: None,
            metadata: None,
        },
        parts: user_parts.clone(),
    };

    if let Err(e) = store.add_message(&session.id, user_message) {
        eprintln!("{}", format!("Failed to add user message: {}", e).red());
        return;
    }

    // Create provider
    let (provider_name, provider_config) = if env::var("ANTHROPIC_API_KEY").is_ok() {
        (
            "anthropic",
            ProviderConfig::custom(
                "anthropic".to_string(),
                "https://api.anthropic.com/v1".to_string(),
                "ANTHROPIC_API_KEY".to_string(),
                "claude-sonnet-4-20250514".to_string(),
            ),
        )
    } else if env::var("OPENAI_API_KEY").is_ok() {
        ("openai", ProviderConfig::openai())
    } else {
        ("moonshot", ProviderConfig::moonshot())
    };

    // Print header (unless quiet/json)
    if mode == OutputMode::Verbose {
        eprintln!();
        eprintln!(
            "{}",
            "═══════════════════════════════════════════════════════════════".dimmed()
        );
        eprintln!(
            "{} {} {}",
            "Session:".dimmed(),
            session.id.yellow(),
            format!("({})", session.title).dimmed()
        );
        eprintln!(
            "{} {} {}",
            "Provider:".dimmed(),
            provider_name.cyan(),
            format!("({})", provider_config.default_model).dimmed()
        );
        eprintln!(
            "{} {}",
            "Working dir:".dimmed(),
            working_dir.display().to_string().dimmed()
        );
        eprintln!(
            "{}",
            "═══════════════════════════════════════════════════════════════".dimmed()
        );
        eprintln!();

        // Show user message
        eprintln!("{}", "▶ USER".white().bold());
        eprintln!("{}", message.white());
        eprintln!();
    }

    let provider = match ProviderClient::new(provider_config.clone()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", format!("Failed to create provider: {}", e).red());
            return;
        }
    };

    // Create executor with full tool registry (including TaskTool for subagents)
    let session_store = Arc::new(store.clone());
    let lock_manager = Arc::new(SessionLockManager::new());

    // Load agents from config (built-in + .crow/agent/*.md)
    let agent_registry = Arc::new(AgentRegistry::new_with_config(&working_dir).await);

    // Create tool registry with TaskTool support
    let tool_registry = ToolRegistry::new_with_deps(
        session_store.clone(),
        agent_registry.clone(),
        lock_manager.clone(),
        provider_config,
    )
    .await;

    let executor = AgentExecutor::new(
        provider,
        tool_registry,
        session_store,
        agent_registry,
        lock_manager,
    );

    // Create streaming channel
    let (tx, mut rx) = mpsc::unbounded_channel::<ExecutionEvent>();

    // Spawn the execution in background
    let session_id_clone = session.id.clone();
    let working_dir_clone = working_dir.clone();
    let exec_handle = tokio::spawn(async move {
        executor
            .execute_turn_streaming(
                &session_id_clone,
                "build",
                &working_dir_clone,
                user_parts,
                tx,
            )
            .await
    });

    // Stream renderer with full observability
    let mut renderer = StreamRenderer::new(mode);

    // Process streaming events
    while let Some(event) = rx.recv().await {
        renderer.handle_event(event);
    }

    // Wait for execution to complete
    match exec_handle.await {
        Ok(Ok(response)) => {
            renderer.finish(start_time.elapsed(), &session.id, &response);
        }
        Ok(Err(e)) => {
            if mode != OutputMode::Json {
                eprintln!();
                eprintln!("{} {}", "🟥 ERROR:".red().bold(), e.red());
            } else {
                println!(
                    "{}",
                    serde_json::json!({
                        "error": e,
                        "session_id": session.id
                    })
                );
            }
            std::process::exit(1);
        }
        Err(e) => {
            if mode != OutputMode::Json {
                eprintln!();
                eprintln!("{} {}", "🟥 TASK ERROR:".red().bold(), e.to_string().red());
            } else {
                println!(
                    "{}",
                    serde_json::json!({
                        "error": e.to_string(),
                        "session_id": session.id
                    })
                );
            }
            std::process::exit(1);
        }
    }
}

/// Full observability streaming renderer
struct StreamRenderer {
    mode: OutputMode,
    in_thinking: bool,
    in_text: bool,
    current_tool: Option<String>,
    tools: Vec<ToolExecution>,
    thinking_content: String,
    text_content: String,
    text_tokens: usize,
    thinking_tokens: usize,
}

struct ToolExecution {
    name: String,
    call_id: String,
    input: serde_json::Value,
    output: Option<String>,
    error: Option<String>,
    duration_ms: Option<u64>,
}

impl StreamRenderer {
    fn new(mode: OutputMode) -> Self {
        Self {
            mode,
            in_thinking: false,
            in_text: false,
            current_tool: None,
            tools: Vec::new(),
            thinking_content: String::new(),
            text_content: String::new(),
            text_tokens: 0,
            thinking_tokens: 0,
        }
    }

    fn handle_event(&mut self, event: ExecutionEvent) {
        match event {
            ExecutionEvent::TextDelta { id, delta } => {
                if id.contains("thinking") {
                    self.handle_thinking_delta(&delta);
                } else {
                    self.handle_text_delta(&delta);
                }
            }
            ExecutionEvent::Part(part) => {
                self.handle_part(&part);
            }
            ExecutionEvent::Complete(_) => {
                // Handled in finish()
            }
            ExecutionEvent::Error(e) => {
                self.end_current_output();
                if self.mode != OutputMode::Json {
                    eprintln!();
                    eprintln!("{} {}", "🟥 ERROR:".red().bold(), e.red());
                }
            }
        }
    }

    fn handle_thinking_delta(&mut self, delta: &str) {
        self.thinking_content.push_str(delta);
        self.thinking_tokens += delta.split_whitespace().count();

        if self.mode == OutputMode::Quiet || self.mode == OutputMode::Json {
            return;
        }

        // End text output if we were in it
        if self.in_text {
            eprintln!();
            self.in_text = false;
        }

        // Start thinking block
        if !self.in_thinking {
            eprintln!("{}", thinking_purple("🔮 THINKING").bold());
            self.in_thinking = true;
        }

        // Stream thinking in blue
        eprint!("{}", thinking_purple(delta));
        let _ = io::stderr().flush();
    }

    fn handle_text_delta(&mut self, delta: &str) {
        self.text_content.push_str(delta);
        self.text_tokens += delta.split_whitespace().count();

        if self.mode == OutputMode::Json {
            return;
        }

        // End thinking output if we were in it
        if self.in_thinking {
            eprintln!();
            eprintln!();
            self.in_thinking = false;
        }

        // Start text block
        if !self.in_text {
            if self.mode == OutputMode::Verbose {
                eprintln!("{}", lime_green("🟢 RESPONSE").bold());
            }
            self.in_text = true;
        }

        // Stream text - lime green
        print!("{}", lime_green(delta));
        let _ = io::stdout().flush();
    }

    fn handle_part(&mut self, part: &Part) {
        match part {
            Part::Thinking { .. } => {
                // Handled via deltas
            }
            Part::Tool {
                tool,
                call_id,
                state,
                ..
            } => {
                self.handle_tool_part(tool, call_id, state);
            }
            Part::Text { .. } => {
                // Handled via deltas
            }
            Part::Patch { files, .. } => {
                if self.mode == OutputMode::Verbose {
                    self.end_current_output();
                    eprintln!();
                    eprintln!("{}", "📝 FILES MODIFIED".cyan().bold());
                    for file in files {
                        eprintln!("   {}", file.yellow());
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_tool_part(&mut self, tool: &str, call_id: &str, state: &ToolState) {
        match state {
            ToolState::Pending { input, .. } | ToolState::Running { input, .. } => {
                // Tool starting
                if self.current_tool.as_deref() != Some(call_id) {
                    self.end_current_output();
                    self.current_tool = Some(call_id.to_string());

                    if self.mode == OutputMode::Verbose {
                        eprintln!();
                        eprintln!("{} {}", purple_bold("🟪 TOOL CALL:"), purple_bold(tool));
                        eprintln!("{} {}", "   Call ID:".dimmed(), call_id.dimmed());
                        eprintln!(
                            "{} {}",
                            "   Input:".dimmed(),
                            serde_json::to_string_pretty(input)
                                .unwrap_or_else(|_| input.to_string())
                                .cyan()
                        );
                    }
                }
            }
            ToolState::Completed {
                input,
                output,
                title,
                time,
            } => {
                let duration_ms = time.end.map(|e| e.saturating_sub(time.start));

                // Store for JSON output
                self.tools.push(ToolExecution {
                    name: tool.to_string(),
                    call_id: call_id.to_string(),
                    input: input.clone(),
                    output: Some(output.clone()),
                    error: None,
                    duration_ms,
                });

                if self.mode == OutputMode::Verbose {
                    // Tool-specific rendering
                    render_tool_result(tool, input, output, title, duration_ms);
                }

                self.current_tool = None;
            }
            ToolState::Error { input, error, time } => {
                let duration_ms = time.end.map(|e| e.saturating_sub(time.start));

                self.tools.push(ToolExecution {
                    name: tool.to_string(),
                    call_id: call_id.to_string(),
                    input: input.clone(),
                    output: None,
                    error: Some(error.clone()),
                    duration_ms,
                });

                if self.mode == OutputMode::Verbose {
                    eprintln!();
                    eprintln!("{} {}", "🟥 TOOL ERROR:".red().bold(), tool.red());
                    eprintln!("   {}", error.red());
                }

                self.current_tool = None;
            }
        }
    }

    fn end_current_output(&mut self) {
        if self.in_thinking {
            eprintln!();
            self.in_thinking = false;
        }
        if self.in_text {
            println!();
            self.in_text = false;
        }
    }

    fn finish(
        &mut self,
        elapsed: std::time::Duration,
        session_id: &str,
        response: &MessageWithParts,
    ) {
        self.end_current_output();

        if self.mode == OutputMode::Json {
            // JSON output
            let output = serde_json::json!({
                "session_id": session_id,
                "message_id": match &response.info {
                    Message::Assistant { id, .. } => id,
                    Message::User { id, .. } => id,
                },
                "thinking": if self.thinking_content.is_empty() { None } else { Some(&self.thinking_content) },
                "response": &self.text_content,
                "tools": self.tools.iter().map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "call_id": t.call_id,
                        "input": t.input,
                        "output": t.output,
                        "error": t.error,
                        "duration_ms": t.duration_ms,
                    })
                }).collect::<Vec<_>>(),
                "stats": {
                    "thinking_tokens": self.thinking_tokens,
                    "response_tokens": self.text_tokens,
                    "tool_calls": self.tools.len(),
                    "duration_ms": elapsed.as_millis(),
                }
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        } else if self.mode == OutputMode::Verbose {
            // Verbose stats
            eprintln!();
            eprintln!(
                "{}",
                "═══════════════════════════════════════════════════════════════".dimmed()
            );
            eprintln!(
                "{} {} thinking, {} response | {} tool calls | {:.1}s",
                mint_green("✓").bold(),
                thinking_purple(&format!("~{}", self.thinking_tokens)),
                light_purple(&format!("~{}", self.text_tokens)),
                mint_green(&self.tools.len().to_string()),
                elapsed.as_secs_f64()
            );
            eprintln!("{} {}", "Session:".dimmed(), session_id.yellow());
            eprintln!(
                "{}",
                "═══════════════════════════════════════════════════════════════".dimmed()
            );
        }
        // Quiet mode - just the response which was already printed
    }
}

async fn list_sessions() {
    let store = SessionStore::new();
    if let Err(e) = store.init_sync() {
        eprintln!("{}", format!("Failed to init storage: {}", e).red());
        return;
    }

    match store.list(None) {
        Ok(sessions) => {
            if sessions.is_empty() {
                println!("{}", "No sessions".dimmed());
                return;
            }
            println!("{}", "Sessions:".cyan().bold());
            for s in sessions {
                println!(
                    "  {} {} {}",
                    s.id.yellow(),
                    s.title.white(),
                    s.directory.dimmed()
                );
            }
        }
        Err(e) => eprintln!("{}", format!("Error: {}", e).red()),
    }
}

async fn create_session(title: Option<&str>) {
    let working_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let store = SessionStore::new();
    if let Err(e) = store.init_sync() {
        eprintln!("{}", format!("Failed to init storage: {}", e).red());
        return;
    }

    match store.create(
        working_dir.to_string_lossy().to_string(),
        None,
        title.map(|s| s.to_string()),
    ) {
        Ok(s) => {
            println!("{}", s.id);
            eprintln!("{} {} - {}", "✚".green(), s.id.yellow(), s.title);
        }
        Err(e) => eprintln!("{}", format!("Error: {}", e).red()),
    }
}

async fn get_messages(session_id: &str) {
    let store = SessionStore::new();
    if let Err(e) = store.init_sync() {
        eprintln!("{}", format!("Failed to init storage: {}", e).red());
        return;
    }

    match store.get_messages(session_id) {
        Ok(messages) => {
            if messages.is_empty() {
                println!("{}", "No messages".dimmed());
                return;
            }

            println!(
                "{}",
                "═══════════════════════════════════════════════════════════════".dimmed()
            );
            println!("{} {}", "Session:".dimmed(), session_id.yellow());
            println!(
                "{}",
                "═══════════════════════════════════════════════════════════════".dimmed()
            );

            for msg in messages {
                println!();
                match &msg.info {
                    Message::User { id, .. } => {
                        println!("{} {}", "▶ USER".white().bold(), id.dimmed());
                    }
                    Message::Assistant { id, .. } => {
                        println!("{} {}", "◀ ASSISTANT".cyan().bold(), id.dimmed());
                    }
                }

                for part in &msg.parts {
                    match part {
                        Part::Text { text, .. } => {
                            println!("{}", text);
                        }
                        Part::Thinking { text, .. } => {
                            println!("{}", thinking_purple("🔮 THINKING:").bold());
                            for line in text.lines() {
                                println!("   {}", thinking_purple(line));
                            }
                        }
                        Part::Tool {
                            tool,
                            call_id,
                            state,
                            ..
                        } => match state {
                            ToolState::Completed { input, output, .. } => {
                                println!(
                                    "{} {} {}",
                                    purple_bold("🟪 TOOL:"),
                                    purple(tool),
                                    call_id.dimmed()
                                );
                                println!(
                                    "   Input: {}",
                                    serde_json::to_string(input).unwrap_or_default().cyan()
                                );
                                println!("{}", mint_green("✓ RESULT:").bold());
                                for line in output.lines().take(10) {
                                    println!("   {}", soft_green(line));
                                }
                                if output.lines().count() > 10 {
                                    println!("   {}", "...".dimmed());
                                }
                            }
                            ToolState::Error { error, .. } => {
                                println!("{} {}", "🟥 TOOL ERROR:".red().bold(), tool.red());
                                println!("   {}", error.red());
                            }
                            _ => {
                                println!(
                                    "{} {} ({})",
                                    light_purple("⏳ TOOL:"),
                                    purple(tool),
                                    call_id.dimmed()
                                );
                            }
                        },
                        _ => {}
                    }
                }
            }
            println!();
            println!(
                "{}",
                "═══════════════════════════════════════════════════════════════".dimmed()
            );
        }
        Err(e) => eprintln!("{}", format!("Error: {}", e).red()),
    }
}

async fn dump_prompt(agent_name: &str) {
    use crow_core::agent::{AgentInfo, AgentRegistry, SystemPromptBuilder};

    let working_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Load agent from registry (includes config files from XDG paths)
    let registry = AgentRegistry::new_with_config(&working_dir).await;

    let agent = registry.get(agent_name).await.unwrap_or_else(|| {
        eprintln!(
            "{}",
            format!("Agent '{}' not found, using default", agent_name).yellow()
        );
        AgentInfo::new(agent_name)
    });

    let provider_name = if env::var("ANTHROPIC_API_KEY").is_ok() {
        "anthropic"
    } else if env::var("OPENAI_API_KEY").is_ok() {
        "openai"
    } else {
        "moonshot"
    };

    let builder = SystemPromptBuilder::new(
        agent.clone(),
        working_dir.clone(),
        provider_name.to_string(),
    );

    let model_id = match provider_name {
        "anthropic" => "claude-sonnet-4-20250514",
        "openai" => "gpt-4",
        _ => "moonshot-v1-auto",
    };

    // Show agent config info
    eprintln!(
        "{}",
        format!(
            "Agent config: mode={:?}, prompt={}",
            agent.mode,
            if agent.prompt.is_some() {
                "CUSTOM"
            } else {
                "default"
            }
        )
        .dimmed()
    );

    let prompts = builder.build(model_id);

    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════".dimmed()
    );
    println!(
        "{} agent={} provider={} model={}",
        "SYSTEM PROMPT".cyan().bold(),
        agent_name.yellow(),
        provider_name.cyan(),
        model_id.dimmed()
    );
    println!("{} {}", "Working dir:".dimmed(), working_dir.display());
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════".dimmed()
    );
    println!();
    println!(
        "{}",
        format!("─── System Message 1 ({} chars) ───", prompts[0].len()).yellow()
    );
    println!("{}", prompts[0]);
    println!();
    println!(
        "{}",
        format!("─── System Message 2 ({} chars) ───", prompts[1].len()).yellow()
    );
    println!("{}", prompts[1]);
    println!();
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════".dimmed()
    );
}

// ============================================================================
// Tool-Specific Renderers
// ============================================================================

/// Render tool result with tool-specific formatting
fn render_tool_result(
    tool: &str,
    input: &serde_json::Value,
    output: &str,
    title: &str,
    duration_ms: Option<u64>,
) {
    eprintln!();

    match tool {
        "bash" => render_bash(input, output, duration_ms),
        "edit" | "str_replace" => render_edit(input, output, duration_ms),
        "read" | "file_read" => render_read(input, output, duration_ms),
        "grep" => render_grep(input, output, duration_ms),
        "list" | "glob" => render_list(input, output, duration_ms),
        "todoread" => render_todoread(output, duration_ms),
        "todowrite" => render_todowrite(input, duration_ms),
        "webfetch" | "web_fetch" => render_webfetch(input, output, duration_ms),
        "websearch" | "web_search" => render_websearch(input, output, duration_ms),
        "task" => render_task(input, output, duration_ms),
        _ => render_generic(tool, input, output, title, duration_ms),
    }
}

/// 🔧 bash - Shell command execution
fn render_bash(input: &serde_json::Value, output: &str, duration_ms: Option<u64>) {
    let cmd = input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("???");
    let desc = input.get("description").and_then(|v| v.as_str());

    eprintln!(
        "{} {}",
        purple_bold("🔧 bash"),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    // Show command
    eprintln!("   {} {}", mint_green("$"), cmd.white());

    // Show description if available
    if let Some(d) = desc {
        eprintln!("   {}", light_purple(d));
    }

    // Show output (limited)
    if !output.is_empty() {
        let lines: Vec<&str> = output.lines().collect();
        let show_lines = lines.len().min(15);
        for line in lines.iter().take(show_lines) {
            eprintln!("   {}", soft_green(line));
        }
        if lines.len() > show_lines {
            eprintln!(
                "   {} ({} more lines)",
                "...".dimmed(),
                lines.len() - show_lines
            );
        }
    }

    // Check for success/failure indicator
    if output.contains("error") || output.contains("Error") || output.contains("ERROR") {
        eprintln!("   {}", "⚠ may contain errors".red().dimmed());
    } else {
        eprintln!("   {}", mint_green("✓"));
    }
}

/// 📝 edit/str_replace - File modification
fn render_edit(input: &serde_json::Value, output: &str, duration_ms: Option<u64>) {
    let file_path = input
        .get("file_path")
        .or_else(|| input.get("filePath"))
        .and_then(|v| v.as_str())
        .unwrap_or("???");

    // Try to extract just the filename
    let filename = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file_path);

    eprintln!(
        "{} {} {}",
        purple_bold("📝 edit"),
        light_purple(&format!("({})", filename)),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    // Try to parse output as JSON for structured info
    if let Ok(edit_output) = serde_json::from_str::<serde_json::Value>(output) {
        let additions = edit_output
            .get("additions")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let deletions = edit_output
            .get("deletions")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        eprintln!(
            "   {} {}, {} {}",
            mint_green(&format!("+{}", additions)),
            "additions".dimmed(),
            format!("-{}", deletions).red(),
            "deletions".dimmed()
        );

        // Show diff if available
        if let Some(diff) = edit_output.get("diff").and_then(|v| v.as_str()) {
            eprintln!("   {}", purple("───"));
            for line in diff.lines().take(20) {
                if line.starts_with('+') && !line.starts_with("+++") {
                    eprintln!("   {}", mint_green(line));
                } else if line.starts_with('-') && !line.starts_with("---") {
                    eprintln!("   {}", line.red());
                } else if line.starts_with("@@") {
                    eprintln!("   {}", light_purple(line));
                } else {
                    eprintln!("   {}", line.dimmed());
                }
            }
        }
    } else {
        // Plain text output
        eprintln!("   {}", file_path.dimmed());
        for line in output.lines().take(10) {
            eprintln!("   {}", soft_green(line));
        }
    }

    eprintln!("   {}", mint_green("✓"));
}

/// 📖 read/file_read - File reading
fn render_read(input: &serde_json::Value, output: &str, duration_ms: Option<u64>) {
    let file_path = input
        .get("file_path")
        .or_else(|| input.get("filePath"))
        .and_then(|v| v.as_str())
        .unwrap_or("???");

    let filename = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file_path);

    // Parse JSON output to get content
    let content = if let Ok(json) = serde_json::from_str::<serde_json::Value>(output) {
        json.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or(output)
            .to_string()
    } else {
        output.to_string()
    };

    let line_count = content.lines().count();

    eprintln!(
        "{} {} {}",
        purple_bold("📖 read"),
        light_purple(&format!("({})", filename)),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    // Show preview
    let preview_lines = 8;
    for line in content.lines().take(preview_lines) {
        let truncated = if line.len() > 80 {
            format!("{}...", &line[..77])
        } else {
            line.to_string()
        };
        eprintln!("   {}", truncated.dimmed());
    }

    if line_count > preview_lines {
        eprintln!("   {} ({} total lines)", "...".dimmed(), line_count);
    }
}

/// 🔍 grep - Search results
fn render_grep(input: &serde_json::Value, output: &str, duration_ms: Option<u64>) {
    let pattern = input
        .get("pattern")
        .and_then(|v| v.as_str())
        .unwrap_or("???");
    let path = input.get("path").and_then(|v| v.as_str());

    let match_count = output.lines().filter(|l| !l.is_empty()).count();

    eprintln!(
        "{} {} {}",
        purple_bold("🔍 grep"),
        light_purple(&format!("\"{}\"", pattern)),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    if let Some(p) = path {
        eprintln!("   {} {}", "in:".dimmed(), p.dimmed());
    }

    eprintln!(
        "   {} {}",
        mint_green(&format!("{}", match_count)).bold(),
        "matches".dimmed()
    );

    // Show matches with context
    for line in output.lines().take(10) {
        if line.contains(':') {
            // Format: file:line:content
            let parts: Vec<&str> = line.splitn(3, ':').collect();
            if parts.len() >= 2 {
                let file = std::path::Path::new(parts[0])
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(parts[0]);
                let rest = parts[1..].join(":");
                eprintln!("   {}:{}", purple(file), soft_green(&rest));
            } else {
                eprintln!("   {}", soft_green(line));
            }
        } else {
            eprintln!("   {}", soft_green(line));
        }
    }

    if match_count > 10 {
        eprintln!("   {} ({} more)", "...".dimmed(), match_count - 10);
    }
}

/// 📁 list/glob - Directory listing
fn render_list(input: &serde_json::Value, output: &str, duration_ms: Option<u64>) {
    let path = input.get("path").and_then(|v| v.as_str()).unwrap_or(".");
    let pattern = input.get("pattern").and_then(|v| v.as_str());

    let item_count = output.lines().filter(|l| !l.is_empty()).count();

    eprintln!(
        "{} {} {}",
        purple_bold("📁 list"),
        light_purple(path),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    if let Some(p) = pattern {
        eprintln!("   {} {}", "pattern:".dimmed(), light_purple(p));
    }

    eprintln!("   {} items", mint_green(&item_count.to_string()));

    // Show items
    for line in output.lines().take(20) {
        let trimmed = line.trim();
        if trimmed.ends_with('/') {
            eprintln!("   {}", purple(trimmed)); // Directories in purple
        } else {
            eprintln!("   {}", trimmed.dimmed());
        }
    }

    if item_count > 20 {
        eprintln!("   {} ({} more)", "...".dimmed(), item_count - 20);
    }
}

/// 📋 todoread - Task list display
fn render_todoread(output: &str, duration_ms: Option<u64>) {
    eprintln!(
        "{} {}",
        purple_bold("📋 todoread"),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    // Try to parse as JSON
    if let Ok(todos) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(items) = todos.get("todos").and_then(|v| v.as_array()) {
            for item in items {
                let content = item
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("???");
                let status = item
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("pending");

                let icon = match status {
                    "completed" => mint_green("✓"),
                    "in_progress" => "⧗".yellow(),
                    _ => light_purple("☐"),
                };

                eprintln!("   {} {}", icon, content);
            }
        }
    } else {
        // Plain text
        for line in output.lines().take(10) {
            eprintln!("   {}", line.dimmed());
        }
    }
}

/// 📋 todowrite - Task list update
fn render_todowrite(input: &serde_json::Value, duration_ms: Option<u64>) {
    eprintln!(
        "{} {}",
        purple_bold("📋 todowrite"),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    if let Some(todos) = input.get("todos").and_then(|v| v.as_array()) {
        for item in todos {
            let content = item
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("???");
            let status = item
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("pending");

            let icon = match status {
                "completed" => mint_green("✓"),
                "in_progress" => "⧗".yellow(),
                _ => light_purple("☐"),
            };

            eprintln!("   {} {}", icon, content);
        }
    }

    eprintln!("   {}", mint_green("✓ updated"));
}

/// 🌐 webfetch - URL fetching
fn render_webfetch(input: &serde_json::Value, output: &str, duration_ms: Option<u64>) {
    let url = input.get("url").and_then(|v| v.as_str()).unwrap_or("???");

    eprintln!(
        "{} {}",
        purple_bold("🌐 webfetch"),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    eprintln!("   {}", light_purple(url).underline());

    let char_count = output.len();
    let line_count = output.lines().count();

    eprintln!(
        "   {} chars, {} lines",
        mint_green(&char_count.to_string()),
        line_count
    );

    // Show preview
    for line in output.lines().take(5) {
        let truncated = if line.len() > 60 {
            format!("{}...", &line[..57])
        } else {
            line.to_string()
        };
        eprintln!("   {}", truncated.dimmed());
    }

    if line_count > 5 {
        eprintln!("   {}", "...".dimmed());
    }
}

/// 🔎 websearch - Web search
fn render_websearch(input: &serde_json::Value, output: &str, duration_ms: Option<u64>) {
    let query = input.get("query").and_then(|v| v.as_str()).unwrap_or("???");

    eprintln!(
        "{} {}",
        purple_bold("🔎 websearch"),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    eprintln!("   {} \"{}\"", "query:".dimmed(), light_purple(query));

    // Try to show result count
    let result_lines = output.lines().filter(|l| !l.is_empty()).count();
    eprintln!("   {} results", mint_green(&result_lines.to_string()));

    // Show preview
    for line in output.lines().take(5) {
        eprintln!("   {}", line.dimmed());
    }
}

/// 🤖 task - Subagent execution
fn render_task(input: &serde_json::Value, output: &str, duration_ms: Option<u64>) {
    let description = input
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("task");
    let subagent_type = input
        .get("subagent_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let prompt = input.get("prompt").and_then(|v| v.as_str()).unwrap_or("");

    eprintln!(
        "{} {} {}",
        purple_bold("🤖 task"),
        light_purple(&format!("[{}]", subagent_type)),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    eprintln!("   {} {}", "description:".dimmed(), description.white());

    // Show prompt (truncated)
    let prompt_preview = if prompt.len() > 100 {
        format!("{}...", &prompt[..97])
    } else {
        prompt.to_string()
    };
    eprintln!("   {} {}", "prompt:".dimmed(), prompt_preview.dimmed());

    // Show result
    eprintln!("   {}", purple("─── subagent result ───"));
    let lines: Vec<&str> = output.lines().collect();
    let show_lines = lines.len().min(20);
    for line in lines.iter().take(show_lines) {
        eprintln!("   {}", soft_green(line));
    }
    if lines.len() > show_lines {
        eprintln!(
            "   {} ({} more lines)",
            "...".dimmed(),
            lines.len() - show_lines
        );
    }

    eprintln!("   {}", mint_green("✓"));
}

/// Generic fallback renderer
fn render_generic(
    tool: &str,
    input: &serde_json::Value,
    output: &str,
    title: &str,
    duration_ms: Option<u64>,
) {
    eprintln!(
        "{} {} {}",
        purple_bold(&format!("🔧 {}", tool)),
        if !title.is_empty() { title } else { "" },
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    // Show input args for debugging
    eprintln!(
        "   {} {}",
        "input:".dimmed(),
        serde_json::to_string(input).unwrap_or_default().cyan()
    );

    // Show output (truncated)
    let lines: Vec<&str> = output.lines().collect();
    let show_lines = lines.len().min(15);

    for line in lines.iter().take(show_lines) {
        eprintln!("   {}", soft_green(line));
    }

    if lines.len() > show_lines {
        eprintln!(
            "   {} ({} more lines)",
            "...".dimmed(),
            lines.len() - show_lines
        );
    }

    eprintln!("   {}", mint_green("✓"));
}
