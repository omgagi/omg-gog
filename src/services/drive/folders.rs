//! Drive folder operations: mkdir, move, rename, delete.

use super::types::DRIVE_BASE_URL;
use serde_json::json;

/// Build request body for creating a folder.
pub fn build_mkdir_body(name: &str, parent: Option<&str>) -> serde_json::Value {
    let mut body = json!({
        "name": name,
        "mimeType": "application/vnd.google-apps.folder"
    });
    if let Some(parent_id) = parent {
        body["parents"] = json!([parent_id]);
    }
    body
}

/// Build request body for moving a file.
pub fn build_move_params(_file_id: &str, new_parent: &str, old_parent: &str) -> (String, String) {
    (new_parent.to_string(), old_parent.to_string())
}

/// Build request body for renaming a file.
pub fn build_rename_body(new_name: &str) -> serde_json::Value {
    json!({
        "name": new_name
    })
}

/// Build URL for trashing a file.
pub fn build_trash_url(file_id: &str) -> String {
    format!("{}/files/{}", DRIVE_BASE_URL, file_id)
}

/// Build URL for permanently deleting a file.
pub fn build_permanent_delete_url(file_id: &str) -> String {
    format!("{}/files/{}", DRIVE_BASE_URL, file_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Requirement: REQ-DRIVE-006 (Must)
    // Acceptance: Mkdir body has correct name and mimeType
    #[test]
    fn req_drive_006_mkdir_body() {
        let body = build_mkdir_body("New Folder", None);
        assert_eq!(body["name"], "New Folder");
        assert_eq!(body["mimeType"], "application/vnd.google-apps.folder");
    }

    // Requirement: REQ-DRIVE-006 (Must)
    // Acceptance: Mkdir body with parent
    #[test]
    fn req_drive_006_mkdir_body_with_parent() {
        let body = build_mkdir_body("Subfolder", Some("parent_id"));
        assert_eq!(body["name"], "Subfolder");
        assert!(body["parents"].is_array());
    }

    // Requirement: REQ-DRIVE-008 (Must)
    // Acceptance: Move params include add/remove parents
    #[test]
    fn req_drive_008_move_params() {
        let (add_parents, remove_parents) = build_move_params("file1", "new_parent", "old_parent");
        assert_eq!(add_parents, "new_parent");
        assert_eq!(remove_parents, "old_parent");
    }

    // Requirement: REQ-DRIVE-009 (Must)
    // Acceptance: Rename body has new name
    #[test]
    fn req_drive_009_rename_body() {
        let body = build_rename_body("New Name.pdf");
        assert_eq!(body["name"], "New Name.pdf");
    }

    // Requirement: REQ-DRIVE-007 (Must)
    // Acceptance: Trash URL
    #[test]
    fn req_drive_007_trash_url() {
        let url = build_trash_url("file_abc");
        assert!(url.contains("files/file_abc"));
    }

    // Requirement: REQ-DRIVE-007 (Must)
    // Acceptance: Permanent delete URL
    #[test]
    fn req_drive_007_permanent_delete_url() {
        let url = build_permanent_delete_url("file_abc");
        assert!(url.contains("files/file_abc"));
    }
}
