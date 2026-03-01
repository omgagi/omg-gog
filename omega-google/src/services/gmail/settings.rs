//! Gmail settings: filters, forwarding, send-as, delegates, vacation, autoforward.

use super::types::GMAIL_BASE_URL;

/// Build URL for listing filters.
pub fn build_filters_list_url() -> String {
    format!("{}/users/me/settings/filters", GMAIL_BASE_URL)
}

/// Build URL for getting a filter.
pub fn build_filter_get_url(filter_id: &str) -> String {
    format!("{}/users/me/settings/filters/{}", GMAIL_BASE_URL, filter_id)
}

/// Build URL for creating a filter.
pub fn build_filter_create_url() -> String {
    format!("{}/users/me/settings/filters", GMAIL_BASE_URL)
}

/// Build URL for deleting a filter.
pub fn build_filter_delete_url(filter_id: &str) -> String {
    format!("{}/users/me/settings/filters/{}", GMAIL_BASE_URL, filter_id)
}

/// Build URL for listing forwarding addresses.
pub fn build_forwarding_list_url() -> String {
    format!("{}/users/me/settings/forwardingAddresses", GMAIL_BASE_URL)
}

/// Build URL for getting vacation settings.
pub fn build_vacation_get_url() -> String {
    format!("{}/users/me/settings/vacation", GMAIL_BASE_URL)
}

/// Build URL for updating vacation settings.
pub fn build_vacation_update_url() -> String {
    format!("{}/users/me/settings/vacation", GMAIL_BASE_URL)
}

/// Build URL for getting autoforward settings.
pub fn build_autoforward_get_url() -> String {
    format!("{}/users/me/settings/autoForwarding", GMAIL_BASE_URL)
}

/// Build URL for listing send-as aliases.
pub fn build_sendas_list_url() -> String {
    format!("{}/users/me/settings/sendAs", GMAIL_BASE_URL)
}

/// Build URL for listing delegates.
pub fn build_delegates_list_url() -> String {
    format!("{}/users/me/settings/delegates", GMAIL_BASE_URL)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Requirement: REQ-GMAIL-015 (Must)
    #[test]
    fn req_gmail_015_filters_list_url() {
        let url = build_filters_list_url();
        assert!(url.contains("users/me/settings/filters"));
    }

    // Requirement: REQ-GMAIL-015 (Must)
    #[test]
    fn req_gmail_015_filter_get_url() {
        let url = build_filter_get_url("filter_abc");
        assert!(url.contains("users/me/settings/filters/filter_abc"));
    }

    // Requirement: REQ-GMAIL-016 (Must)
    #[test]
    fn req_gmail_016_forwarding_list_url() {
        let url = build_forwarding_list_url();
        assert!(url.contains("users/me/settings/forwardingAddresses"));
    }

    // Requirement: REQ-GMAIL-017 (Must)
    #[test]
    fn req_gmail_017_sendas_list_url() {
        let url = build_sendas_list_url();
        assert!(url.contains("users/me/settings/sendAs"));
    }

    // Requirement: REQ-GMAIL-018 (Must)
    #[test]
    fn req_gmail_018_delegates_list_url() {
        let url = build_delegates_list_url();
        assert!(url.contains("users/me/settings/delegates"));
    }

    // Requirement: REQ-GMAIL-019 (Must)
    #[test]
    fn req_gmail_019_vacation_get_url() {
        let url = build_vacation_get_url();
        assert!(url.contains("users/me/settings/vacation"));
    }

    // Requirement: REQ-GMAIL-020 (Must)
    #[test]
    fn req_gmail_020_autoforward_get_url() {
        let url = build_autoforward_get_url();
        assert!(url.contains("users/me/settings/autoForwarding"));
    }
}
