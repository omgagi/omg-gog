//! Sheets formatting operations: batch update, repeat cell, format requests.

use super::SHEETS_BASE_URL;

/// Build URL for spreadsheets.batchUpdate.
///
/// POST /v4/spreadsheets/{spreadsheetId}:batchUpdate
pub fn build_batch_update_url(spreadsheet_id: &str) -> String {
    format!(
        "{}/spreadsheets/{}:batchUpdate",
        SHEETS_BASE_URL, spreadsheet_id
    )
}

/// Build a repeatCell request for applying formatting to a range.
///
/// The `format` parameter should be a CellFormat JSON value.
/// The `fields` parameter specifies which fields to update (e.g.,
/// "userEnteredFormat.textFormat.bold").
pub fn build_repeat_cell_request(
    sheet_id: i64,
    start_row: i64,
    end_row: i64,
    start_col: i64,
    end_col: i64,
    format: serde_json::Value,
    fields: &str,
) -> serde_json::Value {
    serde_json::json!({
        "repeatCell": {
            "range": {
                "sheetId": sheet_id,
                "startRowIndex": start_row,
                "endRowIndex": end_row,
                "startColumnIndex": start_col,
                "endColumnIndex": end_col
            },
            "cell": {
                "userEnteredFormat": format
            },
            "fields": fields
        }
    })
}

/// Build a format request from an A1Range and format JSON string.
///
/// Converts the A1Range start/end into grid coordinates and constructs
/// a repeatCell request. Returns an error if the format JSON is invalid.
pub fn build_format_request(
    sheet_id: i64,
    range: &super::a1::A1Range,
    format_json: &str,
    format_fields: &str,
) -> Result<serde_json::Value, String> {
    let format: serde_json::Value =
        serde_json::from_str(format_json).map_err(|e| format!("invalid format JSON: {}", e))?;

    // Convert A1 range to grid coordinates (0-based indices)
    let start_row = range
        .start_row
        .map(|r| (r as i64) - 1)
        .unwrap_or(0);
    let end_row = range
        .end_row
        .map(|r| r as i64)
        .unwrap_or(start_row + 1);
    let start_col = range
        .start_col
        .as_ref()
        .map(|c| (super::a1::column_to_index(c) as i64) - 1)
        .unwrap_or(0);
    let end_col = range
        .end_col
        .as_ref()
        .map(|c| super::a1::column_to_index(c) as i64)
        .unwrap_or(start_col + 1);

    Ok(build_repeat_cell_request(
        sheet_id,
        start_row,
        end_row,
        start_col,
        end_col,
        format,
        format_fields,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---------------------------------------------------------------
    // REQ-SHEETS-006 (Must): Format request body construction
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-006 (Must)
    // Acceptance: Batch update URL
    #[test]
    fn req_sheets_006_batch_update_url() {
        let url = build_batch_update_url("abc123");
        assert_eq!(
            url,
            "https://sheets.googleapis.com/v4/spreadsheets/abc123:batchUpdate"
        );
    }

    // Requirement: REQ-SHEETS-006 (Must)
    // Acceptance: repeatCell request structure
    #[test]
    fn req_sheets_006_repeat_cell_request() {
        let format = json!({"bold": true});
        let req = build_repeat_cell_request(0, 0, 5, 0, 3, format, "userEnteredFormat.textFormat.bold");

        assert!(req.get("repeatCell").is_some());
        let repeat = &req["repeatCell"];
        assert_eq!(repeat["range"]["sheetId"], json!(0));
        assert_eq!(repeat["range"]["startRowIndex"], json!(0));
        assert_eq!(repeat["range"]["endRowIndex"], json!(5));
        assert_eq!(repeat["range"]["startColumnIndex"], json!(0));
        assert_eq!(repeat["range"]["endColumnIndex"], json!(3));
        assert_eq!(repeat["cell"]["userEnteredFormat"]["bold"], json!(true));
        assert_eq!(repeat["fields"], json!("userEnteredFormat.textFormat.bold"));
    }

    // Requirement: REQ-SHEETS-006 (Must)
    // Acceptance: repeatCell request with complex format
    #[test]
    fn req_sheets_006_repeat_cell_complex_format() {
        let format = json!({
            "textFormat": {
                "bold": true,
                "fontSize": 14
            },
            "backgroundColor": {
                "red": 1.0,
                "green": 0.9,
                "blue": 0.8
            }
        });
        let req = build_repeat_cell_request(1, 0, 1, 0, 5, format.clone(), "userEnteredFormat");
        let cell_format = &req["repeatCell"]["cell"]["userEnteredFormat"];
        assert_eq!(cell_format["textFormat"]["bold"], json!(true));
        assert_eq!(cell_format["textFormat"]["fontSize"], json!(14));
    }

    // Requirement: REQ-SHEETS-006 (Must)
    // Acceptance: build_format_request from A1Range
    #[test]
    fn req_sheets_006_build_format_request_from_a1() {
        let range = super::super::a1::A1Range {
            sheet: Some("Sheet1".to_string()),
            start_col: Some("A".to_string()),
            start_row: Some(1),
            end_col: Some("C".to_string()),
            end_row: Some(5),
        };
        let result = build_format_request(
            0,
            &range,
            r#"{"bold": true}"#,
            "userEnteredFormat.textFormat.bold",
        );
        assert!(result.is_ok());
        let req = result.unwrap();
        let repeat = &req["repeatCell"];
        // A1 (row 1) -> startRowIndex 0
        assert_eq!(repeat["range"]["startRowIndex"], json!(0));
        // row 5 -> endRowIndex 5
        assert_eq!(repeat["range"]["endRowIndex"], json!(5));
        // A -> startColumnIndex 0
        assert_eq!(repeat["range"]["startColumnIndex"], json!(0));
        // C -> endColumnIndex 3
        assert_eq!(repeat["range"]["endColumnIndex"], json!(3));
    }

    // Requirement: REQ-SHEETS-006 (Must)
    // Acceptance: build_format_request returns error for invalid JSON
    #[test]
    fn req_sheets_006_build_format_request_invalid_json() {
        let range = super::super::a1::A1Range {
            sheet: None,
            start_col: Some("A".to_string()),
            start_row: Some(1),
            end_col: Some("B".to_string()),
            end_row: Some(2),
        };
        let result = build_format_request(0, &range, "not valid json", "fields");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid format JSON"));
    }

    // Requirement: REQ-SHEETS-006 (Must)
    // Acceptance: build_format_request with single cell range
    #[test]
    fn req_sheets_006_build_format_request_single_cell() {
        let range = super::super::a1::A1Range {
            sheet: None,
            start_col: Some("B".to_string()),
            start_row: Some(3),
            end_col: None,
            end_row: None,
        };
        let result = build_format_request(
            0,
            &range,
            r#"{"italic": true}"#,
            "userEnteredFormat.textFormat.italic",
        );
        assert!(result.is_ok());
        let req = result.unwrap();
        let repeat = &req["repeatCell"];
        // B3 -> startRow=2, endRow=3, startCol=1, endCol=2
        assert_eq!(repeat["range"]["startRowIndex"], json!(2));
        assert_eq!(repeat["range"]["endRowIndex"], json!(3));
        assert_eq!(repeat["range"]["startColumnIndex"], json!(1));
        assert_eq!(repeat["range"]["endColumnIndex"], json!(2));
    }

    // Requirement: REQ-SHEETS-006 (Must)
    // Acceptance: Batch update URL with different spreadsheet ID
    #[test]
    fn req_sheets_006_batch_update_url_different_id() {
        let url = build_batch_update_url("xyz789");
        assert!(url.contains("xyz789"));
        assert!(url.ends_with(":batchUpdate"));
    }
}
