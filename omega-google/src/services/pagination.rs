// Generic pagination loop for Google API paginated endpoints.
//
// Google APIs use a `nextPageToken` pattern: each page response optionally
// contains a `nextPageToken` field. To fetch the next page, include
// `pageToken=<token>` in the query parameters.

use serde::de::DeserializeOwned;

use crate::http::api::api_get;
use crate::http::circuit_breaker::CircuitBreaker;
use crate::http::RetryConfig;
use crate::services::common::PaginationParams;

/// Maximum number of pages to prevent infinite loops.
pub const MAX_PAGES: usize = 1000;

/// Trait for types that carry a nextPageToken.
pub trait HasNextPageToken {
    fn next_page_token(&self) -> Option<&str>;
}

/// Paginate through all pages of a list endpoint.
///
/// - `url_fn` builds the URL for each page given an optional page token.
/// - `extract_fn` extracts items and next_page_token from the response JSON.
/// - When `params.all_pages` is true, fetches all pages.
/// - When `params.all_pages` is false, fetches a single page.
/// - Returns the accumulated items and an optional next_page_token (for hint).
pub async fn paginate<T>(
    client: &reqwest::Client,
    breaker: &CircuitBreaker,
    retry_config: &RetryConfig,
    verbose: bool,
    params: &PaginationParams,
    url_fn: impl Fn(Option<&str>) -> String,
    extract_fn: impl Fn(serde_json::Value) -> anyhow::Result<(Vec<T>, Option<String>)>,
) -> anyhow::Result<(Vec<T>, Option<String>)> {
    let mut all_items = Vec::new();
    let mut page_token: Option<String> = params.page_token.clone();
    let mut page_num = 0usize;
    let mut last_next_token: Option<String> = None;

    loop {
        page_num += 1;
        if page_num > MAX_PAGES {
            eprintln!("Warning: reached maximum page limit ({}), stopping pagination", MAX_PAGES);
            break;
        }

        if page_num > 1 && verbose {
            eprintln!("Fetching page {}...", page_num);
        }

        let url = url_fn(page_token.as_deref());
        let response: serde_json::Value =
            api_get(client, &url, breaker, retry_config, verbose).await?;
        let (items, next_token) = extract_fn(response)?;

        all_items.extend(items);
        last_next_token = next_token.clone();
        page_token = next_token;

        // Stop if no more pages or not fetching all
        if page_token.is_none() {
            break;
        }
        if !params.all_pages {
            // Single-page mode: return the hint token but don't fetch more
            break;
        }
    }

    // Return items and the next page token (for hint printing)
    let hint_token = if params.all_pages { None } else { last_next_token };
    Ok((all_items, hint_token))
}

/// Check if results are empty and fail_empty is set.
/// Returns EMPTY_RESULTS exit code (3) as an error if so.
pub fn check_fail_empty<T>(items: &[T], fail_empty: bool) -> anyhow::Result<()> {
    if fail_empty && items.is_empty() {
        anyhow::bail!("empty results");
    }
    Ok(())
}

/// Single-page fetch: returns the deserialized response directly.
/// Used when the caller needs the full response type (not just extracted items).
pub async fn fetch_page<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    breaker: &CircuitBreaker,
    retry_config: &RetryConfig,
    verbose: bool,
) -> anyhow::Result<T> {
    api_get(client, url, breaker, retry_config, verbose).await
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // Standard extract function for test JSON responses.
    fn extract_items(value: serde_json::Value) -> anyhow::Result<(Vec<String>, Option<String>)> {
        let items = value
            .get("items")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let next_token = value
            .get("nextPageToken")
            .and_then(|v| v.as_str())
            .map(String::from);
        Ok((items, next_token))
    }

    // ===================================================================
    // REQ-RT-023 (Must): Generic pagination loop for nextPageToken pattern
    // ===================================================================

    // Requirement: REQ-RT-023 (Must)
    // Acceptance: Single page response with no nextPageToken returns items
    #[tokio::test]
    async fn req_rt_023_single_page_no_next_token() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/list")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"items":["a","b","c"]}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams {
            all_pages: true,
            ..Default::default()
        };

        let (items, next_token) = paginate(
            &client,
            &breaker,
            &config,
            false,
            &params,
            |_pt| format!("{}/api/list", base_url),
            extract_items,
        )
        .await
        .unwrap();

        assert_eq!(items, vec!["a", "b", "c"]);
        assert!(next_token.is_none(), "No hint when all pages fetched");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-023 (Must)
    // Acceptance: Multi-page response with --all fetches all pages
    #[tokio::test]
    async fn req_rt_023_multi_page_all_pages_fetches_all() {
        let mut server = mockito::Server::new_async().await;

        // Page 1: has nextPageToken
        let mock1 = server
            .mock("GET", "/api/list")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"items":["a","b"],"nextPageToken":"page2"}"#)
            .create_async()
            .await;

        // Page 2: has nextPageToken
        let mock2 = server
            .mock("GET", "/api/list?pageToken=page2")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"items":["c","d"],"nextPageToken":"page3"}"#)
            .create_async()
            .await;

        // Page 3: no nextPageToken (last page)
        let mock3 = server
            .mock("GET", "/api/list?pageToken=page3")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"items":["e"]}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams {
            all_pages: true,
            ..Default::default()
        };

        let (items, next_token) = paginate(
            &client,
            &breaker,
            &config,
            false,
            &params,
            |pt| {
                if let Some(token) = pt {
                    format!("{}/api/list?pageToken={}", base_url, token)
                } else {
                    format!("{}/api/list", base_url)
                }
            },
            extract_items,
        )
        .await
        .unwrap();

        assert_eq!(items, vec!["a", "b", "c", "d", "e"]);
        assert!(next_token.is_none(), "No hint token when all pages fetched");
        mock1.assert_async().await;
        mock2.assert_async().await;
        mock3.assert_async().await;
    }

    // Requirement: REQ-RT-023 (Must)
    // Acceptance: Progress hint on stderr when fetching page N > 1
    // (Tested by verifying multi-page works with verbose=true without crashing)
    #[tokio::test]
    async fn req_rt_023_multi_page_verbose_progress() {
        let mut server = mockito::Server::new_async().await;

        let _mock1 = server
            .mock("GET", "/api/list")
            .with_status(200)
            .with_body(r#"{"items":["a"],"nextPageToken":"p2"}"#)
            .create_async()
            .await;

        let _mock2 = server
            .mock("GET", "/api/list?pageToken=p2")
            .with_status(200)
            .with_body(r#"{"items":["b"]}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams {
            all_pages: true,
            ..Default::default()
        };

        let (items, _) = paginate(
            &client,
            &breaker,
            &config,
            true, // verbose -- should print progress
            &params,
            |pt| {
                if let Some(token) = pt {
                    format!("{}/api/list?pageToken={}", base_url, token)
                } else {
                    format!("{}/api/list", base_url)
                }
            },
            extract_items,
        )
        .await
        .unwrap();

        assert_eq!(items.len(), 2);
    }

    // ===================================================================
    // REQ-RT-024 (Must): Single-page mode (default, no --all)
    // ===================================================================

    // Requirement: REQ-RT-024 (Must)
    // Acceptance: Without --all, returns one page and hint token
    #[tokio::test]
    async fn req_rt_024_single_page_mode_returns_hint() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/api/list")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"items":["a","b"],"nextPageToken":"page2"}"#)
            .create_async()
            .await;

        // Second page should NOT be fetched
        let mock2 = server
            .mock("GET", "/api/list?pageToken=page2")
            .expect(0)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams {
            all_pages: false, // Single-page mode
            ..Default::default()
        };

        let (items, hint_token) = paginate(
            &client,
            &breaker,
            &config,
            false,
            &params,
            |pt| {
                if let Some(token) = pt {
                    format!("{}/api/list?pageToken={}", base_url, token)
                } else {
                    format!("{}/api/list", base_url)
                }
            },
            extract_items,
        )
        .await
        .unwrap();

        assert_eq!(items, vec!["a", "b"]);
        assert_eq!(
            hint_token,
            Some("page2".to_string()),
            "Should return nextPageToken as hint"
        );
        mock.assert_async().await;
        mock2.assert_async().await; // Verify page 2 was NOT called
    }

    // Requirement: REQ-RT-024 (Must)
    // Acceptance: --page TOKEN continues from a specific page
    #[tokio::test]
    async fn req_rt_024_page_token_starts_from_specific_page() {
        let mut server = mockito::Server::new_async().await;

        // Should use the provided page token in the first request
        let mock = server
            .mock("GET", "/api/list?pageToken=TOKEN123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"items":["x","y"],"nextPageToken":"TOKEN456"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams {
            page_token: Some("TOKEN123".to_string()),
            all_pages: false,
            ..Default::default()
        };

        let (items, hint_token) = paginate(
            &client,
            &breaker,
            &config,
            false,
            &params,
            |pt| {
                if let Some(token) = pt {
                    format!("{}/api/list?pageToken={}", base_url, token)
                } else {
                    format!("{}/api/list", base_url)
                }
            },
            extract_items,
        )
        .await
        .unwrap();

        assert_eq!(items, vec!["x", "y"]);
        assert_eq!(hint_token, Some("TOKEN456".to_string()));
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-024 (Must)
    // Acceptance: Single page mode with no nextPageToken returns None hint
    #[tokio::test]
    async fn req_rt_024_single_page_no_more_pages_no_hint() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/api/list")
            .with_status(200)
            .with_body(r#"{"items":["a"]}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams {
            all_pages: false,
            ..Default::default()
        };

        let (items, hint_token) = paginate(
            &client,
            &breaker,
            &config,
            false,
            &params,
            |_pt| format!("{}/api/list", base_url),
            extract_items,
        )
        .await
        .unwrap();

        assert_eq!(items, vec!["a"]);
        assert!(hint_token.is_none(), "No hint when no more pages");
        mock.assert_async().await;
    }

    // ===================================================================
    // REQ-RT-025 (Must): --fail-empty exits with code 3 on empty results
    // ===================================================================

    // Requirement: REQ-RT-025 (Must)
    // Acceptance: check_fail_empty returns error when items are empty and fail_empty is true
    #[test]
    fn req_rt_025_check_fail_empty_returns_error_on_empty() {
        let items: Vec<String> = vec![];
        let result = check_fail_empty(&items, true);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("empty"),
            "Error should mention empty: {}",
            err_msg
        );
    }

    // Requirement: REQ-RT-025 (Must)
    // Acceptance: check_fail_empty returns Ok when items are empty but fail_empty is false
    #[test]
    fn req_rt_025_check_fail_empty_ok_when_not_set() {
        let items: Vec<String> = vec![];
        let result = check_fail_empty(&items, false);
        assert!(result.is_ok());
    }

    // Requirement: REQ-RT-025 (Must)
    // Acceptance: check_fail_empty returns Ok when items are non-empty and fail_empty is true
    #[test]
    fn req_rt_025_check_fail_empty_ok_when_items_present() {
        let items = vec!["a".to_string()];
        let result = check_fail_empty(&items, true);
        assert!(result.is_ok());
    }

    // Requirement: REQ-RT-025 (Must)
    // Acceptance: Applies after pagination completes (all pages fetched)
    #[tokio::test]
    async fn req_rt_025_fail_empty_after_all_pages_empty() {
        let mut server = mockito::Server::new_async().await;

        // Page 1: empty items but has next page
        let _mock1 = server
            .mock("GET", "/api/empty-list")
            .with_status(200)
            .with_body(r#"{"items":[],"nextPageToken":"p2"}"#)
            .create_async()
            .await;

        // Page 2: also empty, no more pages
        let _mock2 = server
            .mock("GET", "/api/empty-list?pageToken=p2")
            .with_status(200)
            .with_body(r#"{"items":[]}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams {
            all_pages: true,
            fail_empty: true,
            ..Default::default()
        };

        let (items, _) = paginate(
            &client,
            &breaker,
            &config,
            false,
            &params,
            |pt| {
                if let Some(token) = pt {
                    format!("{}/api/empty-list?pageToken={}", base_url, token)
                } else {
                    format!("{}/api/empty-list", base_url)
                }
            },
            extract_items,
        )
        .await
        .unwrap();

        // After pagination, check fail_empty
        let result = check_fail_empty(&items, params.fail_empty);
        assert!(result.is_err(), "Should fail on empty results with --fail-empty");
    }

    // Requirement: REQ-RT-025 (Must)
    // Edge case: Single item prevents fail-empty error
    #[tokio::test]
    async fn req_rt_025_single_item_prevents_fail_empty() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("GET", "/api/list")
            .with_status(200)
            .with_body(r#"{"items":["only-one"]}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams {
            all_pages: false,
            fail_empty: true,
            ..Default::default()
        };

        let (items, _) = paginate(
            &client,
            &breaker,
            &config,
            false,
            &params,
            |_pt| format!("{}/api/list", base_url),
            extract_items,
        )
        .await
        .unwrap();

        let result = check_fail_empty(&items, params.fail_empty);
        assert!(result.is_ok(), "Should not fail when items exist");
    }

    // ===================================================================
    // Failure modes (from architecture)
    // ===================================================================

    // Requirement: REQ-RT-023 (Must)
    // Failure mode: Infinite pagination loop guard at MAX_PAGES
    #[test]
    fn req_rt_023_max_pages_guard_constant() {
        assert_eq!(MAX_PAGES, 1000, "Max pages should be 1000");
    }

    // Requirement: REQ-RT-023 (Must)
    // Failure mode: Error on any page is propagated immediately (fail-fast)
    #[tokio::test]
    async fn req_rt_023_error_on_page_propagates_immediately() {
        let mut server = mockito::Server::new_async().await;

        // Page 1: success
        let _mock1 = server
            .mock("GET", "/api/list")
            .with_status(200)
            .with_body(r#"{"items":["a"],"nextPageToken":"p2"}"#)
            .create_async()
            .await;

        // Page 2: server error
        let _mock2 = server
            .mock("GET", "/api/list?pageToken=p2")
            .with_status(500)
            .with_body(r#"{"error":{"code":500,"message":"Internal error"}}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams {
            all_pages: true,
            ..Default::default()
        };

        let result = paginate(
            &client,
            &breaker,
            &config,
            false,
            &params,
            |pt| {
                if let Some(token) = pt {
                    format!("{}/api/list?pageToken={}", base_url, token)
                } else {
                    format!("{}/api/list", base_url)
                }
            },
            extract_items,
        )
        .await;

        assert!(result.is_err(), "Error on page 2 should propagate immediately");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("500"), "Error should contain status 500: {}", err_msg);
    }

    // Requirement: REQ-RT-023 (Must)
    // Failure mode: Empty items array with nextPageToken is valid (continue to next page)
    #[tokio::test]
    async fn req_rt_023_empty_items_with_next_token_continues() {
        let mut server = mockito::Server::new_async().await;

        // Page 1: empty items, but has next page
        let _mock1 = server
            .mock("GET", "/api/list")
            .with_status(200)
            .with_body(r#"{"items":[],"nextPageToken":"p2"}"#)
            .create_async()
            .await;

        // Page 2: has items, no more pages
        let _mock2 = server
            .mock("GET", "/api/list?pageToken=p2")
            .with_status(200)
            .with_body(r#"{"items":["found-it"]}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams {
            all_pages: true,
            ..Default::default()
        };

        let (items, _) = paginate(
            &client,
            &breaker,
            &config,
            false,
            &params,
            |pt| {
                if let Some(token) = pt {
                    format!("{}/api/list?pageToken={}", base_url, token)
                } else {
                    format!("{}/api/list", base_url)
                }
            },
            extract_items,
        )
        .await
        .unwrap();

        assert_eq!(items, vec!["found-it"], "Should accumulate items from all pages");
    }

    // ===================================================================
    // Edge cases
    // ===================================================================

    // Requirement: REQ-RT-023 (Must)
    // Edge case: Response missing items field entirely
    #[tokio::test]
    async fn req_rt_023_edge_missing_items_field() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("GET", "/api/list")
            .with_status(200)
            .with_body(r#"{"someOtherField":"value"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams::default();

        let (items, _) = paginate(
            &client,
            &breaker,
            &config,
            false,
            &params,
            |_pt| format!("{}/api/list", base_url),
            extract_items,
        )
        .await
        .unwrap();

        // extract_items returns empty vec when "items" field is missing
        assert!(items.is_empty());
    }

    // Requirement: REQ-RT-023 (Must)
    // Edge case: extract_fn returns error
    #[tokio::test]
    async fn req_rt_023_edge_extract_fn_error() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("GET", "/api/list")
            .with_status(200)
            .with_body(r#"{"items":["a"]}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams::default();

        let result = paginate::<String>(
            &client,
            &breaker,
            &config,
            false,
            &params,
            |_pt| format!("{}/api/list", base_url),
            |_value| anyhow::bail!("extract function failed"),
        )
        .await;

        assert!(result.is_err(), "Extract function error should propagate");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("extract function failed"));
    }

    // Requirement: REQ-RT-024 (Must)
    // Edge case: page_token with special characters
    #[tokio::test]
    async fn req_rt_024_edge_page_token_special_chars() {
        let mut server = mockito::Server::new_async().await;

        // Token with characters that might need encoding
        let token = "abc+def/ghi=jkl";
        let mock = server
            .mock("GET", format!("/api/list?pageToken={}", token).as_str())
            .with_status(200)
            .with_body(r#"{"items":["result"]}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams {
            page_token: Some(token.to_string()),
            all_pages: false,
            ..Default::default()
        };

        let (items, _) = paginate(
            &client,
            &breaker,
            &config,
            false,
            &params,
            |pt| {
                if let Some(t) = pt {
                    format!("{}/api/list?pageToken={}", base_url, t)
                } else {
                    format!("{}/api/list", base_url)
                }
            },
            extract_items,
        )
        .await
        .unwrap();

        assert_eq!(items, vec!["result"]);
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-025 (Must)
    // Edge case: check_fail_empty with different types
    #[test]
    fn req_rt_025_edge_check_fail_empty_with_numbers() {
        let items: Vec<i32> = vec![];
        assert!(check_fail_empty(&items, true).is_err());

        let items = vec![1, 2, 3];
        assert!(check_fail_empty(&items, true).is_ok());
    }

    // Requirement: REQ-RT-023 (Must)
    // Edge case: Pagination with 401 on first page
    #[tokio::test]
    async fn req_rt_023_edge_auth_error_on_first_page() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("GET", "/api/list")
            .with_status(401)
            .with_body(r#"{"error":{"code":401,"message":"Token expired"}}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams::default();

        let result = paginate::<String>(
            &client,
            &breaker,
            &config,
            false,
            &params,
            |_pt| format!("{}/api/list", base_url),
            extract_items,
        )
        .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("401"), "Should propagate auth error");
    }

    // Requirement: REQ-RT-023 (Must)
    // Acceptance: Each page request includes maxResults from params
    // (This tests url_fn receives the right page token, max_results is the caller's job)
    #[tokio::test]
    async fn req_rt_023_url_fn_receives_correct_page_tokens() {
        let mut server = mockito::Server::new_async().await;

        let _mock1 = server
            .mock("GET", "/api/list?maxResults=10")
            .with_status(200)
            .with_body(r#"{"items":["a"],"nextPageToken":"tok2"}"#)
            .create_async()
            .await;

        let _mock2 = server
            .mock("GET", "/api/list?maxResults=10&pageToken=tok2")
            .with_status(200)
            .with_body(r#"{"items":["b"]}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let base_url = server.url();
        let params = PaginationParams {
            max_results: Some(10),
            all_pages: true,
            ..Default::default()
        };

        let max = params.max_results.unwrap();
        let (items, _) = paginate(
            &client,
            &breaker,
            &config,
            false,
            &params,
            move |pt| {
                if let Some(token) = pt {
                    format!("{}/api/list?maxResults={}&pageToken={}", base_url, max, token)
                } else {
                    format!("{}/api/list?maxResults={}", base_url, max)
                }
            },
            extract_items,
        )
        .await
        .unwrap();

        assert_eq!(items, vec!["a", "b"]);
    }

    // Requirement: REQ-RT-023 (Must)
    // fetch_page deserializes typed response
    #[tokio::test]
    async fn req_rt_023_fetch_page_deserializes_typed() {
        use serde::Deserialize;

        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(rename_all = "camelCase")]
        struct ListResp {
            items: Vec<String>,
            #[serde(default)]
            next_page_token: Option<String>,
        }

        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/typed")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"items":["typed"],"nextPageToken":"next"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let breaker = fresh_breaker();
        let config = no_retry_config();
        let url = format!("{}/api/typed", server.url());

        let resp: ListResp = fetch_page(&client, &url, &breaker, &config, false).await.unwrap();
        assert_eq!(resp.items, vec!["typed"]);
        assert_eq!(resp.next_page_token, Some("next".to_string()));
        mock.assert_async().await;
    }
}
