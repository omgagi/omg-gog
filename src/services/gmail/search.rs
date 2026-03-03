//! Gmail thread and message search.

use super::types::*;

/// Build the URL for Gmail thread search.
pub fn build_thread_search_url(
    query: &str,
    max_results: Option<u32>,
    page_token: Option<&str>,
) -> String {
    let base = format!("{}/users/me/threads", GMAIL_BASE_URL);
    let max = max_results.unwrap_or(20);
    let mut params = vec![format!("maxResults={}", max)];
    if !query.is_empty() {
        params.push(format!(
            "q={}",
            url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>()
        ));
    }
    if let Some(token) = page_token {
        params.push(format!(
            "pageToken={}",
            url::form_urlencoded::byte_serialize(token.as_bytes()).collect::<String>()
        ));
    }
    format!("{}?{}", base, params.join("&"))
}

/// Build the URL for Gmail message search.
/// When `include_body` is true, adds `format=full` to retrieve the full message body.
pub fn build_message_search_url(
    query: &str,
    max_results: Option<u32>,
    page_token: Option<&str>,
    include_body: bool,
) -> String {
    let base = format!("{}/users/me/messages", GMAIL_BASE_URL);
    let max = max_results.unwrap_or(20);
    let mut params = vec![format!("maxResults={}", max)];
    if !query.is_empty() {
        params.push(format!(
            "q={}",
            url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>()
        ));
    }
    if let Some(token) = page_token {
        params.push(format!(
            "pageToken={}",
            url::form_urlencoded::byte_serialize(token.as_bytes()).collect::<String>()
        ));
    }
    if include_body {
        params.push("format=full".to_string());
    }
    format!("{}?{}", base, params.join("&"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-GMAIL-001 (Must): Thread search URL building
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-001 (Must)
    // Acceptance: Search builds correct URL with query
    #[test]
    fn req_gmail_001_search_builds_url_with_query() {
        let url = build_thread_search_url("from:user@example.com", None, None);
        assert!(url.contains("users/me/threads"));
        assert!(url.contains("q="));
        assert!(url.contains("from%3Auser%40example.com") || url.contains("from:user@example.com"));
    }

    // Requirement: REQ-GMAIL-001 (Must)
    // Acceptance: Search includes max results when specified
    #[test]
    fn req_gmail_001_search_with_max_results() {
        let url = build_thread_search_url("test", Some(50), None);
        assert!(url.contains("maxResults=50"));
    }

    // Requirement: REQ-GMAIL-001 (Must)
    // Acceptance: Search includes page token when specified
    #[test]
    fn req_gmail_001_search_with_page_token() {
        let url = build_thread_search_url("test", None, Some("token123"));
        assert!(url.contains("pageToken=token123"));
    }

    // Requirement: REQ-GMAIL-001 (Must)
    // Acceptance: Default max results is 20 when not specified
    #[test]
    fn req_gmail_001_search_default_max() {
        let url = build_thread_search_url("test", None, None);
        assert!(url.contains("maxResults=20"));
    }

    // Requirement: REQ-GMAIL-001 (Must)
    // Edge case: Empty query
    #[test]
    fn req_gmail_001_search_empty_query() {
        let url = build_thread_search_url("", None, None);
        assert!(url.contains("users/me/threads"));
    }

    // Requirement: REQ-GMAIL-001 (Must)
    // Edge case: Query with special characters
    #[test]
    fn req_gmail_001_search_special_chars() {
        let url = build_thread_search_url("subject:\"hello world\" has:attachment", None, None);
        assert!(url.contains("users/me/threads"));
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-002 (Must): Message search URL building
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-002 (Must)
    // Acceptance: Message search builds correct URL
    #[test]
    fn req_gmail_002_message_search_url() {
        let url = build_message_search_url("test", None, None, false);
        assert!(url.contains("users/me/messages"));
    }

    // Requirement: REQ-GMAIL-002 (Must)
    // Acceptance: Message search supports include-body (adds format=full)
    #[test]
    fn req_gmail_002_message_search_include_body() {
        let url = build_message_search_url("test", None, None, true);
        assert!(url.contains("users/me/messages"));
        assert!(url.contains("format=full"));
    }

    // Requirement: REQ-GMAIL-002 (Must)
    // Acceptance: Message search without include-body does not add format param
    #[test]
    fn req_gmail_002_message_search_no_include_body() {
        let url = build_message_search_url("test", None, None, false);
        assert!(!url.contains("format=full"));
    }
}
