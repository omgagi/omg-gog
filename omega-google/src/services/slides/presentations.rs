//! Presentations API URL builders and body constructors.

use super::SLIDES_BASE_URL;
use super::super::drive::types::DRIVE_BASE_URL;

/// Build URL for getting a presentation via the Slides API.
pub fn build_presentation_get_url(presentation_id: &str) -> String {
    format!("{}/presentations/{}", SLIDES_BASE_URL, presentation_id)
}

/// Build URL for batchUpdate on a presentation.
pub fn build_batch_update_url(presentation_id: &str) -> String {
    format!(
        "{}/presentations/{}:batchUpdate",
        SLIDES_BASE_URL, presentation_id
    )
}

/// Build request body for creating a new blank presentation via the Slides API.
pub fn build_create_presentation_body(title: &str) -> serde_json::Value {
    serde_json::json!({
        "title": title
    })
}

/// Build request body for creating a presentation from a template.
/// This uses the Drive API copy endpoint, so the body is a Drive copy body.
/// The template_id identifies the source presentation to copy.
pub fn build_create_from_template_body(
    title: &str,
    _template_id: &str,
    parent: Option<&str>,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "name": title
    });
    if let Some(p) = parent {
        body["parents"] = serde_json::json!([p]);
    }
    // The template_id is used in the URL path via build_template_copy_url,
    // not in the body. No internal fields should be injected into the API request.
    body
}

/// Build Drive API copy URL for creating from template.
pub fn build_template_copy_url(template_id: &str) -> String {
    format!("{}/files/{}/copy", DRIVE_BASE_URL, template_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-SLIDES-002 (Must): Get URL
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-002 (Must)
    // Acceptance: Get presentation URL uses Slides API
    #[test]
    fn req_slides_002_get_url() {
        let url = build_presentation_get_url("pres_abc");
        assert!(url.starts_with("https://slides.googleapis.com/v1"));
        assert!(url.contains("presentations/pres_abc"));
    }

    // Requirement: REQ-SLIDES-002 (Must)
    // Acceptance: Get URL with different ID
    #[test]
    fn req_slides_002_get_url_different_id() {
        let url = build_presentation_get_url("1234567890");
        assert!(url.contains("presentations/1234567890"));
    }

    // Requirement: REQ-SLIDES-002 (Must)
    // Edge case: Empty presentation ID
    #[test]
    fn req_slides_002_get_url_empty() {
        let url = build_presentation_get_url("");
        assert!(url.ends_with("presentations/"));
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-003 (Must): Create body
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-003 (Must)
    // Acceptance: Create body contains title
    #[test]
    fn req_slides_003_create_body() {
        let body = build_create_presentation_body("My New Deck");
        assert_eq!(body["title"], "My New Deck");
    }

    // Requirement: REQ-SLIDES-003 (Must)
    // Acceptance: Create body with special characters
    #[test]
    fn req_slides_003_create_body_special_chars() {
        let body = build_create_presentation_body("Presentation & \"Quotes\"");
        assert_eq!(body["title"], "Presentation & \"Quotes\"");
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-003 (Must): Template body
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-003 (Must)
    // Acceptance: Template body contains name but no internal fields
    #[test]
    fn req_slides_003_template_body() {
        let body = build_create_from_template_body("From Template", "template_123", None);
        assert_eq!(body["name"], "From Template");
        // _templateId should NOT be in the body -- it is used in the URL path only
        assert!(body.get("_templateId").is_none());
    }

    // Requirement: REQ-SLIDES-003 (Must)
    // Acceptance: Template body with parent folder
    #[test]
    fn req_slides_003_template_body_with_parent() {
        let body =
            build_create_from_template_body("From Template", "template_123", Some("folder_abc"));
        assert_eq!(body["name"], "From Template");
        let parents = body["parents"].as_array().unwrap();
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0], "folder_abc");
    }

    // Requirement: REQ-SLIDES-003 (Must)
    // Acceptance: Template copy URL uses Drive API
    #[test]
    fn req_slides_003_template_copy_url() {
        let url = build_template_copy_url("template_123");
        assert!(url.contains("files/template_123/copy"));
    }

    // ---------------------------------------------------------------
    // Batch update URL
    // ---------------------------------------------------------------

    // Acceptance: batchUpdate URL uses Slides API
    #[test]
    fn batch_update_url() {
        let url = build_batch_update_url("pres_abc");
        assert!(url.starts_with("https://slides.googleapis.com/v1"));
        assert!(url.contains("presentations/pres_abc:batchUpdate"));
    }
}
