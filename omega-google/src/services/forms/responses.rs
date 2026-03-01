//! Google Forms response list/get operations.

use super::FORMS_BASE_URL;

/// Build URL for listing all responses to a form.
pub fn build_responses_list_url(form_id: &str) -> String {
    format!("{}/forms/{}/responses", FORMS_BASE_URL, form_id)
}

/// Build URL for listing responses with optional pagination and filter parameters.
pub fn build_responses_list_url_with_options(
    form_id: &str,
    max: Option<u32>,
    page_token: Option<&str>,
    filter: Option<&str>,
) -> String {
    let base = format!("{}/forms/{}/responses", FORMS_BASE_URL, form_id);
    let mut params = Vec::new();

    if let Some(m) = max {
        params.push(format!("pageSize={}", m));
    }
    if let Some(token) = page_token {
        params.push(format!(
            "pageToken={}",
            url::form_urlencoded::byte_serialize(token.as_bytes()).collect::<String>()
        ));
    }
    if let Some(f) = filter {
        params.push(format!(
            "filter={}",
            url::form_urlencoded::byte_serialize(f.as_bytes()).collect::<String>()
        ));
    }

    if params.is_empty() {
        base
    } else {
        format!("{}?{}", base, params.join("&"))
    }
}

/// Build URL for getting a single response by ID.
pub fn build_response_get_url(form_id: &str, response_id: &str) -> String {
    format!("{}/forms/{}/responses/{}", FORMS_BASE_URL, form_id, response_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-FORMS-003 (Must): List responses URL with pagination
    // ---------------------------------------------------------------

    // Requirement: REQ-FORMS-003 (Must)
    // Acceptance: Basic list responses URL
    #[test]
    fn req_forms_003_list_responses_url() {
        let url = build_responses_list_url("form_123");
        assert_eq!(url, "https://forms.googleapis.com/v1/forms/form_123/responses");
    }

    // Requirement: REQ-FORMS-003 (Must)
    // Acceptance: List URL with all options
    #[test]
    fn req_forms_003_list_responses_url_with_all_options() {
        let url = build_responses_list_url_with_options(
            "form_123",
            Some(50),
            Some("token_abc"),
            Some("timestamp > 2024-01-01T00:00:00Z"),
        );
        assert!(url.contains("forms/form_123/responses"));
        assert!(url.contains("pageSize=50"));
        assert!(url.contains("pageToken=token_abc"));
        assert!(url.contains("filter="));
    }

    // Requirement: REQ-FORMS-003 (Must)
    // Acceptance: List URL with only max
    #[test]
    fn req_forms_003_list_responses_url_max_only() {
        let url = build_responses_list_url_with_options("form_123", Some(10), None, None);
        assert!(url.contains("pageSize=10"));
        assert!(!url.contains("pageToken"));
        assert!(!url.contains("filter"));
    }

    // Requirement: REQ-FORMS-003 (Must)
    // Acceptance: List URL with only page token
    #[test]
    fn req_forms_003_list_responses_url_page_token_only() {
        let url = build_responses_list_url_with_options("form_123", None, Some("next_page"), None);
        assert!(url.contains("pageToken=next_page"));
        assert!(!url.contains("pageSize"));
        assert!(!url.contains("filter"));
    }

    // Requirement: REQ-FORMS-003 (Must)
    // Edge case: No options produces clean URL without query string
    #[test]
    fn req_forms_003_list_responses_url_no_options() {
        let url = build_responses_list_url_with_options("form_123", None, None, None);
        assert_eq!(url, "https://forms.googleapis.com/v1/forms/form_123/responses");
        assert!(!url.contains('?'));
    }

    // ---------------------------------------------------------------
    // REQ-FORMS-004 (Must): Get response URL
    // ---------------------------------------------------------------

    // Requirement: REQ-FORMS-004 (Must)
    // Acceptance: Get response URL is correctly formed
    #[test]
    fn req_forms_004_get_response_url() {
        let url = build_response_get_url("form_123", "resp_456");
        assert_eq!(url, "https://forms.googleapis.com/v1/forms/form_123/responses/resp_456");
    }

    // Requirement: REQ-FORMS-004 (Must)
    // Edge case: Empty form and response IDs
    #[test]
    fn req_forms_004_get_response_url_empty_ids() {
        let url = build_response_get_url("", "");
        assert_eq!(url, "https://forms.googleapis.com/v1/forms//responses/");
    }

    // Requirement: REQ-FORMS-003 (Must)
    // Edge case: Filter with special characters is URL-encoded
    #[test]
    fn req_forms_003_list_responses_url_filter_encoding() {
        let url = build_responses_list_url_with_options(
            "form_123",
            None,
            None,
            Some("timestamp >= 2024-01-01T00:00:00Z"),
        );
        assert!(url.contains("filter="));
        // The >= and spaces should be encoded
        assert!(!url.contains(" >= "));
    }
}
