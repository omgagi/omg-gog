//! Chat space URL and body builders.

use super::CHAT_BASE_URL;

/// Build URL for listing spaces.
/// REQ-CHAT-001
pub fn build_spaces_list_url(max: Option<u32>, page_token: Option<&str>) -> String {
    let base = format!("{}/spaces", CHAT_BASE_URL);
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

/// Build URL for finding spaces by display name using a filter parameter.
/// REQ-CHAT-002
pub fn build_spaces_find_url(display_name: &str, max: Option<u32>) -> String {
    let base = format!("{}/spaces", CHAT_BASE_URL);
    let encoded_filter = url::form_urlencoded::byte_serialize(
        format!("displayName = \"{}\"", display_name).as_bytes(),
    )
    .collect::<String>();
    let mut params = vec![format!("filter={}", encoded_filter)];
    if let Some(m) = max {
        params.push(format!("pageSize={}", m));
    }
    format!("{}?{}", base, params.join("&"))
}

/// Build URL for creating a space.
/// REQ-CHAT-003
pub fn build_space_create_url() -> String {
    format!("{}/spaces", CHAT_BASE_URL)
}

/// Build request body for creating a space.
/// REQ-CHAT-003
pub fn build_space_create_body(display_name: &str, members: &[&str]) -> serde_json::Value {
    let mut body = serde_json::json!({
        "displayName": display_name,
        "spaceType": "SPACE",
    });
    if !members.is_empty() {
        let member_list: Vec<serde_json::Value> = members
            .iter()
            .map(|m| serde_json::json!({"member": {"name": m, "type": "HUMAN"}}))
            .collect();
        body["memberships"] = serde_json::json!(member_list);
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CHAT-001 (Must): Spaces list URL
    // ---------------------------------------------------------------

    // Requirement: REQ-CHAT-001 (Must)
    // Acceptance: Spaces list URL with no parameters
    #[test]
    fn req_chat_001_spaces_list_url_default() {
        let url = build_spaces_list_url(None, None);
        assert_eq!(url, "https://chat.googleapis.com/v1/spaces");
    }

    // Requirement: REQ-CHAT-001 (Must)
    // Acceptance: Spaces list URL with max results
    #[test]
    fn req_chat_001_spaces_list_url_max() {
        let url = build_spaces_list_url(Some(20), None);
        assert!(url.contains("pageSize=20"));
    }

    // Requirement: REQ-CHAT-001 (Must)
    // Acceptance: Spaces list URL with page token
    #[test]
    fn req_chat_001_spaces_list_url_page_token() {
        let url = build_spaces_list_url(None, Some("abc123"));
        assert!(url.contains("pageToken=abc123"));
    }

    // Requirement: REQ-CHAT-001 (Must)
    // Acceptance: Spaces list URL with both parameters
    #[test]
    fn req_chat_001_spaces_list_url_all_params() {
        let url = build_spaces_list_url(Some(10), Some("token"));
        assert!(url.contains("pageSize=10"));
        assert!(url.contains("pageToken=token"));
    }

    // ---------------------------------------------------------------
    // REQ-CHAT-002 (Must): Spaces find URL
    // ---------------------------------------------------------------

    // Requirement: REQ-CHAT-002 (Must)
    // Acceptance: Spaces find URL with display name filter
    #[test]
    fn req_chat_002_spaces_find_url() {
        let url = build_spaces_find_url("Engineering", None);
        assert!(url.contains("filter="));
        assert!(url.contains("Engineering"));
    }

    // Requirement: REQ-CHAT-002 (Must)
    // Acceptance: Spaces find URL with max results
    #[test]
    fn req_chat_002_spaces_find_url_with_max() {
        let url = build_spaces_find_url("Engineering", Some(5));
        assert!(url.contains("filter="));
        assert!(url.contains("pageSize=5"));
    }

    // ---------------------------------------------------------------
    // REQ-CHAT-003 (Must): Space creation
    // ---------------------------------------------------------------

    // Requirement: REQ-CHAT-003 (Must)
    // Acceptance: Space create URL
    #[test]
    fn req_chat_003_space_create_url() {
        let url = build_space_create_url();
        assert_eq!(url, "https://chat.googleapis.com/v1/spaces");
    }

    // Requirement: REQ-CHAT-003 (Must)
    // Acceptance: Space create body has required fields
    #[test]
    fn req_chat_003_space_create_body_basic() {
        let body = build_space_create_body("My Space", &[]);
        assert_eq!(body["displayName"], "My Space");
        assert_eq!(body["spaceType"], "SPACE");
        assert!(body.get("memberships").is_none());
    }

    // Requirement: REQ-CHAT-003 (Must)
    // Acceptance: Space create body with members
    #[test]
    fn req_chat_003_space_create_body_with_members() {
        let body = build_space_create_body("Team Space", &["users/user1", "users/user2"]);
        assert_eq!(body["displayName"], "Team Space");
        let memberships = body["memberships"].as_array().unwrap();
        assert_eq!(memberships.len(), 2);
        assert_eq!(memberships[0]["member"]["name"], "users/user1");
        assert_eq!(memberships[1]["member"]["name"], "users/user2");
    }
}
