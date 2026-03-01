//! Chat message URL and body builders.

use super::CHAT_BASE_URL;

/// Build URL for listing messages in a space.
/// REQ-CHAT-004
pub fn build_messages_list_url(
    space: &str,
    max: Option<u32>,
    page_token: Option<&str>,
    order: Option<&str>,
    thread: Option<&str>,
) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded_space = utf8_percent_encode(space, NON_ALPHANUMERIC).to_string();
    let base = format!("{}/spaces/{}/messages", CHAT_BASE_URL, encoded_space);
    let mut params = Vec::new();
    if let Some(m) = max {
        params.push(format!("pageSize={}", m));
    }
    if let Some(token) = page_token {
        params.push(format!("pageToken={}", token));
    }
    if let Some(ord) = order {
        params.push(format!("orderBy={}", ord));
    }
    if let Some(t) = thread {
        let encoded_filter = url::form_urlencoded::byte_serialize(
            format!("thread.name = \"{}\"", t).as_bytes(),
        )
        .collect::<String>();
        params.push(format!("filter={}", encoded_filter));
    }
    if params.is_empty() {
        base
    } else {
        format!("{}?{}", base, params.join("&"))
    }
}

/// Build URL for sending a message to a space.
/// REQ-CHAT-005
pub fn build_message_send_url(space: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded_space = utf8_percent_encode(space, NON_ALPHANUMERIC).to_string();
    format!("{}/spaces/{}/messages", CHAT_BASE_URL, encoded_space)
}

/// Build request body for sending a message.
/// REQ-CHAT-005
pub fn build_message_send_body(text: &str, thread: Option<&str>) -> serde_json::Value {
    let mut body = serde_json::json!({
        "text": text,
    });
    if let Some(t) = thread {
        body["thread"] = serde_json::json!({
            "name": t,
        });
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CHAT-004 (Must): Messages list URL
    // ---------------------------------------------------------------

    // Requirement: REQ-CHAT-004 (Must)
    // Acceptance: Messages list URL with space name
    #[test]
    fn req_chat_004_messages_list_url_default() {
        let url = build_messages_list_url("AAAA", None, None, None, None);
        assert!(url.contains("/spaces/AAAA/messages"));
    }

    // Requirement: REQ-CHAT-004 (Must)
    // Acceptance: Messages list URL with max results
    #[test]
    fn req_chat_004_messages_list_url_max() {
        let url = build_messages_list_url("AAAA", Some(25), None, None, None);
        assert!(url.contains("pageSize=25"));
    }

    // Requirement: REQ-CHAT-004 (Must)
    // Acceptance: Messages list URL with page token
    #[test]
    fn req_chat_004_messages_list_url_page_token() {
        let url = build_messages_list_url("AAAA", None, Some("token_abc"), None, None);
        assert!(url.contains("pageToken=token_abc"));
    }

    // Requirement: REQ-CHAT-004 (Must)
    // Acceptance: Messages list URL with order
    #[test]
    fn req_chat_004_messages_list_url_order() {
        let url = build_messages_list_url("AAAA", None, None, Some("createTime desc"), None);
        assert!(url.contains("orderBy=createTime desc"));
    }

    // Requirement: REQ-CHAT-004 (Must)
    // Acceptance: Messages list URL with thread filter
    #[test]
    fn req_chat_004_messages_list_url_thread() {
        let url = build_messages_list_url("AAAA", None, None, None, Some("spaces/AAAA/threads/t1"));
        assert!(url.contains("filter="));
        assert!(url.contains("thread"));
    }

    // Requirement: REQ-CHAT-004 (Must)
    // Acceptance: Messages list URL with all parameters
    #[test]
    fn req_chat_004_messages_list_url_all_params() {
        let url = build_messages_list_url("AAAA", Some(10), Some("p2"), Some("createTime"), Some("spaces/AAAA/threads/t1"));
        assert!(url.contains("pageSize=10"));
        assert!(url.contains("pageToken=p2"));
        assert!(url.contains("orderBy=createTime"));
        assert!(url.contains("filter="));
    }

    // ---------------------------------------------------------------
    // REQ-CHAT-005 (Must): Message send URL and body
    // ---------------------------------------------------------------

    // Requirement: REQ-CHAT-005 (Must)
    // Acceptance: Message send URL
    #[test]
    fn req_chat_005_message_send_url() {
        let url = build_message_send_url("AAAA");
        assert!(url.contains("/spaces/AAAA/messages"));
    }

    // Requirement: REQ-CHAT-005 (Must)
    // Acceptance: Message send body with text only
    #[test]
    fn req_chat_005_message_send_body_text_only() {
        let body = build_message_send_body("Hello!", None);
        assert_eq!(body["text"], "Hello!");
        assert!(body.get("thread").is_none());
    }

    // Requirement: REQ-CHAT-005 (Must)
    // Acceptance: Message send body with thread
    #[test]
    fn req_chat_005_message_send_body_with_thread() {
        let body = build_message_send_body("Reply!", Some("spaces/AAAA/threads/t1"));
        assert_eq!(body["text"], "Reply!");
        assert_eq!(body["thread"]["name"], "spaces/AAAA/threads/t1");
    }

    // Requirement: REQ-CHAT-005 (Must)
    // Edge case: Message send URL with special characters in space name
    #[test]
    fn req_chat_005_message_send_url_special_chars() {
        let url = build_message_send_url("space with spaces");
        assert!(url.contains("/spaces/"));
        assert!(url.contains("/messages"));
    }
}
