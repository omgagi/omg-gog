//! Google Forms get/create operations.

use super::FORMS_BASE_URL;

/// Build URL for getting a form by ID.
pub fn build_form_get_url(form_id: &str) -> String {
    format!("{}/forms/{}", FORMS_BASE_URL, form_id)
}

/// Build URL for creating a new form.
pub fn build_form_create_url() -> String {
    format!("{}/forms", FORMS_BASE_URL)
}

/// Build the request body for creating a new form.
///
/// The body includes title and documentTitle in the info object.
/// If a description is provided, it is also included in info.
pub fn build_form_create_body(title: &str, description: Option<&str>) -> serde_json::Value {
    let mut info = serde_json::json!({
        "title": title,
        "documentTitle": title
    });
    if let Some(desc) = description {
        info["description"] = serde_json::Value::String(desc.to_string());
    }
    serde_json::json!({
        "info": info
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-FORMS-001 (Must): Get URL construction
    // ---------------------------------------------------------------

    // Requirement: REQ-FORMS-001 (Must)
    // Acceptance: Form get URL is correctly formed
    #[test]
    fn req_forms_001_form_get_url() {
        let url = build_form_get_url("form_abc123");
        assert_eq!(url, "https://forms.googleapis.com/v1/forms/form_abc123");
    }

    // Requirement: REQ-FORMS-001 (Must)
    // Edge case: Empty form ID
    #[test]
    fn req_forms_001_form_get_url_empty_id() {
        let url = build_form_get_url("");
        assert_eq!(url, "https://forms.googleapis.com/v1/forms/");
    }

    // Requirement: REQ-FORMS-001 (Must)
    // Acceptance: URL uses correct base URL
    #[test]
    fn req_forms_001_form_get_url_base() {
        let url = build_form_get_url("test");
        assert!(url.starts_with("https://forms.googleapis.com/v1"));
        assert!(url.contains("/forms/test"));
    }

    // ---------------------------------------------------------------
    // REQ-FORMS-002 (Must): Create URL and body construction
    // ---------------------------------------------------------------

    // Requirement: REQ-FORMS-002 (Must)
    // Acceptance: Create URL is correctly formed
    #[test]
    fn req_forms_002_form_create_url() {
        let url = build_form_create_url();
        assert_eq!(url, "https://forms.googleapis.com/v1/forms");
    }

    // Requirement: REQ-FORMS-002 (Must)
    // Acceptance: Create body includes title and documentTitle
    #[test]
    fn req_forms_002_form_create_body_without_description() {
        let body = build_form_create_body("My Survey", None);
        assert_eq!(body["info"]["title"], "My Survey");
        assert_eq!(body["info"]["documentTitle"], "My Survey");
        assert!(body["info"].get("description").is_none());
    }

    // Requirement: REQ-FORMS-002 (Must)
    // Acceptance: Create body includes description when provided
    #[test]
    fn req_forms_002_form_create_body_with_description() {
        let body = build_form_create_body("Feedback Form", Some("Please share your feedback"));
        assert_eq!(body["info"]["title"], "Feedback Form");
        assert_eq!(body["info"]["documentTitle"], "Feedback Form");
        assert_eq!(body["info"]["description"], "Please share your feedback");
    }

    // Requirement: REQ-FORMS-002 (Must)
    // Edge case: Empty title
    #[test]
    fn req_forms_002_form_create_body_empty_title() {
        let body = build_form_create_body("", None);
        assert_eq!(body["info"]["title"], "");
        assert_eq!(body["info"]["documentTitle"], "");
    }

    // Requirement: REQ-FORMS-002 (Must)
    // Edge case: Title with special characters
    #[test]
    fn req_forms_002_form_create_body_special_chars() {
        let body = build_form_create_body(
            "Survey: \"Q&A\" <2024>",
            Some("Description with \"quotes\""),
        );
        assert_eq!(body["info"]["title"], "Survey: \"Q&A\" <2024>");
        assert_eq!(body["info"]["description"], "Description with \"quotes\"");
    }
}
