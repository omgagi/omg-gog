//! Contacts URL and body builders.
//! Uses the People API for contact operations.

use super::PEOPLE_API_BASE_URL;

/// Standard read mask for contact fields.
const PERSON_FIELDS: &str = "names,emailAddresses,phoneNumbers,birthdays,biographies,photos";

/// Build URL for searching contacts.
pub fn build_contacts_search_url(query: &str, max: Option<u32>) -> String {
    let max_val = max.unwrap_or(10);
    format!(
        "{}/people:searchContacts?query={}&readMask={}&pageSize={}",
        PEOPLE_API_BASE_URL,
        url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>(),
        PERSON_FIELDS,
        max_val
    )
}

/// Build URL for listing contacts.
pub fn build_contacts_list_url(max: Option<u32>, page_token: Option<&str>) -> String {
    let max_val = max.unwrap_or(100);
    let mut url = format!(
        "{}/people/me/connections?personFields={}&pageSize={}",
        PEOPLE_API_BASE_URL, PERSON_FIELDS, max_val
    );
    if let Some(token) = page_token {
        url.push_str(&format!(
            "&pageToken={}",
            url::form_urlencoded::byte_serialize(token.as_bytes()).collect::<String>()
        ));
    }
    url
}

/// Build URL for getting a single contact.
pub fn build_contact_get_url(resource_name: &str) -> String {
    format!(
        "{}/{}?personFields={}",
        PEOPLE_API_BASE_URL, resource_name, PERSON_FIELDS
    )
}

/// Build URL for creating a contact.
pub fn build_contact_create_url() -> String {
    format!("{}/people:createContact", PEOPLE_API_BASE_URL)
}

/// Build request body for creating a contact.
pub fn build_contact_create_body(
    given: Option<&str>,
    family: Option<&str>,
    email: Option<&str>,
    phone: Option<&str>,
) -> serde_json::Value {
    let mut body = serde_json::json!({});

    if given.is_some() || family.is_some() {
        let mut name = serde_json::json!({});
        if let Some(g) = given {
            name["givenName"] = serde_json::Value::String(g.to_string());
        }
        if let Some(f) = family {
            name["familyName"] = serde_json::Value::String(f.to_string());
        }
        body["names"] = serde_json::json!([name]);
    }

    if let Some(e) = email {
        body["emailAddresses"] = serde_json::json!([{"value": e}]);
    }

    if let Some(p) = phone {
        body["phoneNumbers"] = serde_json::json!([{"value": p}]);
    }

    body
}

/// Build URL for updating a contact.
pub fn build_contact_update_url(resource_name: &str) -> String {
    format!(
        "{}/{}:updateContact?updatePersonFields=names,emailAddresses,phoneNumbers,birthdays,biographies",
        PEOPLE_API_BASE_URL,
        resource_name
    )
}

/// Build request body for updating a contact.
///
/// Returns an error if the birthday string is present but cannot be parsed
/// as a valid `YYYY-MM-DD` date with sensible ranges.
pub fn build_contact_update_body(
    given: Option<&str>,
    family: Option<&str>,
    email: Option<&str>,
    phone: Option<&str>,
    birthday: Option<&str>,
    notes: Option<&str>,
) -> Result<serde_json::Value, String> {
    let mut body = serde_json::json!({});

    if given.is_some() || family.is_some() {
        let mut name = serde_json::json!({});
        if let Some(g) = given {
            name["givenName"] = serde_json::Value::String(g.to_string());
        }
        if let Some(f) = family {
            name["familyName"] = serde_json::Value::String(f.to_string());
        }
        body["names"] = serde_json::json!([name]);
    }

    if let Some(e) = email {
        body["emailAddresses"] = serde_json::json!([{"value": e}]);
    }

    if let Some(p) = phone {
        body["phoneNumbers"] = serde_json::json!([{"value": p}]);
    }

    if let Some(b) = birthday {
        // Parse "YYYY-MM-DD" format
        let parts: Vec<&str> = b.split('-').collect();
        if parts.len() != 3 {
            return Err(format!(
                "invalid birthday format (expected YYYY-MM-DD): {}",
                b
            ));
        }
        let year: i32 = parts[0]
            .parse()
            .map_err(|_| format!("invalid birthday year: {}", parts[0]))?;
        let month: i32 = parts[1]
            .parse()
            .map_err(|_| format!("invalid birthday month: {}", parts[1]))?;
        let day: i32 = parts[2]
            .parse()
            .map_err(|_| format!("invalid birthday day: {}", parts[2]))?;
        if year <= 0 {
            return Err(format!("invalid birthday year (must be > 0): {}", year));
        }
        if !(1..=12).contains(&month) {
            return Err(format!("invalid birthday month (must be 1-12): {}", month));
        }
        if !(1..=31).contains(&day) {
            return Err(format!("invalid birthday day (must be 1-31): {}", day));
        }
        body["birthdays"] = serde_json::json!([{
            "date": {
                "year": year,
                "month": month,
                "day": day
            }
        }]);
    }

    if let Some(n) = notes {
        body["biographies"] = serde_json::json!([{"value": n}]);
    }

    Ok(body)
}

/// Build URL for deleting a contact.
pub fn build_contact_delete_url(resource_name: &str) -> String {
    format!("{}/{}:deleteContact", PEOPLE_API_BASE_URL, resource_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CONTACTS-001 (Must): Contact search URL
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-001 (Must)
    // Acceptance: Search URL with query
    #[test]
    fn req_contacts_001_search_url() {
        let url = build_contacts_search_url("John", None);
        assert!(url.contains("searchContacts"));
        assert!(url.contains("query=John"));
        assert!(url.contains("readMask="));
        assert!(url.contains("pageSize=10"));
    }

    // Requirement: REQ-CONTACTS-001 (Must)
    // Acceptance: Search URL with max
    #[test]
    fn req_contacts_001_search_url_with_max() {
        let url = build_contacts_search_url("Smith", Some(25));
        assert!(url.contains("pageSize=25"));
    }

    // Requirement: REQ-CONTACTS-001 (Must)
    // Acceptance: Search URL encodes special characters
    #[test]
    fn req_contacts_001_search_url_encoded() {
        let url = build_contacts_search_url("John Doe", None);
        assert!(url.contains("John+Doe") || url.contains("John%20Doe"));
    }

    // ---------------------------------------------------------------
    // REQ-CONTACTS-002 (Must): Contact list URL
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-002 (Must)
    // Acceptance: List URL without page token
    #[test]
    fn req_contacts_002_list_url_no_page() {
        let url = build_contacts_list_url(None, None);
        assert!(url.contains("people/me/connections"));
        assert!(url.contains("personFields="));
        assert!(url.contains("pageSize=100"));
    }

    // Requirement: REQ-CONTACTS-002 (Must)
    // Acceptance: List URL with max and page token
    #[test]
    fn req_contacts_002_list_url_with_params() {
        let url = build_contacts_list_url(Some(50), Some("next123"));
        assert!(url.contains("pageSize=50"));
        assert!(url.contains("pageToken=next123"));
    }

    // ---------------------------------------------------------------
    // REQ-CONTACTS-003 (Must): Contact get URL
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-003 (Must)
    // Acceptance: Get URL with resource name
    #[test]
    fn req_contacts_003_get_url() {
        let url = build_contact_get_url("people/c12345");
        assert!(url.contains("people/c12345"));
        assert!(url.contains("personFields="));
    }

    // ---------------------------------------------------------------
    // REQ-CONTACTS-004 (Must): Contact create URL and body
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-004 (Must)
    // Acceptance: Create URL
    #[test]
    fn req_contacts_004_create_url() {
        let url = build_contact_create_url();
        assert!(url.contains("createContact"));
    }

    // Requirement: REQ-CONTACTS-004 (Must)
    // Acceptance: Create body with all fields
    #[test]
    fn req_contacts_004_create_body_full() {
        let body = build_contact_create_body(
            Some("John"),
            Some("Doe"),
            Some("john@example.com"),
            Some("+1234567890"),
        );
        assert_eq!(body["names"][0]["givenName"], "John");
        assert_eq!(body["names"][0]["familyName"], "Doe");
        assert_eq!(body["emailAddresses"][0]["value"], "john@example.com");
        assert_eq!(body["phoneNumbers"][0]["value"], "+1234567890");
    }

    // Requirement: REQ-CONTACTS-004 (Must)
    // Acceptance: Create body with only name
    #[test]
    fn req_contacts_004_create_body_name_only() {
        let body = build_contact_create_body(Some("Jane"), None, None, None);
        assert_eq!(body["names"][0]["givenName"], "Jane");
        assert!(body.get("emailAddresses").is_none());
        assert!(body.get("phoneNumbers").is_none());
    }

    // Requirement: REQ-CONTACTS-004 (Must)
    // Acceptance: Create body with no fields
    #[test]
    fn req_contacts_004_create_body_empty() {
        let body = build_contact_create_body(None, None, None, None);
        assert!(body.as_object().unwrap().is_empty());
    }

    // ---------------------------------------------------------------
    // REQ-CONTACTS-005 (Must): Contact update URL and body
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-005 (Must)
    // Acceptance: Update URL
    #[test]
    fn req_contacts_005_update_url() {
        let url = build_contact_update_url("people/c12345");
        assert!(url.contains("people/c12345"));
        assert!(url.contains("updateContact"));
        assert!(url.contains("updatePersonFields="));
    }

    // Requirement: REQ-CONTACTS-005 (Must)
    // Acceptance: Update body with all fields
    #[test]
    fn req_contacts_005_update_body_full() {
        let body = build_contact_update_body(
            Some("John"),
            Some("Smith"),
            Some("john.smith@example.com"),
            Some("+9876543210"),
            Some("1990-06-15"),
            Some("Updated notes"),
        )
        .unwrap();
        assert_eq!(body["names"][0]["givenName"], "John");
        assert_eq!(body["names"][0]["familyName"], "Smith");
        assert_eq!(body["emailAddresses"][0]["value"], "john.smith@example.com");
        assert_eq!(body["phoneNumbers"][0]["value"], "+9876543210");
        assert_eq!(body["birthdays"][0]["date"]["year"], 1990);
        assert_eq!(body["birthdays"][0]["date"]["month"], 6);
        assert_eq!(body["birthdays"][0]["date"]["day"], 15);
        assert_eq!(body["biographies"][0]["value"], "Updated notes");
    }

    // Requirement: REQ-CONTACTS-005 (Must)
    // Acceptance: Update body with partial fields
    #[test]
    fn req_contacts_005_update_body_partial() {
        let body = build_contact_update_body(None, None, Some("new@example.com"), None, None, None)
            .unwrap();
        assert!(body.get("names").is_none());
        assert_eq!(body["emailAddresses"][0]["value"], "new@example.com");
    }

    // ---------------------------------------------------------------
    // REQ-CONTACTS-006 (Must): Contact delete URL
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-006 (Must)
    // Acceptance: Delete URL
    #[test]
    fn req_contacts_006_delete_url() {
        let url = build_contact_delete_url("people/c12345");
        assert!(url.contains("people/c12345"));
        assert!(url.contains("deleteContact"));
    }
}
