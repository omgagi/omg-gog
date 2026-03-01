pub mod client;
pub mod retry;
pub mod circuit_breaker;
pub mod middleware;

use std::time::Duration;

/// Retry configuration matching gogcli defaults.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries_429: u32,
    pub max_retries_5xx: u32,
    pub base_delay: Duration,
    pub server_error_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries_429: 3,
            max_retries_5xx: 1,
            base_delay: Duration::from_secs(1),
            server_error_delay: Duration::from_secs(1),
        }
    }
}
