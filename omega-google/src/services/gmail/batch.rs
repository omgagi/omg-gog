//! Gmail batch modify/delete operations.

const GMAIL_BASE_URL: &str = "https://gmail.googleapis.com/gmail/v1";

/// Build URL for batch modifying messages.
pub fn build_batch_modify_url() -> String {
    format!("{}/users/me/messages/batchModify", GMAIL_BASE_URL)
}

/// Build URL for batch deleting messages.
pub fn build_batch_delete_url() -> String {
    format!("{}/users/me/messages/batchDelete", GMAIL_BASE_URL)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Requirement: REQ-GMAIL-014 (Must)
    // Acceptance: Batch modify URL
    #[test]
    fn req_gmail_014_batch_modify_url() {
        let url = build_batch_modify_url();
        assert!(url.contains("users/me/messages/batchModify"));
    }

    // Requirement: REQ-GMAIL-014 (Must)
    // Acceptance: Batch delete URL
    #[test]
    fn req_gmail_014_batch_delete_url() {
        let url = build_batch_delete_url();
        assert!(url.contains("users/me/messages/batchDelete"));
    }
}
