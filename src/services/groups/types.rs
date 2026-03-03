//! Google Cloud Identity Groups API request/response types.
//! All types use camelCase serde rename and flatten for forward compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------
// Group types
// ---------------------------------------------------------------

/// A Google Cloud Identity group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub name: Option<String>,
    pub group_key: Option<GroupKey>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub labels: Option<HashMap<String, String>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A group key identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupKey {
    pub id: Option<String>,
    pub namespace: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response for listing groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupListResponse {
    #[serde(default)]
    pub groups: Vec<Group>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Membership types
// ---------------------------------------------------------------

/// A membership in a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Membership {
    pub name: Option<String>,
    pub preferred_member_key: Option<MemberKey>,
    #[serde(default)]
    pub roles: Vec<MembershipRole>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub create_time: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A member key identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberKey {
    pub id: Option<String>,
    pub namespace: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A role within a membership.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipRole {
    pub name: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response for listing memberships.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipListResponse {
    #[serde(default)]
    pub memberships: Vec<Membership>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-GROUPS-001 (Must): Group type deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-GROUPS-001 (Must)
    // Acceptance: Group type deserializes from Cloud Identity API JSON
    #[test]
    fn req_groups_001_group_deserialize() {
        // REQ-GROUPS-001
        let json_str = r#"{
            "name": "groups/abc123",
            "groupKey": {
                "id": "engineering@example.com"
            },
            "displayName": "Engineering",
            "description": "Engineering team group",
            "labels": {
                "cloudidentity.googleapis.com/groups.discussion_forum": ""
            }
        }"#;
        let group: Group = serde_json::from_str(json_str).unwrap();
        assert_eq!(group.name, Some("groups/abc123".to_string()));
        assert_eq!(group.display_name, Some("Engineering".to_string()));
        assert_eq!(
            group.description,
            Some("Engineering team group".to_string())
        );
        let key = group.group_key.unwrap();
        assert_eq!(key.id, Some("engineering@example.com".to_string()));
        let labels = group.labels.unwrap();
        assert!(labels.contains_key("cloudidentity.googleapis.com/groups.discussion_forum"));
    }

    // Requirement: REQ-GROUPS-001 (Must)
    // Acceptance: GroupListResponse deserializes with pagination
    #[test]
    fn req_groups_001_group_list_response_deserialize() {
        // REQ-GROUPS-001
        let json_str = r#"{
            "groups": [
                {
                    "name": "groups/aaa",
                    "groupKey": {"id": "team-a@example.com"},
                    "displayName": "Team A"
                },
                {
                    "name": "groups/bbb",
                    "groupKey": {"id": "team-b@example.com"},
                    "displayName": "Team B"
                }
            ],
            "nextPageToken": "token_xyz"
        }"#;
        let resp: GroupListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.groups.len(), 2);
        assert_eq!(resp.groups[0].name, Some("groups/aaa".to_string()));
        assert_eq!(resp.next_page_token, Some("token_xyz".to_string()));
    }

    // Requirement: REQ-GROUPS-001 (Must)
    // Edge case: GroupListResponse with empty groups list
    #[test]
    fn req_groups_001_group_list_response_empty() {
        // REQ-GROUPS-001
        let json_str = r#"{}"#;
        let resp: GroupListResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.groups.is_empty());
        assert!(resp.next_page_token.is_none());
    }

    // Requirement: REQ-GROUPS-001 (Must)
    // Edge case: Group with unknown fields preserved via flatten
    #[test]
    fn req_groups_001_group_unknown_fields_preserved() {
        // REQ-GROUPS-001
        let json_str = r#"{
            "name": "groups/xxx",
            "futureField": "some_value"
        }"#;
        let group: Group = serde_json::from_str(json_str).unwrap();
        assert_eq!(group.name, Some("groups/xxx".to_string()));
        assert!(group.extra.contains_key("futureField"));
    }

    // Requirement: REQ-GROUPS-001 (Must)
    // Acceptance: Group round-trip serialization
    #[test]
    fn req_groups_001_group_roundtrip() {
        // REQ-GROUPS-001
        let group = Group {
            name: Some("groups/aaa".to_string()),
            group_key: Some(GroupKey {
                id: Some("test@example.com".to_string()),
                namespace: None,
                extra: HashMap::new(),
            }),
            display_name: Some("Test".to_string()),
            description: None,
            labels: None,
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&group).unwrap();
        let parsed: Group = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, Some("groups/aaa".to_string()));
        assert_eq!(parsed.display_name, Some("Test".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-GROUPS-002 (Must): Membership type deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-GROUPS-002 (Must)
    // Acceptance: Membership type deserializes from Cloud Identity API JSON
    #[test]
    fn req_groups_002_membership_deserialize() {
        // REQ-GROUPS-002
        let json_str = r#"{
            "name": "groups/abc123/memberships/mem456",
            "preferredMemberKey": {
                "id": "user@example.com"
            },
            "roles": [
                {"name": "MEMBER"},
                {"name": "OWNER"}
            ],
            "type": "USER",
            "createTime": "2024-01-15T10:30:00Z"
        }"#;
        let membership: Membership = serde_json::from_str(json_str).unwrap();
        assert_eq!(
            membership.name,
            Some("groups/abc123/memberships/mem456".to_string())
        );
        assert_eq!(membership.type_, Some("USER".to_string()));
        assert_eq!(
            membership.create_time,
            Some("2024-01-15T10:30:00Z".to_string())
        );

        let key = membership.preferred_member_key.unwrap();
        assert_eq!(key.id, Some("user@example.com".to_string()));

        assert_eq!(membership.roles.len(), 2);
        assert_eq!(membership.roles[0].name, Some("MEMBER".to_string()));
        assert_eq!(membership.roles[1].name, Some("OWNER".to_string()));
    }

    // Requirement: REQ-GROUPS-002 (Must)
    // Acceptance: MembershipListResponse deserializes with pagination
    #[test]
    fn req_groups_002_membership_list_response_deserialize() {
        // REQ-GROUPS-002
        let json_str = r#"{
            "memberships": [
                {
                    "name": "groups/aaa/memberships/m1",
                    "preferredMemberKey": {"id": "alice@example.com"},
                    "roles": [{"name": "MEMBER"}]
                },
                {
                    "name": "groups/aaa/memberships/m2",
                    "preferredMemberKey": {"id": "bob@example.com"},
                    "roles": [{"name": "OWNER"}]
                }
            ],
            "nextPageToken": "page2"
        }"#;
        let resp: MembershipListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.memberships.len(), 2);
        assert_eq!(
            resp.memberships[0].name,
            Some("groups/aaa/memberships/m1".to_string())
        );
        assert_eq!(resp.next_page_token, Some("page2".to_string()));
    }

    // Requirement: REQ-GROUPS-002 (Must)
    // Edge case: MembershipListResponse with empty memberships
    #[test]
    fn req_groups_002_membership_list_response_empty() {
        // REQ-GROUPS-002
        let json_str = r#"{}"#;
        let resp: MembershipListResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.memberships.is_empty());
        assert!(resp.next_page_token.is_none());
    }

    // Requirement: REQ-GROUPS-002 (Must)
    // Acceptance: Membership round-trip serialization
    #[test]
    fn req_groups_002_membership_roundtrip() {
        // REQ-GROUPS-002
        let membership = Membership {
            name: Some("groups/aaa/memberships/m1".to_string()),
            preferred_member_key: Some(MemberKey {
                id: Some("user@example.com".to_string()),
                namespace: None,
                extra: HashMap::new(),
            }),
            roles: vec![MembershipRole {
                name: Some("MEMBER".to_string()),
                extra: HashMap::new(),
            }],
            type_: Some("USER".to_string()),
            create_time: Some("2024-01-15T10:30:00Z".to_string()),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&membership).unwrap();
        let parsed: Membership = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, Some("groups/aaa/memberships/m1".to_string()));
        assert_eq!(parsed.type_, Some("USER".to_string()));
    }

    // Requirement: REQ-GROUPS-002 (Must)
    // Acceptance: MemberKey unknown fields preserved
    #[test]
    fn req_groups_002_member_key_unknown_fields() {
        // REQ-GROUPS-002
        let json_str = r#"{
            "id": "user@example.com",
            "namespace": "identitysources/abc",
            "futureField": 42
        }"#;
        let key: MemberKey = serde_json::from_str(json_str).unwrap();
        assert_eq!(key.id, Some("user@example.com".to_string()));
        assert_eq!(key.namespace, Some("identitysources/abc".to_string()));
        assert!(key.extra.contains_key("futureField"));
    }
}
