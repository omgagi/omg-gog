pub mod scopes;
pub mod oauth;
pub mod token;
pub mod keyring;
pub mod service_account;
pub mod account;

use serde::{Deserialize, Serialize};

/// All 15 supported Google services.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Service {
    Gmail,
    Calendar,
    Chat,
    Classroom,
    Drive,
    Docs,
    Slides,
    Contacts,
    Tasks,
    People,
    Sheets,
    Forms,
    AppScript,
    Groups,
    Keep,
}

/// Controls scope selection for OAuth consent.
#[derive(Debug, Clone, Default)]
pub struct ScopeOptions {
    pub readonly: bool,
    pub drive_scope: DriveScopeMode,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum DriveScopeMode {
    #[default]
    Full,
    Readonly,
    File,
}

/// Abstraction over credential storage.
pub trait CredentialStore: Send + Sync {
    fn get_token(&self, client: &str, email: &str) -> anyhow::Result<TokenData>;
    fn set_token(&self, client: &str, email: &str, token: &TokenData) -> anyhow::Result<()>;
    fn delete_token(&self, client: &str, email: &str) -> anyhow::Result<()>;
    fn list_tokens(&self) -> anyhow::Result<Vec<TokenData>>;
    fn keys(&self) -> anyhow::Result<Vec<String>>;
    fn get_default_account(&self, client: &str) -> anyhow::Result<Option<String>>;
    fn set_default_account(&self, client: &str, email: &str) -> anyhow::Result<()>;
}

/// Token data stored in the keyring.
#[derive(Clone)]
pub struct TokenData {
    pub client: String,
    pub email: String,
    pub services: Vec<Service>,
    pub scopes: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub refresh_token: String,
    /// Cached access token from the last OAuth exchange or refresh.
    /// Added by REQ-RT-007. None for tokens that predate this field.
    pub access_token: Option<String>,
    /// Expiration time of the access_token.
    /// Added by REQ-RT-007. None for tokens that predate this field.
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl std::fmt::Debug for TokenData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenData")
            .field("client", &self.client)
            .field("email", &self.email)
            .field("services", &self.services)
            .field("scopes", &self.scopes)
            .field("created_at", &self.created_at)
            .field("refresh_token", &"[REDACTED]")
            .field("access_token", &self.access_token.as_ref().map(|_| "[REDACTED]"))
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

/// Service information for display.
#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    pub service: Service,
    pub user: bool,
    pub scopes: Vec<String>,
    pub apis: Vec<String>,
    pub note: String,
}

/// Parse a service name string into a Service enum.
pub fn parse_service(s: &str) -> anyhow::Result<Service> {
    let trimmed = s.trim().to_lowercase();
    match trimmed.as_str() {
        "gmail" => Ok(Service::Gmail),
        "calendar" | "cal" => Ok(Service::Calendar),
        "chat" => Ok(Service::Chat),
        "classroom" => Ok(Service::Classroom),
        "drive" => Ok(Service::Drive),
        "docs" => Ok(Service::Docs),
        "slides" => Ok(Service::Slides),
        "contacts" => Ok(Service::Contacts),
        "tasks" => Ok(Service::Tasks),
        "people" => Ok(Service::People),
        "sheets" => Ok(Service::Sheets),
        "forms" => Ok(Service::Forms),
        "appscript" | "apps-script" | "app-script" => Ok(Service::AppScript),
        "groups" => Ok(Service::Groups),
        "keep" => Ok(Service::Keep),
        "" => anyhow::bail!("empty service name"),
        other => anyhow::bail!("unknown service: {}", other),
    }
}

/// Returns all 15 services in canonical order.
pub fn all_services() -> Vec<Service> {
    vec![
        Service::Gmail,
        Service::Calendar,
        Service::Chat,
        Service::Classroom,
        Service::Drive,
        Service::Docs,
        Service::Slides,
        Service::Contacts,
        Service::Tasks,
        Service::People,
        Service::Sheets,
        Service::Forms,
        Service::AppScript,
        Service::Groups,
        Service::Keep,
    ]
}

/// Returns user services (those marked as user-accessible, excludes Groups/Keep).
pub fn user_services() -> Vec<Service> {
    all_services()
        .into_iter()
        .filter(|s| !matches!(s, Service::Groups | Service::Keep))
        .collect()
}

/// Returns service info for all services.
pub fn services_info() -> Vec<ServiceInfo> {
    all_services()
        .into_iter()
        .map(|service| {
            let user = !matches!(service, Service::Groups | Service::Keep);
            let scopes = scopes::scopes_for_service(service);
            ServiceInfo {
                service,
                user,
                scopes,
                apis: vec![],
                note: String::new(),
            }
        })
        .collect()
}

/// Resolve account from flags > env > config > keyring default > single stored.
pub fn resolve_account(
    account_flag: Option<&str>,
    config: &crate::config::ConfigFile,
    store: &dyn CredentialStore,
    client: &str,
) -> anyhow::Result<String> {
    // 1. --account flag
    if let Some(acct) = account_flag {
        let acct = acct.trim();
        if !acct.is_empty() {
            // Check if it's an alias
            if let Some(resolved) = config.account_aliases.as_ref().and_then(|a| a.get(acct)) {
                return Ok(resolved.clone());
            }
            return Ok(acct.to_string());
        }
    }

    // 2. GOG_ACCOUNT env var
    if let Ok(env_acct) = std::env::var("GOG_ACCOUNT") {
        let env_acct = env_acct.trim().to_string();
        if !env_acct.is_empty() {
            return Ok(env_acct);
        }
    }

    // 3. keyring default account
    if let Ok(Some(default)) = store.get_default_account(client) {
        return Ok(default);
    }

    // 4. Single stored token
    let keys = store.keys()?;
    let matching: Vec<_> = keys
        .iter()
        .filter_map(|k| parse_token_key(k))
        .filter(|(c, _)| c == client)
        .collect();
    if matching.len() == 1 {
        return Ok(matching[0].1.clone());
    }

    anyhow::bail!("no account specified; use --account, set GOG_ACCOUNT, or run 'omega-google auth login'")
}

/// Parse a keyring token key into (client, email).
pub fn parse_token_key(key: &str) -> Option<(String, String)> {
    let rest = key.strip_prefix("token:")?;
    if rest.is_empty() || rest.trim().is_empty() {
        return None;
    }
    // Modern format: token:<client>:<email>
    // Legacy format: token:<email>
    let parts: Vec<&str> = rest.splitn(2, ':').collect();
    if parts.len() == 2 {
        let client = parts[0].trim();
        let email = parts[1].trim();
        if client.is_empty() || email.is_empty() {
            return None;
        }
        // Check if it looks like an email (contains @) -- if so, it's modern format
        if email.contains('@') {
            return Some((client.to_string(), email.to_string()));
        }
        // If the first part contains @ it's a legacy key: token:<email>
        if client.contains('@') {
            return Some(("default".to_string(), format!("{}:{}", client, email)));
        }
        return None;
    }
    // Single part after "token:" -- legacy format
    let email = parts[0].trim();
    if email.is_empty() || !email.contains('@') {
        return None;
    }
    Some(("default".to_string(), email.to_string()))
}

/// Build a keyring token key from client and email.
pub fn token_key(client: &str, email: &str) -> String {
    format!("token:{}:{}", client, email)
}

/// Build a legacy keyring token key (no client prefix).
pub fn legacy_token_key(email: &str) -> String {
    format!("token:{}", email)
}

#[cfg(test)]
mod tests {
    use super::*;

    // =================================================================
    // REQ-RT-007 (Must): Access token caching in TokenData
    // =================================================================

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: Extend TokenData with access_token: Option<String>
    #[test]
    fn req_rt_007_token_data_has_access_token_field() {
        let token = TokenData {
            client: "default".to_string(),
            email: "user@example.com".to_string(),
            services: vec![Service::Gmail],
            scopes: vec!["https://www.googleapis.com/auth/gmail.modify".to_string()],
            created_at: chrono::Utc::now(),
            refresh_token: "refresh_tok".to_string(),
            access_token: Some("ya29.access_token_here".to_string()),
            expires_at: None,
        };
        assert_eq!(token.access_token.as_deref(), Some("ya29.access_token_here"));
    }

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: Extend TokenData with expires_at: Option<DateTime<Utc>>
    #[test]
    fn req_rt_007_token_data_has_expires_at_field() {
        let now = chrono::Utc::now();
        let expires = now + chrono::Duration::hours(1);
        let token = TokenData {
            client: "default".to_string(),
            email: "user@example.com".to_string(),
            services: vec![],
            scopes: vec![],
            created_at: now,
            refresh_token: "rt".to_string(),
            access_token: Some("at".to_string()),
            expires_at: Some(expires),
        };
        assert!(token.expires_at.is_some());
        assert_eq!(token.expires_at.unwrap(), expires);
    }

    // Requirement: REQ-RT-007 (Must)
    // Acceptance: New fields are Option (can be None)
    #[test]
    fn req_rt_007_token_data_new_fields_optional_none() {
        let token = TokenData {
            client: "default".to_string(),
            email: "user@example.com".to_string(),
            services: vec![],
            scopes: vec![],
            created_at: chrono::Utc::now(),
            refresh_token: "rt".to_string(),
            access_token: None,
            expires_at: None,
        };
        assert!(token.access_token.is_none());
        assert!(token.expires_at.is_none());
    }

    // Requirement: REQ-RT-007 (Must)
    // Edge case: access_token is empty string (valid but degenerate)
    #[test]
    fn req_rt_007_token_data_empty_access_token() {
        let token = TokenData {
            client: "default".to_string(),
            email: "user@example.com".to_string(),
            services: vec![],
            scopes: vec![],
            created_at: chrono::Utc::now(),
            refresh_token: "rt".to_string(),
            access_token: Some("".to_string()),
            expires_at: None,
        };
        assert_eq!(token.access_token.as_deref(), Some(""));
    }

    // Requirement: REQ-RT-007 (Must)
    // Edge case: expires_at in the far past
    #[test]
    fn req_rt_007_token_data_expires_at_in_past() {
        let past = chrono::Utc::now() - chrono::Duration::days(365);
        let token = TokenData {
            client: "default".to_string(),
            email: "user@example.com".to_string(),
            services: vec![],
            scopes: vec![],
            created_at: chrono::Utc::now() - chrono::Duration::days(366),
            refresh_token: "rt".to_string(),
            access_token: Some("expired_at".to_string()),
            expires_at: Some(past),
        };
        // Token should be constructible with past expiry
        assert!(token.expires_at.unwrap() < chrono::Utc::now());
    }

    // Requirement: REQ-RT-007 (Must)
    // Edge case: TokenData clone preserves new fields
    #[test]
    fn req_rt_007_token_data_clone_preserves_new_fields() {
        let now = chrono::Utc::now();
        let expires = now + chrono::Duration::hours(1);
        let token = TokenData {
            client: "default".to_string(),
            email: "user@example.com".to_string(),
            services: vec![Service::Drive],
            scopes: vec!["drive".to_string()],
            created_at: now,
            refresh_token: "rt".to_string(),
            access_token: Some("at_clone_test".to_string()),
            expires_at: Some(expires),
        };
        let cloned = token.clone();
        assert_eq!(cloned.access_token, token.access_token);
        assert_eq!(cloned.expires_at, token.expires_at);
    }

    // Requirement: REQ-RT-007 (Must)
    // Security: access_token must not appear in Debug output of errors/logs
    // (This verifies the field exists; actual redaction is tested at serialization level)
    #[test]
    fn req_rt_007_token_data_debug_contains_access_token() {
        // This test documents that Debug DOES contain access_token.
        // A future enhancement could add a custom Debug impl that redacts it.
        // For now, we just verify the struct is Debug-printable.
        let token = TokenData {
            client: "default".to_string(),
            email: "user@example.com".to_string(),
            services: vec![],
            scopes: vec![],
            created_at: chrono::Utc::now(),
            refresh_token: "secret_refresh".to_string(),
            access_token: Some("secret_access".to_string()),
            expires_at: None,
        };
        let debug_str = format!("{:?}", token);
        // Verify it compiles and produces output
        assert!(!debug_str.is_empty());
    }
}
