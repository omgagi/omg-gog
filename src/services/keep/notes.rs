//! Keep notes URL builders and helpers.

use super::types::Note;
use super::KEEP_BASE_URL;

/// Build URL for listing notes.
/// REQ-KEEP-001
pub fn build_notes_list_url(
    max: Option<u32>,
    page_token: Option<&str>,
    filter: Option<&str>,
) -> String {
    let base = format!("{}/notes", KEEP_BASE_URL);
    let mut params = Vec::new();
    if let Some(m) = max {
        params.push(format!("pageSize={}", m));
    }
    if let Some(token) = page_token {
        params.push(format!(
            "pageToken={}",
            url::form_urlencoded::byte_serialize(token.as_bytes()).collect::<String>()
        ));
    }
    if let Some(f) = filter {
        params.push(format!(
            "filter={}",
            url::form_urlencoded::byte_serialize(f.as_bytes()).collect::<String>()
        ));
    }
    if params.is_empty() {
        base
    } else {
        format!("{}?{}", base, params.join("&"))
    }
}

/// Encode a Google API resource name, encoding each path segment individually
/// to preserve `/` separators while encoding special characters within segments.
fn encode_resource_name(name: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    name.split('/')
        .map(|segment| utf8_percent_encode(segment, NON_ALPHANUMERIC).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// Build URL for getting a single note.
/// Handles both raw ID (e.g., "abc123") and full resource name (e.g., "notes/abc123").
/// REQ-KEEP-002
pub fn build_note_get_url(note_id: &str) -> String {
    let name = if note_id.starts_with("notes/") {
        note_id.to_string()
    } else {
        format!("notes/{}", note_id)
    };
    let encoded = encode_resource_name(&name);
    format!("{}/{}", KEEP_BASE_URL, encoded)
}

/// Client-side text search over notes.
/// Searches title and body text (case-insensitive).
/// REQ-KEEP-003
pub fn build_notes_search<'a>(notes: &'a [Note], query: &str) -> Vec<&'a Note> {
    let query_lower = query.to_lowercase();
    notes
        .iter()
        .filter(|note| {
            // Search title
            if let Some(ref title) = note.title {
                if title.to_lowercase().contains(&query_lower) {
                    return true;
                }
            }
            // Search body text
            if let Some(ref body) = note.body {
                if let Some(ref text_content) = body.text {
                    if let Some(ref text) = text_content.text {
                        if text.to_lowercase().contains(&query_lower) {
                            return true;
                        }
                    }
                }
                // Search list items
                if let Some(ref list_content) = body.list {
                    for item in &list_content.list_items {
                        if list_item_contains(item, &query_lower) {
                            return true;
                        }
                    }
                }
            }
            false
        })
        .collect()
}

/// Recursively check if a list item or its children contain the query.
fn list_item_contains(item: &super::types::ListItem, query_lower: &str) -> bool {
    if let Some(ref text_content) = item.text {
        if let Some(ref text) = text_content.text {
            if text.to_lowercase().contains(query_lower) {
                return true;
            }
        }
    }
    for child in &item.child_list_items {
        if list_item_contains(child, query_lower) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::*;
    use std::collections::HashMap;

    // ---------------------------------------------------------------
    // REQ-KEEP-001 (Must): Notes list URL
    // ---------------------------------------------------------------

    // Requirement: REQ-KEEP-001 (Must)
    // Acceptance: Notes list URL with no parameters
    #[test]
    fn req_keep_001_notes_list_url_default() {
        // REQ-KEEP-001
        let url = build_notes_list_url(None, None, None);
        assert_eq!(url, "https://keep.googleapis.com/v1/notes");
    }

    // Requirement: REQ-KEEP-001 (Must)
    // Acceptance: Notes list URL with max results
    #[test]
    fn req_keep_001_notes_list_url_max() {
        // REQ-KEEP-001
        let url = build_notes_list_url(Some(20), None, None);
        assert!(url.contains("pageSize=20"));
    }

    // Requirement: REQ-KEEP-001 (Must)
    // Acceptance: Notes list URL with page token
    #[test]
    fn req_keep_001_notes_list_url_page_token() {
        // REQ-KEEP-001
        let url = build_notes_list_url(None, Some("abc123"), None);
        assert!(url.contains("pageToken=abc123"));
    }

    // Requirement: REQ-KEEP-001 (Must)
    // Acceptance: Notes list URL with filter
    #[test]
    fn req_keep_001_notes_list_url_filter() {
        // REQ-KEEP-001
        let url = build_notes_list_url(None, None, Some("trashed = false"));
        assert!(url.contains("filter="));
        assert!(url.contains("trashed"));
    }

    // Requirement: REQ-KEEP-001 (Must)
    // Acceptance: Notes list URL with all parameters
    #[test]
    fn req_keep_001_notes_list_url_all_params() {
        // REQ-KEEP-001
        let url = build_notes_list_url(Some(10), Some("token"), Some("trashed = false"));
        assert!(url.contains("pageSize=10"));
        assert!(url.contains("pageToken=token"));
        assert!(url.contains("filter="));
    }

    // ---------------------------------------------------------------
    // REQ-KEEP-002 (Must): Note get URL
    // ---------------------------------------------------------------

    // Requirement: REQ-KEEP-002 (Must)
    // Acceptance: Note get URL with raw ID
    #[test]
    fn req_keep_002_note_get_url_raw_id() {
        // REQ-KEEP-002
        let url = build_note_get_url("abc123");
        assert_eq!(url, "https://keep.googleapis.com/v1/notes/abc123");
    }

    // Requirement: REQ-KEEP-002 (Must)
    // Acceptance: Note get URL with full resource name
    #[test]
    fn req_keep_002_note_get_url_full_name() {
        // REQ-KEEP-002
        let url = build_note_get_url("notes/abc123");
        assert_eq!(url, "https://keep.googleapis.com/v1/notes/abc123");
    }

    // ---------------------------------------------------------------
    // REQ-KEEP-003 (Must): Notes search
    // ---------------------------------------------------------------

    // Requirement: REQ-KEEP-003 (Must)
    // Acceptance: Search finds notes by title
    #[test]
    fn req_keep_003_search_by_title() {
        // REQ-KEEP-003
        let notes = vec![
            Note {
                name: Some("notes/a".to_string()),
                title: Some("Shopping List".to_string()),
                body: None,
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

        let results = build_notes_search(&notes, "shopping");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, Some("notes/a".to_string()));
    }

    // Requirement: REQ-KEEP-003 (Must)
    // Acceptance: Search finds notes by body text
    #[test]
    fn req_keep_003_search_by_body() {
        // REQ-KEEP-003
        let notes = vec![Note {
            name: Some("notes/c".to_string()),
            title: Some("Untitled".to_string()),
            body: Some(NoteBody {
                text: Some(TextContent {
                    text: Some("Buy milk and eggs".to_string()),
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
        }];

        let results = build_notes_search(&notes, "milk");
        assert_eq!(results.len(), 1);
    }

    // Requirement: REQ-KEEP-003 (Must)
    // Acceptance: Search is case-insensitive
    #[test]
    fn req_keep_003_search_case_insensitive() {
        // REQ-KEEP-003
        let notes = vec![Note {
            name: Some("notes/d".to_string()),
            title: Some("Important MEETING".to_string()),
            body: None,
            create_time: None,
            update_time: None,
            trash_time: None,
            trashed: None,
            permissions: vec![],
            attachments: vec![],
            extra: HashMap::new(),
        }];

        let results = build_notes_search(&notes, "meeting");
        assert_eq!(results.len(), 1);
    }

    // Requirement: REQ-KEEP-003 (Must)
    // Acceptance: Search returns empty for no matches
    #[test]
    fn req_keep_003_search_no_match() {
        // REQ-KEEP-003
        let notes = vec![Note {
            name: Some("notes/e".to_string()),
            title: Some("Shopping".to_string()),
            body: None,
            create_time: None,
            update_time: None,
            trash_time: None,
            trashed: None,
            permissions: vec![],
            attachments: vec![],
            extra: HashMap::new(),
        }];

        let results = build_notes_search(&notes, "nonexistent");
        assert!(results.is_empty());
    }

    // Requirement: REQ-KEEP-003 (Must)
    // Acceptance: Search finds notes in list items
    #[test]
    fn req_keep_003_search_list_items() {
        // REQ-KEEP-003
        let notes = vec![Note {
            name: Some("notes/f".to_string()),
            title: Some("Todo".to_string()),
            body: Some(NoteBody {
                text: None,
                list: Some(ListContent {
                    list_items: vec![
                        ListItem {
                            text: Some(TextContent {
                                text: Some("Buy groceries".to_string()),
                                extra: HashMap::new(),
                            }),
                            checked: Some(false),
                            child_list_items: vec![],
                            extra: HashMap::new(),
                        },
                        ListItem {
                            text: Some(TextContent {
                                text: Some("Clean house".to_string()),
                                extra: HashMap::new(),
                            }),
                            checked: Some(true),
                            child_list_items: vec![],
                            extra: HashMap::new(),
                        },
                    ],
                    extra: HashMap::new(),
                }),
                extra: HashMap::new(),
            }),
            create_time: None,
            update_time: None,
            trash_time: None,
            trashed: None,
            permissions: vec![],
            attachments: vec![],
            extra: HashMap::new(),
        }];

        let results = build_notes_search(&notes, "groceries");
        assert_eq!(results.len(), 1);
    }
}
