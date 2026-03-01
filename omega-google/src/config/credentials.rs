// OAuth credential file management (installed/web format parsing)

use super::ClientCredentials;

/// Parse Google OAuth client credentials from downloaded JSON.
/// Supports both "installed" and "web" wrapper formats.
pub fn parse_credentials(raw: &serde_json::Value) -> anyhow::Result<ClientCredentials> {
    // Try "installed" wrapper first
    if let Some(inner) = raw.get("installed") {
        return extract_credentials(inner);
    }
    // Try "web" wrapper
    if let Some(inner) = raw.get("web") {
        return extract_credentials(inner);
    }
    // Try flat format (client_id/client_secret at top level)
    extract_credentials(raw)
}

fn extract_credentials(value: &serde_json::Value) -> anyhow::Result<ClientCredentials> {
    let client_id = value
        .get("client_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing client_id in credentials"))?
        .to_string();
    let client_secret = value
        .get("client_secret")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing client_secret in credentials"))?
        .to_string();
    Ok(ClientCredentials {
        client_id,
        client_secret,
    })
}
