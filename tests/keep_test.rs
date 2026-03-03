//! Keep service integration tests.

use omega_google::services::keep::attachments::*;
use omega_google::services::keep::notes::*;
use omega_google::services::keep::types::*;
use std::collections::HashMap;

// ---------------------------------------------------------------
// REQ-KEEP-001 (Must): Note deserialization from full JSON
// ---------------------------------------------------------------

// Requirement: REQ-KEEP-001 (Must)
// Acceptance: Full note structure from a realistic Keep API response
#[test]
fn req_keep_001_integration_full_note_from_api() {
    // REQ-KEEP-001
    let api_response = r#"{
        "name": "notes/abc123def456",
        "title": "Project Ideas",
        "body": {
            "text": {
                "text": "Implement the new feature by Q2.\nNeed to coordinate with the design team."
            }
        },
        "createTime": "2024-01-15T10:30:00Z",
        "updateTime": "2024-03-20T14:00:00Z",
        "trashed": false,
        "permissions": [
            {
                "name": "notes/abc123def456/permissions/p1",
                "email": "owner@example.com",
                "role": "OWNER"
            }
        ],
        "attachments": [
            {
                "name": "notes/abc123def456/attachments/a1",
                "mimeType": ["image/png"]
            }
        ],
        "color": "DEFAULT"
    }"#;

    let note: Note = serde_json::from_str(api_response).unwrap();

    assert_eq!(note.name, Some("notes/abc123def456".to_string()));
    assert_eq!(note.title, Some("Project Ideas".to_string()));
    assert_eq!(note.trashed, Some(false));
    assert_eq!(note.create_time, Some("2024-01-15T10:30:00Z".to_string()));

    let body = note.body.unwrap();
    let text = body.text.unwrap();
    assert!(text.text.unwrap().contains("Implement the new feature"));

    assert_eq!(note.permissions.len(), 1);
    assert_eq!(note.permissions[0].role, Some("OWNER".to_string()));

    assert_eq!(note.attachments.len(), 1);
    assert_eq!(note.attachments[0].mime_type, vec!["image/png"]);

    // Unknown fields preserved via flatten
    assert!(note.extra.contains_key("color"));
}

// ---------------------------------------------------------------
// REQ-KEEP-001 (Must): Note list response from realistic API
// ---------------------------------------------------------------

// Requirement: REQ-KEEP-001 (Must)
// Acceptance: NoteListResponse with multiple notes and pagination
#[test]
fn req_keep_001_integration_note_list_from_api() {
    // REQ-KEEP-001
    let api_response = r#"{
        "notes": [
            {
                "name": "notes/aaa",
                "title": "Shopping List",
                "body": {"text": {"text": "Milk, bread, eggs"}},
                "trashed": false
            },
            {
                "name": "notes/bbb",
                "title": "Meeting Notes",
                "body": {"text": {"text": "Discussed Q2 roadmap"}},
                "trashed": false
            },
            {
                "name": "notes/ccc",
                "title": "Todo",
                "body": {
                    "list": {
                        "listItems": [
                            {"text": {"text": "Review PR"}, "checked": true},
                            {"text": {"text": "Deploy staging"}, "checked": false}
                        ]
                    }
                },
                "trashed": false
            }
        ],
        "nextPageToken": "notes_page_2_token"
    }"#;

    let resp: NoteListResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(resp.notes.len(), 3);
    assert_eq!(resp.next_page_token, Some("notes_page_2_token".to_string()));

    // First note: text
    assert_eq!(resp.notes[0].title, Some("Shopping List".to_string()));

    // Third note: list
    let body = resp.notes[2].body.as_ref().unwrap();
    let list = body.list.as_ref().unwrap();
    assert_eq!(list.list_items.len(), 2);
    assert_eq!(list.list_items[0].checked, Some(true));
}

// ---------------------------------------------------------------
// REQ-KEEP-001 (Must): URL builder verification - notes list
// ---------------------------------------------------------------

// Requirement: REQ-KEEP-001 (Must)
// Acceptance: Notes list URL builds correctly with various params
#[test]
fn req_keep_001_integration_url_builder_notes_list() {
    // REQ-KEEP-001
    // No params
    let url = build_notes_list_url(None, None, None);
    assert_eq!(url, "https://keep.googleapis.com/v1/notes");

    // With page size
    let url = build_notes_list_url(Some(50), None, None);
    assert!(url.contains("pageSize=50"));

    // With page token
    let url = build_notes_list_url(None, Some("token123"), None);
    assert!(url.contains("pageToken=token123"));

    // With filter
    let url = build_notes_list_url(None, None, Some("trashed = false"));
    assert!(url.contains("filter="));

    // With all
    let url = build_notes_list_url(Some(25), Some("next"), Some("trashed = false"));
    assert!(url.contains("pageSize=25"));
    assert!(url.contains("pageToken=next"));
    assert!(url.contains("filter="));
}

// ---------------------------------------------------------------
// REQ-KEEP-002 (Must): URL builder verification - note get
// ---------------------------------------------------------------

// Requirement: REQ-KEEP-002 (Must)
// Acceptance: Note get URL builds correctly
#[test]
fn req_keep_002_integration_url_builder_note_get() {
    // REQ-KEEP-002
    // Raw ID
    let url = build_note_get_url("abc123");
    assert!(url.starts_with("https://keep.googleapis.com/v1/"));
    assert!(url.contains("notes"));
    assert!(url.contains("abc123"));

    // Full resource name
    let url = build_note_get_url("notes/abc123");
    assert!(url.starts_with("https://keep.googleapis.com/v1/"));
    assert!(url.contains("notes"));
    assert!(url.contains("abc123"));
}

// ---------------------------------------------------------------
// REQ-KEEP-003 (Must): Client-side search
// ---------------------------------------------------------------

// Requirement: REQ-KEEP-003 (Must)
// Acceptance: Search across multiple notes
#[test]
fn req_keep_003_integration_notes_search() {
    // REQ-KEEP-003
    let notes = vec![
        Note {
            name: Some("notes/a".to_string()),
            title: Some("Shopping List".to_string()),
            body: Some(NoteBody {
                text: Some(TextContent {
                    text: Some("Buy milk and bread".to_string()),
                    extra: HashMap::new(),
                }),
                list: None,
                extra: HashMap::new(),
            }),
            create_time: None,
            update_time: None,
            trash_time: None,
            trashed: None,
            permissions: vec![],
            attachments: vec![],
            extra: HashMap::new(),
        },
        Note {
            name: Some("notes/b".to_string()),
            title: Some("Meeting Notes".to_string()),
            body: Some(NoteBody {
                text: Some(TextContent {
                    text: Some("Discussed roadmap and Q2 planning".to_string()),
                    extra: HashMap::new(),
                }),
                list: None,
                extra: HashMap::new(),
            }),
            create_time: None,
            update_time: None,
            trash_time: None,
            trashed: None,
            permissions: vec![],
            attachments: vec![],
            extra: HashMap::new(),
        },
        Note {
            name: Some("notes/c".to_string()),
            title: Some("Grocery Todo".to_string()),
            body: None,
            create_time: None,
            update_time: None,
            trash_time: None,
            trashed: None,
            permissions: vec![],
            attachments: vec![],
            extra: HashMap::new(),
        },
    ];

    // Search by title
    let results = build_notes_search(&notes, "meeting");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, Some("notes/b".to_string()));

    // Search by body text
    let results = build_notes_search(&notes, "milk");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, Some("notes/a".to_string()));

    // Case insensitive
    let results = build_notes_search(&notes, "GROCERY");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, Some("notes/c".to_string()));

    // No match
    let results = build_notes_search(&notes, "nonexistent");
    assert!(results.is_empty());
}

// ---------------------------------------------------------------
// REQ-KEEP-004 (Must): Attachment download URL
// ---------------------------------------------------------------

// Requirement: REQ-KEEP-004 (Must)
// Acceptance: Attachment download URL builds correctly
#[test]
fn req_keep_004_integration_attachment_download_url() {
    // REQ-KEEP-004
    let url = build_attachment_download_url("notes/abc123/attachments/att456");
    assert!(url.starts_with("https://keep.googleapis.com/v1/"));
    assert!(url.contains(":media"));
    assert!(url.contains("notes"));
    assert!(url.contains("attachments"));
}
