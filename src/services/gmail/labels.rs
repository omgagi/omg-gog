//! Gmail label CRUD and name-to-ID resolution.

use super::types::*;

/// Build URL for listing all labels.
pub fn build_labels_list_url() -> String {
    format!("{}/users/me/labels", GMAIL_BASE_URL)
}

/// Build URL for getting a label by ID.
pub fn build_label_get_url(label_id: &str) -> String {
    format!("{}/users/me/labels/{}", GMAIL_BASE_URL, label_id)
}

/// Build URL and body for creating a new label.
pub fn build_label_create_request(name: &str) -> (String, serde_json::Value) {
    let url = format!("{}/users/me/labels", GMAIL_BASE_URL);
    let body = serde_json::json!({
        "name": name,
    });
    (url, body)
}

/// Build URL for deleting a label.
pub fn build_label_delete_url(label_id: &str) -> String {
    format!("{}/users/me/labels/{}", GMAIL_BASE_URL, label_id)
}

/// Resolve a label name to its ID. System labels like INBOX, SENT, etc.
/// can be looked up by name directly; user labels require fetching the full label list.
pub fn resolve_label_id(name_or_id: &str, labels: &[Label]) -> Option<String> {
    // First check if it matches a label by ID
    if let Some(label) = labels.iter().find(|l| l.id == name_or_id) {
        return Some(label.id.clone());
    }
    // Then check by name (case-insensitive)
    if let Some(label) = labels
        .iter()
        .find(|l| l.name.eq_ignore_ascii_case(name_or_id))
    {
        return Some(label.id.clone());
    }
    None
}

/// Well-known system label IDs.
pub const SYSTEM_LABELS: &[(&str, &str)] = &[
    ("INBOX", "INBOX"),
    ("SENT", "SENT"),
    ("TRASH", "TRASH"),
    ("SPAM", "SPAM"),
    ("DRAFT", "DRAFT"),
    ("STARRED", "STARRED"),
    ("UNREAD", "UNREAD"),
    ("IMPORTANT", "IMPORTANT"),
    ("CATEGORY_PERSONAL", "CATEGORY_PERSONAL"),
    ("CATEGORY_SOCIAL", "CATEGORY_SOCIAL"),
    ("CATEGORY_PROMOTIONS", "CATEGORY_PROMOTIONS"),
    ("CATEGORY_UPDATES", "CATEGORY_UPDATES"),
    ("CATEGORY_FORUMS", "CATEGORY_FORUMS"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ---------------------------------------------------------------
    // REQ-GMAIL-009 (Must): Label CRUD URLs
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-009 (Must)
    // Acceptance: Labels list URL
    #[test]
    fn req_gmail_009_labels_list_url() {
        let url = build_labels_list_url();
        assert!(url.contains("users/me/labels"));
    }

    // Requirement: REQ-GMAIL-009 (Must)
    // Acceptance: Label get URL includes label ID
    #[test]
    fn req_gmail_009_label_get_url() {
        let url = build_label_get_url("Label_123");
        assert!(url.contains("users/me/labels/Label_123"));
    }

    // Requirement: REQ-GMAIL-009 (Must)
    // Acceptance: Label create request body
    #[test]
    fn req_gmail_009_label_create_request() {
        let (url, body) = build_label_create_request("My Label");
        assert!(url.contains("users/me/labels"));
        assert_eq!(body["name"], "My Label");
    }

    // Requirement: REQ-GMAIL-009 (Must)
    // Acceptance: Label delete URL
    #[test]
    fn req_gmail_009_label_delete_url() {
        let url = build_label_delete_url("Label_123");
        assert!(url.contains("users/me/labels/Label_123"));
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-009 (Must): Label name resolution
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-009 (Must)
    // Acceptance: Resolves system label by name
    #[test]
    fn req_gmail_009_resolve_system_label() {
        let labels = vec![Label {
            id: "INBOX".to_string(),
            name: "INBOX".to_string(),
            r#type: Some("system".to_string()),
            message_list_visibility: None,
            label_list_visibility: None,
            messages_total: None,
            messages_unread: None,
            threads_total: None,
            threads_unread: None,
            color: None,
            extra: HashMap::new(),
        }];
        let id = resolve_label_id("INBOX", &labels).unwrap();
        assert_eq!(id, "INBOX");
    }

    // Requirement: REQ-GMAIL-009 (Must)
    // Acceptance: Resolves user label by name
    #[test]
    fn req_gmail_009_resolve_user_label_by_name() {
        let labels = vec![Label {
            id: "Label_1".to_string(),
            name: "Work".to_string(),
            r#type: Some("user".to_string()),
            message_list_visibility: None,
            label_list_visibility: None,
            messages_total: None,
            messages_unread: None,
            threads_total: None,
            threads_unread: None,
            color: None,
            extra: HashMap::new(),
        }];
        let id = resolve_label_id("Work", &labels).unwrap();
        assert_eq!(id, "Label_1");
    }

    // Requirement: REQ-GMAIL-009 (Must)
    // Acceptance: Resolves label by ID directly
    #[test]
    fn req_gmail_009_resolve_label_by_id() {
        let labels = vec![Label {
            id: "Label_1".to_string(),
            name: "Work".to_string(),
            r#type: Some("user".to_string()),
            message_list_visibility: None,
            label_list_visibility: None,
            messages_total: None,
            messages_unread: None,
            threads_total: None,
            threads_unread: None,
            color: None,
            extra: HashMap::new(),
        }];
        let id = resolve_label_id("Label_1", &labels).unwrap();
        assert_eq!(id, "Label_1");
    }

    // Requirement: REQ-GMAIL-009 (Must)
    // Edge case: Label not found
    #[test]
    fn req_gmail_009_resolve_label_not_found() {
        let labels: Vec<Label> = vec![];
        assert!(resolve_label_id("Nonexistent", &labels).is_none());
    }

    // Requirement: REQ-GMAIL-009 (Must)
    // Edge case: Case-insensitive label name matching
    #[test]
    fn req_gmail_009_resolve_label_case_insensitive() {
        let labels = vec![Label {
            id: "Label_1".to_string(),
            name: "Work Projects".to_string(),
            r#type: Some("user".to_string()),
            message_list_visibility: None,
            label_list_visibility: None,
            messages_total: None,
            messages_unread: None,
            threads_total: None,
            threads_unread: None,
            color: None,
            extra: HashMap::new(),
        }];
        let id = resolve_label_id("work projects", &labels).unwrap();
        assert_eq!(id, "Label_1");
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-009 (Must): System labels constant
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-009 (Must)
    // Acceptance: System labels list includes all standard Gmail labels
    #[test]
    fn req_gmail_009_system_labels_complete() {
        let names: Vec<&str> = SYSTEM_LABELS.iter().map(|(name, _)| *name).collect();
        assert!(names.contains(&"INBOX"));
        assert!(names.contains(&"SENT"));
        assert!(names.contains(&"TRASH"));
        assert!(names.contains(&"SPAM"));
        assert!(names.contains(&"DRAFT"));
        assert!(names.contains(&"STARRED"));
        assert!(names.contains(&"UNREAD"));
        assert!(names.contains(&"IMPORTANT"));
    }
}
