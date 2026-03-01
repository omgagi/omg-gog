//! Classroom course URL and body builders.

use super::CLASSROOM_BASE_URL;

/// Build URL for listing courses.
pub fn build_courses_list_url(
    state: Option<&str>,
    max: Option<u32>,
    page_token: Option<&str>,
) -> String {
    let base = format!("{}/courses", CLASSROOM_BASE_URL);
    let mut params = Vec::new();
    if let Some(s) = state {
        params.push(format!(
            "courseStates={}",
            url::form_urlencoded::byte_serialize(s.as_bytes()).collect::<String>()
        ));
    }
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

/// Build URL for getting a single course.
pub fn build_course_get_url(course_id: &str) -> String {
    format!(
        "{}/courses/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

/// Build URL for creating a course.
pub fn build_course_create_url() -> String {
    format!("{}/courses", CLASSROOM_BASE_URL)
}

/// Build request body for creating a course.
pub fn build_course_create_body(
    name: &str,
    owner: Option<&str>,
    state: Option<&str>,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "name": name,
    });
    if let Some(o) = owner {
        body["ownerId"] = serde_json::Value::String(o.to_string());
    }
    if let Some(s) = state {
        body["courseState"] = serde_json::Value::String(s.to_string());
    }
    body
}

/// Build URL for updating a course.
pub fn build_course_update_url(course_id: &str) -> String {
    format!(
        "{}/courses/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

/// Build request body for updating a course.
pub fn build_course_update_body(
    name: Option<&str>,
    state: Option<&str>,
) -> serde_json::Value {
    let mut body = serde_json::json!({});
    if let Some(n) = name {
        body["name"] = serde_json::Value::String(n.to_string());
    }
    if let Some(s) = state {
        body["courseState"] = serde_json::Value::String(s.to_string());
    }
    body
}

/// Build URL for deleting a course.
pub fn build_course_delete_url(course_id: &str) -> String {
    format!(
        "{}/courses/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

/// Build URL for archiving a course (PATCH with courseState=ARCHIVED).
pub fn build_course_archive_url(course_id: &str) -> String {
    format!(
        "{}/courses/{}?updateMask=courseState",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

/// Generate a Google Classroom web URL for a course.
pub fn build_course_url(course_id: &str) -> String {
    format!("https://classroom.google.com/c/{}", course_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CLASS-001 (Must): Course URL builders
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: Courses list URL without params
    #[test]
    fn req_class_001_courses_list_url_no_params() {
        let url = build_courses_list_url(None, None, None);
        assert_eq!(url, format!("{}/courses", CLASSROOM_BASE_URL));
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: Courses list URL with state filter
    #[test]
    fn req_class_001_courses_list_url_with_state() {
        let url = build_courses_list_url(Some("ACTIVE"), None, None);
        assert!(url.contains("courseStates=ACTIVE"));
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: Courses list URL with max and page token
    #[test]
    fn req_class_001_courses_list_url_with_max_and_page() {
        let url = build_courses_list_url(None, Some(20), Some("token123"));
        assert!(url.contains("pageSize=20"));
        assert!(url.contains("pageToken=token123"));
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: Course get URL
    #[test]
    fn req_class_001_course_get_url() {
        let url = build_course_get_url("12345");
        assert!(url.contains("/courses/12345"));
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: Course create URL
    #[test]
    fn req_class_001_course_create_url() {
        let url = build_course_create_url();
        assert!(url.ends_with("/courses"));
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: Course create body with all fields
    #[test]
    fn req_class_001_course_create_body() {
        let body = build_course_create_body("Math 101", Some("teacher@example.com"), Some("PROVISIONED"));
        assert_eq!(body["name"], "Math 101");
        assert_eq!(body["ownerId"], "teacher@example.com");
        assert_eq!(body["courseState"], "PROVISIONED");
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: Course create body with minimum fields
    #[test]
    fn req_class_001_course_create_body_minimal() {
        let body = build_course_create_body("Science", None, None);
        assert_eq!(body["name"], "Science");
        assert!(body.get("ownerId").is_none());
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: Course update URL
    #[test]
    fn req_class_001_course_update_url() {
        let url = build_course_update_url("12345");
        assert!(url.contains("/courses/12345"));
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: Course update body
    #[test]
    fn req_class_001_course_update_body() {
        let body = build_course_update_body(Some("Updated Name"), Some("ARCHIVED"));
        assert_eq!(body["name"], "Updated Name");
        assert_eq!(body["courseState"], "ARCHIVED");
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: Course update body empty
    #[test]
    fn req_class_001_course_update_body_empty() {
        let body = build_course_update_body(None, None);
        assert!(body.as_object().unwrap().is_empty());
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: Course delete URL
    #[test]
    fn req_class_001_course_delete_url() {
        let url = build_course_delete_url("12345");
        assert!(url.contains("/courses/12345"));
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: Course archive URL includes updateMask
    #[test]
    fn req_class_001_course_archive_url() {
        let url = build_course_archive_url("12345");
        assert!(url.contains("/courses/12345"));
        assert!(url.contains("updateMask=courseState"));
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: Course web URL
    #[test]
    fn req_class_001_course_web_url() {
        let url = build_course_url("12345");
        assert_eq!(url, "https://classroom.google.com/c/12345");
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Edge case: Empty course ID
    #[test]
    fn req_class_001_course_web_url_empty() {
        let url = build_course_url("");
        assert_eq!(url, "https://classroom.google.com/c/");
    }
}
