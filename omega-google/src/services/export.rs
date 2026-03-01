//! Shared export logic for Google Workspace documents.
//!
//! Provides format-to-MIME mapping, supported export formats per Google type,
//! workspace type detection, and default export format resolution.

use crate::services::drive::types::*;

/// Map a format string to a MIME type for export.
pub fn format_to_mime(format: &str) -> Option<&'static str> {
    match format.to_lowercase().as_str() {
        "pdf" => Some("application/pdf"),
        "docx" | "doc" => Some(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ),
        "xlsx" | "xls" => Some(
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ),
        "pptx" | "ppt" => Some(
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        ),
        "csv" => Some("text/csv"),
        "txt" | "text" => Some("text/plain"),
        "png" => Some("image/png"),
        "svg" => Some("image/svg+xml"),
        "html" => Some("text/html"),
        _ => None,
    }
}

/// Supported export formats for each Google Workspace MIME type.
/// Returns vec of (format_name, mime_type, file_extension).
pub fn export_formats(google_mime: &str) -> Vec<(&'static str, &'static str, &'static str)> {
    match google_mime {
        MIME_GOOGLE_DOC => vec![
            ("pdf", "application/pdf", ".pdf"),
            (
                "docx",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                ".docx",
            ),
            ("txt", "text/plain", ".txt"),
            ("html", "text/html", ".html"),
        ],
        MIME_GOOGLE_SHEET => vec![
            ("pdf", "application/pdf", ".pdf"),
            (
                "xlsx",
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                ".xlsx",
            ),
            ("csv", "text/csv", ".csv"),
        ],
        MIME_GOOGLE_SLIDES => vec![
            ("pdf", "application/pdf", ".pdf"),
            (
                "pptx",
                "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                ".pptx",
            ),
            ("txt", "text/plain", ".txt"),
        ],
        MIME_GOOGLE_DRAWING => vec![
            ("pdf", "application/pdf", ".pdf"),
            ("png", "image/png", ".png"),
            ("svg", "image/svg+xml", ".svg"),
        ],
        _ => vec![],
    }
}

/// Check if a MIME type is a Google Workspace type that needs export.
pub fn is_google_workspace_type(mime_type: &str) -> bool {
    matches!(
        mime_type,
        MIME_GOOGLE_DOC | MIME_GOOGLE_SHEET | MIME_GOOGLE_SLIDES | MIME_GOOGLE_DRAWING
    )
}

/// Get the default export format for a Google Workspace type.
pub fn default_export_format(google_mime: &str) -> Option<&'static str> {
    match google_mime {
        MIME_GOOGLE_DOC | MIME_GOOGLE_SHEET | MIME_GOOGLE_SLIDES | MIME_GOOGLE_DRAWING => {
            Some("pdf")
        }
        _ => None,
    }
}

/// Guess MIME type from file extension.
pub fn guess_content_type_from_path(path: &str) -> &'static str {
    match std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
    {
        Some("pdf") => "application/pdf",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("txt") => "text/plain",
        Some("html") | Some("htm") => "text/html",
        Some("csv") => "text/csv",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("zip") => "application/zip",
        Some("doc") => "application/msword",
        Some("docx") => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        Some("xls") => "application/vnd.ms-excel",
        Some("xlsx") => {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        }
        Some("ppt") => "application/vnd.ms-powerpoint",
        Some("pptx") => {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        }
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===================================================================
    // REQ-RT-031 (Should): Shared export function — format_to_mime
    // ===================================================================

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime maps "pdf" correctly
    #[test]
    fn req_rt_031_format_to_mime_pdf() {
        assert_eq!(format_to_mime("pdf"), Some("application/pdf"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime maps "docx" correctly
    #[test]
    fn req_rt_031_format_to_mime_docx() {
        assert_eq!(
            format_to_mime("docx"),
            Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
        );
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime maps "doc" alias to docx MIME
    #[test]
    fn req_rt_031_format_to_mime_doc_alias() {
        assert_eq!(
            format_to_mime("doc"),
            Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
        );
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime maps "xlsx" correctly
    #[test]
    fn req_rt_031_format_to_mime_xlsx() {
        assert_eq!(
            format_to_mime("xlsx"),
            Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        );
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime maps "xls" alias to xlsx MIME
    #[test]
    fn req_rt_031_format_to_mime_xls_alias() {
        assert_eq!(
            format_to_mime("xls"),
            Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        );
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime maps "pptx" correctly
    #[test]
    fn req_rt_031_format_to_mime_pptx() {
        assert_eq!(
            format_to_mime("pptx"),
            Some("application/vnd.openxmlformats-officedocument.presentationml.presentation")
        );
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime maps "ppt" alias to pptx MIME
    #[test]
    fn req_rt_031_format_to_mime_ppt_alias() {
        assert_eq!(
            format_to_mime("ppt"),
            Some("application/vnd.openxmlformats-officedocument.presentationml.presentation")
        );
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime maps "csv" correctly
    #[test]
    fn req_rt_031_format_to_mime_csv() {
        assert_eq!(format_to_mime("csv"), Some("text/csv"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime maps "txt" correctly
    #[test]
    fn req_rt_031_format_to_mime_txt() {
        assert_eq!(format_to_mime("txt"), Some("text/plain"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime maps "text" alias correctly
    #[test]
    fn req_rt_031_format_to_mime_text_alias() {
        assert_eq!(format_to_mime("text"), Some("text/plain"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime maps "png" correctly
    #[test]
    fn req_rt_031_format_to_mime_png() {
        assert_eq!(format_to_mime("png"), Some("image/png"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime maps "svg" correctly
    #[test]
    fn req_rt_031_format_to_mime_svg() {
        assert_eq!(format_to_mime("svg"), Some("image/svg+xml"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime maps "html" correctly
    #[test]
    fn req_rt_031_format_to_mime_html() {
        assert_eq!(format_to_mime("html"), Some("text/html"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime returns None for unknown format
    #[test]
    fn req_rt_031_format_to_mime_unknown() {
        assert_eq!(format_to_mime("mp3"), None);
        assert_eq!(format_to_mime("rar"), None);
        assert_eq!(format_to_mime(""), None);
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: format_to_mime is case-insensitive
    #[test]
    fn req_rt_031_format_to_mime_case_insensitive() {
        assert_eq!(format_to_mime("PDF"), Some("application/pdf"));
        assert_eq!(format_to_mime("Docx"), Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document"));
    }

    // ===================================================================
    // REQ-RT-031 (Should): export_formats
    // ===================================================================

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Google Doc has 4 export formats
    #[test]
    fn req_rt_031_export_formats_google_doc() {
        let formats = export_formats(MIME_GOOGLE_DOC);
        assert_eq!(formats.len(), 4);
        assert!(formats.iter().any(|(name, _, _)| *name == "pdf"));
        assert!(formats.iter().any(|(name, _, _)| *name == "docx"));
        assert!(formats.iter().any(|(name, _, _)| *name == "txt"));
        assert!(formats.iter().any(|(name, _, _)| *name == "html"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Google Sheet has 3 export formats
    #[test]
    fn req_rt_031_export_formats_google_sheet() {
        let formats = export_formats(MIME_GOOGLE_SHEET);
        assert_eq!(formats.len(), 3);
        assert!(formats.iter().any(|(name, _, _)| *name == "pdf"));
        assert!(formats.iter().any(|(name, _, _)| *name == "xlsx"));
        assert!(formats.iter().any(|(name, _, _)| *name == "csv"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Google Slides has 3 export formats
    #[test]
    fn req_rt_031_export_formats_google_slides() {
        let formats = export_formats(MIME_GOOGLE_SLIDES);
        assert_eq!(formats.len(), 3);
        assert!(formats.iter().any(|(name, _, _)| *name == "pdf"));
        assert!(formats.iter().any(|(name, _, _)| *name == "pptx"));
        assert!(formats.iter().any(|(name, _, _)| *name == "txt"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Google Drawing has 3 export formats
    #[test]
    fn req_rt_031_export_formats_google_drawing() {
        let formats = export_formats(MIME_GOOGLE_DRAWING);
        assert_eq!(formats.len(), 3);
        assert!(formats.iter().any(|(name, _, _)| *name == "pdf"));
        assert!(formats.iter().any(|(name, _, _)| *name == "png"));
        assert!(formats.iter().any(|(name, _, _)| *name == "svg"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Non-workspace type returns empty vec
    #[test]
    fn req_rt_031_export_formats_non_workspace() {
        assert!(export_formats("application/pdf").is_empty());
        assert!(export_formats("text/plain").is_empty());
        assert!(export_formats("").is_empty());
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Export format tuples have correct extensions
    #[test]
    fn req_rt_031_export_formats_extensions() {
        let doc_formats = export_formats(MIME_GOOGLE_DOC);
        let pdf_entry = doc_formats.iter().find(|(name, _, _)| *name == "pdf").unwrap();
        assert_eq!(pdf_entry.2, ".pdf");

        let docx_entry = doc_formats.iter().find(|(name, _, _)| *name == "docx").unwrap();
        assert_eq!(docx_entry.2, ".docx");
    }

    // ===================================================================
    // REQ-RT-031 (Should): is_google_workspace_type
    // ===================================================================

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Google Docs type is workspace type
    #[test]
    fn req_rt_031_is_workspace_type_doc() {
        assert!(is_google_workspace_type(MIME_GOOGLE_DOC));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Google Sheet type is workspace type
    #[test]
    fn req_rt_031_is_workspace_type_sheet() {
        assert!(is_google_workspace_type(MIME_GOOGLE_SHEET));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Google Slides type is workspace type
    #[test]
    fn req_rt_031_is_workspace_type_slides() {
        assert!(is_google_workspace_type(MIME_GOOGLE_SLIDES));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Google Drawing type is workspace type
    #[test]
    fn req_rt_031_is_workspace_type_drawing() {
        assert!(is_google_workspace_type(MIME_GOOGLE_DRAWING));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Regular file types are not workspace types
    #[test]
    fn req_rt_031_is_workspace_type_non_workspace() {
        assert!(!is_google_workspace_type("application/pdf"));
        assert!(!is_google_workspace_type("text/plain"));
        assert!(!is_google_workspace_type("image/png"));
        assert!(!is_google_workspace_type("application/octet-stream"));
        assert!(!is_google_workspace_type(""));
    }

    // ===================================================================
    // REQ-RT-031 (Should): default_export_format
    // ===================================================================

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Default export format for Google Doc is pdf
    #[test]
    fn req_rt_031_default_export_format_doc() {
        assert_eq!(default_export_format(MIME_GOOGLE_DOC), Some("pdf"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Default export format for Google Sheet is pdf
    #[test]
    fn req_rt_031_default_export_format_sheet() {
        assert_eq!(default_export_format(MIME_GOOGLE_SHEET), Some("pdf"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Default export format for Google Slides is pdf
    #[test]
    fn req_rt_031_default_export_format_slides() {
        assert_eq!(default_export_format(MIME_GOOGLE_SLIDES), Some("pdf"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Default export format for Google Drawing is pdf
    #[test]
    fn req_rt_031_default_export_format_drawing() {
        assert_eq!(default_export_format(MIME_GOOGLE_DRAWING), Some("pdf"));
    }

    // Requirement: REQ-RT-031 (Should)
    // Acceptance: Non-workspace type has no default export format
    #[test]
    fn req_rt_031_default_export_format_non_workspace() {
        assert_eq!(default_export_format("application/pdf"), None);
        assert_eq!(default_export_format("text/plain"), None);
        assert_eq!(default_export_format(""), None);
    }

    // ===================================================================
    // REQ-RT-028 (Must): guess_content_type_from_path
    // ===================================================================

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: PDF file detected
    #[test]
    fn req_rt_028_guess_content_type_pdf() {
        assert_eq!(guess_content_type_from_path("report.pdf"), "application/pdf");
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: JPEG file detected
    #[test]
    fn req_rt_028_guess_content_type_jpeg() {
        assert_eq!(guess_content_type_from_path("photo.jpg"), "image/jpeg");
        assert_eq!(guess_content_type_from_path("photo.jpeg"), "image/jpeg");
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: PNG file detected
    #[test]
    fn req_rt_028_guess_content_type_png() {
        assert_eq!(guess_content_type_from_path("image.png"), "image/png");
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: GIF file detected
    #[test]
    fn req_rt_028_guess_content_type_gif() {
        assert_eq!(guess_content_type_from_path("animation.gif"), "image/gif");
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: Text file detected
    #[test]
    fn req_rt_028_guess_content_type_txt() {
        assert_eq!(guess_content_type_from_path("notes.txt"), "text/plain");
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: HTML file detected
    #[test]
    fn req_rt_028_guess_content_type_html() {
        assert_eq!(guess_content_type_from_path("page.html"), "text/html");
        assert_eq!(guess_content_type_from_path("page.htm"), "text/html");
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: CSV file detected
    #[test]
    fn req_rt_028_guess_content_type_csv() {
        assert_eq!(guess_content_type_from_path("data.csv"), "text/csv");
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: JSON file detected
    #[test]
    fn req_rt_028_guess_content_type_json() {
        assert_eq!(guess_content_type_from_path("config.json"), "application/json");
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: XML file detected
    #[test]
    fn req_rt_028_guess_content_type_xml() {
        assert_eq!(guess_content_type_from_path("data.xml"), "application/xml");
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: ZIP file detected
    #[test]
    fn req_rt_028_guess_content_type_zip() {
        assert_eq!(guess_content_type_from_path("archive.zip"), "application/zip");
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: DOCX file detected
    #[test]
    fn req_rt_028_guess_content_type_docx() {
        assert_eq!(
            guess_content_type_from_path("document.docx"),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        );
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: XLSX file detected
    #[test]
    fn req_rt_028_guess_content_type_xlsx() {
        assert_eq!(
            guess_content_type_from_path("spreadsheet.xlsx"),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        );
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: PPTX file detected
    #[test]
    fn req_rt_028_guess_content_type_pptx() {
        assert_eq!(
            guess_content_type_from_path("presentation.pptx"),
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        );
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: Legacy Office formats detected
    #[test]
    fn req_rt_028_guess_content_type_legacy_office() {
        assert_eq!(guess_content_type_from_path("document.doc"), "application/msword");
        assert_eq!(guess_content_type_from_path("spreadsheet.xls"), "application/vnd.ms-excel");
        assert_eq!(
            guess_content_type_from_path("presentation.ppt"),
            "application/vnd.ms-powerpoint"
        );
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: SVG file detected
    #[test]
    fn req_rt_028_guess_content_type_svg() {
        assert_eq!(guess_content_type_from_path("drawing.svg"), "image/svg+xml");
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: Unknown extension returns octet-stream
    #[test]
    fn req_rt_028_guess_content_type_unknown() {
        assert_eq!(
            guess_content_type_from_path("file.xyz"),
            "application/octet-stream"
        );
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: No extension returns octet-stream
    #[test]
    fn req_rt_028_guess_content_type_no_extension() {
        assert_eq!(
            guess_content_type_from_path("Makefile"),
            "application/octet-stream"
        );
    }

    // Requirement: REQ-RT-028 (Must)
    // Acceptance: Path with directory components works
    #[test]
    fn req_rt_028_guess_content_type_with_path() {
        assert_eq!(
            guess_content_type_from_path("/home/user/document.pdf"),
            "application/pdf"
        );
    }
}
