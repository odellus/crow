use super::ProviderConfig;
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, ChatCompletionTool, CreateChatCompletionRequestArgs},
    Client,
};
use futures::StreamExt;
use tokio::sync::mpsc;

/// Delta events from streaming LLM response
#[derive(Debug, Clone)]
pub enum StreamDelta {
    /// Text content delta
    Text(String),
    /// Tool call delta (id, name, arguments chunk)
    ToolCall {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments: String,
    },
    /// Usage info (sent at end of stream)
    Usage { input: u64, output: u64 },
    /// Stream finished
    Done,
}

/// OpenAI-compatible client wrapper
#[derive(Clone)]
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

    /// Send a streaming chat completion request with tools
    /// Returns a channel that receives deltas as they arrive
    pub async fn chat_with_tools_stream(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        tools: Vec<ChatCompletionTool>,
        model: Option<&str>,
        tx: mpsc::UnboundedSender<StreamDelta>,
    ) -> Result<(), String> {
        let model = model.unwrap_or(&self.config.default_model);

        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder
            .model(model)
            .messages(messages)
            .stream(true)
            .stream_options(async_openai::types::ChatCompletionStreamOptions {
                include_usage: true,
            });

        if !tools.is_empty() {
            request_builder.tools(tools);
        }

        let request = request_builder
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let mut stream = self
            .client
            .chat()
            .create_stream(request)
            .await
            .map_err(|e| format!("API stream failed: {}", e))?;

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    // Check for usage info (sent in final chunk)
                    if let Some(usage) = &response.usage {
                        let _ = tx.send(StreamDelta::Usage {
                            input: usage.prompt_tokens as u64,
                            output: usage.completion_tokens as u64,
                        });
                    }

                    for choice in &response.choices {
                        // Handle text content
                        if let Some(content) = &choice.delta.content {
                            if !content.is_empty() {
                                let _ = tx.send(StreamDelta::Text(content.clone()));
                            }
                        }

                        // Handle tool calls
                        if let Some(tool_calls) = &choice.delta.tool_calls {
                            for tc in tool_calls {
                                let _ = tx.send(StreamDelta::ToolCall {
                                    index: tc.index as usize,
                                    id: tc.id.clone(),
                                    name: tc.function.as_ref().and_then(|f| f.name.clone()),
                                    arguments: tc
                                        .function
                                        .as_ref()
                                        .and_then(|f| f.arguments.clone())
                                        .unwrap_or_default(),
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Stream error: {}", e);
                    break;
                }
            }
        }

        let _ = tx.send(StreamDelta::Done);
        Ok(())
    }
}
