//! Gmail history listing.

use super::types::GMAIL_BASE_URL;

/// Build URL for listing history changes.
pub fn build_history_list_url(start_history_id: &str, page_token: Option<&str>) -> String {
    let mut url = format!(
        "{}/users/me/history?startHistoryId={}",
        GMAIL_BASE_URL, start_history_id
    );
    if let Some(token) = page_token {
        url.push_str(&format!("&pageToken={}", token));
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    // Requirement: REQ-GMAIL-013 (Must)
    // Acceptance: History list URL includes startHistoryId
    #[test]
    fn req_gmail_013_history_list_url() {
        let url = build_history_list_url("12345", None);
        assert!(url.contains("users/me/history"));
        assert!(url.contains("startHistoryId=12345"));
    }

    // Requirement: REQ-GMAIL-013 (Must)
    // Acceptance: History list URL with page token
    #[test]
    fn req_gmail_013_history_list_url_with_page() {
        let url = build_history_list_url("12345", Some("page_abc"));
        assert!(url.contains("pageToken=page_abc"));
    }
}
