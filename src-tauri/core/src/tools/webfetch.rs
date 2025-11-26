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
    use std::path::PathBuf;

    fn create_test_context() -> ToolContext {
        ToolContext::new(
            "test-session".to_string(),
            "test-message".to_string(),
            "build".to_string(),
            PathBuf::from("/tmp"),
        )
    }

    // ==================== Tool Interface Tests ====================

    #[tokio::test]
    async fn test_webfetch_tool_name() {
        let tool = WebFetchTool;
        assert_eq!(tool.name(), "webfetch");
    }

    #[tokio::test]
    async fn test_webfetch_tool_description() {
        let tool = WebFetchTool;
        let desc = tool.description();
        assert!(desc.contains("URL"));
        assert!(desc.contains("fetch"));
    }

    #[tokio::test]
    async fn test_webfetch_parameters_schema() {
        let tool = WebFetchTool;
        let schema = tool.parameters_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["url"].is_object());
        assert!(schema["properties"]["prompt"].is_object());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("url")));
    }

    // ==================== Input Validation Tests ====================

    #[tokio::test]
    async fn test_webfetch_missing_url() {
        let tool = WebFetchTool;
        let ctx = create_test_context();

        let result = tool.execute(json!({}), &ctx).await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.unwrap().contains("Invalid input"));
    }

    #[tokio::test]
    async fn test_webfetch_invalid_input_type() {
        let tool = WebFetchTool;
        let ctx = create_test_context();

        let result = tool.execute(json!("not an object"), &ctx).await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    // ==================== HTML Stripping Tests ====================

    #[test]
    fn test_strip_html_simple() {
        let html = "<html><body><p>Hello World</p></body></html>";
        let text = strip_html(html);
        assert_eq!(text, "Hello World");
    }

    #[test]
    fn test_strip_html_script_tags() {
        let html = "<html><script>alert('hi')</script><p>Content</p></html>";
        let text = strip_html(html);
        assert_eq!(text, "Content");
    }

    #[test]
    fn test_strip_html_style_tags() {
        let html = "<html><style>.class { color: red; }</style><p>Styled</p></html>";
        let text = strip_html(html);
        assert_eq!(text, "Styled");
    }

    #[test]
    fn test_strip_html_nested_tags() {
        let html = "<div><span><strong>Nested</strong></span></div>";
        let text = strip_html(html);
        assert_eq!(text, "Nested");
    }

    #[test]
    fn test_strip_html_whitespace_cleanup() {
        let html = "<p>  Line 1  </p>   <p>  Line 2  </p>";
        let text = strip_html(html);
        assert!(text.contains("Line 1"));
        assert!(text.contains("Line 2"));
    }

    #[test]
    fn test_strip_html_empty() {
        let html = "<html><body></body></html>";
        let text = strip_html(html);
        assert!(text.is_empty());
    }

    #[test]
    fn test_strip_html_plain_text() {
        let text = "Plain text without HTML";
        let result = strip_html(text);
        assert_eq!(result, "Plain text without HTML");
    }

    #[test]
    fn test_strip_html_special_characters() {
        let html = "<p>&amp; &lt; &gt;</p>";
        let text = strip_html(html);
        // Note: This doesn't decode HTML entities, just strips tags
        assert!(text.contains("&amp;"));
    }

    #[test]
    fn test_strip_html_multiple_scripts() {
        let html = "<script>one</script><p>A</p><script>two</script><p>B</p>";
        let text = strip_html(html);
        assert!(text.contains("A"));
        assert!(text.contains("B"));
        assert!(!text.contains("one"));
        assert!(!text.contains("two"));
    }

    // ==================== URL Upgrade Tests ====================

    #[tokio::test]
    async fn test_webfetch_upgrades_http_to_https() {
        // We can't easily test the actual fetch without a mock server,
        // but we can verify the URL upgrade logic by checking if it fails
        // with the upgraded URL (which implies the upgrade happened)
        let tool = WebFetchTool;
        let ctx = create_test_context();

        // This will fail to connect, but we can verify the error mentions https
        let result = tool
            .execute(
                json!({
                    "url": "http://nonexistent-domain-xyz123.invalid"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        // The URL in metadata should be upgraded
        if let Some(url) = result.metadata["url"].as_str() {
            assert!(url.starts_with("https://"));
        }
    }

    // ==================== Error Handling Tests ====================

    #[tokio::test]
    async fn test_webfetch_invalid_url() {
        let tool = WebFetchTool;
        let ctx = create_test_context();

        let result = tool
            .execute(
                json!({
                    "url": "https://nonexistent-domain-xyz123.invalid"
                }),
                &ctx,
            )
            .await;

        assert_eq!(result.status, ToolStatus::Error);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_webfetch_with_optional_prompt() {
        let tool = WebFetchTool;
        let ctx = create_test_context();

        // Test that prompt is accepted even if fetch fails
        let result = tool
            .execute(
                json!({
                    "url": "https://nonexistent-domain-xyz123.invalid",
                    "prompt": "Extract the main content"
                }),
                &ctx,
            )
            .await;

        // Will fail but shouldn't error on input parsing
        assert_eq!(result.status, ToolStatus::Error);
        // Error should be about connection, not input
        assert!(result.error.unwrap().contains("Failed to fetch"));
    }
}
