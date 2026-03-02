//! Gmail service integration tests.
//! These test the interaction between Gmail types, query builders, and MIME construction.

use omega_google::services::gmail::types::*;

// ---------------------------------------------------------------
// REQ-GMAIL-001 (Must): Thread search integration
// ---------------------------------------------------------------

// Requirement: REQ-GMAIL-001 (Must)
// Acceptance: ThreadListResponse deserialization from realistic API response
#[test]
fn req_gmail_001_integration_thread_list_from_api() {
    let api_response = r#"{
        "threads": [
            {
                "id": "18abc123def",
                "snippet": "Hey, are you available for a meeting tomorrow?",
                "historyId": "99999"
            },
            {
                "id": "18abc456ghi",
                "snippet": "Please review the attached document",
                "historyId": "99998"
            }
        ],
        "nextPageToken": "NEXT_TOKEN_123",
        "resultSizeEstimate": 100
    }"#;
    let resp: ThreadListResponse = serde_json::from_str(api_response).unwrap();
    assert_eq!(resp.threads.len(), 2);
    assert_eq!(resp.threads[0].id, "18abc123def");
    assert_eq!(resp.next_page_token, Some("NEXT_TOKEN_123".to_string()));
    assert_eq!(resp.result_size_estimate, Some(100));
}

// ---------------------------------------------------------------
// REQ-GMAIL-006 (Must): Message with nested MIME parts
// ---------------------------------------------------------------

// Requirement: REQ-GMAIL-006 (Must)
// Acceptance: Complex message with nested MIME structure
#[test]
fn req_gmail_006_integration_nested_mime_message() {
    let api_response = r#"{
        "id": "msg_complex",
        "threadId": "thread_complex",
        "labelIds": ["INBOX", "IMPORTANT"],
        "snippet": "Complex message",
        "internalDate": "1704067200000",
        "sizeEstimate": 50000,
        "payload": {
            "mimeType": "multipart/mixed",
            "headers": [
                {"name": "From", "value": "sender@example.com"},
                {"name": "To", "value": "recipient@example.com"},
                {"name": "Subject", "value": "Report with Attachments"},
                {"name": "Date", "value": "Mon, 01 Jan 2024 00:00:00 +0000"}
            ],
            "parts": [
                {
                    "mimeType": "multipart/alternative",
                    "parts": [
                        {
                            "mimeType": "text/plain",
                            "body": {"size": 100, "data": "SGVsbG8gV29ybGQ="}
                        },
                        {
                            "mimeType": "text/html",
                            "body": {"size": 200, "data": "PGgxPkhlbGxvPC9oMT4="}
                        }
                    ]
                },
                {
                    "mimeType": "application/pdf",
                    "filename": "report.pdf",
                    "body": {"attachmentId": "attach_123", "size": 1024}
                }
            ]
        }
    }"#;
    let msg: Message = serde_json::from_str(api_response).unwrap();
    assert_eq!(msg.id, "msg_complex");
    let payload = msg.payload.as_ref().unwrap();
    assert_eq!(payload.mime_type, Some("multipart/mixed".to_string()));
    assert_eq!(payload.parts.len(), 2);

    // Check nested alternative part
    let alt_part = &payload.parts[0];
    assert_eq!(alt_part.mime_type, Some("multipart/alternative".to_string()));
    assert_eq!(alt_part.parts.len(), 2);

    // Check attachment part
    let attach_part = &payload.parts[1];
    assert_eq!(attach_part.filename, Some("report.pdf".to_string()));
    assert_eq!(
        attach_part.body.as_ref().unwrap().attachment_id,
        Some("attach_123".to_string())
    );
}

// Requirement: REQ-GMAIL-006 (Must)
// Acceptance: Header extraction from nested payload
#[test]
fn req_gmail_006_integration_header_extraction() {
    let msg_json = r#"{
        "id": "msg1",
        "payload": {
            "mimeType": "text/plain",
            "headers": [
                {"name": "From", "value": "Alice <alice@example.com>"},
                {"name": "To", "value": "Bob <bob@example.com>"},
                {"name": "Subject", "value": "Re: Project Update"},
                {"name": "Date", "value": "Mon, 15 Jan 2024 10:30:00 -0500"},
                {"name": "Message-ID", "value": "<abc123@example.com>"},
                {"name": "In-Reply-To", "value": "<xyz789@example.com>"},
                {"name": "List-Unsubscribe", "value": "<https://example.com/unsub>, <mailto:unsub@example.com>"}
            ],
            "body": {"size": 50}
        }
    }"#;
    let msg: Message = serde_json::from_str(msg_json).unwrap();
    let payload = msg.payload.as_ref().unwrap();

    assert_eq!(
        header_value(payload, "From"),
        Some("Alice <alice@example.com>".to_string())
    );
    assert_eq!(
        header_value(payload, "subject"),
        Some("Re: Project Update".to_string())
    );
    assert_eq!(
        header_value(payload, "message-id"),
        Some("<abc123@example.com>".to_string())
    );
    assert_eq!(header_value(payload, "X-Nonexistent"), None);
}

// ---------------------------------------------------------------
// REQ-GMAIL-008 (Must): URL generation for multiple threads
// ---------------------------------------------------------------

// Requirement: REQ-GMAIL-008 (Must)
// Acceptance: Multiple thread URLs generated correctly
#[test]
fn req_gmail_008_integration_multiple_urls() {
    let thread_ids = ["18abc", "18def", "18ghi"];
    let urls: Vec<String> = thread_ids.iter().map(|id| thread_url(id)).collect();
    assert_eq!(urls.len(), 3);
    for url in &urls {
        assert!(url.starts_with("https://mail.google.com/mail/u/0/#all/"));
    }
    assert!(urls[0].ends_with("18abc"));
    assert!(urls[1].ends_with("18def"));
    assert!(urls[2].ends_with("18ghi"));
}

// ---------------------------------------------------------------
// REQ-GMAIL-009 (Must): Label list with mixed types
// ---------------------------------------------------------------

// Requirement: REQ-GMAIL-009 (Must)
// Acceptance: Label list with both system and user labels
#[test]
fn req_gmail_009_integration_mixed_labels() {
    let api_response = r#"{
        "labels": [
            {"id": "INBOX", "name": "INBOX", "type": "system", "messagesTotal": 500},
            {"id": "SENT", "name": "SENT", "type": "system", "messagesTotal": 200},
            {"id": "Label_1", "name": "Work", "type": "user", "messagesTotal": 50},
            {"id": "Label_2", "name": "Personal", "type": "user", "messagesTotal": 30}
        ]
    }"#;
    let resp: LabelListResponse = serde_json::from_str(api_response).unwrap();
    assert_eq!(resp.labels.len(), 4);

    let system_labels: Vec<&Label> = resp
        .labels
        .iter()
        .filter(|l| l.r#type.as_deref() == Some("system"))
        .collect();
    assert_eq!(system_labels.len(), 2);

    let user_labels: Vec<&Label> = resp
        .labels
        .iter()
        .filter(|l| l.r#type.as_deref() == Some("user"))
        .collect();
    assert_eq!(user_labels.len(), 2);
}

// ---------------------------------------------------------------
// REQ-GMAIL-014 (Must): Batch operation types
// ---------------------------------------------------------------

// Requirement: REQ-GMAIL-014 (Must)
// Acceptance: Large batch modify request serialization
#[test]
fn req_gmail_014_integration_large_batch() {
    let ids: Vec<String> = (0..100).map(|i| format!("msg_{}", i)).collect();
    let req = BatchModifyRequest {
        ids: ids.clone(),
        add_label_ids: vec!["STARRED".to_string()],
        remove_label_ids: vec!["UNREAD".to_string()],
    };
    let json = serde_json::to_string(&req).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["ids"].as_array().unwrap().len(), 100);
}

// ---------------------------------------------------------------
// REQ-GMAIL-015 (Must): Complex filter criteria
// ---------------------------------------------------------------

// Requirement: REQ-GMAIL-015 (Must)
// Acceptance: Filter with all criteria fields
#[test]
fn req_gmail_015_integration_complex_filter() {
    let json_str = r#"{
        "id": "filter_complex",
        "criteria": {
            "from": "newsletter@company.com",
            "to": "me@example.com",
            "subject": "Weekly Report",
            "query": "has:attachment",
            "negatedQuery": "is:spam",
            "hasAttachment": true,
            "size": 1048576,
            "sizeComparison": "larger"
        },
        "action": {
            "addLabelIds": ["Label_1", "Label_2"],
            "removeLabelIds": ["INBOX"],
            "forward": "archive@example.com"
        }
    }"#;
    let filter: Filter = serde_json::from_str(json_str).unwrap();
    let criteria = filter.criteria.unwrap();
    assert_eq!(criteria.from, Some("newsletter@company.com".to_string()));
    assert_eq!(criteria.size_comparison, Some("larger".to_string()));
    let action = filter.action.unwrap();
    assert_eq!(action.add_label_ids.len(), 2);
    assert_eq!(action.forward, Some("archive@example.com".to_string()));
}
