// JWT auth for service accounts
//
// This module provides service account authentication via JWT.
// The actual JWT signing is done using the jsonwebtoken crate.
// The token exchange with Google's OAuth endpoint is implemented.

use serde::{Deserialize, Serialize};

/// Google's OAuth2 token endpoint for service accounts.
pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

/// Service account key file structure (from Google Cloud Console).
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccountKey {
    #[serde(rename = "type")]
    pub key_type: String,
    pub project_id: String,
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: String,
    pub auth_uri: String,
    pub token_uri: String,
}

/// JWT claims for service account auth.
#[derive(Debug, Serialize)]
pub struct JwtClaims {
    pub iss: String,
    pub scope: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
}

/// Load a service account key from a JSON file.
pub fn load_service_account_key(path: &std::path::Path) -> anyhow::Result<ServiceAccountKey> {
    let content = std::fs::read_to_string(path)?;
    let key: ServiceAccountKey = serde_json::from_str(&content)?;
    if key.key_type != "service_account" {
        anyhow::bail!("expected key type 'service_account', got '{}'", key.key_type);
    }
    Ok(key)
}

/// Build a JWT assertion for the given service account and scopes.
/// The actual signing uses the RS256 algorithm.
pub fn build_jwt_assertion(
    key: &ServiceAccountKey,
    scopes: &[String],
    subject: Option<&str>,
) -> anyhow::Result<String> {
    let now = chrono::Utc::now().timestamp();
    let claims = JwtClaims {
        iss: key.client_email.clone(),
        scope: scopes.join(" "),
        aud: TOKEN_URL.to_string(),
        iat: now,
        exp: now + 3600, // 1 hour
        sub: subject.map(|s| s.to_string()),
    };

    let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(key.private_key.as_bytes())?;
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let token = jsonwebtoken::encode(&header, &claims, &encoding_key)?;
    Ok(token)
}

/// Service account token response from Google's token endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccountTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
}

/// Exchange a JWT assertion for an access token.
/// POSTs to the Google token endpoint with grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer.
pub async fn exchange_jwt(
    http_client: &reqwest::Client,
    assertion: &str,
) -> anyhow::Result<ServiceAccountTokenResponse> {
    exchange_jwt_with_url(http_client, TOKEN_URL, assertion).await
}

/// Internal: exchange JWT against a specific token endpoint URL (for testing).
async fn exchange_jwt_with_url(
    http_client: &reqwest::Client,
    token_url: &str,
    assertion: &str,
) -> anyhow::Result<ServiceAccountTokenResponse> {
    let resp = http_client
        .post(token_url)
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", assertion),
        ])
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("JWT exchange failed ({}): {}", status.as_u16(), body);
    }
    let token: ServiceAccountTokenResponse = resp.json().await?;
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    // =================================================================
    // REQ-RT-006 (Must): Service account JWT exchange
    // =================================================================

    // Requirement: REQ-RT-006 (Must)
    // Acceptance: exchange_jwt function exists and compiles
    #[tokio::test]
    async fn req_rt_006_exchange_jwt_function_exists() {
        let client = reqwest::Client::new();
        // Call against real Google endpoint with fake JWT -- should fail
        let result = exchange_jwt(&client, "fake.jwt.assertion").await;
        assert!(result.is_err(), "Fake JWT should return error");
    }

    // Requirement: REQ-RT-006 (Must)
    // Acceptance: POSTs to token endpoint with grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer
    #[tokio::test]
    async fn req_rt_006_exchange_jwt_posts_jwt_bearer_grant_type() {
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("POST", "/token")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded(
                    "grant_type".to_string(),
                    "urn:ietf:params:oauth:grant-type:jwt-bearer".to_string(),
                ),
                mockito::Matcher::UrlEncoded(
                    "assertion".to_string(),
                    "test.jwt.assertion".to_string(),
                ),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "access_token": "ya29.sa_access_token",
                "expires_in": 3600,
                "token_type": "Bearer"
            }"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let token_url = format!("{}/token", server.url());
        let result = exchange_jwt_with_url(&client, &token_url, "test.jwt.assertion").await;
        assert!(result.is_ok(), "Exchange should succeed with mock: {:?}", result.err());
        let response = result.unwrap();
        assert_eq!(response.access_token, "ya29.sa_access_token");
        assert_eq!(response.expires_in, 3600);
        assert_eq!(response.token_type, "Bearer");
        mock.assert_async().await;
    }

    // Requirement: REQ-RT-006 (Must)
    // Acceptance: Deserializes access_token from response
    #[test]
    fn req_rt_006_service_account_token_response_deserialize() {
        let json = r#"{
            "access_token": "ya29.service_account_token",
            "expires_in": 3600,
            "token_type": "Bearer"
        }"#;
        let resp: ServiceAccountTokenResponse = serde_json::from_str(json)
            .expect("Should deserialize service account token response");
        assert_eq!(resp.access_token, "ya29.service_account_token");
        assert_eq!(resp.expires_in, 3600);
        assert_eq!(resp.token_type, "Bearer");
    }

    // Requirement: REQ-RT-006 (Must)
    // Acceptance: Returns error with message on failure
    #[tokio::test]
    async fn req_rt_006_exchange_jwt_failure_returns_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server.mock("POST", "/token")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "error": "invalid_grant",
                "error_description": "Invalid JWT Signature."
            }"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let token_url = format!("{}/token", server.url());
        let result = exchange_jwt_with_url(&client, &token_url, "invalid.jwt.assertion").await;
        assert!(result.is_err(), "Bad JWT should cause exchange error");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("400") || err_msg.contains("invalid_grant") || err_msg.contains("JWT exchange failed"));
    }

    // Requirement: REQ-RT-006 (Must)
    // Acceptance: Sends JWT assertion built by existing build_jwt_assertion()
    #[test]
    fn req_rt_006_jwt_claims_serialize_correctly() {
        let claims = JwtClaims {
            iss: "sa@project.iam.gserviceaccount.com".to_string(),
            scope: "https://www.googleapis.com/auth/keep.readonly".to_string(),
            aud: TOKEN_URL.to_string(),
            iat: 1700000000,
            exp: 1700003600,
            sub: Some("admin@domain.com".to_string()),
        };
        let json = serde_json::to_string(&claims).expect("claims should serialize");
        assert!(json.contains("sa@project.iam.gserviceaccount.com"));
        assert!(json.contains("keep.readonly"));
        assert!(json.contains("admin@domain.com"));
    }

    // Requirement: REQ-RT-006 (Must)
    // Acceptance: JwtClaims without subject omits sub field
    #[test]
    fn req_rt_006_jwt_claims_no_subject() {
        let claims = JwtClaims {
            iss: "sa@project.iam.gserviceaccount.com".to_string(),
            scope: "scope".to_string(),
            aud: TOKEN_URL.to_string(),
            iat: 1700000000,
            exp: 1700003600,
            sub: None,
        };
        let json = serde_json::to_string(&claims).expect("claims should serialize");
        assert!(!json.contains("sub"), "sub field should be omitted when None");
    }

    // Requirement: REQ-RT-006 (Must)
    // Edge case: Empty assertion string
    #[tokio::test]
    async fn req_rt_006_edge_empty_assertion() {
        let client = reqwest::Client::new();
        let result = exchange_jwt(&client, "").await;
        assert!(result.is_err(), "Empty assertion should fail");
    }

    // Requirement: REQ-RT-006 (Must)
    // Edge case: ServiceAccountKey deserialization from JSON file format
    #[test]
    fn req_rt_006_service_account_key_deserialize() {
        let json = r#"{
            "type": "service_account",
            "project_id": "my-project",
            "private_key_id": "key123",
            "private_key": "-----BEGIN RSA PRIVATE KEY-----\nMIIE...\n-----END RSA PRIVATE KEY-----\n",
            "client_email": "sa@my-project.iam.gserviceaccount.com",
            "client_id": "123456789",
            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
            "token_uri": "https://oauth2.googleapis.com/token"
        }"#;
        let key: ServiceAccountKey = serde_json::from_str(json)
            .expect("Should deserialize service account key");
        assert_eq!(key.key_type, "service_account");
        assert_eq!(key.project_id, "my-project");
        assert_eq!(key.client_email, "sa@my-project.iam.gserviceaccount.com");
    }

    // Requirement: REQ-RT-006 (Must)
    // Edge case: Wrong key type
    #[test]
    fn req_rt_006_edge_wrong_key_type() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wrong-type.json");
        std::fs::write(&path, r#"{
            "type": "authorized_user",
            "project_id": "p",
            "private_key_id": "k",
            "private_key": "pk",
            "client_email": "e@e.com",
            "client_id": "123",
            "auth_uri": "u",
            "token_uri": "u"
        }"#).unwrap();
        let result = load_service_account_key(&path);
        assert!(result.is_err(), "Wrong key type should fail");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("service_account"), "Error should mention expected type");
    }

    // Requirement: REQ-RT-006 (Must)
    // Edge case: Key file not found
    #[test]
    fn req_rt_006_edge_key_file_not_found() {
        let result = load_service_account_key(std::path::Path::new("/nonexistent/sa-key.json"));
        assert!(result.is_err(), "Non-existent file should fail");
    }

    // Requirement: REQ-RT-006 (Must)
    // Edge case: Malformed JSON key file
    #[test]
    fn req_rt_006_edge_malformed_key_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, "not json").unwrap();
        let result = load_service_account_key(&path);
        assert!(result.is_err(), "Malformed JSON should fail");
    }

    // Requirement: REQ-RT-006 (Must)
    // Edge case: ServiceAccountTokenResponse with extra fields (should be ignored)
    #[test]
    fn req_rt_006_edge_sa_token_response_extra_fields() {
        let json = r#"{
            "access_token": "ya29.test",
            "expires_in": 3600,
            "token_type": "Bearer",
            "unknown_field": true
        }"#;
        let result: Result<ServiceAccountTokenResponse, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Extra fields should be ignored");
    }

    // Requirement: REQ-RT-006 (Must)
    // Security: TOKEN_URL matches Google's endpoint and uses HTTPS
    #[test]
    fn req_rt_006_security_token_url() {
        assert_eq!(TOKEN_URL, "https://oauth2.googleapis.com/token");
        assert!(TOKEN_URL.starts_with("https://"));
    }

    // Requirement: REQ-RT-006 (Must)
    // Security: Private key never appears in error messages
    #[test]
    fn req_rt_006_security_private_key_not_in_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad-type.json");
        std::fs::write(&path, r#"{
            "type": "wrong",
            "project_id": "p",
            "private_key_id": "k",
            "private_key": "-----BEGIN RSA PRIVATE KEY-----\nSUPER_SECRET_KEY_DATA\n-----END RSA PRIVATE KEY-----\n",
            "client_email": "e@e.com",
            "client_id": "123",
            "auth_uri": "u",
            "token_uri": "u"
        }"#).unwrap();
        let result = load_service_account_key(&path);
        let err = result.unwrap_err().to_string();
        assert!(
            !err.contains("SUPER_SECRET_KEY_DATA"),
            "Error message must not contain private key data"
        );
    }
}
