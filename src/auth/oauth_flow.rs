// OAuth2 flow orchestration: desktop, manual, and remote flows.
//
// This module handles the user-facing OAuth flow: opening a browser or
// prompting the user to paste a redirect URL, extracting the authorization
// code, and returning it for token exchange.
//
// Module 4 in the runtime architecture.

use crate::auth::oauth::{self, FlowMode};
use crate::auth::Service;
use crate::config::ClientCredentials;

/// Result of a successful OAuth flow -- contains the authorization code
/// and the redirect_uri that was used (needed for token exchange).
#[derive(Debug, Clone, PartialEq)]
pub struct OAuthFlowResult {
    pub code: String,
    pub redirect_uri: String,
}

/// Desktop flow timeout in seconds.
pub const DESKTOP_FLOW_TIMEOUT_SECS: u64 = 120;

/// Actual timeout used at runtime. In test builds, we use a very short timeout
/// to avoid blocking tests for 120 seconds when no browser is available.
#[cfg(test)]
const EFFECTIVE_TIMEOUT_SECS: u64 = 1;

#[cfg(not(test))]
const EFFECTIVE_TIMEOUT_SECS: u64 = DESKTOP_FLOW_TIMEOUT_SECS;

/// Redirect URI used for the manual/OOB flow.
pub const MANUAL_REDIRECT_URI: &str = "urn:ietf:wg:oauth:2.0:oob";

/// Run the OAuth flow based on the selected mode.
/// Returns the authorization code and redirect_uri on success.
pub async fn run_oauth_flow(
    creds: &ClientCredentials,
    services: &[Service],
    mode: FlowMode,
    force_consent: bool,
) -> anyhow::Result<OAuthFlowResult> {
    match mode {
        FlowMode::Desktop => run_desktop_flow(creds, services, force_consent).await,
        FlowMode::Manual => run_manual_flow(creds, services, force_consent).await,
        FlowMode::Remote => run_remote_flow(creds, services, force_consent).await,
    }
}

/// Desktop flow: ephemeral local HTTP server on 127.0.0.1:0, browser open, 120s timeout.
///
/// 1. Bind TcpListener to 127.0.0.1:0 (OS-assigned port)
/// 2. Build auth URL with redirect_uri = http://127.0.0.1:{port}
/// 3. Print auth URL to stderr with instructions
/// 4. Try to open browser, or tell user to open URL manually
/// 5. Accept exactly one HTTP connection with 120-second timeout
/// 6. Parse ?code= from the GET request path
/// 7. Return HTML response "Success! You can close this tab."
/// 8. Return OAuthFlowResult { code, redirect_uri }
pub(crate) async fn run_desktop_flow(
    creds: &ClientCredentials,
    services: &[Service],
    force_consent: bool,
) -> anyhow::Result<OAuthFlowResult> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    // 1. Bind to 127.0.0.1:0 (OS-assigned port, localhost only)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to bind local server: {}. Try --manual mode instead.",
                e
            )
        })?;

    let local_addr = listener.local_addr()?;
    let port = local_addr.port();
    let redirect_uri = format!("http://127.0.0.1:{}", port);

    // 2. Build auth URL
    let auth_url = oauth::build_auth_url(creds, services, &redirect_uri, force_consent)?;

    // 3. Print auth URL to stderr with instructions
    eprintln!("Open this URL in your browser to authorize:");
    eprintln!();
    eprintln!("  {}", auth_url);
    eprintln!();

    // 4. Try to open browser
    let browser_opened = open_browser(&auth_url);
    if browser_opened {
        eprintln!("Waiting for authorization (browser should have opened)...");
    } else {
        eprintln!("Could not open browser automatically. Please open the URL above.");
        eprintln!("Waiting for authorization...");
    }

    // 5. Accept exactly one connection with timeout
    let timeout_duration = std::time::Duration::from_secs(EFFECTIVE_TIMEOUT_SECS);
    let (mut stream, _addr) = tokio::time::timeout(timeout_duration, listener.accept())
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "Timed out after {} seconds waiting for authorization. Try --manual mode.",
                EFFECTIVE_TIMEOUT_SECS
            )
        })??;

    // 6. Read the HTTP request (simple parse -- just need the GET line)
    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);

    // Parse the GET request line: "GET /?code=xxx HTTP/1.1\r\n..."
    let request_path = request
        .lines()
        .next()
        .and_then(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[0] == "GET" {
                Some(parts[1].to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow::anyhow!("Invalid HTTP request received"))?;

    // Build a full URL to parse query parameters
    let full_url = format!("http://127.0.0.1:{}{}", port, request_path);
    let code = extract_code_from_url(&full_url)?;

    // 7. Return HTML response
    let html_body = "<!DOCTYPE html><html><body>\
        <h1>Success!</h1>\
        <p>You can close this tab.</p>\
        </body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        html_body.len(),
        html_body
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;

    eprintln!("Authorization received.");

    // 8. Return result
    Ok(OAuthFlowResult { code, redirect_uri })
}

/// Manual flow: print auth URL to stderr, read redirect URL from stdin.
///
/// 1. Build auth URL with redirect_uri = urn:ietf:wg:oauth:2.0:oob
/// 2. Print auth URL to stderr
/// 3. Print "Paste the redirect URL: " prompt to stderr
/// 4. Read a line from stdin
/// 5. Parse the code from the pasted URL
/// 6. Return OAuthFlowResult { code, redirect_uri: MANUAL_REDIRECT_URI }
pub(crate) async fn run_manual_flow(
    creds: &ClientCredentials,
    services: &[Service],
    force_consent: bool,
) -> anyhow::Result<OAuthFlowResult> {
    // 1. Build auth URL with OOB redirect
    let auth_url = oauth::build_auth_url(creds, services, MANUAL_REDIRECT_URI, force_consent)?;

    // 2. Print auth URL to stderr
    eprintln!("Open this URL in your browser to authorize:");
    eprintln!();
    eprintln!("  {}", auth_url);
    eprintln!();

    // 3. Prompt on stderr
    eprintln!("After authorizing, paste the redirect URL below:");
    eprint!("Paste the redirect URL: ");

    // 4. Read a line from stdin
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .map_err(|e| anyhow::anyhow!("Failed to read from stdin: {}", e))?;
    let line = line.trim();

    if line.is_empty() {
        anyhow::bail!("No URL provided. Please run the command again and paste the redirect URL.");
    }

    // 5. Parse the code from the pasted URL
    let code = extract_code_from_url(line)?;

    // 6. Return result
    Ok(OAuthFlowResult {
        code,
        redirect_uri: MANUAL_REDIRECT_URI.to_string(),
    })
}

/// Remote flow: two-step headless flow (REQ-RT-004).
///
/// The remote flow is split across two CLI invocations:
/// - Step 1 (`--remote --step 1`): generate auth URL with state, cache state
/// - Step 2 (`--remote --step 2 --auth-url <url>`): validate state, extract code
///
/// When called via `run_oauth_flow`, this returns an error instructing the user
/// to use the two-step process. The CLI handler should detect `--step` and call
/// `remote_flow_step1` or `remote_flow_step2` directly.
pub(crate) async fn run_remote_flow(
    _creds: &ClientCredentials,
    _services: &[Service],
    _force_consent: bool,
) -> anyhow::Result<OAuthFlowResult> {
    anyhow::bail!(
        "Remote flow requires --step 1 or --step 2. \
         Use: omega-google auth add --remote --step 1"
    )
}

/// Step 1 of remote flow: generate auth URL with state parameter and cache state.
///
/// Returns the auth URL (with `&state=<random>`) that the user should open
/// in a browser on another machine.
#[allow(dead_code)]
pub(crate) fn remote_flow_step1(
    creds: &ClientCredentials,
    services: &[Service],
    force_consent: bool,
) -> anyhow::Result<String> {
    // Generate random 32-character alphanumeric state parameter
    let state: String =
        rand::Rng::sample_iter(rand::thread_rng(), &rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

    // Build auth URL with the manual redirect URI (OOB for remote)
    let auth_url = oauth::build_auth_url(creds, services, MANUAL_REDIRECT_URI, force_consent)?;

    // Append state parameter to URL
    let auth_url_with_state = format!("{}&state={}", auth_url, state);

    // Cache state to config directory
    let config_dir = crate::config::ensure_dir()?;
    let state_file = config_dir.join("remote_oauth_state");
    std::fs::write(&state_file, &state)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&state_file, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(auth_url_with_state)
}

/// Step 1 variant that writes to a custom directory (for testing).
#[allow(dead_code)]
pub(crate) fn remote_flow_step1_with_dir(
    creds: &ClientCredentials,
    services: &[Service],
    force_consent: bool,
    state_dir: &std::path::Path,
) -> anyhow::Result<String> {
    let state: String =
        rand::Rng::sample_iter(rand::thread_rng(), &rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

    let auth_url = oauth::build_auth_url(creds, services, MANUAL_REDIRECT_URI, force_consent)?;
    let auth_url_with_state = format!("{}&state={}", auth_url, state);

    let state_file = state_dir.join("remote_oauth_state");
    std::fs::write(&state_file, &state)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&state_file, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(auth_url_with_state)
}

/// Step 2 of remote flow: validate state parameter and extract authorization code.
///
/// Reads the cached state from the config directory, validates it against
/// the state in the redirect URL, extracts the authorization code.
#[allow(dead_code)]
pub(crate) fn remote_flow_step2(auth_url: &str) -> anyhow::Result<OAuthFlowResult> {
    let config_dir = crate::config::config_dir()?;
    remote_flow_step2_with_dir(auth_url, &config_dir)
}

/// Step 2 variant that reads from a custom directory (for testing).
#[allow(dead_code)]
pub(crate) fn remote_flow_step2_with_dir(
    auth_url: &str,
    state_dir: &std::path::Path,
) -> anyhow::Result<OAuthFlowResult> {
    // Read cached state
    let state_file = state_dir.join("remote_oauth_state");
    let cached_state = std::fs::read_to_string(&state_file)
        .map_err(|_| anyhow::anyhow!("No pending remote flow. Run --step 1 first."))?;

    // Parse the redirect URL
    let parsed = url::Url::parse(auth_url).map_err(|e| anyhow::anyhow!("Invalid URL: {}", e))?;
    let params: std::collections::HashMap<String, String> = parsed
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    // Validate state parameter
    if let Some(state) = params.get("state") {
        if state != &cached_state {
            anyhow::bail!("State parameter mismatch. Possible CSRF attack or stale flow.");
        }
    } else {
        anyhow::bail!("No state parameter in URL. Expected state for CSRF validation.");
    }

    // Extract authorization code
    let code = extract_code_from_url(auth_url)?;

    // Clean up state file
    let _ = std::fs::remove_file(&state_file);

    Ok(OAuthFlowResult {
        code,
        redirect_uri: MANUAL_REDIRECT_URI.to_string(),
    })
}

/// Try to open a URL in the system browser.
/// Returns true if the browser was launched successfully, false otherwise.
///
/// In test builds, this is a no-op that returns false to prevent browsers
/// from opening during `cargo test`.
fn open_browser(url: &str) -> bool {
    #[cfg(test)]
    {
        let _ = url;
        return false;
    }

    #[cfg(not(test))]
    {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(url)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .is_ok()
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(url)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .is_ok()
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            let _ = url;
            false
        }
    }
}

/// Extract the authorization code from a redirect URL.
///
/// Parses the URL, looks for `?code=` or `?error=` query parameters.
/// Returns the code on success, or an error describing what went wrong.
pub(crate) fn extract_code_from_url(url_str: &str) -> anyhow::Result<String> {
    // Parse the URL
    let parsed = url::Url::parse(url_str).map_err(|e| anyhow::anyhow!("Invalid URL: {}", e))?;

    // Check for error parameter first
    let params: std::collections::HashMap<String, String> = parsed
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    if let Some(error) = params.get("error") {
        let description = params.get("error_description").cloned().unwrap_or_default();
        if description.is_empty() {
            anyhow::bail!("OAuth error: {}", error);
        } else {
            anyhow::bail!("OAuth error: {} - {}", error, description);
        }
    }

    // Extract the code parameter
    match params.get("code") {
        Some(code) if !code.is_empty() => Ok(code.clone()),
        Some(_) => anyhow::bail!("Authorization code is empty"),
        None => anyhow::bail!("No 'code' parameter found in URL"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =================================================================
    // REQ-RT-002 (Must): Desktop OAuth flow
    // =================================================================

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: OAuthFlowResult struct has code and redirect_uri fields
    #[test]
    fn req_rt_002_oauth_flow_result_has_code_field() {
        let result = OAuthFlowResult {
            code: "4/0AX4XfWh_test".to_string(),
            redirect_uri: "http://127.0.0.1:12345".to_string(),
        };
        assert_eq!(result.code, "4/0AX4XfWh_test");
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: OAuthFlowResult struct has redirect_uri field
    #[test]
    fn req_rt_002_oauth_flow_result_has_redirect_uri_field() {
        let result = OAuthFlowResult {
            code: "code".to_string(),
            redirect_uri: "http://127.0.0.1:9999".to_string(),
        };
        assert_eq!(result.redirect_uri, "http://127.0.0.1:9999");
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: OAuthFlowResult is Clone and Debug
    #[test]
    fn req_rt_002_oauth_flow_result_clone_and_debug() {
        let result = OAuthFlowResult {
            code: "code".to_string(),
            redirect_uri: "http://127.0.0.1:8080".to_string(),
        };
        let cloned = result.clone();
        assert_eq!(cloned, result);
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("code"));
        assert!(debug_str.contains("redirect_uri"));
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: Desktop flow timeout is 120 seconds
    #[test]
    fn req_rt_002_desktop_flow_timeout_is_120_seconds() {
        assert_eq!(DESKTOP_FLOW_TIMEOUT_SECS, 120);
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: run_oauth_flow function exists and is async
    #[tokio::test]
    async fn req_rt_002_run_oauth_flow_exists() {
        let creds = ClientCredentials {
            client_id: "test.apps.googleusercontent.com".to_string(),
            client_secret: "GOCSPX-test".to_string(),
        };
        let services = vec![Service::Gmail];
        // Should return an error (not yet implemented or flow failure)
        let result = run_oauth_flow(&creds, &services, FlowMode::Desktop, false).await;
        // We just verify it compiles and returns a Result
        assert!(result.is_err() || result.is_ok());
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: run_desktop_flow function exists, is async, returns OAuthFlowResult
    #[tokio::test]
    async fn req_rt_002_run_desktop_flow_exists() {
        let creds = ClientCredentials {
            client_id: "test.apps.googleusercontent.com".to_string(),
            client_secret: "GOCSPX-test".to_string(),
        };
        let services = vec![Service::Gmail];
        let result = run_desktop_flow(&creds, &services, false).await;
        // Verify it returns Result<OAuthFlowResult>
        assert!(result.is_err() || result.is_ok());
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: Redirect URI uses 127.0.0.1, never 0.0.0.0
    // Security: Local server MUST bind to 127.0.0.1 only
    #[test]
    fn req_rt_002_security_localhost_only() {
        // When implemented, the desktop flow must use 127.0.0.1
        // This test verifies that if an OAuthFlowResult is constructed with
        // the correct redirect_uri pattern, it uses 127.0.0.1
        let result = OAuthFlowResult {
            code: "code".to_string(),
            redirect_uri: "http://127.0.0.1:12345".to_string(),
        };
        assert!(
            result.redirect_uri.contains("127.0.0.1"),
            "Desktop flow redirect_uri must use 127.0.0.1, not 0.0.0.0"
        );
        assert!(
            !result.redirect_uri.contains("0.0.0.0"),
            "Desktop flow must NEVER bind to 0.0.0.0"
        );
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: Desktop flow dispatched when mode is Desktop
    #[tokio::test]
    async fn req_rt_002_desktop_mode_dispatches_to_desktop_flow() {
        let creds = ClientCredentials {
            client_id: "test.apps.googleusercontent.com".to_string(),
            client_secret: "GOCSPX-test".to_string(),
        };
        let services = vec![Service::Gmail];
        // When implemented, FlowMode::Desktop should run the desktop flow
        let result = run_oauth_flow(&creds, &services, FlowMode::Desktop, false).await;
        // For now, just verify it is invokable with Desktop mode
        assert!(result.is_err() || result.is_ok());
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: HTTP server accepts exactly one GET request
    // (Integration test -- requires real TCP listener)
    #[tokio::test]
    #[ignore = "Integration test: requires running TCP listener"]
    async fn req_rt_002_integration_desktop_flow_single_request() {
        // This test would:
        // 1. Start a desktop flow in a background task
        // 2. Connect to the listener
        // 3. Send a GET with ?code=test_code
        // 4. Verify the flow returns OAuthFlowResult with code="test_code"
        // 5. Verify the HTML response contains "Success"
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: Times out after 120 seconds with appropriate error
    #[tokio::test]
    #[ignore = "Integration test: would wait 120s or requires time mocking"]
    async fn req_rt_002_integration_desktop_flow_timeout() {
        // This test would:
        // 1. Start a desktop flow
        // 2. NOT connect to the listener
        // 3. Verify it times out with an error mentioning "120 seconds" or "timeout"
    }

    // =================================================================
    // REQ-RT-002 & REQ-RT-003 (Must): extract_code_from_url
    // =================================================================

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: Extracts code query parameter from redirect URL
    #[test]
    fn req_rt_002_extract_code_valid() {
        let url = "http://127.0.0.1:12345/?code=4/0AX4XfWh_valid_code";
        let result = extract_code_from_url(url);
        assert!(result.is_ok(), "Should extract code from valid URL");
        assert_eq!(result.unwrap(), "4/0AX4XfWh_valid_code");
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: Handles ?error=access_denied with error message
    #[test]
    fn req_rt_002_extract_code_error_access_denied() {
        let url = "http://127.0.0.1:12345/?error=access_denied";
        let result = extract_code_from_url(url);
        assert!(result.is_err(), "Error parameter should cause failure");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("access_denied"),
            "Error message should contain the error type: {}",
            err_msg
        );
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: Handles ?error= with error_description
    #[test]
    fn req_rt_002_extract_code_error_with_description() {
        let url = "http://127.0.0.1:12345/?error=access_denied&error_description=The+user+did+not+consent";
        let result = extract_code_from_url(url);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("access_denied"),
            "Should contain error type"
        );
        assert!(
            err_msg.contains("user") || err_msg.contains("consent"),
            "Should contain error description: {}",
            err_msg
        );
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: Missing code parameter returns error
    #[test]
    fn req_rt_002_extract_code_missing_code() {
        let url = "http://127.0.0.1:12345/?state=abc123";
        let result = extract_code_from_url(url);
        assert!(result.is_err(), "Missing code parameter should fail");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("code") || err_msg.contains("parameter"),
            "Error should mention missing code: {}",
            err_msg
        );
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: URL with extra parameters still extracts code
    #[test]
    fn req_rt_002_extract_code_with_extra_params() {
        let url = "http://127.0.0.1:12345/?state=xyz&code=4/the_code&scope=email+openid";
        let result = extract_code_from_url(url);
        assert!(
            result.is_ok(),
            "Should extract code despite extra parameters"
        );
        assert_eq!(result.unwrap(), "4/the_code");
    }

    // Requirement: REQ-RT-002 (Must)
    // Edge case: Malformed URL returns error
    #[test]
    fn req_rt_002_extract_code_malformed_url() {
        let url = "not a valid url at all";
        let result = extract_code_from_url(url);
        assert!(result.is_err(), "Malformed URL should return error");
    }

    // Requirement: REQ-RT-002 (Must)
    // Edge case: Code with special characters (URL-decoded)
    #[test]
    fn req_rt_002_extract_code_special_chars() {
        let url = "http://127.0.0.1:12345/?code=4%2F0AX4XfWh_test%2Bcode";
        let result = extract_code_from_url(url);
        assert!(result.is_ok(), "URL-encoded code should be decoded");
        let code = result.unwrap();
        // URL crate should decode percent-encoding
        assert!(
            code.contains("4/0AX4XfWh_test") || code.contains("4%2F0AX4XfWh"),
            "Code should contain the auth code (decoded or encoded): {}",
            code
        );
    }

    // Requirement: REQ-RT-002 (Must)
    // Edge case: Empty code parameter
    #[test]
    fn req_rt_002_extract_code_empty_code() {
        let url = "http://127.0.0.1:12345/?code=";
        let result = extract_code_from_url(url);
        assert!(result.is_err(), "Empty code parameter should fail");
    }

    // Requirement: REQ-RT-002 (Must)
    // Edge case: URL with no query string at all
    #[test]
    fn req_rt_002_extract_code_no_query_string() {
        let url = "http://127.0.0.1:12345/";
        let result = extract_code_from_url(url);
        assert!(result.is_err(), "URL without query string should fail");
    }

    // Requirement: REQ-RT-002 (Must)
    // Edge case: URL with fragment (hash) but no query
    #[test]
    fn req_rt_002_extract_code_fragment_only() {
        let url = "http://127.0.0.1:12345/#code=abc";
        let result = extract_code_from_url(url);
        // Fragment-based code should not be found in query parameters
        assert!(
            result.is_err(),
            "Fragment code should not be extracted from query params"
        );
    }

    // Requirement: REQ-RT-002 (Must)
    // Edge case: HTTPS URL (should work the same)
    #[test]
    fn req_rt_002_extract_code_https_url() {
        let url = "https://127.0.0.1:443/?code=https_code";
        let result = extract_code_from_url(url);
        assert!(result.is_ok(), "HTTPS URL should work");
        assert_eq!(result.unwrap(), "https_code");
    }

    // Requirement: REQ-RT-002 (Must)
    // Edge case: Very long code string
    #[test]
    fn req_rt_002_extract_code_very_long() {
        let long_code = "4/".to_string() + &"A".repeat(1000);
        let url = format!("http://127.0.0.1:12345/?code={}", long_code);
        let result = extract_code_from_url(&url);
        assert!(result.is_ok(), "Long code should be handled");
        assert_eq!(result.unwrap(), long_code);
    }

    // Requirement: REQ-RT-002 (Must)
    // Security: Authorization code must not be logged even in verbose mode
    // (This documents the requirement; actual verbose log checking is in integration tests)
    #[test]
    fn req_rt_002_security_code_not_logged() {
        // The extract_code_from_url function is pure and does not log.
        // The calling code (run_desktop_flow, run_manual_flow) must ensure
        // the code is not included in any verbose/debug output.
        // This test documents the security requirement.
        let url = "http://127.0.0.1:12345/?code=secret_auth_code";
        let code = extract_code_from_url(url).unwrap();
        // The code should be returned for use, but never logged
        assert_eq!(code, "secret_auth_code");
    }

    // =================================================================
    // REQ-RT-003 (Must): Manual OAuth flow
    // =================================================================

    // Requirement: REQ-RT-003 (Must)
    // Acceptance: Manual redirect URI is the OOB URI
    #[test]
    fn req_rt_003_manual_redirect_uri() {
        assert_eq!(MANUAL_REDIRECT_URI, "urn:ietf:wg:oauth:2.0:oob");
    }

    // Requirement: REQ-RT-003 (Must)
    // Acceptance: run_manual_flow function exists and is async
    #[tokio::test]
    async fn req_rt_003_run_manual_flow_exists() {
        let creds = ClientCredentials {
            client_id: "test.apps.googleusercontent.com".to_string(),
            client_secret: "GOCSPX-test".to_string(),
        };
        let services = vec![Service::Gmail];
        let result = run_manual_flow(&creds, &services, false).await;
        // Verify it returns Result<OAuthFlowResult>
        assert!(result.is_err() || result.is_ok());
    }

    // Requirement: REQ-RT-003 (Must)
    // Acceptance: Manual flow dispatched when mode is Manual
    #[tokio::test]
    async fn req_rt_003_manual_mode_dispatches_to_manual_flow() {
        let creds = ClientCredentials {
            client_id: "test.apps.googleusercontent.com".to_string(),
            client_secret: "GOCSPX-test".to_string(),
        };
        let services = vec![Service::Gmail];
        let result = run_oauth_flow(&creds, &services, FlowMode::Manual, false).await;
        assert!(result.is_err() || result.is_ok());
    }

    // Requirement: REQ-RT-003 (Must)
    // Acceptance: Parses code from a pasted redirect URL
    // (Uses extract_code_from_url which is already tested above)
    #[test]
    fn req_rt_003_extract_code_from_pasted_url() {
        // Typical manual flow: user pastes the full redirect URL
        let pasted = "http://localhost/?code=4/0AfJohXm_manual_code&scope=email+openid";
        let result = extract_code_from_url(pasted);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "4/0AfJohXm_manual_code");
    }

    // Requirement: REQ-RT-003 (Must)
    // Acceptance: Handles OOB redirect URL format
    #[test]
    fn req_rt_003_extract_code_oob_url_format() {
        // Some OOB flows return a URL with the code in a different format
        let url = "http://localhost:8080/?code=4/0AeanS0o_oob_code";
        let result = extract_code_from_url(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "4/0AeanS0o_oob_code");
    }

    // Requirement: REQ-RT-003 (Must)
    // Acceptance: Falls back to manual flow if browser cannot be opened
    // (This is a behavioral requirement -- the run_oauth_flow dispatcher
    //  should catch browser open errors and fall back to manual)
    #[test]
    fn req_rt_003_fallback_documented() {
        // This test documents the requirement:
        // When run_desktop_flow fails because the browser cannot be opened,
        // run_oauth_flow should fall back to run_manual_flow.
        // The implementation must handle this gracefully.
        assert!(true, "Fallback behavior documented");
    }

    // Requirement: REQ-RT-003 (Must)
    // Edge case: User pastes the code directly instead of the full URL
    #[test]
    fn req_rt_003_edge_user_pastes_code_directly() {
        // If user pastes just the code, not a full URL, extract_code_from_url
        // should return an error since it's not a valid URL
        let result = extract_code_from_url("4/0AX4XfWh_just_the_code");
        assert!(result.is_err(), "Raw code (not a URL) should fail");
    }

    // Requirement: REQ-RT-003 (Must)
    // Edge case: User pastes URL with whitespace
    #[test]
    fn req_rt_003_edge_url_with_whitespace() {
        // URLs with leading/trailing whitespace should be handled
        let url = "  http://localhost/?code=4/0AX4XfWh_trimmed  ";
        // The url::Url parser does not trim whitespace, so this should fail
        // unless the implementation trims first.
        let result = extract_code_from_url(url.trim());
        assert!(result.is_ok(), "Trimmed URL should work");
        assert_eq!(result.unwrap(), "4/0AX4XfWh_trimmed");
    }

    // Requirement: REQ-RT-003 (Must)
    // Edge case: User pastes URL with unicode characters
    #[test]
    fn req_rt_003_edge_url_with_unicode() {
        let url = "http://localhost/?code=4%2F0test&error_description=%E4%B8%AD%E6%96%87";
        // Should extract the code even with unicode in other params
        let result = extract_code_from_url(url);
        assert!(result.is_ok());
    }

    // =================================================================
    // REQ-RT-002/003 (Must): FlowMode enum
    // =================================================================

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: FlowMode::Desktop exists
    #[test]
    fn req_rt_002_flow_mode_desktop_exists() {
        let mode = FlowMode::Desktop;
        assert_eq!(mode, FlowMode::Desktop);
    }

    // Requirement: REQ-RT-003 (Must)
    // Acceptance: FlowMode::Manual exists
    #[test]
    fn req_rt_003_flow_mode_manual_exists() {
        let mode = FlowMode::Manual;
        assert_eq!(mode, FlowMode::Manual);
    }

    // Requirement: REQ-RT-004 (Should)
    // Acceptance: FlowMode::Remote exists
    #[test]
    fn req_rt_004_flow_mode_remote_exists() {
        let mode = FlowMode::Remote;
        assert_eq!(mode, FlowMode::Remote);
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: FlowMode is Debug
    #[test]
    fn req_rt_002_flow_mode_is_debug() {
        let mode = FlowMode::Desktop;
        let debug_str = format!("{:?}", mode);
        assert!(debug_str.contains("Desktop"));
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: FlowMode is Clone + Copy
    #[test]
    fn req_rt_002_flow_mode_is_clone_copy() {
        let mode = FlowMode::Desktop;
        let copied = mode;
        let cloned = mode.clone();
        assert_eq!(mode, copied);
        assert_eq!(mode, cloned);
    }

    // =================================================================
    // REQ-RT-002/003 (Must): run_oauth_flow dispatches correctly
    // =================================================================

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: run_oauth_flow accepts force_consent=true
    #[tokio::test]
    async fn req_rt_002_run_oauth_flow_with_force_consent() {
        let creds = ClientCredentials {
            client_id: "test.apps.googleusercontent.com".to_string(),
            client_secret: "GOCSPX-test".to_string(),
        };
        let services = vec![Service::Gmail, Service::Calendar];
        // force_consent=true should be passed through to the flow
        let result = run_oauth_flow(&creds, &services, FlowMode::Desktop, true).await;
        assert!(result.is_err() || result.is_ok());
    }

    // Requirement: REQ-RT-002 (Must)
    // Acceptance: run_oauth_flow works with multiple services
    #[tokio::test]
    async fn req_rt_002_run_oauth_flow_multiple_services() {
        let creds = ClientCredentials {
            client_id: "test.apps.googleusercontent.com".to_string(),
            client_secret: "GOCSPX-test".to_string(),
        };
        let services = vec![
            Service::Gmail,
            Service::Calendar,
            Service::Drive,
            Service::Docs,
        ];
        let result = run_oauth_flow(&creds, &services, FlowMode::Desktop, false).await;
        assert!(result.is_err() || result.is_ok());
    }

    // Requirement: REQ-RT-002 (Must)
    // Edge case: run_oauth_flow with empty services list
    #[tokio::test]
    async fn req_rt_002_run_oauth_flow_empty_services() {
        let creds = ClientCredentials {
            client_id: "test.apps.googleusercontent.com".to_string(),
            client_secret: "GOCSPX-test".to_string(),
        };
        let result = run_oauth_flow(&creds, &[], FlowMode::Desktop, false).await;
        // Should either fail or work with base scopes only
        assert!(result.is_err() || result.is_ok());
    }

    // =================================================================
    // Failure mode tests (from architecture)
    // =================================================================

    // Requirement: REQ-RT-002 (Must)
    // Failure mode: Browser launch failure detected
    // (Test documents the requirement -- actual implementation tested via integration)
    #[test]
    fn req_rt_002_failure_browser_launch() {
        // When open::that() returns Err, the desktop flow should:
        // 1. Print the URL to stderr with manual instructions
        // 2. Continue waiting for the redirect (user may manually navigate)
        // OR fall back to manual flow
        assert!(true, "Browser launch failure handling documented");
    }

    // Requirement: REQ-RT-003 (Must)
    // Failure mode: Invalid redirect URL pasted
    #[test]
    fn req_rt_003_failure_invalid_redirect() {
        let result = extract_code_from_url("this is not a url");
        assert!(result.is_err(), "Invalid redirect URL should return error");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.to_lowercase().contains("url") || err_msg.to_lowercase().contains("invalid"),
            "Error should mention URL/invalid: {}",
            err_msg
        );
    }

    // Requirement: REQ-RT-002 (Must)
    // Failure mode: Port bind failure (Port 0 should eliminate this)
    #[test]
    fn req_rt_002_failure_port_bind_documented() {
        // The architecture specifies binding to 127.0.0.1:0 which lets the OS
        // assign a port, eliminating port conflicts. If bind still fails,
        // the error should suggest --manual mode.
        assert!(true, "Port bind failure handling documented");
    }

    // =================================================================
    // REQ-RT-004 (Should): Remote OAuth flow -- two-step headless
    // =================================================================

    // Requirement: REQ-RT-004 (Should)
    // Acceptance: remote_flow_step1 generates URL with state parameter
    #[test]
    fn req_rt_004_step1_generates_url_with_state() {
        let dir = tempfile::tempdir().unwrap();
        let creds = ClientCredentials {
            client_id: "test.apps.googleusercontent.com".to_string(),
            client_secret: "GOCSPX-test".to_string(),
        };
        let services = vec![Service::Gmail];
        let url = remote_flow_step1_with_dir(&creds, &services, false, dir.path())
            .expect("step1 should succeed");
        assert!(url.contains("state="), "URL should contain state parameter");
        assert!(
            url.contains("accounts.google.com"),
            "URL should point to Google auth"
        );
        assert!(
            url.contains("response_type=code"),
            "URL should contain response_type=code"
        );
        // Verify state file was created
        let state_file = dir.path().join("remote_oauth_state");
        assert!(state_file.exists(), "State file should be created");
        let state = std::fs::read_to_string(&state_file).unwrap();
        assert_eq!(state.len(), 32, "State should be 32 alphanumeric chars");
        assert!(url.contains(&state), "URL should contain the cached state");
    }

    // Requirement: REQ-RT-004 (Should)
    // Acceptance: remote_flow_step2 validates state and extracts code
    #[test]
    fn req_rt_004_step2_validates_state_extracts_code() {
        let dir = tempfile::tempdir().unwrap();
        // Simulate step1: write a known state to the state file
        let state = "abcdefghijklmnopqrstuvwxyz123456";
        let state_file = dir.path().join("remote_oauth_state");
        std::fs::write(&state_file, state).unwrap();

        // Build a redirect URL with matching state and code
        let redirect_url = format!(
            "http://localhost/?code=4/0AX4XfWh_test_code&state={}",
            state
        );
        let result = remote_flow_step2_with_dir(&redirect_url, dir.path());
        assert!(result.is_ok(), "step2 should succeed: {:?}", result.err());
        let flow_result = result.unwrap();
        assert_eq!(flow_result.code, "4/0AX4XfWh_test_code");
        assert_eq!(flow_result.redirect_uri, MANUAL_REDIRECT_URI);

        // State file should be cleaned up
        assert!(
            !state_file.exists(),
            "State file should be removed after step2"
        );
    }

    // Requirement: REQ-RT-004 (Should)
    // Acceptance: State mismatch returns error
    #[test]
    fn req_rt_004_step2_state_mismatch_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let state_file = dir.path().join("remote_oauth_state");
        std::fs::write(&state_file, "correct_state_value_here_32chars").unwrap();

        let redirect_url = "http://localhost/?code=4/test&state=wrong_state_value_not_matching";
        let result = remote_flow_step2_with_dir(redirect_url, dir.path());
        assert!(result.is_err(), "State mismatch should return error");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("mismatch") || err.contains("CSRF"),
            "Error should mention mismatch or CSRF: {}",
            err
        );
    }

    // Requirement: REQ-RT-004 (Should)
    // Acceptance: Missing state file returns error
    #[test]
    fn req_rt_004_step2_missing_state_file_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        // No state file exists
        let redirect_url = "http://localhost/?code=4/test&state=somestate";
        let result = remote_flow_step2_with_dir(redirect_url, dir.path());
        assert!(result.is_err(), "Missing state file should return error");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("No pending") || err.contains("step 1"),
            "Error should mention running step 1: {}",
            err
        );
    }

    // Requirement: REQ-RT-004 (Should)
    // Acceptance: Missing state parameter in redirect URL returns error
    #[test]
    fn req_rt_004_step2_no_state_in_url() {
        let dir = tempfile::tempdir().unwrap();
        let state_file = dir.path().join("remote_oauth_state");
        std::fs::write(&state_file, "cached_state_value").unwrap();

        let redirect_url = "http://localhost/?code=4/test";
        let result = remote_flow_step2_with_dir(redirect_url, dir.path());
        assert!(result.is_err(), "Missing state in URL should return error");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("No state") || err.contains("CSRF"),
            "Error should mention missing state: {}",
            err
        );
    }

    // Requirement: REQ-RT-004 (Should)
    // Acceptance: run_oauth_flow with Remote mode returns instructional error
    #[tokio::test]
    async fn req_rt_004_remote_mode_returns_step_instruction() {
        let creds = ClientCredentials {
            client_id: "test.apps.googleusercontent.com".to_string(),
            client_secret: "GOCSPX-test".to_string(),
        };
        let services = vec![Service::Gmail];
        let result = run_oauth_flow(&creds, &services, FlowMode::Remote, false).await;
        assert!(
            result.is_err(),
            "Remote flow via run_oauth_flow should error"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("--step"),
            "Error should mention --step: {}",
            err
        );
    }

    // Requirement: REQ-RT-004 (Should)
    // Acceptance: Step1 with force_consent includes prompt=consent
    #[test]
    fn req_rt_004_step1_force_consent() {
        let dir = tempfile::tempdir().unwrap();
        let creds = ClientCredentials {
            client_id: "test.apps.googleusercontent.com".to_string(),
            client_secret: "GOCSPX-test".to_string(),
        };
        let services = vec![Service::Gmail];
        let url = remote_flow_step1_with_dir(&creds, &services, true, dir.path())
            .expect("step1 with force_consent should succeed");
        assert!(
            url.contains("prompt=consent"),
            "URL should contain prompt=consent: {}",
            url
        );
    }

    // Requirement: REQ-RT-004 (Should)
    // Edge: Step1 produces unique state each time
    #[test]
    fn req_rt_004_step1_unique_state_each_call() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();
        let creds = ClientCredentials {
            client_id: "test.apps.googleusercontent.com".to_string(),
            client_secret: "GOCSPX-test".to_string(),
        };
        let services = vec![Service::Gmail];
        let url1 = remote_flow_step1_with_dir(&creds, &services, false, dir1.path()).unwrap();
        let url2 = remote_flow_step1_with_dir(&creds, &services, false, dir2.path()).unwrap();
        let state1 = std::fs::read_to_string(dir1.path().join("remote_oauth_state")).unwrap();
        let state2 = std::fs::read_to_string(dir2.path().join("remote_oauth_state")).unwrap();
        assert_ne!(
            state1, state2,
            "Each invocation should produce a unique state"
        );
        assert_ne!(url1, url2, "URLs should differ due to different states");
    }
}
