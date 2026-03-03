/// Stable exit codes matching gogcli conventions.
pub mod codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERIC_ERROR: i32 = 1;
    pub const USAGE_ERROR: i32 = 2;
    pub const EMPTY_RESULTS: i32 = 3;
    pub const AUTH_REQUIRED: i32 = 4;
    pub const NOT_FOUND: i32 = 5;
    pub const PERMISSION_DENIED: i32 = 6;
    pub const RATE_LIMITED: i32 = 7;
    pub const RETRYABLE: i32 = 8;
    pub const CONFIG_ERROR: i32 = 10;
    pub const CANCELLED: i32 = 130;
}

#[derive(Debug, thiserror::Error)]
pub enum OmegaError {
    #[error("authentication required: {message}")]
    AuthRequired { message: String },

    #[error("not found: {resource}")]
    NotFound { resource: String },

    #[error("permission denied: {message}")]
    PermissionDenied { message: String },

    #[error("rate limited")]
    RateLimited,

    #[error("circuit breaker open")]
    CircuitBreakerOpen,

    #[error("config error: {message}")]
    ConfigError { message: String },

    #[error("empty results")]
    EmptyResults,

    #[error("usage error: {message}")]
    UsageError { message: String },

    #[error("cancelled")]
    Cancelled,

    #[error("API error ({status}): {message}")]
    ApiError { status: u16, message: String },

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Map an OmegaError to its stable exit code.
pub fn exit_code_for(err: &OmegaError) -> i32 {
    match err {
        OmegaError::AuthRequired { .. } => codes::AUTH_REQUIRED,
        OmegaError::NotFound { .. } => codes::NOT_FOUND,
        OmegaError::PermissionDenied { .. } => codes::PERMISSION_DENIED,
        OmegaError::RateLimited => codes::RATE_LIMITED,
        OmegaError::CircuitBreakerOpen => codes::RETRYABLE,
        OmegaError::ConfigError { .. } => codes::CONFIG_ERROR,
        OmegaError::EmptyResults => codes::EMPTY_RESULTS,
        OmegaError::UsageError { .. } => codes::USAGE_ERROR,
        OmegaError::Cancelled => codes::CANCELLED,
        OmegaError::ApiError { status, .. } => match *status {
            401 => codes::AUTH_REQUIRED,
            403 => codes::PERMISSION_DENIED,
            404 => codes::NOT_FOUND,
            429 => codes::RATE_LIMITED,
            s if (500..600).contains(&s) => codes::RETRYABLE,
            _ => codes::GENERIC_ERROR,
        },
        _ => codes::GENERIC_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CLI-007 (Must): Stable exit codes
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: Exit 0 = success
    #[test]
    fn req_cli_007_success_code() {
        assert_eq!(codes::SUCCESS, 0);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: Exit 1 = generic error
    #[test]
    fn req_cli_007_generic_error_code() {
        assert_eq!(codes::GENERIC_ERROR, 1);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: Exit 2 = usage/parse error
    #[test]
    fn req_cli_007_usage_error_code() {
        assert_eq!(codes::USAGE_ERROR, 2);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: Exit 3 = empty results
    #[test]
    fn req_cli_007_empty_results_code() {
        assert_eq!(codes::EMPTY_RESULTS, 3);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: Exit 4 = auth required
    #[test]
    fn req_cli_007_auth_required_code() {
        assert_eq!(codes::AUTH_REQUIRED, 4);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: Exit 5 = not found
    #[test]
    fn req_cli_007_not_found_code() {
        assert_eq!(codes::NOT_FOUND, 5);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: Exit 6 = permission denied
    #[test]
    fn req_cli_007_permission_denied_code() {
        assert_eq!(codes::PERMISSION_DENIED, 6);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: Exit 7 = rate limited
    #[test]
    fn req_cli_007_rate_limited_code() {
        assert_eq!(codes::RATE_LIMITED, 7);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: Exit 8 = retryable error
    #[test]
    fn req_cli_007_retryable_code() {
        assert_eq!(codes::RETRYABLE, 8);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: Exit 10 = config error
    #[test]
    fn req_cli_007_config_error_code() {
        assert_eq!(codes::CONFIG_ERROR, 10);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: Exit 130 = cancelled (SIGINT)
    #[test]
    fn req_cli_007_cancelled_code() {
        assert_eq!(codes::CANCELLED, 130);
    }

    // ---------------------------------------------------------------
    // REQ-CLI-007 (Must): Error-to-exit-code mapping
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: AuthRequired maps to exit 4
    #[test]
    fn req_cli_007_auth_required_maps_to_4() {
        let err = OmegaError::AuthRequired {
            message: "test".to_string(),
        };
        assert_eq!(exit_code_for(&err), codes::AUTH_REQUIRED);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: NotFound maps to exit 5
    #[test]
    fn req_cli_007_not_found_maps_to_5() {
        let err = OmegaError::NotFound {
            resource: "file".to_string(),
        };
        assert_eq!(exit_code_for(&err), codes::NOT_FOUND);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: PermissionDenied maps to exit 6
    #[test]
    fn req_cli_007_permission_denied_maps_to_6() {
        let err = OmegaError::PermissionDenied {
            message: "no access".to_string(),
        };
        assert_eq!(exit_code_for(&err), codes::PERMISSION_DENIED);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: RateLimited maps to exit 7
    #[test]
    fn req_cli_007_rate_limited_maps_to_7() {
        let err = OmegaError::RateLimited;
        assert_eq!(exit_code_for(&err), codes::RATE_LIMITED);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: CircuitBreakerOpen maps to exit 8
    #[test]
    fn req_cli_007_circuit_breaker_maps_to_8() {
        let err = OmegaError::CircuitBreakerOpen;
        assert_eq!(exit_code_for(&err), codes::RETRYABLE);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: ConfigError maps to exit 10
    #[test]
    fn req_cli_007_config_error_maps_to_10() {
        let err = OmegaError::ConfigError {
            message: "bad config".to_string(),
        };
        assert_eq!(exit_code_for(&err), codes::CONFIG_ERROR);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: EmptyResults maps to exit 3
    #[test]
    fn req_cli_007_empty_results_maps_to_3() {
        let err = OmegaError::EmptyResults;
        assert_eq!(exit_code_for(&err), codes::EMPTY_RESULTS);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: UsageError maps to exit 2
    #[test]
    fn req_cli_007_usage_error_maps_to_2() {
        let err = OmegaError::UsageError {
            message: "bad input".to_string(),
        };
        assert_eq!(exit_code_for(&err), codes::USAGE_ERROR);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: Cancelled maps to exit 130
    #[test]
    fn req_cli_007_cancelled_maps_to_130() {
        let err = OmegaError::Cancelled;
        assert_eq!(exit_code_for(&err), codes::CANCELLED);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: API error with 403 maps to permission denied
    #[test]
    fn req_cli_007_api_403_maps_to_6() {
        let err = OmegaError::ApiError {
            status: 403,
            message: "forbidden".to_string(),
        };
        assert_eq!(exit_code_for(&err), codes::PERMISSION_DENIED);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: API error with 404 maps to not found
    #[test]
    fn req_cli_007_api_404_maps_to_5() {
        let err = OmegaError::ApiError {
            status: 404,
            message: "not found".to_string(),
        };
        assert_eq!(exit_code_for(&err), codes::NOT_FOUND);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: API error with 401 maps to auth required
    #[test]
    fn req_cli_007_api_401_maps_to_4() {
        let err = OmegaError::ApiError {
            status: 401,
            message: "unauthorized".to_string(),
        };
        assert_eq!(exit_code_for(&err), codes::AUTH_REQUIRED);
    }

    // Requirement: REQ-CLI-007 (Must)
    // Acceptance: API error with 429 maps to rate limited
    #[test]
    fn req_cli_007_api_429_maps_to_7() {
        let err = OmegaError::ApiError {
            status: 429,
            message: "too many requests".to_string(),
        };
        assert_eq!(exit_code_for(&err), codes::RATE_LIMITED);
    }
}
