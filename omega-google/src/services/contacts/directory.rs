//! Directory and other contacts URL builders.

use super::PEOPLE_API_BASE_URL;

/// Standard read mask for directory people fields.
const DIRECTORY_FIELDS: &str = "names,emailAddresses,phoneNumbers,photos";

/// Build URL for listing directory people.
pub fn build_directory_list_url(max: Option<u32>, page_token: Option<&str>) -> String {
    let max_val = max.unwrap_or(100);
    let mut url = format!(
        "{}/people:listDirectoryPeople?readMask={}&pageSize={}&sources=DIRECTORY_SOURCE_TYPE_DOMAIN_PROFILE",
        PEOPLE_API_BASE_URL,
        DIRECTORY_FIELDS,
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

/// Build URL for searching directory people.
pub fn build_directory_search_url(query: &str, max: Option<u32>) -> String {
    let max_val = max.unwrap_or(10);
    format!(
        "{}/people:searchDirectoryPeople?query={}&readMask={}&pageSize={}&sources=DIRECTORY_SOURCE_TYPE_DOMAIN_PROFILE",
        PEOPLE_API_BASE_URL,
        url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>(),
        DIRECTORY_FIELDS,
        max_val
    )
}

/// Build URL for listing other contacts.
pub fn build_other_contacts_list_url(max: Option<u32>, page_token: Option<&str>) -> String {
    let max_val = max.unwrap_or(100);
    let mut url = format!(
        "{}/otherContacts?readMask={}&pageSize={}",
        PEOPLE_API_BASE_URL,
        DIRECTORY_FIELDS,
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

/// Build URL for searching other contacts.
pub fn build_other_contacts_search_url(query: &str, max: Option<u32>) -> String {
    let max_val = max.unwrap_or(10);
    format!(
        "{}/otherContacts:search?query={}&readMask={}&pageSize={}",
        PEOPLE_API_BASE_URL,
        url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>(),
        DIRECTORY_FIELDS,
        max_val
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CONTACTS-007 (Must): Directory URL builders
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-007 (Must)
    // Acceptance: Directory list URL without params
    #[test]
    fn req_contacts_007_directory_list_url_no_params() {
        let url = build_directory_list_url(None, None);
        assert!(url.contains("listDirectoryPeople"));
        assert!(url.contains("readMask="));
        assert!(url.contains("pageSize=100"));
        assert!(url.contains("DIRECTORY_SOURCE_TYPE_DOMAIN_PROFILE"));
    }

    // Requirement: REQ-CONTACTS-007 (Must)
    // Acceptance: Directory list URL with params
    #[test]
    fn req_contacts_007_directory_list_url_with_params() {
        let url = build_directory_list_url(Some(50), Some("nextdir"));
        assert!(url.contains("pageSize=50"));
        assert!(url.contains("pageToken=nextdir"));
    }

    // Requirement: REQ-CONTACTS-007 (Must)
    // Acceptance: Directory search URL
    #[test]
    fn req_contacts_007_directory_search_url() {
        let url = build_directory_search_url("Alice", None);
        assert!(url.contains("searchDirectoryPeople"));
        assert!(url.contains("query=Alice"));
        assert!(url.contains("pageSize=10"));
    }

    // Requirement: REQ-CONTACTS-007 (Must)
    // Acceptance: Directory search URL with max
    #[test]
    fn req_contacts_007_directory_search_url_with_max() {
        let url = build_directory_search_url("Bob", Some(20));
        assert!(url.contains("pageSize=20"));
    }

    // ---------------------------------------------------------------
    // REQ-CONTACTS-008 (Must): Other contacts URL builders
    // ---------------------------------------------------------------

    // Requirement: REQ-CONTACTS-008 (Must)
    // Acceptance: Other contacts list URL without params
    #[test]
    fn req_contacts_008_other_contacts_list_url_no_params() {
        let url = build_other_contacts_list_url(None, None);
        assert!(url.contains("otherContacts"));
        assert!(url.contains("readMask="));
        assert!(url.contains("pageSize=100"));
    }

    // Requirement: REQ-CONTACTS-008 (Must)
    // Acceptance: Other contacts list URL with params
    #[test]
    fn req_contacts_008_other_contacts_list_url_with_params() {
        let url = build_other_contacts_list_url(Some(30), Some("nextother"));
        assert!(url.contains("pageSize=30"));
        assert!(url.contains("pageToken=nextother"));
    }

    // Requirement: REQ-CONTACTS-008 (Must)
    // Acceptance: Other contacts search URL
    #[test]
    fn req_contacts_008_other_contacts_search_url() {
        let url = build_other_contacts_search_url("Carol", None);
        assert!(url.contains("otherContacts:search"));
        assert!(url.contains("query=Carol"));
        assert!(url.contains("pageSize=10"));
    }

    // Requirement: REQ-CONTACTS-008 (Must)
    // Acceptance: Other contacts search URL with max
    #[test]
    fn req_contacts_008_other_contacts_search_url_with_max() {
        let url = build_other_contacts_search_url("Dave", Some(5));
        assert!(url.contains("pageSize=5"));
    }
}
