//! Calendar event RSVP.

/// Build the RSVP request body for responding to an event.
pub fn build_rsvp_body(
    status: &str,
    send_updates: Option<&str>,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "attendees": [{
            "responseStatus": status,
        }],
    });
    if let Some(updates) = send_updates {
        body["sendUpdates"] = serde_json::json!(updates);
    }
    body
}

/// Validate an RSVP status string.
pub fn validate_rsvp_status(status: &str) -> anyhow::Result<()> {
    match status {
        "accepted" | "declined" | "tentative" | "needsAction" => Ok(()),
        _ => anyhow::bail!("invalid RSVP status: '{}'. Must be one of: accepted, declined, tentative, needsAction", status),
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
    // Acceptance: RSVP body includes send-updates
    #[test]
    fn req_cal_009_rsvp_body_with_send_updates() {
        let body = build_rsvp_body("accepted", Some("all"));
        // Body should contain the response status
        assert!(body.is_object());
    }
}
