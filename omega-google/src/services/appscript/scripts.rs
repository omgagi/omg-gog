//! Apps Script URL and body builders.

use super::SCRIPT_BASE_URL;

/// Build URL for getting a project.
/// REQ-SCRIPT-001
pub fn build_project_get_url(script_id: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded = utf8_percent_encode(script_id, NON_ALPHANUMERIC).to_string();
    format!("{}/projects/{}", SCRIPT_BASE_URL, encoded)
}

/// Build URL for getting project content (source files).
/// REQ-SCRIPT-002
pub fn build_content_get_url(script_id: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded = utf8_percent_encode(script_id, NON_ALPHANUMERIC).to_string();
    format!("{}/projects/{}/content", SCRIPT_BASE_URL, encoded)
}

/// Build URL for running a script function.
/// REQ-SCRIPT-003
pub fn build_run_url(script_id: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded = utf8_percent_encode(script_id, NON_ALPHANUMERIC).to_string();
    format!("{}/scripts/{}:run", SCRIPT_BASE_URL, encoded)
}

/// Build request body for running a script function.
/// REQ-SCRIPT-003
pub fn build_run_body(
    function: &str,
    params: Option<&str>,
    dev_mode: bool,
) -> Result<serde_json::Value, String> {
    let mut body = serde_json::json!({
        "function": function,
        "devMode": dev_mode,
    });

    if let Some(params_str) = params {
        let parsed: serde_json::Value = serde_json::from_str(params_str)
            .map_err(|e| format!("Invalid JSON parameters: {}", e))?;
        if let Some(arr) = parsed.as_array() {
            body["parameters"] = serde_json::json!(arr);
        } else {
            // Wrap non-array values in an array
            body["parameters"] = serde_json::json!([parsed]);
        }
    }

    Ok(body)
}

/// Build URL for creating a new project.
/// REQ-SCRIPT-004
pub fn build_project_create_url() -> String {
    format!("{}/projects", SCRIPT_BASE_URL)
}

/// Build request body for creating a new project.
/// REQ-SCRIPT-004
pub fn build_project_create_body(title: &str, parent_id: Option<&str>) -> serde_json::Value {
    let mut body = serde_json::json!({
        "title": title,
    });
    if let Some(pid) = parent_id {
        body["parentId"] = serde_json::json!(pid);
    }
    body
}

/// Normalize a Google ID: extract the ID from URLs like
/// `https://docs.google.com/spreadsheets/d/SPREADSHEET_ID/edit` or
/// `https://script.google.com/d/SCRIPT_ID/edit`.
/// If input is already a bare ID, return as-is.
/// REQ-SCRIPT-004
pub fn normalize_google_id(input: &str) -> String {
    // Try to extract ID from URL patterns:
    // https://docs.google.com/*/d/ID/...
    // https://script.google.com/*/d/ID/...
    // https://drive.google.com/file/d/ID/...
    if let Some(pos) = input.find("/d/") {
        let after = &input[pos + 3..];
        if let Some(end) = after.find('/') {
            return after[..end].to_string();
        }
        return after.to_string();
    }

    // Already a bare ID
    input.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-SCRIPT-001 (Must): Project get URL
    // ---------------------------------------------------------------

    // Requirement: REQ-SCRIPT-001 (Must)
    // Acceptance: Project get URL
    #[test]
    fn req_script_001_project_get_url() {
        // REQ-SCRIPT-001
        let url = build_project_get_url("abc123");
        assert_eq!(url, "https://script.googleapis.com/v1/projects/abc123");
    }

    // Requirement: REQ-SCRIPT-001 (Must)
    // Acceptance: Project get URL with special characters
    #[test]
    fn req_script_001_project_get_url_special_chars() {
        // REQ-SCRIPT-001
        let url = build_project_get_url("abc 123");
        assert!(url.starts_with("https://script.googleapis.com/v1/projects/"));
        assert!(!url.contains(' '));
    }

    // ---------------------------------------------------------------
    // REQ-SCRIPT-002 (Must): Content get URL
    // ---------------------------------------------------------------

    // Requirement: REQ-SCRIPT-002 (Must)
    // Acceptance: Content get URL
    #[test]
    fn req_script_002_content_get_url() {
        // REQ-SCRIPT-002
        let url = build_content_get_url("abc123");
        assert_eq!(url, "https://script.googleapis.com/v1/projects/abc123/content");
    }

    // ---------------------------------------------------------------
    // REQ-SCRIPT-003 (Must): Run URL and body
    // ---------------------------------------------------------------

    // Requirement: REQ-SCRIPT-003 (Must)
    // Acceptance: Run URL
    #[test]
    fn req_script_003_run_url() {
        // REQ-SCRIPT-003
        let url = build_run_url("abc123");
        assert_eq!(url, "https://script.googleapis.com/v1/scripts/abc123:run");
    }

    // Requirement: REQ-SCRIPT-003 (Must)
    // Acceptance: Run body without parameters
    #[test]
    fn req_script_003_run_body_no_params() {
        // REQ-SCRIPT-003
        let body = build_run_body("myFunction", None, false).unwrap();
        assert_eq!(body["function"], "myFunction");
        assert_eq!(body["devMode"], false);
        assert!(body.get("parameters").is_none());
    }

    // Requirement: REQ-SCRIPT-003 (Must)
    // Acceptance: Run body with parameters array
    #[test]
    fn req_script_003_run_body_with_params_array() {
        // REQ-SCRIPT-003
        let body = build_run_body("myFunction", Some(r#"["hello", 42]"#), true).unwrap();
        assert_eq!(body["function"], "myFunction");
        assert_eq!(body["devMode"], true);
        let params = body["parameters"].as_array().unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], "hello");
        assert_eq!(params[1], 42);
    }

    // Requirement: REQ-SCRIPT-003 (Must)
    // Acceptance: Run body with single parameter wraps in array
    #[test]
    fn req_script_003_run_body_with_single_param() {
        // REQ-SCRIPT-003
        let body = build_run_body("myFunction", Some(r#""hello""#), false).unwrap();
        let params = body["parameters"].as_array().unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], "hello");
    }

    // Requirement: REQ-SCRIPT-003 (Must)
    // Acceptance: Run body with invalid JSON returns error
    #[test]
    fn req_script_003_run_body_invalid_json() {
        // REQ-SCRIPT-003
        let result = build_run_body("myFunction", Some("not valid json {"), false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    // ---------------------------------------------------------------
    // REQ-SCRIPT-004 (Must): Project create URL and body
    // ---------------------------------------------------------------

    // Requirement: REQ-SCRIPT-004 (Must)
    // Acceptance: Project create URL
    #[test]
    fn req_script_004_project_create_url() {
        // REQ-SCRIPT-004
        let url = build_project_create_url();
        assert_eq!(url, "https://script.googleapis.com/v1/projects");
    }

    // Requirement: REQ-SCRIPT-004 (Must)
    // Acceptance: Project create body without parent
    #[test]
    fn req_script_004_project_create_body_no_parent() {
        // REQ-SCRIPT-004
        let body = build_project_create_body("My Script", None);
        assert_eq!(body["title"], "My Script");
        assert!(body.get("parentId").is_none());
    }

    // Requirement: REQ-SCRIPT-004 (Must)
    // Acceptance: Project create body with parent
    #[test]
    fn req_script_004_project_create_body_with_parent() {
        // REQ-SCRIPT-004
        let body = build_project_create_body("My Script", Some("parent_doc_id"));
        assert_eq!(body["title"], "My Script");
        assert_eq!(body["parentId"], "parent_doc_id");
    }

    // ---------------------------------------------------------------
    // REQ-SCRIPT-004 (Must): normalize_google_id
    // ---------------------------------------------------------------

    // Requirement: REQ-SCRIPT-004 (Must)
    // Acceptance: Bare ID passes through
    #[test]
    fn req_script_004_normalize_bare_id() {
        // REQ-SCRIPT-004
        assert_eq!(normalize_google_id("abc123def456"), "abc123def456");
    }

    // Requirement: REQ-SCRIPT-004 (Must)
    // Acceptance: Script URL extracts ID
    #[test]
    fn req_script_004_normalize_script_url() {
        // REQ-SCRIPT-004
        let url = "https://script.google.com/d/abc123def456/edit";
        assert_eq!(normalize_google_id(url), "abc123def456");
    }

    // Requirement: REQ-SCRIPT-004 (Must)
    // Acceptance: Spreadsheet URL extracts ID
    #[test]
    fn req_script_004_normalize_spreadsheet_url() {
        // REQ-SCRIPT-004
        let url = "https://docs.google.com/spreadsheets/d/SHEET_ID_123/edit#gid=0";
        assert_eq!(normalize_google_id(url), "SHEET_ID_123");
    }

    // Requirement: REQ-SCRIPT-004 (Must)
    // Acceptance: Drive file URL extracts ID
    #[test]
    fn req_script_004_normalize_drive_url() {
        // REQ-SCRIPT-004
        let url = "https://drive.google.com/file/d/FILE_ID_456/view";
        assert_eq!(normalize_google_id(url), "FILE_ID_456");
    }

    // Requirement: REQ-SCRIPT-004 (Must)
    // Acceptance: URL ending with /d/ID (no trailing slash) extracts ID
    #[test]
    fn req_script_004_normalize_url_no_trailing() {
        // REQ-SCRIPT-004
        let url = "https://script.google.com/d/abc123";
        assert_eq!(normalize_google_id(url), "abc123");
    }
}
