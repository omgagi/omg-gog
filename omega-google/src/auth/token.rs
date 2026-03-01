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
    let json_val = serde_json::json!({
        "client": token.client,
        "email": token.email,
        "services": token.services,
        "scopes": token.scopes,
        "created_at": token.created_at.to_rfc3339(),
        "refresh_token": token.refresh_token,
    });
    Ok(serde_json::to_string(&json_val)?)
}

/// Deserialize token data from a JSON string retrieved from the keyring.
pub fn deserialize_token(json_str: &str) -> anyhow::Result<TokenData> {
    let val: serde_json::Value = serde_json::from_str(json_str)?;

    let client = val.get("client")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    let email = val.get("email")
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

    let refresh_token = val.get("refresh_token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(TokenData {
        client,
        email,
        services,
        scopes,
        created_at,
        refresh_token,
    })
}

/// Check if a token might need refreshing.
/// Returns true if the token was created more than 55 minutes ago
/// (access tokens typically expire after 1 hour).
pub fn needs_refresh(token: &TokenData) -> bool {
    let age = chrono::Utc::now() - token.created_at;
    age > chrono::Duration::minutes(55)
}
