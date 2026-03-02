//! People API request/response types.
//! All types use camelCase serde rename and flatten for forward compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------
// Person response types
// ---------------------------------------------------------------

/// A person response from the People API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonResponse {
    pub resource_name: Option<String>,
    pub etag: Option<String>,
    #[serde(default)]
    pub names: Vec<PersonName>,
    #[serde(default)]
    pub email_addresses: Vec<EmailAddress>,
    #[serde(default)]
    pub photos: Vec<Photo>,
    #[serde(default)]
    pub locales: Vec<Locale>,
    #[serde(default)]
    pub relations: Vec<Relation>,
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

/// A photo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Photo {
    pub url: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A locale.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Locale {
    pub value: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Search types
// ---------------------------------------------------------------

/// Search response from the People API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    #[serde(default)]
    pub results: Vec<SearchResult>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A single search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub person: Option<PersonResponse>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Relation types
// ---------------------------------------------------------------

/// A relation to another person.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relation {
    pub person: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub formatted_type: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-PEOPLE-001 (Must): PersonResponse type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-PEOPLE-001 (Must)
    // Acceptance: PersonResponse deserializes from People API JSON
    #[test]
    fn req_people_001_person_response_deserialize() {
        let json_str = r#"{
            "resourceName": "people/me",
            "etag": "etag_me",
            "names": [
                {"givenName": "Alice", "familyName": "Wonder", "displayName": "Alice Wonder"}
            ],
            "emailAddresses": [
                {"value": "alice@example.com", "type": "work", "formattedType": "Work"}
            ],
            "photos": [
                {"url": "https://lh3.googleusercontent.com/photo"}
            ],
            "locales": [
                {"value": "en-US"}
            ]
        }"#;
        let person: PersonResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(person.resource_name, Some("people/me".to_string()));
        assert_eq!(person.names.len(), 1);
        assert_eq!(person.names[0].given_name, Some("Alice".to_string()));
        assert_eq!(person.email_addresses.len(), 1);
        assert_eq!(person.photos.len(), 1);
        assert_eq!(person.locales.len(), 1);
        assert_eq!(person.locales[0].value, Some("en-US".to_string()));
    }

    // Requirement: REQ-PEOPLE-001 (Must)
    // Edge case: PersonResponse with empty lists
    #[test]
    fn req_people_001_person_response_empty() {
        let json_str = r#"{"resourceName": "people/1"}"#;
        let person: PersonResponse = serde_json::from_str(json_str).unwrap();
        assert!(person.names.is_empty());
        assert!(person.email_addresses.is_empty());
        assert!(person.photos.is_empty());
        assert!(person.locales.is_empty());
    }

    // Requirement: REQ-PEOPLE-001 (Must)
    // Edge case: Unknown fields preserved
    #[test]
    fn req_people_001_person_response_unknown_fields() {
        let json_str = r#"{"resourceName": "people/1", "futureField": "value"}"#;
        let person: PersonResponse = serde_json::from_str(json_str).unwrap();
        assert!(person.extra.contains_key("futureField"));
    }

    // ---------------------------------------------------------------
    // REQ-PEOPLE-002 (Must): SearchResponse type
    // ---------------------------------------------------------------

    // Requirement: REQ-PEOPLE-002 (Must)
    // Acceptance: SearchResponse deserializes
    #[test]
    fn req_people_002_search_response_deserialize() {
        let json_str = r#"{
            "results": [
                {
                    "person": {
                        "resourceName": "people/123",
                        "names": [{"displayName": "Bob"}]
                    }
                }
            ],
            "nextPageToken": "searchnext"
        }"#;
        let resp: SearchResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.results.len(), 1);
        let person = resp.results[0].person.as_ref().unwrap();
        assert_eq!(person.resource_name, Some("people/123".to_string()));
        assert_eq!(resp.next_page_token, Some("searchnext".to_string()));
    }

    // Requirement: REQ-PEOPLE-002 (Must)
    // Edge case: Empty search results
    #[test]
    fn req_people_002_search_response_empty() {
        let json_str = r#"{}"#;
        let resp: SearchResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.results.is_empty());
        assert!(resp.next_page_token.is_none());
    }

    // ---------------------------------------------------------------
    // REQ-PEOPLE-003 (Must): Locale type
    // ---------------------------------------------------------------

    // Requirement: REQ-PEOPLE-003 (Must)
    // Acceptance: Locale roundtrip
    #[test]
    fn req_people_003_locale_roundtrip() {
        let locale = Locale {
            value: Some("fr-FR".to_string()),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&locale).unwrap();
        let parsed: Locale = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.value, Some("fr-FR".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-PEOPLE-004 (Must): Relation type
    // ---------------------------------------------------------------

    // Requirement: REQ-PEOPLE-004 (Must)
    // Acceptance: Relation deserializes
    #[test]
    fn req_people_004_relation_deserialize() {
        let json_str = r#"{
            "person": "John Doe",
            "type": "spouse",
            "formattedType": "Spouse"
        }"#;
        let rel: Relation = serde_json::from_str(json_str).unwrap();
        assert_eq!(rel.person, Some("John Doe".to_string()));
        assert_eq!(rel.type_, Some("spouse".to_string()));
        assert_eq!(rel.formatted_type, Some("Spouse".to_string()));
    }

    // Requirement: REQ-PEOPLE-004 (Must)
    // Acceptance: Relation roundtrip
    #[test]
    fn req_people_004_relation_roundtrip() {
        let rel = Relation {
            person: Some("Jane".to_string()),
            type_: Some("friend".to_string()),
            formatted_type: Some("Friend".to_string()),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&rel).unwrap();
        let parsed: Relation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.person, Some("Jane".to_string()));
        assert_eq!(parsed.type_, Some("friend".to_string()));
    }
}
