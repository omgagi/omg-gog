//! Classroom announcements URL and body builders.

use super::CLASSROOM_BASE_URL;

/// Build URL for listing announcements in a course.
pub fn build_announcements_list_url(
    course_id: &str,
    max: Option<u32>,
    page_token: Option<&str>,
) -> String {
    let base = format!(
        "{}/courses/{}/announcements",
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

/// Build URL for getting a single announcement.
pub fn build_announcement_get_url(course_id: &str, announcement_id: &str) -> String {
    format!(
        "{}/courses/{}/announcements/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(announcement_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build URL for creating an announcement.
pub fn build_announcement_create_url(course_id: &str) -> String {
    format!(
        "{}/courses/{}/announcements",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build request body for creating an announcement.
pub fn build_announcement_create_body(text: &str, state: Option<&str>) -> serde_json::Value {
    let mut body = serde_json::json!({
        "text": text,
    });
    if let Some(s) = state {
        body["state"] = serde_json::Value::String(s.to_string());
    }
    body
}

/// Build URL for updating an announcement.
pub fn build_announcement_update_url(course_id: &str, announcement_id: &str) -> String {
    format!(
        "{}/courses/{}/announcements/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(announcement_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build URL for deleting an announcement.
pub fn build_announcement_delete_url(course_id: &str, announcement_id: &str) -> String {
    format!(
        "{}/courses/{}/announcements/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(announcement_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CLASS-008 (Must): Announcements URL builders
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-008 (Must)
    // Acceptance: Announcements list URL without params
    #[test]
    fn req_class_008_announcements_list_url_no_params() {
        let url = build_announcements_list_url("course123", None, None);
        assert!(url.contains("/courses/course123/announcements"));
        assert!(!url.contains("?"));
    }

    // Requirement: REQ-CLASS-008 (Must)
    // Acceptance: Announcements list URL with params
    #[test]
    fn req_class_008_announcements_list_url_with_params() {
        let url = build_announcements_list_url("course123", Some(15), Some("pg3"));
        assert!(url.contains("pageSize=15"));
        assert!(url.contains("pageToken=pg3"));
    }

    // Requirement: REQ-CLASS-008 (Must)
    // Acceptance: Announcement get URL
    #[test]
    fn req_class_008_announcement_get_url() {
        let url = build_announcement_get_url("course123", "ann456");
        assert!(url.contains("/courses/course123/announcements/ann456"));
    }

    // Requirement: REQ-CLASS-008 (Must)
    // Acceptance: Announcement create URL
    #[test]
    fn req_class_008_announcement_create_url() {
        let url = build_announcement_create_url("course123");
        assert!(url.contains("/courses/course123/announcements"));
    }

    // Requirement: REQ-CLASS-008 (Must)
    // Acceptance: Announcement create body
    #[test]
    fn req_class_008_announcement_create_body() {
        let body = build_announcement_create_body("Welcome to class!", Some("PUBLISHED"));
        assert_eq!(body["text"], "Welcome to class!");
        assert_eq!(body["state"], "PUBLISHED");
    }

    // Requirement: REQ-CLASS-008 (Must)
    // Acceptance: Announcement create body minimal
    #[test]
    fn req_class_008_announcement_create_body_minimal() {
        let body = build_announcement_create_body("Hello", None);
        assert_eq!(body["text"], "Hello");
        assert!(body.get("state").is_none());
    }

    // Requirement: REQ-CLASS-008 (Must)
    // Acceptance: Announcement update URL
    #[test]
    fn req_class_008_announcement_update_url() {
        let url = build_announcement_update_url("course123", "ann456");
        assert!(url.contains("/courses/course123/announcements/ann456"));
    }

    // Requirement: REQ-CLASS-008 (Must)
    // Acceptance: Announcement delete URL
    #[test]
    fn req_class_008_announcement_delete_url() {
        let url = build_announcement_delete_url("course123", "ann456");
        assert!(url.contains("/courses/course123/announcements/ann456"));
    }
}
