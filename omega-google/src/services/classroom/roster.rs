//! Classroom roster (students and teachers) URL and body builders.

use super::CLASSROOM_BASE_URL;

// ---------------------------------------------------------------
// Student URL builders
// ---------------------------------------------------------------

/// Build URL for listing students in a course.
pub fn build_students_list_url(
    course_id: &str,
    max: Option<u32>,
    page_token: Option<&str>,
) -> String {
    let base = format!(
        "{}/courses/{}/students",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        )
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

/// Build URL for adding a student to a course.
pub fn build_student_add_url(course_id: &str) -> String {
    format!(
        "{}/courses/{}/students",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

/// Build request body for adding a student.
pub fn build_student_add_body(
    user_id: &str,
    enrollment_code: Option<&str>,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "userId": user_id,
    });
    if let Some(code) = enrollment_code {
        body["enrollmentCode"] = serde_json::Value::String(code.to_string());
    }
    body
}

/// Build URL for getting a student in a course.
pub fn build_student_get_url(course_id: &str, user_id: &str) -> String {
    build_student_remove_url(course_id, user_id) // Same URL pattern
}

/// Build URL for removing a student from a course.
pub fn build_student_remove_url(course_id: &str, user_id: &str) -> String {
    format!(
        "{}/courses/{}/students/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        ),
        percent_encoding::utf8_percent_encode(
            user_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

// ---------------------------------------------------------------
// Teacher URL builders
// ---------------------------------------------------------------

/// Build URL for listing teachers in a course.
pub fn build_teachers_list_url(
    course_id: &str,
    max: Option<u32>,
    page_token: Option<&str>,
) -> String {
    let base = format!(
        "{}/courses/{}/teachers",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        )
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

/// Build URL for adding a teacher to a course.
pub fn build_teacher_add_url(course_id: &str) -> String {
    format!(
        "{}/courses/{}/teachers",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

/// Build request body for adding a teacher.
pub fn build_teacher_add_body(user_id: &str) -> serde_json::Value {
    serde_json::json!({
        "userId": user_id,
    })
}

/// Build URL for getting a teacher in a course.
pub fn build_teacher_get_url(course_id: &str, user_id: &str) -> String {
    build_teacher_remove_url(course_id, user_id) // Same URL pattern
}

/// Build URL for removing a teacher from a course.
pub fn build_teacher_remove_url(course_id: &str, user_id: &str) -> String {
    format!(
        "{}/courses/{}/teachers/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        ),
        percent_encoding::utf8_percent_encode(
            user_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CLASS-002 (Must): Student roster URL builders
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-002 (Must)
    // Acceptance: Students list URL without params
    #[test]
    fn req_class_002_students_list_url_no_params() {
        let url = build_students_list_url("course123", None, None);
        assert!(url.contains("/courses/course123/students"));
        assert!(!url.contains("?"));
    }

    // Requirement: REQ-CLASS-002 (Must)
    // Acceptance: Students list URL with max and page token
    #[test]
    fn req_class_002_students_list_url_with_params() {
        let url = build_students_list_url("course123", Some(30), Some("nextPage"));
        assert!(url.contains("pageSize=30"));
        assert!(url.contains("pageToken=nextPage"));
    }

    // Requirement: REQ-CLASS-002 (Must)
    // Acceptance: Student add URL
    #[test]
    fn req_class_002_student_add_url() {
        let url = build_student_add_url("course123");
        assert!(url.contains("/courses/course123/students"));
    }

    // Requirement: REQ-CLASS-002 (Must)
    // Acceptance: Student add body
    #[test]
    fn req_class_002_student_add_body() {
        let body = build_student_add_body("student@example.com", Some("enroll123"));
        assert_eq!(body["userId"], "student@example.com");
        assert_eq!(body["enrollmentCode"], "enroll123");
    }

    // Requirement: REQ-CLASS-002 (Must)
    // Acceptance: Student add body without enrollment code
    #[test]
    fn req_class_002_student_add_body_no_code() {
        let body = build_student_add_body("student@example.com", None);
        assert_eq!(body["userId"], "student@example.com");
        assert!(body.get("enrollmentCode").is_none());
    }

    // Requirement: REQ-CLASS-002 (Must)
    // Acceptance: Student remove URL
    #[test]
    fn req_class_002_student_remove_url() {
        let url = build_student_remove_url("course123", "student456");
        assert!(url.contains("/courses/course123/students/student456"));
    }

    // ---------------------------------------------------------------
    // REQ-CLASS-003 (Must): Teacher roster URL builders
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-003 (Must)
    // Acceptance: Teachers list URL without params
    #[test]
    fn req_class_003_teachers_list_url_no_params() {
        let url = build_teachers_list_url("course123", None, None);
        assert!(url.contains("/courses/course123/teachers"));
        assert!(!url.contains("?"));
    }

    // Requirement: REQ-CLASS-003 (Must)
    // Acceptance: Teachers list URL with max and page token
    #[test]
    fn req_class_003_teachers_list_url_with_params() {
        let url = build_teachers_list_url("course123", Some(10), Some("page2"));
        assert!(url.contains("pageSize=10"));
        assert!(url.contains("pageToken=page2"));
    }

    // Requirement: REQ-CLASS-003 (Must)
    // Acceptance: Teacher add URL
    #[test]
    fn req_class_003_teacher_add_url() {
        let url = build_teacher_add_url("course123");
        assert!(url.contains("/courses/course123/teachers"));
    }

    // Requirement: REQ-CLASS-003 (Must)
    // Acceptance: Teacher add body
    #[test]
    fn req_class_003_teacher_add_body() {
        let body = build_teacher_add_body("teacher@example.com");
        assert_eq!(body["userId"], "teacher@example.com");
    }

    // Requirement: REQ-CLASS-004 (Must)
    // Acceptance: Teacher remove URL
    #[test]
    fn req_class_004_teacher_remove_url() {
        let url = build_teacher_remove_url("course123", "teacher456");
        assert!(url.contains("/courses/course123/teachers/teacher456"));
    }
}
