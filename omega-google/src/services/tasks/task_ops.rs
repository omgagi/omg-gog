//! Tasks task CRUD URL and body builders.

use super::TASKS_BASE_URL;

/// Build URL for listing tasks in a task list.
/// REQ-TASKS-003
pub fn build_tasks_list_url(
    tasklist_id: &str,
    max: Option<u32>,
    page_token: Option<&str>,
) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded_id = utf8_percent_encode(tasklist_id, NON_ALPHANUMERIC).to_string();
    let base = format!("{}/lists/{}/tasks", TASKS_BASE_URL, encoded_id);
    let mut params = Vec::new();
    if let Some(m) = max {
        params.push(format!("maxResults={}", m));
    }
    if let Some(token) = page_token {
        params.push(format!("pageToken={}", token));
    }
    if params.is_empty() {
        base
    } else {
        format!("{}?{}", base, params.join("&"))
    }
}

/// Build URL for getting a single task.
/// REQ-TASKS-004
pub fn build_task_get_url(tasklist_id: &str, task_id: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded_list = utf8_percent_encode(tasklist_id, NON_ALPHANUMERIC).to_string();
    let encoded_task = utf8_percent_encode(task_id, NON_ALPHANUMERIC).to_string();
    format!("{}/lists/{}/tasks/{}", TASKS_BASE_URL, encoded_list, encoded_task)
}

/// Build URL for creating a task.
/// REQ-TASKS-005
pub fn build_task_create_url(tasklist_id: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded_id = utf8_percent_encode(tasklist_id, NON_ALPHANUMERIC).to_string();
    format!("{}/lists/{}/tasks", TASKS_BASE_URL, encoded_id)
}

/// Build request body for creating a task.
/// REQ-TASKS-005
pub fn build_task_create_body(
    title: &str,
    notes: Option<&str>,
    due: Option<&str>,
    parent: Option<&str>,
    previous: Option<&str>,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "title": title,
    });
    if let Some(n) = notes {
        body["notes"] = serde_json::json!(n);
    }
    if let Some(d) = due {
        body["due"] = serde_json::json!(d);
    }
    if let Some(p) = parent {
        body["parent"] = serde_json::json!(p);
    }
    if let Some(prev) = previous {
        body["previous"] = serde_json::json!(prev);
    }
    body
}

/// Build URL for updating a task.
/// REQ-TASKS-006
pub fn build_task_update_url(tasklist_id: &str, task_id: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded_list = utf8_percent_encode(tasklist_id, NON_ALPHANUMERIC).to_string();
    let encoded_task = utf8_percent_encode(task_id, NON_ALPHANUMERIC).to_string();
    format!("{}/lists/{}/tasks/{}", TASKS_BASE_URL, encoded_list, encoded_task)
}

/// Build request body for updating a task.
/// REQ-TASKS-006
pub fn build_task_update_body(
    title: Option<&str>,
    notes: Option<&str>,
    due: Option<&str>,
    status: Option<&str>,
) -> serde_json::Value {
    let mut body = serde_json::json!({});
    if let Some(t) = title {
        body["title"] = serde_json::json!(t);
    }
    if let Some(n) = notes {
        body["notes"] = serde_json::json!(n);
    }
    if let Some(d) = due {
        body["due"] = serde_json::json!(d);
    }
    if let Some(s) = status {
        body["status"] = serde_json::json!(s);
    }
    body
}

/// Build URL for deleting a task.
/// REQ-TASKS-009
pub fn build_task_delete_url(tasklist_id: &str, task_id: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded_list = utf8_percent_encode(tasklist_id, NON_ALPHANUMERIC).to_string();
    let encoded_task = utf8_percent_encode(task_id, NON_ALPHANUMERIC).to_string();
    format!("{}/lists/{}/tasks/{}", TASKS_BASE_URL, encoded_list, encoded_task)
}

/// Build URL for clearing completed tasks in a task list.
/// REQ-TASKS-010
pub fn build_tasks_clear_url(tasklist_id: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded_id = utf8_percent_encode(tasklist_id, NON_ALPHANUMERIC).to_string();
    format!("{}/lists/{}/clear", TASKS_BASE_URL, encoded_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-TASKS-003 (Must): Tasks list URL
    // ---------------------------------------------------------------

    // Requirement: REQ-TASKS-003 (Must)
    // Acceptance: Tasks list URL with tasklist ID
    #[test]
    fn req_tasks_003_tasks_list_url_default() {
        let url = build_tasks_list_url("list123", None, None);
        assert!(url.contains("/lists/list123/tasks"));
    }

    // Requirement: REQ-TASKS-003 (Must)
    // Acceptance: Tasks list URL with max results
    #[test]
    fn req_tasks_003_tasks_list_url_max() {
        let url = build_tasks_list_url("list123", Some(50), None);
        assert!(url.contains("maxResults=50"));
    }

    // Requirement: REQ-TASKS-003 (Must)
    // Acceptance: Tasks list URL with page token
    #[test]
    fn req_tasks_003_tasks_list_url_page_token() {
        let url = build_tasks_list_url("list123", None, Some("token_abc"));
        assert!(url.contains("pageToken=token_abc"));
    }

    // Requirement: REQ-TASKS-003 (Must)
    // Acceptance: Tasks list URL with both parameters
    #[test]
    fn req_tasks_003_tasks_list_url_all_params() {
        let url = build_tasks_list_url("list123", Some(10), Some("p2"));
        assert!(url.contains("maxResults=10"));
        assert!(url.contains("pageToken=p2"));
    }

    // ---------------------------------------------------------------
    // REQ-TASKS-004 (Must): Task get URL
    // ---------------------------------------------------------------

    // Requirement: REQ-TASKS-004 (Must)
    // Acceptance: Task get URL
    #[test]
    fn req_tasks_004_task_get_url() {
        let url = build_task_get_url("list123", "task456");
        assert!(url.contains("/lists/list123/tasks/task456"));
    }

    // Requirement: REQ-TASKS-004 (Must)
    // Edge case: Task get URL with special characters
    #[test]
    fn req_tasks_004_task_get_url_special_chars() {
        let url = build_task_get_url("list with spaces", "task/id");
        assert!(url.contains("/lists/"));
        assert!(url.contains("/tasks/"));
    }

    // ---------------------------------------------------------------
    // REQ-TASKS-005 (Must): Task creation
    // ---------------------------------------------------------------

    // Requirement: REQ-TASKS-005 (Must)
    // Acceptance: Task create URL
    #[test]
    fn req_tasks_005_task_create_url() {
        let url = build_task_create_url("list123");
        assert!(url.contains("/lists/list123/tasks"));
    }

    // Requirement: REQ-TASKS-005 (Must)
    // Acceptance: Task create body with title only
    #[test]
    fn req_tasks_005_task_create_body_minimal() {
        let body = build_task_create_body("Buy milk", None, None, None, None);
        assert_eq!(body["title"], "Buy milk");
        assert!(body.get("notes").is_none());
        assert!(body.get("due").is_none());
        assert!(body.get("parent").is_none());
        assert!(body.get("previous").is_none());
    }

    // Requirement: REQ-TASKS-005 (Must)
    // Acceptance: Task create body with all fields
    #[test]
    fn req_tasks_005_task_create_body_all_fields() {
        let body = build_task_create_body(
            "Shopping",
            Some("Get items from the store"),
            Some("2024-01-20T00:00:00.000Z"),
            Some("parent_task_id"),
            Some("prev_task_id"),
        );
        assert_eq!(body["title"], "Shopping");
        assert_eq!(body["notes"], "Get items from the store");
        assert_eq!(body["due"], "2024-01-20T00:00:00.000Z");
        assert_eq!(body["parent"], "parent_task_id");
        assert_eq!(body["previous"], "prev_task_id");
    }

    // ---------------------------------------------------------------
    // REQ-TASKS-006 (Must): Task update
    // ---------------------------------------------------------------

    // Requirement: REQ-TASKS-006 (Must)
    // Acceptance: Task update URL
    #[test]
    fn req_tasks_006_task_update_url() {
        let url = build_task_update_url("list123", "task456");
        assert!(url.contains("/lists/list123/tasks/task456"));
    }

    // Requirement: REQ-TASKS-006 (Must)
    // Acceptance: Task update body with title only
    #[test]
    fn req_tasks_006_task_update_body_title_only() {
        let body = build_task_update_body(Some("New Title"), None, None, None);
        assert_eq!(body["title"], "New Title");
        assert!(body.get("notes").is_none());
        assert!(body.get("due").is_none());
        assert!(body.get("status").is_none());
    }

    // Requirement: REQ-TASKS-006 (Must)
    // Acceptance: Task update body with all fields
    #[test]
    fn req_tasks_006_task_update_body_all_fields() {
        let body = build_task_update_body(
            Some("Updated"),
            Some("New notes"),
            Some("2024-02-01T00:00:00.000Z"),
            Some("completed"),
        );
        assert_eq!(body["title"], "Updated");
        assert_eq!(body["notes"], "New notes");
        assert_eq!(body["due"], "2024-02-01T00:00:00.000Z");
        assert_eq!(body["status"], "completed");
    }

    // Requirement: REQ-TASKS-006 (Must)
    // Edge case: Task update body with empty (no fields)
    #[test]
    fn req_tasks_006_task_update_body_empty() {
        let body = build_task_update_body(None, None, None, None);
        let obj = body.as_object().unwrap();
        assert!(obj.is_empty());
    }

    // ---------------------------------------------------------------
    // REQ-TASKS-007 (Must): Task done (mark completed)
    // ---------------------------------------------------------------

    // Requirement: REQ-TASKS-007 (Must)
    // Acceptance: Task done uses update body with status=completed
    #[test]
    fn req_tasks_007_task_done_body() {
        let body = build_task_update_body(None, None, None, Some("completed"));
        assert_eq!(body["status"], "completed");
    }

    // ---------------------------------------------------------------
    // REQ-TASKS-008 (Must): Task undo (mark needsAction)
    // ---------------------------------------------------------------

    // Requirement: REQ-TASKS-008 (Must)
    // Acceptance: Task undo uses update body with status=needsAction
    #[test]
    fn req_tasks_008_task_undo_body() {
        let body = build_task_update_body(None, None, None, Some("needsAction"));
        assert_eq!(body["status"], "needsAction");
    }

    // ---------------------------------------------------------------
    // REQ-TASKS-009 (Must): Task delete URL
    // ---------------------------------------------------------------

    // Requirement: REQ-TASKS-009 (Must)
    // Acceptance: Task delete URL
    #[test]
    fn req_tasks_009_task_delete_url() {
        let url = build_task_delete_url("list123", "task456");
        assert!(url.contains("/lists/list123/tasks/task456"));
    }

    // Requirement: REQ-TASKS-009 (Must)
    // Edge case: Task delete URL matches get URL (same endpoint, different HTTP method)
    #[test]
    fn req_tasks_009_task_delete_url_matches_get() {
        let delete_url = build_task_delete_url("list1", "task1");
        let get_url = build_task_get_url("list1", "task1");
        assert_eq!(delete_url, get_url);
    }

    // ---------------------------------------------------------------
    // REQ-TASKS-010 (Must): Tasks clear URL
    // ---------------------------------------------------------------

    // Requirement: REQ-TASKS-010 (Must)
    // Acceptance: Tasks clear URL
    #[test]
    fn req_tasks_010_tasks_clear_url() {
        let url = build_tasks_clear_url("list123");
        assert!(url.contains("/lists/list123/clear"));
    }

    // Requirement: REQ-TASKS-010 (Must)
    // Edge case: Tasks clear URL with special characters
    #[test]
    fn req_tasks_010_tasks_clear_url_special_chars() {
        let url = build_tasks_clear_url("list/special");
        assert!(url.contains("/lists/"));
        assert!(url.contains("/clear"));
    }
}
