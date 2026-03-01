//! Classroom student submissions URL and body builders.

use super::CLASSROOM_BASE_URL;

/// Build URL for listing student submissions.
pub fn build_submissions_list_url(
    course_id: &str,
    coursework_id: &str,
    max: Option<u32>,
    page_token: Option<&str>,
) -> String {
    let base = format!(
        "{}/courses/{}/courseWork/{}/studentSubmissions",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        ),
        percent_encoding::utf8_percent_encode(
            coursework_id,
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

/// Build URL for getting a single submission.
pub fn build_submission_get_url(
    course_id: &str,
    coursework_id: &str,
    submission_id: &str,
) -> String {
    format!(
        "{}/courses/{}/courseWork/{}/studentSubmissions/{}",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        ),
        percent_encoding::utf8_percent_encode(
            coursework_id,
            percent_encoding::NON_ALPHANUMERIC
        ),
        percent_encoding::utf8_percent_encode(
            submission_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

/// Build URL for turning in a submission.
pub fn build_submission_turn_in_url(
    course_id: &str,
    coursework_id: &str,
    submission_id: &str,
) -> String {
    format!(
        "{}/courses/{}/courseWork/{}/studentSubmissions/{}:turnIn",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        ),
        percent_encoding::utf8_percent_encode(
            coursework_id,
            percent_encoding::NON_ALPHANUMERIC
        ),
        percent_encoding::utf8_percent_encode(
            submission_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

/// Build URL for reclaiming a submission.
pub fn build_submission_reclaim_url(
    course_id: &str,
    coursework_id: &str,
    submission_id: &str,
) -> String {
    format!(
        "{}/courses/{}/courseWork/{}/studentSubmissions/{}:reclaim",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        ),
        percent_encoding::utf8_percent_encode(
            coursework_id,
            percent_encoding::NON_ALPHANUMERIC
        ),
        percent_encoding::utf8_percent_encode(
            submission_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

/// Build URL for returning a submission.
pub fn build_submission_return_url(
    course_id: &str,
    coursework_id: &str,
    submission_id: &str,
) -> String {
    format!(
        "{}/courses/{}/courseWork/{}/studentSubmissions/{}:return",
        CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(
            course_id,
            percent_encoding::NON_ALPHANUMERIC
        ),
        percent_encoding::utf8_percent_encode(
            coursework_id,
            percent_encoding::NON_ALPHANUMERIC
        ),
        percent_encoding::utf8_percent_encode(
            submission_id,
            percent_encoding::NON_ALPHANUMERIC
        )
    )
}

/// Build request body for grading a submission.
pub fn build_submission_grade_body(
    assigned_grade: Option<f64>,
    draft_grade: Option<f64>,
) -> serde_json::Value {
    let mut body = serde_json::json!({});
    if let Some(g) = assigned_grade {
        body["assignedGrade"] = serde_json::json!(g);
    }
    if let Some(g) = draft_grade {
        body["draftGrade"] = serde_json::json!(g);
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CLASS-007 (Must): Submissions URL builders
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-007 (Must)
    // Acceptance: Submissions list URL without params
    #[test]
    fn req_class_007_submissions_list_url_no_params() {
        let url = build_submissions_list_url("course1", "cw1", None, None);
        assert!(url.contains("/courses/course1/courseWork/cw1/studentSubmissions"));
        assert!(!url.contains("?"));
    }

    // Requirement: REQ-CLASS-007 (Must)
    // Acceptance: Submissions list URL with params
    #[test]
    fn req_class_007_submissions_list_url_with_params() {
        let url = build_submissions_list_url("course1", "cw1", Some(25), Some("pg2"));
        assert!(url.contains("pageSize=25"));
        assert!(url.contains("pageToken=pg2"));
    }

    // Requirement: REQ-CLASS-007 (Must)
    // Acceptance: Submission get URL
    #[test]
    fn req_class_007_submission_get_url() {
        let url = build_submission_get_url("course1", "cw1", "sub1");
        assert!(url.contains("/courses/course1/courseWork/cw1/studentSubmissions/sub1"));
    }

    // Requirement: REQ-CLASS-007 (Must)
    // Acceptance: Submission turn in URL
    #[test]
    fn req_class_007_submission_turn_in_url() {
        let url = build_submission_turn_in_url("course1", "cw1", "sub1");
        assert!(url.contains("/studentSubmissions/sub1%3AturnIn") || url.contains("/studentSubmissions/sub1:turnIn"));
    }

    // Requirement: REQ-CLASS-007 (Must)
    // Acceptance: Submission reclaim URL
    #[test]
    fn req_class_007_submission_reclaim_url() {
        let url = build_submission_reclaim_url("course1", "cw1", "sub1");
        assert!(url.contains("/studentSubmissions/sub1%3Areclaim") || url.contains("/studentSubmissions/sub1:reclaim"));
    }

    // Requirement: REQ-CLASS-007 (Must)
    // Acceptance: Submission return URL
    #[test]
    fn req_class_007_submission_return_url() {
        let url = build_submission_return_url("course1", "cw1", "sub1");
        assert!(url.contains("/studentSubmissions/sub1%3Areturn") || url.contains("/studentSubmissions/sub1:return"));
    }

    // Requirement: REQ-CLASS-007 (Must)
    // Acceptance: Grade body with both grades
    #[test]
    fn req_class_007_grade_body_both() {
        let body = build_submission_grade_body(Some(95.0), Some(90.0));
        assert_eq!(body["assignedGrade"], 95.0);
        assert_eq!(body["draftGrade"], 90.0);
    }

    // Requirement: REQ-CLASS-007 (Must)
    // Acceptance: Grade body with only assigned grade
    #[test]
    fn req_class_007_grade_body_assigned_only() {
        let body = build_submission_grade_body(Some(85.0), None);
        assert_eq!(body["assignedGrade"], 85.0);
        assert!(body.get("draftGrade").is_none());
    }
}
