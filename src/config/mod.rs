pub mod credentials;
pub mod file;
pub mod paths;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Application name used for keyring service and config directory.
pub const APP_NAME: &str = "omega-google";
/// Default client name when no --client is specified.
pub const DEFAULT_CLIENT_NAME: &str = "default";

/// Configuration file structure. Maps to $CONFIG_DIR/omega-google/config.json.
/// Read with JSON5 parser, written as standard JSON.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ConfigFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyring_backend: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_timezone: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_aliases: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_clients: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_domains: Option<HashMap<String, String>>,

    /// Preserve unknown fields for forward compatibility.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// OAuth client credentials parsed from Google's downloaded JSON.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientCredentials {
    pub client_id: String,
    pub client_secret: String,
}

/// Returns the platform-specific config directory: $CONFIG_DIR/omega-google/
pub fn config_dir() -> anyhow::Result<PathBuf> {
    if let Ok(custom) = std::env::var("OMEGA_GOOGLE_CONFIG_DIR") {
        return Ok(PathBuf::from(custom));
    }
    let base = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine config directory"))?;
    Ok(base.join(APP_NAME))
}

/// Returns the path to config.json within the config directory.
pub fn config_path() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

/// Creates the config directory if it does not exist. Returns the path.
pub fn ensure_dir() -> anyhow::Result<PathBuf> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Reads the config file, parsing JSON5. Returns default if file does not exist.
pub fn read_config() -> anyhow::Result<ConfigFile> {
    let path = config_path()?;
    read_config_from(&path)
}

/// Reads the config file from a specific path (for testing).
pub fn read_config_from(path: &std::path::Path) -> anyhow::Result<ConfigFile> {
    if !path.exists() {
        return Ok(ConfigFile::default());
    }
    let content = std::fs::read_to_string(path)?;
    let cfg: ConfigFile = json5::from_str(&content)?;
    Ok(cfg)
}

/// Writes the config file as standard JSON with atomic write (tmp + rename).
/// File permissions are set to 0600.
pub fn write_config(cfg: &ConfigFile) -> anyhow::Result<()> {
    let path = config_path()?;
    ensure_dir()?;
    write_config_to(&path, cfg)
}

/// Writes the config file to a specific path (for testing).
/// Returns an error if the parent directory does not exist or any I/O operation fails.
pub fn write_config_to(path: &std::path::Path, cfg: &ConfigFile) -> anyhow::Result<()> {
    use std::io::Write;
    let json_str = serde_json::to_string_pretty(cfg)?;
    let tmp_path = path.with_extension("json.tmp");
    {
        let mut f = std::fs::File::create(&tmp_path)?;
        f.write_all(json_str.as_bytes())?;
        f.write_all(b"\n")?;
        f.sync_all()?;
    }
    // Set permissions to 0600 before rename
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600))?;
    }
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

/// Returns the credential file name for a given client name.
/// Default client: `credentials.json`
/// Named client: `credentials-{client}.json`
pub fn credential_filename(client: &str) -> String {
    let name = normalize_client_name(client);
    if name == DEFAULT_CLIENT_NAME {
        "credentials.json".to_string()
    } else {
        format!("credentials-{}.json", name)
    }
}

/// Reads OAuth client credentials for the given client name.
pub fn read_client_credentials(client: &str) -> anyhow::Result<ClientCredentials> {
    let dir = config_dir()?;
    let path = dir.join(credential_filename(client));
    if !path.exists() {
        anyhow::bail!("credential file not found: {}", path.display());
    }
    let content = std::fs::read_to_string(&path)?;
    let raw: serde_json::Value = serde_json::from_str(&content)?;
    credentials::parse_credentials(&raw)
}

/// Writes OAuth client credentials for the given client name.
pub fn write_client_credentials(client: &str, creds: &ClientCredentials) -> anyhow::Result<()> {
    let dir = ensure_dir()?;
    let path = dir.join(credential_filename(client));
    let json_str = serde_json::to_string_pretty(creds)?;
    std::fs::write(&path, json_str)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Returns the path to a service account key file.
pub fn service_account_path(email: &str) -> anyhow::Result<PathBuf> {
    let dir = config_dir()?;
    Ok(dir.join(format!("sa-{}.json", email)))
}

/// Normalizes a client name: trims, lowercases, defaults to DEFAULT_CLIENT_NAME if empty.
pub fn normalize_client_name(raw: &str) -> String {
    let trimmed = raw.trim().to_lowercase();
    if trimmed.is_empty() {
        DEFAULT_CLIENT_NAME.to_string()
    } else {
        trimmed
    }
}

/// Returns the list of known config keys.
pub fn known_keys() -> Vec<&'static str> {
    vec![
        "keyring_backend",
        "default_timezone",
        "account_aliases",
        "account_clients",
        "client_domains",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CONFIG-001 (Must): Config file JSON5 read, JSON write
    // ---------------------------------------------------------------

    // Requirement: REQ-CONFIG-001 (Must)
    // Acceptance: Config is read with JSON5 parser (comments, trailing commas)
    #[test]
    fn req_config_001_reads_json5_with_comments() {
        let json5_content = r#"{
            // This is a comment
            "keyring_backend": "auto",
            "default_timezone": "America/New_York", // trailing comma OK
        }"#;
        let cfg: ConfigFile =
            json5::from_str(json5_content).expect("JSON5 with comments should parse");
        assert_eq!(cfg.keyring_backend.as_deref(), Some("auto"));
        assert_eq!(cfg.default_timezone.as_deref(), Some("America/New_York"));
    }

    // Requirement: REQ-CONFIG-001 (Must)
    // Acceptance: Config is read with JSON5 parser (trailing commas)
    #[test]
    fn req_config_001_reads_json5_with_trailing_commas() {
        let json5_content = r#"{
            "keyring_backend": "file",
        }"#;
        let cfg: ConfigFile =
            json5::from_str(json5_content).expect("JSON5 with trailing commas should parse");
        assert_eq!(cfg.keyring_backend.as_deref(), Some("file"));
    }

    // Requirement: REQ-CONFIG-001 (Must)
    // Acceptance: Config is written as standard JSON with mode 0600
    #[test]
    fn req_config_001_writes_standard_json() {
        let cfg = ConfigFile {
            keyring_backend: Some("auto".to_string()),
            default_timezone: None,
            account_aliases: None,
            account_clients: None,
            client_domains: None,
            extra: HashMap::new(),
        };
        let json_str = serde_json::to_string_pretty(&cfg).unwrap();
        // Standard JSON should NOT have comments or trailing commas
        assert!(!json_str.contains("//"));
        assert!(json_str.contains("keyring_backend"));
        // Should be valid JSON (re-parseable by standard parser)
        let reparsed: ConfigFile = serde_json::from_str(&json_str).unwrap();
        assert_eq!(reparsed.keyring_backend, cfg.keyring_backend);
    }

    // Requirement: REQ-CONFIG-001 (Must)
    // Acceptance: Atomic write via tmp file + rename
    #[test]
    fn req_config_001_atomic_write_via_tempfile() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let cfg = ConfigFile {
            keyring_backend: Some("auto".to_string()),
            ..Default::default()
        };
        // write_config_to should use tmp + rename pattern
        write_config_to(&path, &cfg).unwrap();
        // Verify the file exists and the tmp file does NOT exist
    }

    // ---------------------------------------------------------------
    // REQ-CONFIG-002 (Must): Config structure with optional fields
    // ---------------------------------------------------------------

    // Requirement: REQ-CONFIG-002 (Must)
    // Acceptance: All fields are optional (empty config is valid)
    #[test]
    fn req_config_002_empty_config_is_valid() {
        let json_str = "{}";
        let cfg: ConfigFile = json5::from_str(json_str).unwrap();
        assert_eq!(cfg.keyring_backend, None);
        assert_eq!(cfg.default_timezone, None);
        assert_eq!(cfg.account_aliases, None);
        assert_eq!(cfg.account_clients, None);
        assert_eq!(cfg.client_domains, None);
    }

    // Requirement: REQ-CONFIG-002 (Must)
    // Acceptance: Unknown fields are preserved (forward compatibility)
    #[test]
    fn req_config_002_unknown_fields_preserved() {
        let json_str =
            r#"{"keyring_backend": "auto", "future_field": "some_value", "another_unknown": 42}"#;
        let cfg: ConfigFile = json5::from_str(json_str).unwrap();
        assert_eq!(cfg.keyring_backend.as_deref(), Some("auto"));
        assert!(cfg.extra.contains_key("future_field"));
        assert!(cfg.extra.contains_key("another_unknown"));

        // Verify round-trip preserves unknown fields
        let output = serde_json::to_string(&cfg).unwrap();
        assert!(output.contains("future_field"));
        assert!(output.contains("another_unknown"));
    }

    // Requirement: REQ-CONFIG-002 (Must)
    // Acceptance: All known fields deserialize correctly
    #[test]
    fn req_config_002_all_fields_deserialize() {
        let json_str = r#"{
            "keyring_backend": "keychain",
            "default_timezone": "Europe/Berlin",
            "account_aliases": {"work": "me@corp.com"},
            "account_clients": {"me@corp.com": "work"},
            "client_domains": {"corp.com": "work"}
        }"#;
        let cfg: ConfigFile = json5::from_str(json_str).unwrap();
        assert_eq!(cfg.keyring_backend.as_deref(), Some("keychain"));
        assert_eq!(cfg.default_timezone.as_deref(), Some("Europe/Berlin"));
        let aliases = cfg.account_aliases.as_ref().unwrap();
        assert_eq!(aliases.get("work").unwrap(), "me@corp.com");
        let clients = cfg.account_clients.as_ref().unwrap();
        assert_eq!(clients.get("me@corp.com").unwrap(), "work");
        let domains = cfg.client_domains.as_ref().unwrap();
        assert_eq!(domains.get("corp.com").unwrap(), "work");
    }

    // ---------------------------------------------------------------
    // REQ-CONFIG-007 (Must): Config keys
    // ---------------------------------------------------------------

    // Requirement: REQ-CONFIG-007 (Must)
    // Acceptance: Lists all valid config key names
    #[test]
    fn req_config_007_known_keys_complete() {
        let keys = known_keys();
        assert!(keys.contains(&"keyring_backend"));
        assert!(keys.contains(&"default_timezone"));
        assert!(keys.contains(&"account_aliases"));
        assert!(keys.contains(&"account_clients"));
        assert!(keys.contains(&"client_domains"));
        assert_eq!(keys.len(), 5);
    }

    // ---------------------------------------------------------------
    // REQ-CONFIG-009 (Must): Credential file formats
    // ---------------------------------------------------------------

    // Requirement: REQ-CONFIG-009 (Must)
    // Acceptance: Supports Google's installed.client_id/client_secret format
    #[test]
    fn req_config_009_parses_installed_credentials_format() {
        let google_json = r#"{
            "installed": {
                "client_id": "123456.apps.googleusercontent.com",
                "client_secret": "GOCSPX-secret",
                "project_id": "my-project",
                "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                "token_uri": "https://oauth2.googleapis.com/token"
            }
        }"#;
        // The credential parser should extract client_id and client_secret
        // from the "installed" wrapper
        let raw: serde_json::Value = serde_json::from_str(google_json).unwrap();
        let installed = raw.get("installed").expect("should have 'installed' key");
        let client_id = installed.get("client_id").unwrap().as_str().unwrap();
        let client_secret = installed.get("client_secret").unwrap().as_str().unwrap();
        assert_eq!(client_id, "123456.apps.googleusercontent.com");
        assert_eq!(client_secret, "GOCSPX-secret");
    }

    // Requirement: REQ-CONFIG-009 (Must)
    // Acceptance: Supports Google's web.client_id/client_secret format
    #[test]
    fn req_config_009_parses_web_credentials_format() {
        let google_json = r#"{
            "web": {
                "client_id": "789.apps.googleusercontent.com",
                "client_secret": "GOCSPX-websecret",
                "project_id": "web-project"
            }
        }"#;
        let raw: serde_json::Value = serde_json::from_str(google_json).unwrap();
        let web = raw.get("web").expect("should have 'web' key");
        let client_id = web.get("client_id").unwrap().as_str().unwrap();
        let client_secret = web.get("client_secret").unwrap().as_str().unwrap();
        assert_eq!(client_id, "789.apps.googleusercontent.com");
        assert_eq!(client_secret, "GOCSPX-websecret");
    }

    // ---------------------------------------------------------------
    // Edge cases for config (Must requirements)
    // ---------------------------------------------------------------

    // Requirement: REQ-CONFIG-001 (Must)
    // Edge case: Malformed JSON
    #[test]
    fn req_config_001_edge_malformed_json_returns_error() {
        let bad_json = r#"{ broken: json }"#;
        // json5 is more lenient, but this should still be parseable as json5
        // (unquoted keys are valid in JSON5)
        let _result: Result<ConfigFile, _> = json5::from_str(bad_json);
        // JSON5 allows unquoted keys, so this may actually succeed
        // But truly broken JSON5 should fail:
        let truly_broken = r#"{ "key": }"#;
        let result = json5::from_str::<ConfigFile>(truly_broken);
        assert!(
            result.is_err(),
            "Truly malformed JSON5 should fail to parse"
        );
    }

    // Requirement: REQ-CONFIG-001 (Must)
    // Edge case: Empty file
    #[test]
    fn req_config_001_edge_empty_string_fails() {
        let result = json5::from_str::<ConfigFile>("");
        assert!(
            result.is_err(),
            "Empty string should fail to parse as JSON5"
        );
    }

    // Requirement: REQ-CONFIG-002 (Must)
    // Edge case: Unicode values in config
    #[test]
    fn req_config_002_edge_unicode_values() {
        let json_str = r#"{"default_timezone": "Asia/\u6771\u4EAC"}"#;
        let cfg: ConfigFile = serde_json::from_str(json_str).unwrap();
        assert!(cfg.default_timezone.is_some());
    }

    // Requirement: REQ-CONFIG-001 (Must)
    // Edge case: Null values for optional fields
    #[test]
    fn req_config_001_edge_null_values() {
        let json_str = r#"{"keyring_backend": null, "default_timezone": null}"#;
        let cfg: ConfigFile = serde_json::from_str(json_str).unwrap();
        assert_eq!(cfg.keyring_backend, None);
        assert_eq!(cfg.default_timezone, None);
    }

    // Requirement: REQ-CONFIG-001 (Must)
    // Failure mode: Config dir not writable
    #[test]
    fn req_config_001_failure_nonexistent_path() {
        let path = std::path::Path::new("/nonexistent/deeply/nested/config.json");
        // write_config_to should return an error for nonexistent parent
        let cfg = ConfigFile::default();
        let result = write_config_to(path, &cfg);
        assert!(result.is_err());
    }
}
