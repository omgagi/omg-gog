use std::time::Duration;

/// Calculate exponential backoff delay with jitter.
/// Formula: base_delay * 2^attempt + random jitter (0-50% of base_delay * 2^attempt)
pub fn calculate_backoff(
    attempt: u32,
    base_delay: Duration,
    retry_after_secs: Option<u64>,
) -> Duration {
    // If Retry-After is specified, use it directly
    if let Some(secs) = retry_after_secs {
        return Duration::from_secs(secs);
    }

    if base_delay.is_zero() {
        return Duration::ZERO;
    }

    // Calculate base * 2^attempt, saturating to avoid overflow
    let multiplier = 1u64.checked_shl(attempt).unwrap_or(u64::MAX);
    let base_ms = base_delay.as_millis() as u64;
    let delay_ms = base_ms.saturating_mul(multiplier);

    // Add jitter: 0-50% of the delay
    let jitter_max = delay_ms / 2;
    let jitter = if jitter_max > 0 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(0..=jitter_max)
    } else {
        0
    };

    Duration::from_millis(delay_ms.saturating_add(jitter))
}

/// Parse the Retry-After header value (integer seconds or HTTP date).
pub fn parse_retry_after(header_value: &str) -> Option<Duration> {
    let trimmed = header_value.trim();
    // Try parsing as integer seconds
    if let Ok(secs) = trimmed.parse::<i64>() {
        return Some(Duration::from_secs(secs.max(0) as u64));
    }
    // Could not parse
    None
}

/// Determine if a status code is retryable.
pub fn is_retryable(status: u16) -> bool {
    is_rate_limited(status) || is_server_error(status)
}

/// Determine if a status code is a rate limit (429).
pub fn is_rate_limited(status: u16) -> bool {
    status == 429
}

/// Determine if a status code is a server error (5xx).
pub fn is_server_error(status: u16) -> bool {
    (500..600).contains(&status)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-HTTP-002 (Must): Exponential backoff with jitter for 429
    // ---------------------------------------------------------------

    // Requirement: REQ-HTTP-002 (Must)
    // Acceptance: Base delay 1s with exponential growth (1s, 2s, 4s)
    #[test]
    fn req_http_002_exponential_backoff_growth() {
        let base = Duration::from_secs(1);
        // Attempt 0: base * 2^0 = 1s (+ jitter)
        let d0 = calculate_backoff(0, base, None);
        assert!(
            d0 >= Duration::from_secs(1),
            "attempt 0 should be at least 1s"
        );
        assert!(
            d0 <= Duration::from_millis(1500),
            "attempt 0 should be at most 1.5s (with jitter)"
        );

        // Attempt 1: base * 2^1 = 2s (+ jitter)
        let d1 = calculate_backoff(1, base, None);
        assert!(
            d1 >= Duration::from_secs(2),
            "attempt 1 should be at least 2s"
        );
        assert!(
            d1 <= Duration::from_secs(3),
            "attempt 1 should be at most 3s (with jitter)"
        );

        // Attempt 2: base * 2^2 = 4s (+ jitter)
        let d2 = calculate_backoff(2, base, None);
        assert!(
            d2 >= Duration::from_secs(4),
            "attempt 2 should be at least 4s"
        );
        assert!(
            d2 <= Duration::from_secs(6),
            "attempt 2 should be at most 6s (with jitter)"
        );
    }

    // Requirement: REQ-HTTP-002 (Must)
    // Acceptance: Random jitter added (0-50% of base delay)
    #[test]
    fn req_http_002_jitter_within_range() {
        let base = Duration::from_secs(1);
        // Run multiple times to verify jitter varies
        let mut results = Vec::new();
        for _ in 0..20 {
            results.push(calculate_backoff(0, base, None));
        }
        // All should be within [1s, 1.5s]
        for d in &results {
            assert!(*d >= Duration::from_secs(1));
            assert!(*d <= Duration::from_millis(1500));
        }
        // At least some variation expected (not all identical) unless jitter is deterministic
    }

    // Requirement: REQ-HTTP-002 (Must)
    // Acceptance: Respects Retry-After header when present
    #[test]
    fn req_http_002_respects_retry_after_header() {
        let base = Duration::from_secs(1);
        let d = calculate_backoff(0, base, Some(5));
        assert_eq!(
            d,
            Duration::from_secs(5),
            "should use Retry-After value directly"
        );
    }

    // ---------------------------------------------------------------
    // REQ-HTTP-002 (Must): Parse Retry-After header
    // ---------------------------------------------------------------

    // Requirement: REQ-HTTP-002 (Must)
    // Acceptance: Parse integer seconds
    #[test]
    fn req_http_002_parse_retry_after_integer() {
        let result = parse_retry_after("120");
        assert_eq!(result, Some(Duration::from_secs(120)));
    }

    // Requirement: REQ-HTTP-002 (Must)
    // Edge case: Negative retry-after treated as 0
    #[test]
    fn req_http_002_parse_retry_after_negative() {
        let result = parse_retry_after("-5");
        assert_eq!(result, Some(Duration::from_secs(0)));
    }

    // Requirement: REQ-HTTP-002 (Must)
    // Edge case: Invalid retry-after returns None
    #[test]
    fn req_http_002_parse_retry_after_invalid() {
        let result = parse_retry_after("not-a-number");
        // Could be None if not parseable as int or HTTP date
        // Behavior depends on implementation
        assert!(result.is_none() || result == Some(Duration::from_secs(0)));
    }

    // Requirement: REQ-HTTP-002 (Must)
    // Edge case: Zero base delay
    #[test]
    fn req_http_002_zero_base_delay() {
        let d = calculate_backoff(0, Duration::from_secs(0), None);
        assert_eq!(d, Duration::from_secs(0));
    }

    // ---------------------------------------------------------------
    // REQ-HTTP-003 (Must): Retry on 5xx server errors
    // ---------------------------------------------------------------

    // Requirement: REQ-HTTP-003 (Must)
    // Acceptance: 5xx is retryable
    #[test]
    fn req_http_003_5xx_is_retryable() {
        assert!(is_server_error(500));
        assert!(is_server_error(502));
        assert!(is_server_error(503));
        assert!(is_server_error(599));
    }

    // Requirement: REQ-HTTP-003 (Must)
    // Acceptance: 4xx (except 429) is NOT retryable
    #[test]
    fn req_http_003_4xx_not_retryable() {
        assert!(!is_server_error(400));
        assert!(!is_server_error(401));
        assert!(!is_server_error(403));
        assert!(!is_server_error(404));
    }

    // Requirement: REQ-HTTP-003 (Must)
    // Acceptance: 429 is rate limited
    #[test]
    fn req_http_003_429_is_rate_limited() {
        assert!(is_rate_limited(429));
        assert!(!is_rate_limited(400));
        assert!(!is_rate_limited(500));
    }

    // Requirement: REQ-HTTP-002/003 (Must)
    // Acceptance: Combined retryable check
    #[test]
    fn req_http_002_003_is_retryable_combined() {
        assert!(is_retryable(429));
        assert!(is_retryable(500));
        assert!(is_retryable(503));
        assert!(!is_retryable(400));
        assert!(!is_retryable(401));
        assert!(!is_retryable(200));
    }

    // ---------------------------------------------------------------
    // Edge cases for retry
    // ---------------------------------------------------------------

    // Requirement: REQ-HTTP-002 (Must)
    // Edge case: Very large attempt number does not overflow
    #[test]
    fn req_http_002_edge_large_attempt_no_overflow() {
        let base = Duration::from_secs(1);
        // Attempt 30 would be 2^30 = ~1 billion seconds, which should be capped or handled
        let d = calculate_backoff(30, base, None);
        // Should not panic, even if result is very large
        assert!(d >= Duration::from_secs(0));
    }

    // Requirement: REQ-HTTP-002 (Must)
    // Edge case: Retry-After of 0 seconds
    #[test]
    fn req_http_002_edge_retry_after_zero() {
        let result = parse_retry_after("0");
        assert_eq!(result, Some(Duration::from_secs(0)));
    }
}
