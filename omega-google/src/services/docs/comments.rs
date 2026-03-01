//! Document comment CRUD operations via the Drive API.

/// Drive API base URL for comment operations.
const DRIVE_BASE_URL: &str = "https://www.googleapis.com/drive/v3";

/// Build URL for listing comments on a file.
pub fn build_comments_list_url(file_id: &str) -> String {
    format!("{}/files/{}/comments?fields=*", DRIVE_BASE_URL, file_id)
}

/// Build URL for getting a specific comment.
pub fn build_comment_get_url(file_id: &str, comment_id: &str) -> String {
    format!(
        "{}/files/{}/comments/{}?fields=*",
        DRIVE_BASE_URL, file_id, comment_id
    )
}

/// Build URL for creating a comment on a file.
pub fn build_comment_create_url(file_id: &str) -> String {
    format!("{}/files/{}/comments?fields=*", DRIVE_BASE_URL, file_id)
}

/// Build the request body for creating a comment.
pub fn build_comment_create_body(content: &str) -> serde_json::Value {
    serde_json::json!({
        "content": content
    })
}

/// Build URL for replying to a comment.
pub fn build_comment_reply_url(file_id: &str, comment_id: &str) -> String {
    format!(
        "{}/files/{}/comments/{}/replies?fields=*",
        DRIVE_BASE_URL, file_id, comment_id
    )
}

/// Build the request body for replying to a comment.
pub fn build_comment_reply_body(content: &str) -> serde_json::Value {
    serde_json::json!({
        "content": content
    })
}

/// Build the request body for resolving (marking as done) a comment.
pub fn build_comment_resolve_body() -> serde_json::Value {
    serde_json::json!({
        "resolved": true
    })
}

/// Build URL for resolving (updating) a comment.
pub fn build_comment_resolve_url(file_id: &str, comment_id: &str) -> String {
    format!(
        "{}/files/{}/comments/{}?fields=*",
        DRIVE_BASE_URL, file_id, comment_id
    )
}

/// Build URL for deleting a comment.
pub fn build_comment_delete_url(file_id: &str, comment_id: &str) -> String {
    format!(
        "{}/files/{}/comments/{}",
        DRIVE_BASE_URL, file_id, comment_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-DOCS-007: Comment CRUD URL builders and body builders
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: Comments list URL contains file ID
    #[test]
    fn req_docs_007_comments_list_url() {
        let url = build_comments_list_url("doc123");
        assert!(url.contains("files/doc123/comments"));
        assert!(url.contains("fields=*"));
    }

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: Comment get URL contains file and comment IDs
    #[test]
    fn req_docs_007_comment_get_url() {
        let url = build_comment_get_url("doc123", "comment456");
        assert!(url.contains("files/doc123/comments/comment456"));
        assert!(url.contains("fields=*"));
    }

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: Comment create URL contains file ID
    #[test]
    fn req_docs_007_comment_create_url() {
        let url = build_comment_create_url("doc123");
        assert!(url.contains("files/doc123/comments"));
        assert!(url.contains("fields=*"));
    }

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: Comment create body has content
    #[test]
    fn req_docs_007_comment_create_body() {
        let body = build_comment_create_body("This is a comment");
        assert_eq!(body["content"], "This is a comment");
    }

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: Comment reply URL contains file, comment IDs
    #[test]
    fn req_docs_007_comment_reply_url() {
        let url = build_comment_reply_url("doc123", "comment456");
        assert!(url.contains("files/doc123/comments/comment456/replies"));
        assert!(url.contains("fields=*"));
    }

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: Comment reply body has content
    #[test]
    fn req_docs_007_comment_reply_body() {
        let body = build_comment_reply_body("This is a reply");
        assert_eq!(body["content"], "This is a reply");
    }

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: Comment resolve body sets resolved to true
    #[test]
    fn req_docs_007_comment_resolve_body() {
        let body = build_comment_resolve_body();
        assert_eq!(body["resolved"], true);
    }

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: Comment resolve URL contains file and comment IDs
    #[test]
    fn req_docs_007_comment_resolve_url() {
        let url = build_comment_resolve_url("doc123", "comment456");
        assert!(url.contains("files/doc123/comments/comment456"));
        assert!(url.contains("fields=*"));
    }

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: Comment delete URL contains file and comment IDs
    #[test]
    fn req_docs_007_comment_delete_url() {
        let url = build_comment_delete_url("doc123", "comment456");
        assert!(url.contains("files/doc123/comments/comment456"));
        // Delete URL should not have fields=* query param
        assert!(!url.ends_with("fields=*"));
    }

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: Empty content in comment create body
    #[test]
    fn req_docs_007_comment_create_body_empty() {
        let body = build_comment_create_body("");
        assert_eq!(body["content"], "");
    }

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: URLs use Drive API v3 base
    #[test]
    fn req_docs_007_urls_use_drive_base() {
        let url = build_comments_list_url("x");
        assert!(url.starts_with("https://www.googleapis.com/drive/v3"));
    }
}
