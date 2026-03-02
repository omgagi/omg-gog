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

/// Build URL for uploading a file (simple multipart).
pub fn build_file_upload_url() -> String {
    "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart".to_string()
}

/// Build URL for initiating a resumable upload session.
pub fn build_resumable_upload_url() -> String {
    "https://www.googleapis.com/upload/drive/v3/files?uploadType=resumable".to_string()
}

/// Threshold in bytes above which resumable upload is used instead of simple multipart.
/// Files larger than 5 MB use the resumable protocol (REQ-RT-029).
pub const RESUMABLE_THRESHOLD: u64 = 5 * 1024 * 1024;

/// Default chunk size for resumable uploads (256 KB).
pub const RESUMABLE_CHUNK_SIZE: usize = 256 * 1024;

/// Build URL for copying a file.
pub fn build_file_copy_url(file_id: &str) -> String {
    format!("{}/files/{}/copy", DRIVE_BASE_URL, file_id)
}

/// Sanitize an API-provided filename by extracting only the base name.
/// Strips directory separators and path traversal sequences (e.g., `..`, leading `/`).
fn sanitize_filename(name: &str) -> &str {
    std::path::Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download")
}

/// Resolve the download output path.
/// If --out is given, use it. Otherwise use the file name (sanitized to prevent
/// path traversal). For exports, replace the extension based on format.
pub fn resolve_download_path(
    filename: &str,
    out_flag: Option<&str>,
    export_mime: Option<&str>,
) -> String {
    if let Some(out) = out_flag {
        return out.to_string();
    }
    // Sanitize the API-provided filename to prevent path traversal
    let safe_name = sanitize_filename(filename);
    if let Some(mime) = export_mime {
        let ext = extension_for_mime(mime);
        if !ext.is_empty() {
            // If filename already has an extension, replace it; otherwise append
            if let Some(dot_pos) = safe_name.rfind('.') {
                return format!("{}{}", &safe_name[..dot_pos], ext);
            } else {
                return format!("{}{}", safe_name, ext);
            }
        }
    }
    safe_name.to_string()
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
    // REQ-RT-029 (Should): Resumable upload URL
    // ---------------------------------------------------------------

    // Requirement: REQ-RT-029 (Should)
    // Acceptance: Resumable upload URL uses uploadType=resumable
    #[test]
    fn req_rt_029_resumable_upload_url() {
        let url = build_resumable_upload_url();
        assert!(url.contains("uploadType=resumable"));
        assert!(url.contains("upload/drive/v3/files"));
    }

    // Requirement: REQ-RT-029 (Should)
    // Acceptance: RESUMABLE_THRESHOLD is 5MB
    #[test]
    fn req_rt_029_resumable_threshold_is_5mb() {
        assert_eq!(RESUMABLE_THRESHOLD, 5 * 1024 * 1024);
    }

    // Requirement: REQ-RT-029 (Should)
    // Acceptance: RESUMABLE_CHUNK_SIZE is 256KB
    #[test]
    fn req_rt_029_resumable_chunk_size() {
        assert_eq!(RESUMABLE_CHUNK_SIZE, 256 * 1024);
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

    // ---------------------------------------------------------------
    // Path traversal sanitization
    // ---------------------------------------------------------------

    #[test]
    fn resolve_path_sanitizes_traversal() {
        let path = resolve_download_path("../../etc/evil", None, None);
        assert_eq!(path, "evil");
    }

    #[test]
    fn resolve_path_sanitizes_absolute() {
        let path = resolve_download_path("/etc/passwd", None, None);
        assert_eq!(path, "passwd");
    }

    #[test]
    fn resolve_path_sanitizes_directory_components() {
        let path = resolve_download_path("subdir/file.txt", None, None);
        assert_eq!(path, "file.txt");
    }

    #[test]
    fn resolve_path_sanitizes_dotdot_only() {
        // ".." has no file_name component, should fallback to "download"
        let path = resolve_download_path("..", None, None);
        assert_eq!(path, "download");
    }

    #[test]
    fn resolve_path_sanitizes_with_export() {
        let path = resolve_download_path("../../evil.txt", None, Some("application/pdf"));
        assert!(path.ends_with(".pdf"));
        assert!(!path.contains(".."));
    }
}
