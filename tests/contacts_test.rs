//! Contacts service integration tests.

use omega_google::services::contacts::contacts::*;
use omega_google::services::contacts::directory::*;
use omega_google::services::contacts::types::*;

// ---------------------------------------------------------------
// REQ-CONTACTS-001 (Must): Person deserialization with names, emails, phones
// ---------------------------------------------------------------

// Requirement: REQ-CONTACTS-001 (Must)
// Acceptance: Full Person from a realistic People API response
#[test]
fn req_contacts_001_integration_person_full_from_api() {
    // REQ-CONTACTS-001
    let api_response = r#"{
        "resourceName": "people/c8472659103",
        "etag": "%EgcBAj0JPjcuGgQBAgUHIgxCWkdoUHllNmhRMD0=",
        "names": [
            {
                "givenName": "Sarah",
                "familyName": "O'Connor",
                "displayName": "Sarah O'Connor",
                "displayNameLastFirst": "O'Connor, Sarah",
                "unstructuredName": "Sarah O'Connor"
            }
        ],
        "emailAddresses": [
            {
                "value": "sarah.oconnor@company.com",
                "type": "work",
                "formattedType": "Work"
            },
            {
                "value": "sarah.personal@gmail.com",
                "type": "home",
                "formattedType": "Home"
            }
        ],
        "phoneNumbers": [
            {
                "value": "+1 (555) 123-4567",
                "type": "mobile",
                "formattedType": "Mobile"
            },
            {
                "value": "+1 (555) 987-6543",
                "type": "work",
                "formattedType": "Work"
            }
        ],
        "birthdays": [
            {
                "date": {
                    "year": 1985,
                    "month": 7,
                    "day": 22
                },
                "text": "July 22, 1985"
            }
        ],
        "biographies": [
            {
                "value": "VP of Engineering at TechCorp. Previously at Google and Amazon."
            }
        ],
        "photos": [
            {
                "url": "https://lh3.googleusercontent.com/contacts/sarah_photo_abc123"
            }
        ],
        "organizations": [
            {
                "name": "TechCorp",
                "title": "VP of Engineering"
            }
        ]
    }"#;

    let person: Person = serde_json::from_str(api_response).unwrap();

    // Verify resource name and etag
    assert_eq!(person.resource_name, Some("people/c8472659103".to_string()));
    assert!(person.etag.is_some());

    // Verify names
    assert_eq!(person.names.len(), 1);
    assert_eq!(person.names[0].given_name, Some("Sarah".to_string()));
    assert_eq!(person.names[0].family_name, Some("O'Connor".to_string()));
    assert_eq!(
        person.names[0].display_name,
        Some("Sarah O'Connor".to_string())
    );
    // Unknown subfields preserved in name
    assert!(person.names[0].extra.contains_key("displayNameLastFirst"));

    // Verify email addresses
    assert_eq!(person.email_addresses.len(), 2);
    assert_eq!(
        person.email_addresses[0].value,
        Some("sarah.oconnor@company.com".to_string())
    );
    assert_eq!(person.email_addresses[0].type_, Some("work".to_string()));
    assert_eq!(
        person.email_addresses[0].formatted_type,
        Some("Work".to_string())
    );
    assert_eq!(
        person.email_addresses[1].value,
        Some("sarah.personal@gmail.com".to_string())
    );
    assert_eq!(person.email_addresses[1].type_, Some("home".to_string()));

    // Verify phone numbers
    assert_eq!(person.phone_numbers.len(), 2);
    assert_eq!(
        person.phone_numbers[0].value,
        Some("+1 (555) 123-4567".to_string())
    );
    assert_eq!(person.phone_numbers[0].type_, Some("mobile".to_string()));
    assert_eq!(
        person.phone_numbers[1].value,
        Some("+1 (555) 987-6543".to_string())
    );
    assert_eq!(person.phone_numbers[1].type_, Some("work".to_string()));

    // Verify birthday
    assert_eq!(person.birthdays.len(), 1);
    let bday = &person.birthdays[0];
    assert_eq!(bday.text, Some("July 22, 1985".to_string()));
    let date = bday.date.as_ref().unwrap();
    assert_eq!(date.year, Some(1985));
    assert_eq!(date.month, Some(7));
    assert_eq!(date.day, Some(22));

    // Verify biography
    assert_eq!(person.biographies.len(), 1);
    assert!(person.biographies[0]
        .value
        .as_ref()
        .unwrap()
        .contains("VP of Engineering"));

    // Verify photo
    assert_eq!(person.photos.len(), 1);
    assert!(person.photos[0]
        .url
        .as_ref()
        .unwrap()
        .contains("googleusercontent.com"));

    // Unknown top-level fields preserved
    assert!(person.extra.contains_key("organizations"));
}

// ---------------------------------------------------------------
// REQ-CONTACTS-002 (Must): PersonListResponse deserialization
// ---------------------------------------------------------------

// Requirement: REQ-CONTACTS-002 (Must)
// Acceptance: Full PersonListResponse from a realistic API response
#[test]
fn req_contacts_002_integration_person_list_from_api() {
    // REQ-CONTACTS-002
    let api_response = r#"{
        "connections": [
            {
                "resourceName": "people/c001",
                "etag": "etag001",
                "names": [
                    {"givenName": "Alice", "familyName": "Baker", "displayName": "Alice Baker"}
                ],
                "emailAddresses": [
                    {"value": "alice@example.com", "type": "work"}
                ],
                "phoneNumbers": [
                    {"value": "+1-555-0101", "type": "mobile"}
                ]
            },
            {
                "resourceName": "people/c002",
                "etag": "etag002",
                "names": [
                    {"givenName": "Bob", "familyName": "Carter", "displayName": "Bob Carter"}
                ],
                "emailAddresses": [
                    {"value": "bob@example.com", "type": "work"},
                    {"value": "bob.carter@gmail.com", "type": "home"}
                ]
            },
            {
                "resourceName": "people/c003",
                "etag": "etag003",
                "names": [
                    {"displayName": "Carol Davis"}
                ],
                "phoneNumbers": [
                    {"value": "+44-20-7946-0958", "type": "work"}
                ]
            }
        ],
        "nextPageToken": "contacts_page2_token",
        "totalPeople": 250,
        "totalItems": 3
    }"#;

    let resp: PersonListResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(resp.connections.len(), 3);
    assert_eq!(
        resp.next_page_token,
        Some("contacts_page2_token".to_string())
    );
    assert_eq!(resp.total_people, Some(250));
    assert_eq!(resp.total_items, Some(3));

    // First contact
    let c1 = &resp.connections[0];
    assert_eq!(c1.resource_name, Some("people/c001".to_string()));
    assert_eq!(c1.names[0].given_name, Some("Alice".to_string()));
    assert_eq!(c1.email_addresses.len(), 1);
    assert_eq!(c1.phone_numbers.len(), 1);

    // Second contact: multiple emails
    let c2 = &resp.connections[1];
    assert_eq!(c2.email_addresses.len(), 2);
    assert_eq!(
        c2.email_addresses[0].value,
        Some("bob@example.com".to_string())
    );
    assert_eq!(
        c2.email_addresses[1].value,
        Some("bob.carter@gmail.com".to_string())
    );

    // Third contact: display name only, no email
    let c3 = &resp.connections[2];
    assert_eq!(c3.names[0].display_name, Some("Carol Davis".to_string()));
    assert!(c3.email_addresses.is_empty());
    assert_eq!(c3.phone_numbers.len(), 1);
}

// ---------------------------------------------------------------
// REQ-CONTACTS-001 (Must): Person with minimal data
// ---------------------------------------------------------------

// Requirement: REQ-CONTACTS-001 (Must)
// Acceptance: Person with only resource name deserializes correctly
#[test]
fn req_contacts_001_integration_person_minimal() {
    // REQ-CONTACTS-001
    let api_response = r#"{
        "resourceName": "people/c999",
        "etag": "minimal_etag"
    }"#;

    let person: Person = serde_json::from_str(api_response).unwrap();

    assert_eq!(person.resource_name, Some("people/c999".to_string()));
    assert!(person.names.is_empty());
    assert!(person.email_addresses.is_empty());
    assert!(person.phone_numbers.is_empty());
    assert!(person.birthdays.is_empty());
    assert!(person.biographies.is_empty());
    assert!(person.photos.is_empty());
}

// ---------------------------------------------------------------
// REQ-CONTACTS-002 (Must): URL builder verification - contacts
// ---------------------------------------------------------------

// Requirement: REQ-CONTACTS-002 (Must)
// Acceptance: Contacts URL builders produce correct People API URLs
#[test]
fn req_contacts_002_integration_url_builders() {
    // REQ-CONTACTS-002
    // List contacts URL
    let url = build_contacts_list_url(None, None);
    assert!(url.contains("people/me/connections"));
    assert!(url.contains("personFields="));
    assert!(url.contains("pageSize=100"));

    // List with custom max and page token
    let url = build_contacts_list_url(Some(50), Some("page2_token"));
    assert!(url.contains("pageSize=50"));
    assert!(url.contains("pageToken=page2_token"));

    // Search URL
    let url = build_contacts_search_url("Sarah", None);
    assert!(url.contains("searchContacts"));
    assert!(url.contains("query=Sarah"));
    assert!(url.contains("pageSize=10"));

    // Search with max
    let url = build_contacts_search_url("Bob", Some(25));
    assert!(url.contains("pageSize=25"));

    // Get contact URL
    let url = build_contact_get_url("people/c8472659103");
    assert!(url.contains("people/c8472659103"));
    assert!(url.contains("personFields="));

    // Create URL
    let url = build_contact_create_url();
    assert!(url.contains("createContact"));

    // Delete URL
    let url = build_contact_delete_url("people/c8472659103");
    assert!(url.contains("people/c8472659103"));
    assert!(url.contains("deleteContact"));
}

// ---------------------------------------------------------------
// REQ-CONTACTS-004 (Must): Contact create body verification
// ---------------------------------------------------------------

// Requirement: REQ-CONTACTS-004 (Must)
// Acceptance: Contact create body constructs correct JSON
#[test]
fn req_contacts_004_integration_create_body() {
    // REQ-CONTACTS-004
    // Full body
    let body = build_contact_create_body(
        Some("Jane"),
        Some("Doe"),
        Some("jane.doe@example.com"),
        Some("+1-555-0199"),
    );
    assert_eq!(body["names"][0]["givenName"], "Jane");
    assert_eq!(body["names"][0]["familyName"], "Doe");
    assert_eq!(body["emailAddresses"][0]["value"], "jane.doe@example.com");
    assert_eq!(body["phoneNumbers"][0]["value"], "+1-555-0199");

    // Minimal body (name only)
    let body = build_contact_create_body(Some("John"), None, None, None);
    assert_eq!(body["names"][0]["givenName"], "John");
    assert!(body.get("emailAddresses").is_none());
    assert!(body.get("phoneNumbers").is_none());

    // Empty body
    let body = build_contact_create_body(None, None, None, None);
    assert!(body.as_object().unwrap().is_empty());
}

// ---------------------------------------------------------------
// REQ-CONTACTS-007 (Must): Directory URL builder verification
// ---------------------------------------------------------------

// Requirement: REQ-CONTACTS-007 (Must)
// Acceptance: Directory URL builders produce correct URLs
#[test]
fn req_contacts_007_integration_directory_url_builders() {
    // REQ-CONTACTS-007
    // Directory list
    let url = build_directory_list_url(None, None);
    assert!(url.contains("listDirectoryPeople"));
    assert!(url.contains("pageSize=100"));
    assert!(url.contains("DIRECTORY_SOURCE_TYPE_DOMAIN_PROFILE"));

    // Directory list with params
    let url = build_directory_list_url(Some(50), Some("dir_next"));
    assert!(url.contains("pageSize=50"));
    assert!(url.contains("pageToken=dir_next"));

    // Directory search
    let url = build_directory_search_url("Alice", None);
    assert!(url.contains("searchDirectoryPeople"));
    assert!(url.contains("query=Alice"));
    assert!(url.contains("pageSize=10"));

    // Directory search with max
    let url = build_directory_search_url("Bob", Some(30));
    assert!(url.contains("pageSize=30"));
}

// ---------------------------------------------------------------
// REQ-CONTACTS-007 (Must): DirectoryListResponse deserialization
// ---------------------------------------------------------------

// Requirement: REQ-CONTACTS-007 (Must)
// Acceptance: DirectoryListResponse from a realistic API response
#[test]
fn req_contacts_007_integration_directory_list_from_api() {
    // REQ-CONTACTS-007
    let api_response = r#"{
        "people": [
            {
                "resourceName": "people/d001",
                "names": [
                    {"givenName": "Eve", "familyName": "Foster", "displayName": "Eve Foster"}
                ],
                "emailAddresses": [
                    {"value": "eve.foster@company.com", "type": "work"}
                ],
                "phoneNumbers": [
                    {"value": "+1-555-0201", "type": "work"}
                ]
            },
            {
                "resourceName": "people/d002",
                "names": [
                    {"displayName": "Frank Garcia"}
                ],
                "emailAddresses": [
                    {"value": "frank.garcia@company.com", "type": "work"}
                ]
            }
        ],
        "nextPageToken": "dir_page2",
        "nextSyncToken": "sync_token_abc123"
    }"#;

    let resp: DirectoryListResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(resp.people.len(), 2);
    assert_eq!(resp.next_page_token, Some("dir_page2".to_string()));
    assert_eq!(resp.next_sync_token, Some("sync_token_abc123".to_string()));

    assert_eq!(resp.people[0].names[0].given_name, Some("Eve".to_string()));
    assert_eq!(
        resp.people[1].names[0].display_name,
        Some("Frank Garcia".to_string())
    );
}

// ---------------------------------------------------------------
// REQ-CONTACTS-005 (Must): Contact update body verification
// ---------------------------------------------------------------

// Requirement: REQ-CONTACTS-005 (Must)
// Acceptance: Contact update body with birthday and notes
#[test]
fn req_contacts_005_integration_update_body() {
    // REQ-CONTACTS-005
    let body = build_contact_update_body(
        Some("Sarah"),
        Some("Connor"),
        Some("sarah.connor@newcompany.com"),
        Some("+1-555-0300"),
        Some("1985-07-22"),
        Some("Updated VP of Engineering title"),
    )
    .unwrap();

    assert_eq!(body["names"][0]["givenName"], "Sarah");
    assert_eq!(body["names"][0]["familyName"], "Connor");
    assert_eq!(
        body["emailAddresses"][0]["value"],
        "sarah.connor@newcompany.com"
    );
    assert_eq!(body["phoneNumbers"][0]["value"], "+1-555-0300");
    assert_eq!(body["birthdays"][0]["date"]["year"], 1985);
    assert_eq!(body["birthdays"][0]["date"]["month"], 7);
    assert_eq!(body["birthdays"][0]["date"]["day"], 22);
    assert_eq!(
        body["biographies"][0]["value"],
        "Updated VP of Engineering title"
    );

    // Update URL verification
    let url = build_contact_update_url("people/c8472659103");
    assert!(url.contains("people/c8472659103"));
    assert!(url.contains("updateContact"));
    assert!(url.contains("updatePersonFields="));
}
