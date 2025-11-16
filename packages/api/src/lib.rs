//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;

pub mod types;
pub use types::*;

pub mod providers;
pub use providers::*;

#[cfg(feature = "server")]
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs,
};

/// Echo the user input on the server.
#[post("/api/echo")]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

/// Send a message and get a response from the LLM
#[post("/api/send_message")]
pub async fn send_message(content: String) -> Result<Vec<Part>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let session_id = "session-1".to_string();
        let message_id = "msg-1".to_string();

        // Create provider client (using Moonshot for now)
        let provider_config = ProviderConfig::moonshot();
        let client = ProviderClient::new(provider_config).map_err(|e| ServerFnError::new(e))?;

        // Build messages
        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are a helpful AI assistant.")
                    .build()
                    .map_err(|e| {
                        ServerFnError::new(format!("Failed to build system message: {}", e))
                    })?,
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(content.clone())
                    .build()
                    .map_err(|e| {
                        ServerFnError::new(format!("Failed to build user message: {}", e))
                    })?,
            ),
        ];

        // Call the LLM
        let assistant_message = client
            .chat(messages, None)
            .await
            .map_err(|e| ServerFnError::new(e))?;

        // Convert to OpenCode-style parts
        let parts = vec![Part::Text {
            id: "part-text-1".to_string(),
            session_id: session_id.clone(),
            message_id: message_id.clone(),
            text: assistant_message,
        }];

        Ok(parts)
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("Server feature not enabled"))
    }
}
