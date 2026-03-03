//! Document export, copy, and create operations via the Drive API.

use crate::services::drive::types::DRIVE_BASE_URL;

/// Google Docs MIME type.
const DOCS_MIME_TYPE: &str = "application/vnd.google-apps.document";

/// Build URL for exporting a document in a given format via Drive API.
/// The `format` should be a MIME type string (e.g., "application/pdf").
pub fn build_doc_export_url(doc_id: &str, format: &str) -> String {
    format!(
        "{}/files/{}/export?mimeType={}",
        DRIVE_BASE_URL,
        doc_id,
        url::form_urlencoded::byte_serialize(format.as_bytes()).collect::<String>()
    )
}

/// Build URL for copying a document via Drive API.
pub fn build_doc_copy_url(doc_id: &str) -> String {
    format!("{}/files/{}/copy", DRIVE_BASE_URL, doc_id)
}

/// Build the request body for creating a new Google Doc via Drive API.
pub fn build_doc_create_body(title: &str, parent: Option<&str>) -> serde_json::Value {
    let mut body = serde_json::json!({
        "name": title,
        "mimeType": DOCS_MIME_TYPE
    });
    if let Some(parent_id) = parent {
        body["parents"] = serde_json::json!([parent_id]);
    }
    body
}

/// Build the request body for copying a document via Drive API.
pub fn build_doc_copy_body(title: &str, parent: Option<&str>) -> serde_json::Value {
    let mut body = serde_json::json!({
        "name": title
    });
    if let Some(parent_id) = parent {
        body["parents"] = serde_json::json!([parent_id]);
    }
    body
}

/// Resolve a human-readable format name to its MIME type.
/// Supports: pdf, docx, txt, html, epub, odt, rtf, zip.
pub fn resolve_export_mime(format: &str) -> &'static str {
    match format.to_lowercase().as_str() {
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "txt" => "text/plain",
        "html" => "text/html",
        "epub" => "application/epub+zip",
        "odt" => "application/vnd.oasis.opendocument.text",
        "rtf" => "application/rtf",
        "zip" => "application/zip",
        _ => "application/pdf",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-DOCS-001: Export URL construction and MIME type mapping
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-001 (Must)
    // Acceptance: Export URL contains document ID and MIME type
    #[test]
    fn req_docs_001_export_url_pdf() {
        let url = build_doc_export_url("doc123", "application/pdf");
        assert!(url.contains("files/doc123/export"));
        assert!(url.contains("mimeType=application"));
    }

    // Requirement: REQ-DOCS-001 (Must)
    // Acceptance: Export URL with docx MIME type
    #[test]
    fn req_docs_001_export_url_docx() {
        let url = build_doc_export_url(
            "doc123",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        );
        assert!(url.contains("files/doc123/export"));
        assert!(url.contains("mimeType="));
    }

    // Requirement: REQ-DOCS-001 (Must)
    // Acceptance: Export URL with plain text
    #[test]
    fn req_docs_001_export_url_txt() {
        let url = build_doc_export_url("doc123", "text/plain");
        assert!(url.contains("files/doc123/export"));
        assert!(url.contains("mimeType=text"));
    }

    // Requirement: REQ-DOCS-001 (Must)
    // Acceptance: resolve_export_mime maps pdf
    #[test]
    fn req_docs_001_resolve_mime_pdf() {
        assert_eq!(resolve_export_mime("pdf"), "application/pdf");
    }

    // Requirement: REQ-DOCS-001 (Must)
    // Acceptance: resolve_export_mime maps docx
    #[test]
    fn req_docs_001_resolve_mime_docx() {
        assert_eq!(
            resolve_export_mime("docx"),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        );
    }

    // Requirement: REQ-DOCS-001 (Must)
    // Acceptance: resolve_export_mime maps txt
    #[test]
    fn req_docs_001_resolve_mime_txt() {
        assert_eq!(resolve_export_mime("txt"), "text/plain");
    }

    // Requirement: REQ-DOCS-001 (Must)
    // Acceptance: resolve_export_mime maps html
    #[test]
    fn req_docs_001_resolve_mime_html() {
        assert_eq!(resolve_export_mime("html"), "text/html");
    }

    // Requirement: REQ-DOCS-001 (Must)
    // Acceptance: resolve_export_mime maps epub
    #[test]
    fn req_docs_001_resolve_mime_epub() {
        assert_eq!(resolve_export_mime("epub"), "application/epub+zip");
    }

    // Requirement: REQ-DOCS-001 (Must)
    // Acceptance: resolve_export_mime maps odt
    #[test]
    fn req_docs_001_resolve_mime_odt() {
        assert_eq!(
            resolve_export_mime("odt"),
            "application/vnd.oasis.opendocument.text"
        );
    }

    // Requirement: REQ-DOCS-001 (Must)
    // Acceptance: resolve_export_mime maps rtf
    #[test]
    fn req_docs_001_resolve_mime_rtf() {
        assert_eq!(resolve_export_mime("rtf"), "application/rtf");
    }

    // Requirement: REQ-DOCS-001 (Must)
    // Acceptance: resolve_export_mime maps zip
    #[test]
    fn req_docs_001_resolve_mime_zip() {
        assert_eq!(resolve_export_mime("zip"), "application/zip");
    }

    // Requirement: REQ-DOCS-001 (Must)
    // Acceptance: resolve_export_mime defaults to pdf for unknown
    #[test]
    fn req_docs_001_resolve_mime_unknown() {
        assert_eq!(resolve_export_mime("xyz"), "application/pdf");
    }

    // Requirement: REQ-DOCS-001 (Must)
    // Acceptance: resolve_export_mime is case-insensitive
    #[test]
    fn req_docs_001_resolve_mime_case_insensitive() {
        assert_eq!(resolve_export_mime("PDF"), "application/pdf");
        assert_eq!(
            resolve_export_mime("Docx"),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        );
    }

    // ---------------------------------------------------------------
    // REQ-DOCS-003: Create body construction
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-003 (Must)
    // Acceptance: Create body has name and mimeType
    #[test]
    fn req_docs_003_create_body_basic() {
        let body = build_doc_create_body("My New Doc", None);
        assert_eq!(body["name"], "My New Doc");
        assert_eq!(body["mimeType"], DOCS_MIME_TYPE);
        assert!(body.get("parents").is_none());
    }

    // Requirement: REQ-DOCS-003 (Must)
    // Acceptance: Create body with parent folder
    #[test]
    fn req_docs_003_create_body_with_parent() {
        let body = build_doc_create_body("My New Doc", Some("folder123"));
        assert_eq!(body["name"], "My New Doc");
        assert_eq!(body["mimeType"], DOCS_MIME_TYPE);
        assert_eq!(body["parents"][0], "folder123");
    }

    // ---------------------------------------------------------------
    // REQ-DOCS-004: Copy URL and body construction
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-004 (Must)
    // Acceptance: Copy URL contains document ID
    #[test]
    fn req_docs_004_copy_url() {
        let url = build_doc_copy_url("doc123");
        assert!(url.contains("files/doc123/copy"));
    }

    // Requirement: REQ-DOCS-004 (Must)
    // Acceptance: Copy body has title
    #[test]
    fn req_docs_004_copy_body_basic() {
        let body = build_doc_copy_body("Copy of Doc", None);
        assert_eq!(body["name"], "Copy of Doc");
        assert!(body.get("parents").is_none());
    }

    // Requirement: REQ-DOCS-004 (Must)
    // Acceptance: Copy body with parent folder
    #[test]
    fn req_docs_004_copy_body_with_parent() {
        let body = build_doc_copy_body("Copy of Doc", Some("folder456"));
        assert_eq!(body["name"], "Copy of Doc");
        assert_eq!(body["parents"][0], "folder456");
    }
}
