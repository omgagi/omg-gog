//! Calendar free/busy queries.

use super::types::*;

/// Build the free/busy query request body.
pub fn build_freebusy_request(
    calendar_ids: &[String],
    time_min: &str,
    time_max: &str,
) -> FreeBusyRequest {
    FreeBusyRequest {
        time_min: time_min.to_string(),
        time_max: time_max.to_string(),
        items: calendar_ids
            .iter()
            .map(|id| FreeBusyCalendarId { id: id.clone() })
            .collect(),
        extra: std::collections::HashMap::new(),
    }
}

/// Build URL for the free/busy endpoint.
pub fn build_freebusy_url() -> String {
    format!("{}/freeBusy", CALENDAR_BASE_URL)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Requirement: REQ-CAL-008 (Must)
    // Acceptance: Free/busy request builder
    #[test]
    fn req_cal_008_freebusy_request_builder() {
        let req = build_freebusy_request(
            &[
                "alice@example.com".to_string(),
                "bob@example.com".to_string(),
            ],
            "2024-01-15T00:00:00Z",
            "2024-01-16T00:00:00Z",
        );
        assert_eq!(req.items.len(), 2);
        assert_eq!(req.time_min, "2024-01-15T00:00:00Z");
    }

    // Requirement: REQ-CAL-008 (Must)
    // Acceptance: Free/busy URL
    #[test]
    fn req_cal_008_freebusy_url() {
        let url = build_freebusy_url();
        assert!(url.contains("freeBusy"));
    }
}
