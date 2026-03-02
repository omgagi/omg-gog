//! Drive API request/response types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Drive API base URL.
pub const DRIVE_BASE_URL: &str = "https://www.googleapis.com/drive/v3";

// ---------------------------------------------------------------
// Google Workspace MIME types
// ---------------------------------------------------------------

pub const MIME_GOOGLE_DOC: &str = "application/vnd.google-apps.document";
pub const MIME_GOOGLE_SHEET: &str = "application/vnd.google-apps.spreadsheet";
pub const MIME_GOOGLE_SLIDES: &str = "application/vnd.google-apps.presentation";
pub const MIME_GOOGLE_DRAWING: &str = "application/vnd.google-apps.drawing";
pub const MIME_GOOGLE_FOLDER: &str = "application/vnd.google-apps.folder";

pub const MIME_PDF: &str = "application/pdf";
pub const MIME_CSV: &str = "text/csv";
pub const MIME_DOCX: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
pub const MIME_XLSX: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
pub const MIME_PPTX: &str = "application/vnd.openxmlformats-officedocument.presentationml.presentation";
pub const MIME_PNG: &str = "image/png";
pub const MIME_TEXT_PLAIN: &str = "text/plain";

// ---------------------------------------------------------------
// File types
// ---------------------------------------------------------------

/// Drive file list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileListResponse {
    #[serde(default)]
    pub files: Vec<DriveFile>,
    pub next_page_token: Option<String>,
    pub incomplete_search: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A Drive file/folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFile {
    pub id: Option<String>,
    pub name: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<String>,
    pub modified_time: Option<String>,
    pub created_time: Option<String>,
    #[serde(default)]
    pub parents: Vec<String>,
    pub web_view_link: Option<String>,
    pub description: Option<String>,
    pub starred: Option<bool>,
    pub trashed: Option<bool>,
    pub file_extension: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Permission types
// ---------------------------------------------------------------

/// Permission list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionListResponse {
    #[serde(default)]
    pub permissions: Vec<Permission>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A Drive permission.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permission {
    pub id: Option<String>,
    pub r#type: Option<String>,
    pub role: Option<String>,
    pub email_address: Option<String>,
    pub domain: Option<String>,
    pub display_name: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Comment types
// ---------------------------------------------------------------

/// Comment list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentListResponse {
    #[serde(default)]
    pub comments: Vec<Comment>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A Drive file comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: Option<String>,
    pub content: Option<String>,
    pub author: Option<CommentAuthor>,
    pub created_time: Option<String>,
    pub modified_time: Option<String>,
    pub resolved: Option<bool>,
    pub quoted_file_content: Option<QuotedContent>,
    #[serde(default)]
    pub replies: Vec<CommentReply>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A comment author.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentAuthor {
    pub display_name: Option<String>,
    pub email_address: Option<String>,
}

/// Quoted content from the file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotedContent {
    pub mime_type: Option<String>,
    pub value: Option<String>,
}

/// A reply to a comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentReply {
    pub id: Option<String>,
    pub content: Option<String>,
    pub author: Option<CommentAuthor>,
    pub created_time: Option<String>,
    pub modified_time: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Shared drive types
// ---------------------------------------------------------------

/// Shared drives list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedDriveListResponse {
    #[serde(default)]
    pub drives: Vec<SharedDrive>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A shared drive (Team Drive).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedDrive {
    pub id: Option<String>,
    pub name: Option<String>,
    pub created_time: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------

/// Determine the display type from a MIME type.
/// Google Workspace MIME types get friendly names; others return "file".
pub fn drive_type(mime_type: &str) -> &str {
    if mime_type == MIME_GOOGLE_FOLDER {
        "folder"
    } else {
        "file"
    }
}

/// Guess the MIME type from a file path extension.
pub fn guess_mime_type(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("");
    // Only treat it as an extension if there was actually a dot
    if !path.contains('.') {
        return "application/octet-stream";
    }
    match ext.to_lowercase().as_str() {
        "pdf" => MIME_PDF,
        "docx" => MIME_DOCX,
        "xlsx" => MIME_XLSX,
        "pptx" => MIME_PPTX,
        "csv" => MIME_CSV,
        "txt" => MIME_TEXT_PLAIN,
        "png" => MIME_PNG,
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "json" => "application/json",
        "html" | "htm" => "text/html",
        "xml" => "application/xml",
        "zip" => "application/zip",
        "mp4" => "video/mp4",
        "mp3" => "audio/mpeg",
        _ => "application/octet-stream",
    }
}

/// Get the default export MIME type for a Google Workspace file.
pub fn default_export_mime(google_mime_type: &str) -> &'static str {
    match google_mime_type {
        MIME_GOOGLE_DOC => MIME_PDF,
        MIME_GOOGLE_SHEET => MIME_CSV,
        MIME_GOOGLE_SLIDES => MIME_PDF,
        MIME_GOOGLE_DRAWING => MIME_PNG,
        _ => MIME_PDF,
    }
}

/// Get the export MIME type for a specific format flag (e.g., "pdf", "docx").
pub fn export_mime_for_format(google_mime_type: &str, format: &str) -> anyhow::Result<&'static str> {
    match (google_mime_type, format) {
        // Google Docs exports
        (MIME_GOOGLE_DOC, "pdf") => Ok(MIME_PDF),
        (MIME_GOOGLE_DOC, "docx") => Ok(MIME_DOCX),
        (MIME_GOOGLE_DOC, "txt") => Ok(MIME_TEXT_PLAIN),
        (MIME_GOOGLE_DOC, "html") => Ok("text/html"),
        // Google Sheets exports
        (MIME_GOOGLE_SHEET, "csv") => Ok(MIME_CSV),
        (MIME_GOOGLE_SHEET, "xlsx") => Ok(MIME_XLSX),
        (MIME_GOOGLE_SHEET, "pdf") => Ok(MIME_PDF),
        // Google Slides exports
        (MIME_GOOGLE_SLIDES, "pdf") => Ok(MIME_PDF),
        (MIME_GOOGLE_SLIDES, "pptx") => Ok(MIME_PPTX),
        // Google Drawing exports
        (MIME_GOOGLE_DRAWING, "png") => Ok(MIME_PNG),
        (MIME_GOOGLE_DRAWING, "pdf") => Ok(MIME_PDF),
        (MIME_GOOGLE_DRAWING, "svg") => Ok("image/svg+xml"),
        _ => anyhow::bail!(
            "unsupported export format '{}' for MIME type '{}'",
            format,
            google_mime_type
        ),
    }
}

/// Generate a Drive web URL for a file.
pub fn file_url(file_id: &str) -> String {
    format!("https://drive.google.com/open?id={}", file_id)
}

/// Determine the file extension for a given export MIME type.
pub fn extension_for_mime(mime_type: &str) -> &'static str {
    match mime_type {
        MIME_PDF => ".pdf",
        MIME_DOCX => ".docx",
        MIME_XLSX => ".xlsx",
        MIME_PPTX => ".pptx",
        MIME_CSV => ".csv",
        MIME_TEXT_PLAIN => ".txt",
        MIME_PNG => ".png",
        "image/jpeg" => ".jpg",
        "image/svg+xml" => ".svg",
        "text/html" => ".html",
        "application/json" => ".json",
        _ => "",
    }
}

/// Determine the Google Workspace MIME type for an upload --convert-to target.
pub fn convert_to_mime(target: &str) -> anyhow::Result<&'static str> {
    match target {
        "doc" | "document" => Ok(MIME_GOOGLE_DOC),
        "sheet" | "spreadsheet" => Ok(MIME_GOOGLE_SHEET),
        "slides" | "presentation" => Ok(MIME_GOOGLE_SLIDES),
        "drawing" => Ok(MIME_GOOGLE_DRAWING),
        _ => anyhow::bail!("unsupported conversion target: '{}'", target),
    }
}

/// Check if a MIME type is a Google Workspace native type (requires export, not download).
pub fn is_google_workspace_type(mime_type: &str) -> bool {
    matches!(
        mime_type,
        MIME_GOOGLE_DOC | MIME_GOOGLE_SHEET | MIME_GOOGLE_SLIDES | MIME_GOOGLE_DRAWING
    )
}

/// Strip Office-format extensions (.docx, .xlsx, .pptx) from a filename
/// when the file has been converted to Google Workspace format.
pub fn strip_office_extension(filename: &str) -> &str {
    for ext in &[".docx", ".xlsx", ".pptx"] {
        if let Some(stripped) = filename.strip_suffix(ext) {
            return stripped;
        }
    }
    filename
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-DRIVE-001 (Must): DriveFile type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: DriveFile deserializes from API JSON
    #[test]
    fn req_drive_001_file_deserialize() {
        let json_str = r#"{
            "id": "file123",
            "name": "document.pdf",
            "mimeType": "application/pdf",
            "size": "12345",
            "modifiedTime": "2024-01-15T14:30:00.000Z",
            "parents": ["folder_abc"],
            "webViewLink": "https://drive.google.com/file/d/file123/view"
        }"#;
        let file: DriveFile = serde_json::from_str(json_str).unwrap();
        assert_eq!(file.id, Some("file123".to_string()));
        assert_eq!(file.name, Some("document.pdf".to_string()));
        assert_eq!(file.mime_type, Some("application/pdf".to_string()));
        assert_eq!(file.size, Some("12345".to_string()));
        assert_eq!(file.parents, vec!["folder_abc"]);
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: FileListResponse round-trip
    #[test]
    fn req_drive_001_file_list_response_roundtrip() {
        let resp = FileListResponse {
            files: vec![DriveFile {
                id: Some("f1".to_string()),
                name: Some("test.txt".to_string()),
                mime_type: Some("text/plain".to_string()),
                size: Some("100".to_string()),
                modified_time: None,
                created_time: None,
                parents: vec![],
                web_view_link: None,
                description: None,
                starred: None,
                trashed: None,
                file_extension: None,
                extra: HashMap::new(),
            }],
            next_page_token: Some("next_page".to_string()),
            incomplete_search: None,
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: FileListResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.files.len(), 1);
        assert_eq!(parsed.next_page_token, Some("next_page".to_string()));
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Edge case: Empty file list
    #[test]
    fn req_drive_001_file_list_response_empty() {
        let json_str = r#"{"files": []}"#;
        let resp: FileListResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.files.is_empty());
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Edge case: Unknown fields preserved
    #[test]
    fn req_drive_001_file_unknown_fields() {
        let json_str = r#"{"id": "f1", "newField": "preserved"}"#;
        let file: DriveFile = serde_json::from_str(json_str).unwrap();
        assert!(file.extra.contains_key("newField"));
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-004 (Must): Export MIME type mapping
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Google Doc default export is PDF
    #[test]
    fn req_drive_004_default_export_doc() {
        assert_eq!(default_export_mime(MIME_GOOGLE_DOC), MIME_PDF);
    }

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Google Sheet default export is CSV
    #[test]
    fn req_drive_004_default_export_sheet() {
        assert_eq!(default_export_mime(MIME_GOOGLE_SHEET), MIME_CSV);
    }

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Google Slides default export is PDF
    #[test]
    fn req_drive_004_default_export_slides() {
        assert_eq!(default_export_mime(MIME_GOOGLE_SLIDES), MIME_PDF);
    }

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Google Drawing export is PNG
    #[test]
    fn req_drive_004_default_export_drawing() {
        assert_eq!(default_export_mime(MIME_GOOGLE_DRAWING), MIME_PNG);
    }

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Export with format flag - Doc to DOCX
    #[test]
    fn req_drive_004_export_doc_to_docx() {
        let mime = export_mime_for_format(MIME_GOOGLE_DOC, "docx").unwrap();
        assert_eq!(mime, MIME_DOCX);
    }

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Export with format flag - Sheet to XLSX
    #[test]
    fn req_drive_004_export_sheet_to_xlsx() {
        let mime = export_mime_for_format(MIME_GOOGLE_SHEET, "xlsx").unwrap();
        assert_eq!(mime, MIME_XLSX);
    }

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Export with format flag - Slides to PPTX
    #[test]
    fn req_drive_004_export_slides_to_pptx() {
        let mime = export_mime_for_format(MIME_GOOGLE_SLIDES, "pptx").unwrap();
        assert_eq!(mime, MIME_PPTX);
    }

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Invalid format flag returns error
    #[test]
    fn req_drive_004_export_invalid_format() {
        assert!(export_mime_for_format(MIME_GOOGLE_DOC, "xlsx").is_err());
    }

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Export with format flag - Doc to TXT
    #[test]
    fn req_drive_004_export_doc_to_txt() {
        let mime = export_mime_for_format(MIME_GOOGLE_DOC, "txt").unwrap();
        assert_eq!(mime, MIME_TEXT_PLAIN);
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-005 (Must): MIME type guessing
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-005 (Must)
    // Acceptance: Common file extensions mapped correctly
    #[test]
    fn req_drive_005_guess_mime_pdf() {
        assert_eq!(guess_mime_type("report.pdf"), MIME_PDF);
    }

    #[test]
    fn req_drive_005_guess_mime_docx() {
        assert_eq!(guess_mime_type("document.docx"), MIME_DOCX);
    }

    #[test]
    fn req_drive_005_guess_mime_xlsx() {
        assert_eq!(guess_mime_type("spreadsheet.xlsx"), MIME_XLSX);
    }

    #[test]
    fn req_drive_005_guess_mime_png() {
        assert_eq!(guess_mime_type("image.png"), MIME_PNG);
    }

    #[test]
    fn req_drive_005_guess_mime_txt() {
        assert_eq!(guess_mime_type("notes.txt"), MIME_TEXT_PLAIN);
    }

    #[test]
    fn req_drive_005_guess_mime_csv() {
        assert_eq!(guess_mime_type("data.csv"), MIME_CSV);
    }

    #[test]
    fn req_drive_005_guess_mime_json() {
        assert_eq!(guess_mime_type("config.json"), "application/json");
    }

    #[test]
    fn req_drive_005_guess_mime_unknown() {
        assert_eq!(guess_mime_type("file.xyz"), "application/octet-stream");
    }

    #[test]
    fn req_drive_005_guess_mime_no_extension() {
        assert_eq!(guess_mime_type("Makefile"), "application/octet-stream");
    }

    #[test]
    fn req_drive_005_guess_mime_jpeg() {
        assert_eq!(guess_mime_type("photo.jpg"), "image/jpeg");
        assert_eq!(guess_mime_type("photo.jpeg"), "image/jpeg");
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-001 (Must): drive_type display
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: Google folder shows as "folder"
    #[test]
    fn req_drive_001_drive_type_folder() {
        assert_eq!(drive_type(MIME_GOOGLE_FOLDER), "folder");
    }

    // Requirement: REQ-DRIVE-001 (Must)
    // Acceptance: Regular files show as "file"
    #[test]
    fn req_drive_001_drive_type_file() {
        assert_eq!(drive_type("application/pdf"), "file");
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-013 (Must): URL generation
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-013 (Must)
    // Acceptance: File URL format
    #[test]
    fn req_drive_013_file_url() {
        assert_eq!(
            file_url("file_abc123"),
            "https://drive.google.com/open?id=file_abc123"
        );
    }

    // Requirement: REQ-DRIVE-013 (Must)
    // Edge case: Empty file ID
    #[test]
    fn req_drive_013_file_url_empty() {
        assert_eq!(
            file_url(""),
            "https://drive.google.com/open?id="
        );
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-005 (Must): convert-to MIME mapping
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-005 (Must)
    // Acceptance: Convert-to doc mapping
    #[test]
    fn req_drive_005_convert_to_doc() {
        assert_eq!(convert_to_mime("doc").unwrap(), MIME_GOOGLE_DOC);
    }

    // Requirement: REQ-DRIVE-005 (Must)
    // Acceptance: Convert-to sheet mapping
    #[test]
    fn req_drive_005_convert_to_sheet() {
        assert_eq!(convert_to_mime("sheet").unwrap(), MIME_GOOGLE_SHEET);
    }

    // Requirement: REQ-DRIVE-005 (Must)
    // Acceptance: Convert-to slides mapping
    #[test]
    fn req_drive_005_convert_to_slides() {
        assert_eq!(convert_to_mime("slides").unwrap(), MIME_GOOGLE_SLIDES);
    }

    // Requirement: REQ-DRIVE-005 (Must)
    // Acceptance: Invalid convert-to target
    #[test]
    fn req_drive_005_convert_to_invalid() {
        assert!(convert_to_mime("invalid").is_err());
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-004 (Must): Google Workspace type detection
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Google Docs types detected
    #[test]
    fn req_drive_004_is_google_workspace_type() {
        assert!(is_google_workspace_type(MIME_GOOGLE_DOC));
        assert!(is_google_workspace_type(MIME_GOOGLE_SHEET));
        assert!(is_google_workspace_type(MIME_GOOGLE_SLIDES));
        assert!(is_google_workspace_type(MIME_GOOGLE_DRAWING));
        assert!(!is_google_workspace_type(MIME_PDF));
        assert!(!is_google_workspace_type("text/plain"));
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-005 (Must): Office extension stripping
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-005 (Must)
    // Acceptance: Strips .docx extension
    #[test]
    fn req_drive_005_strip_docx_extension() {
        assert_eq!(strip_office_extension("document.docx"), "document");
    }

    // Requirement: REQ-DRIVE-005 (Must)
    // Acceptance: Strips .xlsx extension
    #[test]
    fn req_drive_005_strip_xlsx_extension() {
        assert_eq!(strip_office_extension("spreadsheet.xlsx"), "spreadsheet");
    }

    // Requirement: REQ-DRIVE-005 (Must)
    // Acceptance: Strips .pptx extension
    #[test]
    fn req_drive_005_strip_pptx_extension() {
        assert_eq!(strip_office_extension("presentation.pptx"), "presentation");
    }

    // Requirement: REQ-DRIVE-005 (Must)
    // Acceptance: Does not strip non-Office extensions
    #[test]
    fn req_drive_005_strip_non_office_extension() {
        assert_eq!(strip_office_extension("image.png"), "image.png");
        assert_eq!(strip_office_extension("notes.txt"), "notes.txt");
    }

    // Requirement: REQ-DRIVE-005 (Must)
    // Edge case: No extension
    #[test]
    fn req_drive_005_strip_no_extension() {
        assert_eq!(strip_office_extension("README"), "README");
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-004 (Must): Extension for MIME type
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-004 (Must)
    // Acceptance: Correct extensions for export MIME types
    #[test]
    fn req_drive_004_extension_for_mime() {
        assert_eq!(extension_for_mime(MIME_PDF), ".pdf");
        assert_eq!(extension_for_mime(MIME_DOCX), ".docx");
        assert_eq!(extension_for_mime(MIME_XLSX), ".xlsx");
        assert_eq!(extension_for_mime(MIME_PPTX), ".pptx");
        assert_eq!(extension_for_mime(MIME_CSV), ".csv");
        assert_eq!(extension_for_mime(MIME_TEXT_PLAIN), ".txt");
        assert_eq!(extension_for_mime(MIME_PNG), ".png");
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-010/011 (Must): Permission types
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-010 (Must)
    // Acceptance: Permission deserializes
    #[test]
    fn req_drive_010_permission_deserialize() {
        let json_str = r#"{
            "id": "perm123",
            "type": "user",
            "role": "writer",
            "emailAddress": "user@example.com",
            "displayName": "User"
        }"#;
        let perm: Permission = serde_json::from_str(json_str).unwrap();
        assert_eq!(perm.id, Some("perm123".to_string()));
        assert_eq!(perm.r#type, Some("user".to_string()));
        assert_eq!(perm.role, Some("writer".to_string()));
    }

    // Requirement: REQ-DRIVE-011 (Must)
    // Acceptance: PermissionListResponse deserializes
    #[test]
    fn req_drive_011_permission_list_response() {
        let json_str = r#"{
            "permissions": [
                {"id": "p1", "type": "user", "role": "reader"},
                {"id": "p2", "type": "anyone", "role": "reader"}
            ]
        }"#;
        let resp: PermissionListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.permissions.len(), 2);
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-016 (Should): Comment types
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-016 (Should)
    // Acceptance: Comment deserializes
    #[test]
    fn req_drive_016_comment_deserialize() {
        let json_str = r#"{
            "id": "comment1",
            "content": "This is a comment",
            "author": {"displayName": "Alice", "emailAddress": "alice@example.com"},
            "createdTime": "2024-01-15T14:30:00.000Z",
            "resolved": false,
            "replies": []
        }"#;
        let comment: Comment = serde_json::from_str(json_str).unwrap();
        assert_eq!(comment.id, Some("comment1".to_string()));
        assert_eq!(comment.content, Some("This is a comment".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-DRIVE-014 (Must): SharedDrive types
    // ---------------------------------------------------------------

    // Requirement: REQ-DRIVE-014 (Must)
    // Acceptance: SharedDrive deserializes
    #[test]
    fn req_drive_014_shared_drive_deserialize() {
        let json_str = r#"{
            "id": "drive123",
            "name": "Team Drive",
            "createdTime": "2024-01-01T00:00:00.000Z"
        }"#;
        let drive: SharedDrive = serde_json::from_str(json_str).unwrap();
        assert_eq!(drive.id, Some("drive123".to_string()));
        assert_eq!(drive.name, Some("Team Drive".to_string()));
    }
}
