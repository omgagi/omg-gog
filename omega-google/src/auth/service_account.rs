// JWT auth for service accounts
//
// This module provides service account authentication via JWT.
// The actual JWT signing is done using the jsonwebtoken crate.
// The token exchange with Google's OAuth endpoint is stubbed.

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

/// Exchange a JWT assertion for an access token.
/// This function is a placeholder; the actual HTTP exchange is not yet implemented.
pub async fn exchange_jwt(
    _assertion: &str,
) -> anyhow::Result<String> {
    anyhow::bail!("JWT token exchange not yet implemented")
}
