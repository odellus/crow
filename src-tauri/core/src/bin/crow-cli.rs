//! Crow CLI - Full observability streaming interface for agent interaction
//!
//! Color scheme:
//!   🟦 Blue    - Agent thinking/reasoning
//!   🟩 Green   - Tool calls starting
//!   🟨 Yellow  - Tool results
//!   🟥 Red     - Errors
//!   ⬜ White   - Final response text
//!
//! Usage:
//!   crow-cli chat "your message"           - Full verbose streaming output
//!   crow-cli chat --json "message"         - JSON output (no streaming)
//!   crow-cli chat --quiet "message"        - Minimal output (just response)

use colored::Colorize;
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
            dump_prompt(agent);
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
    println!("  crow-cli chat \"message\"              Full verbose streaming (default)");
    println!("  crow-cli chat --quiet \"msg\"         Just the final response");
    println!("  crow-cli chat --json \"msg\"          JSON output, no streaming");
    println!("  crow-cli chat --session ID \"msg\"    Send to specific session");
    println!("  crow-cli new [title]                 Create new session");
    println!("  crow-cli sessions                    List all sessions");
    println!("  crow-cli messages <session-id>       Show messages with full history");
    println!("  crow-cli logs [count]                Show recent agent logs");
    println!("  crow-cli prompt [agent]              Dump full system prompt");
    println!("  crow-cli paths                       Show storage paths");
    println!();
    println!("{}", "COLOR LEGEND:".yellow());
    println!("  {}  Agent thinking/reasoning", "🟦 Blue".blue());
    println!("  {}  Tool calls", "🟩 Green".green());
    println!("  {}  Tool results", "🟨 Yellow".yellow());
    println!("  {}  Errors", "🟥 Red".red());
    println!("  {}  Response text", "⬜ White".white());
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
            eprintln!("{}", "🟦 THINKING".blue().bold());
            self.in_thinking = true;
        }

        // Stream thinking in blue
        eprint!("{}", delta.blue());
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
                eprintln!("{}", "⬜ RESPONSE".white().bold());
            }
            self.in_text = true;
        }

        // Stream text - white/normal
        print!("{}", delta);
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
                        eprintln!("{} {}", "🟩 TOOL CALL:".green().bold(), tool.green().bold());
                        eprintln!("{} {}", "   Call ID:".dimmed(), call_id.dimmed());
                        eprintln!(
                            "{} {}",
                            "   Input:".dimmed(),
                            serde_json::to_string_pretty(input)
                                .unwrap_or_else(|_| input.to_string())
                                .dimmed()
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
                "✓".green().bold(),
                format!("~{}", self.thinking_tokens).blue(),
                format!("~{}", self.text_tokens).white(),
                self.tools.len().to_string().green(),
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
                            println!("{}", "🟦 THINKING:".blue().bold());
                            for line in text.lines() {
                                println!("   {}", line.blue());
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
                                    "🟩 TOOL:".green().bold(),
                                    tool.green(),
                                    call_id.dimmed()
                                );
                                println!(
                                    "   Input: {}",
                                    serde_json::to_string(input).unwrap_or_default().dimmed()
                                );
                                println!("{}", "🟨 RESULT:".yellow().bold());
                                for line in output.lines().take(10) {
                                    println!("   {}", line.yellow());
                                }
                                if output.lines().count() > 10 {
                                    println!("   {}", "...".yellow());
                                }
                            }
                            ToolState::Error { error, .. } => {
                                println!("{} {}", "🟥 TOOL ERROR:".red().bold(), tool.red());
                                println!("   {}", error.red());
                            }
                            _ => {
                                println!("{} {} ({})", "⏳ TOOL:".yellow(), tool, call_id.dimmed());
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

fn dump_prompt(agent_name: &str) {
    use crow_core::agent::{AgentInfo, SystemPromptBuilder};

    let working_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let agent = AgentInfo::new(agent_name);

    let provider_name = if env::var("ANTHROPIC_API_KEY").is_ok() {
        "anthropic"
    } else if env::var("OPENAI_API_KEY").is_ok() {
        "openai"
    } else {
        "moonshot"
    };

    let builder = SystemPromptBuilder::new(agent, working_dir.clone(), provider_name.to_string());

    let model_id = match provider_name {
        "anthropic" => "claude-sonnet-4-20250514",
        "openai" => "gpt-4",
        _ => "moonshot-v1-auto",
    };

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
        "🔧 bash".cyan().bold(),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    // Show command
    eprintln!("   {} {}", "$".green(), cmd.white());

    // Show description if available
    if let Some(d) = desc {
        eprintln!("   {}", d.dimmed());
    }

    // Show output (limited)
    if !output.is_empty() {
        let lines: Vec<&str> = output.lines().collect();
        let show_lines = lines.len().min(15);
        for line in lines.iter().take(show_lines) {
            eprintln!("   {}", line.yellow());
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
        eprintln!("   {}", "✓".green());
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
        "📝 edit".cyan().bold(),
        format!("({})", filename).white(),
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
            format!("+{}", additions).green(),
            "additions".dimmed(),
            format!("-{}", deletions).red(),
            "deletions".dimmed()
        );

        // Show diff if available
        if let Some(diff) = edit_output.get("diff").and_then(|v| v.as_str()) {
            eprintln!("   {}", "───".dimmed());
            for line in diff.lines().take(20) {
                if line.starts_with('+') && !line.starts_with("+++") {
                    eprintln!("   {}", line.green());
                } else if line.starts_with('-') && !line.starts_with("---") {
                    eprintln!("   {}", line.red());
                } else if line.starts_with("@@") {
                    eprintln!("   {}", line.cyan());
                } else {
                    eprintln!("   {}", line.dimmed());
                }
            }
        }
    } else {
        // Plain text output
        eprintln!("   {}", file_path.dimmed());
        for line in output.lines().take(10) {
            eprintln!("   {}", line.yellow());
        }
    }

    eprintln!("   {}", "✓".green());
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
        "📖 read".cyan().bold(),
        format!("({})", filename).white(),
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
        "🔍 grep".cyan().bold(),
        format!("\"{}\"", pattern).yellow(),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    if let Some(p) = path {
        eprintln!("   {} {}", "in:".dimmed(), p.dimmed());
    }

    eprintln!(
        "   {} {}",
        format!("{}", match_count).green().bold(),
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
                eprintln!("   {}:{}", file.cyan(), rest.yellow());
            } else {
                eprintln!("   {}", line.yellow());
            }
        } else {
            eprintln!("   {}", line.yellow());
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
        "📁 list".cyan().bold(),
        path.white(),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    if let Some(p) = pattern {
        eprintln!("   {} {}", "pattern:".dimmed(), p.yellow());
    }

    eprintln!("   {} items", item_count.to_string().green());

    // Show items
    for line in output.lines().take(20) {
        let trimmed = line.trim();
        if trimmed.ends_with('/') {
            eprintln!("   {}", trimmed.blue()); // Directories in blue
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
        "📋 todoread".cyan().bold(),
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
                    "completed" => "✓".green(),
                    "in_progress" => "⧗".yellow(),
                    _ => "☐".white(),
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
        "📋 todowrite".cyan().bold(),
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
                "completed" => "✓".green(),
                "in_progress" => "⧗".yellow(),
                _ => "☐".white(),
            };

            eprintln!("   {} {}", icon, content);
        }
    }

    eprintln!("   {}", "✓ updated".green());
}

/// 🌐 webfetch - URL fetching
fn render_webfetch(input: &serde_json::Value, output: &str, duration_ms: Option<u64>) {
    let url = input.get("url").and_then(|v| v.as_str()).unwrap_or("???");

    eprintln!(
        "{} {}",
        "🌐 webfetch".cyan().bold(),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    eprintln!("   {}", url.blue().underline());

    let char_count = output.len();
    let line_count = output.lines().count();

    eprintln!(
        "   {} chars, {} lines",
        char_count.to_string().green(),
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
        "🔎 websearch".cyan().bold(),
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    eprintln!("   {} \"{}\"", "query:".dimmed(), query.yellow());

    // Try to show result count
    let result_lines = output.lines().filter(|l| !l.is_empty()).count();
    eprintln!("   {} results", result_lines.to_string().green());

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
        "🤖 task".magenta().bold(),
        format!("[{}]", subagent_type).cyan(),
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
    eprintln!("   {}", "─── subagent result ───".dimmed());
    let lines: Vec<&str> = output.lines().collect();
    let show_lines = lines.len().min(20);
    for line in lines.iter().take(show_lines) {
        eprintln!("   {}", line.yellow());
    }
    if lines.len() > show_lines {
        eprintln!(
            "   {} ({} more lines)",
            "...".dimmed(),
            lines.len() - show_lines
        );
    }

    eprintln!("   {}", "✓".green());
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
        format!("🔧 {}", tool).cyan().bold(),
        if !title.is_empty() { title } else { "" },
        format!("({}ms)", duration_ms.unwrap_or(0)).dimmed()
    );

    // Show input args for debugging
    eprintln!(
        "   {} {}",
        "input:".dimmed(),
        serde_json::to_string(input).unwrap_or_default().dimmed()
    );

    // Show output (truncated)
    let lines: Vec<&str> = output.lines().collect();
    let show_lines = lines.len().min(15);

    for line in lines.iter().take(show_lines) {
        eprintln!("   {}", line.yellow());
    }

    if lines.len() > show_lines {
        eprintln!(
            "   {} ({} more lines)",
            "...".dimmed(),
            lines.len() - show_lines
        );
    }

    eprintln!("   {}", "✓".green());
}
