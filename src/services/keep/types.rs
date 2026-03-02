//! Google Keep API request/response types.
//! All types use camelCase serde rename and flatten for forward compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------
// Note types
// ---------------------------------------------------------------

/// A Google Keep note.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub name: Option<String>,
    pub title: Option<String>,
    pub body: Option<NoteBody>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
    pub trash_time: Option<String>,
    pub trashed: Option<bool>,
    #[serde(default)]
    pub permissions: Vec<Permission>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// The body of a Keep note (text or list).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteBody {
    pub text: Option<TextContent>,
    pub list: Option<ListContent>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Text content within a note body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextContent {
    pub text: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// List content within a note body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListContent {
    #[serde(default)]
    pub list_items: Vec<ListItem>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A list item within a list content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    pub text: Option<TextContent>,
    pub checked: Option<bool>,
    #[serde(default)]
    pub child_list_items: Vec<ListItem>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response for listing notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteListResponse {
    #[serde(default)]
    pub notes: Vec<Note>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Attachment types
// ---------------------------------------------------------------

/// A Keep attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub name: Option<String>,
    #[serde(default)]
    pub mime_type: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Permission types
// ---------------------------------------------------------------

/// A permission on a Keep note.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permission {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-KEEP-001 (Must): Note type deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-KEEP-001 (Must)
    // Acceptance: Note type deserializes from Keep API JSON
    #[test]
    fn req_keep_001_note_deserialize() {
        // REQ-KEEP-001
        let json_str = r#"{
            "name": "notes/abc123",
            "title": "Shopping List",
            "body": {
                "text": {
                    "text": "Buy groceries"
                }
            },
            "createTime": "2024-01-15T10:30:00Z",
            "updateTime": "2024-01-16T08:00:00Z",
            "trashed": false
        }"#;
        let note: Note = serde_json::from_str(json_str).unwrap();
        assert_eq!(note.name, Some("notes/abc123".to_string()));
        assert_eq!(note.title, Some("Shopping List".to_string()));
        assert_eq!(note.trashed, Some(false));
        assert_eq!(note.create_time, Some("2024-01-15T10:30:00Z".to_string()));

        let body = note.body.unwrap();
        let text = body.text.unwrap();
        assert_eq!(text.text, Some("Buy groceries".to_string()));
    }

    // Requirement: REQ-KEEP-001 (Must)
    // Acceptance: Note with list body deserializes
    #[test]
    fn req_keep_001_note_list_body_deserialize() {
        // REQ-KEEP-001
        let json_str = r#"{
            "name": "notes/list123",
            "title": "Todo",
            "body": {
                "list": {
                    "listItems": [
                        {"text": {"text": "Item 1"}, "checked": false},
                        {"text": {"text": "Item 2"}, "checked": true},
                        {
                            "text": {"text": "Item 3"},
                            "checked": false,
                            "childListItems": [
                                {"text": {"text": "Sub-item A"}, "checked": false}
                            ]
                        }
                    ]
                }
            }
        }"#;
        let note: Note = serde_json::from_str(json_str).unwrap();
        let body = note.body.unwrap();
        let list = body.list.unwrap();
        assert_eq!(list.list_items.len(), 3);
        assert_eq!(list.list_items[0].checked, Some(false));
        assert_eq!(list.list_items[1].checked, Some(true));
        assert_eq!(list.list_items[2].child_list_items.len(), 1);
    }

    // Requirement: REQ-KEEP-001 (Must)
    // Acceptance: NoteListResponse deserializes with pagination
    #[test]
    fn req_keep_001_note_list_response_deserialize() {
        // REQ-KEEP-001
        let json_str = r#"{
            "notes": [
                {"name": "notes/a", "title": "Note 1"},
                {"name": "notes/b", "title": "Note 2"}
            ],
            "nextPageToken": "token_xyz"
        }"#;
        let resp: NoteListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.notes.len(), 2);
        assert_eq!(resp.notes[0].name, Some("notes/a".to_string()));
        assert_eq!(resp.next_page_token, Some("token_xyz".to_string()));
    }

    // Requirement: REQ-KEEP-001 (Must)
    // Edge case: NoteListResponse with empty notes
    #[test]
    fn req_keep_001_note_list_response_empty() {
        // REQ-KEEP-001
        let json_str = r#"{}"#;
        let resp: NoteListResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.notes.is_empty());
        assert!(resp.next_page_token.is_none());
    }

    // Requirement: REQ-KEEP-001 (Must)
    // Edge case: Note with unknown fields preserved via flatten
    #[test]
    fn req_keep_001_note_unknown_fields_preserved() {
        // REQ-KEEP-001
        let json_str = r#"{
            "name": "notes/xxx",
            "futureField": "some_value"
        }"#;
        let note: Note = serde_json::from_str(json_str).unwrap();
        assert_eq!(note.name, Some("notes/xxx".to_string()));
        assert!(note.extra.contains_key("futureField"));
    }

    // Requirement: REQ-KEEP-001 (Must)
    // Acceptance: Note round-trip serialization
    #[test]
    fn req_keep_001_note_roundtrip() {
        // REQ-KEEP-001
        let note = Note {
            name: Some("notes/aaa".to_string()),
            title: Some("Test".to_string()),
            body: Some(NoteBody {
                text: Some(TextContent {
                    text: Some("Hello".to_string()),
                    extra: HashMap::new(),
                }),
                list: None,
                extra: HashMap::new(),
            }),
            create_time: None,
            update_time: None,
            trash_time: None,
            trashed: Some(false),
            permissions: vec![],
            attachments: vec![],
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&note).unwrap();
        let parsed: Note = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, Some("notes/aaa".to_string()));
        assert_eq!(parsed.title, Some("Test".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-KEEP-002 (Must): Attachment and Permission types
    // ---------------------------------------------------------------

    // Requirement: REQ-KEEP-002 (Must)
    // Acceptance: Attachment type deserializes
    #[test]
    fn req_keep_002_attachment_deserialize() {
        // REQ-KEEP-002
        let json_str = r#"{
            "name": "notes/abc/attachments/att1",
            "mimeType": ["image/png"]
        }"#;
        let att: Attachment = serde_json::from_str(json_str).unwrap();
        assert_eq!(att.name, Some("notes/abc/attachments/att1".to_string()));
        assert_eq!(att.mime_type, vec!["image/png"]);
    }

    // Requirement: REQ-KEEP-002 (Must)
    // Acceptance: Permission type deserializes
    #[test]
    fn req_keep_002_permission_deserialize() {
        // REQ-KEEP-002
        let json_str = r#"{
            "name": "notes/abc/permissions/perm1",
            "email": "user@example.com",
            "role": "WRITER"
        }"#;
        let perm: Permission = serde_json::from_str(json_str).unwrap();
        assert_eq!(perm.name, Some("notes/abc/permissions/perm1".to_string()));
        assert_eq!(perm.email, Some("user@example.com".to_string()));
        assert_eq!(perm.role, Some("WRITER".to_string()));
    }

    // Requirement: REQ-KEEP-002 (Must)
    // Acceptance: Note with attachments and permissions
    #[test]
    fn req_keep_002_note_with_attachments_and_permissions() {
        // REQ-KEEP-002
        let json_str = r#"{
            "name": "notes/full123",
            "title": "Full Note",
            "body": {"text": {"text": "Content here"}},
            "permissions": [
                {"name": "notes/full123/permissions/p1", "email": "owner@example.com", "role": "OWNER"}
            ],
            "attachments": [
                {"name": "notes/full123/attachments/a1", "mimeType": ["image/jpeg"]}
            ]
        }"#;
        let note: Note = serde_json::from_str(json_str).unwrap();
        assert_eq!(note.permissions.len(), 1);
        assert_eq!(note.permissions[0].role, Some("OWNER".to_string()));
        assert_eq!(note.attachments.len(), 1);
        assert_eq!(note.attachments[0].mime_type, vec!["image/jpeg"]);
    }
}
