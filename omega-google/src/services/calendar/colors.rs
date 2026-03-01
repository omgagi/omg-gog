//! Calendar color definitions.

use super::types::CALENDAR_BASE_URL;

/// Build URL for the colors endpoint.
pub fn build_colors_url() -> String {
    format!("{}/colors", CALENDAR_BASE_URL)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Requirement: REQ-CAL-014 (Must)
    // Acceptance: Colors URL
    #[test]
    fn req_cal_014_colors_url() {
        let url = build_colors_url();
        assert!(url.contains("colors"));
    }
}
