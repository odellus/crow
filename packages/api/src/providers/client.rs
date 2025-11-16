use super::ProviderConfig;
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs},
    Client,
};

/// OpenAI-compatible client wrapper
pub struct ProviderClient {
    config: ProviderConfig,
    client: Client<OpenAIConfig>,
}

impl ProviderClient {
    /// Create a new provider client from config
    pub fn new(config: ProviderConfig) -> Result<Self, String> {
        // Load .env file
        let _ = dotenvy::dotenv();

        // Get API key from environment
        let api_key = std::env::var(&config.api_key_env)
            .map_err(|_| format!("{} not found in environment", config.api_key_env))?;

        // Configure client
        let openai_config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(&config.base_url);

        let client = Client::with_config(openai_config);

        Ok(Self { config, client })
    }

    /// Send a chat completion request
    pub async fn chat(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        model: Option<&str>,
    ) -> Result<String, String> {
        let model = model.unwrap_or(&self.config.default_model);

        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(messages)
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| format!("API call failed: {}", e))?;

        let message = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| "No response from LLM".to_string())?;

        Ok(message.clone())
    }

    /// Get the provider config
    pub fn config(&self) -> &ProviderConfig {
        &self.config
    }
}
