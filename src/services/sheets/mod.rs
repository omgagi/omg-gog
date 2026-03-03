//! Google Sheets service module.
//! Provides types, A1 notation parsing, and URL/body builders for the Sheets API.

pub mod a1;
pub mod format;
pub mod read;
pub mod structure;
pub mod types;
pub mod write;

/// Google Sheets API v4 base URL.
pub const SHEETS_BASE_URL: &str = "https://sheets.googleapis.com/v4";

/// Clean a range string by unescaping shell escapes for '!'.
/// Users may type `Sheet1\!A1:B10` to avoid shell interpretation.
pub fn clean_range(range: &str) -> String {
    range.replace("\\!", "!")
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ-SHEETS-012
    #[test]
    fn req_sheets_012_clean_range_basic() {
        assert_eq!(clean_range("Sheet1\\!A1:B10"), "Sheet1!A1:B10");
    }

    // REQ-SHEETS-012
    #[test]
    fn req_sheets_012_clean_range_no_escape() {
        assert_eq!(clean_range("Sheet1!A1:B10"), "Sheet1!A1:B10");
    }

    // REQ-SHEETS-012
    #[test]
    fn req_sheets_012_clean_range_multiple_escapes() {
        assert_eq!(clean_range("Sheet\\!Name\\!A1"), "Sheet!Name!A1");
    }
}
