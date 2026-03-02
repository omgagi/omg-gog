//! Groups service integration tests.

use omega_google::services::groups::types::*;
use omega_google::services::groups::groups::*;

// ---------------------------------------------------------------
// REQ-GROUPS-001 (Must): Group deserialization from full JSON
// ---------------------------------------------------------------

// Requirement: REQ-GROUPS-001 (Must)
// Acceptance: Full group structure from a realistic Cloud Identity API response
#[test]
fn req_groups_001_integration_full_group_from_api() {
    // REQ-GROUPS-001
    let api_response = r#"{
        "name": "groups/abc123def456",
        "groupKey": {
            "id": "engineering@example.com"
        },
        "displayName": "Engineering Team",
        "description": "All engineering staff",
        "labels": {
            "cloudidentity.googleapis.com/groups.discussion_forum": ""
        },
        "createTime": "2023-01-01T00:00:00Z",
        "updateTime": "2024-06-15T12:00:00Z"
    }"#;

    let group: Group = serde_json::from_str(api_response).unwrap();

    assert_eq!(group.name, Some("groups/abc123def456".to_string()));
    assert_eq!(group.display_name, Some("Engineering Team".to_string()));
    assert_eq!(group.description, Some("All engineering staff".to_string()));

    let key = group.group_key.unwrap();
    assert_eq!(key.id, Some("engineering@example.com".to_string()));

    let labels = group.labels.unwrap();
    assert!(labels.contains_key("cloudidentity.googleapis.com/groups.discussion_forum"));

    // Unknown fields preserved via flatten
    assert!(group.extra.contains_key("createTime"));
    assert!(group.extra.contains_key("updateTime"));
}

// ---------------------------------------------------------------
// REQ-GROUPS-001 (Must): Group list response from realistic API
// ---------------------------------------------------------------

// Requirement: REQ-GROUPS-001 (Must)
// Acceptance: GroupListResponse with multiple groups and pagination
#[test]
fn req_groups_001_integration_group_list_from_api() {
    // REQ-GROUPS-001
    let api_response = r#"{
        "groups": [
            {
                "name": "groups/aaa",
                "groupKey": {"id": "team-a@example.com"},
                "displayName": "Team A",
                "description": "First team"
            },
            {
                "name": "groups/bbb",
                "groupKey": {"id": "team-b@example.com"},
                "displayName": "Team B"
            },
            {
                "name": "groups/ccc",
                "groupKey": {"id": "all@example.com"},
                "displayName": "Everyone",
                "labels": {"cloudidentity.googleapis.com/groups.discussion_forum": ""}
            }
        ],
        "nextPageToken": "groups_page_2_token"
    }"#;

    let resp: GroupListResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(resp.groups.len(), 3);
    assert_eq!(resp.next_page_token, Some("groups_page_2_token".to_string()));

    // First group
    assert_eq!(resp.groups[0].name, Some("groups/aaa".to_string()));
    assert_eq!(resp.groups[0].display_name, Some("Team A".to_string()));

    // Second group
    assert_eq!(resp.groups[1].display_name, Some("Team B".to_string()));

    // Third group with labels
    assert_eq!(resp.groups[2].display_name, Some("Everyone".to_string()));
    assert!(resp.groups[2].labels.is_some());
}

// ---------------------------------------------------------------
// REQ-GROUPS-002 (Must): Membership deserialization from full JSON
// ---------------------------------------------------------------

// Requirement: REQ-GROUPS-002 (Must)
// Acceptance: Full membership structure from a realistic API response
#[test]
fn req_groups_002_integration_full_membership_from_api() {
    // REQ-GROUPS-002
    let api_response = r#"{
        "name": "groups/abc123/memberships/mem789",
        "preferredMemberKey": {
            "id": "alice@example.com"
        },
        "roles": [
            {"name": "MEMBER"},
            {"name": "MANAGER"}
        ],
        "type": "USER",
        "createTime": "2024-03-15T08:00:00.000Z",
        "updateTime": "2024-06-01T12:00:00Z"
    }"#;

    let membership: Membership = serde_json::from_str(api_response).unwrap();

    assert_eq!(membership.name, Some("groups/abc123/memberships/mem789".to_string()));
    assert_eq!(membership.type_, Some("USER".to_string()));
    assert_eq!(membership.create_time, Some("2024-03-15T08:00:00.000Z".to_string()));

    let key = membership.preferred_member_key.unwrap();
    assert_eq!(key.id, Some("alice@example.com".to_string()));

    assert_eq!(membership.roles.len(), 2);
    assert_eq!(membership.roles[0].name, Some("MEMBER".to_string()));
    assert_eq!(membership.roles[1].name, Some("MANAGER".to_string()));

    // Unknown fields preserved via flatten
    assert!(membership.extra.contains_key("updateTime"));
}

// ---------------------------------------------------------------
// REQ-GROUPS-002 (Must): Membership list response deserialization
// ---------------------------------------------------------------

// Requirement: REQ-GROUPS-002 (Must)
// Acceptance: MembershipListResponse with full memberships from API
#[test]
fn req_groups_002_integration_membership_list_from_api() {
    // REQ-GROUPS-002
    let api_response = r#"{
        "memberships": [
            {
                "name": "groups/aaa/memberships/m1",
                "preferredMemberKey": {"id": "alice@example.com"},
                "roles": [{"name": "OWNER"}],
                "type": "USER",
                "createTime": "2024-01-01T00:00:00Z"
            },
            {
                "name": "groups/aaa/memberships/m2",
                "preferredMemberKey": {"id": "bob@example.com"},
                "roles": [{"name": "MEMBER"}],
                "type": "USER"
            },
            {
                "name": "groups/aaa/memberships/m3",
                "preferredMemberKey": {"id": "subgroup@example.com"},
                "roles": [{"name": "MEMBER"}],
                "type": "GROUP"
            }
        ],
        "nextPageToken": "members_page_2"
    }"#;

    let resp: MembershipListResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(resp.memberships.len(), 3);
    assert_eq!(resp.next_page_token, Some("members_page_2".to_string()));

    // First member: owner
    let m1 = &resp.memberships[0];
    assert_eq!(m1.type_, Some("USER".to_string()));
    assert_eq!(m1.roles[0].name, Some("OWNER".to_string()));

    // Third member: group type
    let m3 = &resp.memberships[2];
    assert_eq!(m3.type_, Some("GROUP".to_string()));
}

// ---------------------------------------------------------------
// REQ-GROUPS-001 (Must): URL builder verification - groups list
// ---------------------------------------------------------------

// Requirement: REQ-GROUPS-001 (Must)
// Acceptance: Groups list URL builds correctly with various params
#[test]
fn req_groups_001_integration_url_builder_groups_list() {
    // REQ-GROUPS-001
    // Default
    let url = build_groups_list_url(None, None);
    assert!(url.starts_with("https://cloudidentity.googleapis.com/v1/groups:search?"));
    assert!(url.contains("view=FULL"));

    // With page size
    let url = build_groups_list_url(Some(50), None);
    assert!(url.contains("pageSize=50"));

    // With page token
    let url = build_groups_list_url(None, Some("token123"));
    assert!(url.contains("pageToken=token123"));

    // With both
    let url = build_groups_list_url(Some(25), Some("next_token"));
    assert!(url.contains("pageSize=25"));
    assert!(url.contains("pageToken=next_token"));
}

// ---------------------------------------------------------------
// REQ-GROUPS-001 (Must): URL builder verification - group lookup
// ---------------------------------------------------------------

// Requirement: REQ-GROUPS-001 (Must)
// Acceptance: Group lookup URL builds correctly
#[test]
fn req_groups_001_integration_url_builder_group_lookup() {
    // REQ-GROUPS-001
    let url = build_group_lookup_url("engineering@example.com");
    assert!(url.starts_with("https://cloudidentity.googleapis.com/v1/groups:lookup?"));
    assert!(url.contains("groupKey.id=engineering%40example.com"));
}

// ---------------------------------------------------------------
// REQ-GROUPS-002 (Must): URL builder verification - members list
// ---------------------------------------------------------------

// Requirement: REQ-GROUPS-002 (Must)
// Acceptance: Members list URL builds correctly with various params
#[test]
fn req_groups_002_integration_url_builder_members_list() {
    // REQ-GROUPS-002
    // Basic members list
    let url = build_members_list_url("groups/abc123", None, None);
    assert!(url.contains("/memberships"));

    // With all parameters
    let url = build_members_list_url("groups/abc123", Some(100), Some("page2"));
    assert!(url.contains("pageSize=100"));
    assert!(url.contains("pageToken=page2"));
}
