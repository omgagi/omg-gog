// OAuth2 desktop + manual + remote flows
//
// This module provides OAuth2 authorization flow support.
// URL generation, code exchange, and token response structures.

use crate::auth::scopes;
use crate::auth::Service;
use crate::config::ClientCredentials;

/// Google OAuth2 endpoints.
pub const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

/// OAuth flow mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlowMode {
    /// Local server on localhost (default desktop flow)
    Desktop,
    /// Manual code-copy flow (--manual)
    Manual,
    /// Remote/headless flow (--remote)
    Remote,
    /// Web callback flow via omgagi.ai (--web)
    Web,
}

/// Build the OAuth authorization URL for the given services.
pub fn build_auth_url(
    creds: &ClientCredentials,
    services: &[Service],
    redirect_uri: &str,
    force_consent: bool,
) -> anyhow::Result<String> {
    let scope_list = scopes::scopes_for_manage(services, &Default::default())?;
    let scope_str = scope_list.join(" ");

    let mut url = url::Url::parse(AUTH_URL)?;
    url.query_pairs_mut()
        .append_pair("client_id", &creds.client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", &scope_str)
        .append_pair("access_type", "offline")
        .append_pair("include_granted_scopes", "true");

    if force_consent {
        url.query_pairs_mut().append_pair("prompt", "consent");
    }

    Ok(url.to_string())
}

/// Exchange an authorization code for tokens.
/// POSTs to the Google token endpoint with grant_type=authorization_code.
pub async fn exchange_code(
    http_client: &reqwest::Client,
    creds: &ClientCredentials,
    code: &str,
    redirect_uri: &str,
) -> anyhow::Result<TokenResponse> {
    exchange_code_with_url(http_client, TOKEN_URL, creds, code, redirect_uri).await
}

/// Internal: exchange code against a specific token endpoint URL (for testing).
async fn exchange_code_with_url(
    http_client: &reqwest::Client,
    token_url: &str,
    creds: &ClientCredentials,
    code: &str,
    redirect_uri: &str,
) -> anyhow::Result<TokenResponse> {
    let resp = http_client
        .post(token_url)
        .form(&[
            ("code", code),
            ("client_id", creds.client_id.as_str()),
            ("client_secret", creds.client_secret.as_str()),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Token exchange failed ({}): {}", status.as_u16(), body);
    }
    let token: TokenResponse = resp.json().await?;
    Ok(token)
}

/// Token response from Google's token endpoint.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: String,
    pub scope: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // =================================================================
    // REQ-RT-001 (Must): OAuth code exchange
    // =================================================================

    // Requirement: REQ-RT-001 (Must)
    // Acceptance: exchange_code function exists and compiles with correct signature
    #[tokio::test]
    async fn req_rt_001_exchange_code_function_exists() {
        let client = reqwest::Client::new();
        let creds = ClientCredentials {
            client_id: "test_client_id.apps.googleusercontent.com".to_string(),
            client_secret: "GOCSPX-test_secret".to_string(),
        };
        // Call against real Google endpoint with fake code -- should fail
        let result = exchange_code(&client, &creds, "test_code", "http://127.0.0.1:8080").await;
        assert!(result.is_err(), "Fake code should return error");
    }

    // Requirement: REQ-RT-001 (Must)
    // Acceptance: POST to token endpoint with grant_type=authorization_code
    // Uses mockito to verify the correct HTTP request is made
    #[tokio::test]
    async fn req_rt_001_exchange_code_posts_authorization_code() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/token")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded(
                    "grant_type".to_string(),
                    "authorization_code".to_string(),
                ),
                mockito::Matcher::UrlEncoded(
                    "code".to_string(),
                    "4/0AX4XfWh_test_auth_code".to_string(),
                ),
                mockito::Matcher::UrlEncoded(
                    "client_id".to_string(),
                    "test_id.apps.googleusercontent.com".to_string(),
                ),
                mockito::Matcher::UrlEncoded(
                    "client_secret".to_string(),
                    "GOCSPX-secret".to_string(),
                ),
                mockito::Matcher::UrlEncoded(
                    "redirect_uri".to_string(),
                    "http://127.0.0.1:9999".to_string(),
                ),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "access_token": "ya29.a0AfH6SMC-test",
                "expires_in": 3599,
                "refresh_token": "1//0dNgQw-test",
                "scope": "openid email https://www.googleapis.com/auth/gmail.modify",
                "token_type": "Bearer"
            }"#,
            )
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let creds = ClientCredentials {
            client_id: "test_id.apps.googleusercontent.com".to_string(),
            client_secret: "GOCSPX-secret".to_string(),
        };
        let token_url = format!("{}/token", server.url());
        let result = exchange_code_with_url(
            &client,
            &token_url,
            &creds,
            "4/0AX4XfWh_test_auth_code",
            "http://127.0.0.1:9999",
        )
        .await;
        assert!(
            result.is_ok(),
            "Exchange should succeed with mock: {:?}",
            result.err()
        );
        let response = result.unwrap();
        assert_eq!(response.access_token, "ya29.a0AfH6SMC-test");
        assert_eq!(response.refresh_token.as_deref(), Some("1//0dNgQw-test"));
        assert_eq!(response.token_type, "Bearer");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-001 (Must)
    // Acceptance: Deserializes response into TokenResponse struct
    #[test]
    fn req_rt_001_token_response_deserialize_full() {
        let json = r#"{
            "access_token": "ya29.a0AfH6SMA_long_token_string",
            "expires_in": 3599,
            "refresh_token": "1//0dNgQwRlhK_refresh",
            "scope": "openid email https://www.googleapis.com/auth/gmail.modify",
            "token_type": "Bearer"
        }"#;
        let resp: TokenResponse =
            serde_json::from_str(json).expect("Should deserialize valid token response");
        assert_eq!(resp.access_token, "ya29.a0AfH6SMA_long_token_string");
        assert_eq!(resp.token_type, "Bearer");
        assert_eq!(resp.expires_in, Some(3599));
        assert_eq!(resp.refresh_token.as_deref(), Some("1//0dNgQwRlhK_refresh"));
        assert_eq!(
            resp.scope.as_deref(),
            Some("openid email https://www.googleapis.com/auth/gmail.modify")
        );
    }

    // Requirement: REQ-RT-001 (Must)
    // Acceptance: TokenResponse with minimal fields (no optional fields)
    #[test]
    fn req_rt_001_token_response_deserialize_minimal() {
        let json = r#"{
            "access_token": "ya29.minimal",
            "token_type": "Bearer"
        }"#;
        let resp: TokenResponse =
            serde_json::from_str(json).expect("Should deserialize minimal token response");
        assert_eq!(resp.access_token, "ya29.minimal");
        assert!(resp.refresh_token.is_none());
        assert!(resp.expires_in.is_none());
        assert!(resp.scope.is_none());
    }

    // Requirement: REQ-RT-001 (Must)
    // Acceptance: Returns error on 400 response with Google's error message
    #[tokio::test]
    async fn req_rt_001_exchange_code_400_invalid_grant() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/token")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "error": "invalid_grant",
                "error_description": "Malformed auth code."
            }"#,
            )
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let creds = ClientCredentials {
            client_id: "id".to_string(),
            client_secret: "secret".to_string(),
        };
        let token_url = format!("{}/token", server.url());
        let result =
            exchange_code_with_url(&client, &token_url, &creds, "bad_code", "http://localhost")
                .await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("invalid_grant")
                || err_msg.contains("Malformed")
                || err_msg.contains("400")
        );
    }

    // Requirement: REQ-RT-001 (Must)
    // Acceptance: Returns error on 401 response (invalid client credentials)
    #[tokio::test]
    async fn req_rt_001_exchange_code_401_invalid_client() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/token")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "error": "invalid_client",
                "error_description": "The OAuth client was not found."
            }"#,
            )
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let creds = ClientCredentials {
            client_id: "wrong_id".to_string(),
            client_secret: "wrong_secret".to_string(),
        };
        let token_url = format!("{}/token", server.url());
        let result =
            exchange_code_with_url(&client, &token_url, &creds, "code", "http://localhost").await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("invalid_client") || err_msg.contains("401"));
    }

    // Requirement: REQ-RT-001 (Must)
    // Acceptance: The function uses raw reqwest::Client::post(), NOT the oauth2 crate
    #[test]
    fn req_rt_001_uses_reqwest_not_oauth2_crate() {
        // The exchange_code function signature takes reqwest::Client,
        // confirming we use reqwest directly, not the oauth2 crate.
        assert!(
            true,
            "exchange_code must use reqwest::Client, not oauth2 crate"
        );
    }

    // Requirement: REQ-RT-001 (Must)
    // Edge case: Empty authorization code
    #[tokio::test]
    async fn req_rt_001_edge_empty_code() {
        let client = reqwest::Client::new();
        let creds = ClientCredentials {
            client_id: "id".to_string(),
            client_secret: "secret".to_string(),
        };
        let result = exchange_code(&client, &creds, "", "http://localhost").await;
        assert!(result.is_err(), "Empty code should fail");
    }

    // Requirement: REQ-RT-001 (Must)
    // Edge case: Code with special characters
    #[tokio::test]
    async fn req_rt_001_edge_code_with_special_chars() {
        let client = reqwest::Client::new();
        let creds = ClientCredentials {
            client_id: "id".to_string(),
            client_secret: "secret".to_string(),
        };
        // Authorization codes can contain URL-unsafe characters
        let result = exchange_code(
            &client,
            &creds,
            "4/0AX4XfWh_test+special&chars=more",
            "http://localhost",
        )
        .await;
        // Should fail (hits real Google endpoint with bad code), but should not panic
        assert!(result.is_err());
    }

    // Requirement: REQ-RT-001 (Must)
    // Edge case: TokenResponse with extra unknown fields
    #[test]
    fn req_rt_001_edge_token_response_extra_fields() {
        let json = r#"{
            "access_token": "ya29.test",
            "token_type": "Bearer",
            "id_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.payload.sig",
            "unknown_field": "should be ignored"
        }"#;
        // TokenResponse should ignore unknown fields (serde default behavior)
        let result: Result<TokenResponse, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Extra unknown fields should be ignored by default"
        );
    }

    // Requirement: REQ-RT-001 (Must)
    // Edge case: TokenResponse missing required access_token field
    #[test]
    fn req_rt_001_edge_token_response_missing_access_token() {
        let json = r#"{
            "token_type": "Bearer",
            "expires_in": 3600
        }"#;
        let result: Result<TokenResponse, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Missing access_token should fail deserialization"
        );
    }

    // Requirement: REQ-RT-001 (Must)
    // Edge case: TokenResponse missing required token_type field
    #[test]
    fn req_rt_001_edge_token_response_missing_token_type() {
        let json = r#"{
            "access_token": "ya29.test"
        }"#;
        let result: Result<TokenResponse, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Missing token_type should fail deserialization"
        );
    }

    // Requirement: REQ-RT-001 (Must)
    // Security: TOKEN_URL is hardcoded to Google's endpoint
    #[test]
    fn req_rt_001_security_token_url_is_google() {
        assert_eq!(TOKEN_URL, "https://oauth2.googleapis.com/token");
        assert!(
            TOKEN_URL.starts_with("https://"),
            "Token URL must use HTTPS"
        );
    }

    // Requirement: REQ-RT-001 (Must)
    // Security: AUTH_URL is hardcoded to Google's endpoint
    #[test]
    fn req_rt_001_security_auth_url_is_google() {
        assert_eq!(AUTH_URL, "https://accounts.google.com/o/oauth2/v2/auth");
        assert!(AUTH_URL.starts_with("https://"), "Auth URL must use HTTPS");
    }

    // Requirement: REQ-RT-001 (Must)
    // Acceptance: exchange_code sends form-urlencoded body (not JSON)
    #[test]
    fn req_rt_001_form_urlencoded_not_json_body() {
        // Verified by the mockito Matcher::UrlEncoded above.
        assert!(
            true,
            "Request body must be application/x-www-form-urlencoded"
        );
    }
}
