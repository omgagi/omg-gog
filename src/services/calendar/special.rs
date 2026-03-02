//! Focus time, OOO, and working location event creation.

use super::types::*;

/// Build an event body for a Focus Time block.
pub fn build_focus_time_event(
    summary: &str,
    start: &EventDateTime,
    end: &EventDateTime,
) -> serde_json::Value {
    let mut body = super::events::build_event_create_body(
        summary, start, end, None, None, &[], Some("focusTime"),
    );
    body["transparency"] = serde_json::json!("opaque");
    body
}

/// Build an event body for an Out of Office event.
pub fn build_ooo_event(
    summary: &str,
    start: &EventDateTime,
    end: &EventDateTime,
) -> serde_json::Value {
    super::events::build_event_create_body(
        summary, start, end, None, None, &[], Some("outOfOffice"),
    )
}

/// Build an event body for a Working Location event.
pub fn build_working_location_event(
    location_type: &str,
    label: Option<&str>,
    start: &EventDateTime,
    end: &EventDateTime,
) -> serde_json::Value {
    let mut body = super::events::build_event_create_body(
        label.unwrap_or("Working Location"),
        start,
        end,
        None,
        None,
        &[],
        Some("workingLocation"),
    );
    body["workingLocationProperties"] = serde_json::json!({
        "type": location_type,
    });
    body
}

/// Validate a working location type.
pub fn validate_location_type(location_type: &str) -> anyhow::Result<()> {
    match location_type {
        "home" | "office" | "custom" => Ok(()),
        _ => anyhow::bail!(
            "invalid location type: '{}'. Must be one of: home, office, custom",
            location_type
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CAL-017 (Should): Focus Time events
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-017 (Should)
    // Acceptance: Focus time event has correct eventType
    #[test]
    fn req_cal_017_focus_time_event_type() {
        let start = EventDateTime {
            date_time: Some("2024-01-15T14:00:00Z".to_string()),
            date: None,
            time_zone: None,
        };
        let end = EventDateTime {
            date_time: Some("2024-01-15T16:00:00Z".to_string()),
            date: None,
            time_zone: None,
        };
        let body = build_focus_time_event("Focus Time", &start, &end);
        assert_eq!(body["eventType"], "focusTime");
    }

    // ---------------------------------------------------------------
    // REQ-CAL-018 (Should): OOO events
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-018 (Should)
    // Acceptance: OOO event has correct eventType
    #[test]
    fn req_cal_018_ooo_event_type() {
        let start = EventDateTime {
            date_time: None,
            date: Some("2024-01-15".to_string()),
            time_zone: None,
        };
        let end = EventDateTime {
            date_time: None,
            date: Some("2024-01-20".to_string()),
            time_zone: None,
        };
        let body = build_ooo_event("Out of Office", &start, &end);
        assert_eq!(body["eventType"], "outOfOffice");
    }

    // ---------------------------------------------------------------
    // REQ-CAL-019 (Should): Working location events
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-019 (Should)
    // Acceptance: Working location supports home/office/custom
    #[test]
    fn req_cal_019_working_location_home() {
        let start = EventDateTime {
            date_time: None,
            date: Some("2024-01-15".to_string()),
            time_zone: None,
        };
        let end = EventDateTime {
            date_time: None,
            date: Some("2024-01-16".to_string()),
            time_zone: None,
        };
        let body = build_working_location_event("home", None, &start, &end);
        assert_eq!(body["eventType"], "workingLocation");
    }

    // Requirement: REQ-CAL-019 (Should)
    // Acceptance: Valid location types
    #[test]
    fn req_cal_019_validate_location_types() {
        assert!(validate_location_type("home").is_ok());
        assert!(validate_location_type("office").is_ok());
        assert!(validate_location_type("custom").is_ok());
        assert!(validate_location_type("invalid").is_err());
    }
}
