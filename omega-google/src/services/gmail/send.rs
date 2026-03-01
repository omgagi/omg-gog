//! Email sending via Gmail API.

const GMAIL_BASE_URL: &str = "https://gmail.googleapis.com/gmail/v1";

/// Build the URL for sending a message.
pub fn build_send_url() -> String {
    format!("{}/users/me/messages/send", GMAIL_BASE_URL)
}

/// Build the send request body from raw MIME message.
pub fn build_send_body(raw_message: &str) -> serde_json::Value {
    serde_json::json!({
        "raw": raw_message
    })
}

/// Build the URL for sending a draft.
pub fn build_send_draft_url(_draft_id: &str) -> String {
    format!("{}/users/me/drafts/send", GMAIL_BASE_URL)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-GMAIL-010 (Must): Send URL
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-010 (Must)
    // Acceptance: Send URL points to messages/send endpoint
    #[test]
    fn req_gmail_010_send_url() {
        let url = build_send_url();
        assert!(url.contains("users/me/messages/send"));
    }

    // Requirement: REQ-GMAIL-010 (Must)
    // Acceptance: Send body includes raw message
    #[test]
    fn req_gmail_010_send_body() {
        let body = build_send_body("base64urlencoded_message");
        assert!(body["raw"].is_string());
    }
}
