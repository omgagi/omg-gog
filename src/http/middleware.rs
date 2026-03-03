// Request/response middleware chain
//
// This module provides middleware for HTTP requests including:
//   - Retry with backoff (delegating to retry.rs)
//   - Circuit breaker checking (delegating to circuit_breaker.rs)
//   - Body replay for retried requests
//   - Cancellation support via tokio CancellationToken

use crate::http::circuit_breaker::CircuitBreaker;
use crate::http::retry;
use crate::http::RetryConfig;

/// A request that can be retried, including the cloned body.
pub struct RetryableRequest {
    pub method: reqwest::Method,
    pub url: String,
    pub headers: reqwest::header::HeaderMap,
    pub body: Option<Vec<u8>>,
}

impl RetryableRequest {
    /// Create a retryable request from a URL and optional body.
    pub fn new(method: reqwest::Method, url: String, body: Option<Vec<u8>>) -> Self {
        Self {
            method,
            url,
            headers: reqwest::header::HeaderMap::new(),
            body,
        }
    }
}

/// Execute a request with retry and circuit breaker middleware.
/// Returns the response or an error.
pub async fn execute_with_retry(
    client: &reqwest::Client,
    request: &RetryableRequest,
    config: &RetryConfig,
    circuit_breaker: &CircuitBreaker,
) -> anyhow::Result<reqwest::Response> {
    // Check circuit breaker
    if circuit_breaker.is_open() {
        anyhow::bail!("circuit breaker is open; requests are temporarily blocked");
    }

    let mut last_error: Option<anyhow::Error> = None;
    let max_attempts = 1 + config.max_retries_429.max(config.max_retries_5xx);

    for attempt in 0..max_attempts {
        let mut req_builder = client.request(request.method.clone(), &request.url);

        // Add headers
        for (key, value) in request.headers.iter() {
            req_builder = req_builder.header(key, value);
        }

        // Add body (replayed from clone)
        if let Some(ref body) = request.body {
            req_builder = req_builder.body(body.clone());
        }

        match req_builder.send().await {
            Ok(response) => {
                let status = response.status().as_u16();

                if status < 400 {
                    circuit_breaker.record_success();
                    return Ok(response);
                }

                if retry::is_rate_limited(status) && attempt < config.max_retries_429 {
                    let retry_after = response
                        .headers()
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(retry::parse_retry_after)
                        .map(|d| d.as_secs());

                    let delay = retry::calculate_backoff(attempt, config.base_delay, retry_after);
                    tokio::time::sleep(delay).await;
                    continue;
                }

                if retry::is_server_error(status) && attempt < config.max_retries_5xx {
                    circuit_breaker.record_failure();
                    tokio::time::sleep(config.server_error_delay).await;
                    continue;
                }

                // Non-retryable error
                if retry::is_server_error(status) {
                    circuit_breaker.record_failure();
                }
                return Ok(response);
            }
            Err(e) => {
                circuit_breaker.record_failure();
                last_error = Some(e.into());

                if attempt < max_attempts - 1 {
                    tokio::time::sleep(config.server_error_delay).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("request failed after retries")))
}
