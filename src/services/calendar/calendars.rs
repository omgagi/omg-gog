//! Calendar list and ACL operations.

use super::types::CALENDAR_BASE_URL;

/// Build URL for listing calendars.
pub fn build_calendars_list_url(max_results: Option<u32>, page_token: Option<&str>) -> String {
    let base = format!("{}/users/me/calendarList", CALENDAR_BASE_URL);
    let mut params = Vec::new();
    if let Some(max) = max_results {
        params.push(format!("maxResults={}", max));
    }
    if let Some(token) = page_token {
        params.push(format!("pageToken={}", token));
    }
    if params.is_empty() {
        base
    } else {
        format!("{}?{}", base, params.join("&"))
    }
}

/// Build URL for listing ACL entries.
pub fn build_acl_list_url(calendar_id: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded_cal = utf8_percent_encode(calendar_id, NON_ALPHANUMERIC).to_string();
    format!("{}/calendars/{}/acl", CALENDAR_BASE_URL, encoded_cal)
}

/// Resolve a calendar name to its ID.
pub fn resolve_calendar_id(
    name_or_id: &str,
    calendars: &[super::types::CalendarListEntry],
) -> Option<String> {
    // First check by ID
    if let Some(cal) = calendars.iter().find(|c| c.id == name_or_id) {
        return Some(cal.id.clone());
    }
    // Then check by summary (case-insensitive)
    if let Some(cal) = calendars.iter().find(|c| {
        c.summary
            .as_ref()
            .map(|s| s.eq_ignore_ascii_case(name_or_id))
            .unwrap_or(false)
    }) {
        return Some(cal.id.clone());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Requirement: REQ-CAL-001 (Must)
    // Acceptance: Calendars list URL
    #[test]
    fn req_cal_001_calendars_list_url() {
        let url = build_calendars_list_url(Some(100), None);
        assert!(url.contains("calendarList"));
    }

    // Requirement: REQ-CAL-002 (Must)
    // Acceptance: ACL list URL
    #[test]
    fn req_cal_002_acl_list_url() {
        let url = build_acl_list_url("primary");
        assert!(url.contains("calendars/primary/acl"));
    }

    // Requirement: REQ-CAL-003 (Must)
    // Acceptance: Calendar ID resolution by name
    #[test]
    fn req_cal_003_resolve_calendar_by_name() {
        let calendars = vec![super::super::types::CalendarListEntry {
            id: "abc@group.calendar.google.com".to_string(),
            summary: Some("Team Calendar".to_string()),
            description: None,
            time_zone: None,
            access_role: None,
            primary: None,
            background_color: None,
            foreground_color: None,
            selected: None,
            hidden: None,
            extra: HashMap::new(),
        }];
        let id = resolve_calendar_id("Team Calendar", &calendars).unwrap();
        assert_eq!(id, "abc@group.calendar.google.com");
    }

    // Requirement: REQ-CAL-003 (Must)
    // Acceptance: Calendar ID resolution by ID
    #[test]
    fn req_cal_003_resolve_calendar_by_id() {
        let calendars = vec![super::super::types::CalendarListEntry {
            id: "primary".to_string(),
            summary: Some("My Calendar".to_string()),
            description: None,
            time_zone: None,
            access_role: None,
            primary: None,
            background_color: None,
            foreground_color: None,
            selected: None,
            hidden: None,
            extra: HashMap::new(),
        }];
        let id = resolve_calendar_id("primary", &calendars).unwrap();
        assert_eq!(id, "primary");
    }

    // Requirement: REQ-CAL-003 (Must)
    // Edge case: Calendar not found
    #[test]
    fn req_cal_003_resolve_calendar_not_found() {
        let calendars: Vec<super::super::types::CalendarListEntry> = vec![];
        assert!(resolve_calendar_id("nonexistent", &calendars).is_none());
    }
}
