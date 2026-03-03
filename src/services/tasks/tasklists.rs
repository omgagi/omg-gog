//! Tasks tasklist URL and body builders.

use super::TASKS_BASE_URL;

/// Build URL for listing task lists.
/// REQ-TASKS-001
pub fn build_tasklists_list_url(max: Option<u32>, page_token: Option<&str>) -> String {
    let base = format!("{}/users/@me/lists", TASKS_BASE_URL);
    let mut params = Vec::new();
    if let Some(m) = max {
        params.push(format!("maxResults={}", m));
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

/// Build URL for creating a task list.
/// REQ-TASKS-002
pub fn build_tasklist_create_url() -> String {
    format!("{}/users/@me/lists", TASKS_BASE_URL)
}

/// Build request body for creating a task list.
/// REQ-TASKS-002
pub fn build_tasklist_create_body(title: &str) -> serde_json::Value {
    serde_json::json!({
        "title": title,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-TASKS-001 (Must): Tasklists list URL
    // ---------------------------------------------------------------

    // Requirement: REQ-TASKS-001 (Must)
    // Acceptance: Tasklists list URL with no parameters
    #[test]
    fn req_tasks_001_tasklists_list_url_default() {
        let url = build_tasklists_list_url(None, None);
        assert_eq!(url, "https://tasks.googleapis.com/tasks/v1/users/@me/lists");
    }

    // Requirement: REQ-TASKS-001 (Must)
    // Acceptance: Tasklists list URL with max results
    #[test]
    fn req_tasks_001_tasklists_list_url_max() {
        let url = build_tasklists_list_url(Some(10), None);
        assert!(url.contains("maxResults=10"));
    }

    // Requirement: REQ-TASKS-001 (Must)
    // Acceptance: Tasklists list URL with page token
    #[test]
    fn req_tasks_001_tasklists_list_url_page_token() {
        let url = build_tasklists_list_url(None, Some("abc123"));
        assert!(url.contains("pageToken=abc123"));
    }

    // Requirement: REQ-TASKS-001 (Must)
    // Acceptance: Tasklists list URL with both parameters
    #[test]
    fn req_tasks_001_tasklists_list_url_all_params() {
        let url = build_tasklists_list_url(Some(20), Some("token"));
        assert!(url.contains("maxResults=20"));
        assert!(url.contains("pageToken=token"));
    }

    // ---------------------------------------------------------------
    // REQ-TASKS-002 (Must): Tasklist creation
    // ---------------------------------------------------------------

    // Requirement: REQ-TASKS-002 (Must)
    // Acceptance: Tasklist create URL
    #[test]
    fn req_tasks_002_tasklist_create_url() {
        let url = build_tasklist_create_url();
        assert_eq!(url, "https://tasks.googleapis.com/tasks/v1/users/@me/lists");
    }

    // Requirement: REQ-TASKS-002 (Must)
    // Acceptance: Tasklist create body has title
    #[test]
    fn req_tasks_002_tasklist_create_body() {
        let body = build_tasklist_create_body("Shopping");
        assert_eq!(body["title"], "Shopping");
    }

    // Requirement: REQ-TASKS-002 (Must)
    // Edge case: Tasklist create body with empty title
    #[test]
    fn req_tasks_002_tasklist_create_body_empty_title() {
        let body = build_tasklist_create_body("");
        assert_eq!(body["title"], "");
    }
}
