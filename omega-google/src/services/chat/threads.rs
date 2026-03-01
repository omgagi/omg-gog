//! Chat thread URL builders.

use super::CHAT_BASE_URL;

/// Build URL for listing threads in a space.
/// REQ-CHAT-006
pub fn build_threads_list_url(
    space: &str,
    max: Option<u32>,
    page_token: Option<&str>,
) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded_space = utf8_percent_encode(space, NON_ALPHANUMERIC).to_string();
    let base = format!("{}/spaces/{}/messages", CHAT_BASE_URL, encoded_space);
    // Chat API uses messages endpoint with showDeleted=false, filter by thread
    // For listing threads, we use the messages endpoint grouped by thread
    let mut params = Vec::new();
    if let Some(m) = max {
        params.push(format!("pageSize={}", m));
    }
    if let Some(token) = page_token {
        params.push(format!("pageToken={}", token));
    }
    if params.is_empty() {
        base
    } else {
        format!("{}?{}", base, params.join("&"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CHAT-006 (Must): Threads list URL
    // ---------------------------------------------------------------

    // Requirement: REQ-CHAT-006 (Must)
    // Acceptance: Threads list URL with space name
    #[test]
    fn req_chat_006_threads_list_url_default() {
        let url = build_threads_list_url("AAAA", None, None);
        assert!(url.contains("/spaces/AAAA/messages"));
    }

    // Requirement: REQ-CHAT-006 (Must)
    // Acceptance: Threads list URL with max results
    #[test]
    fn req_chat_006_threads_list_url_max() {
        let url = build_threads_list_url("AAAA", Some(10), None);
        assert!(url.contains("pageSize=10"));
    }

    // Requirement: REQ-CHAT-006 (Must)
    // Acceptance: Threads list URL with page token
    #[test]
    fn req_chat_006_threads_list_url_page_token() {
        let url = build_threads_list_url("AAAA", None, Some("token_xyz"));
        assert!(url.contains("pageToken=token_xyz"));
    }

    // Requirement: REQ-CHAT-006 (Must)
    // Acceptance: Threads list URL with both parameters
    #[test]
    fn req_chat_006_threads_list_url_all_params() {
        let url = build_threads_list_url("AAAA", Some(5), Some("p2"));
        assert!(url.contains("pageSize=5"));
        assert!(url.contains("pageToken=p2"));
    }

    // Requirement: REQ-CHAT-006 (Must)
    // Edge case: Threads list URL with special characters
    #[test]
    fn req_chat_006_threads_list_url_special_chars() {
        let url = build_threads_list_url("space@special", None, None);
        assert!(url.contains("/spaces/"));
        assert!(url.contains("/messages"));
    }
}
