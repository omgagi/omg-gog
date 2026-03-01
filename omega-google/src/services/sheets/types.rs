//! Google Sheets API request/response types.
//! All types use camelCase serde rename and flatten for forward compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------
// Spreadsheet types
// ---------------------------------------------------------------

/// A Google Spreadsheet resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spreadsheet {
    pub spreadsheet_id: Option<String>,
    pub properties: Option<SpreadsheetProperties>,
    #[serde(default)]
    pub sheets: Vec<Sheet>,
    #[serde(default)]
    pub named_ranges: Vec<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Properties of a spreadsheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpreadsheetProperties {
    pub title: Option<String>,
    pub locale: Option<String>,
    pub auto_recalc: Option<String>,
    pub time_zone: Option<String>,
    pub default_format: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A single sheet within a spreadsheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sheet {
    pub properties: Option<SheetProperties>,
    #[serde(default)]
    pub data: Vec<GridData>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Properties of a sheet (tab).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetProperties {
    pub sheet_id: Option<i64>,
    pub title: Option<String>,
    pub index: Option<i64>,
    pub sheet_type: Option<String>,
    pub grid_properties: Option<GridProperties>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Grid properties (dimensions) of a sheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridProperties {
    pub row_count: Option<i64>,
    pub column_count: Option<i64>,
    pub frozen_row_count: Option<i64>,
    pub frozen_column_count: Option<i64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Grid data containing rows of cells.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridData {
    #[serde(default)]
    pub row_data: Vec<RowData>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A row of cell data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RowData {
    #[serde(default)]
    pub values: Vec<CellData>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Data for a single cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellData {
    pub formatted_value: Option<String>,
    pub user_entered_value: Option<ExtendedValue>,
    pub effective_value: Option<ExtendedValue>,
    pub effective_format: Option<CellFormat>,
    pub note: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// An extended value that can be a string, number, bool, formula, or error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtendedValue {
    pub string_value: Option<String>,
    pub number_value: Option<f64>,
    pub bool_value: Option<bool>,
    pub formula_value: Option<String>,
    pub error_value: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Cell formatting properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellFormat {
    pub background_color: Option<Color>,
    pub padding: Option<serde_json::Value>,
    pub horizontal_alignment: Option<String>,
    pub vertical_alignment: Option<String>,
    pub text_format: Option<TextFormat>,
    pub borders: Option<serde_json::Value>,
    pub number_format: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Text formatting properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextFormat {
    pub font_family: Option<String>,
    pub font_size: Option<i32>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub strikethrough: Option<bool>,
    pub underline: Option<bool>,
    pub foreground_color: Option<Color>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// An RGBA color representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Color {
    pub red: Option<f64>,
    pub green: Option<f64>,
    pub blue: Option<f64>,
    pub alpha: Option<f64>,
}

// ---------------------------------------------------------------
// Value range types
// ---------------------------------------------------------------

/// A range of values returned by values.get.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueRange {
    pub range: Option<String>,
    pub major_dimension: Option<String>,
    #[serde(default)]
    pub values: Vec<Vec<serde_json::Value>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Response types
// ---------------------------------------------------------------

/// Response from values.update.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateValuesResponse {
    pub spreadsheet_id: Option<String>,
    pub updated_range: Option<String>,
    pub updated_rows: Option<i64>,
    pub updated_columns: Option<i64>,
    pub updated_cells: Option<i64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response from values.append.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendValuesResponse {
    pub spreadsheet_id: Option<String>,
    pub updates: Option<UpdateValuesResponse>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response from spreadsheets.batchUpdate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateResponse {
    pub spreadsheet_id: Option<String>,
    #[serde(default)]
    pub replies: Vec<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response from values.clear.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearValuesResponse {
    pub spreadsheet_id: Option<String>,
    pub cleared_range: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---------------------------------------------------------------
    // REQ-SHEETS-001 (Must): ValueRange deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-001 (Must)
    // Acceptance: ValueRange deserializes from Sheets API JSON
    #[test]
    fn req_sheets_001_value_range_deserialize() {
        let json_str = r#"{
            "range": "Sheet1!A1:B3",
            "majorDimension": "ROWS",
            "values": [
                ["Name", "Age"],
                ["Alice", "30"],
                ["Bob", "25"]
            ]
        }"#;
        let vr: ValueRange = serde_json::from_str(json_str).unwrap();
        assert_eq!(vr.range, Some("Sheet1!A1:B3".to_string()));
        assert_eq!(vr.major_dimension, Some("ROWS".to_string()));
        assert_eq!(vr.values.len(), 3);
        assert_eq!(vr.values[0][0], json!("Name"));
        assert_eq!(vr.values[1][1], json!("30"));
    }

    // Requirement: REQ-SHEETS-001 (Must)
    // Acceptance: ValueRange with empty values
    #[test]
    fn req_sheets_001_value_range_empty() {
        let json_str = r#"{
            "range": "Sheet1!A1:A1",
            "majorDimension": "ROWS"
        }"#;
        let vr: ValueRange = serde_json::from_str(json_str).unwrap();
        assert!(vr.values.is_empty());
    }

    // Requirement: REQ-SHEETS-001 (Must)
    // Acceptance: ValueRange round-trip serialization
    #[test]
    fn req_sheets_001_value_range_roundtrip() {
        let vr = ValueRange {
            range: Some("Sheet1!A1:C2".to_string()),
            major_dimension: Some("ROWS".to_string()),
            values: vec![
                vec![json!("x"), json!("y"), json!("z")],
                vec![json!(1), json!(2), json!(3)],
            ],
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&vr).unwrap();
        let parsed: ValueRange = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.values.len(), 2);
        assert_eq!(parsed.values[0][0], json!("x"));
    }

    // Requirement: REQ-SHEETS-001 (Must)
    // Edge case: ValueRange with unknown fields preserved
    #[test]
    fn req_sheets_001_value_range_unknown_fields() {
        let json_str = r#"{
            "range": "A1:B2",
            "majorDimension": "ROWS",
            "values": [["a"]],
            "unknownField": "preserved"
        }"#;
        let vr: ValueRange = serde_json::from_str(json_str).unwrap();
        assert!(vr.extra.contains_key("unknownField"));
    }

    // Requirement: REQ-SHEETS-001 (Must)
    // Acceptance: ValueRange with COLUMNS dimension
    #[test]
    fn req_sheets_001_value_range_columns() {
        let json_str = r#"{
            "range": "Sheet1!A1:C1",
            "majorDimension": "COLUMNS",
            "values": [["a", "b"], ["c", "d"], ["e", "f"]]
        }"#;
        let vr: ValueRange = serde_json::from_str(json_str).unwrap();
        assert_eq!(vr.major_dimension, Some("COLUMNS".to_string()));
        assert_eq!(vr.values.len(), 3);
    }

    // ---------------------------------------------------------------
    // REQ-SHEETS-008 (Must): Spreadsheet metadata deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-008 (Must)
    // Acceptance: Spreadsheet deserializes from API JSON
    #[test]
    fn req_sheets_008_spreadsheet_deserialize() {
        let json_str = r#"{
            "spreadsheetId": "abc123",
            "properties": {
                "title": "My Spreadsheet",
                "locale": "en_US",
                "autoRecalc": "ON_CHANGE",
                "timeZone": "America/New_York"
            },
            "sheets": [
                {
                    "properties": {
                        "sheetId": 0,
                        "title": "Sheet1",
                        "index": 0,
                        "sheetType": "GRID",
                        "gridProperties": {
                            "rowCount": 1000,
                            "columnCount": 26
                        }
                    }
                }
            ]
        }"#;
        let ss: Spreadsheet = serde_json::from_str(json_str).unwrap();
        assert_eq!(ss.spreadsheet_id, Some("abc123".to_string()));
        let props = ss.properties.unwrap();
        assert_eq!(props.title, Some("My Spreadsheet".to_string()));
        assert_eq!(props.locale, Some("en_US".to_string()));
        assert_eq!(props.time_zone, Some("America/New_York".to_string()));
        assert_eq!(ss.sheets.len(), 1);
        let sheet_props = ss.sheets[0].properties.as_ref().unwrap();
        assert_eq!(sheet_props.sheet_id, Some(0));
        assert_eq!(sheet_props.title, Some("Sheet1".to_string()));
        let grid = sheet_props.grid_properties.as_ref().unwrap();
        assert_eq!(grid.row_count, Some(1000));
        assert_eq!(grid.column_count, Some(26));
    }

    // Requirement: REQ-SHEETS-008 (Must)
    // Acceptance: Spreadsheet with multiple sheets
    #[test]
    fn req_sheets_008_spreadsheet_multiple_sheets() {
        let json_str = r#"{
            "spreadsheetId": "abc123",
            "properties": {"title": "Multi"},
            "sheets": [
                {"properties": {"sheetId": 0, "title": "Sheet1", "index": 0}},
                {"properties": {"sheetId": 1, "title": "Sheet2", "index": 1}},
                {"properties": {"sheetId": 2, "title": "Sheet3", "index": 2}}
            ]
        }"#;
        let ss: Spreadsheet = serde_json::from_str(json_str).unwrap();
        assert_eq!(ss.sheets.len(), 3);
        let s2 = ss.sheets[1].properties.as_ref().unwrap();
        assert_eq!(s2.title, Some("Sheet2".to_string()));
        assert_eq!(s2.sheet_id, Some(1));
    }

    // Requirement: REQ-SHEETS-008 (Must)
    // Acceptance: Spreadsheet round-trip
    #[test]
    fn req_sheets_008_spreadsheet_roundtrip() {
        let ss = Spreadsheet {
            spreadsheet_id: Some("s1".to_string()),
            properties: Some(SpreadsheetProperties {
                title: Some("Test".to_string()),
                locale: Some("en_US".to_string()),
                auto_recalc: None,
                time_zone: None,
                default_format: None,
                extra: HashMap::new(),
            }),
            sheets: vec![],
            named_ranges: vec![],
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&ss).unwrap();
        let parsed: Spreadsheet = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.spreadsheet_id, Some("s1".to_string()));
    }

    // Requirement: REQ-SHEETS-008 (Must)
    // Edge case: Spreadsheet with named ranges
    #[test]
    fn req_sheets_008_spreadsheet_named_ranges() {
        let json_str = r#"{
            "spreadsheetId": "abc",
            "properties": {"title": "Test"},
            "sheets": [],
            "namedRanges": [
                {"namedRangeId": "nr1", "name": "MyRange"}
            ]
        }"#;
        let ss: Spreadsheet = serde_json::from_str(json_str).unwrap();
        assert_eq!(ss.named_ranges.len(), 1);
    }

    // Requirement: REQ-SHEETS-008 (Must)
    // Edge case: Unknown fields preserved
    #[test]
    fn req_sheets_008_spreadsheet_unknown_fields() {
        let json_str = r#"{
            "spreadsheetId": "abc",
            "properties": {"title": "Test"},
            "sheets": [],
            "futureField": 42
        }"#;
        let ss: Spreadsheet = serde_json::from_str(json_str).unwrap();
        assert!(ss.extra.contains_key("futureField"));
    }

    // ---------------------------------------------------------------
    // REQ-SHEETS-002 (Must): UpdateValuesResponse deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-002 (Must)
    // Acceptance: UpdateValuesResponse deserializes
    #[test]
    fn req_sheets_002_update_values_response_deserialize() {
        let json_str = r#"{
            "spreadsheetId": "abc123",
            "updatedRange": "Sheet1!A1:B3",
            "updatedRows": 3,
            "updatedColumns": 2,
            "updatedCells": 6
        }"#;
        let resp: UpdateValuesResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.spreadsheet_id, Some("abc123".to_string()));
        assert_eq!(resp.updated_range, Some("Sheet1!A1:B3".to_string()));
        assert_eq!(resp.updated_rows, Some(3));
        assert_eq!(resp.updated_columns, Some(2));
        assert_eq!(resp.updated_cells, Some(6));
    }

    // Requirement: REQ-SHEETS-002 (Must)
    // Acceptance: UpdateValuesResponse round-trip
    #[test]
    fn req_sheets_002_update_values_response_roundtrip() {
        let resp = UpdateValuesResponse {
            spreadsheet_id: Some("s1".to_string()),
            updated_range: Some("A1:C3".to_string()),
            updated_rows: Some(3),
            updated_columns: Some(3),
            updated_cells: Some(9),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: UpdateValuesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.updated_cells, Some(9));
    }

    // ---------------------------------------------------------------
    // AppendValuesResponse
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-003 (Must)
    // Acceptance: AppendValuesResponse deserializes
    #[test]
    fn req_sheets_003_append_values_response_deserialize() {
        let json_str = r#"{
            "spreadsheetId": "abc123",
            "updates": {
                "spreadsheetId": "abc123",
                "updatedRange": "Sheet1!A4:B4",
                "updatedRows": 1,
                "updatedColumns": 2,
                "updatedCells": 2
            }
        }"#;
        let resp: AppendValuesResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.spreadsheet_id, Some("abc123".to_string()));
        let updates = resp.updates.unwrap();
        assert_eq!(updates.updated_rows, Some(1));
    }

    // ---------------------------------------------------------------
    // BatchUpdateResponse
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-006 (Must)
    // Acceptance: BatchUpdateResponse deserializes
    #[test]
    fn req_sheets_006_batch_update_response_deserialize() {
        let json_str = r#"{
            "spreadsheetId": "abc123",
            "replies": [
                {},
                {"addSheet": {"properties": {"sheetId": 1}}}
            ]
        }"#;
        let resp: BatchUpdateResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.spreadsheet_id, Some("abc123".to_string()));
        assert_eq!(resp.replies.len(), 2);
    }

    // ---------------------------------------------------------------
    // ClearValuesResponse
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-005 (Must)
    // Acceptance: ClearValuesResponse deserializes
    #[test]
    fn req_sheets_005_clear_values_response_deserialize() {
        let json_str = r#"{
            "spreadsheetId": "abc123",
            "clearedRange": "Sheet1!A1:B10"
        }"#;
        let resp: ClearValuesResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.spreadsheet_id, Some("abc123".to_string()));
        assert_eq!(resp.cleared_range, Some("Sheet1!A1:B10".to_string()));
    }

    // ---------------------------------------------------------------
    // Cell and format types
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-001 (Must)
    // Acceptance: CellData deserializes with formattedValue
    #[test]
    fn req_sheets_001_cell_data_deserialize() {
        let json_str = r#"{
            "formattedValue": "Hello",
            "userEnteredValue": {"stringValue": "Hello"},
            "effectiveValue": {"stringValue": "Hello"},
            "note": "A note"
        }"#;
        let cell: CellData = serde_json::from_str(json_str).unwrap();
        assert_eq!(cell.formatted_value, Some("Hello".to_string()));
        assert_eq!(cell.note, Some("A note".to_string()));
        let uev = cell.user_entered_value.unwrap();
        assert_eq!(uev.string_value, Some("Hello".to_string()));
    }

    // Requirement: REQ-SHEETS-001 (Must)
    // Acceptance: ExtendedValue with number
    #[test]
    fn req_sheets_001_extended_value_number() {
        let json_str = r#"{"numberValue": 42.5}"#;
        let ev: ExtendedValue = serde_json::from_str(json_str).unwrap();
        assert_eq!(ev.number_value, Some(42.5));
        assert!(ev.string_value.is_none());
    }

    // Requirement: REQ-SHEETS-001 (Must)
    // Acceptance: ExtendedValue with formula
    #[test]
    fn req_sheets_001_extended_value_formula() {
        let json_str = r#"{"formulaValue": "=SUM(A1:A10)"}"#;
        let ev: ExtendedValue = serde_json::from_str(json_str).unwrap();
        assert_eq!(ev.formula_value, Some("=SUM(A1:A10)".to_string()));
    }

    // Requirement: REQ-SHEETS-001 (Must)
    // Acceptance: ExtendedValue with boolean
    #[test]
    fn req_sheets_001_extended_value_bool() {
        let json_str = r#"{"boolValue": true}"#;
        let ev: ExtendedValue = serde_json::from_str(json_str).unwrap();
        assert_eq!(ev.bool_value, Some(true));
    }

    // Requirement: REQ-SHEETS-006 (Must)
    // Acceptance: TextFormat deserializes
    #[test]
    fn req_sheets_006_text_format_deserialize() {
        let json_str = r#"{
            "fontFamily": "Arial",
            "fontSize": 12,
            "bold": true,
            "italic": false,
            "strikethrough": false,
            "underline": true,
            "foregroundColor": {"red": 0.0, "green": 0.0, "blue": 1.0}
        }"#;
        let tf: TextFormat = serde_json::from_str(json_str).unwrap();
        assert_eq!(tf.font_family, Some("Arial".to_string()));
        assert_eq!(tf.font_size, Some(12));
        assert_eq!(tf.bold, Some(true));
        assert_eq!(tf.underline, Some(true));
        let color = tf.foreground_color.unwrap();
        assert_eq!(color.blue, Some(1.0));
    }

    // Requirement: REQ-SHEETS-006 (Must)
    // Acceptance: Color with alpha
    #[test]
    fn req_sheets_006_color_with_alpha() {
        let json_str = r#"{"red": 1.0, "green": 0.5, "blue": 0.0, "alpha": 0.8}"#;
        let c: Color = serde_json::from_str(json_str).unwrap();
        assert_eq!(c.red, Some(1.0));
        assert_eq!(c.green, Some(0.5));
        assert_eq!(c.blue, Some(0.0));
        assert_eq!(c.alpha, Some(0.8));
    }

    // Requirement: REQ-SHEETS-008 (Must)
    // Acceptance: GridProperties deserializes with frozen counts
    #[test]
    fn req_sheets_008_grid_properties_frozen() {
        let json_str = r#"{
            "rowCount": 1000,
            "columnCount": 26,
            "frozenRowCount": 1,
            "frozenColumnCount": 2
        }"#;
        let gp: GridProperties = serde_json::from_str(json_str).unwrap();
        assert_eq!(gp.row_count, Some(1000));
        assert_eq!(gp.column_count, Some(26));
        assert_eq!(gp.frozen_row_count, Some(1));
        assert_eq!(gp.frozen_column_count, Some(2));
    }
}
