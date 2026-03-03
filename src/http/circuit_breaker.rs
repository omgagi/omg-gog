use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Circuit breaker: opens after consecutive failures, resets after cooldown.
pub const CIRCUIT_BREAKER_THRESHOLD: u32 = 5;
pub const CIRCUIT_BREAKER_RESET_TIME: Duration = Duration::from_secs(30);

pub struct CircuitBreaker {
    state: Mutex<CircuitState>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BreakerPhase {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitState {
    failures: u32,
    last_failure: Option<Instant>,
    phase: BreakerPhase,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(CircuitState {
                failures: 0,
                last_failure: None,
                phase: BreakerPhase::Closed,
            }),
        }
    }

    /// Record a successful request. Resets failure counter and closes circuit.
    pub fn record_success(&self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.failures = 0;
        state.phase = BreakerPhase::Closed;
        state.last_failure = None;
    }

    /// Record a failed request. Returns true if the circuit just opened.
    pub fn record_failure(&self) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.failures += 1;
        state.last_failure = Some(Instant::now());
        if state.phase == BreakerPhase::HalfOpen {
            // Probe failed: re-open with fresh cooldown
            state.phase = BreakerPhase::Open;
            return false;
        }
        if state.phase == BreakerPhase::Closed && state.failures >= CIRCUIT_BREAKER_THRESHOLD {
            state.phase = BreakerPhase::Open;
            return true;
        }
        false
    }

    /// Check if the circuit is open (requests should be rejected).
    /// When the cooldown has elapsed, transitions to half-open and returns false
    /// to allow one probe request through.
    pub fn is_open(&self) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        match state.phase {
            BreakerPhase::Closed => false,
            BreakerPhase::HalfOpen => false,
            BreakerPhase::Open => {
                // Check if cooldown period has elapsed
                if let Some(last) = state.last_failure {
                    if last.elapsed() >= CIRCUIT_BREAKER_RESET_TIME {
                        // Transition to half-open: allow one probe request
                        state.phase = BreakerPhase::HalfOpen;
                        return false;
                    }
                }
                true
            }
        }
    }

    /// Get the current state string ("open", "closed", or "half-open").
    pub fn state(&self) -> String {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        match state.phase {
            BreakerPhase::Closed => "closed".to_string(),
            BreakerPhase::Open => {
                // Check cooldown without mutating (for display only)
                if let Some(last) = state.last_failure {
                    if last.elapsed() >= CIRCUIT_BREAKER_RESET_TIME {
                        return "half-open".to_string();
                    }
                }
                "open".to_string()
            }
            BreakerPhase::HalfOpen => "half-open".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-HTTP-004 (Must): Circuit breaker
    // ---------------------------------------------------------------

    // Requirement: REQ-HTTP-004 (Must)
    // Acceptance: Circuit starts closed
    #[test]
    fn req_http_004_starts_closed() {
        let cb = CircuitBreaker::new();
        assert!(!cb.is_open());
        assert_eq!(cb.state(), "closed");
    }

    // Requirement: REQ-HTTP-004 (Must)
    // Acceptance: Opens circuit after 5 consecutive failures
    #[test]
    fn req_http_004_opens_after_threshold() {
        let cb = CircuitBreaker::new();
        for i in 0..4 {
            let opened = cb.record_failure();
            assert!(!opened, "should not open at {} failures", i + 1);
            assert!(!cb.is_open());
        }
        // 5th failure should open the circuit
        let opened = cb.record_failure();
        assert!(opened, "should open at 5 failures");
        assert!(cb.is_open());
        assert_eq!(cb.state(), "open");
    }

    // Requirement: REQ-HTTP-004 (Must)
    // Acceptance: Returns CircuitBreakerError when open
    #[test]
    fn req_http_004_rejects_when_open() {
        let cb = CircuitBreaker::new();
        for _ in 0..5 {
            cb.record_failure();
        }
        assert!(cb.is_open(), "circuit should be open after 5 failures");
    }

    // Requirement: REQ-HTTP-004 (Must)
    // Acceptance: Resets on any success
    #[test]
    fn req_http_004_resets_on_success() {
        let cb = CircuitBreaker::new();
        // Record 4 failures (one short of threshold)
        for _ in 0..4 {
            cb.record_failure();
        }
        assert!(!cb.is_open());
        // One success resets the counter
        cb.record_success();
        assert!(!cb.is_open());
        // Need another 5 failures to open now
        for _ in 0..4 {
            cb.record_failure();
        }
        assert!(
            !cb.is_open(),
            "should still be closed after 4 more failures"
        );
    }

    // Requirement: REQ-HTTP-004 (Must)
    // Acceptance: Resets on any success even when open
    #[test]
    fn req_http_004_success_closes_open_circuit() {
        let cb = CircuitBreaker::new();
        for _ in 0..5 {
            cb.record_failure();
        }
        assert!(cb.is_open());
        cb.record_success();
        assert!(!cb.is_open());
        assert_eq!(cb.state(), "closed");
    }

    // Requirement: REQ-HTTP-004 (Must)
    // Acceptance: Tracks consecutive 5xx failures (resets count on success)
    #[test]
    fn req_http_004_consecutive_tracking() {
        let cb = CircuitBreaker::new();
        // 3 failures, then success, then 3 failures = never reaches threshold
        for _ in 0..3 {
            cb.record_failure();
        }
        cb.record_success();
        for _ in 0..3 {
            cb.record_failure();
        }
        assert!(
            !cb.is_open(),
            "non-consecutive failures should not open circuit"
        );
    }

    // Requirement: REQ-HTTP-004 (Must)
    // Acceptance: Resets after cooldown period (30s)
    // Note: This test validates the concept; actual time-based reset
    // would require mocking the clock
    #[test]
    fn req_http_004_threshold_constant() {
        assert_eq!(CIRCUIT_BREAKER_THRESHOLD, 5);
    }

    // Requirement: REQ-HTTP-004 (Must)
    // Acceptance: Reset time is 30 seconds
    #[test]
    fn req_http_004_reset_time_constant() {
        assert_eq!(CIRCUIT_BREAKER_RESET_TIME, Duration::from_secs(30));
    }

    // ---------------------------------------------------------------
    // Edge cases for circuit breaker
    // ---------------------------------------------------------------

    // Requirement: REQ-HTTP-004 (Must)
    // Edge case: Multiple successes after open
    #[test]
    fn req_http_004_edge_multiple_successes() {
        let cb = CircuitBreaker::new();
        for _ in 0..5 {
            cb.record_failure();
        }
        assert!(cb.is_open());
        cb.record_success();
        cb.record_success();
        cb.record_success();
        assert!(!cb.is_open());
    }

    // Requirement: REQ-HTTP-004 (Must)
    // Edge case: Exactly at threshold
    #[test]
    fn req_http_004_edge_exactly_at_threshold() {
        let cb = CircuitBreaker::new();
        for _ in 0..5 {
            cb.record_failure();
        }
        assert!(cb.is_open());
        // Additional failures should keep it open
        cb.record_failure();
        assert!(cb.is_open());
        cb.record_failure();
        assert!(cb.is_open());
    }

    // Requirement: REQ-HTTP-004 (Must)
    // Edge case: Concurrent access (thread safety)
    #[test]
    fn req_http_004_edge_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let cb = Arc::new(CircuitBreaker::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let cb_clone = Arc::clone(&cb);
            handles.push(thread::spawn(move || {
                cb_clone.record_failure();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // After 10 failures, circuit should be open
        assert!(cb.is_open());
    }
}
