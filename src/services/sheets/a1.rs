//! A1 notation parser for Google Sheets ranges.
//!
//! Supports formats:
//! - `Sheet1!A1:B10` -- sheet with cell range
//! - `Sheet1!A:C` -- sheet with column range
//! - `Sheet1!1:5` -- sheet with row range
//! - `A1:B10` -- cell range (no sheet)
//! - `A:C` -- column range
//! - `1:5` -- row range
//! - `Sheet1!A1` -- single cell
//! - `'Sheet Name With Spaces'!A1:B10` -- quoted sheet name

/// Parsed A1 range notation.
#[derive(Debug, Clone, PartialEq)]
pub struct A1Range {
    pub sheet: Option<String>,
    pub start_col: Option<String>,
    pub start_row: Option<u32>,
    pub end_col: Option<String>,
    pub end_row: Option<u32>,
}

/// Parse A1 notation into its components.
///
/// The input should already have shell escapes resolved (use `clean_range` first).
pub fn parse_a1(input: &str) -> Result<A1Range, String> {
    if input.is_empty() {
        return Err("empty A1 notation".to_string());
    }

    let (sheet, cell_part) = split_sheet_and_range(input)?;

    if cell_part.is_empty() {
        // Sheet name only, no cell reference -- this is valid (whole sheet)
        return Ok(A1Range {
            sheet,
            start_col: None,
            start_row: None,
            end_col: None,
            end_row: None,
        });
    }

    // Split on ':' to get start and end
    let parts: Vec<&str> = cell_part.splitn(2, ':').collect();

    let (start_col, start_row) = parse_cell_ref(parts[0])?;

    if parts.len() == 1 {
        // Single cell reference
        Ok(A1Range {
            sheet,
            start_col,
            start_row,
            end_col: None,
            end_row: None,
        })
    } else {
        let (end_col, end_row) = parse_cell_ref(parts[1])?;
        Ok(A1Range {
            sheet,
            start_col,
            start_row,
            end_col,
            end_row,
        })
    }
}

/// Validate whether a string is valid A1 notation.
pub fn validate_a1(input: &str) -> bool {
    parse_a1(input).is_ok()
}

/// Convert a column letter string to a 1-based index.
/// A -> 1, B -> 2, Z -> 26, AA -> 27, AZ -> 52, etc.
pub fn column_to_index(col: &str) -> u32 {
    let mut index: u32 = 0;
    for b in col.as_bytes() {
        let c = b.to_ascii_uppercase();
        if !c.is_ascii_uppercase() {
            return 0; // invalid
        }
        index = index * 26 + (c - b'A' + 1) as u32;
    }
    index
}

/// Convert a 1-based column index to a letter string.
/// 1 -> A, 2 -> B, 26 -> Z, 27 -> AA, etc.
pub fn index_to_column(mut index: u32) -> String {
    if index == 0 {
        return String::new();
    }
    let mut result = Vec::new();
    while index > 0 {
        index -= 1;
        result.push((b'A' + (index % 26) as u8) as char);
        index /= 26;
    }
    result.reverse();
    result.into_iter().collect()
}

// ---------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------

/// Split an A1 string into optional sheet name and cell part.
fn split_sheet_and_range(input: &str) -> Result<(Option<String>, &str), String> {
    // Check for quoted sheet name: 'Sheet Name'!...
    if let Some(after_open_quote) = input.strip_prefix('\'') {
        // Find the closing quote
        if let Some(close_quote) = after_open_quote.find('\'') {
            let sheet_name = &after_open_quote[..close_quote];
            let rest = &after_open_quote[close_quote + 1..]; // skip closing quote
            if let Some(after_bang) = rest.strip_prefix('!') {
                Ok((Some(sheet_name.to_string()), after_bang))
            } else if rest.is_empty() {
                Ok((Some(sheet_name.to_string()), ""))
            } else {
                Err(format!(
                    "expected '!' after quoted sheet name, got: {}",
                    rest
                ))
            }
        } else {
            Err("unclosed quote in sheet name".to_string())
        }
    } else if let Some(bang_pos) = input.find('!') {
        let sheet_name = &input[..bang_pos];
        if sheet_name.is_empty() {
            return Err("empty sheet name before '!'".to_string());
        }
        let cell_part = &input[bang_pos + 1..];
        Ok((Some(sheet_name.to_string()), cell_part))
    } else {
        Ok((None, input))
    }
}

/// Parse a single cell reference (e.g., "A1", "A", "1") into column and row parts.
fn parse_cell_ref(s: &str) -> Result<(Option<String>, Option<u32>), String> {
    if s.is_empty() {
        return Err("empty cell reference".to_string());
    }

    let mut col_end = 0;
    for (i, c) in s.char_indices() {
        if c.is_ascii_alphabetic() {
            col_end = i + 1;
        } else {
            break;
        }
    }

    let col_part = &s[..col_end];
    let row_part = &s[col_end..];

    let col = if col_part.is_empty() {
        None
    } else {
        Some(col_part.to_uppercase())
    };

    let row = if row_part.is_empty() {
        None
    } else {
        match row_part.parse::<u32>() {
            Ok(n) if n > 0 => Some(n),
            Ok(_) => return Err("row number must be positive".to_string()),
            Err(_) => return Err(format!("invalid row number: {}", row_part)),
        }
    };

    if col.is_none() && row.is_none() {
        return Err(format!("invalid cell reference: {}", s));
    }

    Ok((col, row))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-SHEETS-012 (Must): Parse various A1 formats
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Sheet with cell range
    #[test]
    fn req_sheets_012_parse_sheet_cell_range() {
        let r = parse_a1("Sheet1!A1:B10").unwrap();
        assert_eq!(r.sheet, Some("Sheet1".to_string()));
        assert_eq!(r.start_col, Some("A".to_string()));
        assert_eq!(r.start_row, Some(1));
        assert_eq!(r.end_col, Some("B".to_string()));
        assert_eq!(r.end_row, Some(10));
    }

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Cell range without sheet
    #[test]
    fn req_sheets_012_parse_cell_range_no_sheet() {
        let r = parse_a1("A1:B10").unwrap();
        assert_eq!(r.sheet, None);
        assert_eq!(r.start_col, Some("A".to_string()));
        assert_eq!(r.start_row, Some(1));
        assert_eq!(r.end_col, Some("B".to_string()));
        assert_eq!(r.end_row, Some(10));
    }

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Single cell
    #[test]
    fn req_sheets_012_parse_single_cell() {
        let r = parse_a1("C5").unwrap();
        assert_eq!(r.sheet, None);
        assert_eq!(r.start_col, Some("C".to_string()));
        assert_eq!(r.start_row, Some(5));
        assert_eq!(r.end_col, None);
        assert_eq!(r.end_row, None);
    }

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Single cell with sheet
    #[test]
    fn req_sheets_012_parse_single_cell_with_sheet() {
        let r = parse_a1("Sheet1!A1").unwrap();
        assert_eq!(r.sheet, Some("Sheet1".to_string()));
        assert_eq!(r.start_col, Some("A".to_string()));
        assert_eq!(r.start_row, Some(1));
        assert_eq!(r.end_col, None);
    }

    // ---------------------------------------------------------------
    // REQ-SHEETS-012 (Must): Quoted sheet names
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Quoted sheet name with spaces
    #[test]
    fn req_sheets_012_parse_quoted_sheet_name() {
        let r = parse_a1("'Sheet Name With Spaces'!A1:B10").unwrap();
        assert_eq!(r.sheet, Some("Sheet Name With Spaces".to_string()));
        assert_eq!(r.start_col, Some("A".to_string()));
        assert_eq!(r.start_row, Some(1));
        assert_eq!(r.end_col, Some("B".to_string()));
        assert_eq!(r.end_row, Some(10));
    }

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Quoted sheet name single cell
    #[test]
    fn req_sheets_012_parse_quoted_sheet_single_cell() {
        let r = parse_a1("'My Sheet'!D4").unwrap();
        assert_eq!(r.sheet, Some("My Sheet".to_string()));
        assert_eq!(r.start_col, Some("D".to_string()));
        assert_eq!(r.start_row, Some(4));
    }

    // ---------------------------------------------------------------
    // REQ-SHEETS-012 (Must): Column-only ranges
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Column range
    #[test]
    fn req_sheets_012_parse_column_range() {
        let r = parse_a1("A:C").unwrap();
        assert_eq!(r.sheet, None);
        assert_eq!(r.start_col, Some("A".to_string()));
        assert_eq!(r.start_row, None);
        assert_eq!(r.end_col, Some("C".to_string()));
        assert_eq!(r.end_row, None);
    }

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Column range with sheet
    #[test]
    fn req_sheets_012_parse_column_range_with_sheet() {
        let r = parse_a1("Sheet1!A:C").unwrap();
        assert_eq!(r.sheet, Some("Sheet1".to_string()));
        assert_eq!(r.start_col, Some("A".to_string()));
        assert_eq!(r.start_row, None);
        assert_eq!(r.end_col, Some("C".to_string()));
        assert_eq!(r.end_row, None);
    }

    // ---------------------------------------------------------------
    // REQ-SHEETS-012 (Must): Row-only ranges
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Row range
    #[test]
    fn req_sheets_012_parse_row_range() {
        let r = parse_a1("1:5").unwrap();
        assert_eq!(r.sheet, None);
        assert_eq!(r.start_col, None);
        assert_eq!(r.start_row, Some(1));
        assert_eq!(r.end_col, None);
        assert_eq!(r.end_row, Some(5));
    }

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Row range with sheet
    #[test]
    fn req_sheets_012_parse_row_range_with_sheet() {
        let r = parse_a1("Sheet1!1:5").unwrap();
        assert_eq!(r.sheet, Some("Sheet1".to_string()));
        assert_eq!(r.start_col, None);
        assert_eq!(r.start_row, Some(1));
        assert_eq!(r.end_col, None);
        assert_eq!(r.end_row, Some(5));
    }

    // ---------------------------------------------------------------
    // REQ-SHEETS-012 (Must): Column/index conversion
    // ---------------------------------------------------------------

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Single-letter columns
    #[test]
    fn req_sheets_012_column_to_index_single() {
        assert_eq!(column_to_index("A"), 1);
        assert_eq!(column_to_index("B"), 2);
        assert_eq!(column_to_index("Z"), 26);
    }

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Multi-letter columns
    #[test]
    fn req_sheets_012_column_to_index_multi() {
        assert_eq!(column_to_index("AA"), 27);
        assert_eq!(column_to_index("AB"), 28);
        assert_eq!(column_to_index("AZ"), 52);
        assert_eq!(column_to_index("BA"), 53);
    }

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Case insensitive
    #[test]
    fn req_sheets_012_column_to_index_case_insensitive() {
        assert_eq!(column_to_index("a"), 1);
        assert_eq!(column_to_index("aa"), 27);
    }

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Index to column single-letter
    #[test]
    fn req_sheets_012_index_to_column_single() {
        assert_eq!(index_to_column(1), "A");
        assert_eq!(index_to_column(2), "B");
        assert_eq!(index_to_column(26), "Z");
    }

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Index to column multi-letter
    #[test]
    fn req_sheets_012_index_to_column_multi() {
        assert_eq!(index_to_column(27), "AA");
        assert_eq!(index_to_column(28), "AB");
        assert_eq!(index_to_column(52), "AZ");
        assert_eq!(index_to_column(53), "BA");
    }

    // Requirement: REQ-SHEETS-012 (Must)
    // Acceptance: Round-trip column conversion
    #[test]
    fn req_sheets_012_column_roundtrip() {
        for i in 1..=702 {
            let col = index_to_column(i);
            assert_eq!(column_to_index(&col), i, "roundtrip failed for index {}", i);
        }
    }

    // Requirement: REQ-SHEETS-012 (Must)
    // Edge case: Zero index returns empty
    #[test]
    fn req_sheets_012_index_to_column_zero() {
        assert_eq!(index_to_column(0), "");
    }

    // ---------------------------------------------------------------
    // Edge cases
    // ---------------------------------------------------------------

    // REQ-SHEETS-012
    // Edge case: Empty input
    #[test]
    fn req_sheets_012_parse_empty_input() {
        assert!(parse_a1("").is_err());
    }

    // REQ-SHEETS-012
    // Edge case: Invalid input
    #[test]
    fn req_sheets_012_parse_invalid_input() {
        assert!(parse_a1("!").is_err());
    }

    // REQ-SHEETS-012
    // Edge case: validate_a1 returns true for valid input
    #[test]
    fn req_sheets_012_validate_valid() {
        assert!(validate_a1("A1:B10"));
        assert!(validate_a1("Sheet1!A1"));
        assert!(validate_a1("'My Sheet'!A:C"));
        assert!(validate_a1("1:5"));
    }

    // REQ-SHEETS-012
    // Edge case: validate_a1 returns false for invalid input
    #[test]
    fn req_sheets_012_validate_invalid() {
        assert!(!validate_a1(""));
        assert!(!validate_a1("!"));
    }

    // REQ-SHEETS-012
    // Acceptance: Multi-column letter range
    #[test]
    fn req_sheets_012_parse_multi_column_range() {
        let r = parse_a1("AA1:AZ100").unwrap();
        assert_eq!(r.start_col, Some("AA".to_string()));
        assert_eq!(r.start_row, Some(1));
        assert_eq!(r.end_col, Some("AZ".to_string()));
        assert_eq!(r.end_row, Some(100));
    }

    // REQ-SHEETS-012
    // Acceptance: Lowercase column letters normalized to uppercase
    #[test]
    fn req_sheets_012_parse_lowercase_columns() {
        let r = parse_a1("a1:b10").unwrap();
        assert_eq!(r.start_col, Some("A".to_string()));
        assert_eq!(r.end_col, Some("B".to_string()));
    }
}
