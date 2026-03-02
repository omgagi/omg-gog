//! Google Tasks API request/response types.
//! All types use camelCase serde rename and flatten for forward compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------
// TaskList types
// ---------------------------------------------------------------

/// A Google Tasks task list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskList {
    pub id: Option<String>,
    pub title: Option<String>,
    pub updated: Option<String>,
    pub self_link: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response for listing task lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskListsResponse {
    #[serde(default)]
    pub items: Vec<TaskList>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Task types
// ---------------------------------------------------------------

/// A Google Tasks task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub due: Option<String>,
    pub notes: Option<String>,
    pub completed: Option<String>,
    pub parent: Option<String>,
    pub position: Option<String>,
    #[serde(default)]
    pub links: Vec<TaskLink>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A link associated with a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLink {
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response for listing tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksResponse {
    #[serde(default)]
    pub items: Vec<Task>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---------------------------------------------------------------
    // REQ-TASKS-001 (Must): TaskList type deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-TASKS-001 (Must)
    // Acceptance: TaskList type deserializes from Tasks API JSON
    #[test]
    fn req_tasks_001_tasklist_deserialize() {
        let json_str = r#"{
            "id": "list123",
            "title": "My Tasks",
            "updated": "2024-01-15T10:00:00.000Z",
            "selfLink": "https://www.googleapis.com/tasks/v1/users/@me/lists/list123"
        }"#;
        let tl: TaskList = serde_json::from_str(json_str).unwrap();
        assert_eq!(tl.id, Some("list123".to_string()));
        assert_eq!(tl.title, Some("My Tasks".to_string()));
        assert_eq!(tl.updated, Some("2024-01-15T10:00:00.000Z".to_string()));
        assert!(tl.self_link.is_some());
    }

    // Requirement: REQ-TASKS-001 (Must)
    // Acceptance: TaskListsResponse deserializes with pagination
    #[test]
    fn req_tasks_001_tasklists_response_deserialize() {
        let json_str = r#"{
            "items": [
                {"id": "list1", "title": "Work"},
                {"id": "list2", "title": "Personal"}
            ],
            "nextPageToken": "page2token"
        }"#;
        let resp: TaskListsResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.items.len(), 2);
        assert_eq!(resp.items[0].title, Some("Work".to_string()));
        assert_eq!(resp.next_page_token, Some("page2token".to_string()));
    }

    // Requirement: REQ-TASKS-001 (Must)
    // Edge case: TaskListsResponse with empty items
    #[test]
    fn req_tasks_001_tasklists_response_empty() {
        let json_str = r#"{}"#;
        let resp: TaskListsResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.items.is_empty());
        assert!(resp.next_page_token.is_none());
    }

    // Requirement: REQ-TASKS-001 (Must)
    // Edge case: TaskList with unknown fields preserved
    #[test]
    fn req_tasks_001_tasklist_unknown_fields_preserved() {
        let json_str = r#"{
            "id": "list1",
            "title": "Test",
            "futureField": 42
        }"#;
        let tl: TaskList = serde_json::from_str(json_str).unwrap();
        assert!(tl.extra.contains_key("futureField"));
        assert_eq!(tl.extra["futureField"], json!(42));
    }

    // Requirement: REQ-TASKS-001 (Must)
    // Acceptance: TaskList round-trip serialization
    #[test]
    fn req_tasks_001_tasklist_roundtrip() {
        let tl = TaskList {
            id: Some("list1".to_string()),
            title: Some("Work".to_string()),
            updated: Some("2024-01-15T10:00:00.000Z".to_string()),
            self_link: None,
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&tl).unwrap();
        let parsed: TaskList = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, Some("list1".to_string()));
        assert_eq!(parsed.title, Some("Work".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-TASKS-003 (Must): Task type deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-TASKS-003 (Must)
    // Acceptance: Task type deserializes from Tasks API JSON
    #[test]
    fn req_tasks_003_task_deserialize() {
        let json_str = r#"{
            "id": "task123",
            "title": "Buy groceries",
            "status": "needsAction",
            "due": "2024-01-20T00:00:00.000Z",
            "notes": "Milk, bread, eggs",
            "completed": null,
            "parent": null,
            "position": "00000000000000000001",
            "links": [
                {
                    "type": "email",
                    "description": "Related email",
                    "link": "https://mail.google.com/..."
                }
            ]
        }"#;
        let task: Task = serde_json::from_str(json_str).unwrap();
        assert_eq!(task.id, Some("task123".to_string()));
        assert_eq!(task.title, Some("Buy groceries".to_string()));
        assert_eq!(task.status, Some("needsAction".to_string()));
        assert_eq!(task.due, Some("2024-01-20T00:00:00.000Z".to_string()));
        assert_eq!(task.notes, Some("Milk, bread, eggs".to_string()));
        assert!(task.completed.is_none());
        assert_eq!(task.position, Some("00000000000000000001".to_string()));
        assert_eq!(task.links.len(), 1);
        assert_eq!(task.links[0].type_, Some("email".to_string()));
        assert_eq!(task.links[0].description, Some("Related email".to_string()));
    }

    // Requirement: REQ-TASKS-003 (Must)
    // Acceptance: TasksResponse deserializes with pagination
    #[test]
    fn req_tasks_003_tasks_response_deserialize() {
        let json_str = r#"{
            "items": [
                {"id": "t1", "title": "Task 1", "status": "needsAction"},
                {"id": "t2", "title": "Task 2", "status": "completed", "completed": "2024-01-15T12:00:00Z"}
            ],
            "nextPageToken": "next_tasks"
        }"#;
        let resp: TasksResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.items.len(), 2);
        assert_eq!(resp.items[0].title, Some("Task 1".to_string()));
        assert_eq!(resp.items[1].status, Some("completed".to_string()));
        assert_eq!(resp.next_page_token, Some("next_tasks".to_string()));
    }

    // Requirement: REQ-TASKS-003 (Must)
    // Edge case: TasksResponse with empty items
    #[test]
    fn req_tasks_003_tasks_response_empty() {
        let json_str = r#"{}"#;
        let resp: TasksResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.items.is_empty());
        assert!(resp.next_page_token.is_none());
    }

    // Requirement: REQ-TASKS-003 (Must)
    // Acceptance: Task round-trip serialization
    #[test]
    fn req_tasks_003_task_roundtrip() {
        let task = Task {
            id: Some("t1".to_string()),
            title: Some("Test Task".to_string()),
            status: Some("needsAction".to_string()),
            due: None,
            notes: Some("Some notes".to_string()),
            completed: None,
            parent: None,
            position: Some("00000000000000000001".to_string()),
            links: vec![],
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&task).unwrap();
        let parsed: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, Some("t1".to_string()));
        assert_eq!(parsed.title, Some("Test Task".to_string()));
    }

    // Requirement: REQ-TASKS-003 (Must)
    // Edge case: Task with unknown fields preserved
    #[test]
    fn req_tasks_003_task_unknown_fields_preserved() {
        let json_str = r#"{
            "id": "t1",
            "title": "Test",
            "newApiField": "value"
        }"#;
        let task: Task = serde_json::from_str(json_str).unwrap();
        assert!(task.extra.contains_key("newApiField"));
    }

    // ---------------------------------------------------------------
    // REQ-TASKS-003 (Must): TaskLink type
    // ---------------------------------------------------------------

    // Requirement: REQ-TASKS-003 (Must)
    // Acceptance: TaskLink "type" field renames correctly
    #[test]
    fn req_tasks_003_task_link_type_rename() {
        let json_str = r#"{
            "type": "email",
            "description": "Email link",
            "link": "https://example.com"
        }"#;
        let link: TaskLink = serde_json::from_str(json_str).unwrap();
        assert_eq!(link.type_, Some("email".to_string()));

        // Round-trip: should serialize back with "type"
        let serialized = serde_json::to_value(&link).unwrap();
        assert_eq!(serialized["type"], "email");
    }

    // Requirement: REQ-TASKS-003 (Must)
    // Acceptance: Task with empty links defaults correctly
    #[test]
    fn req_tasks_003_task_empty_links_default() {
        let json_str = r#"{
            "id": "t1",
            "title": "No Links"
        }"#;
        let task: Task = serde_json::from_str(json_str).unwrap();
        assert!(task.links.is_empty());
    }
}
