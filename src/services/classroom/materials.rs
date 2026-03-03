//! Classroom course materials URL and body builders.

use super::CLASSROOM_BASE_URL;

/// Build URL for listing course materials.
pub fn build_materials_list_url(
    course_id: &str,
    max: Option<u32>,
    page_token: Option<&str>,
) -> String {
    let base = format!(
        "{}/courses/{}/courseWorkMaterials",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC)
    );
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
    if params.is_empty() {
        base
    } else {
        format!("{}?{}", base, params.join("&"))
    }
}

/// Build URL for getting a single course material.
pub fn build_material_get_url(course_id: &str, material_id: &str) -> String {
    format!(
        "{}/courses/{}/courseWorkMaterials/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(material_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build URL for creating a course material.
pub fn build_material_create_url(course_id: &str) -> String {
    format!(
        "{}/courses/{}/courseWorkMaterials",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build request body for creating a course material.
pub fn build_material_create_body(
    title: &str,
    description: Option<&str>,
    topic_id: Option<&str>,
    state: Option<&str>,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "title": title,
    });
    if let Some(d) = description {
        body["description"] = serde_json::Value::String(d.to_string());
    }
    if let Some(t) = topic_id {
        body["topicId"] = serde_json::Value::String(t.to_string());
    }
    if let Some(s) = state {
        body["state"] = serde_json::Value::String(s.to_string());
    }
    body
}

/// Build URL for updating a course material.
pub fn build_material_update_url(course_id: &str, material_id: &str) -> String {
    format!(
        "{}/courses/{}/courseWorkMaterials/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(material_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build URL for deleting a course material.
pub fn build_material_delete_url(course_id: &str, material_id: &str) -> String {
    format!(
        "{}/courses/{}/courseWorkMaterials/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(material_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CLASS-006 (Must): Course materials URL builders
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-006 (Must)
    // Acceptance: Materials list URL without params
    #[test]
    fn req_class_006_materials_list_url_no_params() {
        let url = build_materials_list_url("course123", None, None);
        assert!(url.contains("/courses/course123/courseWorkMaterials"));
        assert!(!url.contains("?"));
    }

    // Requirement: REQ-CLASS-006 (Must)
    // Acceptance: Materials list URL with params
    #[test]
    fn req_class_006_materials_list_url_with_params() {
        let url = build_materials_list_url("course123", Some(20), Some("page2"));
        assert!(url.contains("pageSize=20"));
        assert!(url.contains("pageToken=page2"));
    }

    // Requirement: REQ-CLASS-006 (Must)
    // Acceptance: Material get URL
    #[test]
    fn req_class_006_material_get_url() {
        let url = build_material_get_url("course123", "mat456");
        assert!(url.contains("/courses/course123/courseWorkMaterials/mat456"));
    }

    // Requirement: REQ-CLASS-006 (Must)
    // Acceptance: Material create URL
    #[test]
    fn req_class_006_material_create_url() {
        let url = build_material_create_url("course123");
        assert!(url.contains("/courses/course123/courseWorkMaterials"));
    }

    // Requirement: REQ-CLASS-006 (Must)
    // Acceptance: Material create body with all fields
    #[test]
    fn req_class_006_material_create_body_full() {
        let body = build_material_create_body(
            "Reading",
            Some("Chapter 1"),
            Some("topic1"),
            Some("PUBLISHED"),
        );
        assert_eq!(body["title"], "Reading");
        assert_eq!(body["description"], "Chapter 1");
        assert_eq!(body["topicId"], "topic1");
        assert_eq!(body["state"], "PUBLISHED");
    }

    // Requirement: REQ-CLASS-006 (Must)
    // Acceptance: Material create body minimal
    #[test]
    fn req_class_006_material_create_body_minimal() {
        let body = build_material_create_body("Slides", None, None, None);
        assert_eq!(body["title"], "Slides");
        assert!(body.get("description").is_none());
    }

    // Requirement: REQ-CLASS-006 (Must)
    // Acceptance: Material update URL
    #[test]
    fn req_class_006_material_update_url() {
        let url = build_material_update_url("course123", "mat456");
        assert!(url.contains("/courses/course123/courseWorkMaterials/mat456"));
    }

    // Requirement: REQ-CLASS-006 (Must)
    // Acceptance: Material delete URL
    #[test]
    fn req_class_006_material_delete_url() {
        let url = build_material_delete_url("course123", "mat456");
        assert!(url.contains("/courses/course123/courseWorkMaterials/mat456"));
    }
}
