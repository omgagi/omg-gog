//! Classroom coursework URL and body builders.

use super::CLASSROOM_BASE_URL;

/// Build URL for listing coursework in a course.
pub fn build_coursework_list_url(
    course_id: &str,
    max: Option<u32>,
    page_token: Option<&str>,
) -> String {
    let base = format!(
        "{}/courses/{}/courseWork",
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

/// Build URL for getting a single coursework item.
pub fn build_coursework_get_url(course_id: &str, coursework_id: &str) -> String {
    format!(
        "{}/courses/{}/courseWork/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(coursework_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build URL for creating coursework.
pub fn build_coursework_create_url(course_id: &str) -> String {
    format!(
        "{}/courses/{}/courseWork",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build request body for creating coursework.
pub fn build_coursework_create_body(
    title: &str,
    work_type: &str,
    description: Option<&str>,
    max_points: Option<f64>,
    state: Option<&str>,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "title": title,
        "workType": work_type,
    });
    if let Some(d) = description {
        body["description"] = serde_json::Value::String(d.to_string());
    }
    if let Some(mp) = max_points {
        body["maxPoints"] = serde_json::json!(mp);
    }
    if let Some(s) = state {
        body["state"] = serde_json::Value::String(s.to_string());
    }
    body
}

/// Build URL for updating coursework.
pub fn build_coursework_update_url(course_id: &str, coursework_id: &str) -> String {
    format!(
        "{}/courses/{}/courseWork/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(coursework_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build URL for deleting coursework.
pub fn build_coursework_delete_url(course_id: &str, coursework_id: &str) -> String {
    format!(
        "{}/courses/{}/courseWork/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(coursework_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CLASS-005 (Must): CourseWork URL builders
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-005 (Must)
    // Acceptance: CourseWork list URL without params
    #[test]
    fn req_class_005_coursework_list_url_no_params() {
        let url = build_coursework_list_url("course123", None, None);
        assert!(url.contains("/courses/course123/courseWork"));
        assert!(!url.contains("?"));
    }

    // Requirement: REQ-CLASS-005 (Must)
    // Acceptance: CourseWork list URL with params
    #[test]
    fn req_class_005_coursework_list_url_with_params() {
        let url = build_coursework_list_url("course123", Some(10), Some("next"));
        assert!(url.contains("pageSize=10"));
        assert!(url.contains("pageToken=next"));
    }

    // Requirement: REQ-CLASS-005 (Must)
    // Acceptance: CourseWork get URL
    #[test]
    fn req_class_005_coursework_get_url() {
        let url = build_coursework_get_url("course123", "cw456");
        assert!(url.contains("/courses/course123/courseWork/cw456"));
    }

    // Requirement: REQ-CLASS-005 (Must)
    // Acceptance: CourseWork create URL
    #[test]
    fn req_class_005_coursework_create_url() {
        let url = build_coursework_create_url("course123");
        assert!(url.contains("/courses/course123/courseWork"));
    }

    // Requirement: REQ-CLASS-005 (Must)
    // Acceptance: CourseWork create body with all fields
    #[test]
    fn req_class_005_coursework_create_body_full() {
        let body = build_coursework_create_body(
            "Homework 1",
            "ASSIGNMENT",
            Some("Complete exercises 1-10"),
            Some(100.0),
            Some("PUBLISHED"),
        );
        assert_eq!(body["title"], "Homework 1");
        assert_eq!(body["workType"], "ASSIGNMENT");
        assert_eq!(body["description"], "Complete exercises 1-10");
        assert_eq!(body["maxPoints"], 100.0);
        assert_eq!(body["state"], "PUBLISHED");
    }

    // Requirement: REQ-CLASS-005 (Must)
    // Acceptance: CourseWork create body minimal
    #[test]
    fn req_class_005_coursework_create_body_minimal() {
        let body =
            build_coursework_create_body("Quiz 1", "SHORT_ANSWER_QUESTION", None, None, None);
        assert_eq!(body["title"], "Quiz 1");
        assert_eq!(body["workType"], "SHORT_ANSWER_QUESTION");
        assert!(body.get("description").is_none());
    }

    // Requirement: REQ-CLASS-005 (Must)
    // Acceptance: CourseWork update URL
    #[test]
    fn req_class_005_coursework_update_url() {
        let url = build_coursework_update_url("course123", "cw456");
        assert!(url.contains("/courses/course123/courseWork/cw456"));
    }

    // Requirement: REQ-CLASS-005 (Must)
    // Acceptance: CourseWork delete URL
    #[test]
    fn req_class_005_coursework_delete_url() {
        let url = build_coursework_delete_url("course123", "cw456");
        assert!(url.contains("/courses/course123/courseWork/cw456"));
    }
}
