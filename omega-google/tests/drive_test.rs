//! Drive service integration tests.

use omega_google::services::drive::types::*;
use std::collections::HashMap;

// ---------------------------------------------------------------
// REQ-DRIVE-001 (Must): File list from realistic API response
// ---------------------------------------------------------------

// Requirement: REQ-DRIVE-001 (Must)
// Acceptance: FileListResponse with various file types
#[test]
fn req_drive_001_integration_file_list_from_api() {
    let api_response = r#"{
        "files": [
            {
                "id": "doc_1",
                "name": "Q1 Report",
                "mimeType": "application/vnd.google-apps.document",
                "modifiedTime": "2024-01-15T14:30:00.000Z",
                "parents": ["folder_abc"],
                "webViewLink": "https://docs.google.com/document/d/doc_1/edit"
            },
            {
                "id": "sheet_1",
                "name": "Budget 2024.xlsx",
                "mimeType": "application/vnd.google-apps.spreadsheet",
                "modifiedTime": "2024-01-14T10:00:00.000Z",
                "parents": ["folder_abc"]
            },
            {
                "id": "pdf_1",
                "name": "Contract.pdf",
                "mimeType": "application/pdf",
                "size": "2048576",
                "modifiedTime": "2024-01-10T08:00:00.000Z",
                "parents": ["folder_abc"]
            },
            {
                "id": "folder_1",
                "name": "Subfolder",
                "mimeType": "application/vnd.google-apps.folder",
                "modifiedTime": "2024-01-05T12:00:00.000Z",
                "parents": ["folder_abc"]
            }
        ],
        "nextPageToken": "next_page_token_123"
    }"#;
    let resp: FileListResponse = serde_json::from_str(api_response).unwrap();
    assert_eq!(resp.files.len(), 4);
    assert_eq!(resp.files[0].name, Some("Q1 Report".to_string()));
    assert_eq!(resp.files[0].mime_type, Some(MIME_GOOGLE_DOC.to_string()));
    assert_eq!(resp.files[2].size, Some("2048576".to_string()));
    assert_eq!(resp.files[3].mime_type, Some(MIME_GOOGLE_FOLDER.to_string()));
    assert_eq!(resp.next_page_token, Some("next_page_token_123".to_string()));
}

// ---------------------------------------------------------------
// REQ-DRIVE-013 (Must): URL generation integration
// ---------------------------------------------------------------

// Requirement: REQ-DRIVE-013 (Must)
// Acceptance: URLs generated for multiple files
#[test]
fn req_drive_013_integration_multiple_urls() {
    let file_ids = vec!["doc_1", "sheet_1", "pdf_1"];
    let urls: Vec<String> = file_ids.iter().map(|id| file_url(id)).collect();
    assert_eq!(urls.len(), 3);
    assert_eq!(urls[0], "https://drive.google.com/open?id=doc_1");
    assert_eq!(urls[1], "https://drive.google.com/open?id=sheet_1");
    assert_eq!(urls[2], "https://drive.google.com/open?id=pdf_1");
}

// ---------------------------------------------------------------
// REQ-DRIVE-010 (Must): Permission types from API
// ---------------------------------------------------------------

// Requirement: REQ-DRIVE-010 (Must)
// Acceptance: Permission list with various types
#[test]
fn req_drive_010_integration_permission_list() {
    let api_response = r#"{
        "permissions": [
            {
                "id": "perm_owner",
                "type": "user",
                "role": "owner",
                "emailAddress": "alice@example.com",
                "displayName": "Alice"
            },
            {
                "id": "perm_editor",
                "type": "user",
                "role": "writer",
                "emailAddress": "bob@example.com",
                "displayName": "Bob"
            },
            {
                "id": "perm_anyone",
                "type": "anyone",
                "role": "reader"
            },
            {
                "id": "perm_domain",
                "type": "domain",
                "role": "reader",
                "domain": "example.com"
            }
        ]
    }"#;
    let resp: PermissionListResponse = serde_json::from_str(api_response).unwrap();
    assert_eq!(resp.permissions.len(), 4);
    assert_eq!(resp.permissions[0].role, Some("owner".to_string()));
    assert_eq!(resp.permissions[2].r#type, Some("anyone".to_string()));
    assert_eq!(resp.permissions[3].domain, Some("example.com".to_string()));
}

// ---------------------------------------------------------------
// REQ-DRIVE-016 (Should): Comments with replies
// ---------------------------------------------------------------

// Requirement: REQ-DRIVE-016 (Should)
// Acceptance: Comment list with replies
#[test]
fn req_drive_016_integration_comments_with_replies() {
    let api_response = r#"{
        "comments": [
            {
                "id": "comment_1",
                "content": "Please review section 3",
                "author": {
                    "displayName": "Alice",
                    "emailAddress": "alice@example.com"
                },
                "createdTime": "2024-01-15T14:30:00.000Z",
                "resolved": false,
                "quotedFileContent": {
                    "mimeType": "text/html",
                    "value": "Section 3: Budget"
                },
                "replies": [
                    {
                        "id": "reply_1",
                        "content": "Done, updated the numbers",
                        "author": {
                            "displayName": "Bob",
                            "emailAddress": "bob@example.com"
                        },
                        "createdTime": "2024-01-15T15:00:00.000Z"
                    }
                ]
            },
            {
                "id": "comment_2",
                "content": "LGTM!",
                "author": {
                    "displayName": "Carol",
                    "emailAddress": "carol@example.com"
                },
                "createdTime": "2024-01-15T16:00:00.000Z",
                "resolved": true,
                "replies": []
            }
        ]
    }"#;
    let resp: CommentListResponse = serde_json::from_str(api_response).unwrap();
    assert_eq!(resp.comments.len(), 2);
    assert_eq!(resp.comments[0].replies.len(), 1);
    assert_eq!(resp.comments[0].resolved, Some(false));
    assert_eq!(resp.comments[1].resolved, Some(true));
    let quoted = resp.comments[0].quoted_file_content.as_ref().unwrap();
    assert_eq!(quoted.value, Some("Section 3: Budget".to_string()));
}

// ---------------------------------------------------------------
// REQ-DRIVE-014 (Must): Shared drives list
// ---------------------------------------------------------------

// Requirement: REQ-DRIVE-014 (Must)
// Acceptance: Shared drives list from API
#[test]
fn req_drive_014_integration_shared_drives() {
    let api_response = r#"{
        "drives": [
            {
                "id": "drive_eng",
                "name": "Engineering",
                "createdTime": "2023-06-01T00:00:00.000Z"
            },
            {
                "id": "drive_mktg",
                "name": "Marketing",
                "createdTime": "2023-07-15T00:00:00.000Z"
            }
        ],
        "nextPageToken": "drives_page_2"
    }"#;
    let resp: SharedDriveListResponse = serde_json::from_str(api_response).unwrap();
    assert_eq!(resp.drives.len(), 2);
    assert_eq!(resp.drives[0].name, Some("Engineering".to_string()));
    assert_eq!(resp.next_page_token, Some("drives_page_2".to_string()));
}

// ---------------------------------------------------------------
// REQ-DRIVE-001 (Must): Edge case - empty fields
// ---------------------------------------------------------------

// Requirement: REQ-DRIVE-001 (Must)
// Edge case: File with minimal fields
#[test]
fn req_drive_001_integration_minimal_file() {
    let api_response = r#"{"id": "f1"}"#;
    let file: DriveFile = serde_json::from_str(api_response).unwrap();
    assert_eq!(file.id, Some("f1".to_string()));
    assert_eq!(file.name, None);
    assert_eq!(file.mime_type, None);
    assert_eq!(file.size, None);
    assert!(file.parents.is_empty());
}

// ---------------------------------------------------------------
// REQ-DRIVE-017 (Must): All-drives default flag
// ---------------------------------------------------------------

// Requirement: REQ-DRIVE-017 (Must)
// Acceptance: File list response can include shared drive files
#[test]
fn req_drive_017_integration_shared_drive_files() {
    let api_response = r#"{
        "files": [
            {
                "id": "shared_file_1",
                "name": "Team Doc",
                "mimeType": "application/vnd.google-apps.document",
                "parents": ["shared_drive_id"]
            }
        ],
        "incompleteSearch": false
    }"#;
    let resp: FileListResponse = serde_json::from_str(api_response).unwrap();
    assert_eq!(resp.files.len(), 1);
    assert_eq!(resp.incomplete_search, Some(false));
}

// ---------------------------------------------------------------
// REQ-DRIVE-001 (Must): Size formatting integration
// ---------------------------------------------------------------

// Requirement: REQ-DRIVE-001 (Must)
// Acceptance: Size formatting helper used with file metadata
#[test]
fn req_drive_001_integration_size_formatting() {
    use omega_google::services::common::format_size;

    let file_json = r#"{"id": "f1", "size": "1048576"}"#;
    let file: DriveFile = serde_json::from_str(file_json).unwrap();
    let size_str = file.size.as_ref().map(|s| {
        let bytes: i64 = s.parse().unwrap_or(0);
        format_size(bytes)
    }).unwrap_or("-".to_string());
    assert_eq!(size_str, "1.0 MB");
}

// ---------------------------------------------------------------
// REQ-DRIVE-001 (Must): DateTime formatting integration
// ---------------------------------------------------------------

// Requirement: REQ-DRIVE-001 (Must)
// Acceptance: DateTime formatting helper used with file metadata
#[test]
fn req_drive_001_integration_datetime_formatting() {
    use omega_google::services::common::format_datetime;

    let file_json = r#"{"id": "f1", "modifiedTime": "2024-01-15T14:30:00.000Z"}"#;
    let file: DriveFile = serde_json::from_str(file_json).unwrap();
    let dt = file.modified_time.as_deref().map(format_datetime).unwrap_or("-".to_string());
    assert_eq!(dt, "2024-01-15 14:30");
}
