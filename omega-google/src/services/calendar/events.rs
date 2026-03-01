//! Calendar event list/get/create/update/delete.

use super::types::*;

/// Build URL for listing events from a calendar.
pub fn build_events_list_url(
    calendar_id: &str,
    time_min: Option<&str>,
    time_max: Option<&str>,
    max_results: Option<u32>,
    page_token: Option<&str>,
    query: Option<&str>,
) -> String {
    let base = format!("{}/calendars/{}/events", CALENDAR_BASE_URL, calendar_id);
    let mut params = Vec::new();
    if let Some(tmin) = time_min {
        params.push(format!("timeMin={}", tmin));
    }
    if let Some(tmax) = time_max {
        params.push(format!("timeMax={}", tmax));
    }
    if let Some(max) = max_results {
        params.push(format!("maxResults={}", max));
    }
    if let Some(token) = page_token {
        params.push(format!("pageToken={}", token));
    }
    if let Some(q) = query {
        params.push(format!("q={}", q));
    }
    params.push("singleEvents=true".to_string());
    params.push("orderBy=startTime".to_string());
    if params.is_empty() {
        base
    } else {
        format!("{}?{}", base, params.join("&"))
    }
}

/// Build URL for getting a single event.
pub fn build_event_get_url(calendar_id: &str, event_id: &str) -> String {
    format!(
        "{}/calendars/{}/events/{}",
        CALENDAR_BASE_URL, calendar_id, event_id
    )
}

/// Build the request body for creating an event.
pub fn build_event_create_body(
    summary: &str,
    start: &EventDateTime,
    end: &EventDateTime,
    description: Option<&str>,
    location: Option<&str>,
    attendees: &[String],
    event_type: Option<&str>,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "summary": summary,
    });

    // Build start
    let mut start_obj = serde_json::Map::new();
    if let Some(ref dt) = start.date_time {
        start_obj.insert("dateTime".to_string(), serde_json::json!(dt));
    }
    if let Some(ref d) = start.date {
        start_obj.insert("date".to_string(), serde_json::json!(d));
    }
    if let Some(ref tz) = start.time_zone {
        start_obj.insert("timeZone".to_string(), serde_json::json!(tz));
    }
    body["start"] = serde_json::Value::Object(start_obj);

    // Build end
    let mut end_obj = serde_json::Map::new();
    if let Some(ref dt) = end.date_time {
        end_obj.insert("dateTime".to_string(), serde_json::json!(dt));
    }
    if let Some(ref d) = end.date {
        end_obj.insert("date".to_string(), serde_json::json!(d));
    }
    if let Some(ref tz) = end.time_zone {
        end_obj.insert("timeZone".to_string(), serde_json::json!(tz));
    }
    body["end"] = serde_json::Value::Object(end_obj);

    if let Some(desc) = description {
        body["description"] = serde_json::json!(desc);
    }
    if let Some(loc) = location {
        body["location"] = serde_json::json!(loc);
    }
    if !attendees.is_empty() {
        let att_list: Vec<serde_json::Value> = attendees
            .iter()
            .map(|email| serde_json::json!({"email": email}))
            .collect();
        body["attendees"] = serde_json::json!(att_list);
    }
    if let Some(et) = event_type {
        body["eventType"] = serde_json::json!(et);
    }

    body
}

/// Build URL for creating an event.
pub fn build_event_create_url(calendar_id: &str) -> String {
    format!("{}/calendars/{}/events", CALENDAR_BASE_URL, calendar_id)
}

/// Build URL for updating an event.
pub fn build_event_update_url(calendar_id: &str, event_id: &str) -> String {
    format!(
        "{}/calendars/{}/events/{}",
        CALENDAR_BASE_URL, calendar_id, event_id
    )
}

/// Build URL for deleting an event.
pub fn build_event_delete_url(calendar_id: &str, event_id: &str) -> String {
    format!(
        "{}/calendars/{}/events/{}",
        CALENDAR_BASE_URL, calendar_id, event_id
    )
}

/// Detect scheduling conflicts in a list of events.
/// Returns pairs of (event1_id, event2_id) that overlap.
pub fn find_conflicts(events: &[Event]) -> Vec<(String, String)> {
    let mut conflicts = Vec::new();
    for i in 0..events.len() {
        for j in (i + 1)..events.len() {
            let e1 = &events[i];
            let e2 = &events[j];
            // Get start/end datetimes
            let e1_start = e1
                .start
                .as_ref()
                .and_then(|s| s.date_time.as_deref());
            let e1_end = e1
                .end
                .as_ref()
                .and_then(|e| e.date_time.as_deref());
            let e2_start = e2
                .start
                .as_ref()
                .and_then(|s| s.date_time.as_deref());
            let e2_end = e2
                .end
                .as_ref()
                .and_then(|e| e.date_time.as_deref());

            if let (Some(s1), Some(e1_e), Some(s2), Some(e2_e)) =
                (e1_start, e1_end, e2_start, e2_end)
            {
                // Two events overlap if one starts before the other ends
                if s1 < e2_e && s2 < e1_e {
                    let id1 = e1.id.as_deref().unwrap_or("").to_string();
                    let id2 = e2.id.as_deref().unwrap_or("").to_string();
                    conflicts.push((id1, id2));
                }
            }
        }
    }
    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ---------------------------------------------------------------
    // REQ-CAL-003 (Must): Event list URL building
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-003 (Must)
    // Acceptance: Events list URL with calendar ID
    #[test]
    fn req_cal_003_events_list_url() {
        let url = build_events_list_url("primary", None, None, None, None, None);
        assert!(url.contains("calendars/primary/events"));
    }

    // Requirement: REQ-CAL-003 (Must)
    // Acceptance: Events list URL with time range
    #[test]
    fn req_cal_003_events_list_url_time_range() {
        let url = build_events_list_url(
            "primary",
            Some("2024-01-15T00:00:00Z"),
            Some("2024-01-16T00:00:00Z"),
            None,
            None,
            None,
        );
        assert!(url.contains("timeMin="));
        assert!(url.contains("timeMax="));
    }

    // Requirement: REQ-CAL-003 (Must)
    // Acceptance: Events list URL with max results
    #[test]
    fn req_cal_003_events_list_url_max() {
        let url = build_events_list_url("primary", None, None, Some(50), None, None);
        assert!(url.contains("maxResults=50"));
    }

    // Requirement: REQ-CAL-003 (Must)
    // Acceptance: Events list URL with query
    #[test]
    fn req_cal_003_events_list_url_query() {
        let url = build_events_list_url("primary", None, None, None, None, Some("meeting"));
        assert!(url.contains("q=meeting"));
    }

    // Requirement: REQ-CAL-003 (Must)
    // Edge case: Calendar ID with special characters
    #[test]
    fn req_cal_003_events_list_url_special_cal_id() {
        let url = build_events_list_url("user@example.com", None, None, None, None, None);
        assert!(url.contains("calendars/"));
    }

    // ---------------------------------------------------------------
    // REQ-CAL-004 (Must): Event get URL
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-004 (Must)
    // Acceptance: Event get URL
    #[test]
    fn req_cal_004_event_get_url() {
        let url = build_event_get_url("primary", "event123");
        assert!(url.contains("calendars/primary/events/event123"));
    }

    // ---------------------------------------------------------------
    // REQ-CAL-005 (Must): Event creation
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-005 (Must)
    // Acceptance: Event create body has required fields
    #[test]
    fn req_cal_005_event_create_body() {
        let start = EventDateTime {
            date_time: Some("2024-01-15T10:00:00Z".to_string()),
            date: None,
            time_zone: None,
        };
        let end = EventDateTime {
            date_time: Some("2024-01-15T11:00:00Z".to_string()),
            date: None,
            time_zone: None,
        };
        let body = build_event_create_body(
            "Team Meeting",
            &start,
            &end,
            Some("Weekly standup"),
            Some("Room 42"),
            &["alice@example.com".to_string()],
            None,
        );
        assert_eq!(body["summary"], "Team Meeting");
        assert!(body["start"]["dateTime"].is_string());
        assert!(body["end"]["dateTime"].is_string());
    }

    // Requirement: REQ-CAL-005 (Must)
    // Acceptance: All-day event uses date field
    #[test]
    fn req_cal_005_all_day_event_create() {
        let start = EventDateTime {
            date_time: None,
            date: Some("2024-12-25".to_string()),
            time_zone: None,
        };
        let end = EventDateTime {
            date_time: None,
            date: Some("2024-12-26".to_string()),
            time_zone: None,
        };
        let body = build_event_create_body("Holiday", &start, &end, None, None, &[], None);
        assert!(body["start"]["date"].is_string());
        assert!(body["end"]["date"].is_string());
    }

    // Requirement: REQ-CAL-005 (Must)
    // Acceptance: Event with attendees
    #[test]
    fn req_cal_005_event_with_attendees() {
        let start = EventDateTime {
            date_time: Some("2024-01-15T10:00:00Z".to_string()),
            date: None,
            time_zone: None,
        };
        let end = EventDateTime {
            date_time: Some("2024-01-15T11:00:00Z".to_string()),
            date: None,
            time_zone: None,
        };
        let body = build_event_create_body(
            "Meeting",
            &start,
            &end,
            None,
            None,
            &["a@x.com".to_string(), "b@x.com".to_string()],
            None,
        );
        let attendees = body["attendees"].as_array().unwrap();
        assert_eq!(attendees.len(), 2);
    }

    // ---------------------------------------------------------------
    // REQ-CAL-006 (Must): Event update URL
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-006 (Must)
    // Acceptance: Event update URL
    #[test]
    fn req_cal_006_event_update_url() {
        let url = build_event_update_url("primary", "event123");
        assert!(url.contains("calendars/primary/events/event123"));
    }

    // ---------------------------------------------------------------
    // REQ-CAL-007 (Must): Event delete URL
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-007 (Must)
    // Acceptance: Event delete URL
    #[test]
    fn req_cal_007_event_delete_url() {
        let url = build_event_delete_url("primary", "event123");
        assert!(url.contains("calendars/primary/events/event123"));
    }

    // ---------------------------------------------------------------
    // REQ-CAL-015 (Must): Conflict detection
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-015 (Must)
    // Acceptance: Detects overlapping events
    #[test]
    fn req_cal_015_find_conflicts_overlapping() {
        let events = vec![
            Event {
                id: Some("e1".to_string()),
                summary: Some("Meeting 1".to_string()),
                start: Some(EventDateTime {
                    date_time: Some("2024-01-15T10:00:00Z".to_string()),
                    date: None,
                    time_zone: None,
                }),
                end: Some(EventDateTime {
                    date_time: Some("2024-01-15T11:00:00Z".to_string()),
                    date: None,
                    time_zone: None,
                }),
                description: None, location: None, status: None, html_link: None,
                created: None, updated: None, creator: None, organizer: None,
                attendees: vec![], recurrence: vec![], recurring_event_id: None,
                event_type: None, visibility: None, transparency: None,
                conference_data: None, hangout_link: None, color_id: None,
                extra: HashMap::new(),
            },
            Event {
                id: Some("e2".to_string()),
                summary: Some("Meeting 2".to_string()),
                start: Some(EventDateTime {
                    date_time: Some("2024-01-15T10:30:00Z".to_string()),
                    date: None,
                    time_zone: None,
                }),
                end: Some(EventDateTime {
                    date_time: Some("2024-01-15T11:30:00Z".to_string()),
                    date: None,
                    time_zone: None,
                }),
                description: None, location: None, status: None, html_link: None,
                created: None, updated: None, creator: None, organizer: None,
                attendees: vec![], recurrence: vec![], recurring_event_id: None,
                event_type: None, visibility: None, transparency: None,
                conference_data: None, hangout_link: None, color_id: None,
                extra: HashMap::new(),
            },
        ];
        let conflicts = find_conflicts(&events);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0], ("e1".to_string(), "e2".to_string()));
    }

    // Requirement: REQ-CAL-015 (Must)
    // Acceptance: No conflicts when events don't overlap
    #[test]
    fn req_cal_015_find_conflicts_no_overlap() {
        let events = vec![
            Event {
                id: Some("e1".to_string()),
                summary: Some("Meeting 1".to_string()),
                start: Some(EventDateTime {
                    date_time: Some("2024-01-15T10:00:00Z".to_string()),
                    date: None,
                    time_zone: None,
                }),
                end: Some(EventDateTime {
                    date_time: Some("2024-01-15T11:00:00Z".to_string()),
                    date: None,
                    time_zone: None,
                }),
                description: None, location: None, status: None, html_link: None,
                created: None, updated: None, creator: None, organizer: None,
                attendees: vec![], recurrence: vec![], recurring_event_id: None,
                event_type: None, visibility: None, transparency: None,
                conference_data: None, hangout_link: None, color_id: None,
                extra: HashMap::new(),
            },
            Event {
                id: Some("e2".to_string()),
                summary: Some("Meeting 2".to_string()),
                start: Some(EventDateTime {
                    date_time: Some("2024-01-15T14:00:00Z".to_string()),
                    date: None,
                    time_zone: None,
                }),
                end: Some(EventDateTime {
                    date_time: Some("2024-01-15T15:00:00Z".to_string()),
                    date: None,
                    time_zone: None,
                }),
                description: None, location: None, status: None, html_link: None,
                created: None, updated: None, creator: None, organizer: None,
                attendees: vec![], recurrence: vec![], recurring_event_id: None,
                event_type: None, visibility: None, transparency: None,
                conference_data: None, hangout_link: None, color_id: None,
                extra: HashMap::new(),
            },
        ];
        let conflicts = find_conflicts(&events);
        assert!(conflicts.is_empty());
    }

    // Requirement: REQ-CAL-015 (Must)
    // Edge case: Empty event list
    #[test]
    fn req_cal_015_find_conflicts_empty() {
        let conflicts = find_conflicts(&[]);
        assert!(conflicts.is_empty());
    }
}
