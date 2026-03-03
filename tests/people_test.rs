//! People service integration tests.

use omega_google::services::people::people::*;
use omega_google::services::people::types::*;

// ---------------------------------------------------------------
// REQ-PEOPLE-001 (Must): PersonResponse deserialization
// ---------------------------------------------------------------

// Requirement: REQ-PEOPLE-001 (Must)
// Acceptance: Full PersonResponse from a realistic People API response
#[test]
fn req_people_001_integration_person_response_from_api() {
    // REQ-PEOPLE-001
    let api_response = r#"{
        "resourceName": "people/me",
        "etag": "%EgcBAj0JPjcuGgQBAgUHIgxYYWZ0TVZKcWdEQT0=",
        "names": [
            {
                "givenName": "Alice",
                "familyName": "Wonderland",
                "displayName": "Alice Wonderland",
                "displayNameLastFirst": "Wonderland, Alice",
                "unstructuredName": "Alice Wonderland"
            }
        ],
        "emailAddresses": [
            {
                "value": "alice@company.com",
                "type": "work",
                "formattedType": "Work"
            },
            {
                "value": "alice.wonder@gmail.com",
                "type": "home",
                "formattedType": "Home"
            }
        ],
        "photos": [
            {
                "url": "https://lh3.googleusercontent.com/a/alice_photo_abc123",
                "default": true
            }
        ],
        "locales": [
            {
                "value": "en-US"
            },
            {
                "value": "fr-FR"
            }
        ],
        "coverPhotos": [
            {
                "url": "https://lh3.googleusercontent.com/cover/alice_cover"
            }
        ],
        "ageRanges": [
            {
                "ageRange": "TWENTY_ONE_OR_OLDER"
            }
        ]
    }"#;

    let person: PersonResponse = serde_json::from_str(api_response).unwrap();

    // Verify resource name
    assert_eq!(person.resource_name, Some("people/me".to_string()));
    assert!(person.etag.is_some());

    // Verify names
    assert_eq!(person.names.len(), 1);
    assert_eq!(person.names[0].given_name, Some("Alice".to_string()));
    assert_eq!(person.names[0].family_name, Some("Wonderland".to_string()));
    assert_eq!(
        person.names[0].display_name,
        Some("Alice Wonderland".to_string())
    );
    // Unknown subfields preserved
    assert!(person.names[0].extra.contains_key("displayNameLastFirst"));

    // Verify email addresses
    assert_eq!(person.email_addresses.len(), 2);
    assert_eq!(
        person.email_addresses[0].value,
        Some("alice@company.com".to_string())
    );
    assert_eq!(person.email_addresses[0].type_, Some("work".to_string()));
    assert_eq!(
        person.email_addresses[0].formatted_type,
        Some("Work".to_string())
    );
    assert_eq!(
        person.email_addresses[1].value,
        Some("alice.wonder@gmail.com".to_string())
    );

    // Verify photos
    assert_eq!(person.photos.len(), 1);
    assert!(person.photos[0]
        .url
        .as_ref()
        .unwrap()
        .contains("googleusercontent.com"));
    assert!(person.photos[0].extra.contains_key("default"));

    // Verify locales
    assert_eq!(person.locales.len(), 2);
    assert_eq!(person.locales[0].value, Some("en-US".to_string()));
    assert_eq!(person.locales[1].value, Some("fr-FR".to_string()));

    // Unknown top-level fields preserved
    assert!(person.extra.contains_key("coverPhotos"));
    assert!(person.extra.contains_key("ageRanges"));
}

// ---------------------------------------------------------------
// REQ-PEOPLE-001 (Must): Minimal PersonResponse
// ---------------------------------------------------------------

// Requirement: REQ-PEOPLE-001 (Must)
// Acceptance: Minimal PersonResponse with only resource name
#[test]
fn req_people_001_integration_person_response_minimal() {
    // REQ-PEOPLE-001
    let api_response = r#"{
        "resourceName": "people/12345"
    }"#;

    let person: PersonResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(person.resource_name, Some("people/12345".to_string()));
    assert!(person.etag.is_none());
    assert!(person.names.is_empty());
    assert!(person.email_addresses.is_empty());
    assert!(person.photos.is_empty());
    assert!(person.locales.is_empty());
}

// ---------------------------------------------------------------
// REQ-PEOPLE-002 (Must): Search response deserialization
// ---------------------------------------------------------------

// Requirement: REQ-PEOPLE-002 (Must)
// Acceptance: Full SearchResponse from a realistic People API search
#[test]
fn req_people_002_integration_search_response_from_api() {
    // REQ-PEOPLE-002
    let api_response = r#"{
        "results": [
            {
                "person": {
                    "resourceName": "people/c001",
                    "etag": "search_etag_001",
                    "names": [
                        {
                            "givenName": "Alice",
                            "familyName": "Baker",
                            "displayName": "Alice Baker"
                        }
                    ],
                    "emailAddresses": [
                        {
                            "value": "alice.baker@company.com",
                            "type": "work",
                            "formattedType": "Work"
                        }
                    ],
                    "photos": [
                        {"url": "https://lh3.googleusercontent.com/photo/alice"}
                    ]
                }
            },
            {
                "person": {
                    "resourceName": "people/c002",
                    "etag": "search_etag_002",
                    "names": [
                        {
                            "givenName": "Alice",
                            "familyName": "Chen",
                            "displayName": "Alice Chen"
                        }
                    ],
                    "emailAddresses": [
                        {
                            "value": "alice.chen@company.com",
                            "type": "work"
                        }
                    ]
                }
            },
            {
                "person": {
                    "resourceName": "people/c003",
                    "names": [
                        {
                            "displayName": "Alice Donovan"
                        }
                    ]
                }
            }
        ],
        "nextPageToken": "search_page2_token"
    }"#;

    let resp: SearchResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(resp.results.len(), 3);
    assert_eq!(resp.next_page_token, Some("search_page2_token".to_string()));

    // First result: full person
    let r1 = &resp.results[0];
    let p1 = r1.person.as_ref().unwrap();
    assert_eq!(p1.resource_name, Some("people/c001".to_string()));
    assert_eq!(p1.names[0].given_name, Some("Alice".to_string()));
    assert_eq!(p1.names[0].family_name, Some("Baker".to_string()));
    assert_eq!(
        p1.email_addresses[0].value,
        Some("alice.baker@company.com".to_string())
    );
    assert_eq!(p1.photos.len(), 1);

    // Second result
    let r2 = &resp.results[1];
    let p2 = r2.person.as_ref().unwrap();
    assert_eq!(p2.names[0].display_name, Some("Alice Chen".to_string()));

    // Third result: minimal
    let r3 = &resp.results[2];
    let p3 = r3.person.as_ref().unwrap();
    assert_eq!(p3.names[0].display_name, Some("Alice Donovan".to_string()));
    assert!(p3.email_addresses.is_empty());
}

// ---------------------------------------------------------------
// REQ-PEOPLE-002 (Must): Empty search response
// ---------------------------------------------------------------

// Requirement: REQ-PEOPLE-002 (Must)
// Acceptance: Empty SearchResponse deserializes correctly
#[test]
fn req_people_002_integration_search_response_empty() {
    // REQ-PEOPLE-002
    let api_response = r#"{
        "results": []
    }"#;

    let resp: SearchResponse = serde_json::from_str(api_response).unwrap();
    assert!(resp.results.is_empty());
    assert!(resp.next_page_token.is_none());
}

// ---------------------------------------------------------------
// REQ-PEOPLE-001 (Must): URL builder verification - people me
// ---------------------------------------------------------------

// Requirement: REQ-PEOPLE-001 (Must)
// Acceptance: People me URL includes person fields
#[test]
fn req_people_001_integration_url_builder_me() {
    // REQ-PEOPLE-001
    let url = build_people_me_url();
    assert!(url.contains("people/me"));
    assert!(url.contains("personFields="));
    assert!(url.contains("names"));
    assert!(url.contains("emailAddresses"));
}

// ---------------------------------------------------------------
// REQ-PEOPLE-002 (Must): URL builder verification - people get
// ---------------------------------------------------------------

// Requirement: REQ-PEOPLE-002 (Must)
// Acceptance: People get URL includes resource name and person fields
#[test]
fn req_people_002_integration_url_builder_get() {
    // REQ-PEOPLE-002
    let url = build_people_get_url("people/c001");
    assert!(url.contains("people/c001"));
    assert!(url.contains("personFields="));

    // Directory person
    let url = build_people_get_url("people/d12345");
    assert!(url.contains("people/d12345"));
}

// ---------------------------------------------------------------
// REQ-PEOPLE-003 (Must): URL builder verification - people search
// ---------------------------------------------------------------

// Requirement: REQ-PEOPLE-003 (Must)
// Acceptance: People search URL includes query and read mask
#[test]
fn req_people_003_integration_url_builder_search() {
    // REQ-PEOPLE-003
    // Basic search
    let url = build_people_search_url("Alice", None, None);
    assert!(url.contains("searchContacts"));
    assert!(url.contains("query=Alice"));
    assert!(url.contains("readMask="));
    assert!(url.contains("pageSize=10"));

    // With max
    let url = build_people_search_url("Bob", Some(25), None);
    assert!(url.contains("pageSize=25"));

    // With page token
    let url = build_people_search_url("Carol", Some(10), Some("page2_token"));
    assert!(url.contains("pageSize=10"));
    assert!(url.contains("pageToken=page2_token"));

    // Special characters encoded
    let url = build_people_search_url("John Doe", None, None);
    assert!(url.contains("John+Doe") || url.contains("John%20Doe"));
}

// ---------------------------------------------------------------
// REQ-PEOPLE-004 (Must): URL builder verification - relations
// ---------------------------------------------------------------

// Requirement: REQ-PEOPLE-004 (Must)
// Acceptance: People relations URL defaults to "me" or uses specific resource
#[test]
fn req_people_004_integration_url_builder_relations() {
    // REQ-PEOPLE-004
    // Default: "me"
    let url = build_people_relations_url(None);
    assert!(url.contains("people/me/connections"));
    assert!(url.contains("personFields="));
    assert!(url.contains("relations"));

    // Specific resource
    let url = build_people_relations_url(Some("people/12345"));
    assert!(url.contains("people/12345/connections"));
    assert!(url.contains("personFields="));
}

// ---------------------------------------------------------------
// REQ-PEOPLE-004 (Must): Relation type verification
// ---------------------------------------------------------------

// Requirement: REQ-PEOPLE-004 (Must)
// Acceptance: Relation type deserializes and serializes correctly
#[test]
fn req_people_004_integration_relation_type() {
    // REQ-PEOPLE-004
    let json_str = r#"{
        "person": "Jane Smith",
        "type": "spouse",
        "formattedType": "Spouse"
    }"#;

    let rel: Relation = serde_json::from_str(json_str).unwrap();
    assert_eq!(rel.person, Some("Jane Smith".to_string()));
    assert_eq!(rel.type_, Some("spouse".to_string()));
    assert_eq!(rel.formatted_type, Some("Spouse".to_string()));

    // Round-trip
    let serialized = serde_json::to_value(&rel).unwrap();
    assert_eq!(serialized["type"], "spouse");
    assert_eq!(serialized["person"], "Jane Smith");
}
