//! Calendar service integration tests.

use omega_google::services::calendar::types::*;
use std::collections::HashMap;

// ---------------------------------------------------------------
// REQ-CAL-003 (Must): Event deserialization from realistic API
// ---------------------------------------------------------------

// Requirement: REQ-CAL-003 (Must)
// Acceptance: Full event list response from API
#[test]
fn req_cal_003_integration_event_list_from_api() {
    let api_response = r#"{
        "items": [
            {
                "id": "event_morning",
                "summary": "Morning Standup",
                "start": {"dateTime": "2024-01-15T09:00:00-05:00", "timeZone": "America/New_York"},
                "end": {"dateTime": "2024-01-15T09:15:00-05:00", "timeZone": "America/New_York"},
                "status": "confirmed",
                "attendees": [
                    {"email": "alice@example.com", "responseStatus": "accepted", "self": true},
                    {"email": "bob@example.com", "responseStatus": "tentative"}
                ],
                "recurrence": ["RRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR"]
            },
            {
                "id": "event_lunch",
                "summary": "Team Lunch",
                "start": {"dateTime": "2024-01-15T12:00:00-05:00"},
                "end": {"dateTime": "2024-01-15T13:00:00-05:00"},
                "location": "Restaurant Downtown",
                "status": "confirmed"
            }
        ],
        "nextPageToken": "page_2_token",
        "summary": "primary",
        "timeZone": "America/New_York"
    }"#;
    let resp: EventListResponse = serde_json::from_str(api_response).unwrap();
    assert_eq!(resp.items.len(), 2);
    assert_eq!(resp.items[0].summary, Some("Morning Standup".to_string()));
    assert_eq!(resp.items[0].attendees.len(), 2);
    assert_eq!(resp.items[0].recurrence.len(), 1);
    assert_eq!(
        resp.items[1].location,
        Some("Restaurant Downtown".to_string())
    );
    assert_eq!(resp.time_zone, Some("America/New_York".to_string()));
}

// ---------------------------------------------------------------
// REQ-CAL-001 (Must): Calendar list from API
// ---------------------------------------------------------------

// Requirement: REQ-CAL-001 (Must)
// Acceptance: Calendar list with primary and secondary calendars
#[test]
fn req_cal_001_integration_calendar_list_from_api() {
    let api_response = "{
        \"items\": [
            {
                \"id\": \"alice@example.com\",
                \"summary\": \"Alice's Calendar\",
                \"timeZone\": \"America/New_York\",
                \"accessRole\": \"owner\",
                \"primary\": true,
                \"backgroundColor\": \"#4285f4\",
                \"foregroundColor\": \"#ffffff\"
            },
            {
                \"id\": \"team@group.calendar.google.com\",
                \"summary\": \"Team Calendar\",
                \"timeZone\": \"America/New_York\",
                \"accessRole\": \"writer\",
                \"backgroundColor\": \"#0b8043\",
                \"foregroundColor\": \"#ffffff\"
            },
            {
                \"id\": \"en.usa#holiday@group.v.calendar.google.com\",
                \"summary\": \"Holidays in United States\",
                \"accessRole\": \"reader\"
            }
        ]
    }";
    let resp: CalendarListResponse = serde_json::from_str(api_response).unwrap();
    assert_eq!(resp.items.len(), 3);
    assert_eq!(resp.items[0].primary, Some(true));
    assert_eq!(resp.items[1].access_role, Some("writer".to_string()));
    // Holiday calendar has no primary flag
    assert_eq!(resp.items[2].primary, None);
}

// ---------------------------------------------------------------
// REQ-CAL-005 (Must): Event creation body round-trip
// ---------------------------------------------------------------

// Requirement: REQ-CAL-005 (Must)
// Acceptance: Created event round-trips through JSON
#[test]
fn req_cal_005_integration_event_create_roundtrip() {
    let event = Event {
        id: None, // ID assigned by server
        summary: Some("New Meeting".to_string()),
        description: Some("Discuss Q1 plans".to_string()),
        location: Some("Conference Room A".to_string()),
        start: Some(EventDateTime {
            date_time: Some("2024-01-20T14:00:00-05:00".to_string()),
            date: None,
            time_zone: Some("America/New_York".to_string()),
        }),
        end: Some(EventDateTime {
            date_time: Some("2024-01-20T15:00:00-05:00".to_string()),
            date: None,
            time_zone: Some("America/New_York".to_string()),
        }),
        status: None,
        html_link: None,
        created: None,
        updated: None,
        creator: None,
        organizer: None,
        attendees: vec![Attendee {
            email: "bob@example.com".to_string(),
            display_name: Some("Bob".to_string()),
            response_status: None,
            organizer: None,
            is_self: None,
            optional: None,
            comment: None,
            extra: HashMap::new(),
        }],
        recurrence: vec![],
        recurring_event_id: None,
        event_type: None,
        visibility: None,
        transparency: None,
        conference_data: None,
        hangout_link: None,
        color_id: None,
        extra: HashMap::new(),
    };
    let json = serde_json::to_string(&event).unwrap();
    let parsed: Event = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.summary, Some("New Meeting".to_string()));
    assert_eq!(parsed.attendees.len(), 1);
    assert_eq!(parsed.attendees[0].email, "bob@example.com");
}

// ---------------------------------------------------------------
// REQ-CAL-008 (Must): Free/busy response with multiple calendars
// ---------------------------------------------------------------

// Requirement: REQ-CAL-008 (Must)
// Acceptance: Free/busy response with busy and free periods
#[test]
fn req_cal_008_integration_freebusy_response() {
    let api_response = r#"{
        "kind": "calendar#freeBusy",
        "timeMin": "2024-01-15T00:00:00Z",
        "timeMax": "2024-01-16T00:00:00Z",
        "calendars": {
            "alice@example.com": {
                "busy": [
                    {"start": "2024-01-15T09:00:00Z", "end": "2024-01-15T10:00:00Z"},
                    {"start": "2024-01-15T14:00:00Z", "end": "2024-01-15T15:00:00Z"}
                ],
                "errors": []
            },
            "bob@example.com": {
                "busy": [
                    {"start": "2024-01-15T10:00:00Z", "end": "2024-01-15T11:00:00Z"}
                ],
                "errors": []
            },
            "room-a@resource.calendar.google.com": {
                "busy": [],
                "errors": []
            }
        }
    }"#;
    let resp: FreeBusyResponse = serde_json::from_str(api_response).unwrap();
    let calendars = resp.calendars.unwrap();
    assert_eq!(calendars.len(), 3);
    assert_eq!(calendars["alice@example.com"].busy.len(), 2);
    assert_eq!(calendars["bob@example.com"].busy.len(), 1);
    assert_eq!(
        calendars["room-a@resource.calendar.google.com"].busy.len(),
        0
    );
}

// ---------------------------------------------------------------
// REQ-CAL-002 (Must): ACL list with various scope types
// ---------------------------------------------------------------

// Requirement: REQ-CAL-002 (Must)
// Acceptance: ACL list with user, group, and domain scopes
#[test]
fn req_cal_002_integration_acl_list() {
    let api_response = r#"{
        "items": [
            {
                "id": "user:alice@example.com",
                "role": "owner",
                "scope": {"type": "user", "value": "alice@example.com"}
            },
            {
                "id": "group:team@example.com",
                "role": "reader",
                "scope": {"type": "group", "value": "team@example.com"}
            },
            {
                "id": "domain:example.com",
                "role": "freeBusyReader",
                "scope": {"type": "domain", "value": "example.com"}
            }
        ]
    }"#;
    let resp: AclListResponse = serde_json::from_str(api_response).unwrap();
    assert_eq!(resp.items.len(), 3);
    assert_eq!(resp.items[0].scope.as_ref().unwrap().r#type, "user");
    assert_eq!(resp.items[1].scope.as_ref().unwrap().r#type, "group");
    assert_eq!(resp.items[2].role, Some("freeBusyReader".to_string()));
}

// ---------------------------------------------------------------
// REQ-CAL-021 (Should): Recurring event handling
// ---------------------------------------------------------------

// Requirement: REQ-CAL-021 (Should)
// Acceptance: Recurring event with exceptions
#[test]
fn req_cal_021_integration_recurring_event() {
    let api_response = r#"{
        "id": "recurring_weekly",
        "summary": "Weekly 1:1",
        "recurrence": [
            "RRULE:FREQ=WEEKLY;BYDAY=WE;COUNT=52",
            "EXDATE;TZID=America/New_York:20240320T100000",
            "EXDATE;TZID=America/New_York:20240703T100000"
        ],
        "start": {"dateTime": "2024-01-03T10:00:00-05:00", "timeZone": "America/New_York"},
        "end": {"dateTime": "2024-01-03T10:30:00-05:00", "timeZone": "America/New_York"}
    }"#;
    let event: Event = serde_json::from_str(api_response).unwrap();
    assert_eq!(event.recurrence.len(), 3);
    assert!(event.recurrence[0].starts_with("RRULE:"));
    assert!(event.recurrence[1].starts_with("EXDATE;"));
}

// ---------------------------------------------------------------
// REQ-CAL-009 (Must): Attendee response statuses
// ---------------------------------------------------------------

// Requirement: REQ-CAL-009 (Must)
// Acceptance: Various attendee response statuses
#[test]
fn req_cal_009_integration_attendee_statuses() {
    let api_response = r#"{
        "id": "event_with_attendees",
        "summary": "Team Review",
        "attendees": [
            {"email": "alice@example.com", "responseStatus": "accepted", "organizer": true},
            {"email": "bob@example.com", "responseStatus": "declined"},
            {"email": "carol@example.com", "responseStatus": "tentative"},
            {"email": "dave@example.com", "responseStatus": "needsAction"},
            {"email": "eve@example.com", "responseStatus": "accepted", "optional": true, "comment": "Joining remotely"}
        ]
    }"#;
    let event: Event = serde_json::from_str(api_response).unwrap();
    assert_eq!(event.attendees.len(), 5);
    assert_eq!(
        event.attendees[0].response_status,
        Some("accepted".to_string())
    );
    assert_eq!(event.attendees[0].organizer, Some(true));
    assert_eq!(
        event.attendees[1].response_status,
        Some("declined".to_string())
    );
    assert_eq!(event.attendees[4].optional, Some(true));
    assert_eq!(
        event.attendees[4].comment,
        Some("Joining remotely".to_string())
    );
}
