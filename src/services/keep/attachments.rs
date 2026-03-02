//! Keep attachment URL builders.

use super::KEEP_BASE_URL;

/// Encode a Google API resource name, encoding each path segment individually
/// to preserve `/` separators while encoding special characters within segments.
fn encode_resource_name(name: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    name.split('/')
        .map(|segment| utf8_percent_encode(segment, NON_ALPHANUMERIC).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// Build URL for downloading an attachment.
/// REQ-KEEP-004
pub fn build_attachment_download_url(attachment_name: &str) -> String {
    let encoded = encode_resource_name(attachment_name);
    format!("{}/{}:media", KEEP_BASE_URL, encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-KEEP-004 (Must): Attachment download URL
    // ---------------------------------------------------------------

    // Requirement: REQ-KEEP-004 (Must)
    // Acceptance: Attachment download URL preserves `/` separators
    #[test]
    fn req_keep_004_attachment_download_url() {
        // REQ-KEEP-004
        let url = build_attachment_download_url("notes/abc123/attachments/att456");
        assert_eq!(url, "https://keep.googleapis.com/v1/notes/abc123/attachments/att456:media");
    }

    // Requirement: REQ-KEEP-004 (Must)
    // Acceptance: Attachment download URL encodes special chars within segments but preserves `/`
    #[test]
    fn req_keep_004_attachment_download_url_special_chars() {
        // REQ-KEEP-004
        let url = build_attachment_download_url("notes/abc 123/attachments/att+456");
        assert_eq!(url, "https://keep.googleapis.com/v1/notes/abc%20123/attachments/att%2B456:media");
    }
}
