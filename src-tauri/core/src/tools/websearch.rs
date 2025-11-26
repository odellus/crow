//! WebSearch tool - searches the internet using SearXNG
//! Based on the smolagents-example search.py implementation

use super::ToolContext;
use super::{Tool, ToolResult, ToolStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone)]
pub struct WebSearchTool;

impl WebSearchTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResult {
    url: String,
    title: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct InfoboxUrl {
    title: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Infobox {
    infobox: String,
    id: String,
    content: String,
    urls: Vec<InfoboxUrl>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearXNGResponse {
    query: String,
    number_of_results: i32,
    results: Vec<SearchResult>,
    #[serde(default)]
    infoboxes: Vec<Infobox>,
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "websearch"
    }

    fn description(&self) -> &str {
        r#"Search the internet using SearXNG.

Usage:
  {"query": "rust web frameworks", "limit": 5}

Arguments:
  - query (required): Your search query for the search engine
  - limit (optional): Number of results to return (default: 5)

Returns a formatted text with search results including titles, URLs, and content snippets.

Example:
  To search for information about Rust web frameworks:
  {"query": "rust web frameworks comparison", "limit": 3}
"#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results to return (default: 5)",
                    "default": 5
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> ToolResult {
        #[derive(Deserialize)]
        struct WebSearchInput {
            query: String,
            #[serde(default = "default_limit")]
            limit: usize,
        }

        fn default_limit() -> usize {
            5
        }

        let search_input: WebSearchInput = match serde_json::from_value(input) {
            Ok(i) => i,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Invalid input: {}", e)),
                    metadata: json!({}),
                };
            }
        };

        // Get SearXNG URL from environment, default to localhost:8082
        let searxng_url =
            std::env::var("SEARXNG_URL").unwrap_or_else(|_| "http://localhost:8082".to_string());

        // Make HTTP request to SearXNG
        let client = reqwest::Client::new();
        let response = match client
            .get(format!("{}/search", searxng_url))
            .query(&[("q", &search_input.query), ("format", &"json".to_string())])
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to connect to SearXNG: {}", e)),
                    metadata: json!({}),
                };
            }
        };

        if !response.status().is_success() {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!(
                    "SearXNG returned error status: {}",
                    response.status()
                )),
                metadata: json!({}),
            };
        }

        let data: SearXNGResponse = match response.json().await {
            Ok(d) => d,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to parse SearXNG response: {}", e)),
                    metadata: json!({}),
                };
            }
        };

        // Format output similar to Python version
        let mut text = String::new();

        // Add infoboxes first
        for infobox in &data.infoboxes {
            text.push_str(&format!("Infobox: {}\n", infobox.infobox));
            text.push_str(&format!("ID: {}\n", infobox.id));
            text.push_str(&format!("Content: {}\n", infobox.content));
            text.push_str("\n");
        }

        // Add search results
        if data.results.is_empty() {
            text.push_str("No results found\n");
        } else {
            for (index, result) in data.results.iter().enumerate() {
                if index >= search_input.limit {
                    break;
                }
                text.push_str(&format!("Title: {}\n", result.title));
                text.push_str(&format!("URL: {}\n", result.url));
                text.push_str(&format!("Content: {}\n", result.content));
                text.push_str("\n");
            }
        }

        ToolResult {
            status: ToolStatus::Completed,
            output: text,
            error: None,
            metadata: json!({
                "query": search_input.query,
                "limit": search_input.limit,
                "total_results": data.number_of_results,
                "returned_results": std::cmp::min(data.results.len(), search_input.limit),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websearch_schema() {
        let tool = WebSearchTool::new();
        assert_eq!(tool.name(), "websearch");
        let params = tool.parameters_schema();
        assert!(params["properties"]["query"].is_object());
        assert!(params["properties"]["limit"].is_object());
    }
}
