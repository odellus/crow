//! Structured logging for observability
//!
//! Logs to XDG state directory (~/.local/state/crow/logs/):
//! - agent.log         - Human readable agent log
//! - tool-calls.jsonl  - Every tool call (JSONL)
//! - messages.jsonl    - Every message sent/received (JSONL)

use crate::global::GlobalPaths;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::OnceLock;

// ============================================================================
// Log Entry Types
// ============================================================================

/// Tool call log entry
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallLog {
    pub timestamp: String,
    pub session_id: String,
    pub message_id: String,
    pub tool_name: String,
    pub tool_id: String,
    pub input: serde_json::Value,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Message log entry
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageLog {
    pub timestamp: String,
    pub session_id: String,
    pub message_id: String,
    pub role: String,
    pub content: String,
    pub model: Option<String>,
    pub tokens_in: Option<u64>,
    pub tokens_out: Option<u64>,
    pub cost: Option<f64>,
}

/// Agent execution log entry
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentLog {
    pub timestamp: String,
    pub session_id: String,
    pub agent: String,
    pub provider: String,
    pub model: String,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub cost: f64,
    pub duration_ms: u64,
    pub tool_calls: usize,
    pub error: Option<String>,
}

// ============================================================================
// Logger Implementation
// ============================================================================

/// Structured logger for crow
pub struct CrowLogger {
    log_dir: PathBuf,
}

impl CrowLogger {
    pub fn new() -> Self {
        let paths = GlobalPaths::new();
        let log_dir = paths.state.join("logs");

        // Create log directory
        if let Err(e) = fs::create_dir_all(&log_dir) {
            eprintln!("Failed to create log directory: {}", e);
        }

        Self { log_dir }
    }

    /// Append line to a log file
    fn append(&self, filename: &str, line: &str) {
        let path = self.log_dir.join(filename);
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(file, "{}", line);
        }
    }

    /// Log agent execution
    pub fn log_agent(&self, entry: &AgentLog) {
        let line = format!(
            "{} [{}] session={} agent={} model={} tokens={}/{} cost=${:.6} duration={}ms{}",
            entry.timestamp,
            if entry.error.is_some() { "ERROR" } else { "OK" },
            &entry.session_id[..entry.session_id.len().min(16)],
            entry.agent,
            entry.model,
            entry.tokens_in,
            entry.tokens_out,
            entry.cost,
            entry.duration_ms,
            entry
                .error
                .as_ref()
                .map(|e| format!(" error={}", e))
                .unwrap_or_default()
        );
        self.append("agent.log", &line);
    }

    /// Log tool call (JSONL)
    pub fn log_tool_call(&self, entry: &ToolCallLog) {
        if let Ok(json) = serde_json::to_string(entry) {
            self.append("tool-calls.jsonl", &json);
        }
    }

    /// Log message (JSONL)
    pub fn log_message(&self, entry: &MessageLog) {
        if let Ok(json) = serde_json::to_string(entry) {
            self.append("messages.jsonl", &json);
        }
    }

    /// Get log directory path
    pub fn log_dir(&self) -> &PathBuf {
        &self.log_dir
    }

    /// Read recent agent log lines
    pub fn read_agent_log(&self, lines: usize) -> Vec<String> {
        read_last_lines(&self.log_dir.join("agent.log"), lines)
    }

    /// Read recent tool calls
    pub fn read_tool_calls(&self, count: usize) -> Vec<ToolCallLog> {
        read_jsonl(&self.log_dir.join("tool-calls.jsonl"), count)
    }

    /// Read recent messages
    pub fn read_messages(&self, count: usize) -> Vec<MessageLog> {
        read_jsonl(&self.log_dir.join("messages.jsonl"), count)
    }
}

impl Default for CrowLogger {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Global Logger
// ============================================================================

static LOGGER: OnceLock<CrowLogger> = OnceLock::new();

/// Get the global logger
pub fn logger() -> &'static CrowLogger {
    LOGGER.get_or_init(CrowLogger::new)
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Log an agent execution
pub fn log_agent_execution(
    session_id: &str,
    agent: &str,
    provider: &str,
    model: &str,
    tokens_in: u64,
    tokens_out: u64,
    cost: f64,
    duration_ms: u64,
    tool_calls: usize,
    error: Option<String>,
) {
    logger().log_agent(&AgentLog {
        timestamp: chrono::Utc::now().to_rfc3339(),
        session_id: session_id.to_string(),
        agent: agent.to_string(),
        provider: provider.to_string(),
        model: model.to_string(),
        tokens_in,
        tokens_out,
        cost,
        duration_ms,
        tool_calls,
        error,
    });
}

/// Log a tool call
pub fn log_tool_call(
    session_id: &str,
    message_id: &str,
    tool_name: &str,
    tool_id: &str,
    input: serde_json::Value,
    output: Option<String>,
    error: Option<String>,
    duration_ms: u64,
) {
    logger().log_tool_call(&ToolCallLog {
        timestamp: chrono::Utc::now().to_rfc3339(),
        session_id: session_id.to_string(),
        message_id: message_id.to_string(),
        tool_name: tool_name.to_string(),
        tool_id: tool_id.to_string(),
        input,
        output,
        error,
        duration_ms,
    });
}

/// Log a message
pub fn log_message(
    session_id: &str,
    message_id: &str,
    role: &str,
    content: &str,
    model: Option<&str>,
    tokens_in: Option<u64>,
    tokens_out: Option<u64>,
    cost: Option<f64>,
) {
    logger().log_message(&MessageLog {
        timestamp: chrono::Utc::now().to_rfc3339(),
        session_id: session_id.to_string(),
        message_id: message_id.to_string(),
        role: role.to_string(),
        content: content.to_string(),
        model: model.map(|s| s.to_string()),
        tokens_in,
        tokens_out,
        cost,
    });
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Read last N lines from a file
fn read_last_lines(path: &PathBuf, n: usize) -> Vec<String> {
    if !path.exists() {
        return vec![];
    }
    match File::open(path) {
        Ok(file) => {
            let lines: Vec<String> = BufReader::new(file)
                .lines()
                .filter_map(|l| l.ok())
                .collect();
            lines.into_iter().rev().take(n).rev().collect()
        }
        Err(_) => vec![],
    }
}

/// Read last N entries from a JSONL file
fn read_jsonl<T: for<'de> Deserialize<'de>>(path: &PathBuf, n: usize) -> Vec<T> {
    if !path.exists() {
        return vec![];
    }
    match File::open(path) {
        Ok(file) => {
            let entries: Vec<T> = BufReader::new(file)
                .lines()
                .filter_map(|l| l.ok())
                .filter_map(|line| serde_json::from_str(&line).ok())
                .collect();
            entries.into_iter().rev().take(n).rev().collect()
        }
        Err(_) => vec![],
    }
}
