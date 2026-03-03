//! Integration tests for the output module.
//!
//! Tests cover REQ-OUTPUT-001 through REQ-OUTPUT-005 (Must priority).
//! Validates JSON transforms, field selection, output mode resolution, and plain formatting.

use omega_google::output::transform;
use omega_google::output::*;
use serde_json::json;

// ---------------------------------------------------------------
// REQ-OUTPUT-001 (Must): Output mode resolution
// ---------------------------------------------------------------

// Requirement: REQ-OUTPUT-001 (Must)
// Acceptance: JSON mode from --json flag
#[test]
fn req_output_001_json_mode() {
    let mode = resolve_mode(true, false, true).unwrap();
    assert_eq!(mode, OutputMode::Json);
}

// Requirement: REQ-OUTPUT-001 (Must)
// Acceptance: Plain mode from --plain flag
#[test]
fn req_output_001_plain_mode() {
    let mode = resolve_mode(false, true, true).unwrap();
    assert_eq!(mode, OutputMode::Plain);
}

// Requirement: REQ-OUTPUT-001 (Must)
// Acceptance: Text mode default on TTY
#[test]
fn req_output_001_text_default() {
    let mode = resolve_mode(false, false, true).unwrap();
    assert_eq!(mode, OutputMode::Text);
}

// Requirement: REQ-OUTPUT-001 (Must)
// Acceptance: Text mode even when not TTY (without GOG_AUTO_JSON)
#[test]
fn req_output_001_text_when_piped() {
    let mode = resolve_mode(false, false, false).unwrap();
    // Without GOG_AUTO_JSON, piping does not change default
    assert_eq!(mode, OutputMode::Text);
}

// Requirement: REQ-OUTPUT-001 (Must)
// Acceptance: --json and --plain conflict
#[test]
fn req_output_001_json_plain_conflict() {
    let result = resolve_mode(true, true, true);
    assert!(
        result.is_err(),
        "should error when both --json and --plain are set"
    );
}

// ---------------------------------------------------------------
// REQ-OUTPUT-002 (Must): --results-only transform
// ---------------------------------------------------------------

// Requirement: REQ-OUTPUT-002 (Must)
// Acceptance: Strips nextPageToken from thread list
#[test]
fn req_output_002_results_only_thread_list() {
    let input = json!({
        "threads": [
            {"id": "t1", "snippet": "Hello"},
            {"id": "t2", "snippet": "World"}
        ],
        "nextPageToken": "page2",
        "resultSizeEstimate": 100
    });
    let result = transform::unwrap_primary(input);
    let arr = result.as_array().expect("should unwrap to array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["id"], "t1");
}

// Requirement: REQ-OUTPUT-002 (Must)
// Acceptance: Strips multiple meta keys
#[test]
fn req_output_002_results_only_strips_all_meta() {
    let input = json!({
        "files": [{"id": "f1"}],
        "nextPageToken": "abc",
        "count": 1,
        "query": "test"
    });
    let result = transform::unwrap_primary(input);
    assert_eq!(result, json!([{"id": "f1"}]));
}

// Requirement: REQ-OUTPUT-002 (Must)
// Acceptance: Explicit "results" key is preferred
#[test]
fn req_output_002_results_key_preferred() {
    let input = json!({
        "results": [{"id": "r1"}, {"id": "r2"}],
        "files": [{"id": "f1"}],
        "count": 2
    });
    let result = transform::unwrap_primary(input);
    assert_eq!(result, json!([{"id": "r1"}, {"id": "r2"}]));
}

// Requirement: REQ-OUTPUT-002 (Must)
// Acceptance: Single non-meta key unwrapped
#[test]
fn req_output_002_single_non_meta_unwrapped() {
    let input = json!({
        "labels": [{"id": "L1", "name": "INBOX"}],
        "nextPageToken": null
    });
    let result = transform::unwrap_primary(input);
    assert_eq!(result, json!([{"id": "L1", "name": "INBOX"}]));
}

// Requirement: REQ-OUTPUT-002 (Must)
// Acceptance: Array candidate preferred when multiple non-meta keys
#[test]
fn req_output_002_array_preferred() {
    let input = json!({
        "events": [{"id": "e1"}],
        "summary": "My Calendar",
        "updated": "2024-01-01"
    });
    let result = transform::unwrap_primary(input);
    assert_eq!(result, json!([{"id": "e1"}]));
}

// Requirement: REQ-OUTPUT-002 (Must)
// Acceptance: Non-object values pass through
#[test]
fn req_output_002_passthrough_non_object() {
    assert_eq!(transform::unwrap_primary(json!("hello")), json!("hello"));
    assert_eq!(transform::unwrap_primary(json!(42)), json!(42));
    assert_eq!(transform::unwrap_primary(json!([1, 2])), json!([1, 2]));
    assert_eq!(transform::unwrap_primary(json!(null)), json!(null));
}

// Requirement: REQ-OUTPUT-002 (Must)
// Edge case: All meta keys means no unwrapping
#[test]
fn req_output_002_all_meta_no_change() {
    let input = json!({
        "nextPageToken": "abc",
        "count": 0,
        "query": "test"
    });
    let result = transform::unwrap_primary(input.clone());
    assert_eq!(result, input);
}

// Requirement: REQ-OUTPUT-002 (Must)
// Edge case: Empty object
#[test]
fn req_output_002_empty_object() {
    let input = json!({});
    let result = transform::unwrap_primary(input.clone());
    assert_eq!(result, input);
}

// ---------------------------------------------------------------
// REQ-OUTPUT-003 (Must): --select field projection
// ---------------------------------------------------------------

// Requirement: REQ-OUTPUT-003 (Must)
// Acceptance: Select from flat object
#[test]
fn req_output_003_select_flat_object() {
    let input = json!({"id": "1", "name": "doc", "mimeType": "text/plain", "size": 1024});
    let result = transform::select_fields(input, &["id".to_string(), "name".to_string()]);
    let obj = result.as_object().unwrap();
    assert_eq!(obj.len(), 2);
    assert_eq!(obj["id"], "1");
    assert_eq!(obj["name"], "doc");
}

// Requirement: REQ-OUTPUT-003 (Must)
// Acceptance: Select from array of objects
#[test]
fn req_output_003_select_array() {
    let input = json!([
        {"id": "1", "name": "a", "type": "file"},
        {"id": "2", "name": "b", "type": "folder"},
        {"id": "3", "name": "c", "type": "file"}
    ]);
    let result = transform::select_fields(input, &["id".to_string(), "name".to_string()]);
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    for item in arr {
        let obj = item.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("name"));
        assert!(!obj.contains_key("type"));
    }
}

// Requirement: REQ-OUTPUT-003 (Must)
// Acceptance: Dot-path notation for nested fields
#[test]
fn req_output_003_select_dot_path() {
    let input = json!({
        "file": {"id": "f1", "metadata": {"name": "doc.txt"}},
        "owner": "me"
    });
    let result = transform::select_fields(
        input,
        &["file.id".to_string(), "file.metadata.name".to_string()],
    );
    let obj = result.as_object().unwrap();
    assert_eq!(obj["file.id"], "f1");
    assert_eq!(obj["file.metadata.name"], "doc.txt");
}

// Requirement: REQ-OUTPUT-003 (Must)
// Acceptance: Missing fields silently omitted
#[test]
fn req_output_003_select_missing_field() {
    let input = json!({"id": "1", "name": "test"});
    let result = transform::select_fields(input, &["id".to_string(), "nonexistent".to_string()]);
    let obj = result.as_object().unwrap();
    assert_eq!(obj.len(), 1);
    assert_eq!(obj["id"], "1");
}

// Requirement: REQ-OUTPUT-003 (Must)
// Acceptance: Empty select returns empty object
#[test]
fn req_output_003_select_empty_list() {
    let input = json!({"id": "1", "name": "test"});
    let result = transform::select_fields(input, &[]);
    let obj = result.as_object().unwrap();
    assert_eq!(obj.len(), 0);
}

// Requirement: REQ-OUTPUT-003 (Must)
// Acceptance: Non-object passes through
#[test]
fn req_output_003_select_non_object() {
    let input = json!("just a string");
    let result = transform::select_fields(input.clone(), &["id".to_string()]);
    assert_eq!(result, input);
}

// Requirement: REQ-OUTPUT-003 (Must)
// Acceptance: Array index in dot path
#[test]
fn req_output_003_select_array_index_path() {
    let input = json!({"items": ["a", "b", "c"]});
    let result = transform::get_at_path(&input, "items.1");
    assert_eq!(result, Some(json!("b")));
}

// ---------------------------------------------------------------
// REQ-OUTPUT-003 (Must): get_at_path
// ---------------------------------------------------------------

// Requirement: REQ-OUTPUT-003 (Must)
// Acceptance: Simple path
#[test]
fn req_output_003_path_simple() {
    assert_eq!(
        transform::get_at_path(&json!({"name": "test"}), "name"),
        Some(json!("test"))
    );
}

// Requirement: REQ-OUTPUT-003 (Must)
// Acceptance: Deeply nested path
#[test]
fn req_output_003_path_deep_nested() {
    let input = json!({"a": {"b": {"c": {"d": 42}}}});
    assert_eq!(transform::get_at_path(&input, "a.b.c.d"), Some(json!(42)));
}

// Requirement: REQ-OUTPUT-003 (Must)
// Edge case: Empty path returns None
#[test]
fn req_output_003_path_empty() {
    assert_eq!(transform::get_at_path(&json!({"id": 1}), ""), None);
}

// Requirement: REQ-OUTPUT-003 (Must)
// Edge case: Path with whitespace
#[test]
fn req_output_003_path_whitespace() {
    // Should trim whitespace from path segments
    assert_eq!(
        transform::get_at_path(&json!({"id": 1}), " id "),
        Some(json!(1))
    );
}

// Requirement: REQ-OUTPUT-003 (Must)
// Edge case: Missing key returns None
#[test]
fn req_output_003_path_missing_key() {
    assert_eq!(transform::get_at_path(&json!({"id": 1}), "name"), None);
}

// Requirement: REQ-OUTPUT-003 (Must)
// Edge case: Out-of-bounds array index
#[test]
fn req_output_003_path_oob_array() {
    let input = json!({"items": ["a"]});
    assert_eq!(transform::get_at_path(&input, "items.5"), None);
}

// Requirement: REQ-OUTPUT-003 (Must)
// Edge case: Negative array index
#[test]
fn req_output_003_path_negative_index() {
    let input = json!({"items": ["a", "b"]});
    assert_eq!(transform::get_at_path(&input, "items.-1"), None);
}

// ---------------------------------------------------------------
// REQ-OUTPUT-001 (Must): JSON output formatting
// ---------------------------------------------------------------

// Requirement: REQ-OUTPUT-001 (Must)
// Acceptance: JSON output is pretty-printed with 2-space indent
#[test]
fn req_output_001_json_pretty_printed() {
    let value = json!({"id": "1", "name": "test"});
    let mut buf = Vec::new();
    let _ = write_json(&mut buf, &value, &JsonTransform::default());
    let output = String::from_utf8(buf).unwrap();
    assert!(
        output.contains("  "),
        "JSON should be pretty-printed with indentation"
    );
}

// Requirement: REQ-OUTPUT-001 (Must)
// Acceptance: JSON transform applied when results_only is set
#[test]
fn req_output_001_json_with_results_only() {
    let value = json!({
        "threads": [{"id": "t1"}],
        "nextPageToken": "abc"
    });
    let transform = JsonTransform {
        results_only: true,
        select: vec![],
    };
    let mut buf = Vec::new();
    let _ = write_json(&mut buf, &value, &transform);
    let output = String::from_utf8(buf).unwrap();
    // Should contain the threads array, not the envelope
    assert!(!output.contains("nextPageToken"));
}

// Requirement: REQ-OUTPUT-001 (Must)
// Acceptance: JSON transform applied with select fields
#[test]
fn req_output_001_json_with_select() {
    let value = json!({"id": "1", "name": "test", "extra": "data"});
    let transform = JsonTransform {
        results_only: false,
        select: vec!["id".to_string(), "name".to_string()],
    };
    let mut buf = Vec::new();
    let _ = write_json(&mut buf, &value, &transform);
    let output = String::from_utf8(buf).unwrap();
    assert!(!output.contains("extra"));
}

// ---------------------------------------------------------------
// REQ-OUTPUT-001 (Must): Plain/TSV output
// ---------------------------------------------------------------

// Requirement: REQ-OUTPUT-001 (Must)
// Acceptance: Plain output is tab-separated
#[test]
fn req_output_001_plain_tab_separated() {
    let rows = vec![
        vec!["id".to_string(), "name".to_string()],
        vec!["1".to_string(), "file.txt".to_string()],
        vec!["2".to_string(), "doc.pdf".to_string()],
    ];
    let mut buf = Vec::new();
    let _ = write_plain(&mut buf, &rows);
    let output = String::from_utf8(buf).unwrap();
    // Each row should be tab-separated
    for line in output.lines() {
        assert!(line.contains('\t'), "plain output should be tab-separated");
    }
}

// Requirement: REQ-OUTPUT-001 (Must)
// Acceptance: Plain output has no colors
#[test]
fn req_output_001_plain_no_ansi() {
    let rows = vec![vec!["test".to_string(), "value".to_string()]];
    let mut buf = Vec::new();
    let _ = write_plain(&mut buf, &rows);
    let output = String::from_utf8(buf).unwrap();
    // Should not contain ANSI escape codes
    assert!(
        !output.contains("\x1b["),
        "plain output should not contain ANSI codes"
    );
}

// ---------------------------------------------------------------
// REQ-OUTPUT-002 (Must): Meta keys and result keys completeness
// ---------------------------------------------------------------

// Requirement: REQ-OUTPUT-002 (Must)
// Acceptance: All known meta keys are defined
#[test]
fn req_output_002_meta_keys_defined() {
    let meta = transform::META_KEYS;
    let expected = vec![
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
    for key in &expected {
        assert!(meta.contains(key), "META_KEYS should contain '{}'", key);
    }
}

// Requirement: REQ-OUTPUT-002 (Must)
// Acceptance: All known result keys are defined
#[test]
fn req_output_002_result_keys_defined() {
    let keys = transform::KNOWN_RESULT_KEYS;
    let expected = vec![
        "files",
        "threads",
        "messages",
        "labels",
        "events",
        "calendars",
        "courses",
        "topics",
        "announcements",
        "contacts",
        "people",
        "tasks",
        "groups",
        "spaces",
    ];
    for key in &expected {
        assert!(
            keys.contains(key),
            "KNOWN_RESULT_KEYS should contain '{}'",
            key
        );
    }
}

// ---------------------------------------------------------------
// REQ-OUTPUT-005 (Must): Colors disabled for JSON/plain
// ---------------------------------------------------------------

// Requirement: REQ-OUTPUT-005 (Must)
// Acceptance: JSON output has no ANSI
#[test]
fn req_output_005_json_no_ansi() {
    let value = json!({"status": "error", "message": "something failed"});
    let mut buf = Vec::new();
    let _ = write_json(&mut buf, &value, &JsonTransform::default());
    let output = String::from_utf8(buf).unwrap();
    assert!(
        !output.contains("\x1b["),
        "JSON output must not contain ANSI escape codes"
    );
}

// ---------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------

// Requirement: REQ-OUTPUT-002 (Must)
// Edge case: results_only + select combined
#[test]
fn req_output_002_combined_results_only_and_select() {
    let value = json!({
        "files": [
            {"id": "f1", "name": "a", "size": 100},
            {"id": "f2", "name": "b", "size": 200}
        ],
        "nextPageToken": "abc"
    });
    let transform = JsonTransform {
        results_only: true,
        select: vec!["id".to_string(), "name".to_string()],
    };
    let mut buf = Vec::new();
    let _ = write_json(&mut buf, &value, &transform);
    let output = String::from_utf8(buf).unwrap();
    // Should first unwrap to files array, then project to id+name
    assert!(!output.contains("nextPageToken"));
    assert!(!output.contains("size"));
}

// Requirement: REQ-OUTPUT-001 (Must)
// Edge case: Empty rows in plain output
#[test]
fn req_output_001_plain_empty_rows() {
    let rows: Vec<Vec<String>> = vec![];
    let mut buf = Vec::new();
    let _ = write_plain(&mut buf, &rows);
    let output = String::from_utf8(buf).unwrap();
    assert!(output.is_empty() || output.trim().is_empty());
}

// Requirement: REQ-OUTPUT-003 (Must)
// Edge case: Select with only missing fields
#[test]
fn req_output_003_select_all_missing() {
    let input = json!({"id": "1"});
    let result = transform::select_fields(
        input,
        &["nonexistent1".to_string(), "nonexistent2".to_string()],
    );
    let obj = result.as_object().unwrap();
    assert_eq!(
        obj.len(),
        0,
        "all missing fields should result in empty object"
    );
}
