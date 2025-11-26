use serde::{Deserialize, Serialize};

/// Configuration for an OpenAI-compatible provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Display name for the provider
    pub name: String,
    /// API base URL (e.g., "https://api.moonshot.ai/v1" or "https://api.openai.com/v1")
    pub base_url: String,
    /// Environment variable name for the API key
    pub api_key_env: String,
    /// Default model to use
    pub default_model: String,
}

impl ProviderConfig {
    /// Create a Moonshot/Kimi provider config
    pub fn moonshot() -> Self {
        Self {
            name: "Moonshot (Kimi)".to_string(),
            base_url: "https://api.moonshot.ai/v1".to_string(),
            api_key_env: "MOONSHOT_API_KEY".to_string(),
            default_model: "kimi-k2-thinking".to_string(),
        }
    }

    /// Create an OpenAI provider config
    pub fn openai() -> Self {
        Self {
            name: "OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key_env: "OPENAI_API_KEY".to_string(),
            default_model: "gpt-4o-mini".to_string(),
        }
    }

    /// Create a custom provider config (e.g., local llama.cpp)
    pub fn custom(
        name: String,
        base_url: String,
        api_key_env: String,
        default_model: String,
    ) -> Self {
        Self {
            name,
            base_url,
            api_key_env,
            default_model,
        }
    }
}
