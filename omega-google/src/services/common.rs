//! Shared types for service implementations: pagination parameters, list response helpers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard pagination parameters shared across all list commands.
#[derive(Debug, Clone, Default)]
pub struct PaginationParams {
    /// Maximum number of results per page.
    pub max_results: Option<u32>,
    /// Page token for continuing from a previous request.
    pub page_token: Option<String>,
    /// Fetch all pages automatically.
    pub all_pages: bool,
    /// Fail with exit code 3 if no results.
    pub fail_empty: bool,
}

/// Generic list response wrapper for paginated API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse<T> {
    /// The items in this page.
    pub items: Vec<T>,
    /// Token for the next page, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    /// Estimated total result count (not always present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_size_estimate: Option<u32>,
    /// Unknown fields for forward compatibility.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Format a byte count as a human-readable size string.
/// Matches gogcli behavior: 0 or negative returns "-", otherwise B/KB/MB/GB/TB.
pub fn format_size(bytes: i64) -> String {
    if bytes <= 0 {
        return "-".to_string();
    }
    const UNIT: f64 = 1024.0;
    let mut b = bytes as f64;
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut i = 0;
    while b >= UNIT && i < units.len() - 1 {
        b /= UNIT;
        i += 1;
    }
    if i == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", b, units[i])
    }
}

/// Format an ISO 8601 datetime string for display.
/// Truncates to "YYYY-MM-DD HH:MM" and replaces T with space.
/// Empty string returns "-".
pub fn format_datetime(iso: &str) -> String {
    if iso.is_empty() {
        return "-".to_string();
    }
    if iso.len() >= 16 {
        iso[..16].replace('T', " ")
    } else {
        iso.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-DRIVE-001 (Must): File size formatting
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: File sizes displayed as human-readable (B, KB, MB, GB)
    #[test]
    fn req_drive_001_format_size_zero() {
        assert_eq!(format_size(0), "-");
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: Negative size returns "-"
    #[test]
    fn req_drive_001_format_size_negative() {
        assert_eq!(format_size(-1), "-");
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: Bytes displayed directly for small sizes
    #[test]
    fn req_drive_001_format_size_bytes() {
        assert_eq!(format_size(512), "512 B");
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: KB formatting
    #[test]
    fn req_drive_001_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: MB formatting
    #[test]
    fn req_drive_001_format_size_megabytes() {
        assert_eq!(format_size(1048576), "1.0 MB");
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: GB formatting
    #[test]
    fn req_drive_001_format_size_gigabytes() {
        assert_eq!(format_size(1073741824), "1.0 GB");
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: TB formatting for very large files
    #[test]
    fn req_drive_001_format_size_terabytes() {
        assert_eq!(format_size(1099511627776), "1.0 TB");
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Edge case: Exactly 1 byte
    #[test]
    fn req_drive_001_format_size_one_byte() {
        assert_eq!(format_size(1), "1 B");
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-001 (Must): DateTime formatting
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: ISO datetime truncated and T replaced
    #[test]
    fn req_drive_001_format_datetime_iso() {
        assert_eq!(
            format_datetime("2024-01-15T14:30:00.000Z"),
            "2024-01-15 14:30"
        );
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: Empty datetime returns "-"
    #[test]
    fn req_drive_001_format_datetime_empty() {
        assert_eq!(format_datetime(""), "-");
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Edge case: Short datetime string (< 16 chars)
    #[test]
    fn req_drive_001_format_datetime_short() {
        assert_eq!(format_datetime("2024-01-15"), "2024-01-15");
    }

    // ---------------------------------------------------------------
    // Pagination params construction
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-024 (Must)
    // Acceptance: Pagination params default correctly
    #[test]
    fn req_cli_024_pagination_defaults() {
        let params = PaginationParams::default();
        assert_eq!(params.max_results, None);
        assert_eq!(params.page_token, None);
        assert!(!params.all_pages);
        assert!(!params.fail_empty);
    }

    // Requirement: REQ-CLI-024 (Must)
    // Acceptance: ListResponse serialization round-trip
    #[test]
    fn req_cli_024_list_response_serde_roundtrip() {
        let resp = ListResponse::<String> {
            items: vec!["a".to_string(), "b".to_string()],
            next_page_token: Some("token123".to_string()),
            result_size_estimate: Some(42),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: ListResponse<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.items.len(), 2);
        assert_eq!(parsed.next_page_token, Some("token123".to_string()));
        assert_eq!(parsed.result_size_estimate, Some(42));
    }
}
