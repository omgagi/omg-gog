//! Classroom invitations URL and body builders.

use super::CLASSROOM_BASE_URL;

/// Build URL for listing invitations.
pub fn build_invitations_list_url(
    course_id: Option<&str>,
    user_id: Option<&str>,
    max: Option<u32>,
    page_token: Option<&str>,
) -> String {
    let base = format!("{}/invitations", CLASSROOM_BASE_URL);
    let mut params = Vec::new();
    if let Some(c) = course_id {
        params.push(format!(
            "courseId={}",
            url::form_urlencoded::byte_serialize(c.as_bytes()).collect::<String>()
        ));
    }
    if let Some(u) = user_id {
        params.push(format!(
            "userId={}",
            url::form_urlencoded::byte_serialize(u.as_bytes()).collect::<String>()
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

/// Build URL for getting a single invitation.
pub fn build_invitation_get_url(invitation_id: &str) -> String {
    format!(
        "{}/invitations/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            invitation_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

/// Build URL for creating an invitation.
pub fn build_invitation_create_url() -> String {
    format!("{}/invitations", CLASSROOM_BASE_URL)
}

/// Build request body for creating an invitation.
pub fn build_invitation_create_body(
    user_id: &str,
    course_id: &str,
    role: &str,
) -> serde_json::Value {
    serde_json::json!({
        "userId": user_id,
        "courseId": course_id,
        "role": role,
    })
}

/// Build URL for accepting an invitation.
pub fn build_invitation_accept_url(invitation_id: &str) -> String {
    format!(
        "{}/invitations/{}:accept",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            invitation_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

/// Build URL for deleting an invitation.
pub fn build_invitation_delete_url(invitation_id: &str) -> String {
    format!(
        "{}/invitations/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            invitation_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CLASS-010 (Must): Invitations URL builders
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-010 (Must)
    // Acceptance: Invitations list URL without params
    #[test]
    fn req_class_010_invitations_list_url_no_params() {
        let url = build_invitations_list_url(None, None, None, None);
        assert_eq!(url, format!("{}/invitations", CLASSROOM_BASE_URL));
    }

    // Requirement: REQ-CLASS-010 (Must)
    // Acceptance: Invitations list URL with course filter
    #[test]
    fn req_class_010_invitations_list_url_with_course() {
        let url = build_invitations_list_url(Some("course123"), None, None, None);
        assert!(url.contains("courseId=course123"));
    }

    // Requirement: REQ-CLASS-010 (Must)
    // Acceptance: Invitations list URL with user filter
    #[test]
    fn req_class_010_invitations_list_url_with_user() {
        let url = build_invitations_list_url(None, Some("user@example.com"), None, None);
        assert!(url.contains("userId="));
    }

    // Requirement: REQ-CLASS-010 (Must)
    // Acceptance: Invitations list URL with all params
    #[test]
    fn req_class_010_invitations_list_url_all_params() {
        let url = build_invitations_list_url(Some("c1"), Some("u1"), Some(10), Some("pg2"));
        assert!(url.contains("courseId=c1"));
        assert!(url.contains("userId=u1"));
        assert!(url.contains("pageSize=10"));
        assert!(url.contains("pageToken=pg2"));
    }

    // Requirement: REQ-CLASS-010 (Must)
    // Acceptance: Invitation get URL
    #[test]
    fn req_class_010_invitation_get_url() {
        let url = build_invitation_get_url("inv123");
        assert!(url.contains("/invitations/inv123"));
    }

    // Requirement: REQ-CLASS-010 (Must)
    // Acceptance: Invitation create URL
    #[test]
    fn req_class_010_invitation_create_url() {
        let url = build_invitation_create_url();
        assert!(url.ends_with("/invitations"));
    }

    // Requirement: REQ-CLASS-010 (Must)
    // Acceptance: Invitation create body
    #[test]
    fn req_class_010_invitation_create_body() {
        let body = build_invitation_create_body("user@example.com", "course123", "STUDENT");
        assert_eq!(body["userId"], "user@example.com");
        assert_eq!(body["courseId"], "course123");
        assert_eq!(body["role"], "STUDENT");
    }

    // Requirement: REQ-CLASS-010 (Must)
    // Acceptance: Invitation accept URL
    #[test]
    fn req_class_010_invitation_accept_url() {
        let url = build_invitation_accept_url("inv123");
        assert!(url.contains("/invitations/inv123"));
        assert!(url.contains("accept") || url.contains("%3Aaccept"));
    }

    // Requirement: REQ-CLASS-010 (Must)
    // Acceptance: Invitation delete URL
    #[test]
    fn req_class_010_invitation_delete_url() {
        let url = build_invitation_delete_url("inv123");
        assert!(url.contains("/invitations/inv123"));
    }
}
