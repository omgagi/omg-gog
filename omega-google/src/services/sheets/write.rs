//! Sheets write operations: update, append, clear, create, copy.

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use super::SHEETS_BASE_URL;

/// Build URL for values.update (write cell values).
///
/// PUT /v4/spreadsheets/{spreadsheetId}/values/{range}?valueInputOption={inputOption}
pub fn build_values_update_url(spreadsheet_id: &str, range: &str, input_option: &str) -> String {
    let encoded_range = utf8_percent_encode(range, NON_ALPHANUMERIC).to_string();
    format!(
        "{}/spreadsheets/{}/values/{}?valueInputOption={}",
        SHEETS_BASE_URL, spreadsheet_id, encoded_range, input_option
    )
}

/// Build URL for values.append (append rows).
///
/// POST /v4/spreadsheets/{spreadsheetId}/values/{range}:append?valueInputOption={inputOption}
pub fn build_values_append_url(
    spreadsheet_id: &str,
    range: &str,
    input_option: &str,
    insert_data_option: Option<&str>,
) -> String {
    let encoded_range = utf8_percent_encode(range, NON_ALPHANUMERIC).to_string();
    let mut url = format!(
        "{}/spreadsheets/{}/values/{}:append?valueInputOption={}",
        SHEETS_BASE_URL, spreadsheet_id, encoded_range, input_option
    );
    if let Some(ido) = insert_data_option {
        url.push_str(&format!("&insertDataOption={}", ido));
    }
    url
}

/// Build URL for values.clear (clear cell values).
///
/// POST /v4/spreadsheets/{spreadsheetId}/values/{range}:clear
pub fn build_values_clear_url(spreadsheet_id: &str, range: &str) -> String {
    let encoded_range = utf8_percent_encode(range, NON_ALPHANUMERIC).to_string();
    format!(
        "{}/spreadsheets/{}/values/{}:clear",
        SHEETS_BASE_URL, spreadsheet_id, encoded_range
    )
}

/// Build the request body for values.update or values.append.
///
/// Wraps the 2D values array in a `{"values": [...]}` JSON object.
pub fn build_values_body(values: Vec<Vec<serde_json::Value>>) -> serde_json::Value {
    serde_json::json!({
        "values": values
    })
}

/// Parse pipe-separated cells and comma-separated rows into a 2D values array.
///
/// Format: `a|b|c,d|e|f` produces `[["a","b","c"],["d","e","f"]]`.
/// Each cell value is stored as a JSON string.
pub fn parse_cell_values(input: &str) -> Vec<Vec<serde_json::Value>> {
    if input.is_empty() {
        return vec![];
    }
    input
        .split(',')
        .map(|row| {
            row.split('|')
                .map(|cell| serde_json::Value::String(cell.to_string()))
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::structure::{build_create_spreadsheet_body, build_copy_body};
    use serde_json::json;

    // ---------------------------------------------------------------
    // REQ-SHEETS-002 (Must): Update URL and body construction
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-002 (Must)
    // Acceptance: Update URL with RAW input
    #[test]
    fn req_sheets_002_update_url_raw() {
        let url = build_values_update_url("abc123", "Sheet1!A1:B3", "RAW");
        assert!(url.contains("spreadsheets/abc123/values/"));
        assert!(url.contains("valueInputOption=RAW"));
    }

    // Requirement: REQ-SHEETS-002 (Must)
    // Acceptance: Update URL with USER_ENTERED input
    #[test]
    fn req_sheets_002_update_url_user_entered() {
        let url = build_values_update_url("abc123", "A1:B3", "USER_ENTERED");
        assert!(url.contains("valueInputOption=USER_ENTERED"));
    }

    // Requirement: REQ-SHEETS-002 (Must)
    // Acceptance: Values body construction
    #[test]
    fn req_sheets_002_values_body() {
        let body = build_values_body(vec![
            vec![json!("a"), json!("b")],
            vec![json!("c"), json!("d")],
        ]);
        assert_eq!(body["values"][0][0], json!("a"));
        assert_eq!(body["values"][1][1], json!("d"));
    }

    // Requirement: REQ-SHEETS-002 (Must)
    // Acceptance: Empty values body
    #[test]
    fn req_sheets_002_values_body_empty() {
        let body = build_values_body(vec![]);
        assert_eq!(body["values"], json!([]));
    }

    // ---------------------------------------------------------------
    // REQ-SHEETS-003 (Must): Append URL construction
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-003 (Must)
    // Acceptance: Append URL basic
    #[test]
    fn req_sheets_003_append_url_basic() {
        let url = build_values_append_url("abc123", "Sheet1!A1", "USER_ENTERED", None);
        assert!(url.contains(":append"));
        assert!(url.contains("valueInputOption=USER_ENTERED"));
    }

    // Requirement: REQ-SHEETS-003 (Must)
    // Acceptance: Append URL with insert data option
    #[test]
    fn req_sheets_003_append_url_with_insert_option() {
        let url = build_values_append_url(
            "abc123",
            "Sheet1!A1",
            "RAW",
            Some("INSERT_ROWS"),
        );
        assert!(url.contains("insertDataOption=INSERT_ROWS"));
    }

    // Requirement: REQ-SHEETS-003 (Must)
    // Acceptance: Append URL with OVERWRITE
    #[test]
    fn req_sheets_003_append_url_overwrite() {
        let url = build_values_append_url(
            "abc123",
            "A1",
            "USER_ENTERED",
            Some("OVERWRITE"),
        );
        assert!(url.contains("insertDataOption=OVERWRITE"));
    }

    // ---------------------------------------------------------------
    // REQ-SHEETS-005 (Must): Clear URL construction
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-005 (Must)
    // Acceptance: Clear URL
    #[test]
    fn req_sheets_005_clear_url() {
        let url = build_values_clear_url("abc123", "Sheet1!A1:B10");
        assert!(url.contains("spreadsheets/abc123/values/"));
        assert!(url.contains(":clear"));
    }

    // Requirement: REQ-SHEETS-005 (Must)
    // Acceptance: Clear URL without sheet name
    #[test]
    fn req_sheets_005_clear_url_no_sheet() {
        let url = build_values_clear_url("abc123", "A1:Z100");
        assert!(url.contains(":clear"));
    }

    // ---------------------------------------------------------------
    // REQ-SHEETS-009 (Must): Create spreadsheet body
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-009 (Must)
    // Acceptance: Create body with title only
    #[test]
    fn req_sheets_009_create_body_title_only() {
        let body = build_create_spreadsheet_body("My Sheet", &[]);
        assert_eq!(body["properties"]["title"], json!("My Sheet"));
        assert!(body.get("sheets").is_none());
    }

    // Requirement: REQ-SHEETS-009 (Must)
    // Acceptance: Create body with sheet names
    #[test]
    fn req_sheets_009_create_body_with_sheets() {
        let body = build_create_spreadsheet_body(
            "My Sheet",
            &["Data".to_string(), "Summary".to_string()],
        );
        assert_eq!(body["properties"]["title"], json!("My Sheet"));
        let sheets = body["sheets"].as_array().unwrap();
        assert_eq!(sheets.len(), 2);
        assert_eq!(sheets[0]["properties"]["title"], json!("Data"));
        assert_eq!(sheets[0]["properties"]["index"], json!(0));
        assert_eq!(sheets[1]["properties"]["title"], json!("Summary"));
        assert_eq!(sheets[1]["properties"]["index"], json!(1));
    }

    // Requirement: REQ-SHEETS-009 (Must)
    // Acceptance: Create body with single sheet
    #[test]
    fn req_sheets_009_create_body_single_sheet() {
        let body = build_create_spreadsheet_body("Test", &["Sheet1".to_string()]);
        let sheets = body["sheets"].as_array().unwrap();
        assert_eq!(sheets.len(), 1);
        assert_eq!(sheets[0]["properties"]["title"], json!("Sheet1"));
    }

    // ---------------------------------------------------------------
    // REQ-SHEETS-010 (Must): Copy body
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-010 (Must)
    // Acceptance: Copy body without parent
    #[test]
    fn req_sheets_010_copy_body_no_parent() {
        let body = build_copy_body("Copy of Sheet", None);
        assert_eq!(body["name"], json!("Copy of Sheet"));
        assert!(body.get("parents").is_none());
    }

    // Requirement: REQ-SHEETS-010 (Must)
    // Acceptance: Copy body with parent
    #[test]
    fn req_sheets_010_copy_body_with_parent() {
        let body = build_copy_body("Copy of Sheet", Some("folder_abc"));
        assert_eq!(body["name"], json!("Copy of Sheet"));
        assert_eq!(body["parents"], json!(["folder_abc"]));
    }

    // ---------------------------------------------------------------
    // Value parsing tests
    // ---------------------------------------------------------------

    // REQ-SHEETS-002
    // Acceptance: Parse pipe-separated cells
    #[test]
    fn req_sheets_002_parse_cell_values_basic() {
        let values = parse_cell_values("a|b|c,d|e|f");
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], vec![json!("a"), json!("b"), json!("c")]);
        assert_eq!(values[1], vec![json!("d"), json!("e"), json!("f")]);
    }

    // REQ-SHEETS-002
    // Acceptance: Parse single row
    #[test]
    fn req_sheets_002_parse_cell_values_single_row() {
        let values = parse_cell_values("x|y|z");
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], vec![json!("x"), json!("y"), json!("z")]);
    }

    // REQ-SHEETS-002
    // Acceptance: Parse single cell
    #[test]
    fn req_sheets_002_parse_cell_values_single_cell() {
        let values = parse_cell_values("hello");
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], vec![json!("hello")]);
    }

    // REQ-SHEETS-002
    // Edge case: Empty input
    #[test]
    fn req_sheets_002_parse_cell_values_empty() {
        let values = parse_cell_values("");
        assert!(values.is_empty());
    }

    // REQ-SHEETS-002
    // Edge case: Multiple rows with single cells
    #[test]
    fn req_sheets_002_parse_cell_values_single_cells_per_row() {
        let values = parse_cell_values("a,b,c");
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], vec![json!("a")]);
        assert_eq!(values[1], vec![json!("b")]);
        assert_eq!(values[2], vec![json!("c")]);
    }
}
