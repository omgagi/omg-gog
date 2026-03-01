//! Gmail draft CRUD operations.

use super::types::GMAIL_BASE_URL;

/// Build URL for listing drafts.
pub fn build_drafts_list_url(max_results: Option<u32>, page_token: Option<&str>) -> String {
    let mut url = format!("{}/users/me/drafts", GMAIL_BASE_URL);
    let mut params = Vec::new();
    if let Some(max) = max_results {
        params.push(format!("maxResults={}", max));
    }
    if let Some(token) = page_token {
        params.push(format!("pageToken={}", token));
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }
    url
}

/// Build URL for getting a draft.
pub fn build_draft_get_url(draft_id: &str) -> String {
    format!("{}/users/me/drafts/{}", GMAIL_BASE_URL, draft_id)
}

/// Build URL for creating a draft.
pub fn build_draft_create_url() -> String {
    format!("{}/users/me/drafts", GMAIL_BASE_URL)
}

/// Build URL for updating a draft.
pub fn build_draft_update_url(draft_id: &str) -> String {
    format!("{}/users/me/drafts/{}", GMAIL_BASE_URL, draft_id)
}

/// Build URL for deleting a draft.
pub fn build_draft_delete_url(draft_id: &str) -> String {
    format!("{}/users/me/drafts/{}", GMAIL_BASE_URL, draft_id)
}

/// Build URL for sending a draft.
pub fn build_draft_send_url() -> String {
    format!("{}/users/me/drafts/send", GMAIL_BASE_URL)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-GMAIL-011 (Must): Draft CRUD URLs
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-011 (Must)
    // Acceptance: Drafts list URL
    #[test]
    fn req_gmail_011_drafts_list_url() {
        let url = build_drafts_list_url(Some(10), None);
        assert!(url.contains("users/me/drafts"));
    }

    // Requirement: REQ-GMAIL-011 (Must)
    // Acceptance: Draft get URL
    #[test]
    fn req_gmail_011_draft_get_url() {
        let url = build_draft_get_url("draft123");
        assert!(url.contains("users/me/drafts/draft123"));
    }

    // Requirement: REQ-GMAIL-011 (Must)
    // Acceptance: Draft create URL
    #[test]
    fn req_gmail_011_draft_create_url() {
        let url = build_draft_create_url();
        assert!(url.contains("users/me/drafts"));
    }

    // Requirement: REQ-GMAIL-011 (Must)
    // Acceptance: Draft update URL
    #[test]
    fn req_gmail_011_draft_update_url() {
        let url = build_draft_update_url("draft123");
        assert!(url.contains("users/me/drafts/draft123"));
    }

    // Requirement: REQ-GMAIL-011 (Must)
    // Acceptance: Draft delete URL
    #[test]
    fn req_gmail_011_draft_delete_url() {
        let url = build_draft_delete_url("draft123");
        assert!(url.contains("users/me/drafts/draft123"));
    }

    // Requirement: REQ-GMAIL-011 (Must)
    // Acceptance: Draft send URL
    #[test]
    fn req_gmail_011_draft_send_url() {
        let url = build_draft_send_url();
        assert!(url.contains("users/me/drafts/send"));
    }
}
