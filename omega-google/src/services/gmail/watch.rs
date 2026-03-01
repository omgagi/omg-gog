//! Gmail watch (push notification) management.

const GMAIL_BASE_URL: &str = "https://gmail.googleapis.com/gmail/v1";

/// Build URL for starting a watch.
pub fn build_watch_start_url() -> String {
    format!("{}/users/me/watch", GMAIL_BASE_URL)
}

/// Build URL for stopping a watch.
pub fn build_watch_stop_url() -> String {
    format!("{}/users/me/stop", GMAIL_BASE_URL)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Requirement: REQ-GMAIL-012 (Must)
    // Acceptance: Watch start URL
    #[test]
    fn req_gmail_012_watch_start_url() {
        let url = build_watch_start_url();
        assert!(url.contains("users/me/watch"));
    }

    // Requirement: REQ-GMAIL-012 (Must)
    // Acceptance: Watch stop URL
    #[test]
    fn req_gmail_012_watch_stop_url() {
        let url = build_watch_stop_url();
        assert!(url.contains("users/me/stop"));
    }
}
