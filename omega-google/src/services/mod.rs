pub mod common;
pub mod pagination;
pub mod export;
pub mod gmail;
pub mod calendar;
pub mod drive;
pub mod docs;
pub mod slides;
pub mod sheets;
pub mod forms;
pub mod chat;
pub mod tasks;
pub mod classroom;
pub mod contacts;
pub mod people;
pub mod groups;
pub mod keep;
pub mod appscript;

use std::sync::Arc;

use crate::http::circuit_breaker::CircuitBreaker;
use crate::http::RetryConfig;
use crate::output::{OutputMode, JsonTransform};
use crate::ui::Ui;
use crate::cli::root::RootFlags;

/// Shared context passed to all service handlers.
pub struct ServiceContext {
    pub client: reqwest::Client,
    pub output_mode: OutputMode,
    pub json_transform: JsonTransform,
    pub ui: Ui,
    pub flags: RootFlags,
    /// Shared circuit breaker for all API calls in this invocation.
    pub circuit_breaker: Arc<CircuitBreaker>,
    /// Retry configuration for API calls.
    pub retry_config: RetryConfig,
    /// Resolved account email.
    pub email: String,
}

impl ServiceContext {
    /// Write output in the appropriate format.
    pub fn write_output<T: serde::Serialize>(
        &self,
        value: &T,
    ) -> anyhow::Result<()> {
        match self.output_mode {
            OutputMode::Json => {
                crate::output::write_json(&mut std::io::stdout(), value, &self.json_transform)
            }
            OutputMode::Plain | OutputMode::Text => {
                let json_value = serde_json::to_value(value)?;
                let rows = crate::output::plain::json_to_plain_rows(&json_value);
                crate::output::write_plain(&mut std::io::stdout(), &rows)
            }
            OutputMode::Csv => {
                let json_value = serde_json::to_value(value)?;
                let rows = crate::output::plain::json_to_plain_rows(&json_value);
                if rows.is_empty() {
                    return Ok(());
                }
                let (headers, data) = rows.split_first().unwrap();
                crate::output::write_csv(&mut std::io::stdout(), headers, data)
            }
        }
    }

    /// Write paginated output with nextPageToken hint on stderr.
    pub fn write_paginated<T: serde::Serialize>(
        &self,
        value: &T,
        next_page_token: Option<&str>,
    ) -> anyhow::Result<()> {
        self.write_output(value)?;
        if let Some(token) = next_page_token {
            self.ui.hint(&format!("Next page: --page {}", token));
        }
        Ok(())
    }

    /// Check if this is a dry-run.
    pub fn is_dry_run(&self) -> bool {
        self.flags.dry_run
    }

    /// Check if force mode is enabled.
    pub fn is_force(&self) -> bool {
        self.flags.force
    }

    /// Check if verbose mode is enabled.
    pub fn is_verbose(&self) -> bool {
        self.flags.verbose
    }

    /// Get the account identifier.
    pub fn account(&self) -> Option<&str> {
        self.flags.account.as_deref()
    }
}

/// Bootstrap authentication and build a ServiceContext.
///
/// 1. Load config
/// 2. Build credential store (via factory)
/// 3. Resolve account (flag > env > default > single)
/// 4. Load token from store
/// 5. Check if refresh needed, refresh if so
/// 6. Build authenticated reqwest::Client
/// 7. Build ServiceContext
pub async fn bootstrap_service_context(
    flags: &RootFlags,
) -> anyhow::Result<ServiceContext> {
    // 1. Load config
    let cfg = crate::config::read_config()?;

    // 2. Build credential store
    let store = crate::auth::keyring::credential_store_factory(&cfg)?;

    // 3. Resolve client name
    let client_name = flags
        .client
        .as_deref()
        .unwrap_or(crate::config::DEFAULT_CLIENT_NAME);

    // 4. Resolve account
    let email = crate::auth::resolve_account(
        flags.account.as_deref(),
        &cfg,
        store.as_ref(),
        client_name,
    )?;

    // 5. Load token from store
    let mut token = store.get_token(client_name, &email)?;

    // 6. Refresh if needed
    if crate::auth::token::needs_refresh(&token) {
        let creds = crate::config::read_client_credentials(client_name)?;
        let bare_http = crate::http::client::build_client()?;
        let resp = crate::auth::token::refresh_access_token(&bare_http, &creds, &token.refresh_token).await?;

        token.access_token = Some(resp.access_token.clone());
        if let Some(expires_in) = resp.expires_in {
            token.expires_at = Some(chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64));
        }
        // Persist the refreshed token
        store.set_token(client_name, &email, &token)?;
    }

    // 7. Build authenticated HTTP client
    let access_token = token
        .access_token
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!(
            "No access token available for {}. Re-authenticate with: omega-google auth add",
            email
        ))?;
    let client = crate::http::client::build_authenticated_client(access_token)?;

    // 8. Resolve output mode
    let is_tty = crate::ui::Ui::is_tty_stdout();
    let output_mode = crate::output::resolve_mode_full(flags.json, flags.plain, flags.csv, is_tty)?;

    // 9. Build JSON transform
    let json_transform = crate::output::JsonTransform {
        results_only: flags.results_only,
        select: flags
            .select
            .as_deref()
            .map(|s| s.split(',').map(|f| f.trim().to_string()).collect())
            .unwrap_or_default(),
    };

    // 10. Build UI
    let color_mode: crate::ui::ColorMode = flags.color.parse().unwrap_or(crate::ui::ColorMode::Auto);
    let ui = Ui::new(crate::ui::UiOptions { color: color_mode })?;

    Ok(ServiceContext {
        client,
        output_mode,
        json_transform,
        ui,
        flags: flags.clone(),
        circuit_breaker: Arc::new(CircuitBreaker::new()),
        retry_config: RetryConfig::default(),
        email,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{Ui, UiOptions, ColorMode};

    // Helper: construct a minimal ServiceContext for testing.
    fn test_context() -> ServiceContext {
        let flags = RootFlags {
            verbose: false,
            dry_run: false,
            force: false,
            ..Default::default()
        };
        let ui = Ui::new(UiOptions { color: ColorMode::Never }).unwrap();
        ServiceContext {
            client: reqwest::Client::new(),
            output_mode: OutputMode::Json,
            json_transform: JsonTransform::default(),
            ui,
            flags,
            circuit_breaker: Arc::new(CircuitBreaker::new()),
            retry_config: RetryConfig::default(),
            email: "test@example.com".to_string(),
        }
    }

    fn test_context_with_flags(verbose: bool, dry_run: bool, force: bool) -> ServiceContext {
        let flags = RootFlags {
            verbose,
            dry_run,
            force,
            ..Default::default()
        };
        let ui = Ui::new(UiOptions { color: ColorMode::Never }).unwrap();
        ServiceContext {
            client: reqwest::Client::new(),
            output_mode: OutputMode::Json,
            json_transform: JsonTransform::default(),
            ui,
            flags,
            circuit_breaker: Arc::new(CircuitBreaker::new()),
            retry_config: RetryConfig::default(),
            email: "test@example.com".to_string(),
        }
    }

    // ===================================================================
    // REQ-RT-018 (Must): ServiceContext factory with auth bootstrap
    // ===================================================================

    // Requirement: REQ-RT-018 (Must)
    // Acceptance: ServiceContext can be constructed with all required fields
    #[test]
    fn req_rt_018_service_context_construction_all_fields() {
        let ctx = test_context();
        assert_eq!(ctx.email, "test@example.com");
        assert_eq!(ctx.output_mode, OutputMode::Json);
        assert!(!ctx.flags.verbose);
        assert!(!ctx.flags.dry_run);
    }

    // Requirement: REQ-RT-018 (Must)
    // Acceptance: ServiceContext has circuit_breaker field (Arc-wrapped)
    #[test]
    fn req_rt_018_service_context_has_circuit_breaker() {
        let ctx = test_context();
        // Verify circuit breaker starts closed
        assert!(!ctx.circuit_breaker.is_open());
    }

    // Requirement: REQ-RT-018 (Must)
    // Acceptance: ServiceContext has retry_config field
    #[test]
    fn req_rt_018_service_context_has_retry_config() {
        let ctx = test_context();
        assert_eq!(ctx.retry_config.max_retries_429, 3);
        assert_eq!(ctx.retry_config.max_retries_5xx, 1);
    }

    // Requirement: REQ-RT-018 (Must)
    // Acceptance: ServiceContext has email field
    #[test]
    fn req_rt_018_service_context_has_email() {
        let ctx = test_context();
        assert_eq!(ctx.email, "test@example.com");
    }

    // Requirement: REQ-RT-018 (Must)
    // Acceptance: is_dry_run reflects flags
    #[test]
    fn req_rt_018_is_dry_run_accessor() {
        let ctx = test_context_with_flags(false, true, false);
        assert!(ctx.is_dry_run());

        let ctx = test_context_with_flags(false, false, false);
        assert!(!ctx.is_dry_run());
    }

    // Requirement: REQ-RT-018 (Must)
    // Acceptance: is_verbose reflects flags
    #[test]
    fn req_rt_018_is_verbose_accessor() {
        let ctx = test_context_with_flags(true, false, false);
        assert!(ctx.is_verbose());

        let ctx = test_context_with_flags(false, false, false);
        assert!(!ctx.is_verbose());
    }

    // Requirement: REQ-RT-018 (Must)
    // Acceptance: is_force reflects flags
    #[test]
    fn req_rt_018_is_force_accessor() {
        let ctx = test_context_with_flags(false, false, true);
        assert!(ctx.is_force());

        let ctx = test_context_with_flags(false, false, false);
        assert!(!ctx.is_force());
    }

    // Requirement: REQ-RT-018 (Must)
    // Acceptance: account returns flag value
    #[test]
    fn req_rt_018_account_accessor() {
        let ctx = test_context();
        assert_eq!(ctx.account(), None);

        let flags = RootFlags {
            account: Some("user@gmail.com".to_string()),
            ..Default::default()
        };
        let ui = Ui::new(UiOptions { color: ColorMode::Never }).unwrap();
        let ctx = ServiceContext {
            client: reqwest::Client::new(),
            output_mode: OutputMode::Json,
            json_transform: JsonTransform::default(),
            ui,
            flags,
            circuit_breaker: Arc::new(CircuitBreaker::new()),
            retry_config: RetryConfig::default(),
            email: "user@gmail.com".to_string(),
        };
        assert_eq!(ctx.account(), Some("user@gmail.com"));
    }

    // ===================================================================
    // REQ-RT-017 (Must): Auth bootstrap
    // ===================================================================

    // Requirement: REQ-RT-017 (Must)
    // Acceptance: bootstrap_service_context function signature exists
    #[tokio::test]
    async fn req_rt_017_bootstrap_function_exists() {
        // Use empty config dir + file backend to isolate from real OS keychain
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("OMEGA_GOOGLE_CONFIG_DIR", tmp.path());
        std::env::set_var("GOG_KEYRING_BACKEND", "file");
        let flags = RootFlags::default();
        let result = bootstrap_service_context(&flags).await;
        std::env::remove_var("GOG_KEYRING_BACKEND");
        std::env::remove_var("OMEGA_GOOGLE_CONFIG_DIR");
        assert!(result.is_err(), "Should return error without configured account");
    }

    // Requirement: REQ-RT-017 (Must)
    // Acceptance: bootstrap returns error when no account is configured
    #[tokio::test]
    async fn req_rt_017_bootstrap_no_account_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("OMEGA_GOOGLE_CONFIG_DIR", tmp.path());
        std::env::set_var("GOG_KEYRING_BACKEND", "file");
        let flags = RootFlags::default();
        let result = bootstrap_service_context(&flags).await;
        std::env::remove_var("GOG_KEYRING_BACKEND");
        std::env::remove_var("OMEGA_GOOGLE_CONFIG_DIR");
        assert!(result.is_err());
    }

    // Requirement: REQ-RT-017 (Must)
    // Acceptance: bootstrap returns error when --account specified but not found
    #[tokio::test]
    async fn req_rt_017_bootstrap_missing_account_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("OMEGA_GOOGLE_CONFIG_DIR", tmp.path());
        std::env::set_var("GOG_KEYRING_BACKEND", "file");
        let flags = RootFlags {
            account: Some("nonexistent@example.com".to_string()),
            ..Default::default()
        };
        let result = bootstrap_service_context(&flags).await;
        std::env::remove_var("GOG_KEYRING_BACKEND");
        std::env::remove_var("OMEGA_GOOGLE_CONFIG_DIR");
        assert!(result.is_err());
    }

    // ===================================================================
    // REQ-RT-021 (Must): Single shared CircuitBreaker per CLI invocation
    // ===================================================================

    // Requirement: REQ-RT-021 (Must)
    // Acceptance: CircuitBreaker in ServiceContext is Arc-wrapped for sharing
    #[test]
    fn req_rt_021_circuit_breaker_is_arc_wrapped() {
        let ctx = test_context();
        // Can clone the Arc to share across tasks
        let breaker_clone = Arc::clone(&ctx.circuit_breaker);
        // Both references point to the same breaker
        breaker_clone.record_failure();
        // Original ctx sees the failure
        // (Verifies it's the same instance via Arc)
        assert!(!ctx.circuit_breaker.is_open(), "1 failure should not open");

        for _ in 0..4 {
            breaker_clone.record_failure();
        }
        assert!(ctx.circuit_breaker.is_open(), "5 failures should open");
    }

    // Requirement: REQ-RT-021 (Must)
    // Acceptance: Multiple ServiceContext operations share the same breaker state
    #[test]
    fn req_rt_021_shared_breaker_across_operations() {
        let breaker = Arc::new(CircuitBreaker::new());
        let breaker_ref1 = Arc::clone(&breaker);
        let breaker_ref2 = Arc::clone(&breaker);

        // Simulate failures from different "operations"
        for _ in 0..3 {
            breaker_ref1.record_failure();
        }
        for _ in 0..2 {
            breaker_ref2.record_failure();
        }
        // Total of 5 failures: circuit should be open
        assert!(breaker.is_open());
    }

    // ===================================================================
    // REQ-RT-018 (Must): Output mode construction
    // ===================================================================

    // Requirement: REQ-RT-018 (Must)
    // Acceptance: ServiceContext with different output modes
    #[test]
    fn req_rt_018_service_context_output_modes() {
        let modes = [OutputMode::Json, OutputMode::Plain, OutputMode::Text, OutputMode::Csv];
        for mode in &modes {
            let ui = Ui::new(UiOptions { color: ColorMode::Never }).unwrap();
            let ctx = ServiceContext {
                client: reqwest::Client::new(),
                output_mode: *mode,
                json_transform: JsonTransform::default(),
                ui,
                flags: RootFlags::default(),
                circuit_breaker: Arc::new(CircuitBreaker::new()),
                retry_config: RetryConfig::default(),
                email: "test@example.com".to_string(),
            };
            assert_eq!(ctx.output_mode, *mode);
        }
    }

    // Requirement: REQ-RT-018 (Must)
    // Acceptance: JsonTransform constructed from flags
    #[test]
    fn req_rt_018_json_transform_from_flags() {
        let transform = JsonTransform {
            results_only: true,
            select: vec!["id".to_string(), "name".to_string()],
        };
        assert!(transform.results_only);
        assert_eq!(transform.select.len(), 2);
        assert_eq!(transform.select[0], "id");
        assert_eq!(transform.select[1], "name");
    }

    // Requirement: REQ-RT-018 (Must)
    // Edge case: Empty select list
    #[test]
    fn req_rt_018_json_transform_empty_select() {
        let transform = JsonTransform {
            results_only: false,
            select: vec![],
        };
        assert!(!transform.results_only);
        assert!(transform.select.is_empty());
    }

    // ===================================================================
    // REQ-RT-017 (Must): Failure mode tests
    // ===================================================================

    // Requirement: REQ-RT-017 (Must)
    // Failure mode: No credential file -> error with auth guidance
    #[tokio::test]
    async fn req_rt_017_failure_no_credentials_file() {
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("OMEGA_GOOGLE_CONFIG_DIR", tmp.path());
        std::env::set_var("GOG_KEYRING_BACKEND", "file");
        let flags = RootFlags::default();
        let result = bootstrap_service_context(&flags).await;
        std::env::remove_var("GOG_KEYRING_BACKEND");
        std::env::remove_var("OMEGA_GOOGLE_CONFIG_DIR");
        assert!(result.is_err());
    }

    // Requirement: REQ-RT-017 (Must)
    // Failure mode: Ambiguous account (no accounts in empty store)
    #[tokio::test]
    async fn req_rt_017_failure_ambiguous_account() {
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("OMEGA_GOOGLE_CONFIG_DIR", tmp.path());
        std::env::set_var("GOG_KEYRING_BACKEND", "file");
        let flags = RootFlags::default();
        let result = bootstrap_service_context(&flags).await;
        std::env::remove_var("GOG_KEYRING_BACKEND");
        std::env::remove_var("OMEGA_GOOGLE_CONFIG_DIR");
        assert!(result.is_err());
    }

    // ===================================================================
    // Edge cases for ServiceContext
    // ===================================================================

    // Requirement: REQ-RT-018 (Must)
    // Edge case: ServiceContext with all flags set
    #[test]
    fn req_rt_018_edge_all_flags_set() {
        let flags = RootFlags {
            json: true,
            verbose: true,
            dry_run: true,
            force: true,
            no_input: true,
            results_only: true,
            select: Some("id,name".to_string()),
            account: Some("user@example.com".to_string()),
            client: Some("default".to_string()),
            ..Default::default()
        };

        let ui = Ui::new(UiOptions { color: ColorMode::Never }).unwrap();
        let ctx = ServiceContext {
            client: reqwest::Client::new(),
            output_mode: OutputMode::Json,
            json_transform: JsonTransform {
                results_only: true,
                select: vec!["id".to_string(), "name".to_string()],
            },
            ui,
            flags,
            circuit_breaker: Arc::new(CircuitBreaker::new()),
            retry_config: RetryConfig::default(),
            email: "user@example.com".to_string(),
        };

        assert!(ctx.is_dry_run());
        assert!(ctx.is_verbose());
        assert!(ctx.is_force());
        assert_eq!(ctx.account(), Some("user@example.com"));
        assert_eq!(ctx.email, "user@example.com");
    }

    // Requirement: REQ-RT-018 (Must)
    // Edge case: Default flags
    #[test]
    fn req_rt_018_edge_default_flags() {
        let ctx = test_context();
        assert!(!ctx.is_dry_run());
        assert!(!ctx.is_verbose());
        assert!(!ctx.is_force());
        assert_eq!(ctx.account(), None);
    }
}
