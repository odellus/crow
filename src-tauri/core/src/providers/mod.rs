/// OpenAI-compatible provider system
/// Supports any API that implements the OpenAI chat completions spec
pub mod config;
pub use config::*;

pub mod client;
pub use client::*;
