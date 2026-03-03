//! Classroom guardians and guardian invitations URL and body builders.

use super::CLASSROOM_BASE_URL;

// ---------------------------------------------------------------
// Guardian URL builders
// ---------------------------------------------------------------

/// Build URL for listing guardians for a student.
pub fn build_guardians_list_url(
    student_id: &str,
    max: Option<u32>,
    page_token: Option<&str>,
) -> String {
    let base = format!(
        "{}/userProfiles/{}/guardians",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(student_id, percent_encoding::NON_ALPHANUMERIC)
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

/// Build URL for getting a single guardian.
pub fn build_guardian_get_url(student_id: &str, guardian_id: &str) -> String {
    format!(
        "{}/userProfiles/{}/guardians/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(student_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(guardian_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build URL for deleting a guardian.
pub fn build_guardian_delete_url(student_id: &str, guardian_id: &str) -> String {
    format!(
        "{}/userProfiles/{}/guardians/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(student_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(guardian_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

// ---------------------------------------------------------------
// Guardian Invitation URL builders
// ---------------------------------------------------------------

/// Build URL for listing guardian invitations.
pub fn build_guardian_invitations_list_url(
    student_id: &str,
    states: Option<&str>,
    max: Option<u32>,
    page_token: Option<&str>,
) -> String {
    let base = format!(
        "{}/userProfiles/{}/guardianInvitations",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(student_id, percent_encoding::NON_ALPHANUMERIC)
    );
    let mut params = Vec::new();
    if let Some(s) = states {
        params.push(format!(
            "states={}",
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

/// Build URL for getting a single guardian invitation.
pub fn build_guardian_invitation_get_url(student_id: &str, invitation_id: &str) -> String {
    format!(
        "{}/userProfiles/{}/guardianInvitations/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(student_id, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(invitation_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build URL for creating a guardian invitation.
pub fn build_guardian_invitation_create_url(student_id: &str) -> String {
    format!(
        "{}/userProfiles/{}/guardianInvitations",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(student_id, percent_encoding::NON_ALPHANUMERIC)
    )
}

/// Build request body for creating a guardian invitation.
pub fn build_guardian_invitation_create_body(invited_email_address: &str) -> serde_json::Value {
    serde_json::json!({
        "invitedEmailAddress": invited_email_address,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CLASS-011 (Must): Guardian URL builders
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-011 (Must)
    // Acceptance: Guardians list URL without params
    #[test]
    fn req_class_011_guardians_list_url_no_params() {
        let url = build_guardians_list_url("student123", None, None);
        assert!(url.contains("/userProfiles/student123/guardians"));
        assert!(!url.contains("?"));
    }

    // Requirement: REQ-CLASS-011 (Must)
    // Acceptance: Guardians list URL with params
    #[test]
    fn req_class_011_guardians_list_url_with_params() {
        let url = build_guardians_list_url("student123", Some(10), Some("pg2"));
        assert!(url.contains("pageSize=10"));
        assert!(url.contains("pageToken=pg2"));
    }

    // Requirement: REQ-CLASS-011 (Must)
    // Acceptance: Guardian get URL
    #[test]
    fn req_class_011_guardian_get_url() {
        let url = build_guardian_get_url("student123", "guardian456");
        assert!(url.contains("/userProfiles/student123/guardians/guardian456"));
    }

    // Requirement: REQ-CLASS-011 (Must)
    // Acceptance: Guardian delete URL
    #[test]
    fn req_class_011_guardian_delete_url() {
        let url = build_guardian_delete_url("student123", "guardian456");
        assert!(url.contains("/userProfiles/student123/guardians/guardian456"));
    }

    // ---------------------------------------------------------------
    // REQ-CLASS-012 (Must): Guardian invitation URL builders
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-012 (Must)
    // Acceptance: Guardian invitations list URL without params
    #[test]
    fn req_class_012_guardian_invitations_list_url_no_params() {
        let url = build_guardian_invitations_list_url("student123", None, None, None);
        assert!(url.contains("/userProfiles/student123/guardianInvitations"));
        assert!(!url.contains("?"));
    }

    // Requirement: REQ-CLASS-012 (Must)
    // Acceptance: Guardian invitations list URL with state filter
    #[test]
    fn req_class_012_guardian_invitations_list_url_with_state() {
        let url = build_guardian_invitations_list_url("student123", Some("PENDING"), None, None);
        assert!(url.contains("states=PENDING"));
    }

    // Requirement: REQ-CLASS-012 (Must)
    // Acceptance: Guardian invitations list URL with all params
    #[test]
    fn req_class_012_guardian_invitations_list_url_all_params() {
        let url = build_guardian_invitations_list_url("s1", Some("PENDING"), Some(5), Some("pg2"));
        assert!(url.contains("states=PENDING"));
        assert!(url.contains("pageSize=5"));
        assert!(url.contains("pageToken=pg2"));
    }

    // Requirement: REQ-CLASS-012 (Must)
    // Acceptance: Guardian invitation get URL
    #[test]
    fn req_class_012_guardian_invitation_get_url() {
        let url = build_guardian_invitation_get_url("student123", "ginv456");
        assert!(url.contains("/userProfiles/student123/guardianInvitations/ginv456"));
    }

    // Requirement: REQ-CLASS-013 (Must)
    // Acceptance: Guardian invitation create URL
    #[test]
    fn req_class_013_guardian_invitation_create_url() {
        let url = build_guardian_invitation_create_url("student123");
        assert!(url.contains("/userProfiles/student123/guardianInvitations"));
    }

    // Requirement: REQ-CLASS-013 (Must)
    // Acceptance: Guardian invitation create body
    #[test]
    fn req_class_013_guardian_invitation_create_body() {
        let body = build_guardian_invitation_create_body("parent@example.com");
        assert_eq!(body["invitedEmailAddress"], "parent@example.com");
    }
}
