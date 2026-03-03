//! Calendar event RSVP.

/// Result of building an RSVP request: the JSON body plus the optional sendUpdates query parameter.
pub struct RsvpRequest {
    pub body: serde_json::Value,
    pub send_updates: Option<String>,
}

/// Build the RSVP request body for responding to an event.
/// Returns an `RsvpRequest` containing the JSON body (attendee status only)
/// and the `sendUpdates` value as a separate query parameter, since the
/// Google Calendar API expects `sendUpdates` as a URL query parameter,
/// not in the request body.
pub fn build_rsvp_body(status: &str, send_updates: Option<&str>) -> RsvpRequest {
    let body = serde_json::json!({
        "attendees": [{
            "responseStatus": status,
        }],
    });
    RsvpRequest {
        body,
        send_updates: send_updates.map(|s| s.to_string()),
    }
}

/// Build the RSVP URL with optional sendUpdates query parameter.
pub fn build_rsvp_url(calendar_id: &str, event_id: &str, send_updates: Option<&str>) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let encoded_cal = utf8_percent_encode(calendar_id, NON_ALPHANUMERIC).to_string();
    let encoded_event = utf8_percent_encode(event_id, NON_ALPHANUMERIC).to_string();
    let base = format!(
        "https://www.googleapis.com/calendar/v3/calendars/{}/events/{}",
        encoded_cal, encoded_event
    );
    match send_updates {
        Some(updates) => format!("{}?sendUpdates={}", base, updates),
        None => base,
    }
}

/// Validate an RSVP status string.
pub fn validate_rsvp_status(status: &str) -> anyhow::Result<()> {
    match status {
        "accepted" | "declined" | "tentative" | "needsAction" => Ok(()),
        _ => anyhow::bail!(
            "invalid RSVP status: '{}'. Must be one of: accepted, declined, tentative, needsAction",
            status
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Requirement: REQ-CAL-009 (Must)
    // Acceptance: Valid RSVP statuses accepted
    #[test]
    fn req_cal_009_valid_rsvp_statuses() {
        assert!(validate_rsvp_status("accepted").is_ok());
        assert!(validate_rsvp_status("declined").is_ok());
        assert!(validate_rsvp_status("tentative").is_ok());
    }

    // Requirement: REQ-CAL-009 (Must)
    // Acceptance: Invalid RSVP status rejected
    #[test]
    fn req_cal_009_invalid_rsvp_status() {
        assert!(validate_rsvp_status("maybe").is_err());
        assert!(validate_rsvp_status("").is_err());
    }

    // Requirement: REQ-CAL-009 (Must)
    // Acceptance: RSVP body does NOT include sendUpdates (it's a query param)
    #[test]
    fn req_cal_009_rsvp_body_with_send_updates() {
        let rsvp = build_rsvp_body("accepted", Some("all"));
        // Body should contain the response status but NOT sendUpdates
        assert!(rsvp.body.is_object());
        assert!(rsvp.body.get("sendUpdates").is_none());
        assert_eq!(rsvp.send_updates, Some("all".to_string()));
    }

    // Requirement: REQ-CAL-009 (Must)
    // Acceptance: RSVP URL includes sendUpdates as query parameter
    #[test]
    fn req_cal_009_rsvp_url_with_send_updates() {
        let url = build_rsvp_url("primary", "event123", Some("all"));
        assert!(url.contains("sendUpdates=all"));
        assert!(url.contains("calendars/"));
        assert!(url.contains("events/"));
    }

    // Requirement: REQ-CAL-009 (Must)
    // Acceptance: RSVP URL without sendUpdates has no query string
    #[test]
    fn req_cal_009_rsvp_url_without_send_updates() {
        let url = build_rsvp_url("primary", "event123", None);
        assert!(!url.contains("sendUpdates"));
    }
}
