use super::ProviderConfig;
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, ChatCompletionTool, CreateChatCompletionRequestArgs},
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
        // Try to get API key from auth.json first, fall back to environment
        let api_key = Self::get_api_key(&config)?;

        // Configure client
        let openai_config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(&config.base_url);

        let client = Client::with_config(openai_config);

        Ok(Self { config, client })
    }

    /// Get API key from auth.json or environment
    fn get_api_key(config: &ProviderConfig) -> Result<String, String> {
        // Try auth.json first
        if let Ok(Some(auth_info)) = crate::auth::get("moonshotai") {
            match auth_info {
                crate::auth::AuthInfo::Api { key } => return Ok(key),
                _ => {}
            }
        }

        // Fall back to environment variable
        let _ = dotenvy::dotenv();
        std::env::var(&config.api_key_env).map_err(|_| {
            format!(
                "{} not found in environment or auth.json",
                config.api_key_env
            )
        })
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

    /// Send a chat completion request with tools
    pub async fn chat_with_tools(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        tools: Vec<ChatCompletionTool>,
        model: Option<&str>,
    ) -> Result<async_openai::types::CreateChatCompletionResponse, String> {
        let model = model.unwrap_or(&self.config.default_model);

        eprintln!("[DEBUG] Using model: {}", model);
        eprintln!(
            "[DEBUG] Config default_model: {}",
            self.config.default_model
        );

        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder.model(model).messages(messages);

        if !tools.is_empty() {
            request_builder.tools(tools);
        }

        let request = request_builder
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        self.client
            .chat()
            .create(request)
            .await
            .map_err(|e| format!("API call failed: {}", e))
    }

    /// Get the provider config
    pub fn config(&self) -> &ProviderConfig {
        &self.config
    }
}
