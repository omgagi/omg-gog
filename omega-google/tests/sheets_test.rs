//! Sheets service integration tests.

use omega_google::services::sheets::types::*;
use omega_google::services::sheets::a1::*;

// ---------------------------------------------------------------
// REQ-SHEETS-001 (Must): ValueRange from realistic API response
// ---------------------------------------------------------------

// Requirement: REQ-SHEETS-001 (Must)
// Acceptance: Full ValueRange from a realistic Sheets API response
#[test]
fn req_sheets_001_integration_value_range_from_api() {
    let api_response = r#"{
        "range": "'Sales Data'!A1:E6",
        "majorDimension": "ROWS",
        "values": [
            ["Product", "Q1 Sales", "Q2 Sales", "Q3 Sales", "Q4 Sales"],
            ["Widget A", "1250", "1340", "1180", "1500"],
            ["Widget B", "890", "920", "1050", "1100"],
            ["Widget C", "2100", "1950", "2200", "2350"],
            ["Widget D", "450", "520", "490", "610"],
            ["Total", "4690", "4730", "4920", "5560"]
        ]
    }"#;

    let vr: ValueRange = serde_json::from_str(api_response).unwrap();
    assert_eq!(vr.range, Some("'Sales Data'!A1:E6".to_string()));
    assert_eq!(vr.major_dimension, Some("ROWS".to_string()));
    assert_eq!(vr.values.len(), 6);

    // Header row
    assert_eq!(vr.values[0][0], "Product");
    assert_eq!(vr.values[0][4], "Q4 Sales");

    // Data rows
    assert_eq!(vr.values[1][0], "Widget A");
    assert_eq!(vr.values[1][1], "1250");

    // Total row
    assert_eq!(vr.values[5][0], "Total");
    assert_eq!(vr.values[5][4], "5560");
}

// Requirement: REQ-SHEETS-001 (Must)
// Acceptance: ValueRange with mixed types
#[test]
fn req_sheets_001_integration_value_range_mixed_types() {
    let api_response = r#"{
        "range": "Sheet1!A1:C3",
        "majorDimension": "ROWS",
        "values": [
            ["Name", "Score", "Pass"],
            ["Alice", 95, true],
            ["Bob", 72.5, false]
        ]
    }"#;

    let vr: ValueRange = serde_json::from_str(api_response).unwrap();
    assert_eq!(vr.values.len(), 3);
    // Numbers and booleans
    assert_eq!(vr.values[1][1], 95);
    assert_eq!(vr.values[1][2], true);
    assert_eq!(vr.values[2][1], 72.5);
    assert_eq!(vr.values[2][2], false);
}

// ---------------------------------------------------------------
// REQ-SHEETS-008 (Must): Full spreadsheet metadata deserialization
// ---------------------------------------------------------------

// Requirement: REQ-SHEETS-008 (Must)
// Acceptance: Complex spreadsheet with multiple sheets and grid data
#[test]
fn req_sheets_008_integration_full_spreadsheet_metadata() {
    let api_response = r#"{
        "spreadsheetId": "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgVE2upms",
        "properties": {
            "title": "Company Budget 2024",
            "locale": "en_US",
            "autoRecalc": "ON_CHANGE",
            "timeZone": "America/New_York",
            "defaultFormat": {
                "padding": {"top": 2, "right": 3, "bottom": 2, "left": 3}
            }
        },
        "sheets": [
            {
                "properties": {
                    "sheetId": 0,
                    "title": "Revenue",
                    "index": 0,
                    "sheetType": "GRID",
                    "gridProperties": {
                        "rowCount": 1000,
                        "columnCount": 26,
                        "frozenRowCount": 1,
                        "frozenColumnCount": 0
                    }
                }
            },
            {
                "properties": {
                    "sheetId": 1234567890,
                    "title": "Expenses",
                    "index": 1,
                    "sheetType": "GRID",
                    "gridProperties": {
                        "rowCount": 500,
                        "columnCount": 10,
                        "frozenRowCount": 2,
                        "frozenColumnCount": 1
                    }
                }
            },
            {
                "properties": {
                    "sheetId": 987654321,
                    "title": "Summary",
                    "index": 2,
                    "sheetType": "GRID",
                    "gridProperties": {
                        "rowCount": 100,
                        "columnCount": 5
                    }
                }
            }
        ],
        "namedRanges": [
            {
                "namedRangeId": "nr_001",
                "name": "TotalRevenue",
                "range": {
                    "sheetId": 0,
                    "startRowIndex": 50,
                    "endRowIndex": 51,
                    "startColumnIndex": 5,
                    "endColumnIndex": 6
                }
            }
        ]
    }"#;

    let ss: Spreadsheet = serde_json::from_str(api_response).unwrap();

    // Verify spreadsheet metadata
    assert_eq!(ss.spreadsheet_id, Some("1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgVE2upms".to_string()));
    let props = ss.properties.as_ref().unwrap();
    assert_eq!(props.title, Some("Company Budget 2024".to_string()));
    assert_eq!(props.locale, Some("en_US".to_string()));
    assert_eq!(props.auto_recalc, Some("ON_CHANGE".to_string()));
    assert_eq!(props.time_zone, Some("America/New_York".to_string()));

    // Verify sheets
    assert_eq!(ss.sheets.len(), 3);

    let revenue = ss.sheets[0].properties.as_ref().unwrap();
    assert_eq!(revenue.sheet_id, Some(0));
    assert_eq!(revenue.title, Some("Revenue".to_string()));
    let revenue_grid = revenue.grid_properties.as_ref().unwrap();
    assert_eq!(revenue_grid.row_count, Some(1000));
    assert_eq!(revenue_grid.frozen_row_count, Some(1));

    let expenses = ss.sheets[1].properties.as_ref().unwrap();
    assert_eq!(expenses.sheet_id, Some(1234567890));
    assert_eq!(expenses.title, Some("Expenses".to_string()));
    let expenses_grid = expenses.grid_properties.as_ref().unwrap();
    assert_eq!(expenses_grid.frozen_row_count, Some(2));
    assert_eq!(expenses_grid.frozen_column_count, Some(1));

    let summary = ss.sheets[2].properties.as_ref().unwrap();
    assert_eq!(summary.title, Some("Summary".to_string()));
    assert_eq!(summary.index, Some(2));

    // Verify named ranges
    assert_eq!(ss.named_ranges.len(), 1);
}

// ---------------------------------------------------------------
// REQ-SHEETS-012 (Must): A1 notation parsing integration
// ---------------------------------------------------------------

// Requirement: REQ-SHEETS-012 (Must)
// Acceptance: Parse various real-world A1 notations
#[test]
fn req_sheets_012_integration_a1_parsing_real_world() {
    // Standard range with sheet name
    let r1 = parse_a1("Revenue!A1:Z100").unwrap();
    assert_eq!(r1.sheet, Some("Revenue".to_string()));
    assert_eq!(r1.start_col, Some("A".to_string()));
    assert_eq!(r1.start_row, Some(1));
    assert_eq!(r1.end_col, Some("Z".to_string()));
    assert_eq!(r1.end_row, Some(100));

    // Quoted sheet name with spaces
    let r2 = parse_a1("'Q4 Sales Data'!B2:F50").unwrap();
    assert_eq!(r2.sheet, Some("Q4 Sales Data".to_string()));
    assert_eq!(r2.start_col, Some("B".to_string()));
    assert_eq!(r2.start_row, Some(2));

    // Entire column range
    let r3 = parse_a1("Sheet1!A:A").unwrap();
    assert_eq!(r3.sheet, Some("Sheet1".to_string()));
    assert_eq!(r3.start_col, Some("A".to_string()));
    assert_eq!(r3.start_row, None);
    assert_eq!(r3.end_col, Some("A".to_string()));
    assert_eq!(r3.end_row, None);

    // Row range
    let r4 = parse_a1("1:100").unwrap();
    assert_eq!(r4.sheet, None);
    assert_eq!(r4.start_col, None);
    assert_eq!(r4.start_row, Some(1));
    assert_eq!(r4.end_row, Some(100));

    // Multi-letter column (AA, AB, etc.)
    let r5 = parse_a1("AA1:AZ100").unwrap();
    assert_eq!(r5.start_col, Some("AA".to_string()));
    assert_eq!(r5.end_col, Some("AZ".to_string()));
}

// Requirement: REQ-SHEETS-012 (Must)
// Acceptance: Column index conversion round-trip
#[test]
fn req_sheets_012_integration_column_conversion() {
    // Standard columns
    assert_eq!(column_to_index("A"), 1);
    assert_eq!(column_to_index("Z"), 26);
    assert_eq!(column_to_index("AA"), 27);
    assert_eq!(column_to_index("AZ"), 52);
    assert_eq!(column_to_index("BA"), 53);
    assert_eq!(column_to_index("ZZ"), 702);

    // Reverse
    assert_eq!(index_to_column(1), "A");
    assert_eq!(index_to_column(26), "Z");
    assert_eq!(index_to_column(27), "AA");
    assert_eq!(index_to_column(702), "ZZ");

    // Round-trip for all columns up to ZZ
    for i in 1..=702 {
        let col = index_to_column(i);
        assert_eq!(column_to_index(&col), i, "failed round-trip for column index {}", i);
    }
}

// Requirement: REQ-SHEETS-012 (Must)
// Acceptance: Shell escape handling with clean_range
#[test]
fn req_sheets_012_integration_clean_range_and_parse() {
    use omega_google::services::sheets::clean_range;

    let escaped = "Sheet1\\!A1:B10";
    let cleaned = clean_range(escaped);
    assert_eq!(cleaned, "Sheet1!A1:B10");

    let r = parse_a1(&cleaned).unwrap();
    assert_eq!(r.sheet, Some("Sheet1".to_string()));
    assert_eq!(r.start_col, Some("A".to_string()));
    assert_eq!(r.start_row, Some(1));
}

// ---------------------------------------------------------------
// REQ-SHEETS-001 (Must): CellData with notes
// ---------------------------------------------------------------

// Requirement: REQ-SHEETS-001 (Must)
// Acceptance: CellData with notes from realistic API response
#[test]
fn req_sheets_001_integration_cell_data_with_notes() {
    let api_response = r#"{
        "formattedValue": "Revenue Total",
        "userEnteredValue": {
            "formulaValue": "=SUM(B2:B100)"
        },
        "effectiveValue": {
            "numberValue": 125000.50
        },
        "effectiveFormat": {
            "backgroundColor": {"red": 0.9, "green": 0.95, "blue": 1.0},
            "textFormat": {
                "fontFamily": "Arial",
                "fontSize": 11,
                "bold": true
            },
            "numberFormat": {
                "type": "CURRENCY",
                "pattern": "$#,##0.00"
            }
        },
        "note": "This includes all product lines for Q1-Q4."
    }"#;

    let cell: CellData = serde_json::from_str(api_response).unwrap();
    assert_eq!(cell.formatted_value, Some("Revenue Total".to_string()));
    assert_eq!(cell.note, Some("This includes all product lines for Q1-Q4.".to_string()));

    let uev = cell.user_entered_value.unwrap();
    assert_eq!(uev.formula_value, Some("=SUM(B2:B100)".to_string()));

    let ev = cell.effective_value.unwrap();
    assert_eq!(ev.number_value, Some(125000.50));

    let format = cell.effective_format.unwrap();
    let tf = format.text_format.unwrap();
    assert_eq!(tf.font_family, Some("Arial".to_string()));
    assert_eq!(tf.bold, Some(true));
}

// ---------------------------------------------------------------
// REQ-SHEETS-002 (Must): UpdateValuesResponse integration
// ---------------------------------------------------------------

// Requirement: REQ-SHEETS-002 (Must)
// Acceptance: UpdateValuesResponse from realistic API response
#[test]
fn req_sheets_002_integration_update_response() {
    let api_response = r#"{
        "spreadsheetId": "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgVE2upms",
        "updatedRange": "'Sales Data'!A1:E6",
        "updatedRows": 6,
        "updatedColumns": 5,
        "updatedCells": 30
    }"#;

    let resp: UpdateValuesResponse = serde_json::from_str(api_response).unwrap();
    assert_eq!(resp.updated_rows, Some(6));
    assert_eq!(resp.updated_columns, Some(5));
    assert_eq!(resp.updated_cells, Some(30));
}
