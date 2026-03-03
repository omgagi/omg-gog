//! Contacts API request/response types.
//! Contacts uses the People API internally.
//! All types use camelCase serde rename and flatten for forward compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------
// Person types
// ---------------------------------------------------------------

/// A person (contact) from the People API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub resource_name: Option<String>,
    pub etag: Option<String>,
    #[serde(default)]
    pub names: Vec<PersonName>,
    #[serde(default)]
    pub email_addresses: Vec<EmailAddress>,
    #[serde(default)]
    pub phone_numbers: Vec<PhoneNumber>,
    #[serde(default)]
    pub birthdays: Vec<Birthday>,
    #[serde(default)]
    pub biographies: Vec<Biography>,
    #[serde(default)]
    pub photos: Vec<Photo>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A person's name.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonName {
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub display_name: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// An email address.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailAddress {
    pub value: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub formatted_type: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A phone number.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneNumber {
    pub value: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub formatted_type: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A birthday.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Birthday {
    pub date: Option<DateValue>,
    pub text: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A date value with year/month/day.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateValue {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

/// A biography/note.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Biography {
    pub value: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A photo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Photo {
    pub url: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// List response types
// ---------------------------------------------------------------

/// Person list response (connections).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonListResponse {
    #[serde(default)]
    pub connections: Vec<Person>,
    pub next_page_token: Option<String>,
    pub total_people: Option<i32>,
    pub total_items: Option<i32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Directory list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryListResponse {
    #[serde(default)]
    pub people: Vec<Person>,
    pub next_page_token: Option<String>,
    pub next_sync_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CONTACTS-001 (Must): Person type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-001 (Must)
    // Acceptance: Person deserializes from People API JSON
    #[test]
    fn req_contacts_001_person_deserialize() {
        let json_str = r#"{
            "resourceName": "people/c12345",
            "etag": "etag123",
            "names": [
                {"givenName": "John", "familyName": "Doe", "displayName": "John Doe"}
            ],
            "emailAddresses": [
                {"value": "john@example.com", "type": "home", "formattedType": "Home"}
            ],
            "phoneNumbers": [
                {"value": "+1234567890", "type": "mobile", "formattedType": "Mobile"}
            ],
            "birthdays": [
                {"date": {"year": 1990, "month": 6, "day": 15}}
            ],
            "biographies": [
                {"value": "A contact note"}
            ],
            "photos": [
                {"url": "https://lh3.googleusercontent.com/photo"}
            ]
        }"#;
        let person: Person = serde_json::from_str(json_str).unwrap();
        assert_eq!(person.resource_name, Some("people/c12345".to_string()));
        assert_eq!(person.names.len(), 1);
        assert_eq!(person.names[0].given_name, Some("John".to_string()));
        assert_eq!(person.email_addresses.len(), 1);
        assert_eq!(
            person.email_addresses[0].value,
            Some("john@example.com".to_string())
        );
        assert_eq!(person.phone_numbers.len(), 1);
        assert_eq!(person.birthdays.len(), 1);
        let date = person.birthdays[0].date.as_ref().unwrap();
        assert_eq!(date.year, Some(1990));
        assert_eq!(date.month, Some(6));
        assert_eq!(date.day, Some(15));
    }

    // Requirement: REQ-CONTACTS-001 (Must)
    // Acceptance: Person with empty lists deserializes
    #[test]
    fn req_contacts_001_person_empty_lists() {
        let json_str = r#"{
            "resourceName": "people/c99999",
            "etag": "etag999"
        }"#;
        let person: Person = serde_json::from_str(json_str).unwrap();
        assert_eq!(person.resource_name, Some("people/c99999".to_string()));
        assert!(person.names.is_empty());
        assert!(person.email_addresses.is_empty());
        assert!(person.phone_numbers.is_empty());
        assert!(person.birthdays.is_empty());
    }

    // Requirement: REQ-CONTACTS-001 (Must)
    // Edge case: Person with unknown fields preserved
    #[test]
    fn req_contacts_001_person_unknown_fields() {
        let json_str = r#"{
            "resourceName": "people/c1",
            "customField": "preserved"
        }"#;
        let person: Person = serde_json::from_str(json_str).unwrap();
        assert!(person.extra.contains_key("customField"));
    }

    // ---------------------------------------------------------------
    // REQ-CONTACTS-002 (Must): PersonListResponse type
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-002 (Must)
    // Acceptance: PersonListResponse deserializes with pagination
    #[test]
    fn req_contacts_002_person_list_response_deserialize() {
        let json_str = r#"{
            "connections": [
                {"resourceName": "people/c1", "names": [{"displayName": "Alice"}]},
                {"resourceName": "people/c2", "names": [{"displayName": "Bob"}]}
            ],
            "nextPageToken": "token123",
            "totalPeople": 100,
            "totalItems": 2
        }"#;
        let resp: PersonListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.connections.len(), 2);
        assert_eq!(resp.next_page_token, Some("token123".to_string()));
        assert_eq!(resp.total_people, Some(100));
        assert_eq!(resp.total_items, Some(2));
    }

    // Requirement: REQ-CONTACTS-002 (Must)
    // Edge case: Empty connections list
    #[test]
    fn req_contacts_002_person_list_response_empty() {
        let json_str = r#"{}"#;
        let resp: PersonListResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.connections.is_empty());
        assert!(resp.next_page_token.is_none());
    }

    // ---------------------------------------------------------------
    // REQ-CONTACTS-007 (Must): DirectoryListResponse type
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-007 (Must)
    // Acceptance: DirectoryListResponse deserializes
    #[test]
    fn req_contacts_007_directory_list_response_deserialize() {
        let json_str = r#"{
            "people": [
                {"resourceName": "people/d1", "names": [{"displayName": "Carol"}]}
            ],
            "nextPageToken": "dtoken",
            "nextSyncToken": "sync123"
        }"#;
        let resp: DirectoryListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.people.len(), 1);
        assert_eq!(resp.next_page_token, Some("dtoken".to_string()));
        assert_eq!(resp.next_sync_token, Some("sync123".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-CONTACTS-003 (Must): EmailAddress type
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-003 (Must)
    // Acceptance: EmailAddress roundtrip
    #[test]
    fn req_contacts_003_email_address_roundtrip() {
        let email = EmailAddress {
            value: Some("test@example.com".to_string()),
            type_: Some("work".to_string()),
            formatted_type: Some("Work".to_string()),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&email).unwrap();
        let parsed: EmailAddress = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.value, Some("test@example.com".to_string()));
        assert_eq!(parsed.type_, Some("work".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-CONTACTS-004 (Must): PhoneNumber type
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-004 (Must)
    // Acceptance: PhoneNumber roundtrip
    #[test]
    fn req_contacts_004_phone_number_roundtrip() {
        let phone = PhoneNumber {
            value: Some("+1234567890".to_string()),
            type_: Some("mobile".to_string()),
            formatted_type: Some("Mobile".to_string()),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&phone).unwrap();
        let parsed: PhoneNumber = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.value, Some("+1234567890".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-CONTACTS-005 (Must): Birthday and DateValue types
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-005 (Must)
    // Acceptance: Birthday with DateValue roundtrip
    #[test]
    fn req_contacts_005_birthday_roundtrip() {
        let bday = Birthday {
            date: Some(DateValue {
                year: Some(1985),
                month: Some(12),
                day: Some(25),
            }),
            text: Some("December 25, 1985".to_string()),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&bday).unwrap();
        let parsed: Birthday = serde_json::from_str(&json).unwrap();
        let date = parsed.date.unwrap();
        assert_eq!(date.year, Some(1985));
        assert_eq!(date.month, Some(12));
        assert_eq!(date.day, Some(25));
    }

    // ---------------------------------------------------------------
    // REQ-CONTACTS-006 (Must): PersonName type
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-006 (Must)
    // Acceptance: PersonName roundtrip
    #[test]
    fn req_contacts_006_person_name_roundtrip() {
        let name = PersonName {
            given_name: Some("Jane".to_string()),
            family_name: Some("Smith".to_string()),
            display_name: Some("Jane Smith".to_string()),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&name).unwrap();
        let parsed: PersonName = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.given_name, Some("Jane".to_string()));
        assert_eq!(parsed.display_name, Some("Jane Smith".to_string()));
    }
}
