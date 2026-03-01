//! Gmail thread get/modify operations.

use super::types::*;

/// Build URL for getting a thread by ID.
pub fn build_thread_get_url(thread_id: &str) -> String {
    format!("{}/users/me/threads/{}", GMAIL_BASE_URL, thread_id)
}

/// Build URL and body for modifying thread labels.
pub fn build_thread_modify_request(
    thread_id: &str,
    add_labels: &[String],
    remove_labels: &[String],
) -> (String, serde_json::Value) {
    let url = format!("{}/users/me/threads/{}/modify", GMAIL_BASE_URL, thread_id);
    let body = serde_json::json!({
        "addLabelIds": add_labels,
        "removeLabelIds": remove_labels,
    });
    (url, body)
}

/// Extract the display message from a thread (newest or oldest by date).
pub fn pick_display_message(thread: &Thread, oldest: bool) -> Option<&Message> {
    if thread.messages.is_empty() {
        return None;
    }
    if thread.messages.len() == 1 {
        return Some(&thread.messages[0]);
    }
    if oldest {
        // Return the message with the smallest internal_date
        thread.messages.iter().min_by_key(|m| {
            m.internal_date.as_ref().and_then(|d| d.parse::<i64>().ok()).unwrap_or(0)
        })
    } else {
        // Return the message with the largest internal_date
        thread.messages.iter().max_by_key(|m| {
            m.internal_date.as_ref().and_then(|d| d.parse::<i64>().ok()).unwrap_or(0)
        })
    }
}

/// Get the internal date of a message as milliseconds.
pub fn message_date_millis(msg: &Message) -> Option<i64> {
    msg.internal_date.as_ref().and_then(|d| d.parse::<i64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ---------------------------------------------------------------
    // REQ-GMAIL-003 (Must): Thread get URL
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-003 (Must)
    // Acceptance: Thread get URL is correctly formed
    #[test]
    fn req_gmail_003_thread_get_url() {
        let url = build_thread_get_url("thread_abc123");
        assert!(url.contains("users/me/threads/thread_abc123"));
    }

    // Requirement: REQ-GMAIL-003 (Must)
    // Edge case: Empty thread ID
    #[test]
    fn req_gmail_003_thread_get_url_empty() {
        let url = build_thread_get_url("");
        assert!(url.contains("users/me/threads/"));
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-004 (Must): Thread modify
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-004 (Must)
    // Acceptance: Thread modify builds correct URL and body
    #[test]
    fn req_gmail_004_thread_modify_request() {
        let (url, body) = build_thread_modify_request(
            "thread123",
            &["STARRED".to_string()],
            &["UNREAD".to_string()],
        );
        assert!(url.contains("threads/thread123/modify"));
        assert_eq!(body["addLabelIds"], serde_json::json!(["STARRED"]));
        assert_eq!(body["removeLabelIds"], serde_json::json!(["UNREAD"]));
    }

    // Requirement: REQ-GMAIL-004 (Must)
    // Acceptance: Thread modify with empty labels
    #[test]
    fn req_gmail_004_thread_modify_empty_labels() {
        let (url, body) = build_thread_modify_request("thread123", &[], &[]);
        assert!(url.contains("threads/thread123/modify"));
        assert_eq!(body["addLabelIds"], serde_json::json!([]));
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-001 (Must): Display message selection
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-001 (Must)
    // Acceptance: pick_display_message returns newest by default
    #[test]
    fn req_gmail_001_pick_newest_message() {
        let thread = Thread {
            id: "t1".to_string(),
            snippet: None,
            history_id: None,
            messages: vec![
                Message {
                    id: "msg1".to_string(),
                    thread_id: None,
                    label_ids: vec![],
                    snippet: None,
                    history_id: None,
                    internal_date: Some("1704067200000".to_string()),
                    payload: None,
                    size_estimate: None,
                    raw: None,
                    extra: HashMap::new(),
                },
                Message {
                    id: "msg2".to_string(),
                    thread_id: None,
                    label_ids: vec![],
                    snippet: None,
                    history_id: None,
                    internal_date: Some("1704153600000".to_string()),
                    payload: None,
                    size_estimate: None,
                    raw: None,
                    extra: HashMap::new(),
                },
            ],
            extra: HashMap::new(),
        };
        let msg = pick_display_message(&thread, false).unwrap();
        assert_eq!(msg.id, "msg2"); // newer
    }

    // Requirement: REQ-GMAIL-001 (Must)
    // Acceptance: pick_display_message with oldest flag returns first
    #[test]
    fn req_gmail_001_pick_oldest_message() {
        let thread = Thread {
            id: "t1".to_string(),
            snippet: None,
            history_id: None,
            messages: vec![
                Message {
                    id: "msg1".to_string(),
                    thread_id: None,
                    label_ids: vec![],
                    snippet: None,
                    history_id: None,
                    internal_date: Some("1704067200000".to_string()),
                    payload: None,
                    size_estimate: None,
                    raw: None,
                    extra: HashMap::new(),
                },
                Message {
                    id: "msg2".to_string(),
                    thread_id: None,
                    label_ids: vec![],
                    snippet: None,
                    history_id: None,
                    internal_date: Some("1704153600000".to_string()),
                    payload: None,
                    size_estimate: None,
                    raw: None,
                    extra: HashMap::new(),
                },
            ],
            extra: HashMap::new(),
        };
        let msg = pick_display_message(&thread, true).unwrap();
        assert_eq!(msg.id, "msg1"); // older
    }

    // Requirement: REQ-GMAIL-001 (Must)
    // Edge case: Empty thread messages
    #[test]
    fn req_gmail_001_pick_message_empty_thread() {
        let thread = Thread {
            id: "t1".to_string(),
            snippet: None,
            history_id: None,
            messages: vec![],
            extra: HashMap::new(),
        };
        assert!(pick_display_message(&thread, false).is_none());
    }

    // Requirement: REQ-GMAIL-001 (Must)
    // Edge case: Single message thread
    #[test]
    fn req_gmail_001_pick_message_single() {
        let thread = Thread {
            id: "t1".to_string(),
            snippet: None,
            history_id: None,
            messages: vec![Message {
                id: "msg1".to_string(),
                thread_id: None,
                label_ids: vec![],
                snippet: None,
                history_id: None,
                internal_date: Some("1704067200000".to_string()),
                payload: None,
                size_estimate: None,
                raw: None,
                extra: HashMap::new(),
            }],
            extra: HashMap::new(),
        };
        let msg = pick_display_message(&thread, false).unwrap();
        assert_eq!(msg.id, "msg1");
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-003 (Must): Message date millis extraction
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-003 (Must)
    // Acceptance: Extracts internal date as millis
    #[test]
    fn req_gmail_003_message_date_millis() {
        let msg = Message {
            id: "m1".to_string(),
            thread_id: None,
            label_ids: vec![],
            snippet: None,
            history_id: None,
            internal_date: Some("1704067200000".to_string()),
            payload: None,
            size_estimate: None,
            raw: None,
            extra: HashMap::new(),
        };
        let millis = message_date_millis(&msg).unwrap();
        assert_eq!(millis, 1704067200000);
    }

    // Requirement: REQ-GMAIL-003 (Must)
    // Edge case: Missing internal date
    #[test]
    fn req_gmail_003_message_date_millis_missing() {
        let msg = Message {
            id: "m1".to_string(),
            thread_id: None,
            label_ids: vec![],
            snippet: None,
            history_id: None,
            internal_date: None,
            payload: None,
            size_estimate: None,
            raw: None,
            extra: HashMap::new(),
        };
        assert!(message_date_millis(&msg).is_none());
    }
}
