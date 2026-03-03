//! Chat DM (Direct Message) URL and body builders.

use super::CHAT_BASE_URL;

/// Build URL for finding or creating a DM space.
/// REQ-CHAT-007
pub fn build_dm_space_url() -> String {
    format!("{}/spaces:findDirectMessage", CHAT_BASE_URL)
}

/// Build request body for finding a DM space by email/user.
/// REQ-CHAT-007
pub fn build_dm_space_body(email: &str) -> serde_json::Value {
    serde_json::json!({
        "name": email,
    })
}

/// Build URL for sending a DM message to a space.
/// REQ-CHAT-008
pub fn build_dm_send_url(space_name: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded_space = utf8_percent_encode(space_name, NON_ALPHANUMERIC).to_string();
    format!("{}/spaces/{}/messages", CHAT_BASE_URL, encoded_space)
}

/// Build request body for sending a DM.
/// REQ-CHAT-008
pub fn build_dm_send_body(text: &str, thread: Option<&str>) -> serde_json::Value {
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
    // REQ-CHAT-007 (Must): DM space URL and body
    // ---------------------------------------------------------------

    // Requirement: REQ-CHAT-007 (Must)
    // Acceptance: DM space URL
    #[test]
    fn req_chat_007_dm_space_url() {
        let url = build_dm_space_url();
        assert_eq!(
            url,
            "https://chat.googleapis.com/v1/spaces:findDirectMessage"
        );
    }

    // Requirement: REQ-CHAT-007 (Must)
    // Acceptance: DM space body with email
    #[test]
    fn req_chat_007_dm_space_body() {
        let body = build_dm_space_body("user@example.com");
        assert_eq!(body["name"], "user@example.com");
    }

    // Requirement: REQ-CHAT-007 (Must)
    // Edge case: DM space body with user resource name
    #[test]
    fn req_chat_007_dm_space_body_user_name() {
        let body = build_dm_space_body("users/user123");
        assert_eq!(body["name"], "users/user123");
    }

    // ---------------------------------------------------------------
    // REQ-CHAT-008 (Must): DM send URL and body
    // ---------------------------------------------------------------

    // Requirement: REQ-CHAT-008 (Must)
    // Acceptance: DM send URL
    #[test]
    fn req_chat_008_dm_send_url() {
        let url = build_dm_send_url("spaces/AAAA");
        assert!(url.contains("/spaces/"));
        assert!(url.contains("/messages"));
    }

    // Requirement: REQ-CHAT-008 (Must)
    // Acceptance: DM send body with text only
    #[test]
    fn req_chat_008_dm_send_body_text_only() {
        let body = build_dm_send_body("Hello DM!", None);
        assert_eq!(body["text"], "Hello DM!");
        assert!(body.get("thread").is_none());
    }

    // Requirement: REQ-CHAT-008 (Must)
    // Acceptance: DM send body with thread
    #[test]
    fn req_chat_008_dm_send_body_with_thread() {
        let body = build_dm_send_body("Reply in DM", Some("spaces/AAAA/threads/t1"));
        assert_eq!(body["text"], "Reply in DM");
        assert_eq!(body["thread"]["name"], "spaces/AAAA/threads/t1");
    }

    // Requirement: REQ-CHAT-008 (Must)
    // Edge case: DM send body with empty text
    #[test]
    fn req_chat_008_dm_send_body_empty_text() {
        let body = build_dm_send_body("", None);
        assert_eq!(body["text"], "");
    }
}
