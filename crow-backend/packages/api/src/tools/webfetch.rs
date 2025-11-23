//! WebFetch tool - fetches content from URLs
//! Mirrors OpenCode's webfetch tool

use super::{Tool, ToolContext, ToolResult, ToolStatus};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct WebFetchTool;

#[derive(Deserialize)]
struct WebFetchInput {
    url: String,
    #[serde(default)]
    prompt: Option<String>,
}

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "webfetch"
    }

    fn description(&self) -> &str {
        r#"Fetches content from a specified URL and returns the text content.

Usage:
- Provide a URL to fetch content from
- Optionally provide a prompt to extract specific information
- Returns the page content as text (HTML stripped)

Notes:
- HTTP URLs will be upgraded to HTTPS
- Large pages may be truncated
- Some sites may block automated requests"#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch content from"
                },
                "prompt": {
                    "type": "string",
                    "description": "Optional prompt to extract specific information from the page"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        // Check abort
        if ctx.should_abort() {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some("Aborted".to_string()),
                metadata: json!({}),
            };
        }

        // Parse input
        let fetch_input: WebFetchInput = match serde_json::from_value(input) {
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

        // Upgrade HTTP to HTTPS
        let url = if fetch_input.url.starts_with("http://") {
            fetch_input.url.replacen("http://", "https://", 1)
        } else {
            fetch_input.url.clone()
        };

        // Fetch the URL
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; Crow/1.0)")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let response = match client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to fetch URL: {}", e)),
                    metadata: json!({
                        "url": url,
                    }),
                };
            }
        };

        // Check status
        let status = response.status();
        if !status.is_success() {
            return ToolResult {
                status: ToolStatus::Error,
                output: String::new(),
                error: Some(format!("HTTP error: {}", status)),
                metadata: json!({
                    "url": url,
                    "status": status.as_u16(),
                }),
            };
        }

        // Get content
        let content = match response.text().await {
            Ok(text) => text,
            Err(e) => {
                return ToolResult {
                    status: ToolStatus::Error,
                    output: String::new(),
                    error: Some(format!("Failed to read response: {}", e)),
                    metadata: json!({
                        "url": url,
                    }),
                };
            }
        };

        // Strip HTML tags (simple approach)
        let text_content = strip_html(&content);

        // Truncate if too long
        let max_len = 50000;
        let truncated = if text_content.len() > max_len {
            format!(
                "{}...\n\n[Truncated - {} chars total]",
                &text_content[..max_len],
                text_content.len()
            )
        } else {
            text_content
        };

        ToolResult {
            status: ToolStatus::Completed,
            output: truncated,
            error: None,
            metadata: json!({
                "url": url,
                "length": content.len(),
                "prompt": fetch_input.prompt,
            }),
        }
    }
}

/// Simple HTML tag stripper
fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;

    let lower = html.to_lowercase();
    let chars: Vec<char> = html.chars().collect();
    let lower_chars: Vec<char> = lower.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        // Check for script/style tags
        if i + 7 < lower_chars.len() {
            let slice: String = lower_chars[i..i + 7].iter().collect();
            if slice == "<script" {
                in_script = true;
            } else if slice == "</scrip" {
                in_script = false;
                // Skip to end of tag
                while i < chars.len() && chars[i] != '>' {
                    i += 1;
                }
                i += 1;
                continue;
            }
        }

        if i + 6 < lower_chars.len() {
            let slice: String = lower_chars[i..i + 6].iter().collect();
            if slice == "<style" {
                in_style = true;
            } else if slice == "</styl" {
                in_style = false;
                while i < chars.len() && chars[i] != '>' {
                    i += 1;
                }
                i += 1;
                continue;
            }
        }

        if in_script || in_style {
            i += 1;
            continue;
        }

        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }

        i += 1;
    }

    // Clean up whitespace
    let lines: Vec<&str> = result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html() {
        let html = "<html><body><p>Hello World</p></body></html>";
        let text = strip_html(html);
        assert_eq!(text, "Hello World");
    }

    #[test]
    fn test_strip_script() {
        let html = "<html><script>alert('hi')</script><p>Content</p></html>";
        let text = strip_html(html);
        assert_eq!(text, "Content");
    }

    #[tokio::test]
    async fn test_webfetch_schema() {
        let tool = WebFetchTool;
        assert_eq!(tool.name(), "webfetch");
        let schema = tool.parameters_schema();
        assert!(schema["properties"]["url"].is_object());
    }
}
