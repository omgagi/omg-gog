// Token storage, refresh, resolution
//
// Token key format and parsing live in auth/mod.rs:
//   - token_key()
//   - legacy_token_key()
//   - parse_token_key()
//
// This module provides serialization/deserialization for token data
// and refresh-check logic.

use crate::auth::TokenData;

/// Serialize token data to a JSON string for storage.
/// Note: this includes the refresh_token because it's needed in the keyring.
/// The refresh_token must NEVER be included in user-facing JSON output.
pub fn serialize_token(token: &TokenData) -> anyhow::Result<String> {
    let mut json_val = serde_json::json!({
        "client": token.client,
        "email": token.email,
        "services": token.services,
        "scopes": token.scopes,
        "created_at": token.created_at.to_rfc3339(),
        "refresh_token": token.refresh_token,
    });

    // REQ-RT-007: include access_token and expires_at when present
    if let Some(ref at) = token.access_token {
        json_val["access_token"] = serde_json::Value::String(at.clone());
    }
    if let Some(ref ea) = token.expires_at {
        json_val["expires_at"] = serde_json::Value::String(ea.to_rfc3339());
    }

    Ok(serde_json::to_string(&json_val)?)
}

/// Deserialize token data from a JSON string retrieved from the keyring.
pub fn deserialize_token(json_str: &str) -> anyhow::Result<TokenData> {
    let val: serde_json::Value = serde_json::from_str(json_str)?;

    let client = val
        .get("client")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    let email = val
        .get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing 'email' in token data"))?
        .to_string();

    let services: Vec<crate::auth::Service> = match val.get("services") {
        Some(v) => serde_json::from_value(v.clone())?,
        None => vec![],
    };

    let scopes: Vec<String> = match val.get("scopes") {
        Some(v) => serde_json::from_value(v.clone())?,
        None => vec![],
    };

    let created_at = match val.get("created_at").and_then(|v| v.as_str()) {
        Some(s) => chrono::DateTime::parse_from_rfc3339(s)?.with_timezone(&chrono::Utc),
        None => chrono::Utc::now(),
    };

    let refresh_token = val
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // REQ-RT-007: read new optional fields (backward compatible)
    let access_token = val
        .get("access_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let expires_at = val
        .get("expires_at")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    Ok(TokenData {
        client,
        email,
        services,
        scopes,
        created_at,
        refresh_token,
        access_token,
        expires_at,
    })
}

/// Check if a token might need refreshing.
/// If `expires_at` is set, returns true when fewer than 5 minutes remain.
/// Otherwise falls back to the `created_at` heuristic (> 55 minutes old).
pub fn needs_refresh(token: &TokenData) -> bool {
    if let Some(expires_at) = token.expires_at {
        let now = chrono::Utc::now();
        let buffer = chrono::Duration::minutes(5);
        // Need refresh if expires_at minus buffer is at or before now
        expires_at - buffer <= now
    } else {
        // Fallback: created_at heuristic
        let age = chrono::Utc::now() - token.created_at;
        age > chrono::Duration::minutes(55)
    }
}

/// Refresh an access token using a refresh token.
/// POSTs to the Google token endpoint with grant_type=refresh_token.
/// Returns the new TokenResponse on success.
pub async fn refresh_access_token(
    http_client: &reqwest::Client,
    creds: &crate::config::ClientCredentials,
    refresh_token: &str,
) -> anyhow::Result<crate::auth::oauth::TokenResponse> {
    refresh_access_token_with_url(
        http_client,
        crate::auth::oauth::TOKEN_URL,
        &creds.client_id,
        &creds.client_secret,
        refresh_token,
    )
    .await
}

/// Internal: refresh access token against a specific token endpoint URL (for testing).
async fn refresh_access_token_with_url(
    http_client: &reqwest::Client,
    token_url: &str,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> anyhow::Result<crate::auth::oauth::TokenResponse> {
    let resp = http_client
        .post(token_url)
        .form(&[
            ("refresh_token", refresh_token),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        if body.contains("invalid_grant") {
            anyhow::bail!(
                "Refresh token is invalid or revoked. Re-authenticate with: omega-google auth add"
            );
        }
        anyhow::bail!("Token refresh failed ({}): {}", status.as_u16(), body);
    }
    let token: crate::auth::oauth::TokenResponse = resp.json().await?;
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{Service, TokenData};
    use chrono::Timelike;

    /// Helper to create a basic TokenData for testing.
    fn make_token(
        client: &str,
        email: &str,
        created_at: chrono::DateTime<chrono::Utc>,
        access_token: Option<&str>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> TokenData {
        TokenData {
            client: client.to_string(),
            email: email.to_string(),
            services: vec![Service::Gmail],
            scopes: vec!["https://www.googleapis.com/auth/gmail.modify".to_string()],
            created_at,
            refresh_token: "1//refresh_test".to_string(),
            access_token: access_token.map(|s| s.to_string()),
            expires_at,
        }
    }

    // =================================================================
    // REQ-RT-007 (Must): serialize_token includes new fields
    // =================================================================

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: serialize_token handles access_token field
    #[test]
    fn req_rt_007_serialize_includes_access_token() {
        let now = chrono::Utc::now();
        let token = make_token(
            "default",
            "user@example.com",
            now,
            Some("ya29.test_access"),
            None,
        );
        let json_str = serialize_token(&token).expect("serialize should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(
            parsed.get("access_token").and_then(|v| v.as_str()),
            Some("ya29.test_access"),
            "Serialized JSON must include access_token when Some"
        );
    }

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: serialize_token handles expires_at field
    #[test]
    fn req_rt_007_serialize_includes_expires_at() {
        let now = chrono::Utc::now();
        let expires = now + chrono::Duration::hours(1);
        let token = make_token(
            "default",
            "user@example.com",
            now,
            Some("at"),
            Some(expires),
        );
        let json_str = serialize_token(&token).expect("serialize should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(
            parsed.get("expires_at").is_some(),
            "Serialized JSON must include expires_at when Some"
        );
        // Verify it is an RFC3339 string
        let expires_str = parsed.get("expires_at").unwrap().as_str().unwrap();
        let parsed_dt = chrono::DateTime::parse_from_rfc3339(expires_str);
        assert!(parsed_dt.is_ok(), "expires_at must be valid RFC3339");
    }

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: serialize_token omits access_token when None
    #[test]
    fn req_rt_007_serialize_omits_access_token_when_none() {
        let now = chrono::Utc::now();
        let token = make_token("default", "user@example.com", now, None, None);
        let json_str = serialize_token(&token).expect("serialize should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        // When access_token is None, it should either be absent or null
        let at = parsed.get("access_token");
        assert!(
            at.is_none() || at.unwrap().is_null(),
            "access_token should be absent or null when None"
        );
    }

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: serialize_token omits expires_at when None
    #[test]
    fn req_rt_007_serialize_omits_expires_at_when_none() {
        let now = chrono::Utc::now();
        let token = make_token("default", "user@example.com", now, None, None);
        let json_str = serialize_token(&token).expect("serialize should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let ea = parsed.get("expires_at");
        assert!(
            ea.is_none() || ea.unwrap().is_null(),
            "expires_at should be absent or null when None"
        );
    }

    // =================================================================
    // REQ-RT-007 (Must): deserialize_token handles new fields
    // =================================================================

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: deserialize_token reads access_token from JSON
    #[test]
    fn req_rt_007_deserialize_reads_access_token() {
        let json_str = r#"{
            "client": "default",
            "email": "user@example.com",
            "services": ["gmail"],
            "scopes": [],
            "created_at": "2025-01-01T00:00:00Z",
            "refresh_token": "rt",
            "access_token": "ya29.deserialized"
        }"#;
        let token = deserialize_token(json_str).expect("deserialize should succeed");
        assert_eq!(
            token.access_token.as_deref(),
            Some("ya29.deserialized"),
            "access_token must be read from JSON"
        );
    }

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: deserialize_token reads expires_at from JSON
    #[test]
    fn req_rt_007_deserialize_reads_expires_at() {
        let json_str = r#"{
            "client": "default",
            "email": "user@example.com",
            "services": [],
            "scopes": [],
            "created_at": "2025-01-01T00:00:00Z",
            "refresh_token": "rt",
            "access_token": "at",
            "expires_at": "2025-01-01T01:00:00Z"
        }"#;
        let token = deserialize_token(json_str).expect("deserialize should succeed");
        assert!(token.expires_at.is_some(), "expires_at must be parsed");
        let ea = token.expires_at.unwrap();
        assert_eq!(ea.hour(), 1);
    }

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: Backward compatible -- old data without access_token/expires_at deserializes
    #[test]
    fn req_rt_007_deserialize_backward_compatible_no_new_fields() {
        let json_str = r#"{
            "client": "default",
            "email": "user@example.com",
            "services": ["gmail"],
            "scopes": ["https://www.googleapis.com/auth/gmail.modify"],
            "created_at": "2024-06-15T12:00:00Z",
            "refresh_token": "1//old_refresh_token"
        }"#;
        let token = deserialize_token(json_str).expect("old format must deserialize without error");
        assert_eq!(token.email, "user@example.com");
        assert_eq!(token.refresh_token, "1//old_refresh_token");
        assert!(
            token.access_token.is_none(),
            "Missing access_token should deserialize to None"
        );
        assert!(
            token.expires_at.is_none(),
            "Missing expires_at should deserialize to None"
        );
    }

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: Round-trip: serialize then deserialize preserves new fields
    #[test]
    fn req_rt_007_roundtrip_with_new_fields() {
        let now = chrono::Utc::now();
        let expires = now + chrono::Duration::hours(1);
        let original = make_token(
            "myapp",
            "test@gmail.com",
            now,
            Some("ya29.round_trip"),
            Some(expires),
        );
        let json_str = serialize_token(&original).expect("serialize");
        let restored = deserialize_token(&json_str).expect("deserialize");
        assert_eq!(restored.client, original.client);
        assert_eq!(restored.email, original.email);
        assert_eq!(restored.refresh_token, original.refresh_token);
        assert_eq!(restored.access_token, original.access_token);
        // DateTime comparison: allow 1-second tolerance due to RFC3339 truncation
        if let (Some(orig_ea), Some(rest_ea)) = (original.expires_at, restored.expires_at) {
            let diff = (orig_ea - rest_ea).num_seconds().abs();
            assert!(
                diff <= 1,
                "expires_at round-trip difference too large: {} seconds",
                diff
            );
        } else {
            panic!("Both expires_at should be Some after round-trip");
        }
    }

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: Round-trip: serialize then deserialize preserves None fields
    #[test]
    fn req_rt_007_roundtrip_without_new_fields() {
        let now = chrono::Utc::now();
        let original = make_token("default", "user@test.com", now, None, None);
        let json_str = serialize_token(&original).expect("serialize");
        let restored = deserialize_token(&json_str).expect("deserialize");
        assert!(restored.access_token.is_none());
        assert!(restored.expires_at.is_none());
    }

    // =================================================================
    // REQ-RT-007 (Must): needs_refresh uses expires_at field
    // =================================================================

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: needs_refresh checks expires_at - 5min buffer; within buffer = true
    #[test]
    fn req_rt_007_needs_refresh_expires_at_within_buffer() {
        let now = chrono::Utc::now();
        // Token expires in 3 minutes (within 5-min buffer)
        let expires = now + chrono::Duration::minutes(3);
        let token = make_token(
            "default",
            "user@example.com",
            now,
            Some("at"),
            Some(expires),
        );
        assert!(
            needs_refresh(&token),
            "Token expiring in 3 minutes (within 5-min buffer) should need refresh"
        );
    }

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: needs_refresh checks expires_at - 5min buffer; outside buffer = false
    #[test]
    fn req_rt_007_needs_refresh_expires_at_outside_buffer() {
        let now = chrono::Utc::now();
        // Token expires in 10 minutes (outside 5-min buffer)
        let expires = now + chrono::Duration::minutes(10);
        let token = make_token(
            "default",
            "user@example.com",
            now,
            Some("at"),
            Some(expires),
        );
        assert!(
            !needs_refresh(&token),
            "Token expiring in 10 minutes (outside 5-min buffer) should NOT need refresh"
        );
    }

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: needs_refresh with expires_at exactly at 5-min boundary
    #[test]
    fn req_rt_007_needs_refresh_expires_at_exactly_five_minutes() {
        let now = chrono::Utc::now();
        // Token expires exactly in 5 minutes
        let expires = now + chrono::Duration::minutes(5);
        let token = make_token(
            "default",
            "user@example.com",
            now,
            Some("at"),
            Some(expires),
        );
        // At exactly 5 minutes, it should need refresh (5min remaining <= 5min buffer)
        assert!(
            needs_refresh(&token),
            "Token expiring in exactly 5 minutes should need refresh (at the boundary)"
        );
    }

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: needs_refresh with expires_at already passed
    #[test]
    fn req_rt_007_needs_refresh_expires_at_already_expired() {
        let now = chrono::Utc::now();
        let expires = now - chrono::Duration::minutes(10);
        let token = make_token(
            "default",
            "user@example.com",
            now - chrono::Duration::hours(2),
            Some("at"),
            Some(expires),
        );
        assert!(
            needs_refresh(&token),
            "Already-expired token must need refresh"
        );
    }

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: needs_refresh with expires_at=None falls back to created_at heuristic
    #[test]
    fn req_rt_007_needs_refresh_no_expires_at_fresh_token() {
        let now = chrono::Utc::now();
        // Created 10 minutes ago, no expires_at -- should NOT need refresh (< 55 min)
        let token = make_token(
            "default",
            "user@example.com",
            now - chrono::Duration::minutes(10),
            None,
            None,
        );
        assert!(
            !needs_refresh(&token),
            "Token created 10 min ago without expires_at should not need refresh (created_at fallback)"
        );
    }

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: needs_refresh with expires_at=None, old created_at triggers refresh
    #[test]
    fn req_rt_007_needs_refresh_no_expires_at_old_token() {
        let now = chrono::Utc::now();
        // Created 60 minutes ago, no expires_at -- should need refresh (> 55 min)
        let token = make_token(
            "default",
            "user@example.com",
            now - chrono::Duration::minutes(60),
            None,
            None,
        );
        assert!(
            needs_refresh(&token),
            "Token created 60 min ago without expires_at should need refresh (created_at fallback)"
        );
    }

    // Requirement: REQ-RT-007 (Must)
    // Failure mode: Old token missing new fields treated as expired
    #[test]
    fn req_rt_007_old_token_without_new_fields_triggers_refresh() {
        let json_str = r#"{
            "email": "legacy@example.com",
            "services": [],
            "scopes": [],
            "created_at": "2020-01-01T00:00:00Z",
            "refresh_token": "1//old"
        }"#;
        let token = deserialize_token(json_str).expect("old format should deserialize");
        assert!(token.access_token.is_none());
        assert!(token.expires_at.is_none());
        assert!(
            needs_refresh(&token),
            "Old token without new fields must need refresh"
        );
    }

    // =================================================================
    // REQ-RT-007 (Must): Edge cases for serialization
    // =================================================================

    // Requirement: REQ-RT-007 (Must)
    // Edge case: Empty/null input to deserialize
    #[test]
    fn req_rt_007_edge_deserialize_empty_string() {
        let result = deserialize_token("");
        assert!(result.is_err(), "Empty string should fail to deserialize");
    }

    // Requirement: REQ-RT-007 (Must)
    // Edge case: Invalid JSON input
    #[test]
    fn req_rt_007_edge_deserialize_invalid_json() {
        let result = deserialize_token("not json at all");
        assert!(result.is_err(), "Invalid JSON should fail to deserialize");
    }

    // Requirement: REQ-RT-007 (Must)
    // Edge case: JSON missing required email field
    #[test]
    fn req_rt_007_edge_deserialize_missing_email() {
        let json_str = r#"{"client": "default", "refresh_token": "rt"}"#;
        let result = deserialize_token(json_str);
        assert!(
            result.is_err(),
            "Missing email should cause deserialization error"
        );
    }

    // Requirement: REQ-RT-007 (Must)
    // Edge case: access_token with special characters / unicode
    #[test]
    fn req_rt_007_edge_access_token_special_chars() {
        let json_str = r#"{
            "email": "user@example.com",
            "access_token": "ya29.a0AfB_byC-D_e\u00e9\u00f1\u00fc",
            "created_at": "2025-01-01T00:00:00Z",
            "refresh_token": "rt"
        }"#;
        let token = deserialize_token(json_str).expect("should handle unicode in access_token");
        assert!(token.access_token.is_some());
    }

    // Requirement: REQ-RT-007 (Must)
    // Edge case: expires_at with invalid RFC3339
    #[test]
    fn req_rt_007_edge_expires_at_invalid_format() {
        let json_str = r#"{
            "email": "user@example.com",
            "expires_at": "not-a-date",
            "created_at": "2025-01-01T00:00:00Z",
            "refresh_token": "rt"
        }"#;
        let result = deserialize_token(json_str);
        assert!(
            result.is_err() || result.as_ref().unwrap().expires_at.is_none(),
            "Invalid expires_at should either fail or default to None"
        );
    }

    // Requirement: REQ-RT-007 (Must)
    // Edge case: Extremely large expires_at value
    #[test]
    fn req_rt_007_edge_expires_at_far_future() {
        let json_str = r#"{
            "email": "user@example.com",
            "access_token": "at",
            "expires_at": "2099-12-31T23:59:59Z",
            "created_at": "2025-01-01T00:00:00Z",
            "refresh_token": "rt"
        }"#;
        let token = deserialize_token(json_str).expect("far future expires_at should parse");
        assert!(token.expires_at.is_some());
        assert!(
            !needs_refresh(&token),
            "Token with far-future expires_at should not need refresh"
        );
    }

    // =================================================================
    // REQ-RT-005 (Must): Token refresh
    // =================================================================

    // Requirement: REQ-RT-005 (Must)
    // Acceptance: refresh_access_token function exists with correct signature
    #[tokio::test]
    async fn req_rt_005_refresh_access_token_exists() {
        let client = reqwest::Client::new();
        let creds = crate::config::ClientCredentials {
            client_id: "test_id".to_string(),
            client_secret: "test_secret".to_string(),
        };
        let result = refresh_access_token(&client, &creds, "1//test_refresh_token").await;
        // Will fail trying to reach Google's real endpoint
        assert!(result.is_err(), "Fake refresh token should return error");
    }

    // Requirement: REQ-RT-005 (Must)
    // Acceptance: Token refresh POSTs with grant_type=refresh_token
    // Uses mockito to simulate the Google token endpoint
    #[tokio::test]
    async fn req_rt_005_refresh_posts_to_token_endpoint() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/token")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("grant_type".to_string(), "refresh_token".to_string()),
                mockito::Matcher::UrlEncoded("refresh_token".to_string(), "1//test_rt".to_string()),
                mockito::Matcher::UrlEncoded("client_id".to_string(), "test_client_id".to_string()),
                mockito::Matcher::UrlEncoded(
                    "client_secret".to_string(),
                    "test_secret".to_string(),
                ),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "access_token": "ya29.new_access_token",
                "expires_in": 3600,
                "token_type": "Bearer",
                "scope": "openid email"
            }"#,
            )
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let token_url = format!("{}/token", server.url());
        let result = refresh_access_token_with_url(
            &client,
            &token_url,
            "test_client_id",
            "test_secret",
            "1//test_rt",
        )
        .await;
        let response = result.expect("refresh should succeed with mock");
        assert_eq!(response.access_token, "ya29.new_access_token");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-005 (Must)
    // Acceptance: Deserialize TokenResponse from known JSON (happy path)
    #[test]
    fn req_rt_005_token_response_deserialize_happy_path() {
        let json = r#"{
            "access_token": "ya29.a0AfH6SMA...",
            "expires_in": 3599,
            "refresh_token": "1//0dNgQw...",
            "scope": "openid email https://www.googleapis.com/auth/gmail.modify",
            "token_type": "Bearer"
        }"#;
        let resp: crate::auth::oauth::TokenResponse =
            serde_json::from_str(json).expect("Should deserialize valid token response");
        assert_eq!(resp.access_token, "ya29.a0AfH6SMA...");
        assert_eq!(resp.token_type, "Bearer");
        assert_eq!(resp.expires_in, Some(3599));
        assert_eq!(resp.refresh_token.as_deref(), Some("1//0dNgQw..."));
    }

    // Requirement: REQ-RT-005 (Must)
    // Acceptance: TokenResponse without optional refresh_token (refresh response)
    #[test]
    fn req_rt_005_token_response_deserialize_no_refresh_token() {
        let json = r#"{
            "access_token": "ya29.refreshed",
            "expires_in": 3600,
            "token_type": "Bearer"
        }"#;
        let resp: crate::auth::oauth::TokenResponse = serde_json::from_str(json)
            .expect("Should deserialize token response without refresh_token");
        assert_eq!(resp.access_token, "ya29.refreshed");
        assert!(resp.refresh_token.is_none());
    }

    // Requirement: REQ-RT-005 (Must)
    // Failure mode: Refresh fails with invalid_grant
    #[tokio::test]
    async fn req_rt_005_refresh_invalid_grant_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server.mock("POST", "/token")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "invalid_grant", "error_description": "Token has been expired or revoked."}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let token_url = format!("{}/token", server.url());
        let result =
            refresh_access_token_with_url(&client, &token_url, "id", "secret", "1//revoked").await;
        assert!(result.is_err(), "Revoked token refresh should fail");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("invalid")
                || err_msg.contains("revoked")
                || err_msg.contains("Re-authenticate"),
            "Error should mention invalid/revoked: {}",
            err_msg,
        );
    }

    // Requirement: REQ-RT-005 (Must)
    // Failure mode: Token endpoint unreachable (network error)
    #[tokio::test]
    async fn req_rt_005_refresh_network_error() {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(100))
            .build()
            .unwrap();
        let creds = crate::config::ClientCredentials {
            client_id: "id".to_string(),
            client_secret: "secret".to_string(),
        };
        // Attempting to refresh against the real endpoint with bad creds should fail
        let result = refresh_access_token(&client, &creds, "1//some_token").await;
        assert!(
            result.is_err(),
            "Refresh against real endpoint with bad creds should fail"
        );
    }

    // Requirement: REQ-RT-005 (Must)
    // Edge case: Empty refresh token
    #[tokio::test]
    async fn req_rt_005_refresh_empty_refresh_token() {
        let client = reqwest::Client::new();
        let creds = crate::config::ClientCredentials {
            client_id: "id".to_string(),
            client_secret: "secret".to_string(),
        };
        let result = refresh_access_token(&client, &creds, "").await;
        assert!(result.is_err(), "Empty refresh token should fail");
    }

    // Requirement: REQ-RT-005 (Must)
    // Security: Token endpoint URL is hardcoded (not user-configurable)
    #[test]
    fn req_rt_005_security_token_url_hardcoded() {
        assert_eq!(
            crate::auth::oauth::TOKEN_URL,
            "https://oauth2.googleapis.com/token",
            "Token URL must be hardcoded to Google's endpoint"
        );
    }

    // =================================================================
    // REQ-RT-005 (Must): needs_refresh used before API calls
    // =================================================================

    // Requirement: REQ-RT-005 (Must)
    // Acceptance: needs_refresh returns true for token expiring within 5min buffer
    #[test]
    fn req_rt_005_needs_refresh_within_five_min_buffer() {
        let now = chrono::Utc::now();
        let token = make_token(
            "default",
            "u@e.com",
            now,
            Some("at"),
            Some(now + chrono::Duration::minutes(4)),
        );
        assert!(
            needs_refresh(&token),
            "4 min until expiry: should need refresh"
        );
    }

    // Requirement: REQ-RT-005 (Must)
    // Acceptance: needs_refresh returns false for token with plenty of time
    #[test]
    fn req_rt_005_needs_refresh_outside_five_min_buffer() {
        let now = chrono::Utc::now();
        let token = make_token(
            "default",
            "u@e.com",
            now,
            Some("at"),
            Some(now + chrono::Duration::minutes(30)),
        );
        assert!(
            !needs_refresh(&token),
            "30 min until expiry: should NOT need refresh"
        );
    }
}
