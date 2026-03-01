//! Cross-calendar event search.

/// Build a search query that searches across multiple calendars.
/// Returns a list of (calendar_id, query) pairs, one per calendar.
pub fn build_cross_calendar_search_params(
    query: &str,
    calendar_ids: &[String],
) -> Vec<(String, String)> {
    calendar_ids
        .iter()
        .map(|cal_id| (cal_id.clone(), query.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Requirement: REQ-CAL-010 (Must)
    // Acceptance: Cross-calendar search generates params per calendar
    #[test]
    fn req_cal_010_cross_calendar_search() {
        let params = build_cross_calendar_search_params(
            "meeting",
            &["primary".to_string(), "team@group.calendar.google.com".to_string()],
        );
        assert_eq!(params.len(), 2);
    }

    // Requirement: REQ-CAL-010 (Must)
    // Edge case: Empty calendar list
    #[test]
    fn req_cal_010_cross_calendar_search_empty() {
        let params = build_cross_calendar_search_params("meeting", &[]);
        assert!(params.is_empty());
    }
}
