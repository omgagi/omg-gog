// Authenticated reqwest client builder
//
// Builds a reqwest::Client configured with:
//   - TLS 1.2+ via rustls
//   - Bearer token injection
//   - User-Agent header
//   - Timeout settings

use std::time::Duration;

/// Default request timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

/// User-Agent string for requests.
pub const USER_AGENT: &str = concat!("omega-google/", env!("CARGO_PKG_VERSION"));

/// Build a bare reqwest client (no auth token).
pub fn build_client() -> anyhow::Result<reqwest::Client> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(DEFAULT_TIMEOUT)
        .min_tls_version(reqwest::tls::Version::TLS_1_2)
        .build()?;
    Ok(client)
}

/// Build an authenticated reqwest client with a bearer token.
pub fn build_authenticated_client(token: &str) -> anyhow::Result<reqwest::Client> {
    use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

    let mut headers = HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth_value)?,
    );

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(DEFAULT_TIMEOUT)
        .min_tls_version(reqwest::tls::Version::TLS_1_2)
        .default_headers(headers)
        .build()?;
    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_client_succeeds() {
        let client = build_client();
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_authenticated_client_succeeds() {
        let client = build_authenticated_client("test-token-12345");
        assert!(client.is_ok());
    }

    #[test]
    fn test_user_agent_contains_version() {
        assert!(USER_AGENT.starts_with("omega-google/"));
    }
}
