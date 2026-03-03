//! Sheets read operations: values.get and spreadsheets.get URL builders.

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use super::SHEETS_BASE_URL;

/// Build URL for values.get (read cell values).
///
/// GET /v4/spreadsheets/{spreadsheetId}/values/{range}
pub fn build_values_get_url(spreadsheet_id: &str, range: &str) -> String {
    let encoded_range = utf8_percent_encode(range, NON_ALPHANUMERIC).to_string();
    format!(
        "{}/spreadsheets/{}/values/{}",
        SHEETS_BASE_URL, spreadsheet_id, encoded_range
    )
}

/// Build URL for values.get with optional query parameters.
///
/// Supports `majorDimension` and `valueRenderOption`.
pub fn build_values_get_url_with_options(
    spreadsheet_id: &str,
    range: &str,
    dimension: Option<&str>,
    render: Option<&str>,
) -> String {
    let base = build_values_get_url(spreadsheet_id, range);
    let mut params = Vec::new();

    if let Some(dim) = dimension {
        params.push(format!("majorDimension={}", dim));
    }
    if let Some(r) = render {
        params.push(format!("valueRenderOption={}", r));
    }

    if params.is_empty() {
        base
    } else {
        format!("{}?{}", base, params.join("&"))
    }
}

/// Build URL for spreadsheets.get (spreadsheet metadata).
///
/// GET /v4/spreadsheets/{spreadsheetId}
pub fn build_metadata_url(spreadsheet_id: &str) -> String {
    format!("{}/spreadsheets/{}", SHEETS_BASE_URL, spreadsheet_id)
}

/// Build URL for spreadsheets.get with optional fields parameter.
pub fn build_metadata_url_with_fields(spreadsheet_id: &str, fields: Option<&str>) -> String {
    let base = build_metadata_url(spreadsheet_id);
    match fields {
        Some(f) => format!(
            "{}?fields={}",
            base,
            utf8_percent_encode(f, NON_ALPHANUMERIC)
        ),
        None => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-SHEETS-001 (Must): Values get URL construction
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-001 (Must)
    // Acceptance: Basic values get URL
    #[test]
    fn req_sheets_001_values_get_url_basic() {
        let url = build_values_get_url("abc123", "Sheet1!A1:B10");
        assert!(url.starts_with("https://sheets.googleapis.com/v4/spreadsheets/abc123/values/"));
        // Range should be percent-encoded
        assert!(url.contains("Sheet1"));
        assert!(url.contains("A1"));
    }

    // Requirement: REQ-SHEETS-001 (Must)
    // Acceptance: Values get URL with dimension option
    #[test]
    fn req_sheets_001_values_get_url_with_dimension() {
        let url = build_values_get_url_with_options("abc123", "A1:B10", Some("COLUMNS"), None);
        assert!(url.contains("majorDimension=COLUMNS"));
    }

    // Requirement: REQ-SHEETS-001 (Must)
    // Acceptance: Values get URL with render option
    #[test]
    fn req_sheets_001_values_get_url_with_render() {
        let url = build_values_get_url_with_options("abc123", "A1:B10", None, Some("FORMULA"));
        assert!(url.contains("valueRenderOption=FORMULA"));
    }

    // Requirement: REQ-SHEETS-001 (Must)
    // Acceptance: Values get URL with both options
    #[test]
    fn req_sheets_001_values_get_url_with_both_options() {
        let url = build_values_get_url_with_options(
            "abc123",
            "A1:B10",
            Some("ROWS"),
            Some("FORMATTED_VALUE"),
        );
        assert!(url.contains("majorDimension=ROWS"));
        assert!(url.contains("valueRenderOption=FORMATTED_VALUE"));
        assert!(url.contains("&"));
    }

    // Requirement: REQ-SHEETS-001 (Must)
    // Acceptance: Values get URL with no options (no query string)
    #[test]
    fn req_sheets_001_values_get_url_no_options() {
        let url = build_values_get_url_with_options("abc123", "A1:B10", None, None);
        assert!(!url.contains("?"));
    }

    // ---------------------------------------------------------------
    // REQ-SHEETS-008 (Must): Metadata URL construction
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-008 (Must)
    // Acceptance: Basic metadata URL
    #[test]
    fn req_sheets_008_metadata_url_basic() {
        let url = build_metadata_url("abc123");
        assert_eq!(url, "https://sheets.googleapis.com/v4/spreadsheets/abc123");
    }

    // Requirement: REQ-SHEETS-008 (Must)
    // Acceptance: Metadata URL with fields
    #[test]
    fn req_sheets_008_metadata_url_with_fields() {
        let url =
            build_metadata_url_with_fields("abc123", Some("properties.title,sheets.properties"));
        assert!(url.contains("spreadsheets/abc123"));
        assert!(url.contains("?fields="));
    }

    // Requirement: REQ-SHEETS-008 (Must)
    // Acceptance: Metadata URL without fields
    #[test]
    fn req_sheets_008_metadata_url_without_fields() {
        let url = build_metadata_url_with_fields("abc123", None);
        assert!(!url.contains("?"));
    }

    // ---------------------------------------------------------------
    // Edge cases
    // ---------------------------------------------------------------

    // REQ-SHEETS-001
    // Edge case: Special characters in range are percent-encoded
    #[test]
    fn req_sheets_001_values_get_url_special_chars() {
        let url = build_values_get_url("abc123", "Sheet 1!A1:B10");
        // Space should be percent-encoded
        assert!(url.contains("Sheet%201"));
    }
}
