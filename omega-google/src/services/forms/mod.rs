//! Google Forms service module.
//! Provides types and URL builders for the Forms API.

pub mod types;

/// Google Forms API v1 base URL.
pub const FORMS_BASE_URL: &str = "https://forms.googleapis.com/v1";

/// Build URL for getting a form by ID.
pub fn build_form_get_url(form_id: &str) -> String {
    format!("{}/forms/{}", FORMS_BASE_URL, form_id)
}

/// Build URL for creating a new form.
pub fn build_form_create_url() -> String {
    format!("{}/forms", FORMS_BASE_URL)
}

/// Build the request body for creating a new form.
pub fn build_form_create_body(title: &str, description: Option<&str>) -> serde_json::Value {
    let mut info = serde_json::json!({"title": title});
    if let Some(desc) = description {
        info["description"] = serde_json::json!(desc);
    }
    serde_json::json!({"info": info})
}

/// Build URL for listing form responses.
pub fn build_responses_list_url(form_id: &str) -> String {
    format!("{}/forms/{}/responses", FORMS_BASE_URL, form_id)
}

/// Build URL for getting a specific form response.
pub fn build_response_get_url(form_id: &str, response_id: &str) -> String {
    format!("{}/forms/{}/responses/{}", FORMS_BASE_URL, form_id, response_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_get_url() {
        let url = build_form_get_url("form123");
        assert!(url.contains("forms/form123"));
    }

    #[test]
    fn test_form_create_url() {
        let url = build_form_create_url();
        assert!(url.ends_with("/forms"));
    }

    #[test]
    fn test_form_create_body() {
        let body = build_form_create_body("My Form", Some("A description"));
        assert_eq!(body["info"]["title"], "My Form");
        assert_eq!(body["info"]["description"], "A description");
    }

    #[test]
    fn test_form_create_body_no_desc() {
        let body = build_form_create_body("My Form", None);
        assert_eq!(body["info"]["title"], "My Form");
        assert!(body["info"].get("description").is_none());
    }

    #[test]
    fn test_responses_list_url() {
        let url = build_responses_list_url("form123");
        assert!(url.contains("forms/form123/responses"));
    }

    #[test]
    fn test_response_get_url() {
        let url = build_response_get_url("form123", "resp456");
        assert!(url.contains("forms/form123/responses/resp456"));
    }
}
