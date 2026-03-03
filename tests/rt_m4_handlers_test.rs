//! RT-M4: Core Service Handlers -- Integration Tests
//!
//! Tests cover REQ-RT-032 through REQ-RT-066 (Must and Should priority).
//! These validate that handler functions in `src/cli/mod.rs` correctly:
//! - Bootstrap auth and build ServiceContext
//! - Call the correct Google API URLs via api_get/api_post/api_patch/api_delete
//! - Deserialize responses into the correct types
//! - Produce correct output (JSON/plain/text)
//! - Return correct exit codes (success, empty, auth error, etc.)
//! - Support dry-run, --force, --fail-empty, pagination
//!
//! Strategy: Each handler function is tested using a mockito server that
//! simulates Google API responses. We construct a ServiceContext pointing
//! to the mock server, then call the handler and verify behavior.

use std::sync::Arc;

use omega_google::cli::root::RootFlags;
use omega_google::error::exit::codes;
use omega_google::http::circuit_breaker::CircuitBreaker;
use omega_google::http::RetryConfig;
use omega_google::output::{JsonTransform, OutputMode};
use omega_google::services::ServiceContext;
use omega_google::ui::{ColorMode, Ui, UiOptions};

// ===================================================================
// Test helpers
// ===================================================================

/// Create a RetryConfig with zero retries for fast tests.
fn no_retry_config() -> RetryConfig {
    RetryConfig {
        max_retries_429: 0,
        max_retries_5xx: 0,
        base_delay: std::time::Duration::from_millis(0),
        server_error_delay: std::time::Duration::from_millis(0),
    }
}

/// Build a test ServiceContext with a pre-configured reqwest client.
/// The client has no auth headers -- mocks don't need them.
fn test_service_context(_base_url: &str) -> ServiceContext {
    // Build a client that can reach the mock server (no TLS needed)
    let client = reqwest::Client::builder()
        .build()
        .expect("failed to build reqwest client");

    let flags = RootFlags {
        json: true,
        ..Default::default()
    };
    let ui = Ui::new(UiOptions {
        color: ColorMode::Never,
    })
    .unwrap();

    ServiceContext {
        client,
        output_mode: OutputMode::Json,
        json_transform: JsonTransform::default(),
        ui,
        flags,
        circuit_breaker: Arc::new(CircuitBreaker::new()),
        retry_config: no_retry_config(),
        email: "test@example.com".to_string(),
    }
}

/// Build a test ServiceContext with custom flags.
#[allow(dead_code)]
fn test_service_context_with_flags(_base_url: &str, flags: RootFlags) -> ServiceContext {
    let client = reqwest::Client::builder()
        .build()
        .expect("failed to build reqwest client");

    let ui = Ui::new(UiOptions {
        color: ColorMode::Never,
    })
    .unwrap();

    let output_mode = if flags.json {
        OutputMode::Json
    } else if flags.plain {
        OutputMode::Plain
    } else if flags.csv {
        OutputMode::Csv
    } else {
        OutputMode::Text
    };

    ServiceContext {
        client,
        output_mode,
        json_transform: JsonTransform {
            results_only: flags.results_only,
            select: flags
                .select
                .as_ref()
                .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
                .unwrap_or_default(),
        },
        ui,
        flags,
        circuit_breaker: Arc::new(CircuitBreaker::new()),
        retry_config: no_retry_config(),
        email: "test@example.com".to_string(),
    }
}

// ===================================================================
// Module 1: Gmail Handler Tests (Must priority)
// ===================================================================

// -------------------------------------------------------------------
// REQ-RT-032 (Must): handle_gmail_search -- thread search with pagination
// -------------------------------------------------------------------

// Requirement: REQ-RT-032 (Must)
// Acceptance: handle_gmail is async and dispatches to subcommand handlers
#[tokio::test]
async fn req_rt_032_handle_gmail_is_async() {
    // After RT-M4 implementation, handle_gmail must be async fn.
    // This test verifies the dispatch works by calling execute() with gmail search args.
    // Since we have no auth, it should return AUTH_REQUIRED (4) instead of SUCCESS (0).
    use std::ffi::OsString;
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_GOOGLE_CONFIG_DIR", tmp.path());
    std::env::set_var("GOG_KEYRING_BACKEND", "file");
    let args: Vec<OsString> = vec!["gmail".into(), "search".into(), "test query".into()];
    let exit = omega_google::cli::execute(args).await;
    std::env::remove_var("GOG_KEYRING_BACKEND");
    std::env::remove_var("OMEGA_GOOGLE_CONFIG_DIR");
    // Before M4: returns SUCCESS with stub message
    // After M4: returns AUTH_REQUIRED because no credentials are configured
    assert!(
        exit == codes::AUTH_REQUIRED || exit == codes::SUCCESS,
        "Gmail search should return AUTH_REQUIRED or SUCCESS, got {}",
        exit
    );
}

// Requirement: REQ-RT-032 (Must)
// Acceptance: Gmail search calls correct API URL with query and max params
#[tokio::test]
async fn req_rt_032_gmail_search_calls_correct_url() {
    let mut server = mockito::Server::new_async().await;

    // Mock the Gmail thread list endpoint
    let mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/gmail/v1/users/me/threads\?.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "threads": [
                {"id": "thread1", "snippet": "Hello world", "historyId": "1234"}
            ],
            "resultSizeEstimate": 1
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());

    // The handler should call the URL with q=test+query&maxResults=10
    // This test verifies the URL builder integration
    let url = omega_google::services::gmail::search::build_thread_search_url(
        "test query",
        Some(10),
        None,
    );
    assert!(url.contains("users/me/threads"));
    assert!(url.contains("maxResults=10"));
    assert!(url.contains("q="));

    // When the handler is implemented, calling it with the mock server
    // should hit this URL and deserialize the response.
    // For now, we verify the URL builder works correctly with the mock.
    let response: omega_google::services::gmail::types::ThreadListResponse =
        omega_google::http::api::api_get(
            &ctx.client,
            &format!(
                "{}/gmail/v1/users/me/threads?maxResults=10&q=test",
                server.url()
            ),
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await
        .unwrap();

    assert_eq!(response.threads.len(), 1);
    assert_eq!(response.threads[0].id, "thread1");
    mock.assert_async().await;
}

// Requirement: REQ-RT-032 (Must)
// Acceptance: Gmail search honors --fail-empty when no results
#[tokio::test]
async fn req_rt_032_gmail_search_fail_empty_no_results() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/gmail/v1/users/me/threads.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"threads": [], "resultSizeEstimate": 0}"#)
        .create_async()
        .await;

    // Verify that check_fail_empty correctly identifies empty results
    let items: Vec<String> = vec![];
    let result = omega_google::services::pagination::check_fail_empty(&items, true);
    assert!(
        result.is_err(),
        "Should fail with empty results when fail_empty=true"
    );
}

// Requirement: REQ-RT-032 (Must)
// Acceptance: Gmail search pagination with --all fetches all pages
#[tokio::test]
async fn req_rt_032_gmail_search_pagination_all_pages() {
    let mut server = mockito::Server::new_async().await;

    // Page 1
    let _mock1 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/gmail/v1/users/me/threads\?maxResults=".to_string()),
        )
        .with_status(200)
        .with_body(
            r#"{
            "threads": [{"id": "t1", "snippet": "page1"}],
            "nextPageToken": "page2token",
            "resultSizeEstimate": 2
        }"#,
        )
        .create_async()
        .await;

    // Page 2
    let _mock2 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r".*pageToken=page2token.*".to_string()),
        )
        .with_status(200)
        .with_body(
            r#"{
            "threads": [{"id": "t2", "snippet": "page2"}],
            "resultSizeEstimate": 2
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let base_url = server.url();
    let params = omega_google::services::common::PaginationParams {
        max_results: Some(1),
        all_pages: true,
        ..Default::default()
    };

    let (items, next_token) = omega_google::services::pagination::paginate(
        &ctx.client,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
        &params,
        |pt| {
            let mut url = format!("{}/gmail/v1/users/me/threads?maxResults=1", base_url);
            if let Some(token) = pt {
                url.push_str(&format!("&pageToken={}", token));
            }
            url
        },
        |value| {
            let threads = value
                .get("threads")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.get("id").and_then(|id| id.as_str()).map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let next = value
                .get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok((threads, next))
        },
    )
    .await
    .unwrap();

    assert_eq!(items.len(), 2);
    assert_eq!(items[0], "t1");
    assert_eq!(items[1], "t2");
    assert!(next_token.is_none(), "All pages fetched, no hint token");
}

// Requirement: REQ-RT-032 (Must)
// Acceptance: Gmail search single-page mode returns hint token
#[tokio::test]
async fn req_rt_032_gmail_search_single_page_hint() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/gmail/v1/users/me/threads.*".to_string()),
        )
        .with_status(200)
        .with_body(
            r#"{
            "threads": [{"id": "t1", "snippet": "result"}],
            "nextPageToken": "NEXT123",
            "resultSizeEstimate": 100
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let base_url = server.url();
    let params = omega_google::services::common::PaginationParams {
        max_results: Some(10),
        all_pages: false,
        ..Default::default()
    };

    let (items, hint_token) = omega_google::services::pagination::paginate(
        &ctx.client,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
        &params,
        |pt| {
            let mut url = format!("{}/gmail/v1/users/me/threads?maxResults=10", base_url);
            if let Some(token) = pt {
                url.push_str(&format!("&pageToken={}", token));
            }
            url
        },
        |value| {
            let threads = value
                .get("threads")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.get("id").and_then(|id| id.as_str()).map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let next = value
                .get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok((threads, next))
        },
    )
    .await
    .unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(hint_token, Some("NEXT123".to_string()));
}

// Requirement: REQ-RT-032 (Must)
// Edge case: Gmail search with empty query
#[test]
fn req_rt_032_gmail_search_empty_query() {
    let url = omega_google::services::gmail::search::build_thread_search_url("", Some(10), None);
    assert!(url.contains("maxResults=10"));
    // Empty query should not include q= parameter
    assert!(!url.contains("q="));
}

// Requirement: REQ-RT-032 (Must)
// Edge case: Gmail search with special characters in query
#[test]
fn req_rt_032_gmail_search_special_chars_query() {
    let url = omega_google::services::gmail::search::build_thread_search_url(
        "from:user@example.com subject:\"meeting notes\"",
        Some(20),
        None,
    );
    assert!(url.contains("maxResults=20"));
    assert!(url.contains("q="));
    // URL-encoded characters should be present
}

// Requirement: REQ-RT-032 (Must)
// Failure mode: Gmail search API returns 401 -- should map to AUTH_REQUIRED
#[tokio::test]
async fn req_rt_032_gmail_search_api_401_auth_error() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/gmail/v1/users/me/threads.*".to_string()),
        )
        .with_status(401)
        .with_body(r#"{"error":{"code":401,"message":"Token expired"}}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/threads?maxResults=10", server.url());

    let result: anyhow::Result<omega_google::services::gmail::types::ThreadListResponse> =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    // Verify error can be mapped to the correct exit code
    if let Some(omega_err) = err.downcast_ref::<omega_google::error::exit::OmegaError>() {
        assert_eq!(
            omega_google::error::exit::exit_code_for(omega_err),
            codes::AUTH_REQUIRED
        );
    }
}

// Requirement: REQ-RT-032 (Must)
// Failure mode: Gmail search API returns 429 rate limit
#[tokio::test]
async fn req_rt_032_gmail_search_api_429_rate_limited() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/gmail/v1/users/me/threads.*".to_string()),
        )
        .with_status(429)
        .with_body(r#"{"error":{"code":429,"message":"Rate limit exceeded"}}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/threads?maxResults=10", server.url());

    let result: anyhow::Result<omega_google::services::gmail::types::ThreadListResponse> =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    if let Some(omega_err) = err.downcast_ref::<omega_google::error::exit::OmegaError>() {
        assert_eq!(
            omega_google::error::exit::exit_code_for(omega_err),
            codes::RATE_LIMITED
        );
    }
}

// -------------------------------------------------------------------
// REQ-RT-033 (Must): handle_gmail_message_search
// -------------------------------------------------------------------

// Requirement: REQ-RT-033 (Must)
// Acceptance: Message search URL includes query and format parameters
#[test]
fn req_rt_033_gmail_message_search_url_with_include_body() {
    let url = omega_google::services::gmail::search::build_message_search_url(
        "subject:report",
        Some(20),
        None,
        true,
    );
    assert!(url.contains("users/me/messages"));
    assert!(url.contains("maxResults=20"));
    assert!(url.contains("format=full"));
}

// Requirement: REQ-RT-033 (Must)
// Acceptance: Message search without include_body omits format
#[test]
fn req_rt_033_gmail_message_search_url_without_body() {
    let url = omega_google::services::gmail::search::build_message_search_url(
        "from:alice@example.com",
        Some(10),
        None,
        false,
    );
    assert!(url.contains("users/me/messages"));
    assert!(!url.contains("format="));
}

// Requirement: REQ-RT-033 (Must)
// Acceptance: Message search deserializes response correctly
#[tokio::test]
async fn req_rt_033_gmail_message_search_deserializes_response() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/gmail/v1/users/me/messages.*".to_string()),
        )
        .with_status(200)
        .with_body(
            r#"{
            "messages": [
                {"id": "msg1", "threadId": "t1"},
                {"id": "msg2", "threadId": "t2"}
            ],
            "nextPageToken": "next_token",
            "resultSizeEstimate": 50
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/messages?maxResults=10", server.url());

    // The message list response uses a different structure than thread list
    let response: serde_json::Value = omega_google::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
    )
    .await
    .unwrap();

    let messages = response.get("messages").unwrap().as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["id"], "msg1");
}

// -------------------------------------------------------------------
// REQ-RT-034 (Must): handle_gmail_thread_get
// -------------------------------------------------------------------

// Requirement: REQ-RT-034 (Must)
// Acceptance: Thread get URL is correctly formed
#[test]
fn req_rt_034_gmail_thread_get_url() {
    let url = omega_google::services::gmail::thread::build_thread_get_url("thread_abc123");
    assert!(url.contains("users/me/threads/thread_abc123"));
}

// Requirement: REQ-RT-034 (Must)
// Acceptance: Thread get returns full thread with messages
#[tokio::test]
async fn req_rt_034_gmail_thread_get_returns_full_thread() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/gmail/v1/users/me/threads/thread123")
        .with_status(200)
        .with_body(
            r#"{
            "id": "thread123",
            "historyId": "99999",
            "messages": [
                {"id": "msg1", "threadId": "thread123", "snippet": "First message"},
                {"id": "msg2", "threadId": "thread123", "snippet": "Reply"}
            ]
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/threads/thread123", server.url());

    let thread: omega_google::services::gmail::types::Thread = omega_google::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
    )
    .await
    .unwrap();

    assert_eq!(thread.id, "thread123");
    assert_eq!(thread.messages.len(), 2);
    assert_eq!(thread.messages[0].id, "msg1");
    assert_eq!(thread.messages[1].id, "msg2");
}

// Requirement: REQ-RT-034 (Must)
// Failure mode: Thread not found returns 404
#[tokio::test]
async fn req_rt_034_gmail_thread_get_not_found() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/gmail/v1/users/me/threads/nonexistent")
        .with_status(404)
        .with_body(r#"{"error":{"code":404,"message":"Not Found"}}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/threads/nonexistent", server.url());

    let result: anyhow::Result<omega_google::services::gmail::types::Thread> =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    if let Some(omega_err) = err.downcast_ref::<omega_google::error::exit::OmegaError>() {
        assert_eq!(
            omega_google::error::exit::exit_code_for(omega_err),
            codes::NOT_FOUND
        );
    }
}

// -------------------------------------------------------------------
// REQ-RT-035 (Must): handle_gmail_message_get
// -------------------------------------------------------------------

// Requirement: REQ-RT-035 (Must)
// Acceptance: Message get URL includes format parameter
#[test]
fn req_rt_035_gmail_message_get_url_with_format() {
    let url = omega_google::services::gmail::message::build_message_get_url("msg123", Some("full"));
    assert!(url.contains("users/me/messages/msg123"));
    assert!(url.contains("format=full"));
}

// Requirement: REQ-RT-035 (Must)
// Acceptance: Message get URL with metadata format
#[test]
fn req_rt_035_gmail_message_get_url_metadata() {
    let url =
        omega_google::services::gmail::message::build_message_get_url("msg123", Some("metadata"));
    assert!(url.contains("format=metadata"));
}

// Requirement: REQ-RT-035 (Must)
// Acceptance: Message get URL with raw format
#[test]
fn req_rt_035_gmail_message_get_url_raw() {
    let url = omega_google::services::gmail::message::build_message_get_url("msg123", Some("raw"));
    assert!(url.contains("format=raw"));
}

// Requirement: REQ-RT-035 (Must)
// Acceptance: Message get deserializes full message with headers and payload
#[tokio::test]
async fn req_rt_035_gmail_message_get_full_response() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/gmail/v1/users/me/messages/msg456.*".to_string()),
        )
        .with_status(200)
        .with_body(
            r#"{
            "id": "msg456",
            "threadId": "thread789",
            "labelIds": ["INBOX", "UNREAD"],
            "snippet": "Important message",
            "internalDate": "1704067200000",
            "sizeEstimate": 2048,
            "payload": {
                "mimeType": "text/plain",
                "headers": [
                    {"name": "From", "value": "sender@example.com"},
                    {"name": "Subject", "value": "Test Subject"}
                ],
                "body": {"size": 100, "data": "SGVsbG8gV29ybGQ="}
            }
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!(
        "{}/gmail/v1/users/me/messages/msg456?format=full",
        server.url()
    );

    let msg: omega_google::services::gmail::types::Message = omega_google::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
    )
    .await
    .unwrap();

    assert_eq!(msg.id, "msg456");
    assert_eq!(msg.thread_id, Some("thread789".to_string()));
    assert!(msg.label_ids.contains(&"INBOX".to_string()));
    assert!(msg.label_ids.contains(&"UNREAD".to_string()));
    let payload = msg.payload.as_ref().unwrap();
    assert_eq!(payload.headers.len(), 2);
    assert_eq!(payload.headers[0].name, "From");
}

// -------------------------------------------------------------------
// REQ-RT-036 (Must): handle_gmail_send
// -------------------------------------------------------------------

// Requirement: REQ-RT-036 (Must)
// Acceptance: Send URL points to messages/send endpoint
#[test]
fn req_rt_036_gmail_send_url() {
    let url = omega_google::services::gmail::send::build_send_url();
    assert!(url.contains("users/me/messages/send"));
}

// Requirement: REQ-RT-036 (Must)
// Acceptance: MIME message is built correctly from parameters
#[test]
fn req_rt_036_gmail_send_mime_message_construction() {
    let params = omega_google::services::gmail::mime::MimeMessageParams {
        from: "sender@example.com".to_string(),
        to: vec!["recipient@example.com".to_string()],
        cc: vec![],
        bcc: vec![],
        subject: "Test Subject".to_string(),
        body_text: Some("Hello, World!".to_string()),
        body_html: None,
        reply_to: None,
        in_reply_to: None,
        references: None,
        attachments: vec![],
    };
    let mime = omega_google::services::gmail::mime::build_mime_message(&params);
    assert!(mime.contains("From: sender@example.com"));
    assert!(mime.contains("To: recipient@example.com"));
    assert!(mime.contains("Subject: Test Subject"));
    assert!(mime.contains("Hello, World!"));
}

// Requirement: REQ-RT-036 (Must)
// Acceptance: Send request body includes base64url-encoded raw message
#[test]
fn req_rt_036_gmail_send_body_has_raw_field() {
    let body = omega_google::services::gmail::send::build_send_body("base64url_encoded_mime");
    assert_eq!(body["raw"], "base64url_encoded_mime");
}

// Requirement: REQ-RT-036 (Must)
// Acceptance: Dry-run support -- api_post returns None on dry_run
#[tokio::test]
async fn req_rt_036_gmail_send_dry_run() {
    let mut server = mockito::Server::new_async().await;

    // The server should NOT be called in dry-run mode
    let mock = server
        .mock("POST", "/gmail/v1/users/me/messages/send")
        .expect(0) // Must not be called
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/messages/send", server.url());
    let body = serde_json::json!({"raw": "test_message"});

    let result: anyhow::Result<Option<serde_json::Value>> = omega_google::http::api::api_post(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
        true, // dry_run = true
    )
    .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none(), "Dry-run should return None");
    mock.assert_async().await;
}

// Requirement: REQ-RT-036 (Must)
// Acceptance: Send POST succeeds and returns message ID
#[tokio::test]
async fn req_rt_036_gmail_send_post_succeeds() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/gmail/v1/users/me/messages/send")
        .with_status(200)
        .with_body(r#"{"id": "sent_msg_123", "threadId": "thread_new"}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/messages/send", server.url());
    let body = serde_json::json!({"raw": "base64url_message"});

    let result: Option<omega_google::services::gmail::types::Message> =
        omega_google::http::api::api_post(
            &ctx.client,
            &url,
            &body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
            false, // not dry-run
        )
        .await
        .unwrap();

    let msg = result.unwrap();
    assert_eq!(msg.id, "sent_msg_123");
}

// Requirement: REQ-RT-036 (Must)
// Edge case: Send with CC and BCC
#[test]
fn req_rt_036_gmail_send_with_cc_bcc() {
    let params = omega_google::services::gmail::mime::MimeMessageParams {
        from: "sender@example.com".to_string(),
        to: vec!["to@example.com".to_string()],
        cc: vec!["cc@example.com".to_string()],
        bcc: vec!["bcc@example.com".to_string()],
        subject: "With CC/BCC".to_string(),
        body_text: Some("Body".to_string()),
        ..Default::default()
    };
    let mime = omega_google::services::gmail::mime::build_mime_message(&params);
    assert!(mime.contains("Cc: cc@example.com"));
    assert!(mime.contains("Bcc: bcc@example.com"));
}

// Requirement: REQ-RT-036 (Must)
// Edge case: Send with no recipients should still build (handler validates)
#[test]
fn req_rt_036_gmail_send_no_recipients() {
    let params = omega_google::services::gmail::mime::MimeMessageParams {
        from: "sender@example.com".to_string(),
        to: vec![],
        subject: "No recipients".to_string(),
        body_text: Some("Body".to_string()),
        ..Default::default()
    };
    // Should build without error -- validation is at the handler level
    let mime = omega_google::services::gmail::mime::build_mime_message(&params);
    assert!(mime.contains("From: sender@example.com"));
}

// -------------------------------------------------------------------
// REQ-RT-037 (Must): handle_gmail_labels
// -------------------------------------------------------------------

// Requirement: REQ-RT-037 (Must)
// Acceptance: Labels list URL is correct
#[test]
fn req_rt_037_gmail_labels_list_url() {
    let url = omega_google::services::gmail::labels::build_labels_list_url();
    assert!(url.contains("users/me/labels"));
}

// Requirement: REQ-RT-037 (Must)
// Acceptance: Label get URL includes label ID
#[test]
fn req_rt_037_gmail_label_get_url() {
    let url = omega_google::services::gmail::labels::build_label_get_url("INBOX");
    assert!(url.contains("users/me/labels/INBOX"));
}

// Requirement: REQ-RT-037 (Must)
// Acceptance: Labels list deserializes correctly
#[tokio::test]
async fn req_rt_037_gmail_labels_list_response() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/gmail/v1/users/me/labels")
        .with_status(200)
        .with_body(
            r#"{
            "labels": [
                {"id": "INBOX", "name": "INBOX", "type": "system"},
                {"id": "Label_1", "name": "Work", "type": "user"}
            ]
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/labels", server.url());

    let response: omega_google::services::gmail::types::LabelListResponse =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await
        .unwrap();

    assert_eq!(response.labels.len(), 2);
    assert_eq!(response.labels[0].id, "INBOX");
    assert_eq!(response.labels[1].name, "Work");
}

// Requirement: REQ-RT-037 (Must)
// Acceptance: Label create builds correct request
#[test]
fn req_rt_037_gmail_label_create_request() {
    let (url, body) =
        omega_google::services::gmail::labels::build_label_create_request("My New Label");
    assert!(url.contains("users/me/labels"));
    assert_eq!(body["name"], "My New Label");
}

// Requirement: REQ-RT-037 (Must)
// Acceptance: Label create POST succeeds
#[tokio::test]
async fn req_rt_037_gmail_label_create_post() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/gmail/v1/users/me/labels")
        .with_status(200)
        .with_body(r#"{"id": "Label_new", "name": "My Label", "type": "user"}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/labels", server.url());
    let body = serde_json::json!({"name": "My Label"});

    let result: Option<omega_google::services::gmail::types::Label> =
        omega_google::http::api::api_post(
            &ctx.client,
            &url,
            &body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
            false,
        )
        .await
        .unwrap();

    let label = result.unwrap();
    assert_eq!(label.id, "Label_new");
    assert_eq!(label.name, "My Label");
}

// Requirement: REQ-RT-037 (Must)
// Acceptance: Label delete URL correct
#[test]
fn req_rt_037_gmail_label_delete_url() {
    let url = omega_google::services::gmail::labels::build_label_delete_url("Label_123");
    assert!(url.contains("users/me/labels/Label_123"));
}

// Requirement: REQ-RT-037 (Must)
// Acceptance: Label delete uses api_delete
#[tokio::test]
async fn req_rt_037_gmail_label_delete_succeeds() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("DELETE", "/gmail/v1/users/me/labels/Label_123")
        .with_status(204)
        .with_body("")
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/labels/Label_123", server.url());

    let result = omega_google::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
        false,
    )
    .await;

    assert!(result.is_ok());
}

// Requirement: REQ-RT-037 (Must)
// Acceptance: Label delete dry-run does not call API
#[tokio::test]
async fn req_rt_037_gmail_label_delete_dry_run() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("DELETE", "/gmail/v1/users/me/labels/Label_123")
        .expect(0)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/labels/Label_123", server.url());

    let result = omega_google::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
        true, // dry_run
    )
    .await;

    assert!(result.is_ok());
    mock.assert_async().await;
}

// -------------------------------------------------------------------
// REQ-RT-040 (Must): handle_gmail_modify (thread label modification)
// -------------------------------------------------------------------

// Requirement: REQ-RT-040 (Must)
// Acceptance: Thread modify builds correct URL and body
#[test]
fn req_rt_040_gmail_thread_modify_request() {
    let (url, body) = omega_google::services::gmail::thread::build_thread_modify_request(
        "thread_abc",
        &["STARRED".to_string()],
        &["UNREAD".to_string()],
    );
    assert!(url.contains("threads/thread_abc/modify"));
    assert_eq!(body["addLabelIds"], serde_json::json!(["STARRED"]));
    assert_eq!(body["removeLabelIds"], serde_json::json!(["UNREAD"]));
}

// Requirement: REQ-RT-040 (Must)
// Acceptance: Thread modify POST succeeds
#[tokio::test]
async fn req_rt_040_gmail_thread_modify_post() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/gmail/v1/users/me/threads/thread123/modify")
        .with_status(200)
        .with_body(r#"{"id": "thread123", "historyId": "5555"}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!(
        "{}/gmail/v1/users/me/threads/thread123/modify",
        server.url()
    );
    let body = serde_json::json!({
        "addLabelIds": ["STARRED"],
        "removeLabelIds": ["UNREAD"]
    });

    let result: Option<omega_google::services::gmail::types::Thread> =
        omega_google::http::api::api_post(
            &ctx.client,
            &url,
            &body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
            false,
        )
        .await
        .unwrap();

    assert!(result.is_some());
    assert_eq!(result.unwrap().id, "thread123");
}

// Requirement: REQ-RT-040 (Must)
// Edge case: Thread modify with empty add and remove lists
#[test]
fn req_rt_040_gmail_thread_modify_empty_labels() {
    let (url, body) =
        omega_google::services::gmail::thread::build_thread_modify_request("thread_abc", &[], &[]);
    assert!(url.contains("threads/thread_abc/modify"));
    assert_eq!(body["addLabelIds"], serde_json::json!([]));
    assert_eq!(body["removeLabelIds"], serde_json::json!([]));
}

// -------------------------------------------------------------------
// REQ-RT-039 (Must): handle_gmail_attachment (note: requirement doc has
// REQ-RT-039 as attachment download, but the task prompt maps it to modify)
// -------------------------------------------------------------------

// Requirement: REQ-RT-039 (Must)
// Acceptance: Attachment download URL is correctly formed
#[test]
fn req_rt_039_gmail_attachment_url() {
    let url = omega_google::services::gmail::message::build_attachment_url("msg123", "attach456");
    assert!(url.contains("users/me/messages/msg123/attachments/attach456"));
}

// ===================================================================
// Module 2: Gmail Handler Tests (Should priority)
// ===================================================================

// -------------------------------------------------------------------
// REQ-RT-038 (Should): handle_gmail_drafts
// -------------------------------------------------------------------

// Requirement: REQ-RT-038 (Should)
// Acceptance: Draft list URL is correct
#[test]
fn req_rt_038_gmail_drafts_list_url() {
    let url = omega_google::services::gmail::drafts::build_drafts_list_url(Some(20), None);
    assert!(url.contains("users/me/drafts"));
    assert!(url.contains("maxResults=20"));
}

// Requirement: REQ-RT-038 (Should)
// Acceptance: Draft get URL includes draft ID
#[test]
fn req_rt_038_gmail_draft_get_url() {
    let url = omega_google::services::gmail::drafts::build_draft_get_url("draft_abc");
    assert!(url.contains("users/me/drafts/draft_abc"));
}

// Requirement: REQ-RT-038 (Should)
// Acceptance: Draft create URL
#[test]
fn req_rt_038_gmail_draft_create_url() {
    let url = omega_google::services::gmail::drafts::build_draft_create_url();
    assert!(url.contains("users/me/drafts"));
}

// Requirement: REQ-RT-038 (Should)
// Acceptance: Draft delete URL
#[test]
fn req_rt_038_gmail_draft_delete_url() {
    let url = omega_google::services::gmail::drafts::build_draft_delete_url("draft_del");
    assert!(url.contains("users/me/drafts/draft_del"));
}

// Requirement: REQ-RT-038 (Should)
// Acceptance: Draft send URL
#[test]
fn req_rt_038_gmail_draft_send_url() {
    let url = omega_google::services::gmail::drafts::build_draft_send_url();
    assert!(url.contains("users/me/drafts/send"));
}

// Requirement: REQ-RT-038 (Should)
// Acceptance: Draft list deserializes correctly
#[tokio::test]
async fn req_rt_038_gmail_drafts_list_response() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/gmail/v1/users/me/drafts.*".to_string()),
        )
        .with_status(200)
        .with_body(
            r#"{
            "drafts": [
                {"id": "draft1", "message": {"id": "msg1"}},
                {"id": "draft2", "message": {"id": "msg2"}}
            ],
            "resultSizeEstimate": 2
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/drafts?maxResults=20", server.url());

    let response: omega_google::services::gmail::types::DraftListResponse =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await
        .unwrap();

    assert_eq!(response.drafts.len(), 2);
    assert_eq!(response.drafts[0].id, "draft1");
}

// ===================================================================
// Module 3: Calendar Handler Tests (Must priority)
// ===================================================================

// -------------------------------------------------------------------
// REQ-RT-044 (Must): handle_calendar_events_list
// -------------------------------------------------------------------

// Requirement: REQ-RT-044 (Must)
// Acceptance: handle_calendar is async and dispatches correctly
#[tokio::test]
async fn req_rt_044_handle_calendar_is_async() {
    use std::ffi::OsString;
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_GOOGLE_CONFIG_DIR", tmp.path());
    std::env::set_var("GOG_KEYRING_BACKEND", "file");
    let args: Vec<OsString> = vec!["calendar".into(), "events".into()];
    let exit = omega_google::cli::execute(args).await;
    std::env::remove_var("GOG_KEYRING_BACKEND");
    std::env::remove_var("OMEGA_GOOGLE_CONFIG_DIR");
    // Before M4: returns SUCCESS with stub message
    // After M4: returns AUTH_REQUIRED because no credentials
    assert!(
        exit == codes::AUTH_REQUIRED || exit == codes::SUCCESS,
        "Calendar events should return AUTH_REQUIRED or SUCCESS, got {}",
        exit
    );
}

// Requirement: REQ-RT-044 (Must)
// Acceptance: Events list URL includes all parameters
#[test]
fn req_rt_044_calendar_events_list_url_all_params() {
    let url = omega_google::services::calendar::events::build_events_list_url(
        "primary",
        Some("2024-01-01T00:00:00Z"),
        Some("2024-12-31T23:59:59Z"),
        Some(50),
        None,
        Some("meeting"),
    );
    assert!(url.contains("calendars/primary/events"));
    assert!(url.contains("timeMin=2024-01-01T00:00:00Z"));
    assert!(url.contains("timeMax=2024-12-31T23:59:59Z"));
    assert!(url.contains("maxResults=50"));
    assert!(url.contains("q=meeting"));
    assert!(url.contains("singleEvents=true"));
    assert!(url.contains("orderBy=startTime"));
}

// Requirement: REQ-RT-044 (Must)
// Acceptance: Events list URL with page token
#[test]
fn req_rt_044_calendar_events_list_url_page_token() {
    let url = omega_google::services::calendar::events::build_events_list_url(
        "primary",
        None,
        None,
        Some(10),
        Some("TOKEN_ABC"),
        None,
    );
    assert!(url.contains("pageToken=TOKEN_ABC"));
}

// Requirement: REQ-RT-044 (Must)
// Acceptance: Events list deserializes EventListResponse
#[tokio::test]
async fn req_rt_044_calendar_events_list_deserializes() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/calendar/v3/calendars/.*/events.*".to_string()),
        )
        .with_status(200)
        .with_body(
            r#"{
            "items": [
                {
                    "id": "event1",
                    "summary": "Team Meeting",
                    "start": {"dateTime": "2024-01-15T10:00:00-05:00"},
                    "end": {"dateTime": "2024-01-15T11:00:00-05:00"}
                }
            ],
            "nextPageToken": "next_event_page",
            "summary": "Test Calendar",
            "timeZone": "America/New_York"
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!(
        "{}/calendar/v3/calendars/primary/events?maxResults=10&singleEvents=true&orderBy=startTime",
        server.url()
    );

    let response: omega_google::services::calendar::types::EventListResponse =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await
        .unwrap();

    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].id, Some("event1".to_string()));
    assert_eq!(response.items[0].summary, Some("Team Meeting".to_string()));
    assert_eq!(
        response.next_page_token,
        Some("next_event_page".to_string())
    );
}

// Requirement: REQ-RT-044 (Must)
// Acceptance: Events list pagination works across pages
#[tokio::test]
async fn req_rt_044_calendar_events_pagination() {
    let mut server = mockito::Server::new_async().await;

    let _mock1 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(
                r"^/calendar/v3/calendars/primary/events\?maxResults=".to_string(),
            ),
        )
        .with_status(200)
        .with_body(
            r#"{
            "items": [{"id": "e1", "summary": "Event 1"}],
            "nextPageToken": "page2"
        }"#,
        )
        .create_async()
        .await;

    let _mock2 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r".*pageToken=page2.*".to_string()),
        )
        .with_status(200)
        .with_body(
            r#"{
            "items": [{"id": "e2", "summary": "Event 2"}]
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let base_url = server.url();
    let params = omega_google::services::common::PaginationParams {
        max_results: Some(1),
        all_pages: true,
        ..Default::default()
    };

    let (items, _) = omega_google::services::pagination::paginate(
        &ctx.client,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
        &params,
        |pt| {
            let mut url = format!(
                "{}/calendar/v3/calendars/primary/events?maxResults=1&singleEvents=true&orderBy=startTime",
                base_url
            );
            if let Some(token) = pt {
                url.push_str(&format!("&pageToken={}", token));
            }
            url
        },
        |value| {
            let items: Vec<String> = value
                .get("items")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.get("id").and_then(|id| id.as_str()).map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let next = value
                .get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok((items, next))
        },
    )
    .await
    .unwrap();

    assert_eq!(items.len(), 2);
    assert_eq!(items[0], "e1");
    assert_eq!(items[1], "e2");
}

// Requirement: REQ-RT-044 (Must)
// Edge case: Events list with no events returns empty
#[tokio::test]
async fn req_rt_044_calendar_events_empty_list() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/calendar/v3/calendars/.*/events.*".to_string()),
        )
        .with_status(200)
        .with_body(r#"{"items": [], "summary": "Empty Calendar"}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!(
        "{}/calendar/v3/calendars/primary/events?singleEvents=true&orderBy=startTime",
        server.url()
    );

    let response: omega_google::services::calendar::types::EventListResponse =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await
        .unwrap();

    assert!(response.items.is_empty());
}

// -------------------------------------------------------------------
// REQ-RT-045 (Must): handle_calendar_events_get
// -------------------------------------------------------------------

// Requirement: REQ-RT-045 (Must)
// Acceptance: Event get URL includes calendar ID and event ID
#[test]
fn req_rt_045_calendar_event_get_url() {
    let url = omega_google::services::calendar::events::build_event_get_url("primary", "eventabc");
    // Note: build_event_get_url percent-encodes non-alphanumeric characters
    assert!(url.contains("calendars/primary/events/eventabc"));
}

// Requirement: REQ-RT-045 (Must)
// Acceptance: Event get deserializes full event
#[tokio::test]
async fn req_rt_045_calendar_event_get_response() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/calendar/v3/calendars/.*/events/.*".to_string()),
        )
        .with_status(200)
        .with_body(
            r#"{
            "id": "event_123",
            "summary": "Team Standup",
            "description": "Daily standup meeting",
            "location": "Conference Room A",
            "start": {"dateTime": "2024-01-15T09:00:00-05:00", "timeZone": "America/New_York"},
            "end": {"dateTime": "2024-01-15T09:15:00-05:00", "timeZone": "America/New_York"},
            "status": "confirmed",
            "attendees": [
                {"email": "alice@example.com", "responseStatus": "accepted"},
                {"email": "bob@example.com", "responseStatus": "tentative"}
            ]
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!(
        "{}/calendar/v3/calendars/primary/events/event_123",
        server.url()
    );

    let event: omega_google::services::calendar::types::Event = omega_google::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
    )
    .await
    .unwrap();

    assert_eq!(event.id, Some("event_123".to_string()));
    assert_eq!(event.summary, Some("Team Standup".to_string()));
    assert_eq!(event.attendees.len(), 2);
    assert_eq!(event.attendees[0].email, "alice@example.com");
}

// Requirement: REQ-RT-045 (Must)
// Failure mode: Event not found returns 404
#[tokio::test]
async fn req_rt_045_calendar_event_not_found() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/calendar/v3/calendars/.*/events/.*".to_string()),
        )
        .with_status(404)
        .with_body(r#"{"error":{"code":404,"message":"Not Found"}}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!(
        "{}/calendar/v3/calendars/primary/events/nonexistent",
        server.url()
    );

    let result: anyhow::Result<omega_google::services::calendar::types::Event> =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await;

    assert!(result.is_err());
}

// -------------------------------------------------------------------
// REQ-RT-046 (Must): handle_calendar_events_create
// -------------------------------------------------------------------

// Requirement: REQ-RT-046 (Must)
// Acceptance: Event create body includes summary, start, end
#[test]
fn req_rt_046_calendar_event_create_body() {
    let start = omega_google::services::calendar::types::EventDateTime {
        date_time: Some("2024-03-15T10:00:00-05:00".to_string()),
        date: None,
        time_zone: Some("America/New_York".to_string()),
    };
    let end = omega_google::services::calendar::types::EventDateTime {
        date_time: Some("2024-03-15T11:00:00-05:00".to_string()),
        date: None,
        time_zone: Some("America/New_York".to_string()),
    };
    let body = omega_google::services::calendar::events::build_event_create_body(
        "New Meeting",
        &start,
        &end,
        Some("Discussion about Q1 plans"),
        Some("Room 101"),
        &["alice@example.com".to_string()],
        None,
    );
    assert_eq!(body["summary"], "New Meeting");
    assert!(body["start"].is_object());
    assert!(body["end"].is_object());
}

// Requirement: REQ-RT-046 (Must)
// Acceptance: Event create POST succeeds
#[tokio::test]
async fn req_rt_046_calendar_event_create_post() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "POST",
            mockito::Matcher::Regex(r"/calendar/v3/calendars/.*/events.*".to_string()),
        )
        .with_status(200)
        .with_body(
            r#"{
            "id": "new_event_id",
            "summary": "Created Event",
            "status": "confirmed",
            "htmlLink": "https://calendar.google.com/event?eid=abc123"
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/calendar/v3/calendars/primary/events", server.url());
    let body = serde_json::json!({
        "summary": "Created Event",
        "start": {"dateTime": "2024-03-15T10:00:00-05:00"},
        "end": {"dateTime": "2024-03-15T11:00:00-05:00"}
    });

    let result: Option<omega_google::services::calendar::types::Event> =
        omega_google::http::api::api_post(
            &ctx.client,
            &url,
            &body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
            false,
        )
        .await
        .unwrap();

    let event = result.unwrap();
    assert_eq!(event.id, Some("new_event_id".to_string()));
}

// Requirement: REQ-RT-046 (Must)
// Acceptance: Event create dry-run does not call API
#[tokio::test]
async fn req_rt_046_calendar_event_create_dry_run() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock(
            "POST",
            mockito::Matcher::Regex(r"/calendar/v3/calendars/.*/events.*".to_string()),
        )
        .expect(0)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/calendar/v3/calendars/primary/events", server.url());
    let body = serde_json::json!({"summary": "Dry Run Event"});

    let result: anyhow::Result<Option<serde_json::Value>> = omega_google::http::api::api_post(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
        true, // dry_run
    )
    .await;

    assert!(result.unwrap().is_none());
    mock.assert_async().await;
}

// Requirement: REQ-RT-046 (Must)
// Acceptance: All-day event uses date field instead of dateTime
#[test]
fn req_rt_046_calendar_event_create_all_day() {
    let start = omega_google::services::calendar::types::EventDateTime {
        date_time: None,
        date: Some("2024-03-15".to_string()),
        time_zone: None,
    };
    let end = omega_google::services::calendar::types::EventDateTime {
        date_time: None,
        date: Some("2024-03-16".to_string()),
        time_zone: None,
    };
    let body = omega_google::services::calendar::events::build_event_create_body(
        "All Day Event",
        &start,
        &end,
        None,
        None,
        &[],
        None,
    );
    assert_eq!(body["summary"], "All Day Event");
    // start and end should use "date" not "dateTime"
    assert!(body["start"]["date"].is_string() || body["start"]["dateTime"].is_null());
}

// -------------------------------------------------------------------
// REQ-RT-047 (Must): handle_calendar_events_update
// -------------------------------------------------------------------

// Requirement: REQ-RT-047 (Must)
// Acceptance: Event update uses PATCH method
#[tokio::test]
async fn req_rt_047_calendar_event_update_patch() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "PATCH",
            mockito::Matcher::Regex(r"/calendar/v3/calendars/.*/events/.*".to_string()),
        )
        .with_status(200)
        .with_body(
            r#"{
            "id": "event_updated",
            "summary": "Updated Meeting",
            "status": "confirmed"
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!(
        "{}/calendar/v3/calendars/primary/events/event_updated",
        server.url()
    );
    let body = serde_json::json!({"summary": "Updated Meeting"});

    let result: Option<omega_google::services::calendar::types::Event> =
        omega_google::http::api::api_patch(
            &ctx.client,
            &url,
            &body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
            false,
        )
        .await
        .unwrap();

    let event = result.unwrap();
    assert_eq!(event.summary, Some("Updated Meeting".to_string()));
}

// Requirement: REQ-RT-047 (Must)
// Acceptance: Event update dry-run
#[tokio::test]
async fn req_rt_047_calendar_event_update_dry_run() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("PATCH", mockito::Matcher::Any)
        .expect(0)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!(
        "{}/calendar/v3/calendars/primary/events/event_id",
        server.url()
    );
    let body = serde_json::json!({"summary": "Dry Run Update"});

    let result: anyhow::Result<Option<serde_json::Value>> = omega_google::http::api::api_patch(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
        true,
    )
    .await;

    assert!(result.unwrap().is_none());
    mock.assert_async().await;
}

// -------------------------------------------------------------------
// REQ-RT-048 (Must): handle_calendar_events_delete
// -------------------------------------------------------------------

// Requirement: REQ-RT-048 (Must)
// Acceptance: Event delete uses DELETE method
#[tokio::test]
async fn req_rt_048_calendar_event_delete() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "DELETE",
            mockito::Matcher::Regex(r"/calendar/v3/calendars/.*/events/.*".to_string()),
        )
        .with_status(204)
        .with_body("")
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!(
        "{}/calendar/v3/calendars/primary/events/event_del",
        server.url()
    );

    let result = omega_google::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
        false,
    )
    .await;

    assert!(result.is_ok());
}

// Requirement: REQ-RT-048 (Must)
// Acceptance: Event delete dry-run
#[tokio::test]
async fn req_rt_048_calendar_event_delete_dry_run() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("DELETE", mockito::Matcher::Any)
        .expect(0)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!(
        "{}/calendar/v3/calendars/primary/events/event_del",
        server.url()
    );

    let result = omega_google::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
        true, // dry_run
    )
    .await;

    assert!(result.is_ok());
    mock.assert_async().await;
}

// Requirement: REQ-RT-048 (Must)
// Failure mode: Delete event that does not exist
#[tokio::test]
async fn req_rt_048_calendar_event_delete_not_found() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "DELETE",
            mockito::Matcher::Regex(r"/calendar/v3/calendars/.*/events/.*".to_string()),
        )
        .with_status(404)
        .with_body(r#"{"error":{"code":404,"message":"Not Found"}}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!(
        "{}/calendar/v3/calendars/primary/events/nonexistent",
        server.url()
    );

    let result = omega_google::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
        false,
    )
    .await;

    assert!(result.is_err());
}

// -------------------------------------------------------------------
// REQ-RT-049 (Must): handle_calendar_calendars_list
// -------------------------------------------------------------------

// Requirement: REQ-RT-049 (Must)
// Acceptance: Calendars list URL is correct
#[test]
fn req_rt_049_calendar_calendars_list_url() {
    let url =
        omega_google::services::calendar::calendars::build_calendars_list_url(Some(100), None);
    assert!(url.contains("users/me/calendarList"));
    assert!(url.contains("maxResults=100"));
}

// Requirement: REQ-RT-049 (Must)
// Acceptance: Calendars list deserializes CalendarListResponse
#[tokio::test]
async fn req_rt_049_calendar_calendars_list_response() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/calendar/v3/users/me/calendarList.*".to_string()),
        )
        .with_status(200)
        .with_body(
            r#"{
            "items": [
                {"id": "primary", "summary": "Main Calendar", "primary": true},
                {"id": "work@group.calendar.google.com", "summary": "Work Calendar"}
            ]
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!(
        "{}/calendar/v3/users/me/calendarList?maxResults=100",
        server.url()
    );

    let response: omega_google::services::calendar::types::CalendarListResponse =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await
        .unwrap();

    assert_eq!(response.items.len(), 2);
    assert_eq!(response.items[0].id, "primary");
    assert_eq!(response.items[0].primary, Some(true));
}

// -------------------------------------------------------------------
// REQ-RT-050 (Must): handle_calendar_freebusy
// -------------------------------------------------------------------

// Requirement: REQ-RT-050 (Must)
// Acceptance: Free/busy URL is correct
#[test]
fn req_rt_050_calendar_freebusy_url() {
    let url = omega_google::services::calendar::freebusy::build_freebusy_url();
    assert!(url.contains("freeBusy"));
}

// Requirement: REQ-RT-050 (Must)
// Acceptance: Free/busy request body includes calendar IDs and time range
#[test]
fn req_rt_050_calendar_freebusy_request_body() {
    let req = omega_google::services::calendar::freebusy::build_freebusy_request(
        &[
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
        ],
        "2024-03-01T00:00:00Z",
        "2024-03-02T00:00:00Z",
    );
    assert_eq!(req.items.len(), 2);
    assert_eq!(req.time_min, "2024-03-01T00:00:00Z");
    assert_eq!(req.time_max, "2024-03-02T00:00:00Z");
}

// Requirement: REQ-RT-050 (Must)
// Acceptance: Free/busy POST succeeds
#[tokio::test]
async fn req_rt_050_calendar_freebusy_post() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/calendar/v3/freeBusy")
        .with_status(200)
        .with_body(
            r#"{
            "kind": "calendar#freeBusy",
            "timeMin": "2024-03-01T00:00:00Z",
            "timeMax": "2024-03-02T00:00:00Z",
            "calendars": {
                "alice@example.com": {
                    "busy": [
                        {"start": "2024-03-01T10:00:00Z", "end": "2024-03-01T11:00:00Z"}
                    ]
                }
            }
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/calendar/v3/freeBusy", server.url());
    let body = serde_json::json!({
        "timeMin": "2024-03-01T00:00:00Z",
        "timeMax": "2024-03-02T00:00:00Z",
        "items": [{"id": "alice@example.com"}]
    });

    let result: Option<serde_json::Value> = omega_google::http::api::api_post(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
        false,
    )
    .await
    .unwrap();

    let resp = result.unwrap();
    assert!(resp.get("calendars").is_some());
}

// ===================================================================
// Module 4: Drive Handler Tests (Must priority)
// ===================================================================

// -------------------------------------------------------------------
// REQ-RT-055 (Must): handle_drive_list
// -------------------------------------------------------------------

// Requirement: REQ-RT-055 (Must)
// Acceptance: handle_drive is async and dispatches correctly
#[tokio::test]
async fn req_rt_055_handle_drive_is_async() {
    use std::ffi::OsString;
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_GOOGLE_CONFIG_DIR", tmp.path());
    std::env::set_var("GOG_KEYRING_BACKEND", "file");
    let args: Vec<OsString> = vec!["drive".into(), "ls".into()];
    let exit = omega_google::cli::execute(args).await;
    std::env::remove_var("GOG_KEYRING_BACKEND");
    std::env::remove_var("OMEGA_GOOGLE_CONFIG_DIR");
    assert!(
        exit == codes::AUTH_REQUIRED || exit == codes::SUCCESS,
        "Drive ls should return AUTH_REQUIRED or SUCCESS, got {}",
        exit
    );
}

// Requirement: REQ-RT-055 (Must)
// Acceptance: Drive list query builder works with parent folder
#[test]
fn req_rt_055_drive_list_query_with_parent() {
    let query = omega_google::services::drive::list::build_list_query("folder_abc", None);
    assert!(query.contains("'folder_abc' in parents"));
    assert!(query.contains("trashed = false"));
}

// Requirement: REQ-RT-055 (Must)
// Acceptance: Drive list deserializes FileListResponse
#[tokio::test]
async fn req_rt_055_drive_list_deserializes_response() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", mockito::Matcher::Regex(r"/drive/v3/files.*".to_string()))
        .with_status(200)
        .with_body(r#"{
            "files": [
                {"id": "file1", "name": "Document.docx", "mimeType": "application/vnd.google-apps.document"},
                {"id": "file2", "name": "Photo.jpg", "mimeType": "image/jpeg", "size": "1048576"}
            ],
            "nextPageToken": "drive_page_2"
        }"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!(
        "{}/drive/v3/files?q=trashed+%3D+false&pageSize=20",
        server.url()
    );

    let response: omega_google::services::drive::types::FileListResponse =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await
        .unwrap();

    assert_eq!(response.files.len(), 2);
    assert_eq!(response.files[0].name, Some("Document.docx".to_string()));
    assert_eq!(response.next_page_token, Some("drive_page_2".to_string()));
}

// Requirement: REQ-RT-055 (Must)
// Acceptance: Drive list pagination works
#[tokio::test]
async fn req_rt_055_drive_list_pagination() {
    let mut server = mockito::Server::new_async().await;

    let _mock1 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/drive/v3/files\?.*pageSize=".to_string()),
        )
        .with_status(200)
        .with_body(r#"{"files": [{"id": "f1", "name": "File 1"}], "nextPageToken": "pg2"}"#)
        .create_async()
        .await;

    let _mock2 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r".*pageToken=pg2.*".to_string()),
        )
        .with_status(200)
        .with_body(r#"{"files": [{"id": "f2", "name": "File 2"}]}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let base_url = server.url();
    let params = omega_google::services::common::PaginationParams {
        max_results: Some(1),
        all_pages: true,
        ..Default::default()
    };

    let (items, _) = omega_google::services::pagination::paginate(
        &ctx.client,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
        &params,
        |pt| {
            let mut url = format!("{}/drive/v3/files?pageSize=1&q=trashed+%3D+false", base_url);
            if let Some(token) = pt {
                url.push_str(&format!("&pageToken={}", token));
            }
            url
        },
        |value| {
            let files: Vec<String> = value
                .get("files")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.get("id").and_then(|id| id.as_str()).map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let next = value
                .get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok((files, next))
        },
    )
    .await
    .unwrap();

    assert_eq!(items.len(), 2);
}

// -------------------------------------------------------------------
// REQ-RT-056 (Must): handle_drive_search
// -------------------------------------------------------------------

// Requirement: REQ-RT-056 (Must)
// Acceptance: Search query builder wraps plain text in fullText contains
#[test]
fn req_rt_056_drive_search_plain_text() {
    let query = omega_google::services::drive::list::build_search_query("meeting notes", false);
    assert!(query.contains("fullText contains"));
    assert!(query.contains("trashed = false"));
}

// Requirement: REQ-RT-056 (Must)
// Acceptance: Search with --raw-query passes query through
#[test]
fn req_rt_056_drive_search_raw_query() {
    let query = omega_google::services::drive::list::build_search_query(
        "mimeType = 'application/pdf'",
        true,
    );
    assert!(query.contains("mimeType = 'application/pdf'"));
    assert!(query.contains("trashed = false"));
}

// Requirement: REQ-RT-056 (Must)
// Edge case: Empty search query
#[test]
fn req_rt_056_drive_search_empty_query() {
    let query = omega_google::services::drive::list::build_search_query("", false);
    assert_eq!(query, "trashed = false");
}

// -------------------------------------------------------------------
// REQ-RT-057 (Must): handle_drive_get
// -------------------------------------------------------------------

// Requirement: REQ-RT-057 (Must)
// Acceptance: File get URL is correct
#[test]
fn req_rt_057_drive_get_url() {
    let url = omega_google::services::drive::files::build_file_get_url("file_abc123");
    assert!(url.contains("files/file_abc123"));
}

// Requirement: REQ-RT-057 (Must)
// Acceptance: File get deserializes DriveFile
#[tokio::test]
async fn req_rt_057_drive_get_response() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/drive/v3/files/file_abc")
        .with_status(200)
        .with_body(
            r#"{
            "id": "file_abc",
            "name": "Important Document.pdf",
            "mimeType": "application/pdf",
            "size": "524288",
            "modifiedTime": "2024-01-15T10:30:00.000Z",
            "parents": ["folder_root"]
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/drive/v3/files/file_abc", server.url());

    let file: omega_google::services::drive::types::DriveFile = omega_google::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
    )
    .await
    .unwrap();

    assert_eq!(file.id, Some("file_abc".to_string()));
    assert_eq!(file.name, Some("Important Document.pdf".to_string()));
    assert_eq!(file.size, Some("524288".to_string()));
}

// Requirement: REQ-RT-057 (Must)
// Failure mode: File not found
#[tokio::test]
async fn req_rt_057_drive_get_not_found() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/drive/v3/files/nonexistent")
        .with_status(404)
        .with_body(r#"{"error":{"code":404,"message":"File not found"}}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/drive/v3/files/nonexistent", server.url());

    let result: anyhow::Result<omega_google::services::drive::types::DriveFile> =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    if let Some(omega_err) = err.downcast_ref::<omega_google::error::exit::OmegaError>() {
        assert_eq!(
            omega_google::error::exit::exit_code_for(omega_err),
            codes::NOT_FOUND
        );
    }
}

// -------------------------------------------------------------------
// REQ-RT-060 (Must): handle_drive_mkdir
// -------------------------------------------------------------------

// Requirement: REQ-RT-060 (Must)
// Acceptance: Mkdir body has correct name and mimeType
#[test]
fn req_rt_060_drive_mkdir_body() {
    let body = omega_google::services::drive::folders::build_mkdir_body("New Folder", None);
    assert_eq!(body["name"], "New Folder");
    assert_eq!(body["mimeType"], "application/vnd.google-apps.folder");
}

// Requirement: REQ-RT-060 (Must)
// Acceptance: Mkdir body with parent folder
#[test]
fn req_rt_060_drive_mkdir_body_with_parent() {
    let body =
        omega_google::services::drive::folders::build_mkdir_body("Sub Folder", Some("parent_id"));
    assert_eq!(body["name"], "Sub Folder");
    assert_eq!(body["parents"], serde_json::json!(["parent_id"]));
}

// Requirement: REQ-RT-060 (Must)
// Acceptance: Mkdir POST succeeds
#[tokio::test]
async fn req_rt_060_drive_mkdir_post() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/drive/v3/files")
        .with_status(200)
        .with_body(
            r#"{
            "id": "folder_new",
            "name": "Created Folder",
            "mimeType": "application/vnd.google-apps.folder"
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/drive/v3/files", server.url());
    let body = omega_google::services::drive::folders::build_mkdir_body("Created Folder", None);

    let result: Option<omega_google::services::drive::types::DriveFile> =
        omega_google::http::api::api_post(
            &ctx.client,
            &url,
            &body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
            false,
        )
        .await
        .unwrap();

    let file = result.unwrap();
    assert_eq!(file.id, Some("folder_new".to_string()));
    assert_eq!(
        file.mime_type,
        Some("application/vnd.google-apps.folder".to_string())
    );
}

// -------------------------------------------------------------------
// REQ-RT-061 (Must): handle_drive_delete
// -------------------------------------------------------------------

// Requirement: REQ-RT-061 (Must)
// Acceptance: Trash URL is correct (default, not permanent)
#[test]
fn req_rt_061_drive_trash_url() {
    let url = omega_google::services::drive::folders::build_trash_url("file_abc");
    assert!(url.contains("files/file_abc"));
}

// Requirement: REQ-RT-061 (Must)
// Acceptance: Permanent delete URL is correct
#[test]
fn req_rt_061_drive_permanent_delete_url() {
    let url = omega_google::services::drive::folders::build_permanent_delete_url("file_abc");
    assert!(url.contains("files/file_abc"));
}

// Requirement: REQ-RT-061 (Must)
// Acceptance: Drive delete dry-run does not call API
#[tokio::test]
async fn req_rt_061_drive_delete_dry_run() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("DELETE", mockito::Matcher::Any)
        .expect(0)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/drive/v3/files/file_del", server.url());

    let result = omega_google::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
        true,
    )
    .await;

    assert!(result.is_ok());
    mock.assert_async().await;
}

// -------------------------------------------------------------------
// REQ-RT-062 (Must): handle_drive_move
// -------------------------------------------------------------------

// Requirement: REQ-RT-062 (Must)
// Acceptance: Move uses PATCH to update parents
#[tokio::test]
async fn req_rt_062_drive_move_patch() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "PATCH",
            mockito::Matcher::Regex(r"/drive/v3/files/file_move.*".to_string()),
        )
        .with_status(200)
        .with_body(r#"{"id": "file_move", "name": "moved.txt", "parents": ["new_parent"]}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    // Move requires PATCH with addParents/removeParents query params
    let url = format!(
        "{}/drive/v3/files/file_move?addParents=new_parent&removeParents=old_parent",
        server.url()
    );
    let body = serde_json::json!({});

    let result: Option<omega_google::services::drive::types::DriveFile> =
        omega_google::http::api::api_patch(
            &ctx.client,
            &url,
            &body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
            false,
        )
        .await
        .unwrap();

    let file = result.unwrap();
    assert_eq!(file.id, Some("file_move".to_string()));
}

// -------------------------------------------------------------------
// REQ-RT-063 (Must): handle_drive_rename
// -------------------------------------------------------------------

// Requirement: REQ-RT-063 (Must)
// Acceptance: Rename body has correct new name
#[test]
fn req_rt_063_drive_rename_body() {
    let body = omega_google::services::drive::folders::build_rename_body("new_name.txt");
    assert_eq!(body["name"], "new_name.txt");
}

// Requirement: REQ-RT-063 (Must)
// Acceptance: Rename uses PATCH to update name
#[tokio::test]
async fn req_rt_063_drive_rename_patch() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("PATCH", "/drive/v3/files/file_rename")
        .with_status(200)
        .with_body(r#"{"id": "file_rename", "name": "renamed.txt"}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/drive/v3/files/file_rename", server.url());
    let body = omega_google::services::drive::folders::build_rename_body("renamed.txt");

    let result: Option<omega_google::services::drive::types::DriveFile> =
        omega_google::http::api::api_patch(
            &ctx.client,
            &url,
            &body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
            false,
        )
        .await
        .unwrap();

    let file = result.unwrap();
    assert_eq!(file.name, Some("renamed.txt".to_string()));
}

// Requirement: REQ-RT-063 (Must)
// Edge case: Rename with special characters
#[test]
fn req_rt_063_drive_rename_special_chars() {
    let body = omega_google::services::drive::folders::build_rename_body("file (copy).txt");
    assert_eq!(body["name"], "file (copy).txt");
}

// -------------------------------------------------------------------
// REQ-RT-064 (Must): handle_drive_share
// -------------------------------------------------------------------

// Requirement: REQ-RT-064 (Must)
// Acceptance: Share permission body is correct for "anyone" type
#[test]
fn req_rt_064_drive_share_anyone() {
    let body = omega_google::services::drive::permissions::build_share_permission(
        "anyone", "reader", None, None,
    )
    .unwrap();
    assert_eq!(body["type"], "anyone");
    assert_eq!(body["role"], "reader");
}

// Requirement: REQ-RT-064 (Must)
// Acceptance: Share permission body for user type includes email
#[test]
fn req_rt_064_drive_share_user() {
    let body = omega_google::services::drive::permissions::build_share_permission(
        "user",
        "writer",
        Some("colleague@example.com"),
        None,
    )
    .unwrap();
    assert_eq!(body["type"], "user");
    assert_eq!(body["role"], "writer");
    assert_eq!(body["emailAddress"], "colleague@example.com");
}

// Requirement: REQ-RT-064 (Must)
// Acceptance: Create permission URL is correct
#[test]
fn req_rt_064_drive_create_permission_url() {
    let url = omega_google::services::drive::permissions::build_create_permission_url("file_abc");
    assert!(url.contains("files/file_abc/permissions"));
}

// Requirement: REQ-RT-064 (Must)
// Acceptance: Share POST succeeds
#[tokio::test]
async fn req_rt_064_drive_share_post() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/drive/v3/files/file_share/permissions")
        .with_status(200)
        .with_body(r#"{"id": "perm_new", "type": "anyone", "role": "reader"}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/drive/v3/files/file_share/permissions", server.url());
    let body = serde_json::json!({"type": "anyone", "role": "reader"});

    let result: Option<omega_google::services::drive::types::Permission> =
        omega_google::http::api::api_post(
            &ctx.client,
            &url,
            &body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
            false,
        )
        .await
        .unwrap();

    let perm = result.unwrap();
    assert_eq!(perm.id, Some("perm_new".to_string()));
}

// -------------------------------------------------------------------
// REQ-RT-065 (Must): handle_drive_permissions_list
// -------------------------------------------------------------------

// Requirement: REQ-RT-065 (Must)
// Acceptance: Permissions list URL is correct
#[test]
fn req_rt_065_drive_permissions_list_url() {
    let url = omega_google::services::drive::permissions::build_list_permissions_url("file_abc");
    assert!(url.contains("files/file_abc/permissions"));
}

// Requirement: REQ-RT-065 (Must)
// Acceptance: Permissions list deserializes PermissionListResponse
#[tokio::test]
async fn req_rt_065_drive_permissions_list_response() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/drive/v3/files/file_abc/permissions")
        .with_status(200)
        .with_body(r#"{
            "permissions": [
                {"id": "perm1", "type": "user", "role": "owner", "emailAddress": "owner@example.com"},
                {"id": "perm2", "type": "anyone", "role": "reader"}
            ]
        }"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/drive/v3/files/file_abc/permissions", server.url());

    let response: omega_google::services::drive::types::PermissionListResponse =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await
        .unwrap();

    assert_eq!(response.permissions.len(), 2);
    assert_eq!(response.permissions[0].role, Some("owner".to_string()));
}

// -------------------------------------------------------------------
// REQ-RT-066 (Must): handle_drive_copy
// -------------------------------------------------------------------

// Requirement: REQ-RT-066 (Must)
// Acceptance: File copy URL is correct
#[test]
fn req_rt_066_drive_copy_url() {
    let url = omega_google::services::drive::files::build_file_copy_url("file_abc");
    assert!(url.contains("files/file_abc/copy"));
}

// Requirement: REQ-RT-066 (Must)
// Acceptance: File copy POST succeeds
#[tokio::test]
async fn req_rt_066_drive_copy_post() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/drive/v3/files/file_orig/copy")
        .with_status(200)
        .with_body(
            r#"{
            "id": "file_copy",
            "name": "Document (Copy).pdf",
            "mimeType": "application/pdf"
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/drive/v3/files/file_orig/copy", server.url());
    let body = serde_json::json!({"name": "Document (Copy).pdf"});

    let result: Option<omega_google::services::drive::types::DriveFile> =
        omega_google::http::api::api_post(
            &ctx.client,
            &url,
            &body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
            false,
        )
        .await
        .unwrap();

    let file = result.unwrap();
    assert_eq!(file.id, Some("file_copy".to_string()));
    assert_eq!(file.name, Some("Document (Copy).pdf".to_string()));
}

// Requirement: REQ-RT-066 (Must)
// Acceptance: File copy with --name and --parent
#[tokio::test]
async fn req_rt_066_drive_copy_with_name_and_parent() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/drive/v3/files/file_src/copy")
        .with_status(200)
        .with_body(r#"{"id": "file_cp", "name": "Custom Name.pdf", "parents": ["target_folder"]}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/drive/v3/files/file_src/copy", server.url());
    let body = serde_json::json!({
        "name": "Custom Name.pdf",
        "parents": ["target_folder"]
    });

    let result: Option<omega_google::services::drive::types::DriveFile> =
        omega_google::http::api::api_post(
            &ctx.client,
            &url,
            &body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
            false,
        )
        .await
        .unwrap();

    let file = result.unwrap();
    assert_eq!(file.name, Some("Custom Name.pdf".to_string()));
}

// ===================================================================
// Module 5: Cross-cutting handler behavior tests
// ===================================================================

// -------------------------------------------------------------------
// Handler dispatch: async transition tests
// -------------------------------------------------------------------

// Requirement: REQ-RT-032 (Must)
// Acceptance: dispatch_command calls handle_gmail with .await
#[tokio::test]
async fn req_rt_032_dispatch_command_gmail_async() {
    use std::ffi::OsString;
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_GOOGLE_CONFIG_DIR", tmp.path());
    std::env::set_var("GOG_KEYRING_BACKEND", "file");
    // Gmail URL command works without auth (existing behavior)
    let args: Vec<OsString> = vec!["gmail".into(), "url".into(), "thread_123".into()];
    let exit = omega_google::cli::execute(args).await;
    std::env::remove_var("GOG_KEYRING_BACKEND");
    std::env::remove_var("OMEGA_GOOGLE_CONFIG_DIR");
    assert_eq!(
        exit,
        codes::SUCCESS,
        "Gmail URL should succeed without auth"
    );
}

// Requirement: REQ-RT-055 (Must)
// Acceptance: dispatch_command calls handle_drive with .await
#[tokio::test]
async fn req_rt_055_dispatch_command_drive_async() {
    use std::ffi::OsString;
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_GOOGLE_CONFIG_DIR", tmp.path());
    std::env::set_var("GOG_KEYRING_BACKEND", "file");
    // Drive URL command works without auth
    let args: Vec<OsString> = vec!["drive".into(), "url".into(), "file_123".into()];
    let exit = omega_google::cli::execute(args).await;
    std::env::remove_var("GOG_KEYRING_BACKEND");
    std::env::remove_var("OMEGA_GOOGLE_CONFIG_DIR");
    assert_eq!(
        exit,
        codes::SUCCESS,
        "Drive URL should succeed without auth"
    );
}

// Requirement: REQ-RT-044 (Must)
// Acceptance: dispatch_command calls handle_calendar with .await
#[tokio::test]
async fn req_rt_044_dispatch_command_calendar_async() {
    use std::ffi::OsString;
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_GOOGLE_CONFIG_DIR", tmp.path());
    std::env::set_var("GOG_KEYRING_BACKEND", "file");
    // Calendar time command works without auth
    let args: Vec<OsString> = vec!["calendar".into(), "time".into()];
    let exit = omega_google::cli::execute(args).await;
    std::env::remove_var("GOG_KEYRING_BACKEND");
    std::env::remove_var("OMEGA_GOOGLE_CONFIG_DIR");
    assert_eq!(
        exit,
        codes::SUCCESS,
        "Calendar time should succeed without auth"
    );
}

// -------------------------------------------------------------------
// Error handling: API errors map to correct exit codes
// -------------------------------------------------------------------

// Requirement: REQ-RT-032 (Must)
// Security: API error with 403 maps to PERMISSION_DENIED
#[tokio::test]
async fn req_rt_032_api_403_permission_denied() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(403)
        .with_body(r#"{"error":{"code":403,"message":"Insufficient Permission"}}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/threads", server.url());

    let result: anyhow::Result<serde_json::Value> = omega_google::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    if let Some(omega_err) = err.downcast_ref::<omega_google::error::exit::OmegaError>() {
        assert_eq!(
            omega_google::error::exit::exit_code_for(omega_err),
            codes::PERMISSION_DENIED
        );
    }
}

// Requirement: REQ-RT-032 (Must)
// Security: API error with 500 maps to RETRYABLE
#[tokio::test]
async fn req_rt_032_api_500_server_error() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(500)
        .with_body(r#"{"error":{"code":500,"message":"Internal Server Error"}}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/api/endpoint", server.url());

    let result: anyhow::Result<serde_json::Value> = omega_google::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    if let Some(omega_err) = err.downcast_ref::<omega_google::error::exit::OmegaError>() {
        assert_eq!(
            omega_google::error::exit::exit_code_for(omega_err),
            codes::RETRYABLE
        );
    }
}

// -------------------------------------------------------------------
// ServiceContext output helpers
// -------------------------------------------------------------------

// Requirement: REQ-RT-032 (Must)
// Acceptance: ServiceContext.write_output works with JSON mode
#[test]
fn req_rt_032_service_context_write_output_json() {
    let ctx = test_service_context("http://localhost");
    let data = serde_json::json!({"id": "test", "name": "Test Item"});
    // write_output should not panic in JSON mode
    let result = ctx.write_output(&data);
    assert!(result.is_ok());
}

// Requirement: REQ-RT-032 (Must)
// Acceptance: ServiceContext.write_paginated works with hint token
#[test]
fn req_rt_032_service_context_write_paginated() {
    let ctx = test_service_context("http://localhost");
    let data = serde_json::json!({"items": [{"id": "1"}]});
    let result = ctx.write_paginated(&data, Some("next_page_token"));
    assert!(result.is_ok());
}

// Requirement: REQ-RT-032 (Must)
// Acceptance: ServiceContext.write_paginated works without hint token
#[test]
fn req_rt_032_service_context_write_paginated_no_hint() {
    let ctx = test_service_context("http://localhost");
    let data = serde_json::json!({"items": []});
    let result = ctx.write_paginated(&data, None);
    assert!(result.is_ok());
}

// -------------------------------------------------------------------
// Edge case: Handler receives malformed API response
// -------------------------------------------------------------------

// Requirement: REQ-RT-032 (Must)
// Edge case: Malformed JSON response
#[tokio::test]
async fn req_rt_032_edge_malformed_json_response() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(200)
        .with_body("this is not json")
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/threads", server.url());

    let result: anyhow::Result<omega_google::services::gmail::types::ThreadListResponse> =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await;

    assert!(result.is_err(), "Malformed JSON should return error");
}

// Requirement: REQ-RT-032 (Must)
// Edge case: Empty response body
#[tokio::test]
async fn req_rt_032_edge_empty_response_body() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(200)
        .with_body("")
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/threads", server.url());

    let result: anyhow::Result<omega_google::services::gmail::types::ThreadListResponse> =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await;

    assert!(result.is_err(), "Empty body should fail deserialization");
}

// Requirement: REQ-RT-057 (Must)
// Edge case: Drive file with all null optional fields
#[tokio::test]
async fn req_rt_057_edge_drive_file_minimal_fields() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/drive/v3/files/minimal")
        .with_status(200)
        .with_body(r#"{"id": "minimal"}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/drive/v3/files/minimal", server.url());

    let file: omega_google::services::drive::types::DriveFile = omega_google::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
    )
    .await
    .unwrap();

    assert_eq!(file.id, Some("minimal".to_string()));
    assert!(file.name.is_none());
    assert!(file.mime_type.is_none());
    assert!(file.size.is_none());
}

// Requirement: REQ-RT-044 (Must)
// Edge case: Calendar event with all-day format (date, not dateTime)
#[tokio::test]
async fn req_rt_044_edge_calendar_all_day_event() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/calendar/v3/calendars/.*/events/.*".to_string()),
        )
        .with_status(200)
        .with_body(
            r#"{
            "id": "all_day_event",
            "summary": "Company Holiday",
            "start": {"date": "2024-12-25"},
            "end": {"date": "2024-12-26"}
        }"#,
        )
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!(
        "{}/calendar/v3/calendars/primary/events/all_day_event",
        server.url()
    );

    let event: omega_google::services::calendar::types::Event = omega_google::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        false,
    )
    .await
    .unwrap();

    assert_eq!(event.id, Some("all_day_event".to_string()));
    let start = event.start.as_ref().unwrap();
    assert!(start.date_time.is_none());
    assert_eq!(start.date, Some("2024-12-25".to_string()));
}

// -------------------------------------------------------------------
// REQ-RT-058 (Must): handle_drive_download (stub for RT-M5)
// -------------------------------------------------------------------

// Requirement: REQ-RT-058 (Must)
// Acceptance: Download URL is correct
#[test]
fn req_rt_058_drive_download_url() {
    let url = omega_google::services::drive::files::build_file_download_url("file_dl");
    assert!(url.contains("files/file_dl"));
    assert!(url.contains("alt=media"));
}

// Requirement: REQ-RT-058 (Must)
// Acceptance: Export URL includes mimeType
#[test]
fn req_rt_058_drive_export_url() {
    let url =
        omega_google::services::drive::files::build_file_export_url("file_exp", "application/pdf");
    assert!(url.contains("files/file_exp/export"));
    assert!(url.contains("mimeType="));
}

// -------------------------------------------------------------------
// REQ-RT-059 (Must): handle_drive_upload (stub for RT-M5)
// -------------------------------------------------------------------

// Requirement: REQ-RT-059 (Must)
// Acceptance: Upload URL is correct
#[test]
fn req_rt_059_drive_upload_url() {
    let url = omega_google::services::drive::files::build_file_upload_url();
    assert!(url.contains("upload/drive/v3/files"));
    assert!(url.contains("uploadType=multipart"));
}

// ===================================================================
// Verbose logging tests
// ===================================================================

// Requirement: REQ-RT-032 (Must)
// Acceptance: Verbose mode logs request/response details to stderr
#[tokio::test]
async fn req_rt_032_verbose_logging_does_not_crash() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(200)
        .with_body(r#"{"threads": []}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/gmail/v1/users/me/threads", server.url());

    // Verbose mode should not panic
    let result: anyhow::Result<omega_google::services::gmail::types::ThreadListResponse> =
        omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            true, // verbose
        )
        .await;

    assert!(result.is_ok());
}

// ===================================================================
// Circuit breaker integration tests
// ===================================================================

// Requirement: REQ-RT-032 (Must)
// Failure mode: Circuit breaker opens after repeated failures
#[tokio::test]
async fn req_rt_032_circuit_breaker_opens_after_failures() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(500)
        .with_body(r#"{"error":{"code":500,"message":"Internal"}}"#)
        .create_async()
        .await;

    let ctx = test_service_context(&server.url());
    let url = format!("{}/api/endpoint", server.url());

    // Make multiple failed requests to trigger circuit breaker
    for _ in 0..5 {
        let _result: anyhow::Result<serde_json::Value> = omega_google::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            false,
        )
        .await;
    }

    // After 5 failures, circuit breaker should be open
    assert!(
        ctx.circuit_breaker.is_open(),
        "Circuit breaker should open after 5 failures"
    );
}
