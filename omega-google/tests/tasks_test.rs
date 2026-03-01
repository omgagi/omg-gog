//! Tasks service integration tests.

use omega_google::services::tasks::types::*;
use omega_google::services::tasks::tasklists::*;
use omega_google::services::tasks::task_ops::*;

// ---------------------------------------------------------------
// REQ-TASKS-001 (Must): TaskList deserialization
// ---------------------------------------------------------------

// Requirement: REQ-TASKS-001 (Must)
// Acceptance: Full TaskListsResponse from a realistic Tasks API response
#[test]
fn req_tasks_001_integration_tasklist_from_api() {
    // REQ-TASKS-001
    let api_response = r#"{
        "kind": "tasks#taskLists",
        "etag": "\"etag_tasklists_v1\"",
        "items": [
            {
                "kind": "tasks#taskList",
                "id": "MTIzNDU2Nzg5MA",
                "title": "My Tasks",
                "updated": "2024-03-15T10:30:00.000Z",
                "selfLink": "https://www.googleapis.com/tasks/v1/users/@me/lists/MTIzNDU2Nzg5MA"
            },
            {
                "kind": "tasks#taskList",
                "id": "QWJjRGVmR2hJ",
                "title": "Work Projects",
                "updated": "2024-03-14T16:45:00.000Z",
                "selfLink": "https://www.googleapis.com/tasks/v1/users/@me/lists/QWJjRGVmR2hJ"
            },
            {
                "kind": "tasks#taskList",
                "id": "WFlaQUJDREVG",
                "title": "Shopping List",
                "updated": "2024-03-10T08:00:00.000Z",
                "selfLink": "https://www.googleapis.com/tasks/v1/users/@me/lists/WFlaQUJDREVG"
            }
        ],
        "nextPageToken": "tasklists_page2"
    }"#;

    let resp: TaskListsResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(resp.items.len(), 3);
    assert_eq!(resp.next_page_token, Some("tasklists_page2".to_string()));

    // First task list
    let tl1 = &resp.items[0];
    assert_eq!(tl1.id, Some("MTIzNDU2Nzg5MA".to_string()));
    assert_eq!(tl1.title, Some("My Tasks".to_string()));
    assert_eq!(tl1.updated, Some("2024-03-15T10:30:00.000Z".to_string()));
    assert!(tl1.self_link.as_ref().unwrap().contains("MTIzNDU2Nzg5MA"));

    // Second task list
    let tl2 = &resp.items[1];
    assert_eq!(tl2.title, Some("Work Projects".to_string()));

    // Third task list
    let tl3 = &resp.items[2];
    assert_eq!(tl3.title, Some("Shopping List".to_string()));

    // Unknown fields preserved via flatten
    assert!(tl1.extra.contains_key("kind"));
}

// ---------------------------------------------------------------
// REQ-TASKS-003 (Must): Task deserialization with all fields
// ---------------------------------------------------------------

// Requirement: REQ-TASKS-003 (Must)
// Acceptance: Full Task from a realistic Tasks API response with all fields
#[test]
fn req_tasks_003_integration_task_full_from_api() {
    // REQ-TASKS-003
    let api_response = r#"{
        "kind": "tasks#task",
        "id": "dGFzazAwMQ",
        "title": "Prepare Q1 quarterly report",
        "status": "needsAction",
        "due": "2024-03-31T00:00:00.000Z",
        "notes": "Include revenue breakdown by region, customer acquisition metrics, and YoY growth comparison. Check with finance team for final numbers.",
        "completed": null,
        "parent": null,
        "position": "00000000000000000001",
        "links": [
            {
                "type": "email",
                "description": "Email from CFO with requirements",
                "link": "https://mail.google.com/mail/u/0/#inbox/18e1234567890abc"
            },
            {
                "type": "related",
                "description": "Previous quarter report",
                "link": "https://docs.google.com/document/d/1abc/edit"
            }
        ],
        "updated": "2024-03-15T14:00:00.000Z",
        "selfLink": "https://www.googleapis.com/tasks/v1/lists/MTIzNDU2/tasks/dGFzazAwMQ",
        "hidden": false,
        "deleted": false
    }"#;

    let task: Task = serde_json::from_str(api_response).unwrap();

    assert_eq!(task.id, Some("dGFzazAwMQ".to_string()));
    assert_eq!(task.title, Some("Prepare Q1 quarterly report".to_string()));
    assert_eq!(task.status, Some("needsAction".to_string()));
    assert_eq!(task.due, Some("2024-03-31T00:00:00.000Z".to_string()));
    assert!(task.notes.as_ref().unwrap().contains("revenue breakdown by region"));
    assert!(task.completed.is_none());
    assert!(task.parent.is_none());
    assert_eq!(task.position, Some("00000000000000000001".to_string()));

    // Verify links
    assert_eq!(task.links.len(), 2);
    assert_eq!(task.links[0].type_, Some("email".to_string()));
    assert_eq!(task.links[0].description, Some("Email from CFO with requirements".to_string()));
    assert!(task.links[0].link.as_ref().unwrap().contains("mail.google.com"));
    assert_eq!(task.links[1].type_, Some("related".to_string()));
    assert!(task.links[1].link.as_ref().unwrap().contains("docs.google.com"));

    // Unknown fields preserved
    assert!(task.extra.contains_key("kind"));
    assert!(task.extra.contains_key("hidden"));
}

// ---------------------------------------------------------------
// REQ-TASKS-003 (Must): Task list response with mixed statuses
// ---------------------------------------------------------------

// Requirement: REQ-TASKS-003 (Must)
// Acceptance: TasksResponse with completed and pending tasks
#[test]
fn req_tasks_003_integration_tasks_response_from_api() {
    // REQ-TASKS-003
    let api_response = r#"{
        "kind": "tasks#tasks",
        "etag": "\"etag_tasks_v2\"",
        "items": [
            {
                "id": "task001",
                "title": "Buy groceries",
                "status": "needsAction",
                "due": "2024-03-16T00:00:00.000Z",
                "notes": "Milk, bread, eggs, vegetables",
                "position": "00000000000000000001",
                "links": []
            },
            {
                "id": "task002",
                "title": "Call dentist",
                "status": "completed",
                "completed": "2024-03-14T10:30:00.000Z",
                "position": "00000000000000000002",
                "links": []
            },
            {
                "id": "task003",
                "title": "Review PR #42",
                "status": "needsAction",
                "parent": "task_parent_001",
                "notes": "Focus on error handling changes",
                "position": "00000000000000000003",
                "links": [
                    {"type": "related", "description": "PR link", "link": "https://github.com/org/repo/pull/42"}
                ]
            }
        ],
        "nextPageToken": "tasks_next_page"
    }"#;

    let resp: TasksResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(resp.items.len(), 3);
    assert_eq!(resp.next_page_token, Some("tasks_next_page".to_string()));

    // First: pending task with notes
    let t1 = &resp.items[0];
    assert_eq!(t1.title, Some("Buy groceries".to_string()));
    assert_eq!(t1.status, Some("needsAction".to_string()));
    assert_eq!(t1.due, Some("2024-03-16T00:00:00.000Z".to_string()));
    assert!(t1.completed.is_none());

    // Second: completed task
    let t2 = &resp.items[1];
    assert_eq!(t2.status, Some("completed".to_string()));
    assert_eq!(t2.completed, Some("2024-03-14T10:30:00.000Z".to_string()));

    // Third: subtask with parent and link
    let t3 = &resp.items[2];
    assert_eq!(t3.parent, Some("task_parent_001".to_string()));
    assert_eq!(t3.links.len(), 1);
    assert!(t3.links[0].link.as_ref().unwrap().contains("github.com"));
}

// ---------------------------------------------------------------
// REQ-TASKS-001 (Must): URL builder verification - tasklists
// ---------------------------------------------------------------

// Requirement: REQ-TASKS-001 (Must)
// Acceptance: Tasklist URL builders produce correct URLs
#[test]
fn req_tasks_001_integration_url_builders() {
    // REQ-TASKS-001
    // Default list URL
    let url = build_tasklists_list_url(None, None);
    assert_eq!(url, "https://tasks.googleapis.com/tasks/v1/users/@me/lists");

    // With max results
    let url = build_tasklists_list_url(Some(20), None);
    assert!(url.contains("maxResults=20"));

    // With page token
    let url = build_tasklists_list_url(None, Some("next_tl"));
    assert!(url.contains("pageToken=next_tl"));

    // With both
    let url = build_tasklists_list_url(Some(10), Some("page2"));
    assert!(url.contains("maxResults=10"));
    assert!(url.contains("pageToken=page2"));

    // Create URL
    let url = build_tasklist_create_url();
    assert_eq!(url, "https://tasks.googleapis.com/tasks/v1/users/@me/lists");

    // Create body
    let body = build_tasklist_create_body("Shopping");
    assert_eq!(body["title"], "Shopping");
}

// ---------------------------------------------------------------
// REQ-TASKS-003 (Must): URL builder verification - tasks
// ---------------------------------------------------------------

// Requirement: REQ-TASKS-003 (Must)
// Acceptance: Task URL builders produce correct URLs
#[test]
fn req_tasks_003_integration_task_url_builders() {
    // REQ-TASKS-003
    // Tasks list URL
    let url = build_tasks_list_url("MTIzNDU2", None, None);
    assert!(url.contains("/lists/MTIzNDU2/tasks"));

    // Tasks list URL with params
    let url = build_tasks_list_url("MTIzNDU2", Some(100), Some("task_page2"));
    assert!(url.contains("maxResults=100"));
    assert!(url.contains("pageToken=task_page2"));

    // Task get URL
    let url = build_task_get_url("MTIzNDU2", "dGFzazAwMQ");
    assert!(url.contains("/lists/MTIzNDU2/tasks/dGFzazAwMQ"));

    // Task create URL
    let url = build_task_create_url("MTIzNDU2");
    assert!(url.contains("/lists/MTIzNDU2/tasks"));

    // Task create body with all fields
    let body = build_task_create_body(
        "New Important Task",
        Some("Detailed notes here"),
        Some("2024-04-01T00:00:00.000Z"),
        Some("parent_task_id"),
        Some("previous_task_id"),
    );
    assert_eq!(body["title"], "New Important Task");
    assert_eq!(body["notes"], "Detailed notes here");
    assert_eq!(body["due"], "2024-04-01T00:00:00.000Z");
    assert_eq!(body["parent"], "parent_task_id");
    assert_eq!(body["previous"], "previous_task_id");

    // Task delete URL
    let url = build_task_delete_url("MTIzNDU2", "dGFzazAwMQ");
    assert!(url.contains("/lists/MTIzNDU2/tasks/dGFzazAwMQ"));

    // Tasks clear URL
    let url = build_tasks_clear_url("MTIzNDU2");
    assert!(url.contains("/lists/MTIzNDU2/clear"));
}

// ---------------------------------------------------------------
// REQ-TASKS-006 (Must): Task update body verification
// ---------------------------------------------------------------

// Requirement: REQ-TASKS-006 (Must)
// Acceptance: Task update and status change bodies are correct
#[test]
fn req_tasks_006_integration_task_update_bodies() {
    // REQ-TASKS-006
    // Full update
    let body = build_task_update_body(
        Some("Updated Title"),
        Some("New notes"),
        Some("2024-05-01T00:00:00.000Z"),
        Some("completed"),
    );
    assert_eq!(body["title"], "Updated Title");
    assert_eq!(body["notes"], "New notes");
    assert_eq!(body["due"], "2024-05-01T00:00:00.000Z");
    assert_eq!(body["status"], "completed");

    // Mark as done (REQ-TASKS-007)
    let body = build_task_update_body(None, None, None, Some("completed"));
    assert_eq!(body["status"], "completed");
    assert!(body.get("title").is_none());

    // Undo done (REQ-TASKS-008)
    let body = build_task_update_body(None, None, None, Some("needsAction"));
    assert_eq!(body["status"], "needsAction");

    // Empty update
    let body = build_task_update_body(None, None, None, None);
    assert!(body.as_object().unwrap().is_empty());
}
