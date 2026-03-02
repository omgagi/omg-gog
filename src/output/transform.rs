use serde_json::Value;

/// Known envelope/meta keys that --results-only strips.
pub const META_KEYS: &[&str] = &[
    "nextPageToken",
    "next_cursor",
    "has_more",
    "count",
    "query",
    "dry_run",
    "dryRun",
    "op",
    "action",
    "note",
];

/// Known result keys for fallback unwrapping.
pub const KNOWN_RESULT_KEYS: &[&str] = &[
    "files",
    "threads",
    "messages",
    "labels",
    "events",
    "calendars",
    "courses",
    "topics",
    "announcements",
    "materials",
    "coursework",
    "submissions",
    "invitations",
    "guardians",
    "notes",
    "contacts",
    "people",
    "tasks",
    "lists",
    "groups",
    "members",
    "drives",
    "rules",
    "colors",
    "spaces",
    "request",
];

/// Unwrap the primary result from a JSON envelope.
/// Strips known meta keys, returns the single remaining value if exactly one non-meta key.
pub fn unwrap_primary(value: Value) -> Value {
    let obj = match value.as_object() {
        Some(o) => o,
        None => return value, // Non-object passes through
    };

    if obj.is_empty() {
        return value;
    }

    // Check for explicit "results" key first
    if let Some(results) = obj.get("results") {
        return results.clone();
    }

    // Partition keys into meta and non-meta
    let non_meta: Vec<(&String, &Value)> = obj
        .iter()
        .filter(|(k, _)| !META_KEYS.contains(&k.as_str()))
        .collect();

    if non_meta.is_empty() {
        return value; // All keys are meta
    }

    // If exactly one non-meta key, return its value
    if non_meta.len() == 1 {
        return non_meta[0].1.clone();
    }

    // Multiple non-meta keys: prefer known result keys that are arrays
    for (key, val) in &non_meta {
        if KNOWN_RESULT_KEYS.contains(&key.as_str()) && val.is_array() {
            return (*val).clone();
        }
    }

    // Fallback: look for any array value among non-meta keys
    for (_, val) in &non_meta {
        if val.is_array() {
            return (*val).clone();
        }
    }

    // No clear primary result, return original
    value
}

/// Select specific fields from a JSON value (object or array of objects).
/// Supports dot-path notation for nested fields.
pub fn select_fields(value: Value, fields: &[String]) -> Value {
    match &value {
        Value::Object(_) => select_fields_from_object(&value, fields),
        Value::Array(arr) => {
            let projected: Vec<Value> = arr
                .iter()
                .map(|item| select_fields_from_object(item, fields))
                .collect();
            Value::Array(projected)
        }
        _ => value, // Non-object/array passes through
    }
}

fn select_fields_from_object(value: &Value, fields: &[String]) -> Value {
    if !value.is_object() {
        return value.clone();
    }
    let mut result = serde_json::Map::new();
    for field in fields {
        if let Some(val) = get_at_path(value, field) {
            result.insert(field.clone(), val);
        }
    }
    Value::Object(result)
}

/// Get a value at a dot-separated path.
pub fn get_at_path(value: &Value, path: &str) -> Option<Value> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parts: Vec<&str> = trimmed.split('.').collect();
    let mut current = value;
    for part in parts {
        let part = part.trim();
        match current {
            Value::Object(map) => {
                current = map.get(part)?;
            }
            Value::Array(arr) => {
                let index: usize = part.parse().ok()?;
                current = arr.get(index)?;
            }
            _ => return None,
        }
    }
    Some(current.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---------------------------------------------------------------
    // REQ-OUTPUT-002 (Must): --results-only strips envelope fields
    // ---------------------------------------------------------------

    // Requirement: REQ-OUTPUT-002 (Must)
    // Acceptance: Strips pagination metadata
    #[test]
    fn req_output_002_strips_next_page_token() {
        let input = json!({
            "threads": [{"id": "1"}, {"id": "2"}],
            "nextPageToken": "abc123"
        });
        let result = unwrap_primary(input);
        // Should return just the threads array
        assert_eq!(result, json!([{"id": "1"}, {"id": "2"}]));
    }

    // Requirement: REQ-OUTPUT-002 (Must)
    // Acceptance: Unwraps single non-meta key
    #[test]
    fn req_output_002_unwraps_single_result_key() {
        let input = json!({
            "files": [{"id": "f1"}],
            "nextPageToken": "page2",
            "count": 1
        });
        let result = unwrap_primary(input);
        assert_eq!(result, json!([{"id": "f1"}]));
    }

    // Requirement: REQ-OUTPUT-002 (Must)
    // Acceptance: Explicit "results" key takes priority
    #[test]
    fn req_output_002_explicit_results_key() {
        let input = json!({
            "results": [{"id": "r1"}],
            "extra_data": "something"
        });
        let result = unwrap_primary(input);
        assert_eq!(result, json!([{"id": "r1"}]));
    }

    // Requirement: REQ-OUTPUT-002 (Must)
    // Acceptance: Falls back to known result keys
    #[test]
    fn req_output_002_fallback_to_known_keys() {
        let input = json!({
            "events": [{"id": "e1"}],
            "other_stuff": "data",
            "more_stuff": "data2"
        });
        let result = unwrap_primary(input);
        assert_eq!(result, json!([{"id": "e1"}]));
    }

    // Requirement: REQ-OUTPUT-002 (Must)
    // Acceptance: Non-object input returned as-is
    #[test]
    fn req_output_002_non_object_passthrough() {
        let input = json!([1, 2, 3]);
        let result = unwrap_primary(input.clone());
        assert_eq!(result, input);
    }

    // Requirement: REQ-OUTPUT-002 (Must)
    // Acceptance: Multiple non-meta keys with array prefers array
    #[test]
    fn req_output_002_prefers_array_candidate() {
        let input = json!({
            "labels": [{"id": "l1"}],
            "total": 1,
            "status": "ok"
        });
        let result = unwrap_primary(input);
        // "labels" is an array and a known key, should be preferred
        assert_eq!(result, json!([{"id": "l1"}]));
    }

    // Requirement: REQ-OUTPUT-002 (Must)
    // Edge case: All keys are meta keys
    #[test]
    fn req_output_002_edge_all_meta_returns_original() {
        let input = json!({
            "nextPageToken": "abc",
            "count": 0,
            "query": "test"
        });
        let result = unwrap_primary(input.clone());
        // When no result keys remain, return original
        assert_eq!(result, input);
    }

    // ---------------------------------------------------------------
    // REQ-OUTPUT-003 (Must): --select field filtering
    // ---------------------------------------------------------------

    // Requirement: REQ-OUTPUT-003 (Must)
    // Acceptance: Selects specific fields from object
    #[test]
    fn req_output_003_select_fields_from_object() {
        let input = json!({"id": "1", "name": "test", "email": "a@b.com", "age": 30});
        let result = select_fields(input, &["id".to_string(), "name".to_string()]);
        assert_eq!(result, json!({"id": "1", "name": "test"}));
    }

    // Requirement: REQ-OUTPUT-003 (Must)
    // Acceptance: Selects specific fields from array of objects
    #[test]
    fn req_output_003_select_fields_from_array() {
        let input = json!([
            {"id": "1", "name": "a", "extra": "x"},
            {"id": "2", "name": "b", "extra": "y"}
        ]);
        let result = select_fields(input, &["id".to_string(), "name".to_string()]);
        assert_eq!(result, json!([
            {"id": "1", "name": "a"},
            {"id": "2", "name": "b"}
        ]));
    }

    // Requirement: REQ-OUTPUT-003 (Must)
    // Acceptance: Supports dot-path notation (e.g., file.id)
    #[test]
    fn req_output_003_dot_path_selection() {
        let input = json!({"file": {"id": "f1", "name": "doc"}, "other": "data"});
        let result = select_fields(input, &["file.id".to_string()]);
        assert_eq!(result, json!({"file.id": "f1"}));
    }

    // Requirement: REQ-OUTPUT-003 (Must)
    // Acceptance: Missing fields silently omitted
    #[test]
    fn req_output_003_missing_fields_omitted() {
        let input = json!({"id": "1", "name": "test"});
        let result = select_fields(input, &["id".to_string(), "nonexistent".to_string()]);
        assert_eq!(result, json!({"id": "1"}));
    }

    // Requirement: REQ-OUTPUT-003 (Must)
    // Edge case: Empty select list
    #[test]
    fn req_output_003_edge_empty_select() {
        let input = json!({"id": "1", "name": "test"});
        let result = select_fields(input, &[]);
        assert_eq!(result, json!({}));
    }

    // Requirement: REQ-OUTPUT-003 (Must)
    // Edge case: Non-object value passed through
    #[test]
    fn req_output_003_edge_non_object_passthrough() {
        let input = json!("just a string");
        let result = select_fields(input.clone(), &["id".to_string()]);
        assert_eq!(result, input);
    }

    // ---------------------------------------------------------------
    // REQ-OUTPUT-003 (Must): get_at_path
    // ---------------------------------------------------------------

    // Requirement: REQ-OUTPUT-003 (Must)
    // Acceptance: Simple key lookup
    #[test]
    fn req_output_003_get_at_path_simple() {
        let input = json!({"id": "1"});
        assert_eq!(get_at_path(&input, "id"), Some(json!("1")));
    }

    // Requirement: REQ-OUTPUT-003 (Must)
    // Acceptance: Nested path lookup
    #[test]
    fn req_output_003_get_at_path_nested() {
        let input = json!({"file": {"metadata": {"name": "doc.txt"}}});
        assert_eq!(get_at_path(&input, "file.metadata.name"), Some(json!("doc.txt")));
    }

    // Requirement: REQ-OUTPUT-003 (Must)
    // Edge case: Empty path
    #[test]
    fn req_output_003_get_at_path_empty() {
        let input = json!({"id": "1"});
        assert_eq!(get_at_path(&input, ""), None);
    }

    // Requirement: REQ-OUTPUT-003 (Must)
    // Edge case: Missing intermediate key
    #[test]
    fn req_output_003_get_at_path_missing_intermediate() {
        let input = json!({"id": "1"});
        assert_eq!(get_at_path(&input, "file.id"), None);
    }

    // Requirement: REQ-OUTPUT-003 (Must)
    // Edge case: Array index in path
    #[test]
    fn req_output_003_get_at_path_array_index() {
        let input = json!({"items": ["a", "b", "c"]});
        assert_eq!(get_at_path(&input, "items.1"), Some(json!("b")));
    }

    // Requirement: REQ-OUTPUT-003 (Must)
    // Edge case: Out-of-bounds array index
    #[test]
    fn req_output_003_get_at_path_oob_index() {
        let input = json!({"items": ["a"]});
        assert_eq!(get_at_path(&input, "items.5"), None);
    }

    // ---------------------------------------------------------------
    // REQ-OUTPUT-001 (Must): Output mode resolution
    // ---------------------------------------------------------------

    // Requirement: REQ-OUTPUT-001 (Must)
    // Acceptance: JSON mode via flag
    #[test]
    fn req_output_001_json_mode_from_flag() {
        let mode = crate::output::resolve_mode(true, false, true).unwrap();
        assert_eq!(mode, crate::output::OutputMode::Json);
    }

    // Requirement: REQ-OUTPUT-001 (Must)
    // Acceptance: Plain mode via flag
    #[test]
    fn req_output_001_plain_mode_from_flag() {
        let mode = crate::output::resolve_mode(false, true, true).unwrap();
        assert_eq!(mode, crate::output::OutputMode::Plain);
    }

    // Requirement: REQ-OUTPUT-001 (Must)
    // Acceptance: Text mode default on TTY
    #[test]
    fn req_output_001_text_mode_default_tty() {
        let mode = crate::output::resolve_mode(false, false, true).unwrap();
        assert_eq!(mode, crate::output::OutputMode::Text);
    }

    // Requirement: REQ-OUTPUT-001 (Must)
    // Acceptance: JSON and plain mutually exclusive
    #[test]
    fn req_output_001_json_plain_conflict() {
        let result = crate::output::resolve_mode(true, true, true);
        assert!(result.is_err());
    }

    // ---------------------------------------------------------------
    // REQ-OUTPUT-002 (Must): Meta keys list completeness
    // ---------------------------------------------------------------

    // Requirement: REQ-OUTPUT-002 (Must)
    // Acceptance: Meta keys match gogcli behavior
    #[test]
    fn req_output_002_meta_keys_complete() {
        assert!(META_KEYS.contains(&"nextPageToken"));
        assert!(META_KEYS.contains(&"next_cursor"));
        assert!(META_KEYS.contains(&"has_more"));
        assert!(META_KEYS.contains(&"count"));
        assert!(META_KEYS.contains(&"query"));
        assert!(META_KEYS.contains(&"dry_run"));
        assert!(META_KEYS.contains(&"dryRun"));
    }

    // Requirement: REQ-OUTPUT-002 (Must)
    // Acceptance: Known result keys include all Google API patterns
    #[test]
    fn req_output_002_known_result_keys_complete() {
        assert!(KNOWN_RESULT_KEYS.contains(&"files"));
        assert!(KNOWN_RESULT_KEYS.contains(&"threads"));
        assert!(KNOWN_RESULT_KEYS.contains(&"messages"));
        assert!(KNOWN_RESULT_KEYS.contains(&"events"));
        assert!(KNOWN_RESULT_KEYS.contains(&"calendars"));
        assert!(KNOWN_RESULT_KEYS.contains(&"labels"));
        assert!(KNOWN_RESULT_KEYS.contains(&"spaces"));
    }
}
