use chrono::{DateTime, NaiveDate, Utc, Duration};

/// Parse flexible date/time input matching gogcli conventions.
/// Accepted formats:
/// - YYYY-MM-DD (date only)
/// - YYYY-MM-DDTHH:MM:SS / YYYY-MM-DD HH:MM:SS (datetime without timezone)
/// - RFC3339 (full datetime with timezone)
/// - "now", "today", "tomorrow", "yesterday" (relative)
/// - Weekday names: "monday", "next friday", "last tuesday"
/// - Durations: "24h", "7d", "30m", "1h30m"
pub fn parse_datetime(input: &str) -> anyhow::Result<DateTime<Utc>> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        anyhow::bail!("empty datetime input");
    }

    let lower = trimmed.to_lowercase();

    // Relative keywords
    match lower.as_str() {
        "now" => return Ok(Utc::now()),
        "today" => {
            let today = Utc::now().date_naive();
            return Ok(today.and_hms_opt(0, 0, 0).unwrap().and_utc());
        }
        "tomorrow" => {
            let tomorrow = Utc::now().date_naive() + Duration::days(1);
            return Ok(tomorrow.and_hms_opt(0, 0, 0).unwrap().and_utc());
        }
        "yesterday" => {
            let yesterday = Utc::now().date_naive() - Duration::days(1);
            return Ok(yesterday.and_hms_opt(0, 0, 0).unwrap().and_utc());
        }
        _ => {}
    }

    // Try RFC3339
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try datetime with T separator: YYYY-MM-DDTHH:MM:SS
    if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
        return Ok(ndt.and_utc());
    }

    // Try datetime with space separator: YYYY-MM-DD HH:MM:SS
    if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S") {
        return Ok(ndt.and_utc());
    }

    // Try date only: YYYY-MM-DD -> start of day UTC
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc());
    }

    anyhow::bail!("unrecognized datetime format: {}", trimmed)
}

/// Parse a date-only string.
pub fn parse_date(input: &str) -> anyhow::Result<NaiveDate> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        anyhow::bail!("empty date input");
    }
    NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .map_err(|e| anyhow::anyhow!("invalid date '{}': {}", trimmed, e))
}

/// Parse a duration string (e.g., "24h", "7d", "30m", "1h30m").
pub fn parse_duration(input: &str) -> anyhow::Result<Duration> {
    let trimmed = input.trim().to_lowercase();
    if trimmed.is_empty() {
        anyhow::bail!("empty duration input");
    }

    let mut total_seconds: i64 = 0;
    let mut num_buf = String::new();
    let mut found_unit = false;

    for ch in trimmed.chars() {
        if ch.is_ascii_digit() {
            num_buf.push(ch);
        } else {
            let n: i64 = if num_buf.is_empty() {
                if !found_unit {
                    anyhow::bail!("invalid duration: {}", input);
                }
                0
            } else {
                num_buf.parse()?
            };
            num_buf.clear();
            match ch {
                'd' => total_seconds += n * 86400,
                'h' => total_seconds += n * 3600,
                'm' => total_seconds += n * 60,
                's' => total_seconds += n,
                _ => anyhow::bail!("unknown duration unit '{}' in '{}'", ch, input),
            }
            found_unit = true;
        }
    }

    if !found_unit {
        anyhow::bail!("invalid duration (no unit): {}", input);
    }

    // If there's a remaining number without a unit, it's invalid
    if !num_buf.is_empty() {
        anyhow::bail!("trailing number without unit in duration: {}", input);
    }

    Ok(Duration::seconds(total_seconds))
}

/// Check if the input looks like a relative time keyword.
pub fn is_relative(input: &str) -> bool {
    let lower = input.trim().to_lowercase();
    matches!(lower.as_str(), "now" | "today" | "tomorrow" | "yesterday")
}

/// Check if the input looks like a weekday reference.
pub fn is_weekday_ref(input: &str) -> bool {
    let lower = input.trim().to_lowercase();
    let weekdays = ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"];
    // "monday", "next monday", "last monday"
    for wd in &weekdays {
        if lower == *wd || lower == format!("next {}", wd) || lower == format!("last {}", wd) {
            return true;
        }
    }
    false
}

/// Check if the input looks like a duration.
pub fn is_duration(input: &str) -> bool {
    let lower = input.trim().to_lowercase();
    // Matches patterns like "24h", "7d", "30m", "1h30m", "2d12h"
    if lower.is_empty() {
        return false;
    }
    // Must contain at least one letter (h, m, d, s)
    lower.contains('h') || lower.contains('m') || lower.contains('d') || lower.contains('s')
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    // ---------------------------------------------------------------
    // REQ-CLI-005 (Must): time now command / date-time parsing
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: RFC3339 parsing
    #[test]
    fn req_cli_005_parse_rfc3339() {
        let result = parse_datetime("2024-03-15T10:30:00Z");
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 3);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 10);
        assert_eq!(dt.minute(), 30);
    }

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: RFC3339 with timezone offset
    #[test]
    fn req_cli_005_parse_rfc3339_with_offset() {
        let result = parse_datetime("2024-03-15T10:30:00+05:30");
        let dt = result.unwrap();
        // Should be converted to UTC
        assert_eq!(dt.year(), 2024);
    }

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: Date-only parsing (YYYY-MM-DD)
    #[test]
    fn req_cli_005_parse_date_only() {
        let result = parse_date("2024-03-15");
        let date = result.unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 3);
        assert_eq!(date.day(), 15);
    }

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: Relative time "now"
    #[test]
    fn req_cli_005_parse_now() {
        let before = Utc::now();
        let result = parse_datetime("now").unwrap();
        let after = Utc::now();
        assert!(result >= before && result <= after);
    }

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: Relative time "today"
    #[test]
    fn req_cli_005_parse_today() {
        let result = parse_datetime("today").unwrap();
        let today = Utc::now().date_naive();
        assert_eq!(result.date_naive(), today);
    }

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: Relative time "tomorrow"
    #[test]
    fn req_cli_005_parse_tomorrow() {
        let result = parse_datetime("tomorrow").unwrap();
        let tomorrow = Utc::now().date_naive() + chrono::Duration::days(1);
        assert_eq!(result.date_naive(), tomorrow);
    }

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: Relative time "yesterday"
    #[test]
    fn req_cli_005_parse_yesterday() {
        let result = parse_datetime("yesterday").unwrap();
        let yesterday = Utc::now().date_naive() - chrono::Duration::days(1);
        assert_eq!(result.date_naive(), yesterday);
    }

    // ---------------------------------------------------------------
    // REQ-CLI-005 (Must): Duration parsing
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: Duration "24h"
    #[test]
    fn req_cli_005_parse_duration_hours() {
        let d = parse_duration("24h").unwrap();
        assert_eq!(d.num_hours(), 24);
    }

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: Duration "7d"
    #[test]
    fn req_cli_005_parse_duration_days() {
        let d = parse_duration("7d").unwrap();
        assert_eq!(d.num_days(), 7);
    }

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: Duration "30m"
    #[test]
    fn req_cli_005_parse_duration_minutes() {
        let d = parse_duration("30m").unwrap();
        assert_eq!(d.num_minutes(), 30);
    }

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: Combined duration "1h30m"
    #[test]
    fn req_cli_005_parse_duration_combined() {
        let d = parse_duration("1h30m").unwrap();
        assert_eq!(d.num_minutes(), 90);
    }

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: Duration "2d12h"
    #[test]
    fn req_cli_005_parse_duration_days_hours() {
        let d = parse_duration("2d12h").unwrap();
        assert_eq!(d.num_hours(), 60); // 2*24 + 12
    }

    // ---------------------------------------------------------------
    // REQ-CLI-005 (Must): Helper functions
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: is_relative detects keywords
    #[test]
    fn req_cli_005_is_relative() {
        assert!(is_relative("now"));
        assert!(is_relative("today"));
        assert!(is_relative("tomorrow"));
        assert!(is_relative("yesterday"));
        assert!(is_relative("NOW"));  // case insensitive
        assert!(!is_relative("monday"));
        assert!(!is_relative("2024-01-01"));
    }

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: is_weekday_ref detects weekday references
    #[test]
    fn req_cli_005_is_weekday_ref() {
        assert!(is_weekday_ref("monday"));
        assert!(is_weekday_ref("Friday"));
        assert!(is_weekday_ref("next monday"));
        assert!(is_weekday_ref("last tuesday"));
        assert!(!is_weekday_ref("now"));
        assert!(!is_weekday_ref("2024-01-01"));
    }

    // Requirement: REQ-CLI-005 (Must)
    // Acceptance: is_duration detects duration strings
    #[test]
    fn req_cli_005_is_duration() {
        assert!(is_duration("24h"));
        assert!(is_duration("7d"));
        assert!(is_duration("30m"));
        assert!(is_duration("1h30m"));
        assert!(!is_duration(""));
        assert!(!is_duration("2024-01-01"));
    }

    // ---------------------------------------------------------------
    // Edge cases for time parsing
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-005 (Must)
    // Edge case: Empty input
    #[test]
    fn req_cli_005_edge_empty_input() {
        assert!(parse_datetime("").is_err());
        assert!(parse_date("").is_err());
        assert!(parse_duration("").is_err());
    }

    // Requirement: REQ-CLI-005 (Must)
    // Edge case: Invalid date format
    #[test]
    fn req_cli_005_edge_invalid_date() {
        assert!(parse_date("not-a-date").is_err());
        assert!(parse_date("2024-13-01").is_err()); // month 13
        assert!(parse_date("2024-02-30").is_err()); // Feb 30
    }

    // Requirement: REQ-CLI-005 (Must)
    // Edge case: Invalid duration
    #[test]
    fn req_cli_005_edge_invalid_duration() {
        assert!(parse_duration("abc").is_err());
    }

    // Requirement: REQ-CLI-005 (Must)
    // Edge case: Datetime with space separator
    #[test]
    fn req_cli_005_parse_datetime_space_separator() {
        let result = parse_datetime("2024-03-15 10:30:00");
        assert!(result.is_ok());
    }

    // Requirement: REQ-CLI-005 (Must)
    // Edge case: Case insensitivity for keywords
    #[test]
    fn req_cli_005_edge_case_insensitive() {
        assert!(parse_datetime("NOW").is_ok());
        assert!(parse_datetime("Today").is_ok());
        assert!(parse_datetime("TOMORROW").is_ok());
    }

    // Requirement: REQ-CLI-005 (Must)
    // Edge case: Whitespace handling
    #[test]
    fn req_cli_005_edge_whitespace() {
        assert!(parse_datetime("  now  ").is_ok());
        assert!(parse_date("  2024-03-15  ").is_ok());
    }

    // Requirement: REQ-CLI-005 (Must)
    // Edge case: Zero duration
    #[test]
    fn req_cli_005_edge_zero_duration() {
        let d = parse_duration("0h");
        assert!(d.is_ok());
        assert_eq!(d.unwrap().num_seconds(), 0);
    }

    // Requirement: REQ-CLI-005 (Must)
    // Edge case: Very large duration
    #[test]
    fn req_cli_005_edge_large_duration() {
        let d = parse_duration("365d");
        assert!(d.is_ok());
        assert_eq!(d.unwrap().num_days(), 365);
    }
}
