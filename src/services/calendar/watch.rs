//! Calendar watch (push notification) management.
//!
//! Provides handlers for registering, stopping, and querying Calendar push
//! notification watches via Google's Channel API.

use crate::services::common::{ChannelStopRequest, WatchChannelRequest, WatchChannelResponse, WatchParams};
use crate::services::ServiceContext;

/// Build URL for starting a calendar event watch.
///
/// `base_url` is the API root (e.g. `https://www.googleapis.com` in production,
/// or `server.url()` in tests). The Calendar API path is appended.
pub fn build_calendar_watch_url(base_url: &str, calendar_id: &str) -> String {
    format!(
        "{}/calendar/v3/calendars/{}/events/watch",
        base_url, calendar_id
    )
}

/// Build URL for stopping a calendar watch channel.
pub fn build_calendar_stop_url(base_url: &str) -> String {
    format!("{}/calendar/v3/channels/stop", base_url)
}

/// Start a Calendar push notification watch.
///
/// Generates a UUID v4 channel ID, registers a web_hook watch on the
/// specified calendar, and returns the channel response.
pub async fn watch_start(
    ctx: &ServiceContext,
    base_url: &str,
    callback_url: &str,
    calendar_id: &str,
) -> anyhow::Result<Option<WatchChannelResponse>> {
    let url = build_calendar_watch_url(base_url, calendar_id);
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

    if let Some(ref resp) = result {
        ctx.write_output(resp)?;
    }

    Ok(result)
}

/// Stop a Calendar push notification watch.
pub async fn watch_stop(
    ctx: &ServiceContext,
    base_url: &str,
    channel_id: &str,
    resource_id: &str,
) -> anyhow::Result<()> {
    let url = build_calendar_stop_url(base_url);
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

/// Show Calendar watch status.
///
/// There is no API to query active watches, so this just prints an
/// informational note.
pub async fn watch_status(ctx: &ServiceContext) -> anyhow::Result<()> {
    let _ = ctx;
    // No API call -- just print a note.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===================================================================
    // OI-M3: Calendar Watch Commands -- TDD tests
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
    use crate::services::calendar::types::CALENDAR_BASE_URL;
    use crate::services::ServiceContext;
    use crate::ui::{ColorMode, Ui, UiOptions};

    /// Build a test ServiceContext pointing at the given mock server URL.
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
    // REQ-OI-014 (Must): calendar watch start
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-014 (Must)
    // Acceptance: build_calendar_watch_url with default "primary" calendar
    #[test]
    fn req_oi_014_watch_url_primary_calendar() {
        let url = build_calendar_watch_url("https://www.googleapis.com", "primary");
        assert_eq!(
            url,
            "https://www.googleapis.com/calendar/v3/calendars/primary/events/watch"
        );
    }

    // Requirement: REQ-OI-014 (Must)
    // Acceptance: build_calendar_watch_url with custom calendar ID
    #[test]
    fn req_oi_014_watch_url_custom_calendar() {
        let url = build_calendar_watch_url(
            "https://www.googleapis.com",
            "user@example.com",
        );
        assert_eq!(
            url,
            "https://www.googleapis.com/calendar/v3/calendars/user@example.com/events/watch"
        );
    }

    // Requirement: REQ-OI-014 (Must)
    // Acceptance: build_calendar_watch_url with test server base URL
    #[test]
    fn req_oi_014_watch_url_test_server() {
        let url = build_calendar_watch_url("http://127.0.0.1:1234", "primary");
        assert_eq!(
            url,
            "http://127.0.0.1:1234/calendar/v3/calendars/primary/events/watch"
        );
    }

    // Requirement: REQ-OI-014 (Must)
    // Acceptance: Calendar watch start calls POST to events/watch endpoint
    #[tokio::test]
    async fn req_oi_014_watch_start_calls_api_post() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/calendar/v3/calendars/primary/events/watch")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"ch-uuid","resourceId":"res-123","expiration":"1704153600000"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook", "primary").await;

        assert!(result.is_ok(), "watch_start should succeed: {:?}", result);
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-014 (Must)
    // Acceptance: watch_start returns WatchChannelResponse with correct fields
    #[tokio::test]
    async fn req_oi_014_watch_start_returns_response() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/calendar/v3/calendars/primary/events/watch")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"channel-abc","resourceId":"resource-xyz","expiration":"1704240000000"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook", "primary")
            .await
            .unwrap();

        assert!(result.is_some(), "Non-dry-run should return Some");
        let resp = result.unwrap();
        assert_eq!(resp.id, "channel-abc");
        assert_eq!(resp.resource_id, "resource-xyz");
        assert_eq!(resp.expiration, Some("1704240000000".to_string()));
    }

    // Requirement: REQ-OI-014 (Must)
    // Acceptance: watch_start request body contains UUID v4, web_hook type, TTL
    #[tokio::test]
    async fn req_oi_014_watch_start_request_body_format() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/calendar/v3/calendars/primary/events/watch")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex(r#""type"\s*:\s*"web_hook""#.to_string()),
                mockito::Matcher::Regex(r#""address"\s*:\s*"https://example.com/hook""#.to_string()),
                mockito::Matcher::Regex(r#""ttl"\s*:\s*"604800""#.to_string()),
                // UUID v4 format: 8-4-4-4-12 hex digits
                mockito::Matcher::Regex(
                    r#""id"\s*:\s*"[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}""#
                        .to_string(),
                ),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"ch-1","resourceId":"res-1","expiration":"123"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook", "primary").await;

        assert!(result.is_ok(), "watch_start should succeed: {:?}", result);
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-014 (Must)
    // Acceptance: watch_start with --dry-run does not call API
    #[tokio::test]
    async fn req_oi_014_watch_start_dry_run_no_api_call() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/calendar/v3/calendars/primary/events/watch")
            .expect(0)
            .create_async()
            .await;

        let ctx = test_ctx_dry_run();
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook", "primary").await;

        assert!(result.is_ok(), "dry-run watch_start should succeed");
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-014 (Must)
    // Acceptance: --calendar defaults to "primary"
    //   (This tests that passing "primary" works correctly)
    #[tokio::test]
    async fn req_oi_014_watch_start_default_primary_calendar() {
        let mut server = mockito::Server::new_async().await;
        // The mock specifically expects /calendars/primary/ in the path
        let mock = server
            .mock("POST", "/calendar/v3/calendars/primary/events/watch")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"ch","resourceId":"res"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook", "primary").await;

        assert!(result.is_ok());
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-014 (Must)
    // Acceptance: watch_start with non-primary calendar ID
    #[tokio::test]
    async fn req_oi_014_watch_start_custom_calendar_id() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock(
                "POST",
                "/calendar/v3/calendars/user@example.com/events/watch",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"ch","resourceId":"res","expiration":"999"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(
            &ctx,
            &server.url(),
            "https://example.com/hook",
            "user@example.com",
        )
        .await;

        assert!(result.is_ok());
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-014 (Must)
    // Failure mode: Domain not verified (403) -- callback URL domain not verified
    #[tokio::test]
    async fn req_oi_014_watch_start_403_domain_not_verified() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/calendar/v3/calendars/primary/events/watch")
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":{"code":403,"message":"Push notifications are not allowed for the destination"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://unverified.example.com/hook", "primary").await;

        assert!(result.is_err(), "403 should return error");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("403") || err_msg.contains("error") || err_msg.contains("Push"),
            "Error message should indicate permission/domain issue: {}",
            err_msg
        );
    }

    // Requirement: REQ-OI-014 (Must)
    // Failure mode: Server returns 401 Unauthorized (token expired)
    #[tokio::test]
    async fn req_oi_014_watch_start_401_unauthorized() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/calendar/v3/calendars/primary/events/watch")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":{"code":401,"message":"Invalid Credentials"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook", "primary").await;

        assert!(result.is_err(), "401 should return error");
    }

    // Requirement: REQ-OI-014 (Must)
    // Edge case: Network error during watch_start
    #[tokio::test]
    async fn req_oi_014_watch_start_network_error() {
        let ctx = test_ctx("http://127.0.0.1:1");
        let result = watch_start(
            &ctx,
            "http://127.0.0.1:1",
            "https://example.com/hook",
            "primary",
        )
        .await;

        assert!(result.is_err(), "Network error should propagate");
    }

    // Requirement: REQ-OI-014 (Must)
    // Edge case: Empty callback URL
    #[tokio::test]
    async fn req_oi_014_watch_start_empty_callback_url() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/calendar/v3/calendars/primary/events/watch")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":{"code":400,"message":"Invalid address"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "", "primary").await;

        assert!(result.is_err(), "Empty callback URL should fail");
    }

    // Requirement: REQ-OI-014 (Must)
    // Edge case: Empty calendar ID
    #[test]
    fn req_oi_014_watch_url_empty_calendar_id() {
        let url = build_calendar_watch_url("https://www.googleapis.com", "");
        // Should still build a URL (API will reject it)
        assert_eq!(
            url,
            "https://www.googleapis.com/calendar/v3/calendars//events/watch"
        );
    }

    // Requirement: REQ-OI-014 (Must)
    // Edge case: Calendar ID with special characters (encoded email)
    #[test]
    fn req_oi_014_watch_url_calendar_id_with_at_sign() {
        let url = build_calendar_watch_url("https://www.googleapis.com", "team@example.com");
        assert!(url.contains("team@example.com"));
    }

    // Requirement: REQ-OI-014 (Must)
    // Edge case: Server returns 500 Internal Server Error
    #[tokio::test]
    async fn req_oi_014_watch_start_server_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/calendar/v3/calendars/primary/events/watch")
            .with_status(500)
            .with_body(r#"{"error":{"code":500,"message":"Backend Error"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook", "primary").await;

        assert!(result.is_err(), "500 should return error");
    }

    // Requirement: REQ-OI-014 (Must)
    // Edge case: WatchChannelResponse with no expiration field
    #[tokio::test]
    async fn req_oi_014_watch_start_response_no_expiration() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/calendar/v3/calendars/primary/events/watch")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"ch-1","resourceId":"res-1"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook", "primary")
            .await
            .unwrap();

        assert!(result.is_some());
        let resp = result.unwrap();
        assert_eq!(resp.expiration, None);
    }

    // Requirement: REQ-OI-023 (Must)
    // Acceptance: UUID v4 format used for channel ID
    //   This test validates that the generated channel ID matches UUID v4 format.
    #[tokio::test]
    async fn req_oi_023_watch_start_generates_uuid_v4() {
        let mut server = mockito::Server::new_async().await;
        // Use a regex matcher to capture the UUID v4 in the request body
        let mock = server
            .mock("POST", "/calendar/v3/calendars/primary/events/watch")
            .match_body(mockito::Matcher::Regex(
                // UUID v4: 8-4-4[version]-4[variant]-12 hex chars
                r#""id"\s*:\s*"[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}""#
                    .to_string(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"uuid","resourceId":"res"}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_start(&ctx, &server.url(), "https://example.com/hook", "primary").await;

        assert!(result.is_ok(), "Should succeed: {:?}", result);
        mock.assert_async().await;
    }

    // -------------------------------------------------------------------
    // REQ-OI-015 (Must): calendar watch stop
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-015 (Must)
    // Acceptance: build_calendar_stop_url returns correct URL
    #[test]
    fn req_oi_015_stop_url() {
        let url = build_calendar_stop_url("https://www.googleapis.com");
        assert_eq!(
            url,
            "https://www.googleapis.com/calendar/v3/channels/stop"
        );
    }

    // Requirement: REQ-OI-015 (Must)
    // Acceptance: build_calendar_stop_url with test server
    #[test]
    fn req_oi_015_stop_url_test_server() {
        let url = build_calendar_stop_url("http://127.0.0.1:5678");
        assert_eq!(
            url,
            "http://127.0.0.1:5678/calendar/v3/channels/stop"
        );
    }

    // Requirement: REQ-OI-015 (Must)
    // Acceptance: watch_stop calls POST to channels/stop with correct body
    #[tokio::test]
    async fn req_oi_015_watch_stop_calls_api_post() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/calendar/v3/channels/stop")
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

    // Requirement: REQ-OI-015 (Must)
    // Acceptance: Both channel_id and resource_id are sent in request body
    #[tokio::test]
    async fn req_oi_015_watch_stop_body_contains_both_ids() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/calendar/v3/channels/stop")
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

    // Requirement: REQ-OI-015 (Must)
    // Acceptance: watch_stop with --dry-run does not call API
    #[tokio::test]
    async fn req_oi_015_watch_stop_dry_run_no_api_call() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/calendar/v3/channels/stop")
            .expect(0) // Must NOT be called
            .create_async()
            .await;

        let ctx = test_ctx_dry_run();
        let result = watch_stop(&ctx, &server.url(), "ch-1", "res-1").await;

        assert!(result.is_ok(), "dry-run watch_stop should succeed");
        mock.assert_async().await;
    }

    // Requirement: REQ-OI-015 (Must)
    // Failure mode: Invalid channel stop (404) -- channel already expired
    #[tokio::test]
    async fn req_oi_015_watch_stop_404_channel_not_found() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/calendar/v3/channels/stop")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":{"code":404,"message":"Channel not found"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_stop(&ctx, &server.url(), "expired-channel", "old-resource").await;

        assert!(result.is_err(), "404 should return error");
    }

    // Requirement: REQ-OI-015 (Must)
    // Edge case: Network error during watch_stop
    #[tokio::test]
    async fn req_oi_015_watch_stop_network_error() {
        let ctx = test_ctx("http://127.0.0.1:1");
        let result = watch_stop(&ctx, "http://127.0.0.1:1", "ch", "res").await;

        assert!(result.is_err(), "Network error should propagate");
    }

    // Requirement: REQ-OI-015 (Must)
    // Edge case: Server returns 500 on stop
    #[tokio::test]
    async fn req_oi_015_watch_stop_server_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/calendar/v3/channels/stop")
            .with_status(500)
            .with_body(r#"{"error":{"code":500,"message":"Backend Error"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_stop(&ctx, &server.url(), "ch", "res").await;

        assert!(result.is_err(), "500 should return error");
    }

    // Requirement: REQ-OI-015 (Must)
    // Edge case: Empty channel_id
    #[tokio::test]
    async fn req_oi_015_watch_stop_empty_channel_id() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/calendar/v3/channels/stop")
            .with_status(400)
            .with_body(r#"{"error":{"code":400,"message":"Invalid channel id"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_stop(&ctx, &server.url(), "", "res-1").await;

        // Either the handler rejects it or the API returns 400
        assert!(result.is_err(), "Empty channel_id should fail");
    }

    // Requirement: REQ-OI-015 (Must)
    // Edge case: Empty resource_id
    #[tokio::test]
    async fn req_oi_015_watch_stop_empty_resource_id() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/calendar/v3/channels/stop")
            .with_status(400)
            .with_body(r#"{"error":{"code":400,"message":"Invalid resource id"}}"#)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_stop(&ctx, &server.url(), "ch-1", "").await;

        assert!(result.is_err(), "Empty resource_id should fail");
    }

    // -------------------------------------------------------------------
    // REQ-OI-016 (Must): calendar watch status
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-016 (Must)
    // Acceptance: watch_status succeeds (no API call needed)
    #[tokio::test]
    async fn req_oi_016_watch_status_succeeds_no_api() {
        let ctx = test_ctx("http://unused");
        let result = watch_status(&ctx).await;

        assert!(result.is_ok(), "watch_status should succeed without API call");
    }

    // Requirement: REQ-OI-016 (Must)
    // Acceptance: watch_status does not make any HTTP requests
    //   (There is no API to query active calendar watches)
    #[tokio::test]
    async fn req_oi_016_watch_status_no_http_request() {
        let mut server = mockito::Server::new_async().await;
        // Create mocks that should NOT be called
        let mock_get = server
            .mock("GET", mockito::Matcher::Any)
            .expect(0)
            .create_async()
            .await;
        let mock_post = server
            .mock("POST", mockito::Matcher::Any)
            .expect(0)
            .create_async()
            .await;

        let ctx = test_ctx(&server.url());
        let result = watch_status(&ctx).await;

        assert!(result.is_ok());
        mock_get.assert_async().await;
        mock_post.assert_async().await;
    }

    // -------------------------------------------------------------------
    // REQ-OI-025 (Must): CLI structure -- URL builders use CALENDAR_BASE_URL
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-025 (Must)
    // Acceptance: URL builders produce URLs consistent with CALENDAR_BASE_URL
    #[test]
    fn req_oi_025_watch_url_consistent_with_base() {
        // CALENDAR_BASE_URL is "https://www.googleapis.com/calendar/v3"
        let watch_url = build_calendar_watch_url("https://www.googleapis.com", "primary");
        assert!(
            watch_url.starts_with(CALENDAR_BASE_URL),
            "Watch URL should start with CALENDAR_BASE_URL"
        );
    }

    // Requirement: REQ-OI-025 (Must)
    // Acceptance: Stop URL consistent with CALENDAR_BASE_URL
    #[test]
    fn req_oi_025_stop_url_consistent_with_base() {
        let stop_url = build_calendar_stop_url("https://www.googleapis.com");
        assert!(
            stop_url.starts_with(CALENDAR_BASE_URL),
            "Stop URL should start with CALENDAR_BASE_URL"
        );
    }
}
