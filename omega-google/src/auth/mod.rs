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
#[derive(Debug, Clone)]
pub struct TokenData {
    pub client: String,
    pub email: String,
    pub services: Vec<Service>,
    pub scopes: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub refresh_token: String,
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
            if let Some(aliases) = &config.account_aliases {
                if let Some(resolved) = aliases.get(acct) {
                    return Ok(resolved.clone());
                }
            }
            return Ok(acct.to_string());
        }
    }

    // 2. OMEGA_GOOGLE_ACCOUNT env var
    if let Ok(env_acct) = std::env::var("OMEGA_GOOGLE_ACCOUNT") {
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

    anyhow::bail!("no account specified; use --account, set OMEGA_GOOGLE_ACCOUNT, or run 'omega-google auth login'")
}

/// Parse a keyring token key into (client, email).
pub fn parse_token_key(key: &str) -> Option<(String, String)> {
    if !key.starts_with("token:") {
        return None;
    }
    let rest = &key[6..]; // after "token:"
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
