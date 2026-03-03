//! Open command: offline URL generation for Google resource IDs.
//!
//! REQ-CLI-019: `open <target>` command generates browser-ready URLs
//! for Google Workspace resources without making any API calls.

use clap::Args;

/// Resource type for URL generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceType {
    /// Auto-detect from URL or default to Drive file
    Auto,
    /// Google Drive file
    Drive,
    /// Google Drive folder
    Folder,
    /// Google Docs document
    Docs,
    /// Google Sheets spreadsheet
    Sheets,
    /// Google Slides presentation
    Slides,
    /// Gmail thread
    GmailThread,
}

impl std::str::FromStr for ResourceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(ResourceType::Auto),
            "drive" | "file" => Ok(ResourceType::Drive),
            "folder" | "dir" => Ok(ResourceType::Folder),
            "docs" | "doc" | "document" => Ok(ResourceType::Docs),
            "sheets" | "sheet" | "spreadsheet" => Ok(ResourceType::Sheets),
            "slides" | "slide" | "presentation" => Ok(ResourceType::Slides),
            "gmail-thread" | "gmail" | "thread" => Ok(ResourceType::GmailThread),
            _ => Err(format!("unknown resource type: '{}'. Valid types: auto, drive, folder, docs, sheets, slides, gmail-thread", s)),
        }
    }
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceType::Auto => write!(f, "auto"),
            ResourceType::Drive => write!(f, "drive"),
            ResourceType::Folder => write!(f, "folder"),
            ResourceType::Docs => write!(f, "docs"),
            ResourceType::Sheets => write!(f, "sheets"),
            ResourceType::Slides => write!(f, "slides"),
            ResourceType::GmailThread => write!(f, "gmail-thread"),
        }
    }
}

/// Arguments for the `open` command.
#[derive(Args, Debug)]
pub struct OpenArgs {
    /// Google resource ID or URL to open
    pub target: String,

    /// Resource type (auto-detects from URL if omitted)
    #[arg(long, short = 't', default_value = "auto")]
    pub r#type: String,
}

/// Generate a URL for a Google resource ID given a resource type.
pub fn generate_url(id: &str, resource_type: &ResourceType) -> String {
    match resource_type {
        ResourceType::Drive | ResourceType::Auto => {
            format!("https://drive.google.com/file/d/{}/view", id)
        }
        ResourceType::Folder => {
            format!("https://drive.google.com/drive/folders/{}", id)
        }
        ResourceType::Docs => {
            format!("https://docs.google.com/document/d/{}/edit", id)
        }
        ResourceType::Sheets => {
            format!("https://docs.google.com/spreadsheets/d/{}/edit", id)
        }
        ResourceType::Slides => {
            format!("https://docs.google.com/presentation/d/{}/edit", id)
        }
        ResourceType::GmailThread => {
            format!("https://mail.google.com/mail/u/0/#all/{}", id)
        }
    }
}

/// Try to detect the resource type from a URL and extract/canonicalize it.
/// Returns (canonical_url, detected_type) if the input is a recognized URL,
/// or None if it appears to be a bare ID.
pub fn detect_from_url(input: &str) -> Option<(String, ResourceType)> {
    // Must start with http:// or https:// to be considered a URL
    if !input.starts_with("http://") && !input.starts_with("https://") {
        return None;
    }

    // Google Drive file: https://drive.google.com/file/d/{id}/...
    if let Some(id) = extract_path_segment(input, "drive.google.com", "/file/d/") {
        let url = generate_url(&id, &ResourceType::Drive);
        return Some((url, ResourceType::Drive));
    }

    // Google Drive folder: https://drive.google.com/drive/folders/{id}
    if let Some(id) = extract_path_segment(input, "drive.google.com", "/drive/folders/") {
        let url = generate_url(&id, &ResourceType::Folder);
        return Some((url, ResourceType::Folder));
    }

    // Google Docs: https://docs.google.com/document/d/{id}/...
    if let Some(id) = extract_path_segment(input, "docs.google.com", "/document/d/") {
        let url = generate_url(&id, &ResourceType::Docs);
        return Some((url, ResourceType::Docs));
    }

    // Google Sheets: https://docs.google.com/spreadsheets/d/{id}/...
    if let Some(id) = extract_path_segment(input, "docs.google.com", "/spreadsheets/d/") {
        let url = generate_url(&id, &ResourceType::Sheets);
        return Some((url, ResourceType::Sheets));
    }

    // Google Slides: https://docs.google.com/presentation/d/{id}/...
    if let Some(id) = extract_path_segment(input, "docs.google.com", "/presentation/d/") {
        let url = generate_url(&id, &ResourceType::Slides);
        return Some((url, ResourceType::Slides));
    }

    // Gmail thread: https://mail.google.com/mail/u/0/#all/{id}
    if let Ok(parsed) = url::Url::parse(input) {
        if parsed.host_str() == Some("mail.google.com") {
            if let Some(hash_pos) = input.find('#') {
                let fragment = &input[hash_pos + 1..];
                // Fragment is like "all/{id}" or "inbox/{id}"
                if let Some(slash_pos) = fragment.rfind('/') {
                    let id = &fragment[slash_pos + 1..];
                    if !id.is_empty() {
                        let url = generate_url(id, &ResourceType::GmailThread);
                        return Some((url, ResourceType::GmailThread));
                    }
                }
            }
        }
    }

    None
}

/// Extract a resource ID from a URL given the expected host and path prefix.
/// Uses proper URL parsing to prevent host spoofing via substring matches.
fn extract_path_segment(raw_url: &str, expected_host: &str, path_prefix: &str) -> Option<String> {
    let parsed = url::Url::parse(raw_url).ok()?;

    // Proper hostname comparison — prevents spoofing via path injection
    if parsed.host_str()? != expected_host {
        return None;
    }

    let path = parsed.path();
    if !path.starts_with(path_prefix) {
        return None;
    }

    let after_prefix = &path[path_prefix.len()..];

    // Extract the ID (everything up to the next '/' or end of string)
    let id_end = after_prefix.find('/').unwrap_or(after_prefix.len());

    let id = &after_prefix[..id_end];
    if id.is_empty() {
        return None;
    }

    Some(id.to_string())
}

/// Resolve the target into a final URL.
/// If the target is a URL, canonicalize it. If it's a bare ID, use the type flag.
pub fn resolve_target(target: &str, type_str: &str) -> Result<String, String> {
    let resource_type: ResourceType = type_str.parse()?;

    // If target looks like a URL, try to auto-detect and canonicalize
    if let Some((canonical_url, _detected_type)) = detect_from_url(target) {
        return Ok(canonical_url);
    }

    // Bare ID: use the specified type
    Ok(generate_url(target, &resource_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CLI-019: URL generation for bare IDs
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Drive file URL generation
    #[test]
    fn req_cli_019_drive_file_url() {
        let url = generate_url("abc123", &ResourceType::Drive);
        assert_eq!(url, "https://drive.google.com/file/d/abc123/view");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Drive folder URL generation
    #[test]
    fn req_cli_019_folder_url() {
        let url = generate_url("folder_id", &ResourceType::Folder);
        assert_eq!(url, "https://drive.google.com/drive/folders/folder_id");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Google Docs URL generation
    #[test]
    fn req_cli_019_docs_url() {
        let url = generate_url("doc_id", &ResourceType::Docs);
        assert_eq!(url, "https://docs.google.com/document/d/doc_id/edit");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Google Sheets URL generation
    #[test]
    fn req_cli_019_sheets_url() {
        let url = generate_url("sheet_id", &ResourceType::Sheets);
        assert_eq!(url, "https://docs.google.com/spreadsheets/d/sheet_id/edit");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Google Slides URL generation
    #[test]
    fn req_cli_019_slides_url() {
        let url = generate_url("slide_id", &ResourceType::Slides);
        assert_eq!(url, "https://docs.google.com/presentation/d/slide_id/edit");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Gmail thread URL generation
    #[test]
    fn req_cli_019_gmail_thread_url() {
        let url = generate_url("thread_id", &ResourceType::GmailThread);
        assert_eq!(url, "https://mail.google.com/mail/u/0/#all/thread_id");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Auto defaults to Drive file
    #[test]
    fn req_cli_019_auto_defaults_to_drive() {
        let url = generate_url("some_id", &ResourceType::Auto);
        assert_eq!(url, "https://drive.google.com/file/d/some_id/view");
    }

    // ---------------------------------------------------------------
    // REQ-CLI-019: ResourceType parsing
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Resource type string parsing
    #[test]
    fn req_cli_019_resource_type_parsing() {
        assert_eq!("auto".parse::<ResourceType>().unwrap(), ResourceType::Auto);
        assert_eq!(
            "drive".parse::<ResourceType>().unwrap(),
            ResourceType::Drive
        );
        assert_eq!("file".parse::<ResourceType>().unwrap(), ResourceType::Drive);
        assert_eq!(
            "folder".parse::<ResourceType>().unwrap(),
            ResourceType::Folder
        );
        assert_eq!("dir".parse::<ResourceType>().unwrap(), ResourceType::Folder);
        assert_eq!("docs".parse::<ResourceType>().unwrap(), ResourceType::Docs);
        assert_eq!("doc".parse::<ResourceType>().unwrap(), ResourceType::Docs);
        assert_eq!(
            "document".parse::<ResourceType>().unwrap(),
            ResourceType::Docs
        );
        assert_eq!(
            "sheets".parse::<ResourceType>().unwrap(),
            ResourceType::Sheets
        );
        assert_eq!(
            "sheet".parse::<ResourceType>().unwrap(),
            ResourceType::Sheets
        );
        assert_eq!(
            "spreadsheet".parse::<ResourceType>().unwrap(),
            ResourceType::Sheets
        );
        assert_eq!(
            "slides".parse::<ResourceType>().unwrap(),
            ResourceType::Slides
        );
        assert_eq!(
            "slide".parse::<ResourceType>().unwrap(),
            ResourceType::Slides
        );
        assert_eq!(
            "presentation".parse::<ResourceType>().unwrap(),
            ResourceType::Slides
        );
        assert_eq!(
            "gmail-thread".parse::<ResourceType>().unwrap(),
            ResourceType::GmailThread
        );
        assert_eq!(
            "gmail".parse::<ResourceType>().unwrap(),
            ResourceType::GmailThread
        );
        assert_eq!(
            "thread".parse::<ResourceType>().unwrap(),
            ResourceType::GmailThread
        );
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Case insensitive parsing
    #[test]
    fn req_cli_019_resource_type_case_insensitive() {
        assert_eq!(
            "DRIVE".parse::<ResourceType>().unwrap(),
            ResourceType::Drive
        );
        assert_eq!("Docs".parse::<ResourceType>().unwrap(), ResourceType::Docs);
        assert_eq!(
            "GMAIL-THREAD".parse::<ResourceType>().unwrap(),
            ResourceType::GmailThread
        );
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Invalid type returns error
    #[test]
    fn req_cli_019_resource_type_invalid() {
        assert!("invalid".parse::<ResourceType>().is_err());
        assert!("".parse::<ResourceType>().is_err());
    }

    // ---------------------------------------------------------------
    // REQ-CLI-019: URL auto-detection
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Detect Drive file URL
    #[test]
    fn req_cli_019_detect_drive_file() {
        let input = "https://drive.google.com/file/d/abc123/view?usp=sharing";
        let (url, rt) = detect_from_url(input).unwrap();
        assert_eq!(rt, ResourceType::Drive);
        assert_eq!(url, "https://drive.google.com/file/d/abc123/view");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Detect Drive folder URL
    #[test]
    fn req_cli_019_detect_drive_folder() {
        let input = "https://drive.google.com/drive/folders/folder123";
        let (url, rt) = detect_from_url(input).unwrap();
        assert_eq!(rt, ResourceType::Folder);
        assert_eq!(url, "https://drive.google.com/drive/folders/folder123");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Detect Google Docs URL
    #[test]
    fn req_cli_019_detect_docs() {
        let input = "https://docs.google.com/document/d/doc123/edit";
        let (url, rt) = detect_from_url(input).unwrap();
        assert_eq!(rt, ResourceType::Docs);
        assert_eq!(url, "https://docs.google.com/document/d/doc123/edit");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Detect Google Sheets URL
    #[test]
    fn req_cli_019_detect_sheets() {
        let input = "https://docs.google.com/spreadsheets/d/sheet123/edit#gid=0";
        let (url, rt) = detect_from_url(input).unwrap();
        assert_eq!(rt, ResourceType::Sheets);
        assert_eq!(url, "https://docs.google.com/spreadsheets/d/sheet123/edit");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Detect Google Slides URL
    #[test]
    fn req_cli_019_detect_slides() {
        let input = "https://docs.google.com/presentation/d/slide123/edit";
        let (url, rt) = detect_from_url(input).unwrap();
        assert_eq!(rt, ResourceType::Slides);
        assert_eq!(url, "https://docs.google.com/presentation/d/slide123/edit");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Detect Gmail thread URL
    #[test]
    fn req_cli_019_detect_gmail_thread() {
        let input = "https://mail.google.com/mail/u/0/#all/18abc123def";
        let (url, rt) = detect_from_url(input).unwrap();
        assert_eq!(rt, ResourceType::GmailThread);
        assert_eq!(url, "https://mail.google.com/mail/u/0/#all/18abc123def");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Detect Gmail inbox URL
    #[test]
    fn req_cli_019_detect_gmail_inbox() {
        let input = "https://mail.google.com/mail/u/0/#inbox/18abc123def";
        let (url, rt) = detect_from_url(input).unwrap();
        assert_eq!(rt, ResourceType::GmailThread);
        // Canonicalized to #all/
        assert_eq!(url, "https://mail.google.com/mail/u/0/#all/18abc123def");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Bare ID returns None (not a URL)
    #[test]
    fn req_cli_019_bare_id_not_detected() {
        assert!(detect_from_url("abc123").is_none());
        assert!(detect_from_url("1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgVE2upms").is_none());
    }

    // ---------------------------------------------------------------
    // REQ-CLI-019: resolve_target
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Bare ID with type flag
    #[test]
    fn req_cli_019_resolve_bare_id_with_type() {
        let url = resolve_target("abc123", "docs").unwrap();
        assert_eq!(url, "https://docs.google.com/document/d/abc123/edit");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: URL auto-detected regardless of type flag
    #[test]
    fn req_cli_019_resolve_url_auto_detected() {
        let url = resolve_target(
            "https://docs.google.com/spreadsheets/d/sheet_id/edit",
            "auto",
        )
        .unwrap();
        assert_eq!(url, "https://docs.google.com/spreadsheets/d/sheet_id/edit");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Bare ID with default auto type
    #[test]
    fn req_cli_019_resolve_bare_id_auto() {
        let url = resolve_target("abc123", "auto").unwrap();
        assert_eq!(url, "https://drive.google.com/file/d/abc123/view");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Invalid type returns error
    #[test]
    fn req_cli_019_resolve_invalid_type() {
        assert!(resolve_target("abc123", "invalid").is_err());
    }

    // ---------------------------------------------------------------
    // REQ-CLI-019: ResourceType Display
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Display trait for ResourceType
    #[test]
    fn req_cli_019_resource_type_display() {
        assert_eq!(format!("{}", ResourceType::Auto), "auto");
        assert_eq!(format!("{}", ResourceType::Drive), "drive");
        assert_eq!(format!("{}", ResourceType::Folder), "folder");
        assert_eq!(format!("{}", ResourceType::Docs), "docs");
        assert_eq!(format!("{}", ResourceType::Sheets), "sheets");
        assert_eq!(format!("{}", ResourceType::Slides), "slides");
        assert_eq!(format!("{}", ResourceType::GmailThread), "gmail-thread");
    }

    // ---------------------------------------------------------------
    // REQ-CLI-019: Edge cases
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: URL with query params stripped from ID
    #[test]
    fn req_cli_019_url_query_params_stripped() {
        let input = "https://drive.google.com/file/d/abc123?usp=sharing";
        let (url, _) = detect_from_url(input).unwrap();
        assert_eq!(url, "https://drive.google.com/file/d/abc123/view");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: URL with trailing slash handled
    #[test]
    fn req_cli_019_url_trailing_slash() {
        let input = "https://drive.google.com/file/d/abc123/";
        let (url, _) = detect_from_url(input).unwrap();
        assert_eq!(url, "https://drive.google.com/file/d/abc123/view");
    }

    // Requirement: REQ-CLI-019 (Must)
    // Acceptance: Long real-world Google IDs work
    #[test]
    fn req_cli_019_real_world_ids() {
        let long_id = "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgVE2upms";
        let url = generate_url(long_id, &ResourceType::Sheets);
        assert_eq!(
            url,
            format!("https://docs.google.com/spreadsheets/d/{}/edit", long_id)
        );
    }
}
