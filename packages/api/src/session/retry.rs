//! Session retry logic with exponential backoff
//! Handles transient failures like rate limits and timeouts

use std::time::Duration;
use tokio::time::sleep;

pub struct SessionRetry;

impl SessionRetry {
    const MAX_RETRIES: u32 = 10;
    const BASE_DELAY_MS: u64 = 1000;
    const MAX_DELAY_MS: u64 = 30000;

    /// Calculate bounded exponential backoff delay
    pub fn get_bounded_delay(retry_count: u32) -> Duration {
        let delay_ms = Self::BASE_DELAY_MS * 2u64.pow(retry_count);
        let bounded = delay_ms.min(Self::MAX_DELAY_MS);
        Duration::from_millis(bounded)
    }

    /// Retry async operation with exponential backoff
    pub async fn with_retry<F, Fut, T>(mut f: F) -> Result<T, String>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        let mut retry_count = 0;

        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if retry_count >= Self::MAX_RETRIES {
                        return Err(format!(
                            "Max retries ({}) exceeded. Last error: {}",
                            Self::MAX_RETRIES,
                            e
                        ));
                    }

                    // Check if error is retryable
                    if !Self::is_retryable(&e) {
                        eprintln!("[RETRY] Non-retryable error: {}", e);
                        return Err(e);
                    }

                    let delay = Self::get_bounded_delay(retry_count);
                    eprintln!(
                        "[RETRY] Attempt {} failed: {}. Retrying in {:?}",
                        retry_count + 1,
                        e,
                        delay
                    );

                    sleep(delay).await;
                    retry_count += 1;
                }
            }
        }
    }

    /// Determine if error is retryable
    fn is_retryable(error: &str) -> bool {
        let error_lower = error.to_lowercase();

        // Retryable: rate limits, timeouts, network errors
        error_lower.contains("rate limit")
            || error_lower.contains("timeout")
            || error_lower.contains("timed out")
            || error_lower.contains("connection")
            || error_lower.contains("503")
            || error_lower.contains("429")
            || error_lower.contains("502")
            || error_lower.contains("504")
            || error_lower.contains("temporary")
            || error_lower.contains("unavailable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_delay() {
        assert_eq!(SessionRetry::get_bounded_delay(0).as_millis(), 1000);
        assert_eq!(SessionRetry::get_bounded_delay(1).as_millis(), 2000);
        assert_eq!(SessionRetry::get_bounded_delay(2).as_millis(), 4000);
        assert_eq!(SessionRetry::get_bounded_delay(3).as_millis(), 8000);

        // Should be capped at MAX_DELAY_MS (30000)
        assert_eq!(SessionRetry::get_bounded_delay(10).as_millis(), 30000);
        assert_eq!(SessionRetry::get_bounded_delay(20).as_millis(), 30000);
    }

    #[test]
    fn test_is_retryable() {
        assert!(SessionRetry::is_retryable("Rate limit exceeded"));
        assert!(SessionRetry::is_retryable("Connection timeout"));
        assert!(SessionRetry::is_retryable("HTTP 429"));
        assert!(SessionRetry::is_retryable("Service unavailable"));
        assert!(SessionRetry::is_retryable("503 error"));

        assert!(!SessionRetry::is_retryable("Invalid API key"));
        assert!(!SessionRetry::is_retryable("Bad request"));
        assert!(!SessionRetry::is_retryable("404 not found"));
    }

    #[tokio::test]
    async fn test_retry_success_on_second_attempt() {
        let mut attempts = 0;

        let result = SessionRetry::with_retry(|| async {
            attempts += 1;
            if attempts == 1 {
                Err("timeout".to_string())
            } else {
                Ok("success")
            }
        })
        .await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts, 2);
    }

    #[tokio::test]
    async fn test_retry_fails_on_non_retryable() {
        let result =
            SessionRetry::with_retry(|| async { Err::<(), String>("Invalid API key".to_string()) })
                .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid API key"));
    }
}
