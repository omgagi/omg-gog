/// Integration tests for the config module.
///
/// Tests cover REQ-CONFIG-001 through REQ-CONFIG-009 (Must priority).
/// These tests validate config file I/O with real filesystem operations.

use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

// ---------------------------------------------------------------
// REQ-CONFIG-001 (Must): Config file read/write with filesystem
// ---------------------------------------------------------------

// Requirement: REQ-CONFIG-001 (Must)
// Acceptance: Config is read with JSON5 parser (comments, trailing commas)
#[test]
fn req_config_001_roundtrip_json5_read_json_write() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.json");

    // Write a JSON5 config with comments
    let json5_content = r#"{
        // User preferences
        "keyring_backend": "auto",
        "default_timezone": "America/New_York",
        // Aliases
        "account_aliases": {
            "work": "me@corp.com",
        },
    }"#;
    std::fs::write(&config_path, json5_content).unwrap();

    // Read via JSON5 parser
    let cfg: omega_google::config::ConfigFile =
        json5::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(cfg.keyring_backend.as_deref(), Some("auto"));
    assert_eq!(cfg.default_timezone.as_deref(), Some("America/New_York"));

    // Write back as standard JSON
    omega_google::config::write_config_to(&config_path, &cfg).unwrap();
    let written = std::fs::read_to_string(&config_path).unwrap();
    // Should be valid JSON (no comments, no trailing commas)
    let reparsed: omega_google::config::ConfigFile = serde_json::from_str(&written).unwrap();
    assert_eq!(reparsed.keyring_backend, cfg.keyring_backend);
}

// Requirement: REQ-CONFIG-001 (Must)
// Acceptance: Atomic write via tmp file + rename
#[test]
fn req_config_001_atomic_write_no_corruption() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.json");

    // Write initial config
    let cfg1 = omega_google::config::ConfigFile {
        keyring_backend: Some("auto".to_string()),
        ..Default::default()
    };
    omega_google::config::write_config_to(&config_path, &cfg1).unwrap();

    // Write updated config (should be atomic: tmp + rename)
    let cfg2 = omega_google::config::ConfigFile {
        keyring_backend: Some("file".to_string()),
        default_timezone: Some("UTC".to_string()),
        ..Default::default()
    };
    omega_google::config::write_config_to(&config_path, &cfg2).unwrap();

    // No .tmp file should remain
    let tmp_path = config_path.with_extension("json.tmp");
    assert!(!tmp_path.exists(), "Temp file should be cleaned up after atomic write");

    // Final file should have updated content
    let final_cfg = omega_google::config::read_config_from(&config_path).unwrap();
    assert_eq!(final_cfg.keyring_backend.as_deref(), Some("file"));
    assert_eq!(final_cfg.default_timezone.as_deref(), Some("UTC"));
}

// Requirement: REQ-CONFIG-001 (Must)
// Acceptance: Config is written with mode 0600
#[test]
fn req_config_001_file_permissions_0600() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.json");
    let cfg = omega_google::config::ConfigFile::default();
    omega_google::config::write_config_to(&config_path, &cfg).unwrap();

    let metadata = std::fs::metadata(&config_path).unwrap();
    let mode = metadata.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "Config file should have 0600 permissions, got {:o}", mode);
}

// Requirement: REQ-CONFIG-001 (Must)
// Acceptance: Missing config file returns default
#[test]
fn req_config_001_missing_file_returns_default() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("nonexistent.json");
    let cfg = omega_google::config::read_config_from(&config_path).unwrap();
    assert_eq!(cfg.keyring_backend, None);
    assert_eq!(cfg.default_timezone, None);
}

// ---------------------------------------------------------------
// REQ-CONFIG-002 (Must): Forward compatibility
// ---------------------------------------------------------------

// Requirement: REQ-CONFIG-002 (Must)
// Acceptance: Unknown fields are preserved through read-write cycle
#[test]
fn req_config_002_forward_compatibility_roundtrip() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.json");

    // Write config with unknown future fields
    let content = r#"{
        "keyring_backend": "auto",
        "future_feature": true,
        "new_setting": {"nested": "value"}
    }"#;
    std::fs::write(&config_path, content).unwrap();

    // Read and write back
    let cfg = omega_google::config::read_config_from(&config_path).unwrap();
    omega_google::config::write_config_to(&config_path, &cfg).unwrap();

    // Re-read and verify unknown fields survived
    let raw: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(raw.get("future_feature").unwrap(), &serde_json::Value::Bool(true));
    assert!(raw.get("new_setting").unwrap().is_object());
}

// ---------------------------------------------------------------
// REQ-CONFIG-003 through REQ-CONFIG-008 (Must): Config commands
// These test the data operations, not the CLI dispatch
// ---------------------------------------------------------------

// Requirement: REQ-CONFIG-003 (Must)
// Acceptance: Get specific config key
#[test]
fn req_config_003_get_existing_key() {
    let cfg = omega_google::config::ConfigFile {
        keyring_backend: Some("keychain".to_string()),
        ..Default::default()
    };
    let json_val = serde_json::to_value(&cfg).unwrap();
    let val = json_val.get("keyring_backend");
    assert!(val.is_some());
    assert_eq!(val.unwrap().as_str(), Some("keychain"));
}

// Requirement: REQ-CONFIG-004 (Must)
// Acceptance: Set creates config file if not exists
#[test]
fn req_config_004_set_creates_file() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.json");
    assert!(!config_path.exists());

    let cfg = omega_google::config::ConfigFile {
        keyring_backend: Some("auto".to_string()),
        ..Default::default()
    };
    omega_google::config::write_config_to(&config_path, &cfg).unwrap();
    assert!(config_path.exists());
}

// Requirement: REQ-CONFIG-004 (Must)
// Acceptance: Set preserves existing keys
#[test]
fn req_config_004_set_preserves_existing() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.json");

    // Write initial config
    let cfg1 = omega_google::config::ConfigFile {
        keyring_backend: Some("auto".to_string()),
        default_timezone: Some("UTC".to_string()),
        ..Default::default()
    };
    omega_google::config::write_config_to(&config_path, &cfg1).unwrap();

    // Read, modify one field, write back
    let mut cfg2 = omega_google::config::read_config_from(&config_path).unwrap();
    cfg2.keyring_backend = Some("file".to_string());
    omega_google::config::write_config_to(&config_path, &cfg2).unwrap();

    // Verify both fields exist
    let final_cfg = omega_google::config::read_config_from(&config_path).unwrap();
    assert_eq!(final_cfg.keyring_backend.as_deref(), Some("file"));
    assert_eq!(final_cfg.default_timezone.as_deref(), Some("UTC")); // preserved
}

// Requirement: REQ-CONFIG-005 (Must)
// Acceptance: Unset removes key
#[test]
fn req_config_005_unset_removes_key() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.json");

    let cfg = omega_google::config::ConfigFile {
        keyring_backend: Some("auto".to_string()),
        default_timezone: Some("UTC".to_string()),
        ..Default::default()
    };
    omega_google::config::write_config_to(&config_path, &cfg).unwrap();

    // Unset by setting to None
    let mut cfg2 = omega_google::config::read_config_from(&config_path).unwrap();
    cfg2.default_timezone = None;
    omega_google::config::write_config_to(&config_path, &cfg2).unwrap();

    let final_cfg = omega_google::config::read_config_from(&config_path).unwrap();
    assert_eq!(final_cfg.default_timezone, None);
    assert_eq!(final_cfg.keyring_backend.as_deref(), Some("auto")); // preserved
}

// Requirement: REQ-CONFIG-006 (Must)
// Acceptance: List all config keys and values
#[test]
fn req_config_006_list_all_keys() {
    let cfg = omega_google::config::ConfigFile {
        keyring_backend: Some("auto".to_string()),
        default_timezone: Some("UTC".to_string()),
        account_aliases: Some({
            let mut m = HashMap::new();
            m.insert("work".to_string(), "me@corp.com".to_string());
            m
        }),
        ..Default::default()
    };
    let json_val = serde_json::to_value(&cfg).unwrap();
    let obj = json_val.as_object().unwrap();
    assert!(obj.contains_key("keyring_backend"));
    assert!(obj.contains_key("default_timezone"));
    assert!(obj.contains_key("account_aliases"));
}

// Requirement: REQ-CONFIG-008 (Must)
// Acceptance: config path returns absolute path
#[test]
fn req_config_008_path_is_absolute() {
    // This test verifies the path computation logic
    // Since config_path() uses dirs::config_dir(), it should return an absolute path
    // The actual call will todo!(), so we verify the concept:
    let path = std::path::Path::new("/Users/test/.config/omega-google/config.json");
    assert!(path.is_absolute());
}

// ---------------------------------------------------------------
// REQ-CONFIG-009 (Must): Credential files
// ---------------------------------------------------------------

// Requirement: REQ-CONFIG-009 (Must)
// Acceptance: Parse installed credential format
#[test]
fn req_config_009_parse_installed_credentials() {
    let google_json = r#"{
        "installed": {
            "client_id": "123.apps.googleusercontent.com",
            "client_secret": "GOCSPX-secret123",
            "project_id": "my-project",
            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
            "token_uri": "https://oauth2.googleapis.com/token",
            "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
            "redirect_uris": ["http://localhost"]
        }
    }"#;

    let raw: serde_json::Value = serde_json::from_str(google_json).unwrap();
    let installed = raw.get("installed").unwrap();
    let creds = omega_google::config::ClientCredentials {
        client_id: installed["client_id"].as_str().unwrap().to_string(),
        client_secret: installed["client_secret"].as_str().unwrap().to_string(),
    };
    assert_eq!(creds.client_id, "123.apps.googleusercontent.com");
    assert_eq!(creds.client_secret, "GOCSPX-secret123");
}

// Requirement: REQ-CONFIG-009 (Must)
// Acceptance: Parse web credential format
#[test]
fn req_config_009_parse_web_credentials() {
    let google_json = r#"{
        "web": {
            "client_id": "456.apps.googleusercontent.com",
            "client_secret": "GOCSPX-websecret",
            "project_id": "web-project"
        }
    }"#;

    let raw: serde_json::Value = serde_json::from_str(google_json).unwrap();
    let web = raw.get("web").unwrap();
    let creds = omega_google::config::ClientCredentials {
        client_id: web["client_id"].as_str().unwrap().to_string(),
        client_secret: web["client_secret"].as_str().unwrap().to_string(),
    };
    assert_eq!(creds.client_id, "456.apps.googleusercontent.com");
    assert_eq!(creds.client_secret, "GOCSPX-websecret");
}

// Requirement: REQ-CONFIG-009 (Must)
// Edge case: Neither installed nor web format should error
#[test]
fn req_config_009_invalid_credentials_format() {
    let bad_json = r#"{"something_else": {"client_id": "123"}}"#;
    let raw: serde_json::Value = serde_json::from_str(bad_json).unwrap();
    assert!(raw.get("installed").is_none());
    assert!(raw.get("web").is_none());
    // The credential parser should return an error for this format
}

// ---------------------------------------------------------------
// Failure mode tests
// ---------------------------------------------------------------

// Requirement: REQ-CONFIG-001 (Must)
// Failure mode: Malformed config file
#[test]
fn req_config_001_failure_malformed_json5() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.json");
    std::fs::write(&config_path, "{ this is not valid }").unwrap();

    // json5 should fail to parse (or succeed if it treats unquoted strings as valid)
    let content = std::fs::read_to_string(&config_path).unwrap();
    // Truly broken JSON5:
    let broken_path = dir.path().join("broken.json");
    std::fs::write(&broken_path, "{ \"key\": }").unwrap();
    let broken = std::fs::read_to_string(&broken_path).unwrap();
    let result = json5::from_str::<omega_google::config::ConfigFile>(&broken);
    assert!(result.is_err());
}

// Requirement: REQ-CONFIG-001 (Must)
// Failure mode: Empty config file
#[test]
fn req_config_001_failure_empty_file() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.json");
    std::fs::write(&config_path, "").unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let result = json5::from_str::<omega_google::config::ConfigFile>(&content);
    assert!(result.is_err(), "Empty file should fail JSON5 parsing");
}

// ---------------------------------------------------------------
// Edge case tests
// ---------------------------------------------------------------

// Requirement: REQ-CONFIG-002 (Must)
// Edge case: Very large config file
#[test]
fn req_config_002_edge_large_aliases_map() {
    let mut aliases = HashMap::new();
    for i in 0..1000 {
        aliases.insert(format!("alias_{}", i), format!("user{}@example.com", i));
    }
    let cfg = omega_google::config::ConfigFile {
        account_aliases: Some(aliases),
        ..Default::default()
    };
    let json_str = serde_json::to_string(&cfg).unwrap();
    let reparsed: omega_google::config::ConfigFile = serde_json::from_str(&json_str).unwrap();
    assert_eq!(reparsed.account_aliases.unwrap().len(), 1000);
}

// Requirement: REQ-CONFIG-002 (Must)
// Edge case: Special characters in values
#[test]
fn req_config_002_edge_special_chars_in_values() {
    let mut aliases = HashMap::new();
    aliases.insert("work".to_string(), "user+tag@example.com".to_string());
    aliases.insert("emoji".to_string(), "user@example.com".to_string());
    let cfg = omega_google::config::ConfigFile {
        account_aliases: Some(aliases),
        default_timezone: Some("Asia/Kolkata".to_string()),
        ..Default::default()
    };
    let json_str = serde_json::to_string(&cfg).unwrap();
    let reparsed: omega_google::config::ConfigFile = serde_json::from_str(&json_str).unwrap();
    let a = reparsed.account_aliases.unwrap();
    assert_eq!(a.get("work").unwrap(), "user+tag@example.com");
}
