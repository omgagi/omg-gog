//! RT-M5 File I/O integration tests.
//!
//! Tests for:
//! - REQ-RT-026: Drive binary file download
//! - REQ-RT-027: Drive Workspace file export
//! - REQ-RT-028: Drive simple upload
//! - REQ-RT-030: Gmail attachment download
//! - REQ-RT-031: Shared export module

use omega_google::services::drive::files;
use omega_google::services::drive::types::*;
use omega_google::services::export;
use omega_google::services::gmail::message;

// ===================================================================
// REQ-RT-031 (Should): export module — format_to_mime
// ===================================================================

#[test]
fn req_rt_031_integration_format_to_mime_all_formats() {
    // PDF
    assert_eq!(export::format_to_mime("pdf"), Some("application/pdf"));
    // Word formats
    assert_eq!(
        export::format_to_mime("docx"),
        Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
    );
    assert_eq!(
        export::format_to_mime("doc"),
        export::format_to_mime("docx")
    );
    // Excel formats
    assert_eq!(
        export::format_to_mime("xlsx"),
        Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
    );
    assert_eq!(
        export::format_to_mime("xls"),
        export::format_to_mime("xlsx")
    );
    // PowerPoint formats
    assert_eq!(
        export::format_to_mime("pptx"),
        Some("application/vnd.openxmlformats-officedocument.presentationml.presentation")
    );
    assert_eq!(
        export::format_to_mime("ppt"),
        export::format_to_mime("pptx")
    );
    // Text formats
    assert_eq!(export::format_to_mime("csv"), Some("text/csv"));
    assert_eq!(export::format_to_mime("txt"), Some("text/plain"));
    assert_eq!(export::format_to_mime("text"), Some("text/plain"));
    // Image formats
    assert_eq!(export::format_to_mime("png"), Some("image/png"));
    assert_eq!(export::format_to_mime("svg"), Some("image/svg+xml"));
    // HTML
    assert_eq!(export::format_to_mime("html"), Some("text/html"));
    // Unknown
    assert_eq!(export::format_to_mime("avi"), None);
    assert_eq!(export::format_to_mime(""), None);
}

#[test]
fn req_rt_031_integration_format_to_mime_case_insensitive() {
    assert_eq!(export::format_to_mime("PDF"), Some("application/pdf"));
    assert_eq!(
        export::format_to_mime("Docx"),
        export::format_to_mime("docx")
    );
    assert_eq!(
        export::format_to_mime("XLSX"),
        export::format_to_mime("xlsx")
    );
}

// ===================================================================
// REQ-RT-031 (Should): export module — export_formats
// ===================================================================

#[test]
fn req_rt_031_integration_export_formats_google_doc() {
    let formats = export::export_formats(MIME_GOOGLE_DOC);
    assert_eq!(formats.len(), 4);
    let names: Vec<&str> = formats.iter().map(|(n, _, _)| *n).collect();
    assert!(names.contains(&"pdf"));
    assert!(names.contains(&"docx"));
    assert!(names.contains(&"txt"));
    assert!(names.contains(&"html"));
}

#[test]
fn req_rt_031_integration_export_formats_google_sheet() {
    let formats = export::export_formats(MIME_GOOGLE_SHEET);
    assert_eq!(formats.len(), 3);
    let names: Vec<&str> = formats.iter().map(|(n, _, _)| *n).collect();
    assert!(names.contains(&"pdf"));
    assert!(names.contains(&"xlsx"));
    assert!(names.contains(&"csv"));
}

#[test]
fn req_rt_031_integration_export_formats_google_slides() {
    let formats = export::export_formats(MIME_GOOGLE_SLIDES);
    assert_eq!(formats.len(), 3);
    let names: Vec<&str> = formats.iter().map(|(n, _, _)| *n).collect();
    assert!(names.contains(&"pdf"));
    assert!(names.contains(&"pptx"));
    assert!(names.contains(&"txt"));
}

#[test]
fn req_rt_031_integration_export_formats_google_drawing() {
    let formats = export::export_formats(MIME_GOOGLE_DRAWING);
    assert_eq!(formats.len(), 3);
    let names: Vec<&str> = formats.iter().map(|(n, _, _)| *n).collect();
    assert!(names.contains(&"pdf"));
    assert!(names.contains(&"png"));
    assert!(names.contains(&"svg"));
}

#[test]
fn req_rt_031_integration_export_formats_non_workspace() {
    assert!(export::export_formats("application/pdf").is_empty());
    assert!(export::export_formats("text/plain").is_empty());
    assert!(export::export_formats("").is_empty());
}

// ===================================================================
// REQ-RT-031 (Should): export module — is_google_workspace_type
// ===================================================================

#[test]
fn req_rt_031_integration_is_google_workspace_type() {
    assert!(export::is_google_workspace_type(MIME_GOOGLE_DOC));
    assert!(export::is_google_workspace_type(MIME_GOOGLE_SHEET));
    assert!(export::is_google_workspace_type(MIME_GOOGLE_SLIDES));
    assert!(export::is_google_workspace_type(MIME_GOOGLE_DRAWING));
    assert!(!export::is_google_workspace_type("application/pdf"));
    assert!(!export::is_google_workspace_type("text/plain"));
    assert!(!export::is_google_workspace_type(
        "application/octet-stream"
    ));
    assert!(!export::is_google_workspace_type(""));
}

// ===================================================================
// REQ-RT-031 (Should): export module — default_export_format
// ===================================================================

#[test]
fn req_rt_031_integration_default_export_format() {
    assert_eq!(export::default_export_format(MIME_GOOGLE_DOC), Some("pdf"));
    assert_eq!(
        export::default_export_format(MIME_GOOGLE_SHEET),
        Some("pdf")
    );
    assert_eq!(
        export::default_export_format(MIME_GOOGLE_SLIDES),
        Some("pdf")
    );
    assert_eq!(
        export::default_export_format(MIME_GOOGLE_DRAWING),
        Some("pdf")
    );
    assert_eq!(export::default_export_format("application/pdf"), None);
    assert_eq!(export::default_export_format(""), None);
}

// ===================================================================
// REQ-RT-026 (Must): Drive download URL builders
// ===================================================================

#[test]
fn req_rt_026_integration_download_url() {
    let url = files::build_file_download_url("file_xyz");
    assert!(url.starts_with("https://www.googleapis.com/drive/v3/files/file_xyz"));
    assert!(url.contains("alt=media"));
}

#[test]
fn req_rt_026_integration_download_url_special_chars() {
    let url = files::build_file_download_url("abc-123_DEF");
    assert!(url.contains("files/abc-123_DEF"));
    assert!(url.contains("alt=media"));
}

// ===================================================================
// REQ-RT-027 (Must): Drive export URL builders
// ===================================================================

#[test]
fn req_rt_027_integration_export_url_pdf() {
    let url = files::build_file_export_url("doc_id", "application/pdf");
    assert!(url.contains("files/doc_id/export"));
    assert!(url.contains("mimeType="));
    assert!(url.contains("application"));
}

#[test]
fn req_rt_027_integration_export_url_docx() {
    let mime = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
    let url = files::build_file_export_url("doc_id", mime);
    assert!(url.contains("files/doc_id/export"));
    assert!(url.contains("mimeType="));
}

#[test]
fn req_rt_027_integration_export_url_csv() {
    let url = files::build_file_export_url("sheet_id", "text/csv");
    assert!(url.contains("files/sheet_id/export"));
    assert!(url.contains("mimeType=text"));
}

// ===================================================================
// REQ-RT-028 (Must): Drive upload URL builder
// ===================================================================

#[test]
fn req_rt_028_integration_upload_url() {
    let url = files::build_file_upload_url();
    assert!(url.contains("upload"));
    assert!(url.contains("drive"));
    assert!(url.contains("files"));
    assert!(url.contains("uploadType=multipart"));
}

// ===================================================================
// REQ-RT-028 (Must): Multipart body construction
// ===================================================================

#[test]
fn req_rt_028_integration_multipart_body_construction() {
    let boundary = "omega_google_upload_boundary";
    let metadata = serde_json::json!({
        "name": "test.txt",
    });
    let metadata_json = serde_json::to_string(&metadata).unwrap();
    let file_data = b"Hello, world!";
    let content_type = "text/plain";

    let mut body = Vec::new();
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
    body.extend_from_slice(metadata_json.as_bytes());
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", content_type).as_bytes());
    body.extend_from_slice(file_data);
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    let body_str = String::from_utf8_lossy(&body);
    // Must contain boundary delimiters
    assert!(body_str.contains(&format!("--{}", boundary)));
    assert!(body_str.contains(&format!("--{}--", boundary)));
    // Must contain metadata
    assert!(body_str.contains("test.txt"));
    assert!(body_str.contains("application/json"));
    // Must contain file content
    assert!(body_str.contains("Hello, world!"));
    assert!(body_str.contains("text/plain"));
}

#[test]
fn req_rt_028_integration_multipart_body_with_parent() {
    let metadata = serde_json::json!({
        "name": "report.pdf",
        "parents": ["folder_abc"],
    });
    let metadata_json = serde_json::to_string(&metadata).unwrap();
    assert!(metadata_json.contains("folder_abc"));
    assert!(metadata_json.contains("parents"));
}

// ===================================================================
// REQ-RT-028 (Must): Content type guessing
// ===================================================================

#[test]
fn req_rt_028_integration_guess_content_type() {
    assert_eq!(
        export::guess_content_type_from_path("file.pdf"),
        "application/pdf"
    );
    assert_eq!(
        export::guess_content_type_from_path("photo.jpg"),
        "image/jpeg"
    );
    assert_eq!(
        export::guess_content_type_from_path("image.png"),
        "image/png"
    );
    assert_eq!(export::guess_content_type_from_path("data.csv"), "text/csv");
    assert_eq!(
        export::guess_content_type_from_path("config.json"),
        "application/json"
    );
    assert_eq!(
        export::guess_content_type_from_path("doc.docx"),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    );
    assert_eq!(
        export::guess_content_type_from_path("sheet.xlsx"),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    );
    assert_eq!(
        export::guess_content_type_from_path("slides.pptx"),
        "application/vnd.openxmlformats-officedocument.presentationml.presentation"
    );
    assert_eq!(
        export::guess_content_type_from_path("file.xyz"),
        "application/octet-stream"
    );
    assert_eq!(
        export::guess_content_type_from_path("Makefile"),
        "application/octet-stream"
    );
}

// ===================================================================
// REQ-RT-030 (Must): Gmail attachment URL builder
// ===================================================================

#[test]
fn req_rt_030_integration_attachment_url() {
    let url = message::build_attachment_url("msg_abc", "att_xyz");
    assert!(url.contains("messages/msg_abc/attachments/att_xyz"));
    assert!(url.contains("gmail"));
}

#[test]
fn req_rt_030_integration_attachment_url_format() {
    let url = message::build_attachment_url("m1", "a1");
    // Should follow Gmail API v1 format
    assert!(url.starts_with("https://"));
    assert!(url.contains("users/me/messages/m1/attachments/a1"));
}

// ===================================================================
// REQ-RT-030 (Must): Base64url decode
// ===================================================================

#[test]
fn req_rt_030_integration_base64url_decode() {
    use base64::Engine;
    // "Hello, World!" in base64url encoding (no padding)
    let encoded = "SGVsbG8sIFdvcmxkIQ";
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .unwrap();
    assert_eq!(decoded, b"Hello, World!");
}

#[test]
fn req_rt_030_integration_base64url_decode_binary() {
    use base64::Engine;
    // Binary data with URL-safe chars
    let original = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10]; // JPEG header bytes
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&original);
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&encoded)
        .unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn req_rt_030_integration_base64url_decode_empty() {
    use base64::Engine;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode("")
        .unwrap();
    assert!(decoded.is_empty());
}

// ===================================================================
// REQ-RT-026/027 (Must): Download path resolution
// ===================================================================

#[test]
fn req_rt_026_integration_resolve_download_path_binary() {
    // Binary file: use original filename
    let path = files::resolve_download_path("report.pdf", None, None);
    assert_eq!(path, "report.pdf");
}

#[test]
fn req_rt_026_integration_resolve_download_path_out_flag() {
    // --out flag overrides everything
    let path = files::resolve_download_path("report.pdf", Some("/tmp/out.pdf"), None);
    assert_eq!(path, "/tmp/out.pdf");
}

#[test]
fn req_rt_027_integration_resolve_download_path_export() {
    // Export: replace extension with format-specific extension
    let path = files::resolve_download_path("My Document", None, Some("application/pdf"));
    assert!(
        path.ends_with(".pdf"),
        "Expected .pdf extension, got: {}",
        path
    );
}

#[test]
fn req_rt_027_integration_resolve_download_path_export_replaces_extension() {
    // Export of a named file replaces existing extension
    let path = files::resolve_download_path("spreadsheet.gsheet", None, Some("text/csv"));
    assert!(
        path.ends_with(".csv"),
        "Expected .csv extension, got: {}",
        path
    );
}

// ===================================================================
// REQ-RT-031 (Should): Export + Drive types integration
// ===================================================================

#[test]
fn req_rt_031_integration_workspace_type_detection_matches_drive_types() {
    // Verify that the export module's workspace detection matches drive types module
    assert_eq!(
        export::is_google_workspace_type(MIME_GOOGLE_DOC),
        is_google_workspace_type(MIME_GOOGLE_DOC)
    );
    assert_eq!(
        export::is_google_workspace_type(MIME_GOOGLE_SHEET),
        is_google_workspace_type(MIME_GOOGLE_SHEET)
    );
    assert_eq!(
        export::is_google_workspace_type(MIME_GOOGLE_SLIDES),
        is_google_workspace_type(MIME_GOOGLE_SLIDES)
    );
    assert_eq!(
        export::is_google_workspace_type(MIME_GOOGLE_DRAWING),
        is_google_workspace_type(MIME_GOOGLE_DRAWING)
    );
    assert_eq!(
        export::is_google_workspace_type("application/pdf"),
        is_google_workspace_type("application/pdf")
    );
}

#[test]
fn req_rt_031_integration_export_format_mime_matches_drive_constants() {
    // Verify format_to_mime returns values consistent with drive type constants
    assert_eq!(export::format_to_mime("pdf"), Some(MIME_PDF));
    assert_eq!(export::format_to_mime("csv"), Some(MIME_CSV));
    assert_eq!(export::format_to_mime("docx"), Some(MIME_DOCX));
    assert_eq!(export::format_to_mime("xlsx"), Some(MIME_XLSX));
    assert_eq!(export::format_to_mime("pptx"), Some(MIME_PPTX));
    assert_eq!(export::format_to_mime("txt"), Some(MIME_TEXT_PLAIN));
    assert_eq!(export::format_to_mime("png"), Some(MIME_PNG));
}

// ===================================================================
// REQ-RT-027 (Must): Export flow — format_to_mime + build_file_export_url
// ===================================================================

#[test]
fn req_rt_027_integration_export_flow_doc_to_pdf() {
    let format = "pdf";
    let mime = export::format_to_mime(format).unwrap();
    let url = files::build_file_export_url("doc_123", mime);
    assert!(url.contains("files/doc_123/export"));
    assert!(url.contains("mimeType="));
}

#[test]
fn req_rt_027_integration_export_flow_sheet_to_csv() {
    let format = "csv";
    let mime = export::format_to_mime(format).unwrap();
    let url = files::build_file_export_url("sheet_456", mime);
    assert!(url.contains("files/sheet_456/export"));
    assert!(url.contains("mimeType=text"));
}

#[test]
fn req_rt_027_integration_export_flow_slides_to_pptx() {
    let format = "pptx";
    let mime = export::format_to_mime(format).unwrap();
    let url = files::build_file_export_url("slides_789", mime);
    assert!(url.contains("files/slides_789/export"));
}

#[test]
fn req_rt_027_integration_export_flow_drawing_to_svg() {
    let format = "svg";
    let mime = export::format_to_mime(format).unwrap();
    let url = files::build_file_export_url("draw_abc", mime);
    assert!(url.contains("files/draw_abc/export"));
    assert!(url.contains("mimeType=image"));
}

// ===================================================================
// REQ-RT-026 (Must): File metadata URL
// ===================================================================

#[test]
fn req_rt_026_integration_file_get_url() {
    let url = files::build_file_get_url("fileXYZ");
    assert!(url.contains("files/fileXYZ"));
    assert!(url.starts_with("https://"));
}
