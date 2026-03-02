//! Shared drives (Team Drives) operations.

use super::types::DRIVE_BASE_URL;

/// Build URL for listing shared drives.
pub fn build_drives_list_url(max_results: Option<u32>, page_token: Option<&str>) -> String {
    let mut url = format!("{}/drives", DRIVE_BASE_URL);
    let mut params = Vec::new();
    if let Some(max) = max_results {
        params.push(format!("pageSize={}", max));
    }
    if let Some(token) = page_token {
        params.push(format!("pageToken={}", token));
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    // Requirement: REQ-DRIVE-014 (Must)
    // Acceptance: Shared drives list URL
    #[test]
    fn req_drive_014_drives_list_url() {
        let url = build_drives_list_url(Some(100), None);
        assert!(url.contains("drives"));
    }

    // Requirement: REQ-DRIVE-014 (Must)
    // Acceptance: Shared drives list with page token
    #[test]
    fn req_drive_014_drives_list_url_with_page() {
        let url = build_drives_list_url(None, Some("next_token"));
        assert!(url.contains("pageToken=next_token"));
    }
}
