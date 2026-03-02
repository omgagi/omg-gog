//! Drive list/search files with query building.
//! Implements the heuristic query detection from gogcli.

/// Build a Drive list query for a folder.
/// Adds parent folder constraint and trashed=false if not already present.
pub fn build_list_query(folder_id: &str, user_query: Option<&str>) -> String {
    let parent_clause = format!("'{}' in parents", folder_id);
    match user_query {
        Some(uq) if !uq.is_empty() => {
            if has_trashed_predicate(uq) {
                format!("{} and {}", parent_clause, uq)
            } else {
                format!("{} and {} and trashed = false", parent_clause, uq)
            }
        }
        _ => format!("{} and trashed = false", parent_clause),
    }
}

/// Build a Drive search query from user input.
/// If the input looks like Drive query language, pass it through.
/// Otherwise, wrap it in fullText contains '...' syntax.
pub fn build_search_query(text: &str, raw_query: bool) -> String {
    if text.is_empty() {
        return "trashed = false".to_string();
    }
    if raw_query || looks_like_drive_query_language(text) {
        if has_trashed_predicate(text) {
            text.to_string()
        } else {
            format!("{} and trashed = false", text)
        }
    } else {
        let escaped = escape_query_string(text);
        format!("fullText contains '{}' and trashed = false", escaped)
    }
}

/// Build a Drive filter query, appending trashed=false if needed.
pub fn build_filter_query(query: &str) -> String {
    if query.is_empty() {
        return "trashed = false".to_string();
    }
    if has_trashed_predicate(query) {
        query.to_string()
    } else {
        format!("{} and trashed = false", query)
    }
}

/// Heuristic: detect if a query string looks like Drive query language
/// rather than a plain text search.
pub fn looks_like_drive_query_language(query: &str) -> bool {
    // Known Drive query keywords
    if query == "sharedWithMe" || query.starts_with("sharedWithMe ") {
        return true;
    }
    // Field comparison patterns: field = 'value', field != 'value', field > 'value', etc.
    if query.contains(" = ") || query.contains(" != ") || query.contains(" > ") || query.contains(" < ") {
        return true;
    }
    // contains operator: name contains 'text', fullText contains 'text'
    if query.contains(" contains ") {
        return true;
    }
    // Membership pattern: 'id' in parents
    if query.contains(" in parents") || query.contains(" in owners") || query.contains(" in writers") || query.contains(" in readers") {
        return true;
    }
    // trashed predicate
    if query.contains("trashed") {
        return true;
    }
    false
}

/// Check if a query already contains a trashed predicate.
pub fn has_trashed_predicate(query: &str) -> bool {
    // We need to detect "trashed = ..." or "trashed != ..." as a query predicate,
    // but NOT "trashed" appearing inside a quoted string (e.g., "contains 'trashed items'").
    // Strategy: find each occurrence of "trashed" and check if it's followed by an operator
    // and NOT inside single quotes.
    let bytes = query.as_bytes();
    let trashed_bytes = b"trashed";
    let mut i = 0;
    let mut in_quote = false;
    while i < bytes.len() {
        if bytes[i] == b'\'' {
            in_quote = !in_quote;
            i += 1;
            continue;
        }
        if !in_quote && i + trashed_bytes.len() <= bytes.len()
            && &bytes[i..i + trashed_bytes.len()] == trashed_bytes
        {
            // Check character after "trashed" is whitespace or operator
            let after = i + trashed_bytes.len();
            if after >= bytes.len() {
                return false; // just "trashed" alone isn't a predicate
            }
            // Skip whitespace after "trashed"
            let mut j = after;
            while j < bytes.len() && bytes[j] == b' ' {
                j += 1;
            }
            // Check for operator: =, !=, <, >
            if j < bytes.len() && (bytes[j] == b'=' || bytes[j] == b'!' || bytes[j] == b'<' || bytes[j] == b'>') {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Escape special characters in a Drive query string value.
/// Backslashes and single quotes must be escaped.
pub fn escape_query_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-DRIVE-001 (Must): List query building
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: Default list query for root folder
    #[test]
    fn req_drive_001_list_query_root() {
        let q = build_list_query("root", None);
        assert!(q.contains("'root' in parents"));
        assert!(q.contains("trashed = false"));
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: List query for specific folder
    #[test]
    fn req_drive_001_list_query_specific_folder() {
        let q = build_list_query("folder_abc", None);
        assert!(q.contains("'folder_abc' in parents"));
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: List query with user query appended
    #[test]
    fn req_drive_001_list_query_with_user_query() {
        let q = build_list_query("root", Some("mimeType = 'application/pdf'"));
        assert!(q.contains("mimeType = 'application/pdf'"));
        assert!(q.contains("'root' in parents"));
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: List query does not double-add trashed predicate
    #[test]
    fn req_drive_001_list_query_no_double_trashed() {
        let q = build_list_query("root", Some("trashed = true"));
        // Should not add "and trashed = false" since user specified trashed
        let trashed_count = q.matches("trashed").count();
        assert_eq!(trashed_count, 1);
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-002 (Must): Search query building
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-002 (Must)
    // Acceptance: Plain text search wraps in fullText contains
    #[test]
    fn req_drive_002_search_query_plain_text() {
        let q = build_search_query("meeting notes", false);
        assert!(q.contains("fullText contains"));
        assert!(q.contains("meeting notes"));
        assert!(q.contains("trashed = false"));
    }

    // Requirement: REQ-DRIVE-002 (Must)
    // Acceptance: Raw query passes through
    #[test]
    fn req_drive_002_search_query_raw() {
        let q = build_search_query("mimeType = 'application/pdf'", true);
        assert!(q.contains("mimeType = 'application/pdf'"));
    }

    // Requirement: REQ-DRIVE-002 (Must)
    // Acceptance: Drive query language detected and passed through
    #[test]
    fn req_drive_002_search_query_drive_language() {
        let q = build_search_query("name contains 'report'", false);
        // Should be detected as Drive query language and passed through
        assert!(q.contains("name contains 'report'"));
    }

    // Requirement: REQ-DRIVE-002 (Must)
    // Acceptance: Empty search returns trashed=false
    #[test]
    fn req_drive_002_search_query_empty() {
        let q = build_search_query("", false);
        assert_eq!(q, "trashed = false");
    }

    // Requirement: REQ-DRIVE-002 (Must)
    // Edge case: sharedWithMe query detected
    #[test]
    fn req_drive_002_search_query_shared_with_me() {
        let q = build_search_query("sharedWithMe", false);
        assert!(q.contains("sharedWithMe"));
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-002 (Must): Drive query language detection
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-002 (Must)
    // Acceptance: Detects field comparison patterns
    #[test]
    fn req_drive_002_detects_field_comparison() {
        assert!(looks_like_drive_query_language("mimeType = 'application/pdf'"));
        assert!(looks_like_drive_query_language("name != 'test'"));
        assert!(looks_like_drive_query_language("modifiedTime > '2024-01-01'"));
    }

    // Requirement: REQ-DRIVE-002 (Must)
    // Acceptance: Detects contains pattern
    #[test]
    fn req_drive_002_detects_contains() {
        assert!(looks_like_drive_query_language("name contains 'report'"));
        assert!(looks_like_drive_query_language("fullText contains 'hello'"));
    }

    // Requirement: REQ-DRIVE-002 (Must)
    // Acceptance: Detects membership pattern
    #[test]
    fn req_drive_002_detects_membership() {
        assert!(looks_like_drive_query_language("'folder123' in parents"));
    }

    // Requirement: REQ-DRIVE-002 (Must)
    // Acceptance: Detects sharedWithMe
    #[test]
    fn req_drive_002_detects_shared_with_me() {
        assert!(looks_like_drive_query_language("sharedWithMe"));
    }

    // Requirement: REQ-DRIVE-002 (Must)
    // Acceptance: Plain text not detected as query language
    #[test]
    fn req_drive_002_plain_text_not_detected() {
        assert!(!looks_like_drive_query_language("meeting notes"));
        assert!(!looks_like_drive_query_language("hello world"));
        assert!(!looks_like_drive_query_language("project plan 2024"));
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-001 (Must): Trashed predicate detection
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: Detects trashed = true/false
    #[test]
    fn req_drive_001_has_trashed_predicate() {
        assert!(has_trashed_predicate("name = 'test' and trashed = false"));
        assert!(has_trashed_predicate("trashed = true"));
        assert!(has_trashed_predicate("trashed != false"));
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: Does not false-positive on "trashed" in string
    #[test]
    fn req_drive_001_no_false_positive_trashed() {
        assert!(!has_trashed_predicate("name contains 'trashed items'"));
        assert!(!has_trashed_predicate("fullText contains 'trashed'"));
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: No trashed predicate in plain query
    #[test]
    fn req_drive_001_no_trashed_predicate() {
        assert!(!has_trashed_predicate("mimeType = 'application/pdf'"));
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-002 (Must): Query string escaping
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-002 (Must)
    // Acceptance: Single quotes escaped
    #[test]
    fn req_drive_002_escape_single_quotes() {
        assert_eq!(escape_query_string("it's a test"), "it\\'s a test");
    }

    // Requirement: REQ-DRIVE-002 (Must)
    // Acceptance: Backslashes escaped
    #[test]
    fn req_drive_002_escape_backslashes() {
        assert_eq!(escape_query_string("path\\to\\file"), "path\\\\to\\\\file");
    }

    // Requirement: REQ-DRIVE-002 (Must)
    // Acceptance: Combined escaping
    #[test]
    fn req_drive_002_escape_combined() {
        assert_eq!(escape_query_string("it's a \\test"), "it\\'s a \\\\test");
    }

    // Requirement: REQ-DRIVE-002 (Must)
    // Edge case: Empty string
    #[test]
    fn req_drive_002_escape_empty() {
        assert_eq!(escape_query_string(""), "");
    }

    // Requirement: REQ-DRIVE-002 (Must)
    // Edge case: No special characters
    #[test]
    fn req_drive_002_escape_no_special() {
        assert_eq!(escape_query_string("hello world"), "hello world");
    }
}
