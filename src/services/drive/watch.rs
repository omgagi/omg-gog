//! Drive watch (push notification) management.
//!
//! Provides handlers for registering, stopping, and querying Drive push
//! notification watches via Google's Channel API.

use crate::services::common::{
    ChannelStopRequest, StartPageTokenResponse, WatchChannelRequest, WatchChannelResponse,
    WatchParams,
};
use crate::services::ServiceContext;

/// Build URL for getting the start page token for change tracking.
pub fn build_start_page_token_url(base_url: &str) -> String {
    format!("{}/drive/v3/changes/startPageToken", base_url)
}

/// Build URL for starting a drive changes watch.
///
/// `page_token` is the start page token obtained from `getStartPageToken`.
pub fn build_drive_watch_url(base_url: &str, page_token: &str) -> String {
    format!(
        "{}/drive/v3/changes/watch?pageToken={}",
        base_url, page_token
    )
}

/// Build URL for stopping a drive watch channel.
pub fn build_drive_stop_url(base_url: &str) -> String {
    format!("{}/drive/v3/channels/stop", base_url)
}

/// Fetch the current start page token for Drive change tracking.
pub async fn get_start_page_token(
    ctx: &ServiceContext,
    base_url: &str,
) -> anyhow::Result<StartPageTokenResponse> {
    let url = build_start_page_token_url(base_url);

    let resp: StartPageTokenResponse = crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await?;

    Ok(resp)
}

/// Start a Drive push notification watch.
///
/// First fetches the start page token, then registers a web_hook watch
/// on the changes endpoint. Returns the channel response and page token.
pub async fn watch_start(
    ctx: &ServiceContext,
    base_url: &str,
    callback_url: &str,
) -> anyhow::Result<Option<(WatchChannelResponse, String)>> {
    // Step 1: Get start page token
    let token_resp = get_start_page_token(ctx, base_url).await?;
    let page_token = token_resp.start_page_token;

    // Step 2: Register watch
    let url = build_drive_watch_url(base_url, &page_token);
    let channel_id = uuid::Uuid::new_v4().to_string();

    let body = WatchChannelRequest {
        id: channel_id,
        channel_type: "web_hook".to_string(),
        address: callback_url.to_string(),
        params: Some(WatchParams {
            ttl: "604800".to_string(),
        }),
    };

    let result: Option<WatchChannelResponse> = crate::http::api::api_post(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await?;

    match result {
        Some(resp) => {
            ctx.write_output(&resp)?;
            Ok(Some((resp, page_token)))
        }
        None => Ok(None),
    }
}

/// Stop a Drive push notification watch.
pub async fn watch_stop(
    ctx: &ServiceContext,
    base_url: &str,
    channel_id: &str,
    resource_id: &str,
) -> anyhow::Result<()> {
    let url = build_drive_stop_url(base_url);
    let body = ChannelStopRequest {
        id: channel_id.to_string(),
        resource_id: resource_id.to_string(),
    };

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

/// Show Drive watch status by fetching the current start page token.
pub async fn watch_status(ctx: &ServiceContext, base_url: &str) -> anyhow::Result<()> {
    let resp = get_start_page_token(ctx, base_url).await?;
    ctx.write_output(&resp)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===================================================================
    // OI-M3: Drive Watch Commands -- TDD tests
    //
    // These tests are written BEFORE implementation. They define the
    // contract that watch_start, watch_stop, watch_status, and the
    // URL builders must fulfill.
    // ===================================================================

    use std::sync::Arc;

    use crate::cli::root::RootFlags;
    use crate::http::circuit_breaker::CircuitBreaker;
    use crate::http::RetryConfig;
    use crate::output::{JsonTransform, OutputMode};
    use crate::services::drive::types::DRIVE_BASE_URL;
    use crate::services::ServiceContext;
    use crate::ui::{ColorMode, Ui, UiOptions};

    /// Build a test ServiceContext.
    fn test_ctx(base_url: &str) -> ServiceContext {
        let _ = base_url;
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
    // REQ-OI-017 (Must): drive watch start -- URL builders
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-017 (Must)
    // Acceptance: build_start_page_token_url returns correct URL
    #[test]
    fn req_oi_017_start_page_token_url() {
        let url = build_start_page_token_url("https://www.googleapis.com");
        assert_eq!(
            url,
            "https://www.googleapis.com/drive/v3/changes/startPageToken"
        );
    }

    // Requirement: REQ-OI-017 (Must)
    // Acceptance: build_start_page_token_url with test server
    #[test]
    fn req_oi_017_start_page_token_url_test_server() {
        let url = build_start_page_token_url("http://127.0.0.1:9999");
        assert_eq!(
            url,
            "http://127.0.0.1:9999/drive/v3/changes/startPageToken"
        );
    }

    // Requirement: REQ-OI-017 (Must)
    // Acceptance: build_drive_watch_url includes pageToken query param
    #[test]
    fn req_oi_017_watch_url_with_page_token() {
        let url = build_drive_watch_url("https://www.googleapis.com", "12345");
        assert_eq!(
            url,
            "https://www.googleapis.com/drive/v3/changes/watch?pageToken=12345"
        );
    }

    // Requirement: REQ-OI-017 (Must)
    // Acceptance: build_drive_watch_url with test server
    #[test]
    fn req_oi_017_watch_url_test_server() {
        let url = build_drive_watch_url("http://127.0.0.1:1234", "token-abc");
        assert_eq!(
            url,
            "http://127.0.0.1:1234/drive/v3/changes/watch?pageToken=token-abc"
        );
    }

    // Requirement: REQ-OI-017 (Must)
    // Edge case: build_drive_watch_url with empty page token
    #[test]
    fn req_oi_017_watch_url_empty_page_token() {
        let url = build_drive_watch_url("https://www.googleapis.com", "");
        assert_eq!(
            url,
            "https://www.googleapis.com/drive/v3/changes/watch?pageToken="
        );
    }

    // -------------------------------------------------------------------
    // REQ-OI-017 (Must): drive watch start -- handler tests
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-017 (Must)
    // Acceptance: watch_start first calls getStartPageToken, then changes/watch
    #[tokio::test]
    async fn req_oi_017_watch_start_two_sequential_api_calls() {
        let mut server = mockito::Server::new_async().await;

        // Mock 1: GET startPageToken
        let mock_token = server
            .mock("GET", "/drive/v3/changes/startPageToken")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"startPageToken":"42"}"#)
            .create_async()
            .await;

        // Mock 2: POST changes/watch with pageToken=42
        let mock_watch = server
            .mock("POST", "/drive/v3/changes/watch?pageToken=42")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"ch-uuid","resourceId":"res-123","expiration":"1704153600000"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook").await;

        assert!(result.is_ok(), "watch_start should succeed: {:?}", result);
        mock_token.assert_async().await;
        mock_watch.assert_async().await;
    }

    // Requirement: REQ-OI-017 (Must)
    // Acceptance: watch_start returns WatchChannelResponse AND start page token
    #[tokio::test]
    async fn req_oi_017_watch_start_returns_response_and_token() {
        let mut server = mockito::Server::new_async().await;

        let _mock_token = server
            .mock("GET", "/drive/v3/changes/startPageToken")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"startPageToken":"99"}"#)
            .create_async()
            .await;

        let _mock_watch = server
            .mock("POST", "/drive/v3/changes/watch?pageToken=99")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"channel-abc","resourceId":"resource-xyz","expiration":"1704240000000"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook")
            .await
            .unwrap();

        assert!(result.is_some(), "Non-dry-run should return Some");
        let (resp, page_token) = result.unwrap();
        assert_eq!(resp.id, "channel-abc");
        assert_eq!(resp.resource_id, "resource-xyz");
        assert_eq!(resp.expiration, Some("1704240000000".to_string()));
        assert_eq!(page_token, "99");
    }

    // Requirement: REQ-OI-017 (Must)
    // Acceptance: watch_start request body contains UUID v4, web_hook, TTL
    #[tokio::test]
    async fn req_oi_017_watch_start_request_body_format() {
        let mut server = mockito::Server::new_async().await;

        let _mock_token = server
            .mock("GET", "/drive/v3/changes/startPageToken")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"startPageToken":"1"}"#)
            .create_async()
            .await;

        let mock_watch = server
            .mock("POST", "/drive/v3/changes/watch?pageToken=1")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex(r#""type"\s*:\s*"web_hook""#.to_string()),
                mockito::Matcher::Regex(r#""address"\s*:\s*"https://example.com/hook""#.to_string()),
                mockito::Matcher::Regex(r#""ttl"\s*:\s*"604800""#.to_string()),
                // UUID v4 format
                mockito::Matcher::Regex(
                    r#""id"\s*:\s*"[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}""#
                        .to_string(),
                ),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"ch","resourceId":"res"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook").await;

        assert!(result.is_ok(), "watch_start should succeed: {:?}", result);
        mock_watch.assert_async().await;
    }

    // Requirement: REQ-OI-017 (Must)
    // Acceptance: watch_start with --dry-run still fetches page token but skips watch POST
    #[tokio::test]
    async fn req_oi_017_watch_start_dry_run() {
        let mut server = mockito::Server::new_async().await;

        // GET startPageToken is still called (not a mutating operation)
        let mock_token = server
            .mock("GET", "/drive/v3/changes/startPageToken")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"startPageToken":"1"}"#)
            .create_async()
            .await;

        // POST changes/watch should NOT be called (dry-run handled by api_post)
        // Note: api_post in dry-run mode returns None without calling the API,
        // but the request is still built. The mock is used to verify no call.
        let mock_watch = server
            .mock("POST", mockito::Matcher::Regex(r"/drive/v3/changes/watch.*".to_string()))
            .expect(0)
            .create_async()
            .await;

        let ctx = test_ctx_dry_run();
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook").await;

        assert!(result.is_ok(), "dry-run watch_start should succeed");
        mock_token.assert_async().await;
        mock_watch.assert_async().await;
    }

    // Requirement: REQ-OI-017 (Must)
    // Failure mode: getStartPageToken fails (network error)
    #[tokio::test]
    async fn req_oi_017_watch_start_page_token_network_error() {
        let ctx = test_ctx("http://127.0.0.1:1");
        let result = watch_start(&ctx, "http://127.0.0.1:1", "https://example.com/hook").await;

        assert!(
            result.is_err(),
            "Network error on getStartPageToken should propagate"
        );
    }

    // Requirement: REQ-OI-017 (Must)
    // Failure mode: getStartPageToken succeeds but changes/watch fails (403)
    #[tokio::test]
    async fn req_oi_017_watch_start_403_domain_not_verified() {
        let mut server = mockito::Server::new_async().await;

        let _mock_token = server
            .mock("GET", "/drive/v3/changes/startPageToken")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"startPageToken":"1"}"#)
            .create_async()
            .await;

        let _mock_watch = server
            .mock("POST", mockito::Matcher::Regex(r"/drive/v3/changes/watch.*".to_string()))
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":{"code":403,"message":"Push notifications are not allowed for the destination"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://unverified.example.com/hook").await;

        assert!(result.is_err(), "403 should return error");
    }

    // Requirement: REQ-OI-017 (Must)
    // Failure mode: Invalid page token (400) -- stale or malformed token
    #[tokio::test]
    async fn req_oi_017_watch_start_400_invalid_page_token() {
        let mut server = mockito::Server::new_async().await;

        let _mock_token = server
            .mock("GET", "/drive/v3/changes/startPageToken")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"startPageToken":"stale-token"}"#)
            .create_async()
            .await;

        let _mock_watch = server
            .mock("POST", mockito::Matcher::Regex(r"/drive/v3/changes/watch.*".to_string()))
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":{"code":400,"message":"Invalid pageToken"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook").await;

        assert!(result.is_err(), "400 should return error");
    }

    // Requirement: REQ-OI-017 (Must)
    // Edge case: Server returns 500 on getStartPageToken
    #[tokio::test]
    async fn req_oi_017_watch_start_page_token_server_error() {
        let mut server = mockito::Server::new_async().await;

        let _mock_token = server
            .mock("GET", "/drive/v3/changes/startPageToken")
            .with_status(500)
            .with_body(r#"{"error":{"code":500,"message":"Backend Error"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook").await;

        assert!(result.is_err(), "500 on getStartPageToken should propagate");
    }

    // Requirement: REQ-OI-017 (Must)
    // Edge case: WatchChannelResponse with no expiration
    #[tokio::test]
    async fn req_oi_017_watch_start_response_no_expiration() {
        let mut server = mockito::Server::new_async().await;

        let _mock_token = server
            .mock("GET", "/drive/v3/changes/startPageToken")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"startPageToken":"7"}"#)
            .create_async()
            .await;

        let _mock_watch = server
            .mock("POST", "/drive/v3/changes/watch?pageToken=7")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"ch","resourceId":"res"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook")
            .await
            .unwrap();

        assert!(result.is_some());
        let (resp, _) = result.unwrap();
        assert_eq!(resp.expiration, None);
    }

    // Requirement: REQ-OI-023 (Must)
    // Acceptance: UUID v4 used for channel ID in drive watch
    #[tokio::test]
    async fn req_oi_023_drive_watch_start_generates_uuid_v4() {
        let mut server = mockito::Server::new_async().await;

        let _mock_token = server
            .mock("GET", "/drive/v3/changes/startPageToken")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"startPageToken":"1"}"#)
            .create_async()
            .await;

        let mock_watch = server
            .mock("POST", mockito::Matcher::Regex(r"/drive/v3/changes/watch.*".to_string()))
            .match_body(mockito::Matcher::Regex(
                r#""id"\s*:\s*"[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}""#
                    .to_string(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"uuid","resourceId":"res"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook").await;

        assert!(result.is_ok(), "Should succeed: {:?}", result);
        mock_watch.assert_async().await;
    }

    // -------------------------------------------------------------------
    // REQ-OI-018 (Must): drive watch stop
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-018 (Must)
    // Acceptance: build_drive_stop_url returns correct URL
    #[test]
    fn req_oi_018_stop_url() {
        let url = build_drive_stop_url("https://www.googleapis.com");
        assert_eq!(
            url,
            "https://www.googleapis.com/drive/v3/channels/stop"
        );
    }

    // Requirement: REQ-OI-018 (Must)
    // Acceptance: build_drive_stop_url with test server
    #[test]
    fn req_oi_018_stop_url_test_server() {
        let url = build_drive_stop_url("http://127.0.0.1:5678");
        assert_eq!(
            url,
            "http://127.0.0.1:5678/drive/v3/channels/stop"
        );
    }

    // Requirement: REQ-OI-018 (Must)
    // Acceptance: watch_stop calls POST to channels/stop with correct body
    #[tokio::test]
    async fn req_oi_018_watch_stop_calls_api_post() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/drive/v3/channels/stop")
            .match_header("content-type", "application/json")
            .match_body(mockito::Matcher::JsonString(
                r#"{"id":"channel-uuid-123","resourceId":"resource-abc-456"}"#.to_string(),
            ))
            .with_status(204)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_stop(
            &ctx,
            &server.url(),
            "channel-uuid-123",
            "resource-abc-456",
        )
        .await;

        assert!(result.is_ok(), "watch_stop should succeed: {:?}", result);
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-018 (Must)
    // Acceptance: Both channel_id and resource_id are sent in the body
    #[tokio::test]
    async fn req_oi_018_watch_stop_body_contains_both_ids() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/drive/v3/channels/stop")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex(r#""id"\s*:\s*"my-channel""#.to_string()),
                mockito::Matcher::Regex(r#""resourceId"\s*:\s*"my-resource""#.to_string()),
            ]))
            .with_status(204)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_stop(&ctx, &server.url(), "my-channel", "my-resource").await;

        assert!(result.is_ok());
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-018 (Must)
    // Acceptance: watch_stop with --dry-run does not call API
    #[tokio::test]
    async fn req_oi_018_watch_stop_dry_run_no_api_call() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/drive/v3/channels/stop")
            .expect(0) // Must NOT be called
            .create_async()
            .await;

        let ctx = test_ctx_dry_run();
        let result = watch_stop(&ctx, &server.url(), "ch-1", "res-1").await;

        assert!(result.is_ok(), "dry-run watch_stop should succeed");
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-018 (Must)
    // Failure mode: Channel not found (404)
    #[tokio::test]
    async fn req_oi_018_watch_stop_404_channel_not_found() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/drive/v3/channels/stop")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":{"code":404,"message":"Channel not found"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_stop(&ctx, &server.url(), "expired-channel", "old-resource").await;

        assert!(result.is_err(), "404 should return error");
    }

    // Requirement: REQ-OI-018 (Must)
    // Edge case: Network error during watch_stop
    #[tokio::test]
    async fn req_oi_018_watch_stop_network_error() {
        let ctx = test_ctx("http://127.0.0.1:1");
        let result = watch_stop(&ctx, "http://127.0.0.1:1", "ch", "res").await;

        assert!(result.is_err(), "Network error should propagate");
    }

    // Requirement: REQ-OI-018 (Must)
    // Edge case: Server returns 500 on stop
    #[tokio::test]
    async fn req_oi_018_watch_stop_server_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/drive/v3/channels/stop")
            .with_status(500)
            .with_body(r#"{"error":{"code":500,"message":"Backend Error"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_stop(&ctx, &server.url(), "ch", "res").await;

        assert!(result.is_err(), "500 should return error");
    }

    // Requirement: REQ-OI-018 (Must)
    // Edge case: Empty channel_id
    #[tokio::test]
    async fn req_oi_018_watch_stop_empty_channel_id() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/drive/v3/channels/stop")
            .with_status(400)
            .with_body(r#"{"error":{"code":400,"message":"Invalid channel id"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_stop(&ctx, &server.url(), "", "res-1").await;

        assert!(result.is_err(), "Empty channel_id should fail");
    }

    // Requirement: REQ-OI-018 (Must)
    // Edge case: Empty resource_id
    #[tokio::test]
    async fn req_oi_018_watch_stop_empty_resource_id() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/drive/v3/channels/stop")
            .with_status(400)
            .with_body(r#"{"error":{"code":400,"message":"Invalid resource id"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_stop(&ctx, &server.url(), "ch-1", "").await;

        assert!(result.is_err(), "Empty resource_id should fail");
    }

    // -------------------------------------------------------------------
    // REQ-OI-019 (Must): drive watch status
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-019 (Must)
    // Acceptance: watch_status calls GET startPageToken
    #[tokio::test]
    async fn req_oi_019_watch_status_calls_start_page_token() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/drive/v3/changes/startPageToken")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"startPageToken":"54321"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_status(&ctx, &server.url()).await;

        assert!(result.is_ok(), "watch_status should succeed: {:?}", result);
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-019 (Must)
    // Acceptance: get_start_page_token returns the token
    #[tokio::test]
    async fn req_oi_019_get_start_page_token_returns_token() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/drive/v3/changes/startPageToken")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"startPageToken":"99999"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let resp = get_start_page_token(&ctx, &server.url()).await.unwrap();

        assert_eq!(resp.start_page_token, "99999");
    }

    // Requirement: REQ-OI-019 (Must)
    // Failure mode: 401 Unauthorized on status
    #[tokio::test]
    async fn req_oi_019_watch_status_401_unauthorized() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/drive/v3/changes/startPageToken")
            .with_status(401)
            .with_body(r#"{"error":{"code":401,"message":"Invalid Credentials"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_status(&ctx, &server.url()).await;

        assert!(result.is_err(), "401 should return error");
    }

    // Requirement: REQ-OI-019 (Must)
    // Edge case: Network error during watch_status
    #[tokio::test]
    async fn req_oi_019_watch_status_network_error() {
        let ctx = test_ctx("http://127.0.0.1:1");
        let result = watch_status(&ctx, "http://127.0.0.1:1").await;

        assert!(result.is_err(), "Network error should propagate");
    }

    // Requirement: REQ-OI-019 (Must)
    // Edge case: Server returns 500 on status
    #[tokio::test]
    async fn req_oi_019_watch_status_server_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/drive/v3/changes/startPageToken")
            .with_status(500)
            .with_body(r#"{"error":{"code":500,"message":"Backend Error"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_status(&ctx, &server.url()).await;

        assert!(result.is_err(), "500 should return error");
    }

    // -------------------------------------------------------------------
    // REQ-OI-025 (Must): CLI structure -- URL builders use DRIVE_BASE_URL
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-025 (Must)
    // Acceptance: URL builders produce URLs consistent with DRIVE_BASE_URL
    #[test]
    fn req_oi_025_watch_url_consistent_with_base() {
        let watch_url = build_drive_watch_url("https://www.googleapis.com", "123");
        assert!(
            watch_url.starts_with(DRIVE_BASE_URL),
            "Watch URL should start with DRIVE_BASE_URL"
        );
    }

    // Requirement: REQ-OI-025 (Must)
    // Acceptance: Stop URL consistent with DRIVE_BASE_URL
    #[test]
    fn req_oi_025_stop_url_consistent_with_base() {
        let stop_url = build_drive_stop_url("https://www.googleapis.com");
        assert!(
            stop_url.starts_with(DRIVE_BASE_URL),
            "Stop URL should start with DRIVE_BASE_URL"
        );
    }

    // Requirement: REQ-OI-025 (Must)
    // Acceptance: Start page token URL consistent with DRIVE_BASE_URL
    #[test]
    fn req_oi_025_start_page_token_url_consistent_with_base() {
        let url = build_start_page_token_url("https://www.googleapis.com");
        assert!(
            url.starts_with(DRIVE_BASE_URL),
            "StartPageToken URL should start with DRIVE_BASE_URL"
        );
    }
}
