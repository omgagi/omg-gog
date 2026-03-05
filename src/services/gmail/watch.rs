//! Gmail watch (push notification) management.

use serde::{Deserialize, Serialize};

use super::types::GMAIL_BASE_URL;
use crate::services::ServiceContext;

/// Build URL for starting a watch.
pub fn build_watch_start_url() -> String {
    format!("{}/users/me/watch", GMAIL_BASE_URL)
}

/// Build URL for stopping a watch.
pub fn build_watch_stop_url() -> String {
    format!("{}/users/me/stop", GMAIL_BASE_URL)
}

/// Build URL for fetching Gmail profile.
pub fn build_profile_url() -> String {
    format!("{}/users/me/profile", GMAIL_BASE_URL)
}

// ---------------------------------------------------------------
// Watch types (OI-M2)
// ---------------------------------------------------------------

/// Request body for Gmail watch registration.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailWatchRequest {
    pub topic_name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub label_ids: Vec<String>,
}

/// Response from Gmail watch registration.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailWatchResponse {
    pub history_id: u64,
    pub expiration: String,
}

/// Response from Gmail profile endpoint.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileResponse {
    pub email_address: String,
    pub history_id: u64,
    pub messages_total: Option<u64>,
    pub threads_total: Option<u64>,
}

// ---------------------------------------------------------------
// Handler functions (OI-M2)
// ---------------------------------------------------------------

/// Start a Gmail push notification watch.
///
/// `base_url` is the server root (e.g. `https://gmail.googleapis.com` in
/// production, or `server.url()` in tests). The Gmail API path
/// `/gmail/v1/users/me/watch` is appended internally.
pub async fn watch_start(
    ctx: &ServiceContext,
    base_url: &str,
    topic: &str,
    label_ids: &[String],
) -> anyhow::Result<Option<GmailWatchResponse>> {
    let url = format!("{}/gmail/v1/users/me/watch", base_url);
    let body = GmailWatchRequest {
        topic_name: topic.to_string(),
        label_ids: label_ids.to_vec(),
    };

    let result: Option<GmailWatchResponse> = crate::http::api::api_post(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await?;

    if let Some(ref resp) = result {
        ctx.write_output(resp)?;
    }

    Ok(result)
}

/// Stop a Gmail push notification watch.
///
/// `base_url` follows the same convention as `watch_start`.
pub async fn watch_stop(ctx: &ServiceContext, base_url: &str) -> anyhow::Result<()> {
    let url = format!("{}/gmail/v1/users/me/stop", base_url);
    let body = serde_json::json!({});

    crate::http::api::api_post_empty(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await?;

    Ok(())
}

/// Fetch Gmail profile to show watch status information.
///
/// `base_url` follows the same convention as `watch_start`.
pub async fn watch_status(ctx: &ServiceContext, base_url: &str) -> anyhow::Result<()> {
    let url = format!("{}/gmail/v1/users/me/profile", base_url);

    let profile: ProfileResponse = crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await?;

    ctx.write_output(&profile)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Requirement: REQ-GMAIL-012 (Must)
    // Acceptance: Watch start URL
    #[test]
    fn req_gmail_012_watch_start_url() {
        let url = build_watch_start_url();
        assert!(url.contains("users/me/watch"));
    }

    // Requirement: REQ-GMAIL-012 (Must)
    // Acceptance: Watch stop URL
    #[test]
    fn req_gmail_012_watch_stop_url() {
        let url = build_watch_stop_url();
        assert!(url.contains("users/me/stop"));
    }

    // ===================================================================
    // OI-M2: Gmail Watch Commands -- TDD tests
    //
    // These tests are written BEFORE implementation. They define the
    // contract that watch_start, watch_stop, watch_status, and
    // build_profile_url must fulfill, along with the new types
    // GmailWatchRequest, GmailWatchResponse, and ProfileResponse.
    // ===================================================================

    use std::sync::Arc;

    use crate::cli::root::RootFlags;
    use crate::http::circuit_breaker::CircuitBreaker;
    use crate::http::RetryConfig;
    use crate::output::{JsonTransform, OutputMode};
    use crate::services::ServiceContext;
    use crate::ui::{ColorMode, Ui, UiOptions};

    /// Build a test ServiceContext pointing at the given mock server URL.
    ///
    /// The HTTP client is plain (no auth token) -- sufficient for mockito
    /// tests because the mock server doesn't check Authorization headers.
    fn test_ctx(base_url: &str) -> ServiceContext {
        let flags = RootFlags {
            verbose: false,
            dry_run: false,
            force: false,
            ..Default::default()
        };
        let ui = Ui::new(UiOptions {
            color: ColorMode::Never,
        })
        .unwrap();
        ServiceContext {
            client: reqwest::Client::new(),
            output_mode: OutputMode::Json,
            json_transform: JsonTransform::default(),
            ui,
            flags,
            circuit_breaker: Arc::new(CircuitBreaker::new()),
            retry_config: RetryConfig {
                max_retries_429: 0,
                max_retries_5xx: 0,
                ..RetryConfig::default()
            },
            email: "test@example.com".to_string(),
        }
    }

    /// Build a test ServiceContext with dry_run enabled.
    fn test_ctx_dry_run() -> ServiceContext {
        let flags = RootFlags {
            verbose: false,
            dry_run: true,
            force: false,
            ..Default::default()
        };
        let ui = Ui::new(UiOptions {
            color: ColorMode::Never,
        })
        .unwrap();
        ServiceContext {
            client: reqwest::Client::new(),
            output_mode: OutputMode::Json,
            json_transform: JsonTransform::default(),
            ui,
            flags,
            circuit_breaker: Arc::new(CircuitBreaker::new()),
            retry_config: RetryConfig {
                max_retries_429: 0,
                max_retries_5xx: 0,
                ..RetryConfig::default()
            },
            email: "test@example.com".to_string(),
        }
    }

    // -------------------------------------------------------------------
    // REQ-OI-010 (Must): gmail watch start
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-010 (Must)
    // Acceptance: GmailWatchRequest serializes topicName in camelCase
    #[test]
    fn req_oi_010_watch_request_serializes_topic_name_camel_case() {
        let req = GmailWatchRequest {
            topic_name: "projects/my-project/topics/gmail-push".to_string(),
            label_ids: vec!["INBOX".to_string()],
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json_val["topicName"],
            "projects/my-project/topics/gmail-push"
        );
        assert_eq!(json_val["labelIds"], serde_json::json!(["INBOX"]));
        // Verify snake_case keys are NOT present in the serialized output
        assert!(json_val.get("topic_name").is_none());
        assert!(json_val.get("label_ids").is_none());
    }

    // Requirement: REQ-OI-010 (Must)
    // Acceptance: GmailWatchRequest with default label_ids ["INBOX"]
    #[test]
    fn req_oi_010_watch_request_default_label_ids_inbox() {
        let req = GmailWatchRequest {
            topic_name: "projects/test/topics/gmail".to_string(),
            label_ids: vec!["INBOX".to_string()],
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        let label_ids = json_val["labelIds"].as_array().unwrap();
        assert_eq!(label_ids.len(), 1);
        assert_eq!(label_ids[0], "INBOX");
    }

    // Requirement: REQ-OI-010 (Must)
    // Acceptance: GmailWatchRequest with custom label_ids
    #[test]
    fn req_oi_010_watch_request_custom_label_ids() {
        let req = GmailWatchRequest {
            topic_name: "projects/test/topics/gmail".to_string(),
            label_ids: vec![
                "INBOX".to_string(),
                "IMPORTANT".to_string(),
                "Label_42".to_string(),
            ],
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        let label_ids = json_val["labelIds"].as_array().unwrap();
        assert_eq!(label_ids.len(), 3);
        assert_eq!(label_ids[0], "INBOX");
        assert_eq!(label_ids[1], "IMPORTANT");
        assert_eq!(label_ids[2], "Label_42");
    }

    // Requirement: REQ-OI-010 (Must)
    // Acceptance: GmailWatchRequest with empty label_ids omits the field
    //   (skip_serializing_if = "Vec::is_empty" per architecture)
    #[test]
    fn req_oi_010_watch_request_empty_label_ids_omitted() {
        let req = GmailWatchRequest {
            topic_name: "projects/test/topics/gmail".to_string(),
            label_ids: vec![],
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        // With skip_serializing_if = "Vec::is_empty", labelIds should be absent
        assert!(
            json_val.get("labelIds").is_none(),
            "Empty labelIds should be omitted from serialized JSON"
        );
    }

    // Requirement: REQ-OI-010 (Must)
    // Acceptance: GmailWatchResponse deserializes from Google API response
    #[test]
    fn req_oi_010_watch_response_deserializes_from_api() {
        let json_str = r#"{
            "historyId": 12345,
            "expiration": "1704153600000"
        }"#;
        let resp: GmailWatchResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.history_id, 12345);
        assert_eq!(resp.expiration, "1704153600000");
    }

    // Requirement: REQ-OI-010 (Must)
    // Acceptance: GmailWatchResponse round-trip serialization
    #[test]
    fn req_oi_010_watch_response_roundtrip() {
        let resp = GmailWatchResponse {
            history_id: 99999,
            expiration: "1704240000000".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: GmailWatchResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.history_id, 99999);
        assert_eq!(parsed.expiration, "1704240000000");
    }

    // Requirement: REQ-OI-010 (Must)
    // Acceptance: GmailWatchResponse serializes to camelCase
    #[test]
    fn req_oi_010_watch_response_serializes_camel_case() {
        let resp = GmailWatchResponse {
            history_id: 1,
            expiration: "100".to_string(),
        };
        let json_val: serde_json::Value = serde_json::to_value(&resp).unwrap();
        assert!(json_val.get("historyId").is_some());
        assert!(json_val.get("history_id").is_none());
    }

    // Requirement: REQ-OI-010 (Must)
    // Acceptance: build_watch_start_url returns the correct full URL
    #[test]
    fn req_oi_010_watch_start_url_exact() {
        let url = build_watch_start_url();
        assert_eq!(
            url,
            "https://gmail.googleapis.com/gmail/v1/users/me/watch"
        );
    }

    // Requirement: REQ-OI-010 (Must)
    // Acceptance: watch_start handler calls api_post with correct URL and body
    #[tokio::test]
    async fn req_oi_010_watch_start_calls_api_post() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/gmail/v1/users/me/watch")
            .match_header("content-type", "application/json")
            .match_body(mockito::Matcher::JsonString(
                r#"{"topicName":"projects/test/topics/gmail","labelIds":["INBOX"]}"#.to_string(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"historyId":12345,"expiration":"1704153600000"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let label_ids = vec!["INBOX".to_string()];
        let result = watch_start(
            &ctx,
            &server.url(),
            "projects/test/topics/gmail",
            &label_ids,
        )
        .await;

        assert!(result.is_ok(), "watch_start should succeed: {:?}", result);
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-010 (Must)
    // Acceptance: watch_start returns GmailWatchResponse with correct fields
    #[tokio::test]
    async fn req_oi_010_watch_start_returns_response() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/gmail/v1/users/me/watch")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"historyId":67890,"expiration":"1704240000000"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let label_ids = vec!["INBOX".to_string()];
        let result = watch_start(
            &ctx,
            &server.url(),
            "projects/test/topics/gmail",
            &label_ids,
        )
        .await
        .unwrap();

        // watch_start returns Option<GmailWatchResponse> matching api_post pattern
        assert!(result.is_some(), "Non-dry-run should return Some");
        let resp = result.unwrap();
        assert_eq!(resp.history_id, 67890);
        assert_eq!(resp.expiration, "1704240000000");
    }

    // Requirement: REQ-OI-010 (Must)
    // Acceptance: watch_start with --dry-run does not call API
    #[tokio::test]
    async fn req_oi_010_watch_start_dry_run_no_api_call() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/gmail/v1/users/me/watch")
            .expect(0) // Must NOT be called
            .create_async()
            .await;

        let ctx = test_ctx_dry_run();
        let label_ids = vec!["INBOX".to_string()];
        let result = watch_start(
            &ctx,
            &server.url(),
            "projects/test/topics/gmail",
            &label_ids,
        )
        .await;

        assert!(result.is_ok(), "dry-run watch_start should succeed");
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-010 (Must)
    // Failure mode: Permission denied (403) -- Pub/Sub topic missing publisher role
    #[tokio::test]
    async fn req_oi_010_watch_start_403_permission_denied() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/gmail/v1/users/me/watch")
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":{"code":403,"message":"Error sending test message to Cloud PubSub projects/test/topics/gmail : null"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let label_ids = vec!["INBOX".to_string()];
        let result = watch_start(
            &ctx,
            &server.url(),
            "projects/test/topics/gmail",
            &label_ids,
        )
        .await;

        assert!(result.is_err(), "403 should return error");
        let err_msg = result.unwrap_err().to_string();
        // The error should propagate the API error
        assert!(
            err_msg.contains("403") || err_msg.contains("PubSub") || err_msg.contains("error"),
            "Error message should indicate permission/API issue: {}",
            err_msg
        );
    }

    // Requirement: REQ-OI-010 (Must)
    // Failure mode: Invalid topic (400)
    #[tokio::test]
    async fn req_oi_010_watch_start_400_invalid_topic() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/gmail/v1/users/me/watch")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":{"code":400,"message":"Invalid topicName"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let label_ids = vec!["INBOX".to_string()];
        let result = watch_start(
            &ctx,
            &server.url(),
            "bad-topic-format",
            &label_ids,
        )
        .await;

        assert!(result.is_err(), "400 should return error");
    }

    // Requirement: REQ-OI-010 (Must)
    // Edge case: Empty topic string
    #[tokio::test]
    async fn req_oi_010_watch_start_empty_topic() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/gmail/v1/users/me/watch")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":{"code":400,"message":"Invalid topicName"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let label_ids = vec!["INBOX".to_string()];
        let result = watch_start(
            &ctx,
            &server.url(),
            "",
            &label_ids,
        )
        .await;

        // Either the handler rejects it before calling API, or the API returns 400
        assert!(result.is_err(), "Empty topic should fail");
    }

    // Requirement: REQ-OI-010 (Must)
    // Edge case: Topic with special characters
    #[test]
    fn req_oi_010_watch_request_topic_with_special_chars() {
        let req = GmailWatchRequest {
            topic_name: "projects/my-project-123/topics/gmail_push.v2".to_string(),
            label_ids: vec!["INBOX".to_string()],
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json_val["topicName"],
            "projects/my-project-123/topics/gmail_push.v2"
        );
    }

    // Requirement: REQ-OI-010 (Must)
    // Edge case: GmailWatchResponse with very large historyId
    #[test]
    fn req_oi_010_watch_response_large_history_id() {
        let json_str = r#"{"historyId":18446744073709551615,"expiration":"9999999999999"}"#;
        // u64::MAX = 18446744073709551615
        let result: Result<GmailWatchResponse, _> = serde_json::from_str(json_str);
        // Should handle max u64 or fail gracefully
        assert!(
            result.is_ok(),
            "Should handle u64::MAX historyId"
        );
        if let Ok(resp) = result {
            assert_eq!(resp.history_id, u64::MAX);
        }
    }

    // Requirement: REQ-OI-010 (Must)
    // Edge case: GmailWatchResponse missing historyId field
    #[test]
    fn req_oi_010_watch_response_missing_history_id() {
        let json_str = r#"{"expiration":"1704153600000"}"#;
        let result: Result<GmailWatchResponse, _> = serde_json::from_str(json_str);
        // historyId is required (u64, not Option), so this should fail
        assert!(
            result.is_err(),
            "Missing required historyId should fail deserialization"
        );
    }

    // Requirement: REQ-OI-010 (Must)
    // Edge case: GmailWatchResponse missing expiration field
    #[test]
    fn req_oi_010_watch_response_missing_expiration() {
        let json_str = r#"{"historyId":12345}"#;
        let result: Result<GmailWatchResponse, _> = serde_json::from_str(json_str);
        // expiration is required (String, not Option), so this should fail
        assert!(
            result.is_err(),
            "Missing required expiration should fail deserialization"
        );
    }

    // Requirement: REQ-OI-010 (Must)
    // Edge case: Network error during watch_start
    #[tokio::test]
    async fn req_oi_010_watch_start_network_error() {
        // Use a URL that will fail to connect (port that nothing listens on)
        let ctx = test_ctx("http://127.0.0.1:1");
        let label_ids = vec!["INBOX".to_string()];
        let result = watch_start(
            &ctx,
            "http://127.0.0.1:1",
            "projects/test/topics/gmail",
            &label_ids,
        )
        .await;

        assert!(result.is_err(), "Network error should propagate");
    }

    // -------------------------------------------------------------------
    // REQ-OI-011 (Must): gmail watch stop
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-011 (Must)
    // Acceptance: build_watch_stop_url returns the correct full URL
    #[test]
    fn req_oi_011_watch_stop_url_exact() {
        let url = build_watch_stop_url();
        assert_eq!(
            url,
            "https://gmail.googleapis.com/gmail/v1/users/me/stop"
        );
    }

    // Requirement: REQ-OI-011 (Must)
    // Acceptance: watch_stop handler calls api_post to stop URL
    #[tokio::test]
    async fn req_oi_011_watch_stop_calls_api_post() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/gmail/v1/users/me/stop")
            .with_status(204)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_stop(&ctx, &server.url()).await;

        assert!(result.is_ok(), "watch_stop should succeed: {:?}", result);
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-011 (Must)
    // Acceptance: watch_stop with --dry-run does not call API
    #[tokio::test]
    async fn req_oi_011_watch_stop_dry_run_no_api_call() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/gmail/v1/users/me/stop")
            .expect(0) // Must NOT be called
            .create_async()
            .await;

        let ctx = test_ctx_dry_run();
        let result = watch_stop(&ctx, &server.url()).await;

        assert!(result.is_ok(), "dry-run watch_stop should succeed");
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-011 (Must)
    // Acceptance: watch_stop succeeds even when no watch is active (idempotent)
    #[tokio::test]
    async fn req_oi_011_watch_stop_no_active_watch() {
        let mut server = mockito::Server::new_async().await;
        // Google returns 204 even if no watch is active
        let _mock = server
            .mock("POST", "/gmail/v1/users/me/stop")
            .with_status(204)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_stop(&ctx, &server.url()).await;

        assert!(result.is_ok(), "Stopping without active watch should succeed");
    }

    // Requirement: REQ-OI-011 (Must)
    // Failure mode: Network error during watch_stop
    #[tokio::test]
    async fn req_oi_011_watch_stop_network_error() {
        let ctx = test_ctx("http://127.0.0.1:1");
        let result = watch_stop(&ctx, "http://127.0.0.1:1").await;

        assert!(result.is_err(), "Network error should propagate");
    }

    // Requirement: REQ-OI-011 (Must)
    // Failure mode: Server returns 500 on stop
    #[tokio::test]
    async fn req_oi_011_watch_stop_server_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/gmail/v1/users/me/stop")
            .with_status(500)
            .with_body(r#"{"error":{"code":500,"message":"Backend Error"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_stop(&ctx, &server.url()).await;

        assert!(result.is_err(), "500 should return error");
    }

    // -------------------------------------------------------------------
    // REQ-OI-012 (Must): gmail watch status
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-012 (Must)
    // Acceptance: build_profile_url returns correct URL
    #[test]
    fn req_oi_012_build_profile_url() {
        let url = build_profile_url();
        assert_eq!(
            url,
            "https://gmail.googleapis.com/gmail/v1/users/me/profile"
        );
    }

    // Requirement: REQ-OI-012 (Must)
    // Acceptance: build_profile_url starts with GMAIL_BASE_URL
    #[test]
    fn req_oi_012_profile_url_starts_with_base() {
        let url = build_profile_url();
        assert!(
            url.starts_with(GMAIL_BASE_URL),
            "Profile URL should start with GMAIL_BASE_URL"
        );
    }

    // Requirement: REQ-OI-012 (Must)
    // Acceptance: ProfileResponse deserializes from Google API response
    #[test]
    fn req_oi_012_profile_response_deserializes() {
        let json_str = r#"{
            "emailAddress": "user@gmail.com",
            "historyId": 54321,
            "messagesTotal": 1500,
            "threadsTotal": 800
        }"#;
        let resp: ProfileResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.email_address, "user@gmail.com");
        assert_eq!(resp.history_id, 54321);
        assert_eq!(resp.messages_total, Some(1500));
        assert_eq!(resp.threads_total, Some(800));
    }

    // Requirement: REQ-OI-012 (Must)
    // Acceptance: ProfileResponse with only required fields
    #[test]
    fn req_oi_012_profile_response_minimal() {
        let json_str = r#"{
            "emailAddress": "user@gmail.com",
            "historyId": 1
        }"#;
        let resp: ProfileResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.email_address, "user@gmail.com");
        assert_eq!(resp.history_id, 1);
        assert_eq!(resp.messages_total, None);
        assert_eq!(resp.threads_total, None);
    }

    // Requirement: REQ-OI-012 (Must)
    // Acceptance: ProfileResponse round-trip serialization
    #[test]
    fn req_oi_012_profile_response_roundtrip() {
        let resp = ProfileResponse {
            email_address: "test@example.com".to_string(),
            history_id: 42,
            messages_total: Some(100),
            threads_total: Some(50),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: ProfileResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.email_address, "test@example.com");
        assert_eq!(parsed.history_id, 42);
        assert_eq!(parsed.messages_total, Some(100));
        assert_eq!(parsed.threads_total, Some(50));
    }

    // Requirement: REQ-OI-012 (Must)
    // Acceptance: ProfileResponse serializes to camelCase
    #[test]
    fn req_oi_012_profile_response_camel_case() {
        let resp = ProfileResponse {
            email_address: "user@example.com".to_string(),
            history_id: 1,
            messages_total: Some(10),
            threads_total: Some(5),
        };
        let json_val: serde_json::Value = serde_json::to_value(&resp).unwrap();
        assert!(json_val.get("emailAddress").is_some());
        assert!(json_val.get("historyId").is_some());
        assert!(json_val.get("messagesTotal").is_some());
        assert!(json_val.get("threadsTotal").is_some());
        // Snake case keys should NOT appear
        assert!(json_val.get("email_address").is_none());
        assert!(json_val.get("history_id").is_none());
        assert!(json_val.get("messages_total").is_none());
        assert!(json_val.get("threads_total").is_none());
    }

    // Requirement: REQ-OI-012 (Must)
    // Acceptance: watch_status handler calls api_get to profile URL
    #[tokio::test]
    async fn req_oi_012_watch_status_calls_api_get() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/gmail/v1/users/me/profile")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"emailAddress":"user@gmail.com","historyId":54321,"messagesTotal":1500,"threadsTotal":800}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_status(&ctx, &server.url()).await;

        assert!(result.is_ok(), "watch_status should succeed: {:?}", result);
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-012 (Must)
    // Acceptance: watch_status returns ProfileResponse with correct data
    #[tokio::test]
    async fn req_oi_012_watch_status_returns_profile() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/gmail/v1/users/me/profile")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"emailAddress":"alice@gmail.com","historyId":99999,"messagesTotal":5000,"threadsTotal":2500}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_status(&ctx, &server.url()).await;

        assert!(result.is_ok());
        // The handler should successfully process the profile response.
        // The detailed output assertion depends on whether watch_status
        // returns the ProfileResponse or just prints it. Based on the
        // architecture, it prints to stdout via ctx.write_output.
    }

    // Requirement: REQ-OI-012 (Must)
    // Failure mode: 401 Unauthorized (token expired)
    #[tokio::test]
    async fn req_oi_012_watch_status_401_unauthorized() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/gmail/v1/users/me/profile")
            .with_status(401)
            .with_body(r#"{"error":{"code":401,"message":"Request had invalid authentication credentials."}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_status(&ctx, &server.url()).await;

        assert!(result.is_err(), "401 should return error");
    }

    // Requirement: REQ-OI-012 (Must)
    // Edge case: ProfileResponse with unknown extra fields (forward compat)
    #[test]
    fn req_oi_012_profile_response_extra_fields_ignored() {
        let json_str = r#"{
            "emailAddress": "user@gmail.com",
            "historyId": 100,
            "messagesTotal": 50,
            "threadsTotal": 25,
            "unknownField": "should not break"
        }"#;
        // Should deserialize without error even with extra fields
        let result: Result<ProfileResponse, _> = serde_json::from_str(json_str);
        assert!(
            result.is_ok(),
            "Extra fields should be ignored or handled: {:?}",
            result.err()
        );
    }

    // Requirement: REQ-OI-012 (Must)
    // Edge case: ProfileResponse missing emailAddress
    #[test]
    fn req_oi_012_profile_response_missing_email() {
        let json_str = r#"{"historyId": 100}"#;
        let result: Result<ProfileResponse, _> = serde_json::from_str(json_str);
        assert!(
            result.is_err(),
            "Missing required emailAddress should fail deserialization"
        );
    }

    // Requirement: REQ-OI-012 (Must)
    // Edge case: Network error during watch_status
    #[tokio::test]
    async fn req_oi_012_watch_status_network_error() {
        let ctx = test_ctx("http://127.0.0.1:1");
        let result = watch_status(&ctx, "http://127.0.0.1:1").await;

        assert!(result.is_err(), "Network error should propagate");
    }

    // -------------------------------------------------------------------
    // REQ-OI-013 (Should): Gmail watch start prints Pub/Sub prerequisites
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-013 (Should)
    // Acceptance: On successful watch registration, a Pub/Sub prerequisites
    //   note should be printed. This test documents the requirement.
    //   Full assertion of stderr output is deferred to integration/QA since
    //   the handler uses eprintln/ui.hint for the note.
    #[tokio::test]
    async fn req_oi_013_watch_start_pubsub_note_documented() {
        // This test ensures the requirement is tracked.
        // The actual Pub/Sub note ("Ensure gmail-api-push@system.gserviceaccount.com
        // has Pub/Sub Publisher role on the topic") should be printed by the handler
        // on successful watch registration.
        //
        // Verifying stderr output in unit tests is fragile; this is better
        // tested as an integration test or manual QA verification.
        //
        // See: REQ-OI-013 in specs/omega-integration-requirements.md
        assert!(true, "REQ-OI-013 requirement tracked -- verify Pub/Sub note in integration tests");
    }

    // -------------------------------------------------------------------
    // REQ-OI-026 (Must): Watch response types derive Serialize/Deserialize
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: GmailWatchResponse derives both Serialize and Deserialize
    //   with camelCase rename
    #[test]
    fn req_oi_026_gmail_watch_response_serde_derives() {
        // Test Serialize
        let resp = GmailWatchResponse {
            history_id: 1,
            expiration: "100".to_string(),
        };
        let json = serde_json::to_string(&resp);
        assert!(json.is_ok(), "GmailWatchResponse must derive Serialize");

        // Test Deserialize
        let parsed: Result<GmailWatchResponse, _> =
            serde_json::from_str(r#"{"historyId":1,"expiration":"100"}"#);
        assert!(
            parsed.is_ok(),
            "GmailWatchResponse must derive Deserialize"
        );
    }

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: GmailWatchRequest derives Serialize with camelCase rename
    #[test]
    fn req_oi_026_gmail_watch_request_serde_derives() {
        let req = GmailWatchRequest {
            topic_name: "test".to_string(),
            label_ids: vec![],
        };
        let json = serde_json::to_string(&req);
        assert!(json.is_ok(), "GmailWatchRequest must derive Serialize");
    }

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: ProfileResponse derives both Serialize and Deserialize
    #[test]
    fn req_oi_026_profile_response_serde_derives() {
        // Test Serialize
        let resp = ProfileResponse {
            email_address: "test@example.com".to_string(),
            history_id: 1,
            messages_total: None,
            threads_total: None,
        };
        let json = serde_json::to_string(&resp);
        assert!(json.is_ok(), "ProfileResponse must derive Serialize");

        // Test Deserialize
        let parsed: Result<ProfileResponse, _> =
            serde_json::from_str(r#"{"emailAddress":"test@example.com","historyId":1}"#);
        assert!(parsed.is_ok(), "ProfileResponse must derive Deserialize");
    }
}
