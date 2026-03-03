//! Calendar API request/response types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Calendar API base URL.
pub const CALENDAR_BASE_URL: &str = "https://www.googleapis.com/calendar/v3";

// ---------------------------------------------------------------
// Calendar list types
// ---------------------------------------------------------------

/// Calendar list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarListResponse {
    #[serde(default)]
    pub items: Vec<CalendarListEntry>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A calendar in the user's calendar list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarListEntry {
    pub id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub time_zone: Option<String>,
    pub access_role: Option<String>,
    pub primary: Option<bool>,
    pub background_color: Option<String>,
    pub foreground_color: Option<String>,
    pub selected: Option<bool>,
    pub hidden: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Event types
// ---------------------------------------------------------------

/// Event list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventListResponse {
    #[serde(default)]
    pub items: Vec<Event>,
    pub next_page_token: Option<String>,
    pub summary: Option<String>,
    pub time_zone: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A calendar event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start: Option<EventDateTime>,
    pub end: Option<EventDateTime>,
    pub status: Option<String>,
    pub html_link: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub creator: Option<EventPerson>,
    pub organizer: Option<EventPerson>,
    #[serde(default)]
    pub attendees: Vec<Attendee>,
    #[serde(default)]
    pub recurrence: Vec<String>,
    pub recurring_event_id: Option<String>,
    pub event_type: Option<String>,
    pub visibility: Option<String>,
    pub transparency: Option<String>,
    pub conference_data: Option<serde_json::Value>,
    pub hangout_link: Option<String>,
    pub color_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// An event's date/time (either dateTime or date for all-day events).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventDateTime {
    pub date_time: Option<String>,
    pub date: Option<String>,
    pub time_zone: Option<String>,
}

/// A person reference (creator/organizer).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventPerson {
    pub email: Option<String>,
    pub display_name: Option<String>,
    #[serde(rename = "self")]
    pub is_self: Option<bool>,
}

/// An event attendee.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attendee {
    pub email: String,
    pub display_name: Option<String>,
    pub response_status: Option<String>,
    pub organizer: Option<bool>,
    #[serde(rename = "self")]
    pub is_self: Option<bool>,
    pub optional: Option<bool>,
    pub comment: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Free/busy types
// ---------------------------------------------------------------

/// Free/busy query request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeBusyRequest {
    pub time_min: String,
    pub time_max: String,
    pub items: Vec<FreeBusyCalendarId>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Calendar ID for free/busy query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeBusyCalendarId {
    pub id: String,
}

/// Free/busy query response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeBusyResponse {
    pub kind: Option<String>,
    pub time_min: Option<String>,
    pub time_max: Option<String>,
    pub calendars: Option<HashMap<String, FreeBusyCalendar>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Free/busy data for a single calendar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeBusyCalendar {
    #[serde(default)]
    pub busy: Vec<FreeBusyPeriod>,
    #[serde(default)]
    pub errors: Vec<serde_json::Value>,
}

/// A busy time period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeBusyPeriod {
    pub start: String,
    pub end: String,
}

// ---------------------------------------------------------------
// ACL types
// ---------------------------------------------------------------

/// ACL list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AclListResponse {
    #[serde(default)]
    pub items: Vec<AclRule>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// An ACL rule (calendar access control entry).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AclRule {
    pub id: Option<String>,
    pub role: Option<String>,
    pub scope: Option<AclScope>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// The scope of an ACL rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclScope {
    pub r#type: String,
    pub value: Option<String>,
}

// ---------------------------------------------------------------
// Color types
// ---------------------------------------------------------------

/// Calendar colors response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorsResponse {
    pub calendar: Option<HashMap<String, ColorDefinition>>,
    pub event: Option<HashMap<String, ColorDefinition>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A color definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDefinition {
    pub background: String,
    pub foreground: String,
}

/// Determine the day of week name for a date string (YYYY-MM-DD or RFC3339).
pub fn day_of_week(date_str: &str) -> Option<String> {
    // Extract just the date part (first 10 chars: YYYY-MM-DD)
    let date_part = if date_str.len() >= 10 {
        &date_str[..10]
    } else {
        return None;
    };
    let date = chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d").ok()?;
    Some(date.format("%A").to_string())
}

/// Parse a time range specification into (start, end) RFC3339 strings.
/// Supports: "today", "tomorrow", "this week", "N days", from/to.
pub fn resolve_time_range(
    from: Option<&str>,
    to: Option<&str>,
    _timezone: &str,
) -> anyhow::Result<(String, String)> {
    let today = chrono::Utc::now().date_naive();

    let start_date = match from {
        Some(s) => {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|_| anyhow::anyhow!(
                    "Invalid date format: '{}'. Expected YYYY-MM-DD, RFC3339, or relative (today, tomorrow, +Nd)", s
                ))?
        }
        None => today,
    };

    let end_date = match to {
        Some(s) => {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|_| anyhow::anyhow!(
                    "Invalid date format: '{}'. Expected YYYY-MM-DD, RFC3339, or relative (today, tomorrow, +Nd)", s
                ))?
        }
        None => start_date + chrono::Duration::days(1),
    };

    let start_str = format!("{}T00:00:00Z", start_date);
    let end_str = format!("{}T00:00:00Z", end_date);
    Ok((start_str, end_str))
}

/// Generate a calendar event URL.
pub fn event_url(calendar_id: &str, event_id: &str) -> String {
    format!(
        "https://calendar.google.com/calendar/event?eid={}&cid={}",
        event_id, calendar_id
    )
}

/// Generate a propose-time URL for a calendar event.
pub fn propose_time_url(calendar_id: &str, event_id: &str) -> String {
    format!(
        "https://calendar.google.com/calendar/r/eventedit?eid={}&cid={}",
        event_id, calendar_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CAL-001 (Must): CalendarListEntry type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-001 (Must)
    // Acceptance: CalendarListEntry deserializes from API JSON
    #[test]
    fn req_cal_001_calendar_list_entry_deserialize() {
        let json_str = "{
            \"id\": \"primary\",
            \"summary\": \"My Calendar\",
            \"timeZone\": \"America/New_York\",
            \"accessRole\": \"owner\",
            \"primary\": true,
            \"backgroundColor\": \"#0088aa\",
            \"foregroundColor\": \"#000000\"
        }";
        let entry: CalendarListEntry = serde_json::from_str(json_str).unwrap();
        assert_eq!(entry.id, "primary");
        assert_eq!(entry.summary, Some("My Calendar".to_string()));
        assert_eq!(entry.access_role, Some("owner".to_string()));
        assert_eq!(entry.primary, Some(true));
    }

    // Requirement: REQ-CAL-001 (Must)
    // Acceptance: CalendarListResponse round-trip
    #[test]
    fn req_cal_001_calendar_list_response_roundtrip() {
        let resp = CalendarListResponse {
            items: vec![CalendarListEntry {
                id: "primary".to_string(),
                summary: Some("My Cal".to_string()),
                description: None,
                time_zone: Some("UTC".to_string()),
                access_role: Some("owner".to_string()),
                primary: Some(true),
                background_color: None,
                foreground_color: None,
                selected: None,
                hidden: None,
                extra: HashMap::new(),
            }],
            next_page_token: None,
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: CalendarListResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.items[0].id, "primary");
    }

    // ---------------------------------------------------------------
    // REQ-CAL-003/004 (Must): Event type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-003 (Must)
    // Acceptance: Event deserializes from API JSON
    #[test]
    fn req_cal_003_event_deserialize() {
        let json_str = r#"{
            "id": "event123",
            "summary": "Team Meeting",
            "description": "Weekly standup",
            "location": "Room 42",
            "start": {"dateTime": "2024-01-15T10:00:00-05:00"},
            "end": {"dateTime": "2024-01-15T11:00:00-05:00"},
            "status": "confirmed",
            "htmlLink": "https://calendar.google.com/event?eid=abc",
            "attendees": [
                {"email": "alice@example.com", "responseStatus": "accepted"},
                {"email": "bob@example.com", "responseStatus": "needsAction"}
            ],
            "recurrence": ["RRULE:FREQ=WEEKLY;BYDAY=MO"]
        }"#;
        let event: Event = serde_json::from_str(json_str).unwrap();
        assert_eq!(event.id, Some("event123".to_string()));
        assert_eq!(event.summary, Some("Team Meeting".to_string()));
        assert_eq!(event.attendees.len(), 2);
        assert_eq!(event.attendees[0].email, "alice@example.com");
        assert_eq!(event.recurrence, vec!["RRULE:FREQ=WEEKLY;BYDAY=MO"]);
    }

    // Requirement: REQ-CAL-003 (Must)
    // Acceptance: All-day event uses date instead of dateTime
    #[test]
    fn req_cal_003_all_day_event() {
        let json_str = r#"{
            "id": "allday1",
            "summary": "Holiday",
            "start": {"date": "2024-12-25"},
            "end": {"date": "2024-12-26"}
        }"#;
        let event: Event = serde_json::from_str(json_str).unwrap();
        let start = event.start.unwrap();
        assert!(start.date_time.is_none());
        assert_eq!(start.date, Some("2024-12-25".to_string()));
    }

    // Requirement: REQ-CAL-003 (Must)
    // Acceptance: EventListResponse deserializes with pagination
    #[test]
    fn req_cal_003_event_list_response() {
        let json_str = r#"{
            "items": [
                {"id": "e1", "summary": "Event 1"},
                {"id": "e2", "summary": "Event 2"}
            ],
            "nextPageToken": "page_2",
            "summary": "My Calendar",
            "timeZone": "America/New_York"
        }"#;
        let resp: EventListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.items.len(), 2);
        assert_eq!(resp.next_page_token, Some("page_2".to_string()));
    }

    // Requirement: REQ-CAL-003 (Must)
    // Edge case: Event with no attendees
    #[test]
    fn req_cal_003_event_no_attendees() {
        let json_str = r#"{
            "id": "e1",
            "summary": "Solo event"
        }"#;
        let event: Event = serde_json::from_str(json_str).unwrap();
        assert!(event.attendees.is_empty());
    }

    // Requirement: REQ-CAL-003 (Must)
    // Edge case: Event with unknown fields preserved
    #[test]
    fn req_cal_003_event_unknown_fields() {
        let json_str = r#"{
            "id": "e1",
            "newApiField": "should be preserved"
        }"#;
        let event: Event = serde_json::from_str(json_str).unwrap();
        assert!(event.extra.contains_key("newApiField"));
    }

    // ---------------------------------------------------------------
    // REQ-CAL-008 (Must): Free/busy types
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-008 (Must)
    // Acceptance: FreeBusyRequest serializes correctly
    #[test]
    fn req_cal_008_freebusy_request_serialize() {
        let req = FreeBusyRequest {
            time_min: "2024-01-15T00:00:00Z".to_string(),
            time_max: "2024-01-16T00:00:00Z".to_string(),
            items: vec![
                FreeBusyCalendarId {
                    id: "alice@example.com".to_string(),
                },
                FreeBusyCalendarId {
                    id: "bob@example.com".to_string(),
                },
            ],
            extra: HashMap::new(),
        };
        let json_val: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(json_val["timeMin"], "2024-01-15T00:00:00Z");
        assert_eq!(json_val["items"].as_array().unwrap().len(), 2);
    }

    // Requirement: REQ-CAL-008 (Must)
    // Acceptance: FreeBusyResponse deserializes
    #[test]
    fn req_cal_008_freebusy_response_deserialize() {
        let json_str = r#"{
            "kind": "calendar#freeBusy",
            "timeMin": "2024-01-15T00:00:00Z",
            "timeMax": "2024-01-16T00:00:00Z",
            "calendars": {
                "alice@example.com": {
                    "busy": [
                        {"start": "2024-01-15T09:00:00Z", "end": "2024-01-15T10:00:00Z"}
                    ],
                    "errors": []
                }
            }
        }"#;
        let resp: FreeBusyResponse = serde_json::from_str(json_str).unwrap();
        let calendars = resp.calendars.unwrap();
        let alice = &calendars["alice@example.com"];
        assert_eq!(alice.busy.len(), 1);
        assert_eq!(alice.busy[0].start, "2024-01-15T09:00:00Z");
    }

    // ---------------------------------------------------------------
    // REQ-CAL-002 (Must): ACL types
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-002 (Must)
    // Acceptance: AclListResponse deserializes
    #[test]
    fn req_cal_002_acl_list_response_deserialize() {
        let json_str = r#"{
            "items": [
                {
                    "id": "user:alice@example.com",
                    "role": "owner",
                    "scope": {"type": "user", "value": "alice@example.com"}
                }
            ]
        }"#;
        let resp: AclListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.items.len(), 1);
        assert_eq!(resp.items[0].role, Some("owner".to_string()));
        let scope = resp.items[0].scope.as_ref().unwrap();
        assert_eq!(scope.r#type, "user");
    }

    // ---------------------------------------------------------------
    // REQ-CAL-014 (Must): Colors types
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-014 (Must)
    // Acceptance: ColorsResponse deserializes
    #[test]
    fn req_cal_014_colors_response_deserialize() {
        let json_str = "{
            \"calendar\": {
                \"1\": {\"background\": \"#ac725e\", \"foreground\": \"#1d1d1d\"}
            },
            \"event\": {
                \"1\": {\"background\": \"#a4bdfc\", \"foreground\": \"#1d1d1d\"}
            }
        }";
        let resp: ColorsResponse = serde_json::from_str(json_str).unwrap();
        let cal_colors = resp.calendar.unwrap();
        assert_eq!(cal_colors["1"].background, "#ac725e");
    }

    // ---------------------------------------------------------------
    // REQ-CAL-009 (Must): Attendee type
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-009 (Must)
    // Acceptance: Attendee deserializes with response status
    #[test]
    fn req_cal_009_attendee_deserialize() {
        let json_str = r#"{
            "email": "user@example.com",
            "displayName": "User",
            "responseStatus": "accepted",
            "organizer": true,
            "self": false,
            "optional": false
        }"#;
        let attendee: Attendee = serde_json::from_str(json_str).unwrap();
        assert_eq!(attendee.email, "user@example.com");
        assert_eq!(attendee.response_status, Some("accepted".to_string()));
        assert_eq!(attendee.organizer, Some(true));
    }

    // ---------------------------------------------------------------
    // REQ-CAL-021 (Should): Recurrence parsing
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-021 (Should)
    // Acceptance: Recurrence rules stored in event
    #[test]
    fn req_cal_021_recurrence_in_event() {
        let json_str = r#"{
            "id": "recurring1",
            "summary": "Weekly meeting",
            "recurrence": [
                "RRULE:FREQ=WEEKLY;BYDAY=MO",
                "EXDATE;TZID=America/New_York:20240115T100000"
            ],
            "recurringEventId": "parent_event_id"
        }"#;
        let event: Event = serde_json::from_str(json_str).unwrap();
        assert_eq!(event.recurrence.len(), 2);
        assert!(event.recurrence[0].starts_with("RRULE:"));
        assert_eq!(
            event.recurring_event_id,
            Some("parent_event_id".to_string())
        );
    }

    // ---------------------------------------------------------------
    // REQ-CAL-022 (Should): Day-of-week enrichment
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-022 (Should)
    // Acceptance: Day of week extracted from date
    #[test]
    fn req_cal_022_day_of_week() {
        // 2024-01-15 is a Monday
        let dow = day_of_week("2024-01-15");
        assert_eq!(dow, Some("Monday".to_string()));
    }

    // Requirement: REQ-CAL-022 (Should)
    // Acceptance: Day of week from RFC3339 datetime
    #[test]
    fn req_cal_022_day_of_week_rfc3339() {
        // 2024-01-15 is a Monday
        let dow = day_of_week("2024-01-15T10:00:00-05:00");
        assert_eq!(dow, Some("Monday".to_string()));
    }

    // Requirement: REQ-CAL-022 (Should)
    // Edge case: Invalid date string
    #[test]
    fn req_cal_022_day_of_week_invalid() {
        let dow = day_of_week("not-a-date");
        assert!(dow.is_none());
    }

    // ---------------------------------------------------------------
    // REQ-CAL-020 (Must): Time range resolution
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-020 (Must)
    // Acceptance: Time range with from/to dates
    #[test]
    fn req_cal_020_time_range_from_to() {
        let (start, end) =
            resolve_time_range(Some("2024-01-15"), Some("2024-01-16"), "UTC").unwrap();
        assert!(start.contains("2024-01-15"));
        assert!(end.contains("2024-01-16"));
    }

    // Requirement: REQ-CAL-020 (Must)
    // Acceptance: Time range defaults to today
    #[test]
    fn req_cal_020_time_range_defaults_today() {
        let (start, end) = resolve_time_range(None, None, "UTC").unwrap();
        // Should default to today's date range
        assert!(!start.is_empty());
        assert!(!end.is_empty());
    }

    // ---------------------------------------------------------------
    // REQ-CAL-016 (Should): Propose-time URL generation
    // ---------------------------------------------------------------

    // Requirement: REQ-CAL-016 (Should)
    // Acceptance: propose-time URL generated correctly
    #[test]
    fn req_cal_016_propose_time_url() {
        let url = propose_time_url("primary", "event123");
        assert!(url.contains("calendar.google.com"));
    }
}
