//! Groups URL builders.

use super::GROUPS_BASE_URL;

/// Build URL for listing groups using transitive memberships search.
/// REQ-GROUPS-001
pub fn build_groups_list_url(max: Option<u32>, page_token: Option<&str>) -> String {
    let base = format!("{}/groups:search", GROUPS_BASE_URL);
    let mut params = Vec::new();

    // Default view for transitive memberships — "FULL" is static and URL-safe
    params.push("view=FULL".to_string());

    if let Some(m) = max {
        params.push(format!("pageSize={}", m));
    }
    if let Some(token) = page_token {
        params.push(format!(
            "pageToken={}",
            url::form_urlencoded::byte_serialize(token.as_bytes()).collect::<String>()
        ));
    }
    format!("{}?{}", base, params.join("&"))
}

/// Build URL for looking up a group by email.
/// REQ-GROUPS-001
pub fn build_group_lookup_url(email: &str) -> String {
    let base = format!("{}/groups:lookup", GROUPS_BASE_URL);
    let encoded_email =
        url::form_urlencoded::byte_serialize(email.as_bytes()).collect::<String>();
    format!("{}?groupKey.id={}", base, encoded_email)
}

/// Encode a Google API resource name, encoding each path segment individually
/// to preserve `/` separators while encoding special characters within segments.
fn encode_resource_name(name: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    name.split('/')
        .map(|segment| utf8_percent_encode(segment, NON_ALPHANUMERIC).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// Build URL for listing memberships of a group.
/// REQ-GROUPS-002
pub fn build_members_list_url(
    group_name: &str,
    max: Option<u32>,
    page_token: Option<&str>,
) -> String {
    let encoded_group = encode_resource_name(group_name);
    let base = format!("{}/{}/memberships", GROUPS_BASE_URL, encoded_group);
    let mut params = Vec::new();
    if let Some(m) = max {
        params.push(format!("pageSize={}", m));
    }
    if let Some(token) = page_token {
        params.push(format!(
            "pageToken={}",
            url::form_urlencoded::byte_serialize(token.as_bytes()).collect::<String>()
        ));
    }
    if params.is_empty() {
        base
    } else {
        format!("{}?{}", base, params.join("&"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-GROUPS-001 (Must): Groups list URL
    // ---------------------------------------------------------------

    // Requirement: REQ-GROUPS-001 (Must)
    // Acceptance: Groups list URL with no parameters
    #[test]
    fn req_groups_001_groups_list_url_default() {
        // REQ-GROUPS-001
        let url = build_groups_list_url(None, None);
        assert_eq!(url, "https://cloudidentity.googleapis.com/v1/groups:search?view=FULL");
    }

    // Requirement: REQ-GROUPS-001 (Must)
    // Acceptance: Groups list URL with max results
    #[test]
    fn req_groups_001_groups_list_url_max() {
        // REQ-GROUPS-001
        let url = build_groups_list_url(Some(20), None);
        assert_eq!(url, "https://cloudidentity.googleapis.com/v1/groups:search?view=FULL&pageSize=20");
    }

    // Requirement: REQ-GROUPS-001 (Must)
    // Acceptance: Groups list URL with page token
    #[test]
    fn req_groups_001_groups_list_url_page_token() {
        // REQ-GROUPS-001
        let url = build_groups_list_url(None, Some("abc123"));
        assert_eq!(url, "https://cloudidentity.googleapis.com/v1/groups:search?view=FULL&pageToken=abc123");
    }

    // Requirement: REQ-GROUPS-001 (Must)
    // Acceptance: Groups list URL with both parameters
    #[test]
    fn req_groups_001_groups_list_url_all_params() {
        // REQ-GROUPS-001
        let url = build_groups_list_url(Some(10), Some("token"));
        assert_eq!(url, "https://cloudidentity.googleapis.com/v1/groups:search?view=FULL&pageSize=10&pageToken=token");
    }

    // ---------------------------------------------------------------
    // REQ-GROUPS-001 (Must): Group lookup URL
    // ---------------------------------------------------------------

    // Requirement: REQ-GROUPS-001 (Must)
    // Acceptance: Group lookup URL with email
    #[test]
    fn req_groups_001_group_lookup_url() {
        // REQ-GROUPS-001
        let url = build_group_lookup_url("engineering@example.com");
        assert!(url.starts_with("https://cloudidentity.googleapis.com/v1/groups:lookup?"));
        assert!(url.contains("groupKey.id=engineering%40example.com"));
    }

    // Requirement: REQ-GROUPS-001 (Must)
    // Acceptance: Group lookup URL with special characters in email
    #[test]
    fn req_groups_001_group_lookup_url_special_chars() {
        // REQ-GROUPS-001
        let url = build_group_lookup_url("team+dev@example.com");
        assert!(url.contains("groupKey.id="));
        assert!(url.contains("example.com"));
    }

    // ---------------------------------------------------------------
    // REQ-GROUPS-002 (Must): Members list URL
    // ---------------------------------------------------------------

    // Requirement: REQ-GROUPS-002 (Must)
    // Acceptance: Members list URL with group name — preserves `/` in resource name
    #[test]
    fn req_groups_002_members_list_url_default() {
        // REQ-GROUPS-002
        let url = build_members_list_url("groups/abc123", None, None);
        assert_eq!(url, "https://cloudidentity.googleapis.com/v1/groups/abc123/memberships");
    }

    // Requirement: REQ-GROUPS-002 (Must)
    // Acceptance: Members list URL with max results
    #[test]
    fn req_groups_002_members_list_url_max() {
        // REQ-GROUPS-002
        let url = build_members_list_url("groups/abc123", Some(50), None);
        assert_eq!(url, "https://cloudidentity.googleapis.com/v1/groups/abc123/memberships?pageSize=50");
    }

    // Requirement: REQ-GROUPS-002 (Must)
    // Acceptance: Members list URL with page token
    #[test]
    fn req_groups_002_members_list_url_page_token() {
        // REQ-GROUPS-002
        let url = build_members_list_url("groups/abc123", None, Some("next_token"));
        assert_eq!(url, "https://cloudidentity.googleapis.com/v1/groups/abc123/memberships?pageToken=next_token");
    }

    // Requirement: REQ-GROUPS-002 (Must)
    // Acceptance: Members list URL with all parameters
    #[test]
    fn req_groups_002_members_list_url_all_params() {
        // REQ-GROUPS-002
        let url = build_members_list_url("groups/abc123", Some(25), Some("p2"));
        assert_eq!(url, "https://cloudidentity.googleapis.com/v1/groups/abc123/memberships?pageSize=25&pageToken=p2");
    }
}
