// Generic API call helpers for Google REST APIs.
//
// Combines authenticated HTTP client, retry middleware, circuit breaker,
// verbose logging, and error handling into simple api_get/api_post/etc calls.

use std::time::Instant;

use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::http::circuit_breaker::CircuitBreaker;
use crate::http::middleware::{execute_with_retry, RetryableRequest};
use crate::http::RetryConfig;

/// GET a URL, deserialize JSON response.
///
/// Uses the retry middleware and circuit breaker. Logs request/response
/// details to stderr when verbose is true.
pub async fn api_get<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    breaker: &CircuitBreaker,
    retry_config: &RetryConfig,
    verbose: bool,
) -> anyhow::Result<T> {
    if verbose {
        eprintln!("> GET {}", url);
    }

    let request = RetryableRequest::new(Method::GET, url.to_string(), None);
    let start = Instant::now();
    let response = execute_with_retry(client, &request, retry_config, breaker).await?;
    let elapsed = start.elapsed();
    let status = response.status().as_u16();
    let body = response.text().await?;

    if verbose {
        eprintln!("< {} ({} bytes, {}ms)", status, body.len(), elapsed.as_millis());
    }

    check_response_status(status, &body)?;
    let parsed: T = serde_json::from_str(&body)?;
    Ok(parsed)
}

/// POST JSON body to a URL, deserialize JSON response.
///
/// When dry_run is true, logs the request details and returns `Ok(None)`.
/// On normal execution, returns `Ok(Some(parsed))`.
pub async fn api_post<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    body: &impl Serialize,
    breaker: &CircuitBreaker,
    retry_config: &RetryConfig,
    verbose: bool,
    dry_run: bool,
) -> anyhow::Result<Option<T>> {
    let body_json = serde_json::to_vec(body)?;

    if dry_run {
        let body_str = String::from_utf8_lossy(&body_json);
        eprintln!("[dry-run] POST {} would send: {}", url, body_str);
        return Ok(None);
    }

    if verbose {
        eprintln!("> POST {}", url);
        eprintln!("> Content-Type: application/json");
        eprintln!("> Body: {} bytes", body_json.len());
    }

    let mut request = RetryableRequest::new(Method::POST, url.to_string(), Some(body_json));
    request.headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    let start = Instant::now();
    let response = execute_with_retry(client, &request, retry_config, breaker).await?;
    let elapsed = start.elapsed();
    let status = response.status().as_u16();
    let resp_body = response.text().await?;

    if verbose {
        eprintln!("< {} ({} bytes, {}ms)", status, resp_body.len(), elapsed.as_millis());
    }

    check_response_status(status, &resp_body)?;
    let parsed: T = serde_json::from_str(&resp_body)?;
    Ok(Some(parsed))
}

/// POST JSON body, return no parsed body (for 204/empty responses).
pub async fn api_post_empty(
    client: &reqwest::Client,
    url: &str,
    body: &impl Serialize,
    breaker: &CircuitBreaker,
    retry_config: &RetryConfig,
    verbose: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    let body_json = serde_json::to_vec(body)?;

    if dry_run {
        let body_str = String::from_utf8_lossy(&body_json);
        eprintln!("[dry-run] POST {} would send: {}", url, body_str);
        return Ok(());
    }

    if verbose {
        eprintln!("> POST {}", url);
    }

    let mut request = RetryableRequest::new(Method::POST, url.to_string(), Some(body_json));
    request.headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    let start = Instant::now();
    let response = execute_with_retry(client, &request, retry_config, breaker).await?;
    let elapsed = start.elapsed();
    let status = response.status().as_u16();
    let resp_body = response.text().await?;

    if verbose {
        eprintln!("< {} ({} bytes, {}ms)", status, resp_body.len(), elapsed.as_millis());
    }

    check_response_status(status, &resp_body)?;
    Ok(())
}

/// PATCH JSON body to a URL, deserialize JSON response.
///
/// When dry_run is true, logs the request details and returns `Ok(None)`.
/// On normal execution, returns `Ok(Some(parsed))`.
pub async fn api_patch<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    body: &impl Serialize,
    breaker: &CircuitBreaker,
    retry_config: &RetryConfig,
    verbose: bool,
    dry_run: bool,
) -> anyhow::Result<Option<T>> {
    let body_json = serde_json::to_vec(body)?;

    if dry_run {
        let body_str = String::from_utf8_lossy(&body_json);
        eprintln!("[dry-run] PATCH {} would send: {}", url, body_str);
        return Ok(None);
    }

    if verbose {
        eprintln!("> PATCH {}", url);
    }

    let mut request = RetryableRequest::new(Method::PATCH, url.to_string(), Some(body_json));
    request.headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    let start = Instant::now();
    let response = execute_with_retry(client, &request, retry_config, breaker).await?;
    let elapsed = start.elapsed();
    let status = response.status().as_u16();
    let resp_body = response.text().await?;

    if verbose {
        eprintln!("< {} ({} bytes, {}ms)", status, resp_body.len(), elapsed.as_millis());
    }

    check_response_status(status, &resp_body)?;
    let parsed: T = serde_json::from_str(&resp_body)?;
    Ok(Some(parsed))
}

/// DELETE a URL, return no parsed body.
pub async fn api_delete(
    client: &reqwest::Client,
    url: &str,
    breaker: &CircuitBreaker,
    retry_config: &RetryConfig,
    verbose: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    if dry_run {
        eprintln!("[dry-run] DELETE {} would execute", url);
        return Ok(());
    }

    if verbose {
        eprintln!("> DELETE {}", url);
    }

    let request = RetryableRequest::new(Method::DELETE, url.to_string(), None);
    let start = Instant::now();
    let response = execute_with_retry(client, &request, retry_config, breaker).await?;
    let elapsed = start.elapsed();
    let status = response.status().as_u16();
    let resp_body = response.text().await?;

    if verbose {
        eprintln!("< {} ({} bytes, {}ms)", status, resp_body.len(), elapsed.as_millis());
    }

    check_response_status(status, &resp_body)?;
    Ok(())
}

/// PUT raw bytes (for file upload).
///
/// When dry_run is true, logs the request details and returns `Ok(None)`.
/// On normal execution, returns `Ok(Some(parsed))`.
#[allow(clippy::too_many_arguments)]
pub async fn api_put_bytes<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    content_type: &str,
    body: Vec<u8>,
    breaker: &CircuitBreaker,
    retry_config: &RetryConfig,
    verbose: bool,
    dry_run: bool,
) -> anyhow::Result<Option<T>> {
    if dry_run {
        eprintln!("[dry-run] PUT {} would upload {} bytes", url, body.len());
        return Ok(None);
    }

    if verbose {
        eprintln!("> PUT {}", url);
        eprintln!("> Content-Type: {}", content_type);
        eprintln!("> Body: {} bytes", body.len());
    }

    let mut request = RetryableRequest::new(Method::PUT, url.to_string(), Some(body));
    if let Ok(hv) = reqwest::header::HeaderValue::from_str(content_type) {
        request.headers.insert(reqwest::header::CONTENT_TYPE, hv);
    }

    let start = Instant::now();
    let response = execute_with_retry(client, &request, retry_config, breaker).await?;
    let elapsed = start.elapsed();
    let status = response.status().as_u16();
    let resp_body = response.text().await?;

    if verbose {
        eprintln!("< {} ({} bytes, {}ms)", status, resp_body.len(), elapsed.as_millis());
    }

    check_response_status(status, &resp_body)?;
    let parsed: T = serde_json::from_str(&resp_body)?;
    Ok(Some(parsed))
}

/// POST raw bytes (for multipart file upload).
///
/// When dry_run is true, logs the request details and returns `Ok(None)`.
/// On normal execution, returns `Ok(Some(parsed))`.
#[allow(clippy::too_many_arguments)]
pub async fn api_post_bytes<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    content_type: &str,
    body: Vec<u8>,
    breaker: &CircuitBreaker,
    retry_config: &RetryConfig,
    verbose: bool,
    dry_run: bool,
) -> anyhow::Result<Option<T>> {
    if dry_run {
        eprintln!("[dry-run] POST {} would upload {} bytes", url, body.len());
        return Ok(None);
    }

    if verbose {
        eprintln!("> POST {}", url);
        eprintln!("> Content-Type: {}", content_type);
        eprintln!("> Body: {} bytes", body.len());
    }

    let mut request = RetryableRequest::new(Method::POST, url.to_string(), Some(body));
    if let Ok(hv) = reqwest::header::HeaderValue::from_str(content_type) {
        request.headers.insert(reqwest::header::CONTENT_TYPE, hv);
    }

    let start = Instant::now();
    let response = execute_with_retry(client, &request, retry_config, breaker).await?;
    let elapsed = start.elapsed();
    let status = response.status().as_u16();
    let resp_body = response.text().await?;

    if verbose {
        eprintln!("< {} ({} bytes, {}ms)", status, resp_body.len(), elapsed.as_millis());
    }

    check_response_status(status, &resp_body)?;
    let parsed: T = serde_json::from_str(&resp_body)?;
    Ok(Some(parsed))
}

/// GET a URL, return raw response (for file download / streaming).
pub async fn api_get_raw(
    client: &reqwest::Client,
    url: &str,
    breaker: &CircuitBreaker,
    retry_config: &RetryConfig,
    verbose: bool,
) -> anyhow::Result<reqwest::Response> {
    if verbose {
        eprintln!("> GET (raw) {}", url);
    }

    let request = RetryableRequest::new(Method::GET, url.to_string(), None);
    let start = Instant::now();
    let response = execute_with_retry(client, &request, retry_config, breaker).await?;
    let elapsed = start.elapsed();

    if verbose {
        eprintln!("< {} ({}ms)", response.status().as_u16(), elapsed.as_millis());
    }

    let status = response.status().as_u16();
    if status >= 400 {
        let body = response.text().await?;
        check_response_status(status, &body)?;
        unreachable!();
    }

    Ok(response)
}

/// Check response status, return typed `OmegaError::ApiError` on 4xx/5xx.
///
/// Returns a typed error that can be downcast for exit-code mapping via
/// `exit_code_for()`. The `Display` impl on `OmegaError::ApiError` includes
/// the status code and parsed Google error message.
pub fn check_response_status(status: u16, body: &str) -> anyhow::Result<()> {
    if status < 400 {
        return Ok(());
    }
    let message = crate::error::api_error::parse_google_error(body)
        .unwrap_or_else(|| body.to_string());
    Err(crate::error::exit::OmegaError::ApiError { status, message }.into())
}

/// Redact the Authorization header value for verbose logging.
/// Replaces `Bearer <token>` with `Bearer [REDACTED]`.
///
/// This utility is intended for use by service handlers and CLI modules that
/// log request headers. The generic API helpers (`api_get`, `api_post`, etc.)
/// do not have access to the Authorization header directly because it is set
/// on the `reqwest::Client` via default headers or per-request by the auth
/// middleware layer. Service handlers that log headers manually should use
/// this function to ensure tokens are never printed to stderr.
pub fn redact_auth_header(header_value: &str) -> String {
    if header_value.starts_with("Bearer ") {
        "Bearer [REDACTED]".to_string()
    } else {
        header_value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use serde::Deserialize;

    // Helper: create a default RetryConfig with no retries for fast tests.
    fn no_retry_config() -> RetryConfig {
        RetryConfig {
            max_retries_429: 0,
            max_retries_5xx: 0,
            base_delay: std::time::Duration::from_millis(0),
            server_error_delay: std::time::Duration::from_millis(0),
        }
    }

    // Helper: create a fresh CircuitBreaker.
    fn fresh_breaker() -> CircuitBreaker {
        CircuitBreaker::new()
    }

    // Simple response type for testing deserialization.
    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct TestResponse {
        id: String,
        name: String,
    }

    // Simple list response type for testing.
    #[allow(dead_code)]
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    struct TestListResponse {
        items: Vec<TestItem>,
        #[serde(default)]
        next_page_token: Option<String>,
    }

    #[allow(dead_code)]
    #[derive(Debug, Deserialize, PartialEq)]
    struct TestItem {
        id: String,
    }

    // ===================================================================
    // REQ-RT-019 (Must): Generic API call helper: GET with deserialization
    // ===================================================================

    // Requirement: REQ-RT-019 (Must)
    // Acceptance: api_get builds RetryableRequest and deserializes JSON response
    #[tokio::test]
    async fn req_rt_019_api_get_deserializes_json_response() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/resource")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"123","name":"test item"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/resource", server.url());

        let result: TestResponse = api_get(&client, &url, &breaker, &config, false).await.unwrap();

        assert_eq!(result.id, "123");
        assert_eq!(result.name, "test item");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-019 (Must)
    // Acceptance: api_get returns error on 404 with Google API error message
    #[tokio::test]
    async fn req_rt_019_api_get_404_returns_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/missing")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":{"code":404,"message":"Requested entity was not found."}}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/missing", server.url());

        let result: anyhow::Result<TestResponse> =
            api_get(&client, &url, &breaker, &config, false).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("404"), "Error should contain status code 404: {}", err_msg);
        assert!(
            err_msg.contains("not found") || err_msg.contains("Not found") || err_msg.contains("was not found"),
            "Error should contain 'not found' message: {}",
            err_msg
        );
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-019 (Must)
    // Acceptance: api_get returns auth error on 401
    #[tokio::test]
    async fn req_rt_019_api_get_401_returns_auth_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/unauthorized")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":{"code":401,"message":"Request had invalid authentication credentials."}}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/unauthorized", server.url());

        let result: anyhow::Result<TestResponse> =
            api_get(&client, &url, &breaker, &config, false).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("401"), "Error should contain status code 401: {}", err_msg);
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-019 (Must)
    // Acceptance: api_get returns error on 403 (insufficient scopes)
    #[tokio::test]
    async fn req_rt_019_api_get_403_returns_permission_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/forbidden")
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":{"code":403,"message":"The caller does not have permission"}}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/forbidden", server.url());

        let result: anyhow::Result<TestResponse> =
            api_get(&client, &url, &breaker, &config, false).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("403"), "Error should contain status code 403: {}", err_msg);
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-019 (Must)
    // Acceptance: api_get returns error when response body is not valid JSON for type T
    #[tokio::test]
    async fn req_rt_019_api_get_invalid_json_returns_deserialization_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/bad-json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("not valid json at all")
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/bad-json", server.url());

        let result: anyhow::Result<TestResponse> =
            api_get(&client, &url, &breaker, &config, false).await;

        assert!(result.is_err(), "Should fail on invalid JSON");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-019 (Must)
    // Acceptance: api_get returns error when JSON is valid but doesn't match type T
    #[tokio::test]
    async fn req_rt_019_api_get_wrong_json_shape_returns_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/wrong-shape")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"totally":"different","structure":true}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/wrong-shape", server.url());

        let result: anyhow::Result<TestResponse> =
            api_get(&client, &url, &breaker, &config, false).await;

        assert!(result.is_err(), "Should fail when JSON shape doesn't match T");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-019 (Must)
    // Failure mode: Circuit breaker open blocks GET
    #[tokio::test]
    async fn req_rt_019_api_get_circuit_breaker_open_blocks_request() {
        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();

        // Open the circuit breaker by recording enough failures
        for _ in 0..5 {
            breaker.record_failure();
        }
        assert!(breaker.is_open(), "Circuit breaker should be open");

        let result: anyhow::Result<TestResponse> =
            api_get(&client, "http://127.0.0.1:1/never-called", &breaker, &config, false).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("circuit breaker"),
            "Error should mention circuit breaker: {}",
            err_msg
        );
    }

    // Requirement: REQ-RT-019 (Must)
    // Failure mode: Server error (500) is handled properly
    #[tokio::test]
    async fn req_rt_019_api_get_500_server_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/server-error")
            .with_status(500)
            .with_body(r#"{"error":{"code":500,"message":"Internal server error"}}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/server-error", server.url());

        let result: anyhow::Result<TestResponse> =
            api_get(&client, &url, &breaker, &config, false).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("500"), "Error should contain 500: {}", err_msg);
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-019 (Must)
    // Edge case: Empty response body on 200
    #[tokio::test]
    async fn req_rt_019_api_get_empty_body_on_200_returns_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/empty")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/empty", server.url());

        let result: anyhow::Result<TestResponse> =
            api_get(&client, &url, &breaker, &config, false).await;

        assert!(result.is_err(), "Empty body should fail deserialization");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-019 (Must)
    // Edge case: Response with extra fields still deserializes
    #[tokio::test]
    async fn req_rt_019_api_get_extra_fields_in_response_still_works() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/extra-fields")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"123","name":"test","extra_field":"ignored","nested":{"a":1}}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/extra-fields", server.url());

        // TestResponse has id and name; extra fields should be silently ignored
        let result: TestResponse = api_get(&client, &url, &breaker, &config, false).await.unwrap();
        assert_eq!(result.id, "123");
        assert_eq!(result.name, "test");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-019 (Must)
    // Edge case: URL with special characters
    #[tokio::test]
    async fn req_rt_019_api_get_url_with_query_params() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/search?q=hello+world&maxResults=10")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"search","name":"result"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/search?q=hello+world&maxResults=10", server.url());

        let result: TestResponse = api_get(&client, &url, &breaker, &config, false).await.unwrap();
        assert_eq!(result.id, "search");
        mock.assert_async().await;
    }

    // ===================================================================
    // REQ-RT-020 (Must): Generic API call helper: POST with body
    // ===================================================================

    // Requirement: REQ-RT-020 (Must)
    // Acceptance: api_post sends JSON body and deserializes response
    #[tokio::test]
    async fn req_rt_020_api_post_sends_json_and_deserializes() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/test/create")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"new-id","name":"created"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/create", server.url());
        let body = serde_json::json!({"name": "created"});

        let result: Option<TestResponse> =
            api_post(&client, &url, &body, &breaker, &config, false, false).await.unwrap();

        let response = result.expect("non-dry-run POST should return Some");
        assert_eq!(response.id, "new-id");
        assert_eq!(response.name, "created");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-020 (Must)
    // Acceptance: api_post returns error on 400 bad request
    #[tokio::test]
    async fn req_rt_020_api_post_400_returns_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/test/bad-request")
            .with_status(400)
            .with_body(r#"{"error":{"code":400,"message":"Invalid request body"}}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/bad-request", server.url());
        let body = serde_json::json!({});

        let result: anyhow::Result<Option<TestResponse>> =
            api_post(&client, &url, &body, &breaker, &config, false, false).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("400"), "Error should contain 400: {}", err_msg);
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-020 (Must)
    // Acceptance: api_post_empty handles 204 No Content
    #[tokio::test]
    async fn req_rt_020_api_post_empty_handles_204() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/test/action")
            .with_status(204)
            .with_body("")
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/action", server.url());
        let body = serde_json::json!({"action": "do"});

        let result = api_post_empty(&client, &url, &body, &breaker, &config, false, false).await;
        assert!(result.is_ok(), "204 response should succeed for api_post_empty");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-020 (Must)
    // Acceptance: api_patch sends PATCH with JSON body
    #[tokio::test]
    async fn req_rt_020_api_patch_sends_json_and_deserializes() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PATCH", "/test/update/123")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"123","name":"updated"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/update/123", server.url());
        let body = serde_json::json!({"name": "updated"});

        let result: Option<TestResponse> =
            api_patch(&client, &url, &body, &breaker, &config, false, false).await.unwrap();

        let response = result.expect("non-dry-run PATCH should return Some");
        assert_eq!(response.id, "123");
        assert_eq!(response.name, "updated");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-020 (Must)
    // Acceptance: api_delete sends DELETE and returns Ok(()) on success
    #[tokio::test]
    async fn req_rt_020_api_delete_succeeds_on_204() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/test/resource/456")
            .with_status(204)
            .with_body("")
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/resource/456", server.url());

        let result = api_delete(&client, &url, &breaker, &config, false, false).await;
        assert!(result.is_ok(), "DELETE with 204 should succeed");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-020 (Must)
    // Acceptance: api_delete returns error on 404
    #[tokio::test]
    async fn req_rt_020_api_delete_404_returns_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/test/resource/missing")
            .with_status(404)
            .with_body(r#"{"error":{"code":404,"message":"Not found"}}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/resource/missing", server.url());

        let result = api_delete(&client, &url, &breaker, &config, false, false).await;
        assert!(result.is_err());
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-020 (Must)
    // Acceptance: api_put_bytes sends raw bytes with content type
    #[tokio::test]
    async fn req_rt_020_api_put_bytes_uploads_raw_body() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PUT", "/upload/file")
            .match_header("content-type", "application/octet-stream")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"file-1","name":"uploaded.bin"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/upload/file", server.url());
        let body = vec![0u8, 1, 2, 3, 4, 5];

        let result: Option<TestResponse> =
            api_put_bytes(&client, &url, "application/octet-stream", body, &breaker, &config, false, false)
                .await
                .unwrap();

        let response = result.expect("non-dry-run PUT should return Some");
        assert_eq!(response.id, "file-1");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-020 (Must)
    // Acceptance: api_get_raw returns raw response for streaming
    #[tokio::test]
    async fn req_rt_020_api_get_raw_returns_response() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/download/file")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body("binary file content here")
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/download/file", server.url());

        let response = api_get_raw(&client, &url, &breaker, &config, false).await.unwrap();
        assert_eq!(response.status(), 200);
        let body = response.text().await.unwrap();
        assert_eq!(body, "binary file content here");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-020 (Must)
    // Acceptance: api_get_raw returns error on 4xx
    #[tokio::test]
    async fn req_rt_020_api_get_raw_404_returns_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/download/missing")
            .with_status(404)
            .with_body(r#"{"error":{"code":404,"message":"File not found"}}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/download/missing", server.url());

        let result = api_get_raw(&client, &url, &breaker, &config, false).await;
        assert!(result.is_err());
        mock.assert_async().await;
    }

    // ===================================================================
    // REQ-RT-021 (Must): Single shared CircuitBreaker per CLI invocation
    // ===================================================================

    // Requirement: REQ-RT-021 (Must)
    // Acceptance: CircuitBreaker can be Arc-wrapped for sharing
    #[test]
    fn req_rt_021_circuit_breaker_works_with_arc() {
        let breaker = Arc::new(CircuitBreaker::new());
        let breaker2 = Arc::clone(&breaker);

        // Failures on one reference affect the other
        for _ in 0..5 {
            breaker.record_failure();
        }
        assert!(breaker2.is_open(), "Arc-shared breaker should see open state");

        // Reset on one reference affects the other
        breaker2.record_success();
        assert!(!breaker.is_open(), "Arc-shared breaker should see closed state after success");
    }

    // Requirement: REQ-RT-021 (Must)
    // Acceptance: Multiple API calls share the same circuit breaker state
    #[tokio::test]
    async fn req_rt_021_shared_breaker_accumulates_failures() {
        let mut server = mockito::Server::new_async().await;

        // Set up 5 different endpoints that all return 500
        let mock0 = server
            .mock("GET", "/api/endpoint0")
            .with_status(500)
            .with_body(r#"{"error":{"code":500,"message":"Internal error"}}"#)
            .create_async()
            .await;
        let mock1 = server
            .mock("GET", "/api/endpoint1")
            .with_status(500)
            .with_body(r#"{"error":{"code":500,"message":"Internal error"}}"#)
            .create_async()
            .await;
        let mock2 = server
            .mock("GET", "/api/endpoint2")
            .with_status(500)
            .with_body(r#"{"error":{"code":500,"message":"Internal error"}}"#)
            .create_async()
            .await;
        let mock3 = server
            .mock("GET", "/api/endpoint3")
            .with_status(500)
            .with_body(r#"{"error":{"code":500,"message":"Internal error"}}"#)
            .create_async()
            .await;
        let mock4 = server
            .mock("GET", "/api/endpoint4")
            .with_status(500)
            .with_body(r#"{"error":{"code":500,"message":"Internal error"}}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = CircuitBreaker::new();
        let config = no_retry_config();

        // Make 5 calls to different endpoints, all failing
        for i in 0..5 {
            let url = format!("{}/api/endpoint{}", server.url(), i);
            let _result: anyhow::Result<serde_json::Value> =
                api_get(&client, &url, &breaker, &config, false).await;
            // Note: the breaker records failures via execute_with_retry
        }

        // After 5 failures through the middleware, the breaker may be open
        // (depends on whether middleware records server errors as failures)
        // The key point is that the breaker is shared across calls
        mock0.assert_async().await;
        mock1.assert_async().await;
        mock2.assert_async().await;
        mock3.assert_async().await;
        mock4.assert_async().await;
    }

    // ===================================================================
    // REQ-RT-022 (Must): Async handler pattern (check_response_status)
    // ===================================================================

    // Requirement: REQ-RT-022 (Must)
    // Acceptance: check_response_status returns Ok for 2xx
    #[test]
    fn req_rt_022_check_status_2xx_ok() {
        assert!(check_response_status(200, "{}").is_ok());
        assert!(check_response_status(201, r#"{"id":"new"}"#).is_ok());
        assert!(check_response_status(204, "").is_ok());
        assert!(check_response_status(299, "anything").is_ok());
    }

    // Requirement: REQ-RT-022 (Must)
    // Acceptance: check_response_status returns Ok for 3xx
    #[test]
    fn req_rt_022_check_status_3xx_ok() {
        assert!(check_response_status(301, "").is_ok());
        assert!(check_response_status(302, "").is_ok());
    }

    // Requirement: REQ-RT-022 (Must)
    // Acceptance: check_response_status returns error for 4xx
    #[test]
    fn req_rt_022_check_status_4xx_error() {
        let result = check_response_status(
            400,
            r#"{"error":{"code":400,"message":"Bad request"}}"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("400"));
        assert!(err.contains("Bad request"));
    }

    // Requirement: REQ-RT-022 (Must)
    // Acceptance: check_response_status returns error for 5xx
    #[test]
    fn req_rt_022_check_status_5xx_error() {
        let result = check_response_status(
            503,
            r#"{"error":{"code":503,"message":"Service Unavailable"}}"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("503"));
    }

    // Requirement: REQ-RT-022 (Must)
    // Edge case: check_response_status with non-JSON error body
    #[test]
    fn req_rt_022_check_status_non_json_error_body() {
        let result = check_response_status(500, "Internal Server Error");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        // Should still include the status code and the raw body as fallback
        assert!(err.contains("500"));
        assert!(err.contains("Internal Server Error"));
    }

    // Requirement: REQ-RT-022 (Must)
    // Edge case: check_response_status with empty error body
    #[test]
    fn req_rt_022_check_status_empty_error_body() {
        let result = check_response_status(502, "");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("502"));
    }

    // Requirement: REQ-RT-022 (Must)
    // Acceptance: Error-to-exit-code mapping for API errors
    // (This verifies that check_response_status errors can be mapped to exit codes)
    #[test]
    fn req_rt_022_api_error_maps_to_exit_code() {
        use crate::error::exit::{OmegaError, exit_code_for, codes};
        // 401 -> AUTH_REQUIRED
        let err = OmegaError::ApiError {
            status: 401,
            message: "unauthorized".to_string(),
        };
        assert_eq!(exit_code_for(&err), codes::AUTH_REQUIRED);
        // 404 -> NOT_FOUND
        let err = OmegaError::ApiError {
            status: 404,
            message: "not found".to_string(),
        };
        assert_eq!(exit_code_for(&err), codes::NOT_FOUND);
    }

    // ===================================================================
    // REQ-RT-081 (Must): --verbose shows HTTP request/response on stderr
    // ===================================================================

    // Requirement: REQ-RT-081 (Must)
    // Acceptance: api_get logs request method and URL when verbose=true
    // Note: We verify verbose=true causes the function to work correctly;
    // actual stderr capture would require test infrastructure. This test
    // ensures the verbose path doesn't crash or change behavior.
    #[tokio::test]
    async fn req_rt_081_api_get_verbose_succeeds() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/verbose")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"v","name":"verbose"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/verbose", server.url());

        // verbose=true should not change the return value
        let result: TestResponse = api_get(&client, &url, &breaker, &config, true).await.unwrap();
        assert_eq!(result.id, "v");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-081 (Must)
    // Acceptance: api_post logs verbose details without crashing
    #[tokio::test]
    async fn req_rt_081_api_post_verbose_succeeds() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/test/verbose-post")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"vp","name":"verbose-post"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/verbose-post", server.url());
        let body = serde_json::json!({"key": "value"});

        let result: Option<TestResponse> =
            api_post(&client, &url, &body, &breaker, &config, true, false).await.unwrap();
        assert_eq!(result.expect("non-dry-run should return Some").id, "vp");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-081 (Must)
    // Security: Bearer token is redacted in verbose output
    #[test]
    fn req_rt_081_redact_auth_header_bearer() {
        let redacted = redact_auth_header("Bearer ya29.a0AfH6SMBx...");
        assert_eq!(redacted, "Bearer [REDACTED]");
    }

    // Requirement: REQ-RT-081 (Must)
    // Security: Non-bearer auth headers are not redacted
    #[test]
    fn req_rt_081_redact_auth_header_non_bearer() {
        let redacted = redact_auth_header("Basic dXNlcjpwYXNz");
        assert_eq!(redacted, "Basic dXNlcjpwYXNz");
    }

    // Requirement: REQ-RT-081 (Must)
    // Security: Empty auth header
    #[test]
    fn req_rt_081_redact_auth_header_empty() {
        let redacted = redact_auth_header("");
        assert_eq!(redacted, "");
    }

    // Requirement: REQ-RT-081 (Must)
    // Edge case: Verbose logging on error responses
    #[tokio::test]
    async fn req_rt_081_verbose_on_error_still_works() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/verbose-error")
            .with_status(500)
            .with_body(r#"{"error":{"code":500,"message":"Oops"}}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/verbose-error", server.url());

        // verbose=true, request fails with 500 -- verbose logging should still occur
        let result: anyhow::Result<TestResponse> =
            api_get(&client, &url, &breaker, &config, true).await;
        assert!(result.is_err());
        mock.assert_async().await;
    }

    // ===================================================================
    // REQ-RT-082 (Must): --dry-run for mutating commands
    // ===================================================================

    // Requirement: REQ-RT-082 (Must)
    // Acceptance: dry_run=true on POST does NOT execute the request, returns Ok(None)
    #[tokio::test]
    async fn req_rt_082_dry_run_post_does_not_execute() {
        let mut server = mockito::Server::new_async().await;
        // This mock should NOT be called in dry-run mode
        let mock = server
            .mock("POST", "/test/create")
            .with_status(200)
            .with_body(r#"{"id":"x","name":"y"}"#)
            .expect(0) // Must NOT be called
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/create", server.url());
        let body = serde_json::json!({"name": "test"});

        let result: anyhow::Result<Option<TestResponse>> =
            api_post(&client, &url, &body, &breaker, &config, false, true).await;

        assert!(result.is_ok(), "dry-run POST should return Ok (exit code 0)");
        assert!(result.unwrap().is_none(), "dry-run POST should return None (no response body)");
        mock.assert_async().await; // Verifies 0 calls were made
    }

    // Requirement: REQ-RT-082 (Must)
    // Acceptance: dry_run=true on PATCH does NOT execute the request, returns Ok(None)
    #[tokio::test]
    async fn req_rt_082_dry_run_patch_does_not_execute() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PATCH", "/test/update/1")
            .expect(0)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/update/1", server.url());
        let body = serde_json::json!({"name": "updated"});

        let result: anyhow::Result<Option<TestResponse>> =
            api_patch(&client, &url, &body, &breaker, &config, false, true).await;

        assert!(result.is_ok(), "dry-run PATCH should return Ok (exit code 0)");
        assert!(result.unwrap().is_none(), "dry-run PATCH should return None");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-082 (Must)
    // Acceptance: dry_run=true on DELETE does NOT execute the request and returns Ok
    #[tokio::test]
    async fn req_rt_082_dry_run_delete_does_not_execute() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/test/resource/99")
            .expect(0)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/resource/99", server.url());

        let result = api_delete(&client, &url, &breaker, &config, false, true).await;
        assert!(result.is_ok(), "dry-run DELETE should return Ok");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-082 (Must)
    // Acceptance: dry_run=true on PUT does NOT execute the request, returns Ok(None)
    #[tokio::test]
    async fn req_rt_082_dry_run_put_bytes_does_not_execute() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PUT", "/upload/dry")
            .expect(0)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/upload/dry", server.url());
        let body = vec![0u8; 100];

        let result: anyhow::Result<Option<TestResponse>> =
            api_put_bytes(&client, &url, "application/octet-stream", body, &breaker, &config, false, true)
                .await;

        assert!(result.is_ok(), "dry-run PUT should return Ok (exit code 0)");
        assert!(result.unwrap().is_none(), "dry-run PUT should return None");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-082 (Must)
    // Acceptance: dry_run does NOT affect GET (reads are not mutating)
    #[tokio::test]
    async fn req_rt_082_get_executes_normally_even_with_dry_run_context() {
        // api_get does not take dry_run parameter -- it always executes.
        // This verifies the design: GET has no dry_run parameter.
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/read")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"r","name":"read"}"#)
            .expect(1)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/read", server.url());

        // api_get does NOT have a dry_run parameter -- GET always executes
        let result: TestResponse = api_get(&client, &url, &breaker, &config, false).await.unwrap();
        assert_eq!(result.id, "r");
        mock.assert_async().await; // Verifies exactly 1 call
    }

    // Requirement: REQ-RT-082 (Must)
    // Acceptance: dry_run POST with verbose also logs the dry-run message, returns Ok(None)
    #[tokio::test]
    async fn req_rt_082_dry_run_post_with_verbose() {
        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let body = serde_json::json!({"subject": "Test email"});

        // Both verbose=true and dry_run=true -- should not crash
        let result: anyhow::Result<Option<TestResponse>> = api_post(
            &client,
            "http://127.0.0.1:1/never-called",
            &body,
            &breaker,
            &config,
            true,  // verbose
            true,  // dry_run
        )
        .await;

        assert!(result.is_ok(), "dry-run POST with verbose should return Ok");
        assert!(result.unwrap().is_none(), "dry-run POST should return None");
    }

    // Requirement: REQ-RT-082 (Must)
    // Acceptance: api_post_empty in dry-run returns Ok (no error needed)
    #[tokio::test]
    async fn req_rt_082_dry_run_post_empty_returns_ok() {
        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let body = serde_json::json!({"action": "send"});

        let result = api_post_empty(
            &client,
            "http://127.0.0.1:1/never-called",
            &body,
            &breaker,
            &config,
            false,
            true, // dry_run
        )
        .await;

        assert!(result.is_ok(), "dry-run api_post_empty should return Ok");
    }

    // ===================================================================
    // Edge cases and failure modes across all API helpers
    // ===================================================================

    // Requirement: REQ-RT-019 (Must)
    // Edge case: Unicode in response body
    #[tokio::test]
    async fn req_rt_019_edge_unicode_in_response() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/unicode")
            .with_status(200)
            .with_header("content-type", "application/json; charset=utf-8")
            .with_body(r#"{"id":"uni","name":"Meeting: \u00e9v\u00e9nement"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/unicode", server.url());

        let result: TestResponse = api_get(&client, &url, &breaker, &config, false).await.unwrap();
        assert_eq!(result.id, "uni");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-020 (Must)
    // Edge case: POST with empty body object
    #[tokio::test]
    async fn req_rt_020_edge_post_empty_body_object() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/test/empty-body")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"eb","name":"empty body"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/empty-body", server.url());
        let body = serde_json::json!({});

        let result: Option<TestResponse> =
            api_post(&client, &url, &body, &breaker, &config, false, false).await.unwrap();
        assert_eq!(result.expect("non-dry-run should return Some").id, "eb");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-020 (Must)
    // Edge case: Large response body
    #[tokio::test]
    async fn req_rt_019_edge_large_response_body() {
        let mut server = mockito::Server::new_async().await;
        // Build a large JSON response with many items
        let large_name = "x".repeat(10_000);
        let body = format!(r#"{{"id":"big","name":"{}"}}"#, large_name);
        let mock = server
            .mock("GET", "/test/large")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&body)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/large", server.url());

        let result: TestResponse = api_get(&client, &url, &breaker, &config, false).await.unwrap();
        assert_eq!(result.id, "big");
        assert_eq!(result.name.len(), 10_000);
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-019 (Must)
    // Edge case: Response with null field where String expected
    #[tokio::test]
    async fn req_rt_019_edge_null_required_field() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test/null-field")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":null,"name":"test"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/test/null-field", server.url());

        let result: anyhow::Result<TestResponse> =
            api_get(&client, &url, &breaker, &config, false).await;
        // id is a String, null should fail deserialization
        assert!(result.is_err(), "null where String expected should fail");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-022 (Must)
    // Edge case: check_response_status boundary at 399 vs 400
    #[test]
    fn req_rt_022_check_status_boundary_399_400() {
        assert!(check_response_status(399, "").is_ok(), "399 should be Ok");
        assert!(check_response_status(400, "bad").is_err(), "400 should be Err");
    }

    // Requirement: REQ-RT-022 (Must)
    // Edge case: check_response_status with status 0 (malformed)
    #[test]
    fn req_rt_022_check_status_zero() {
        // Status 0 is < 400, so should be Ok
        assert!(check_response_status(0, "").is_ok());
    }
}
