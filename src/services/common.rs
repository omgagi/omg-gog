//! Shared types for service implementations: pagination parameters, list response helpers,
//! and shared watch channel types for Calendar/Drive push notifications.

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

// ---------------------------------------------------------------
// Shared watch channel types (OI-M3)
// Used by both Calendar and Drive push notification watch commands.
// Gmail has its own types (different response schema: historyId + expiration).
// ---------------------------------------------------------------

/// Request body for registering a push notification watch channel.
///
/// Used by Calendar `events/watch` and Drive `changes/watch`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchChannelRequest {
    /// UUID v4 channel identifier.
    pub id: String,
    /// Channel type -- always "web_hook".
    #[serde(rename = "type")]
    pub channel_type: String,
    /// HTTPS callback URL that receives notifications.
    pub address: String,
    /// Optional parameters (e.g. TTL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<WatchParams>,
}

/// Optional parameters for a watch channel request.
#[derive(Debug, Clone, Serialize)]
pub struct WatchParams {
    /// Time-to-live in seconds (e.g. "604800" for 7 days).
    pub ttl: String,
}

/// Response from registering a push notification watch channel.
///
/// Returned by Calendar `events/watch` and Drive `changes/watch`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchChannelResponse {
    /// The channel ID (same UUID v4 that was sent in the request).
    pub id: String,
    /// Server-assigned resource ID for this watch.
    pub resource_id: String,
    /// Expiration time in milliseconds since epoch (as a string), if present.
    pub expiration: Option<String>,
}

/// Request body for stopping a push notification watch channel.
///
/// Used by Calendar `channels/stop` and Drive `channels/stop`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelStopRequest {
    /// The channel ID from the original watch response.
    pub id: String,
    /// The resource ID from the original watch response.
    pub resource_id: String,
}

/// Response from the Drive `changes/getStartPageToken` endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPageTokenResponse {
    /// The start page token for watching future changes.
    pub start_page_token: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===================================================================
    // OI-M3: Shared Watch Channel Types -- TDD tests
    //
    // These tests define the serialization/deserialization contract for
    // WatchChannelRequest, WatchChannelResponse, ChannelStopRequest,
    // and StartPageTokenResponse. Written BEFORE implementation.
    // ===================================================================

    // -------------------------------------------------------------------
    // REQ-OI-026 (Must): WatchChannelRequest serialization
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: WatchChannelRequest serializes with camelCase field names
    #[test]
    fn req_oi_026_watch_channel_request_serializes_camel_case() {
        let req = WatchChannelRequest {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            channel_type: "web_hook".to_string(),
            address: "https://example.com/webhook".to_string(),
            params: Some(WatchParams {
                ttl: "604800".to_string(),
            }),
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        // "type" field should be present (renamed from channel_type)
        assert_eq!(json_val["type"], "web_hook");
        assert_eq!(json_val["id"], "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(json_val["address"], "https://example.com/webhook");
        assert_eq!(json_val["params"]["ttl"], "604800");
        // snake_case keys should NOT appear
        assert!(json_val.get("channel_type").is_none());
    }

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: WatchChannelRequest with None params omits params field
    #[test]
    fn req_oi_026_watch_channel_request_omits_none_params() {
        let req = WatchChannelRequest {
            id: "test-uuid".to_string(),
            channel_type: "web_hook".to_string(),
            address: "https://example.com/hook".to_string(),
            params: None,
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert!(
            json_val.get("params").is_none(),
            "None params should be omitted from serialized JSON"
        );
    }

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: WatchChannelRequest "type" field serializes correctly
    //   (tests the #[serde(rename = "type")] on channel_type)
    #[test]
    fn req_oi_026_watch_channel_request_type_field_rename() {
        let req = WatchChannelRequest {
            id: "uuid".to_string(),
            channel_type: "web_hook".to_string(),
            address: "https://example.com".to_string(),
            params: None,
        };
        let json_str = serde_json::to_string(&req).unwrap();
        assert!(
            json_str.contains(r#""type":"web_hook"#),
            "Should serialize as 'type', not 'channel_type': {}",
            json_str
        );
        assert!(
            !json_str.contains("channelType"),
            "Should not contain 'channelType': {}",
            json_str
        );
    }

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: WatchChannelRequest with TTL 604800 (7 days)
    #[test]
    fn req_oi_026_watch_channel_request_ttl_seven_days() {
        let req = WatchChannelRequest {
            id: "channel-1".to_string(),
            channel_type: "web_hook".to_string(),
            address: "https://example.com/callback".to_string(),
            params: Some(WatchParams {
                ttl: "604800".to_string(),
            }),
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(json_val["params"]["ttl"], "604800");
    }

    // Requirement: REQ-OI-026 (Must)
    // Edge case: WatchChannelRequest with empty id
    #[test]
    fn req_oi_026_watch_channel_request_empty_id() {
        let req = WatchChannelRequest {
            id: "".to_string(),
            channel_type: "web_hook".to_string(),
            address: "https://example.com".to_string(),
            params: None,
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(json_val["id"], "");
    }

    // Requirement: REQ-OI-026 (Must)
    // Edge case: WatchChannelRequest with URL containing special characters
    #[test]
    fn req_oi_026_watch_channel_request_special_url_chars() {
        let req = WatchChannelRequest {
            id: "uuid-1".to_string(),
            channel_type: "web_hook".to_string(),
            address: "https://example.com/hook?param=value&other=123".to_string(),
            params: None,
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json_val["address"],
            "https://example.com/hook?param=value&other=123"
        );
    }

    // -------------------------------------------------------------------
    // REQ-OI-026 (Must): WatchChannelResponse deserialization
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: WatchChannelResponse deserializes from Google API JSON
    #[test]
    fn req_oi_026_watch_channel_response_deserializes() {
        let json_str = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "resourceId": "resource-abc-123",
            "expiration": "1704153600000"
        }"#;
        let resp: WatchChannelResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(resp.resource_id, "resource-abc-123");
        assert_eq!(resp.expiration, Some("1704153600000".to_string()));
    }

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: WatchChannelResponse round-trip serialization
    #[test]
    fn req_oi_026_watch_channel_response_roundtrip() {
        let resp = WatchChannelResponse {
            id: "test-channel".to_string(),
            resource_id: "res-123".to_string(),
            expiration: Some("9999999999999".to_string()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: WatchChannelResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "test-channel");
        assert_eq!(parsed.resource_id, "res-123");
        assert_eq!(parsed.expiration, Some("9999999999999".to_string()));
    }

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: WatchChannelResponse serializes to camelCase
    #[test]
    fn req_oi_026_watch_channel_response_camel_case() {
        let resp = WatchChannelResponse {
            id: "ch".to_string(),
            resource_id: "res".to_string(),
            expiration: Some("100".to_string()),
        };
        let json_val: serde_json::Value = serde_json::to_value(&resp).unwrap();
        assert!(json_val.get("resourceId").is_some());
        assert!(json_val.get("resource_id").is_none());
    }

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: WatchChannelResponse with missing expiration
    #[test]
    fn req_oi_026_watch_channel_response_no_expiration() {
        let json_str = r#"{
            "id": "ch-1",
            "resourceId": "res-1"
        }"#;
        let resp: WatchChannelResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.id, "ch-1");
        assert_eq!(resp.resource_id, "res-1");
        assert_eq!(resp.expiration, None);
    }

    // Requirement: REQ-OI-026 (Must)
    // Edge case: WatchChannelResponse with extra unknown fields (forward compat)
    #[test]
    fn req_oi_026_watch_channel_response_extra_fields() {
        let json_str = r#"{
            "id": "ch-1",
            "resourceId": "res-1",
            "expiration": "123",
            "unknownField": "should not break"
        }"#;
        let result: Result<WatchChannelResponse, _> = serde_json::from_str(json_str);
        assert!(
            result.is_ok(),
            "Extra fields should be ignored: {:?}",
            result.err()
        );
    }

    // Requirement: REQ-OI-026 (Must)
    // Edge case: WatchChannelResponse missing required id
    #[test]
    fn req_oi_026_watch_channel_response_missing_id() {
        let json_str = r#"{"resourceId": "res-1"}"#;
        let result: Result<WatchChannelResponse, _> = serde_json::from_str(json_str);
        assert!(
            result.is_err(),
            "Missing required 'id' should fail deserialization"
        );
    }

    // Requirement: REQ-OI-026 (Must)
    // Edge case: WatchChannelResponse missing required resourceId
    #[test]
    fn req_oi_026_watch_channel_response_missing_resource_id() {
        let json_str = r#"{"id": "ch-1"}"#;
        let result: Result<WatchChannelResponse, _> = serde_json::from_str(json_str);
        assert!(
            result.is_err(),
            "Missing required 'resourceId' should fail deserialization"
        );
    }

    // -------------------------------------------------------------------
    // REQ-OI-026 (Must): ChannelStopRequest serialization
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: ChannelStopRequest serializes with camelCase
    #[test]
    fn req_oi_026_channel_stop_request_serializes_camel_case() {
        let req = ChannelStopRequest {
            id: "channel-uuid".to_string(),
            resource_id: "resource-abc".to_string(),
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(json_val["id"], "channel-uuid");
        assert_eq!(json_val["resourceId"], "resource-abc");
        // snake_case key should NOT appear
        assert!(json_val.get("resource_id").is_none());
    }

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: ChannelStopRequest matches expected JSON body format
    #[test]
    fn req_oi_026_channel_stop_request_json_format() {
        let req = ChannelStopRequest {
            id: "ch-123".to_string(),
            resource_id: "res-456".to_string(),
        };
        let json_str = serde_json::to_string(&req).unwrap();
        // Should contain exactly "id" and "resourceId" keys
        assert!(json_str.contains(r#""id":"ch-123""#));
        assert!(json_str.contains(r#""resourceId":"res-456""#));
    }

    // -------------------------------------------------------------------
    // REQ-OI-026 (Must): StartPageTokenResponse deserialization
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: StartPageTokenResponse deserializes from Google API JSON
    #[test]
    fn req_oi_026_start_page_token_response_deserializes() {
        let json_str = r#"{"startPageToken": "12345"}"#;
        let resp: StartPageTokenResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.start_page_token, "12345");
    }

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: StartPageTokenResponse round-trip serialization
    #[test]
    fn req_oi_026_start_page_token_response_roundtrip() {
        let resp = StartPageTokenResponse {
            start_page_token: "67890".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: StartPageTokenResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.start_page_token, "67890");
    }

    // Requirement: REQ-OI-026 (Must)
    // Acceptance: StartPageTokenResponse serializes to camelCase
    #[test]
    fn req_oi_026_start_page_token_response_camel_case() {
        let resp = StartPageTokenResponse {
            start_page_token: "999".to_string(),
        };
        let json_val: serde_json::Value = serde_json::to_value(&resp).unwrap();
        assert!(json_val.get("startPageToken").is_some());
        assert!(json_val.get("start_page_token").is_none());
    }

    // Requirement: REQ-OI-026 (Must)
    // Edge case: StartPageTokenResponse missing required field
    #[test]
    fn req_oi_026_start_page_token_response_missing_field() {
        let json_str = r#"{}"#;
        let result: Result<StartPageTokenResponse, _> = serde_json::from_str(json_str);
        assert!(
            result.is_err(),
            "Missing required 'startPageToken' should fail deserialization"
        );
    }

    // Requirement: REQ-OI-026 (Must)
    // Edge case: StartPageTokenResponse with extra unknown fields
    #[test]
    fn req_oi_026_start_page_token_response_extra_fields() {
        let json_str = r#"{"startPageToken": "1", "kind": "drive#startPageToken"}"#;
        let result: Result<StartPageTokenResponse, _> = serde_json::from_str(json_str);
        assert!(
            result.is_ok(),
            "Extra fields should be ignored: {:?}",
            result.err()
        );
    }

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
