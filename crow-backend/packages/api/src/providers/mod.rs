/// OpenAI-compatible provider system
/// Supports any API that implements the OpenAI chat completions spec
pub mod config;
pub use config::*;

#[cfg(feature = "server")]
pub mod client;
#[cfg(feature = "server")]
pub use client::*;
