//! Drive sharing and permissions management.

use serde_json::json;
use super::types::DRIVE_BASE_URL;

/// Build a permission request body for sharing.
pub fn build_share_permission(
    to: &str,
    role: &str,
    email: Option<&str>,
    domain: Option<&str>,
) -> anyhow::Result<serde_json::Value> {
    validate_share_target(to)?;
    validate_role(role)?;

    let mut body = json!({
        "type": to,
        "role": role
    });
    if let Some(email_addr) = email {
        body["emailAddress"] = json!(email_addr);
    }
    if let Some(dom) = domain {
        body["domain"] = json!(dom);
    }
    Ok(body)
}

/// Build URL for creating a permission on a file.
pub fn build_create_permission_url(file_id: &str) -> String {
    format!("{}/files/{}/permissions", DRIVE_BASE_URL, file_id)
}

/// Build URL for listing permissions on a file.
pub fn build_list_permissions_url(file_id: &str) -> String {
    format!("{}/files/{}/permissions", DRIVE_BASE_URL, file_id)
}

/// Build URL for deleting a permission.
pub fn build_delete_permission_url(file_id: &str, permission_id: &str) -> String {
    format!(
        "{}/files/{}/permissions/{}",
        DRIVE_BASE_URL, file_id, permission_id
    )
}

/// Validate a permission role.
pub fn validate_role(role: &str) -> anyhow::Result<()> {
    match role {
        "reader" | "writer" | "commenter" | "owner" | "organizer" | "fileOrganizer" => Ok(()),
        _ => anyhow::bail!("invalid role: '{}'", role),
    }
}

/// Validate a share target type.
pub fn validate_share_target(to: &str) -> anyhow::Result<()> {
    match to {
        "anyone" | "user" | "group" | "domain" => Ok(()),
        _ => anyhow::bail!("invalid share target: '{}'", to),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-DRIVE-010 (Must): Share permission building
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-010 (Must)
    // Acceptance: Share to anyone with reader role
    #[test]
    fn req_drive_010_share_anyone_reader() {
        let body = build_share_permission("anyone", "reader", None, None).unwrap();
        assert_eq!(body["type"], "anyone");
        assert_eq!(body["role"], "reader");
    }

    // Requirement: REQ-DRIVE-010 (Must)
    // Acceptance: Share to specific user with writer role
    #[test]
    fn req_drive_010_share_user_writer() {
        let body = build_share_permission(
            "user",
            "writer",
            Some("user@example.com"),
            None,
        ).unwrap();
        assert_eq!(body["type"], "user");
        assert_eq!(body["role"], "writer");
        assert_eq!(body["emailAddress"], "user@example.com");
    }

    // Requirement: REQ-DRIVE-010 (Must)
    // Acceptance: Share to domain
    #[test]
    fn req_drive_010_share_domain() {
        let body = build_share_permission(
            "domain",
            "reader",
            None,
            Some("example.com"),
        ).unwrap();
        assert_eq!(body["type"], "domain");
        assert_eq!(body["domain"], "example.com");
    }

    // Requirement: REQ-DRIVE-010 (Must)
    // Acceptance: Invalid share target rejected
    #[test]
    fn req_drive_010_invalid_share_target() {
        assert!(validate_share_target("invalid").is_err());
    }

    // Requirement: REQ-DRIVE-010 (Must)
    // Acceptance: Valid share targets accepted
    #[test]
    fn req_drive_010_valid_share_targets() {
        assert!(validate_share_target("anyone").is_ok());
        assert!(validate_share_target("user").is_ok());
        assert!(validate_share_target("domain").is_ok());
    }

    // Requirement: REQ-DRIVE-010 (Must)
    // Acceptance: Valid roles accepted
    #[test]
    fn req_drive_010_valid_roles() {
        assert!(validate_role("reader").is_ok());
        assert!(validate_role("writer").is_ok());
    }

    // Requirement: REQ-DRIVE-010 (Must)
    // Acceptance: Invalid role rejected
    #[test]
    fn req_drive_010_invalid_role() {
        assert!(validate_role("admin").is_err());
        assert!(validate_role("").is_err());
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-011 (Must): Permissions URL
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-011 (Must)
    // Acceptance: List permissions URL
    #[test]
    fn req_drive_011_list_permissions_url() {
        let url = build_list_permissions_url("file_abc");
        assert!(url.contains("files/file_abc/permissions"));
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-012 (Must): Unshare URL
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-012 (Must)
    // Acceptance: Delete permission URL
    #[test]
    fn req_drive_012_delete_permission_url() {
        let url = build_delete_permission_url("file_abc", "perm_123");
        assert!(url.contains("files/file_abc/permissions/perm_123"));
    }
}
