//! People API URL builders.

use super::PEOPLE_BASE_URL;

/// Standard read mask for people profile fields.
const PERSON_FIELDS: &str = "names,emailAddresses,photos,locales";

/// Build URL for getting the authenticated user's profile.
pub fn build_people_me_url() -> String {
    format!(
        "{}/people/me?personFields={}",
        PEOPLE_BASE_URL, PERSON_FIELDS
    )
}

/// Build URL for getting a person by resource name.
pub fn build_people_get_url(resource_name: &str) -> String {
    format!(
        "{}/{}?personFields={}",
        PEOPLE_BASE_URL, resource_name, PERSON_FIELDS
    )
}

/// Build URL for searching people.
pub fn build_people_search_url(query: &str, max: Option<u32>, page_token: Option<&str>) -> String {
    let max_val = max.unwrap_or(10);
    let mut url = format!(
        "{}/people:searchContacts?query={}&readMask={}&pageSize={}",
        PEOPLE_BASE_URL,
        url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>(),
        PERSON_FIELDS,
        max_val
    );
    if let Some(token) = page_token {
        url.push_str(&format!(
            "&pageToken={}",
            url::form_urlencoded::byte_serialize(token.as_bytes()).collect::<String>()
        ));
    }
    url
}

/// Build URL for getting a person's connections/relations.
pub fn build_people_relations_url(resource_name: Option<&str>) -> String {
    let rn = resource_name.unwrap_or("people/me");
    format!(
        "{}/{}/connections?personFields=names,emailAddresses,relations",
        PEOPLE_BASE_URL, rn
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-PEOPLE-001 (Must): People me URL
    // ---------------------------------------------------------------

    // Requirement: REQ-PEOPLE-001 (Must)
    // Acceptance: Me URL includes personFields
    #[test]
    fn req_people_001_me_url() {
        let url = build_people_me_url();
        assert!(url.contains("people/me"));
        assert!(url.contains("personFields="));
    }

    // ---------------------------------------------------------------
    // REQ-PEOPLE-002 (Must): People get URL
    // ---------------------------------------------------------------

    // Requirement: REQ-PEOPLE-002 (Must)
    // Acceptance: Get URL with resource name
    #[test]
    fn req_people_002_get_url() {
        let url = build_people_get_url("people/12345");
        assert!(url.contains("people/12345"));
        assert!(url.contains("personFields="));
    }

    // Requirement: REQ-PEOPLE-002 (Must)
    // Acceptance: Get URL for directory person
    #[test]
    fn req_people_002_get_url_directory() {
        let url = build_people_get_url("people/c12345");
        assert!(url.contains("people/c12345"));
    }

    // ---------------------------------------------------------------
    // REQ-PEOPLE-003 (Must): People search URL
    // ---------------------------------------------------------------

    // Requirement: REQ-PEOPLE-003 (Must)
    // Acceptance: Search URL with query
    #[test]
    fn req_people_003_search_url() {
        let url = build_people_search_url("Alice", None, None);
        assert!(url.contains("searchContacts"));
        assert!(url.contains("query=Alice"));
        assert!(url.contains("pageSize=10"));
    }

    // Requirement: REQ-PEOPLE-003 (Must)
    // Acceptance: Search URL with max and page token
    #[test]
    fn req_people_003_search_url_with_params() {
        let url = build_people_search_url("Bob", Some(25), Some("pg2"));
        assert!(url.contains("pageSize=25"));
        assert!(url.contains("pageToken=pg2"));
    }

    // Requirement: REQ-PEOPLE-003 (Must)
    // Acceptance: Search URL encodes special characters
    #[test]
    fn req_people_003_search_url_encoded() {
        let url = build_people_search_url("John Doe", None, None);
        assert!(url.contains("John+Doe") || url.contains("John%20Doe"));
    }

    // ---------------------------------------------------------------
    // REQ-PEOPLE-004 (Must): People relations URL
    // ---------------------------------------------------------------

    // Requirement: REQ-PEOPLE-004 (Must)
    // Acceptance: Relations URL defaults to "me"
    #[test]
    fn req_people_004_relations_url_default() {
        let url = build_people_relations_url(None);
        assert!(url.contains("people/me/connections"));
        assert!(url.contains("personFields="));
        assert!(url.contains("relations"));
    }

    // Requirement: REQ-PEOPLE-004 (Must)
    // Acceptance: Relations URL with specific resource
    #[test]
    fn req_people_004_relations_url_specific() {
        let url = build_people_relations_url(Some("people/12345"));
        assert!(url.contains("people/12345/connections"));
    }
}
