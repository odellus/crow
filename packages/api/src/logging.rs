//! Verbose logging module for debugging and tracing
//! Logs full prompts, messages, and tool calls like Langfuse

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Log entry for a complete LLM interaction
#[derive(Debug, Serialize, Deserialize)]
pub struct LLMLogEntry {
    pub timestamp: String, // ISO 8601 format
    pub session_id: String,
    pub message_id: String,
    pub agent: String,
    pub provider: String,
    pub model: String,
    pub system_prompt: String,
    pub messages: Vec<LogMessage>,
    pub tools: Vec<String>,
    pub response: Option<LogResponse>,
    pub tokens: Option<LogTokens>,
    pub cost: Option<f64>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<LogToolCall>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
    pub result: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogTokens {
    pub input: u64,
    pub output: u64,
}

/// Logger for verbose LLM tracing
pub struct VerboseLogger {
    log_dir: PathBuf,
    enabled: bool,
}

impl VerboseLogger {
    pub fn new() -> Self {
        // Use XDG data directory
        let log_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("crow")
            .join("logs");

        // Check if verbose logging is enabled via env var
        let enabled = std::env::var("CROW_VERBOSE_LOG")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        if enabled {
            // Create log directory if it doesn't exist
            if let Err(e) = fs::create_dir_all(&log_dir) {
                tracing::warn!("Failed to create log directory: {}", e);
            }
        }

        Self { log_dir, enabled }
    }

    /// Log a complete LLM interaction
    pub fn log_interaction(&self, entry: &LLMLogEntry) {
        if !self.enabled {
            return;
        }

        // Create filename with timestamp (extract date part from ISO string)
        let date_part = entry.timestamp.replace(":", "").replace("-", "");
        let filename = format!(
            "{}-{}.json",
            &date_part[..15.min(date_part.len())],
            &entry.message_id[..8.min(entry.message_id.len())]
        );
        let filepath = self.log_dir.join(filename);

        // Serialize and write
        match serde_json::to_string_pretty(entry) {
            Ok(json) => {
                if let Err(e) = fs::write(&filepath, json) {
                    tracing::error!("Failed to write log file: {}", e);
                } else {
                    tracing::debug!("Logged interaction to {}", filepath.display());
                }
            }
            Err(e) => {
                tracing::error!("Failed to serialize log entry: {}", e);
            }
        }

        // Also append to a summary log
        self.append_summary(entry);
    }

    /// Append a summary line to the daily log
    fn append_summary(&self, entry: &LLMLogEntry) {
        // Extract date from timestamp (YYYY-MM-DD)
        let date = &entry.timestamp[..10.min(entry.timestamp.len())];
        let summary_file = self.log_dir.join(format!("{}.log", date));

        let summary = format!(
            "{} session={} agent={} model={} tokens={}/{} cost=${:.6}\n",
            &entry.timestamp[11..19.min(entry.timestamp.len())], // HH:MM:SS
            &entry.session_id[..12.min(entry.session_id.len())],
            entry.agent,
            entry.model,
            entry.tokens.as_ref().map(|t| t.input).unwrap_or(0),
            entry.tokens.as_ref().map(|t| t.output).unwrap_or(0),
            entry.cost.unwrap_or(0.0),
        );

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(summary_file)
        {
            let _ = file.write_all(summary.as_bytes());
        }
    }

    /// Log a simple info message
    pub fn info(&self, message: &str) {
        if self.enabled {
            tracing::info!("{}", message);
        }
    }
}

impl Default for VerboseLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Global logger instance
static LOGGER: std::sync::OnceLock<VerboseLogger> = std::sync::OnceLock::new();

/// Get the global verbose logger
pub fn logger() -> &'static VerboseLogger {
    LOGGER.get_or_init(VerboseLogger::new)
}

/// Log an LLM interaction (convenience function)
pub fn log_llm_interaction(entry: LLMLogEntry) {
    logger().log_interaction(&entry);
}
