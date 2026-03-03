//! Slides export, copy, and info operations via the Drive API.

use super::super::drive::types::DRIVE_BASE_URL;

/// Build URL for exporting a presentation via Drive API.
/// Uses Drive files.export endpoint with the resolved MIME type.
pub fn build_presentation_export_url(presentation_id: &str, format: &str) -> String {
    let mime = resolve_export_mime(format);
    format!(
        "{}/files/{}/export?mimeType={}",
        DRIVE_BASE_URL,
        presentation_id,
        url::form_urlencoded::byte_serialize(mime.as_bytes()).collect::<String>()
    )
}

/// Build URL for copying a presentation via Drive API.
pub fn build_presentation_copy_url(presentation_id: &str) -> String {
    format!("{}/files/{}/copy", DRIVE_BASE_URL, presentation_id)
}

/// Build the request body for copying a presentation.
pub fn build_presentation_copy_body(title: &str, parent: Option<&str>) -> serde_json::Value {
    let mut body = serde_json::json!({
        "name": title
    });
    if let Some(p) = parent {
        body["parents"] = serde_json::json!([p]);
    }
    body
}

/// Resolve a human-friendly format name to a MIME type for Slides export.
///
/// Supported formats:
/// - "pdf" -> application/pdf
/// - "pptx" -> application/vnd.openxmlformats-officedocument.presentationml.presentation
/// - "odp" -> application/vnd.oasis.opendocument.presentation
/// - "txt" -> text/plain
pub fn resolve_export_mime(format: &str) -> &'static str {
    match format.to_lowercase().as_str() {
        "pdf" => "application/pdf",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "odp" => "application/vnd.oasis.opendocument.presentation",
        "txt" | "text" => "text/plain",
        _ => "application/pdf",
    }
}

/// Build URL for getting presentation file info via Drive API.
pub fn build_presentation_info_url(presentation_id: &str) -> String {
    format!("{}/files/{}", DRIVE_BASE_URL, presentation_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-SLIDES-001 (Must): Export URL and mime mapping
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-001 (Must)
    // Acceptance: Export URL contains file ID and mimeType
    #[test]
    fn req_slides_001_export_url_pdf() {
        let url = build_presentation_export_url("pres_abc", "pdf");
        assert!(url.contains("files/pres_abc/export"));
        assert!(url.contains("mimeType=application%2Fpdf"));
    }

    // Requirement: REQ-SLIDES-001 (Must)
    // Acceptance: Export URL for pptx format
    #[test]
    fn req_slides_001_export_url_pptx() {
        let url = build_presentation_export_url("pres_abc", "pptx");
        assert!(url.contains("files/pres_abc/export"));
        assert!(url.contains("mimeType="));
        // PPTX MIME should be URL-encoded
        assert!(url.contains("presentationml"));
    }

    // Requirement: REQ-SLIDES-001 (Must)
    // Acceptance: resolve_export_mime returns correct MIME for pdf
    #[test]
    fn req_slides_001_resolve_mime_pdf() {
        assert_eq!(resolve_export_mime("pdf"), "application/pdf");
    }

    // Requirement: REQ-SLIDES-001 (Must)
    // Acceptance: resolve_export_mime returns correct MIME for pptx
    #[test]
    fn req_slides_001_resolve_mime_pptx() {
        assert_eq!(
            resolve_export_mime("pptx"),
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        );
    }

    // Requirement: REQ-SLIDES-001 (Must)
    // Acceptance: resolve_export_mime returns correct MIME for odp
    #[test]
    fn req_slides_001_resolve_mime_odp() {
        assert_eq!(
            resolve_export_mime("odp"),
            "application/vnd.oasis.opendocument.presentation"
        );
    }

    // Requirement: REQ-SLIDES-001 (Must)
    // Acceptance: resolve_export_mime returns correct MIME for txt
    #[test]
    fn req_slides_001_resolve_mime_txt() {
        assert_eq!(resolve_export_mime("txt"), "text/plain");
    }

    // Requirement: REQ-SLIDES-001 (Must)
    // Edge case: Unknown format defaults to PDF
    #[test]
    fn req_slides_001_resolve_mime_unknown() {
        assert_eq!(resolve_export_mime("xyz"), "application/pdf");
    }

    // Requirement: REQ-SLIDES-001 (Must)
    // Edge case: Case-insensitive format resolution
    #[test]
    fn req_slides_001_resolve_mime_case_insensitive() {
        assert_eq!(resolve_export_mime("PDF"), "application/pdf");
        assert_eq!(
            resolve_export_mime("Pptx"),
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        );
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-003 (Must): Create via Drive
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-003 (Must)
    // Acceptance: Info URL uses Drive files endpoint
    #[test]
    fn req_slides_003_info_url() {
        let url = build_presentation_info_url("pres_abc");
        assert!(url.contains("files/pres_abc"));
        assert!(!url.contains("copy"));
        assert!(!url.contains("export"));
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-005 (Must): Copy URL and body
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-005 (Must)
    // Acceptance: Copy URL uses Drive copy endpoint
    #[test]
    fn req_slides_005_copy_url() {
        let url = build_presentation_copy_url("pres_abc");
        assert!(url.contains("files/pres_abc/copy"));
    }

    // Requirement: REQ-SLIDES-005 (Must)
    // Acceptance: Copy body contains title
    #[test]
    fn req_slides_005_copy_body_title_only() {
        let body = build_presentation_copy_body("My Copy", None);
        assert_eq!(body["name"], "My Copy");
        assert!(body.get("parents").is_none());
    }

    // Requirement: REQ-SLIDES-005 (Must)
    // Acceptance: Copy body with parent folder
    #[test]
    fn req_slides_005_copy_body_with_parent() {
        let body = build_presentation_copy_body("My Copy", Some("folder123"));
        assert_eq!(body["name"], "My Copy");
        let parents = body["parents"].as_array().unwrap();
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0], "folder123");
    }
}
