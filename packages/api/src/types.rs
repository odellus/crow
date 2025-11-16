//! OpenCode-compatible message types for crow
//! Based on OpenCode's TypeScript SDK types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "role")]
pub enum Message {
    #[serde(rename = "user")]
    User {
        id: String,
        session_id: String,
        time: MessageTime,
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<MessageSummary>,
    },
    #[serde(rename = "assistant")]
    Assistant {
        id: String,
        session_id: String,
        parent_id: String,
        model_id: String,
        provider_id: String,
        mode: String,
        time: MessageTime,
        path: MessagePath,
        cost: f64,
        tokens: TokenUsage,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<bool>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageTime {
    pub created: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessagePath {
    pub cwd: String,
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
    pub reasoning: u64,
    pub cache: CacheTokens,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheTokens {
    pub read: u64,
    pub write: u64,
}

/// Message parts - the actual content of a message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Part {
    #[serde(rename = "text")]
    Text {
        id: String,
        session_id: String,
        message_id: String,
        text: String,
    },
    #[serde(rename = "thinking")]
    Thinking {
        id: String,
        session_id: String,
        message_id: String,
        text: String,
    },
    #[serde(rename = "tool")]
    Tool {
        id: String,
        session_id: String,
        message_id: String,
        call_id: String,
        tool: String,
        state: ToolState,
    },
    #[serde(rename = "file")]
    File {
        id: String,
        session_id: String,
        message_id: String,
        mime: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        url: String,
    },
}

/// Tool execution state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status")]
pub enum ToolState {
    #[serde(rename = "pending")]
    Pending {
        input: serde_json::Value,
        raw: String,
    },
    #[serde(rename = "running")]
    Running {
        input: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        time: ToolTime,
    },
    #[serde(rename = "completed")]
    Completed {
        input: serde_json::Value,
        output: String,
        title: String,
        time: ToolTime,
    },
    #[serde(rename = "error")]
    Error {
        input: serde_json::Value,
        error: String,
        time: ToolTime,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolTime {
    pub start: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
}

/// Chat conversation with messages and parts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Conversation {
    pub id: String,
    pub messages: Vec<Message>,
    pub parts: Vec<Part>,
}
