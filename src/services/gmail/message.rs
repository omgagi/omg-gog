//! Gmail message get, attachment download.

use super::types::*;

/// Build URL for getting a single message.
pub fn build_message_get_url(message_id: &str, format: Option<&str>) -> String {
    let base = format!("{}/users/me/messages/{}", GMAIL_BASE_URL, message_id);
    match format {
        Some(fmt) => format!("{}?format={}", base, fmt),
        None => base,
    }
}

/// Build URL for downloading an attachment.
pub fn build_attachment_url(message_id: &str, attachment_id: &str) -> String {
    format!(
        "{}/users/me/messages/{}/attachments/{}",
        GMAIL_BASE_URL, message_id, attachment_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-GMAIL-006 (Must): Message get URL
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-006 (Must)
    // Acceptance: Message get URL with format parameter
    #[test]
    fn req_gmail_006_message_get_url_full() {
        let url = build_message_get_url("msg123", Some("full"));
        assert!(url.contains("users/me/messages/msg123"));
        assert!(url.contains("format=full"));
    }

    // Requirement: REQ-GMAIL-006 (Must)
    // Acceptance: Message get URL with metadata format
    #[test]
    fn req_gmail_006_message_get_url_metadata() {
        let url = build_message_get_url("msg123", Some("metadata"));
        assert!(url.contains("format=metadata"));
    }

    // Requirement: REQ-GMAIL-006 (Must)
    // Acceptance: Message get URL without format defaults correctly
    #[test]
    fn req_gmail_006_message_get_url_no_format() {
        let url = build_message_get_url("msg123", None);
        assert!(url.contains("users/me/messages/msg123"));
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-007 (Must): Attachment URL
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-007 (Must)
    // Acceptance: Attachment URL is correctly formed
    #[test]
    fn req_gmail_007_attachment_url() {
        let url = build_attachment_url("msg123", "attach_abc");
        assert!(url.contains("messages/msg123/attachments/attach_abc"));
    }

    // Requirement: REQ-GMAIL-007 (Must)
    // Edge case: Empty IDs
    #[test]
    fn req_gmail_007_attachment_url_empty_ids() {
        let url = build_attachment_url("", "");
        assert!(url.contains("messages//attachments/"));
    }
}
