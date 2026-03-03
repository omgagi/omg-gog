//! Classroom topics URL and body builders.

use super::CLASSROOM_BASE_URL;

/// Build URL for listing topics in a course.
pub fn build_topics_list_url(
    course_id: &str,
    max: Option<u32>,
    page_token: Option<&str>,
) -> String {
    let base = format!(
        "{}/courses/{}/topics",
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

/// Build URL for getting a single topic.
pub fn build_topic_get_url(course_id: &str, topic_id: &str) -> String {
    format!(
        "{}/courses/{}/topics/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(topic_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build URL for creating a topic.
pub fn build_topic_create_url(course_id: &str) -> String {
    format!(
        "{}/courses/{}/topics",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build request body for creating a topic.
pub fn build_topic_create_body(name: &str) -> serde_json::Value {
    serde_json::json!({
        "name": name,
    })
}

/// Build URL for updating a topic.
pub fn build_topic_update_url(course_id: &str, topic_id: &str) -> String {
    format!(
        "{}/courses/{}/topics/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(topic_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build request body for updating a topic.
pub fn build_topic_update_body(name: &str) -> serde_json::Value {
    serde_json::json!({
        "name": name,
    })
}

/// Build URL for deleting a topic.
pub fn build_topic_delete_url(course_id: &str, topic_id: &str) -> String {
    format!(
        "{}/courses/{}/topics/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(course_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(topic_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CLASS-009 (Must): Topics URL builders
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-009 (Must)
    // Acceptance: Topics list URL without params
    #[test]
    fn req_class_009_topics_list_url_no_params() {
        let url = build_topics_list_url("course123", None, None);
        assert!(url.contains("/courses/course123/topics"));
        assert!(!url.contains("?"));
    }

    // Requirement: REQ-CLASS-009 (Must)
    // Acceptance: Topics list URL with params
    #[test]
    fn req_class_009_topics_list_url_with_params() {
        let url = build_topics_list_url("course123", Some(10), Some("pg2"));
        assert!(url.contains("pageSize=10"));
        assert!(url.contains("pageToken=pg2"));
    }

    // Requirement: REQ-CLASS-009 (Must)
    // Acceptance: Topic get URL
    #[test]
    fn req_class_009_topic_get_url() {
        let url = build_topic_get_url("course123", "topic456");
        assert!(url.contains("/courses/course123/topics/topic456"));
    }

    // Requirement: REQ-CLASS-009 (Must)
    // Acceptance: Topic create URL
    #[test]
    fn req_class_009_topic_create_url() {
        let url = build_topic_create_url("course123");
        assert!(url.contains("/courses/course123/topics"));
    }

    // Requirement: REQ-CLASS-009 (Must)
    // Acceptance: Topic create body
    #[test]
    fn req_class_009_topic_create_body() {
        let body = build_topic_create_body("Week 1");
        assert_eq!(body["name"], "Week 1");
    }

    // Requirement: REQ-CLASS-009 (Must)
    // Acceptance: Topic update URL
    #[test]
    fn req_class_009_topic_update_url() {
        let url = build_topic_update_url("course123", "topic456");
        assert!(url.contains("/courses/course123/topics/topic456"));
    }

    // Requirement: REQ-CLASS-009 (Must)
    // Acceptance: Topic update body
    #[test]
    fn req_class_009_topic_update_body() {
        let body = build_topic_update_body("Week 2");
        assert_eq!(body["name"], "Week 2");
    }

    // Requirement: REQ-CLASS-009 (Must)
    // Acceptance: Topic delete URL
    #[test]
    fn req_class_009_topic_delete_url() {
        let url = build_topic_delete_url("course123", "topic456");
        assert!(url.contains("/courses/course123/topics/topic456"));
    }
}
