//! Drive file get/download/upload/copy operations.

use super::types::{DRIVE_BASE_URL, extension_for_mime};

/// Build URL for getting file metadata.
pub fn build_file_get_url(file_id: &str) -> String {
    format!("{}/files/{}", DRIVE_BASE_URL, file_id)
}

/// Build URL for downloading a file (binary content).
pub fn build_file_download_url(file_id: &str) -> String {
    format!("{}/files/{}?alt=media", DRIVE_BASE_URL, file_id)
}

/// Build URL for exporting a Google Workspace file.
pub fn build_file_export_url(file_id: &str, mime_type: &str) -> String {
    format!(
        "{}/files/{}/export?mimeType={}",
        DRIVE_BASE_URL,
        file_id,
        url::form_urlencoded::byte_serialize(mime_type.as_bytes()).collect::<String>()
    )
}

/// Build URL for uploading a file.
pub fn build_file_upload_url() -> String {
    "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart".to_string()
}

/// Build URL for copying a file.
pub fn build_file_copy_url(file_id: &str) -> String {
    format!("{}/files/{}/copy", DRIVE_BASE_URL, file_id)
}

/// Resolve the download output path.
/// If --out is given, use it. Otherwise use the file name.
/// For exports, replace the extension based on format.
pub fn resolve_download_path(
    filename: &str,
    out_flag: Option<&str>,
    export_mime: Option<&str>,
) -> String {
    if let Some(out) = out_flag {
        return out.to_string();
    }
    if let Some(mime) = export_mime {
        let ext = extension_for_mime(mime);
        if !ext.is_empty() {
            // If filename already has an extension, replace it; otherwise append
            if let Some(dot_pos) = filename.rfind('.') {
                return format!("{}{}", &filename[..dot_pos], ext);
            } else {
                return format!("{}{}", filename, ext);
            }
        }
    }
    filename.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-DRIVE-003 (Must): File get URL
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-003 (Must)
    // Acceptance: File get URL
    #[test]
    fn req_drive_003_file_get_url() {
        let url = build_file_get_url("file_abc");
        assert!(url.contains("files/file_abc"));
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-004 (Must): Download URL
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Binary download URL
    #[test]
    fn req_drive_004_file_download_url() {
        let url = build_file_download_url("file_abc");
        assert!(url.contains("files/file_abc"));
        assert!(url.contains("alt=media"));
    }

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Export URL for Google Workspace files
    #[test]
    fn req_drive_004_file_export_url() {
        let url = build_file_export_url("file_abc", "application/pdf");
        assert!(url.contains("files/file_abc/export"));
        assert!(url.contains("mimeType=application"));
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-005 (Must): Upload URL
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-005 (Must)
    // Acceptance: Upload URL
    #[test]
    fn req_drive_005_file_upload_url() {
        let url = build_file_upload_url();
        assert!(url.contains("upload"));
        assert!(url.contains("files"));
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-015 (Must): Copy URL
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-015 (Must)
    // Acceptance: Copy URL
    #[test]
    fn req_drive_015_file_copy_url() {
        let url = build_file_copy_url("file_abc");
        assert!(url.contains("files/file_abc/copy"));
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-004 (Must): Download path resolution
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Uses filename when no --out flag
    #[test]
    fn req_drive_004_resolve_path_default() {
        let path = resolve_download_path("document.pdf", None, None);
        assert_eq!(path, "document.pdf");
    }

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Uses --out flag when specified
    #[test]
    fn req_drive_004_resolve_path_out_flag() {
        let path = resolve_download_path("document.pdf", Some("/tmp/output.pdf"), None);
        assert_eq!(path, "/tmp/output.pdf");
    }

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Replaces extension for exported Google Workspace files
    #[test]
    fn req_drive_004_resolve_path_export_extension() {
        let path = resolve_download_path("My Document", None, Some("application/pdf"));
        assert!(path.ends_with(".pdf"));
    }
}
