//! Gmail API request/response types.
//! All types use camelCase serde rename and flatten for forward compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------
// Thread types
// ---------------------------------------------------------------

/// Gmail thread list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadListResponse {
    #[serde(default)]
    pub threads: Vec<Thread>,
    pub next_page_token: Option<String>,
    pub result_size_estimate: Option<u32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A Gmail thread containing messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub id: String,
    pub snippet: Option<String>,
    pub history_id: Option<String>,
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Message types
// ---------------------------------------------------------------

/// A single Gmail message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub thread_id: Option<String>,
    #[serde(default)]
    pub label_ids: Vec<String>,
    pub snippet: Option<String>,
    pub history_id: Option<String>,
    pub internal_date: Option<String>,
    pub payload: Option<MessagePart>,
    pub size_estimate: Option<i64>,
    pub raw: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A MIME message part (recursive structure).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagePart {
    pub part_id: Option<String>,
    pub mime_type: Option<String>,
    pub filename: Option<String>,
    #[serde(default)]
    pub headers: Vec<Header>,
    pub body: Option<MessagePartBody>,
    #[serde(default)]
    pub parts: Vec<MessagePart>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A header name/value pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

/// The body of a message part.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartBody {
    pub attachment_id: Option<String>,
    pub size: Option<i64>,
    pub data: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Label types
// ---------------------------------------------------------------

/// Gmail label list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelListResponse {
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A Gmail label.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: String,
    pub name: String,
    pub r#type: Option<String>,
    pub message_list_visibility: Option<String>,
    pub label_list_visibility: Option<String>,
    pub messages_total: Option<u32>,
    pub messages_unread: Option<u32>,
    pub threads_total: Option<u32>,
    pub threads_unread: Option<u32>,
    pub color: Option<LabelColor>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Label color information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelColor {
    pub text_color: Option<String>,
    pub background_color: Option<String>,
}

// ---------------------------------------------------------------
// Draft types
// ---------------------------------------------------------------

/// Gmail draft list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftListResponse {
    #[serde(default)]
    pub drafts: Vec<Draft>,
    pub next_page_token: Option<String>,
    pub result_size_estimate: Option<u32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A Gmail draft.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    pub id: String,
    pub message: Option<Message>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// History types
// ---------------------------------------------------------------

/// Gmail history list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryListResponse {
    #[serde(default)]
    pub history: Vec<History>,
    pub next_page_token: Option<String>,
    pub history_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A history record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct History {
    pub id: String,
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(default)]
    pub messages_added: Vec<HistoryMessageAdded>,
    #[serde(default)]
    pub messages_deleted: Vec<HistoryMessageDeleted>,
    #[serde(default)]
    pub labels_added: Vec<HistoryLabelAdded>,
    #[serde(default)]
    pub labels_removed: Vec<HistoryLabelRemoved>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A message added in history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMessageAdded {
    pub message: Option<Message>,
}

/// A message deleted in history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMessageDeleted {
    pub message: Option<Message>,
}

/// A label added to a message in history.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryLabelAdded {
    pub message: Option<Message>,
    #[serde(default)]
    pub label_ids: Vec<String>,
}

/// A label removed from a message in history.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryLabelRemoved {
    pub message: Option<Message>,
    #[serde(default)]
    pub label_ids: Vec<String>,
}

// ---------------------------------------------------------------
// Watch types
// ---------------------------------------------------------------

/// Gmail watch response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchResponse {
    pub history_id: Option<String>,
    pub expiration: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Gmail watch request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchRequest {
    pub topic_name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub label_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_filter_action: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Batch types
// ---------------------------------------------------------------

/// Batch modify request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchModifyRequest {
    pub ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub add_label_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub remove_label_ids: Vec<String>,
}

/// Batch delete request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<String>,
}

// ---------------------------------------------------------------
// Settings types
// ---------------------------------------------------------------

/// A Gmail filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    pub id: Option<String>,
    pub criteria: Option<FilterCriteria>,
    pub action: Option<FilterAction>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Filter matching criteria.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterCriteria {
    pub from: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
    pub query: Option<String>,
    pub negated_query: Option<String>,
    pub has_attachment: Option<bool>,
    pub size: Option<i64>,
    pub size_comparison: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Filter action to apply.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterAction {
    #[serde(default)]
    pub add_label_ids: Vec<String>,
    #[serde(default)]
    pub remove_label_ids: Vec<String>,
    pub forward: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A forwarding address.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForwardingAddress {
    pub forwarding_email: String,
    pub verification_status: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A send-as alias.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendAs {
    pub send_as_email: String,
    pub display_name: Option<String>,
    pub reply_to_address: Option<String>,
    pub signature: Option<String>,
    pub is_primary: Option<bool>,
    pub is_default: Option<bool>,
    pub treat_as_alias: Option<bool>,
    pub verification_status: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A delegate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Delegate {
    pub delegate_email: String,
    pub verification_status: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Vacation responder settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VacationSettings {
    pub enable_auto_reply: Option<bool>,
    pub response_subject: Option<String>,
    pub response_body_plain_text: Option<String>,
    pub response_body_html: Option<String>,
    pub restrict_to_contacts: Option<bool>,
    pub restrict_to_domain: Option<bool>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Auto-forwarding settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoForwarding {
    pub enabled: Option<bool>,
    pub email_address: Option<String>,
    pub disposition: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------

/// Extract a header value by name (case-insensitive) from a MessagePart.
pub fn header_value(part: &MessagePart, name: &str) -> Option<String> {
    part.headers.iter().find(|h| h.name.eq_ignore_ascii_case(name)).map(|h| h.value.clone())
}

/// Check if a list of header names contains a given name (case-insensitive).
pub fn has_header_name(header_names: &[String], name: &str) -> bool {
    header_names.iter().any(|h| h.trim().eq_ignore_ascii_case(name))
}

/// Generate a Gmail web URL for a thread ID.
pub fn thread_url(thread_id: &str) -> String {
    format!("https://mail.google.com/mail/u/0/#all/{}", thread_id)
}

/// Gmail API base URL.
pub const GMAIL_BASE_URL: &str = "https://gmail.googleapis.com/gmail/v1";

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---------------------------------------------------------------
    // REQ-GMAIL-001 (Must): Thread type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-001 (Must)
    // Acceptance: Thread type deserializes from Gmail API JSON
    #[test]
    fn req_gmail_001_thread_deserialize() {
        let json_str = r#"{
            "id": "thread123",
            "snippet": "Hello world",
            "historyId": "12345",
            "messages": []
        }"#;
        let thread: Thread = serde_json::from_str(json_str).unwrap();
        assert_eq!(thread.id, "thread123");
        assert_eq!(thread.snippet, Some("Hello world".to_string()));
        assert_eq!(thread.history_id, Some("12345".to_string()));
        assert!(thread.messages.is_empty());
    }

    // Requirement: REQ-GMAIL-001 (Must)
    // Acceptance: ThreadListResponse deserializes with pagination
    #[test]
    fn req_gmail_001_thread_list_response_deserialize() {
        let json_str = r#"{
            "threads": [{"id": "t1"}, {"id": "t2"}],
            "nextPageToken": "token_abc",
            "resultSizeEstimate": 42
        }"#;
        let resp: ThreadListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.threads.len(), 2);
        assert_eq!(resp.threads[0].id, "t1");
        assert_eq!(resp.next_page_token, Some("token_abc".to_string()));
        assert_eq!(resp.result_size_estimate, Some(42));
    }

    // Requirement: REQ-GMAIL-001 (Must)
    // Acceptance: ThreadListResponse round-trip serialization
    #[test]
    fn req_gmail_001_thread_list_response_roundtrip() {
        let resp = ThreadListResponse {
            threads: vec![Thread {
                id: "t1".to_string(),
                snippet: Some("test".to_string()),
                history_id: None,
                messages: vec![],
                extra: HashMap::new(),
            }],
            next_page_token: Some("next".to_string()),
            result_size_estimate: Some(1),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: ThreadListResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.threads.len(), 1);
        assert_eq!(parsed.threads[0].id, "t1");
    }

    // Requirement: REQ-GMAIL-001 (Must)
    // Edge case: ThreadListResponse with empty threads list
    #[test]
    fn req_gmail_001_thread_list_response_empty() {
        let json_str = r#"{"resultSizeEstimate": 0}"#;
        let resp: ThreadListResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.threads.is_empty());
        assert!(resp.next_page_token.is_none());
    }

    // Requirement: REQ-GMAIL-001 (Must)
    // Edge case: Thread with unknown fields preserved via flatten
    #[test]
    fn req_gmail_001_thread_unknown_fields_preserved() {
        let json_str = r#"{
            "id": "t1",
            "unknownField": "should be preserved"
        }"#;
        let thread: Thread = serde_json::from_str(json_str).unwrap();
        assert_eq!(thread.id, "t1");
        assert!(thread.extra.contains_key("unknownField"));
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-006 (Must): Message type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-006 (Must)
    // Acceptance: Message deserializes from Gmail API JSON
    #[test]
    fn req_gmail_006_message_deserialize() {
        let json_str = r#"{
            "id": "msg123",
            "threadId": "thread456",
            "labelIds": ["INBOX", "UNREAD"],
            "snippet": "Preview text",
            "historyId": "9999",
            "internalDate": "1704067200000",
            "sizeEstimate": 12345,
            "payload": {
                "mimeType": "text/plain",
                "headers": [
                    {"name": "From", "value": "sender@example.com"},
                    {"name": "Subject", "value": "Test Subject"},
                    {"name": "Date", "value": "Mon, 01 Jan 2024 00:00:00 +0000"}
                ],
                "body": {"size": 100, "data": "SGVsbG8gV29ybGQ="},
                "parts": []
            }
        }"#;
        let msg: Message = serde_json::from_str(json_str).unwrap();
        assert_eq!(msg.id, "msg123");
        assert_eq!(msg.thread_id, Some("thread456".to_string()));
        assert_eq!(msg.label_ids, vec!["INBOX", "UNREAD"]);
        assert_eq!(msg.size_estimate, Some(12345));

        let payload = msg.payload.unwrap();
        assert_eq!(payload.mime_type, Some("text/plain".to_string()));
        assert_eq!(payload.headers.len(), 3);
    }

    // Requirement: REQ-GMAIL-006 (Must)
    // Acceptance: Message round-trip serialization preserves structure
    #[test]
    fn req_gmail_006_message_roundtrip() {
        let msg = Message {
            id: "m1".to_string(),
            thread_id: Some("t1".to_string()),
            label_ids: vec!["INBOX".to_string()],
            snippet: Some("test".to_string()),
            history_id: None,
            internal_date: Some("1704067200000".to_string()),
            payload: None,
            size_estimate: Some(100),
            raw: None,
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "m1");
        assert_eq!(parsed.label_ids, vec!["INBOX"]);
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-006 (Must): Header extraction
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-006 (Must)
    // Acceptance: Headers extracted case-insensitively
    #[test]
    fn req_gmail_006_header_value_case_insensitive() {
        let part = MessagePart {
            part_id: None,
            mime_type: Some("text/plain".to_string()),
            filename: None,
            headers: vec![
                Header { name: "From".to_string(), value: "sender@example.com".to_string() },
                Header { name: "Subject".to_string(), value: "Test".to_string() },
                Header { name: "date".to_string(), value: "Mon, 01 Jan 2024".to_string() },
            ],
            body: None,
            parts: vec![],
            extra: HashMap::new(),
        };
        assert_eq!(header_value(&part, "from"), Some("sender@example.com".to_string()));
        assert_eq!(header_value(&part, "FROM"), Some("sender@example.com".to_string()));
        assert_eq!(header_value(&part, "subject"), Some("Test".to_string()));
        assert_eq!(header_value(&part, "Date"), Some("Mon, 01 Jan 2024".to_string()));
    }

    // Requirement: REQ-GMAIL-006 (Must)
    // Acceptance: Missing header returns None
    #[test]
    fn req_gmail_006_header_value_missing() {
        let part = MessagePart {
            part_id: None,
            mime_type: None,
            filename: None,
            headers: vec![],
            body: None,
            parts: vec![],
            extra: HashMap::new(),
        };
        assert_eq!(header_value(&part, "From"), None);
    }

    // Requirement: REQ-GMAIL-006 (Must)
    // Acceptance: has_header_name works case-insensitively
    #[test]
    fn req_gmail_006_has_header_name() {
        let names = vec!["From".to_string(), "Subject".to_string(), "Date".to_string()];
        assert!(has_header_name(&names, "from"));
        assert!(has_header_name(&names, "SUBJECT"));
        assert!(!has_header_name(&names, "CC"));
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-008 (Must): URL generation
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-008 (Must)
    // Acceptance: Thread URL follows Gmail format
    #[test]
    fn req_gmail_008_thread_url() {
        assert_eq!(
            thread_url("18abc123def"),
            "https://mail.google.com/mail/u/0/#all/18abc123def"
        );
    }

    // Requirement: REQ-GMAIL-008 (Must)
    // Edge case: Empty thread ID
    #[test]
    fn req_gmail_008_thread_url_empty() {
        assert_eq!(
            thread_url(""),
            "https://mail.google.com/mail/u/0/#all/"
        );
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-009 (Must): Label type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-009 (Must)
    // Acceptance: Label deserializes correctly
    #[test]
    fn req_gmail_009_label_deserialize() {
        let json_str = r#"{
            "id": "INBOX",
            "name": "INBOX",
            "type": "system",
            "messageListVisibility": "show",
            "labelListVisibility": "labelShow",
            "messagesTotal": 100,
            "messagesUnread": 5,
            "threadsTotal": 80,
            "threadsUnread": 3
        }"#;
        let label: Label = serde_json::from_str(json_str).unwrap();
        assert_eq!(label.id, "INBOX");
        assert_eq!(label.name, "INBOX");
        assert_eq!(label.r#type, Some("system".to_string()));
        assert_eq!(label.messages_total, Some(100));
        assert_eq!(label.messages_unread, Some(5));
    }

    // Requirement: REQ-GMAIL-009 (Must)
    // Acceptance: LabelListResponse deserializes
    #[test]
    fn req_gmail_009_label_list_response_deserialize() {
        let json_str = r#"{
            "labels": [
                {"id": "INBOX", "name": "INBOX", "type": "system"},
                {"id": "Label_1", "name": "Custom Label", "type": "user"}
            ]
        }"#;
        let resp: LabelListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.labels.len(), 2);
        assert_eq!(resp.labels[1].name, "Custom Label");
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-014 (Must): Batch types serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-014 (Must)
    // Acceptance: BatchModifyRequest serializes correctly
    #[test]
    fn req_gmail_014_batch_modify_request_serialize() {
        let req = BatchModifyRequest {
            ids: vec!["msg1".to_string(), "msg2".to_string()],
            add_label_ids: vec!["STARRED".to_string()],
            remove_label_ids: vec!["UNREAD".to_string()],
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(json_val["ids"], json!(["msg1", "msg2"]));
        assert_eq!(json_val["addLabelIds"], json!(["STARRED"]));
        assert_eq!(json_val["removeLabelIds"], json!(["UNREAD"]));
    }

    // Requirement: REQ-GMAIL-014 (Must)
    // Acceptance: BatchDeleteRequest serializes correctly
    #[test]
    fn req_gmail_014_batch_delete_request_serialize() {
        let req = BatchDeleteRequest {
            ids: vec!["msg1".to_string(), "msg2".to_string(), "msg3".to_string()],
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(json_val["ids"], json!(["msg1", "msg2", "msg3"]));
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-012 (Must): Watch types serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-012 (Must)
    // Acceptance: WatchRequest serializes correctly
    #[test]
    fn req_gmail_012_watch_request_serialize() {
        let req = WatchRequest {
            topic_name: "projects/my-project/topics/gmail".to_string(),
            label_ids: vec!["INBOX".to_string()],
            label_filter_action: Some("include".to_string()),
            extra: HashMap::new(),
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(json_val["topicName"], "projects/my-project/topics/gmail");
        assert_eq!(json_val["labelIds"], json!(["INBOX"]));
    }

    // Requirement: REQ-GMAIL-012 (Must)
    // Acceptance: WatchResponse deserializes correctly
    #[test]
    fn req_gmail_012_watch_response_deserialize() {
        let json_str = r#"{
            "historyId": "12345",
            "expiration": "1704153600000"
        }"#;
        let resp: WatchResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.history_id, Some("12345".to_string()));
        assert_eq!(resp.expiration, Some("1704153600000".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-015 (Must): Filter types serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-015 (Must)
    // Acceptance: Filter deserializes with criteria and action
    #[test]
    fn req_gmail_015_filter_deserialize() {
        let json_str = r#"{
            "id": "filter123",
            "criteria": {
                "from": "newsletter@example.com",
                "hasAttachment": true
            },
            "action": {
                "addLabelIds": ["Label_1"],
                "removeLabelIds": ["INBOX"]
            }
        }"#;
        let filter: Filter = serde_json::from_str(json_str).unwrap();
        assert_eq!(filter.id, Some("filter123".to_string()));
        let criteria = filter.criteria.unwrap();
        assert_eq!(criteria.from, Some("newsletter@example.com".to_string()));
        assert_eq!(criteria.has_attachment, Some(true));
        let action = filter.action.unwrap();
        assert_eq!(action.add_label_ids, vec!["Label_1"]);
        assert_eq!(action.remove_label_ids, vec!["INBOX"]);
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-019 (Must): Vacation settings
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-019 (Must)
    // Acceptance: VacationSettings round-trip
    #[test]
    fn req_gmail_019_vacation_settings_roundtrip() {
        let settings = VacationSettings {
            enable_auto_reply: Some(true),
            response_subject: Some("Out of Office".to_string()),
            response_body_plain_text: Some("I am away.".to_string()),
            response_body_html: None,
            restrict_to_contacts: Some(false),
            restrict_to_domain: Some(false),
            start_time: Some("1704067200000".to_string()),
            end_time: Some("1704326400000".to_string()),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&settings).unwrap();
        let parsed: VacationSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.enable_auto_reply, Some(true));
        assert_eq!(parsed.response_subject, Some("Out of Office".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-020 (Must): AutoForwarding settings
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-020 (Must)
    // Acceptance: AutoForwarding round-trip
    #[test]
    fn req_gmail_020_auto_forwarding_roundtrip() {
        let af = AutoForwarding {
            enabled: Some(true),
            email_address: Some("forward@example.com".to_string()),
            disposition: Some("leaveInInbox".to_string()),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&af).unwrap();
        let parsed: AutoForwarding = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.enabled, Some(true));
        assert_eq!(parsed.email_address, Some("forward@example.com".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-013 (Must): History types
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-013 (Must)
    // Acceptance: HistoryListResponse deserializes
    #[test]
    fn req_gmail_013_history_list_response_deserialize() {
        let json_str = r#"{
            "history": [
                {
                    "id": "12345",
                    "messages": [],
                    "messagesAdded": [],
                    "messagesDeleted": [],
                    "labelsAdded": [],
                    "labelsRemoved": []
                }
            ],
            "nextPageToken": "next123",
            "historyId": "12346"
        }"#;
        let resp: HistoryListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.history.len(), 1);
        assert_eq!(resp.history[0].id, "12345");
        assert_eq!(resp.next_page_token, Some("next123".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-016 (Must): ForwardingAddress type
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-016 (Must)
    // Acceptance: ForwardingAddress round-trip
    #[test]
    fn req_gmail_016_forwarding_address_roundtrip() {
        let fa = ForwardingAddress {
            forwarding_email: "forward@example.com".to_string(),
            verification_status: Some("accepted".to_string()),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&fa).unwrap();
        let parsed: ForwardingAddress = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.forwarding_email, "forward@example.com");
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-017 (Must): SendAs type
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-017 (Must)
    // Acceptance: SendAs round-trip
    #[test]
    fn req_gmail_017_sendas_roundtrip() {
        let sa = SendAs {
            send_as_email: "alias@example.com".to_string(),
            display_name: Some("My Alias".to_string()),
            reply_to_address: Some("reply@example.com".to_string()),
            signature: Some("<b>Signature</b>".to_string()),
            is_primary: Some(false),
            is_default: Some(false),
            treat_as_alias: Some(true),
            verification_status: Some("accepted".to_string()),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&sa).unwrap();
        let parsed: SendAs = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.send_as_email, "alias@example.com");
        assert_eq!(parsed.treat_as_alias, Some(true));
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-018 (Must): Delegate type
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-018 (Must)
    // Acceptance: Delegate round-trip
    #[test]
    fn req_gmail_018_delegate_roundtrip() {
        let d = Delegate {
            delegate_email: "delegate@example.com".to_string(),
            verification_status: Some("accepted".to_string()),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&d).unwrap();
        let parsed: Delegate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.delegate_email, "delegate@example.com");
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-011 (Must): Draft type
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-011 (Must)
    // Acceptance: Draft deserializes correctly
    #[test]
    fn req_gmail_011_draft_deserialize() {
        let json_str = r#"{
            "id": "draft_abc",
            "message": {
                "id": "msg_xyz",
                "labelIds": ["DRAFT"]
            }
        }"#;
        let draft: Draft = serde_json::from_str(json_str).unwrap();
        assert_eq!(draft.id, "draft_abc");
        let msg = draft.message.unwrap();
        assert_eq!(msg.id, "msg_xyz");
        assert_eq!(msg.label_ids, vec!["DRAFT"]);
    }
}
