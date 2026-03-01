// OAuth2 desktop + manual + remote flows
//
// This module provides OAuth2 authorization flow support.
// The actual browser/server parts are not yet implemented
// (requires full OAuth infrastructure), but the URL generation
// and code exchange structures are defined.

use crate::config::ClientCredentials;
use crate::auth::scopes;
use crate::auth::Service;

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
/// This function is a placeholder; the actual HTTP exchange is not yet implemented.
pub async fn exchange_code(
    _creds: &ClientCredentials,
    _code: &str,
    _redirect_uri: &str,
) -> anyhow::Result<TokenResponse> {
    anyhow::bail!("OAuth code exchange not yet implemented")
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
