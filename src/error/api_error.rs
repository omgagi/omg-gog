/// Parse a Google API error JSON body into a user-friendly message.
/// Google returns errors like: {"error": {"code": 404, "message": "..."}}
pub fn format_api_error(status: u16, body: &str) -> String {
    if let Some(msg) = parse_google_error(body) {
        format!("API error ({}): {}", status, msg)
    } else {
        format!("API error ({}): {}", status, body)
    }
}

/// Extract the error message from a Google API error response body.
pub fn parse_google_error(body: &str) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(body).ok()?;
    let error_obj = parsed.get("error")?;
    let message = error_obj.get("message")?.as_str()?;
    Some(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-UI-003 (Must): Error formatting with Google API errors
    // ---------------------------------------------------------------

    // Requirement: REQ-UI-003 (Must)
    // Acceptance: Google API errors include helpful messages
    #[test]
    fn req_ui_003_parses_google_api_error() {
        let body = r#"{"error": {"code": 404, "message": "Requested entity was not found."}}"#;
        let msg = parse_google_error(body);
        assert_eq!(msg, Some("Requested entity was not found.".to_string()));
    }

    // Requirement: REQ-UI-003 (Must)
    // Acceptance: Handles 401 with re-auth guidance
    #[test]
    fn req_ui_003_format_401_error() {
        let body = r#"{"error": {"code": 401, "message": "Request had invalid authentication credentials."}}"#;
        let formatted = format_api_error(401, body);
        assert!(formatted.contains("401"));
        // Should mention re-authentication
    }

    // Requirement: REQ-UI-003 (Must)
    // Acceptance: Handles 403 with permission guidance
    #[test]
    fn req_ui_003_format_403_error() {
        let body = r#"{"error": {"code": 403, "message": "The caller does not have permission"}}"#;
        let formatted = format_api_error(403, body);
        assert!(formatted.contains("403"));
    }

    // Requirement: REQ-UI-003 (Must)
    // Acceptance: Handles 404 with not found guidance
    #[test]
    fn req_ui_003_format_404_error() {
        let body = r#"{"error": {"code": 404, "message": "File not found: abc123"}}"#;
        let formatted = format_api_error(404, body);
        assert!(formatted.contains("404"));
        assert!(formatted.contains("File not found"));
    }

    // Requirement: REQ-UI-003 (Must)
    // Edge case: Non-JSON error body
    #[test]
    fn req_ui_003_non_json_error_body() {
        let body = "Internal Server Error";
        let msg = parse_google_error(body);
        assert_eq!(msg, None, "Non-JSON should return None");
    }

    // Requirement: REQ-UI-003 (Must)
    // Edge case: Empty error body
    #[test]
    fn req_ui_003_empty_error_body() {
        let msg = parse_google_error("");
        assert_eq!(msg, None);
    }

    // Requirement: REQ-UI-003 (Must)
    // Edge case: Nested error details
    #[test]
    fn req_ui_003_nested_error_details() {
        let body = r#"{"error": {"code": 400, "message": "Invalid value", "errors": [{"message": "detail", "reason": "invalid"}]}}"#;
        let msg = parse_google_error(body);
        assert!(msg.is_some());
    }
}
