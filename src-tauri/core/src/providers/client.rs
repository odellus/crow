use super::ProviderConfig;
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, ChatCompletionTool, CreateChatCompletionRequestArgs},
    Client,
};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// Custom stream chunk that includes reasoning_content
#[derive(Debug, serde::Deserialize)]
struct StreamChunkDelta {
    content: Option<String>,
    reasoning_content: Option<String>,
    tool_calls: Option<Vec<StreamToolCallChunk>>,
    role: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct StreamToolCallChunk {
    index: usize,
    id: Option<String>,
    function: Option<StreamFunctionChunk>,
}

#[derive(Debug, serde::Deserialize)]
struct StreamFunctionChunk {
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct StreamChoice {
    delta: StreamChunkDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct CompletionTokensDetails {
    reasoning_tokens: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
struct StreamUsage {
    prompt_tokens: u64,
    completion_tokens: u64,
    completion_tokens_details: Option<CompletionTokensDetails>,
}

#[derive(Debug, serde::Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
    usage: Option<StreamUsage>,
}

/// Delta events from streaming LLM response
#[derive(Debug, Clone)]
pub enum StreamDelta {
    /// Text content delta
    Text(String),
    /// Reasoning/thinking content delta (for reasoning models like kimi-k2-thinking)
    Reasoning(String),
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
        request_builder.model(model).messages(messages.clone());

        if !tools.is_empty() {
            request_builder.tools(tools.clone());
        }

        let request = request_builder
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        // Log full request if CROW_VERBOSE_LOG is set
        if std::env::var("CROW_VERBOSE_LOG").is_ok() {
            let log_dir = dirs::data_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("crow")
                .join("requests");
            let _ = std::fs::create_dir_all(&log_dir);

            let timestamp = chrono::Local::now().format("%Y%m%dT%H%M%S").to_string();
            let log_file = log_dir.join(format!("{}-request.json", timestamp));

            let log_data = serde_json::json!({
                "timestamp": chrono::Local::now().to_rfc3339(),
                "model": model,
                "messages": messages.iter().map(|m| {
                    match m {
                        ChatCompletionRequestMessage::System(s) => serde_json::json!({
                            "role": "system",
                            "content": s.content
                        }),
                        ChatCompletionRequestMessage::User(u) => serde_json::json!({
                            "role": "user",
                            "content": format!("{:?}", u.content)
                        }),
                        ChatCompletionRequestMessage::Assistant(a) => serde_json::json!({
                            "role": "assistant",
                            "content": a.content,
                            "tool_calls": a.tool_calls
                        }),
                        ChatCompletionRequestMessage::Tool(t) => serde_json::json!({
                            "role": "tool",
                            "tool_call_id": t.tool_call_id,
                            "content": t.content
                        }),
                        _ => serde_json::json!({"role": "unknown"})
                    }
                }).collect::<Vec<_>>(),
                "tools": tools.iter().map(|t| {
                    serde_json::json!({
                        "name": t.function.name,
                        "description": t.function.description,
                        "parameters": t.function.parameters
                    })
                }).collect::<Vec<_>>(),
                "tool_count": tools.len()
            });

            if let Ok(json) = serde_json::to_string_pretty(&log_data) {
                let _ = std::fs::write(&log_file, json);
                tracing::debug!("Request logged to: {}", log_file.display());
            }
        }

        tracing::debug!("Calling LLM API...");
        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| format!("API call failed: {}", e))?;
        tracing::debug!("LLM API returned");

        // Log response if CROW_VERBOSE_LOG is set
        if std::env::var("CROW_VERBOSE_LOG").is_ok() {
            tracing::debug!("Logging response to disk...");
            let log_dir = dirs::data_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("crow")
                .join("requests");

            let timestamp = chrono::Local::now().format("%Y%m%dT%H%M%S").to_string();
            let log_file = log_dir.join(format!("{}-response.json", timestamp));

            let response_data = serde_json::json!({
                "timestamp": chrono::Local::now().to_rfc3339(),
                "model": model,
                "id": response.id,
                "choices": response.choices.iter().map(|c| {
                    serde_json::json!({
                        "index": c.index,
                        "finish_reason": format!("{:?}", c.finish_reason),
                        "message": {
                            "role": format!("{:?}", c.message.role),
                            "content": c.message.content,
                            "tool_calls": c.message.tool_calls.as_ref().map(|tcs| {
                                tcs.iter().map(|tc| {
                                    serde_json::json!({
                                        "id": tc.id,
                                        "type": format!("{:?}", tc.r#type),
                                        "function": {
                                            "name": tc.function.name,
                                            "arguments": tc.function.arguments
                                        }
                                    })
                                }).collect::<Vec<_>>()
                            })
                        }
                    })
                }).collect::<Vec<_>>(),
                "usage": response.usage.as_ref().map(|u| {
                    serde_json::json!({
                        "prompt_tokens": u.prompt_tokens,
                        "completion_tokens": u.completion_tokens,
                        "total_tokens": u.total_tokens
                    })
                })
            });

            if let Ok(json) = serde_json::to_string_pretty(&response_data) {
                let _ = std::fs::write(&log_file, json);
                tracing::debug!("Response logged to: {}", log_file.display());
            }
        }

        Ok(response)
    }

    /// Get the provider config
    pub fn config(&self) -> &ProviderConfig {
        &self.config
    }

    /// Send a streaming chat completion request with tools
    /// Returns a channel that receives deltas as they arrive
    /// Supports mid-stream cancellation via the cancellation token
    pub async fn chat_with_tools_stream(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        tools: Vec<ChatCompletionTool>,
        model: Option<&str>,
        tx: mpsc::UnboundedSender<StreamDelta>,
        cancellation: Option<CancellationToken>,
    ) -> Result<(), String> {
        let model = model.unwrap_or(&self.config.default_model);

        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder
            .model(model)
            .messages(messages.clone())
            .stream(true)
            .stream_options(async_openai::types::ChatCompletionStreamOptions {
                include_usage: true,
            });

        if !tools.is_empty() {
            request_builder.tools(tools.clone());
        }

        let request = request_builder
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        // Log full request if CROW_VERBOSE_LOG is set
        if std::env::var("CROW_VERBOSE_LOG").is_ok() {
            tracing::debug!("Logging request to disk...");
            let log_dir = dirs::data_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("crow")
                .join("requests");
            if let Err(e) = std::fs::create_dir_all(&log_dir) {
                tracing::warn!("Failed to create request log dir: {}", e);
            }

            let timestamp = chrono::Local::now().format("%Y%m%dT%H%M%S").to_string();
            let log_file = log_dir.join(format!("{}-request.json", timestamp));

            let log_data = serde_json::json!({
                "timestamp": chrono::Local::now().to_rfc3339(),
                "model": model,
                "messages": messages.iter().map(|m| {
                    match m {
                        ChatCompletionRequestMessage::System(s) => serde_json::json!({
                            "role": "system",
                            "content": s.content
                        }),
                        ChatCompletionRequestMessage::User(u) => serde_json::json!({
                            "role": "user",
                            "content": format!("{:?}", u.content)
                        }),
                        ChatCompletionRequestMessage::Assistant(a) => serde_json::json!({
                            "role": "assistant",
                            "content": a.content,
                            "tool_calls": a.tool_calls
                        }),
                        ChatCompletionRequestMessage::Tool(t) => serde_json::json!({
                            "role": "tool",
                            "tool_call_id": t.tool_call_id,
                            "content": t.content
                        }),
                        _ => serde_json::json!({"role": "unknown"})
                    }
                }).collect::<Vec<_>>(),
                "tools": tools.iter().map(|t| {
                    serde_json::json!({
                        "name": t.function.name,
                        "description": t.function.description,
                        "parameters": t.function.parameters
                    })
                }).collect::<Vec<_>>(),
                "tool_count": tools.len()
            });

            if let Ok(json) = serde_json::to_string_pretty(&log_data) {
                let _ = std::fs::write(&log_file, json);
                tracing::debug!("Request logged to: {}", log_file.display());
            }
        }

        // Use raw HTTP to capture reasoning_content which async-openai doesn't support
        let api_key = Self::get_api_key(&self.config)?;
        let http_client = reqwest::Client::new();

        // Build request body
        let tools_json: Vec<serde_json::Value> = tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.function.name,
                        "description": t.function.description,
                        "parameters": t.function.parameters
                    }
                })
            })
            .collect();

        let messages_json: Vec<serde_json::Value> = messages.iter().map(|m| {
            match m {
                ChatCompletionRequestMessage::System(s) => serde_json::json!({
                    "role": "system",
                    "content": s.content
                }),
                ChatCompletionRequestMessage::User(u) => {
                    let content = match &u.content {
                        async_openai::types::ChatCompletionRequestUserMessageContent::Text(t) => t.clone(),
                        async_openai::types::ChatCompletionRequestUserMessageContent::Array(parts) => {
                            parts.iter().filter_map(|p| {
                                if let async_openai::types::ChatCompletionRequestUserMessageContentPart::Text(t) = p {
                                    Some(t.text.clone())
                                } else {
                                    None
                                }
                            }).collect::<Vec<_>>().join("")
                        }
                    };
                    serde_json::json!({
                        "role": "user",
                        "content": content
                    })
                },
                ChatCompletionRequestMessage::Assistant(a) => {
                    let mut msg = serde_json::json!({
                        "role": "assistant",
                    });
                    if let Some(content) = &a.content {
                        msg["content"] = serde_json::json!(content);
                    }
                    if let Some(tool_calls) = &a.tool_calls {
                        msg["tool_calls"] = serde_json::json!(tool_calls);
                    }
                    msg
                },
                ChatCompletionRequestMessage::Tool(t) => serde_json::json!({
                    "role": "tool",
                    "tool_call_id": t.tool_call_id,
                    "content": t.content
                }),
                _ => serde_json::json!({"role": "unknown"})
            }
        }).collect();

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages_json,
            "stream": true,
            "stream_options": {"include_usage": true}
        });

        if !tools.is_empty() {
            body["tools"] = serde_json::json!(tools_json);
        }

        let response = http_client
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, text));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        // Accumulate response for logging
        let mut accumulated_text = String::new();
        let mut accumulated_tool_calls: std::collections::HashMap<usize, (String, String, String)> =
            std::collections::HashMap::new();
        let mut usage_info: Option<(u64, u64, Option<u64>)> = None;

        loop {
            // Check for cancellation before each chunk
            if let Some(ref token) = cancellation {
                if token.is_cancelled() {
                    let _ = tx.send(StreamDelta::Done);
                    return Err("Stream aborted".to_string());
                }
            }

            // Use select to allow cancellation to interrupt waiting for next chunk
            let result = if let Some(ref token) = cancellation {
                tokio::select! {
                    biased;
                    _ = token.cancelled() => {
                        let _ = tx.send(StreamDelta::Done);
                        return Err("Stream aborted".to_string());
                    }
                    item = stream.next() => item,
                }
            } else {
                stream.next().await
            };

            let Some(result) = result else {
                break;
            };

            let bytes = result.map_err(|e| format!("Stream read error: {}", e))?;
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            // Process complete SSE lines
            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer = buffer[pos + 1..].to_string();

                if line.is_empty() || line.starts_with(':') {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        break;
                    }

                    // Parse the JSON chunk
                    if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                        // Debug: log raw response to see all fields
                        if std::env::var("CROW_DEBUG_STREAM").is_ok() {
                            eprintln!("[STREAM DEBUG] {:?}", chunk);
                        }

                        // Check for usage info (sent in final chunk)
                        if let Some(usage) = &chunk.usage {
                            let reasoning = usage
                                .completion_tokens_details
                                .as_ref()
                                .and_then(|d| d.reasoning_tokens);
                            usage_info =
                                Some((usage.prompt_tokens, usage.completion_tokens, reasoning));
                            let _ = tx.send(StreamDelta::Usage {
                                input: usage.prompt_tokens,
                                output: usage.completion_tokens,
                            });
                        }

                        for choice in &chunk.choices {
                            // Handle reasoning content (thinking tokens)
                            if let Some(reasoning) = &choice.delta.reasoning_content {
                                if !reasoning.is_empty() {
                                    let _ = tx.send(StreamDelta::Reasoning(reasoning.clone()));
                                }
                            }

                            // Handle text content
                            if let Some(content) = &choice.delta.content {
                                if !content.is_empty() {
                                    accumulated_text.push_str(content);
                                    let _ = tx.send(StreamDelta::Text(content.clone()));
                                }
                            }

                            // Handle tool calls
                            if let Some(tool_calls) = &choice.delta.tool_calls {
                                for tc in tool_calls {
                                    let entry =
                                        accumulated_tool_calls.entry(tc.index).or_insert_with(
                                            || (String::new(), String::new(), String::new()),
                                        );
                                    if let Some(id) = &tc.id {
                                        entry.0 = id.clone();
                                    }
                                    if let Some(func) = &tc.function {
                                        if let Some(name) = &func.name {
                                            entry.1 = name.clone();
                                        }
                                        if let Some(args) = &func.arguments {
                                            entry.2.push_str(args);
                                        }
                                    }

                                    let _ = tx.send(StreamDelta::ToolCall {
                                        index: tc.index,
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
                }
            }
        }

        // Log accumulated response if CROW_VERBOSE_LOG is set
        if std::env::var("CROW_VERBOSE_LOG").is_ok() {
            let log_dir = dirs::data_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("crow")
                .join("requests");

            let timestamp = chrono::Local::now().format("%Y%m%dT%H%M%S").to_string();
            let log_file = log_dir.join(format!("{}-stream-response.json", timestamp));

            let tool_calls_json: Vec<_> = accumulated_tool_calls
                .iter()
                .map(|(idx, (id, name, args))| {
                    serde_json::json!({
                        "index": idx,
                        "id": id,
                        "function": {
                            "name": name,
                            "arguments": args
                        }
                    })
                })
                .collect();

            let response_data = serde_json::json!({
                "timestamp": chrono::Local::now().to_rfc3339(),
                "model": model,
                "streaming": true,
                "content": if accumulated_text.is_empty() { None } else { Some(&accumulated_text) },
                "tool_calls": if tool_calls_json.is_empty() { None } else { Some(tool_calls_json) },
                "usage": usage_info.map(|(input, output, reasoning)| {
                    let mut usage = serde_json::json!({
                        "prompt_tokens": input,
                        "completion_tokens": output,
                        "total_tokens": input + output
                    });
                    if let Some(r) = reasoning {
                        usage["reasoning_tokens"] = serde_json::json!(r);
                    }
                    usage
                })
            });

            if let Ok(json) = serde_json::to_string_pretty(&response_data) {
                let _ = std::fs::write(&log_file, json);
                tracing::debug!("Stream response logged to: {}", log_file.display());
            }
        }

        let _ = tx.send(StreamDelta::Done);
        Ok(())
    }
}
