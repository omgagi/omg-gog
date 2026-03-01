//! Drive file comments.

use super::types::DRIVE_BASE_URL;

/// Build URL for listing comments on a file.
pub fn build_comments_list_url(file_id: &str) -> String {
    format!("{}/files/{}/comments?fields=*", DRIVE_BASE_URL, file_id)
}

/// Build URL for creating a comment.
pub fn build_comment_create_url(file_id: &str) -> String {
    format!("{}/files/{}/comments?fields=*", DRIVE_BASE_URL, file_id)
}

/// Build URL for replying to a comment.
pub fn build_comment_reply_url(file_id: &str, comment_id: &str) -> String {
    format!(
        "{}/files/{}/comments/{}/replies?fields=*",
        DRIVE_BASE_URL, file_id, comment_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Requirement: REQ-DRIVE-016 (Should)
    // Acceptance: Comments list URL
    #[test]
    fn req_drive_016_comments_list_url() {
        let url = build_comments_list_url("file_abc");
        assert!(url.contains("files/file_abc/comments"));
    }

    // Requirement: REQ-DRIVE-016 (Should)
    // Acceptance: Comment create URL
    #[test]
    fn req_drive_016_comment_create_url() {
        let url = build_comment_create_url("file_abc");
        assert!(url.contains("files/file_abc/comments"));
    }

    // Requirement: REQ-DRIVE-016 (Should)
    // Acceptance: Comment reply URL
    #[test]
    fn req_drive_016_comment_reply_url() {
        let url = build_comment_reply_url("file_abc", "comment_1");
        assert!(url.contains("files/file_abc/comments/comment_1/replies"));
    }
}
