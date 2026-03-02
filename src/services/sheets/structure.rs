//! Sheets structural operations: insert rows/columns, create, copy, export.

use super::SHEETS_BASE_URL;
use crate::services::drive::types::DRIVE_BASE_URL;

/// Build an insertDimension request for inserting rows or columns.
///
/// `dimension` should be `"ROWS"` or `"COLUMNS"`.
/// `start_index` and `end_index` are 0-based. To insert 1 row at position 5,
/// use `start_index=5, end_index=6`.
/// `inherit_before` controls whether the inserted range inherits properties
/// from the range before it (true) or after it (false).
pub fn build_insert_dimension_request(
    sheet_id: i64,
    dimension: &str,
    start_index: i64,
    end_index: i64,
    inherit_before: bool,
) -> serde_json::Value {
    serde_json::json!({
        "insertDimension": {
            "range": {
                "sheetId": sheet_id,
                "dimension": dimension,
                "startIndex": start_index,
                "endIndex": end_index
            },
            "inheritFromBefore": inherit_before
        }
    })
}

/// Build the request body for creating a new spreadsheet.
pub fn build_create_spreadsheet_body(title: &str, sheet_names: &[String]) -> serde_json::Value {
    let sheets: Vec<serde_json::Value> = if sheet_names.is_empty() {
        vec![]
    } else {
        sheet_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                serde_json::json!({
                    "properties": {
                        "title": name,
                        "index": i
                    }
                })
            })
            .collect()
    };

    let mut body = serde_json::json!({
        "properties": {
            "title": title
        }
    });
    if !sheets.is_empty() {
        body["sheets"] = serde_json::json!(sheets);
    }
    body
}

/// Build URL for creating a new spreadsheet.
pub fn build_create_spreadsheet_url() -> String {
    format!("{}/spreadsheets", SHEETS_BASE_URL)
}

/// Build URL for copying a spreadsheet via Drive API.
pub fn build_copy_spreadsheet_url(spreadsheet_id: &str) -> String {
    format!(
        "{}/files/{}/copy",
        DRIVE_BASE_URL, spreadsheet_id
    )
}

/// Build the copy request body.
pub fn build_copy_body(title: &str, parent: Option<&str>) -> serde_json::Value {
    let mut body = serde_json::json!({"name": title});
    if let Some(p) = parent {
        body["parents"] = serde_json::json!([p]);
    }
    body
}

/// Build URL for exporting a spreadsheet via Drive API.
pub fn build_export_url(spreadsheet_id: &str, format: &str) -> String {
    let mime = resolve_export_mime(format);
    format!(
        "{}/files/{}/export?mimeType={}",
        DRIVE_BASE_URL,
        spreadsheet_id,
        url::form_urlencoded::byte_serialize(mime.as_bytes()).collect::<String>()
    )
}

/// Resolve a format name to its MIME type for Sheets export.
pub fn resolve_export_mime(format: &str) -> &'static str {
    match format.to_lowercase().as_str() {
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "csv" => "text/csv",
        "pdf" => "application/pdf",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        "tsv" => "text/tab-separated-values",
        _ => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---------------------------------------------------------------
    // REQ-SHEETS-004 (Must): Insert dimension body builder
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-004 (Must)
    // Acceptance: Insert rows request
    #[test]
    fn req_sheets_004_insert_rows() {
        let req = build_insert_dimension_request(0, "ROWS", 5, 6, false);
        assert!(req.get("insertDimension").is_some());
        let insert = &req["insertDimension"];
        assert_eq!(insert["range"]["sheetId"], json!(0));
        assert_eq!(insert["range"]["dimension"], json!("ROWS"));
        assert_eq!(insert["range"]["startIndex"], json!(5));
        assert_eq!(insert["range"]["endIndex"], json!(6));
        assert_eq!(insert["inheritFromBefore"], json!(false));
    }

    // Requirement: REQ-SHEETS-004 (Must)
    // Acceptance: Insert columns request
    #[test]
    fn req_sheets_004_insert_columns() {
        let req = build_insert_dimension_request(0, "COLUMNS", 2, 5, true);
        let insert = &req["insertDimension"];
        assert_eq!(insert["range"]["dimension"], json!("COLUMNS"));
        assert_eq!(insert["range"]["startIndex"], json!(2));
        assert_eq!(insert["range"]["endIndex"], json!(5));
        assert_eq!(insert["inheritFromBefore"], json!(true));
    }

    // Requirement: REQ-SHEETS-004 (Must)
    // Acceptance: Insert with different sheet ID
    #[test]
    fn req_sheets_004_insert_different_sheet() {
        let req = build_insert_dimension_request(42, "ROWS", 0, 3, true);
        let insert = &req["insertDimension"];
        assert_eq!(insert["range"]["sheetId"], json!(42));
        assert_eq!(insert["range"]["startIndex"], json!(0));
        assert_eq!(insert["range"]["endIndex"], json!(3));
    }

    // Requirement: REQ-SHEETS-004 (Must)
    // Edge case: Insert single row with inherit before
    #[test]
    fn req_sheets_004_insert_single_row() {
        let req = build_insert_dimension_request(0, "ROWS", 10, 11, true);
        let insert = &req["insertDimension"];
        assert_eq!(insert["range"]["startIndex"], json!(10));
        assert_eq!(insert["range"]["endIndex"], json!(11));
        assert_eq!(insert["inheritFromBefore"], json!(true));
    }

    // Requirement: REQ-SHEETS-004 (Must)
    // Edge case: Insert at position 0 (beginning)
    #[test]
    fn req_sheets_004_insert_at_beginning() {
        let req = build_insert_dimension_request(0, "COLUMNS", 0, 2, false);
        let insert = &req["insertDimension"];
        assert_eq!(insert["range"]["startIndex"], json!(0));
        assert_eq!(insert["range"]["endIndex"], json!(2));
        assert_eq!(insert["inheritFromBefore"], json!(false));
    }

    // ---------------------------------------------------------------
    // Create, copy, export tests
    // ---------------------------------------------------------------

    #[test]
    fn test_create_body_basic() {
        let body = build_create_spreadsheet_body("Test", &[]);
        assert_eq!(body["properties"]["title"], "Test");
    }

    #[test]
    fn test_create_body_with_sheets() {
        let names = vec!["Sheet1".to_string(), "Sheet2".to_string()];
        let body = build_create_spreadsheet_body("Test", &names);
        let sheets = body["sheets"].as_array().unwrap();
        assert_eq!(sheets.len(), 2);
    }

    #[test]
    fn test_resolve_export_mime() {
        assert_eq!(resolve_export_mime("xlsx"), "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
        assert_eq!(resolve_export_mime("csv"), "text/csv");
        assert_eq!(resolve_export_mime("pdf"), "application/pdf");
    }

    #[test]
    fn test_copy_body_no_parent() {
        let body = build_copy_body("Copy", None);
        assert_eq!(body["name"], json!("Copy"));
        assert!(body.get("parents").is_none());
    }

    #[test]
    fn test_copy_body_with_parent() {
        let body = build_copy_body("Copy", Some("folder123"));
        assert_eq!(body["parents"], json!(["folder123"]));
    }
}
