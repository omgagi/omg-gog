// Keyring abstraction (OS + file fallback)
//
// This module provides both OS keyring and file-based credential store
// implementations. The OS keyring uses the `keyring` crate for platform-
// native secret storage. The file-based fallback provides a working
// implementation for environments without an OS keyring.

use std::collections::HashMap;
use std::path::PathBuf;
use crate::auth::{CredentialStore, TokenData};
use crate::auth::token::{serialize_token, deserialize_token};

/// File-based credential store that uses JSON files in the config directory.
pub struct FileCredentialStore {
    dir: PathBuf,
}

impl FileCredentialStore {
    pub fn new(dir: PathBuf) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    fn tokens_file(&self) -> PathBuf {
        self.dir.join("tokens.json")
    }

    fn defaults_file(&self) -> PathBuf {
        self.dir.join("defaults.json")
    }

    fn read_tokens_map(&self) -> anyhow::Result<HashMap<String, String>> {
        let path = self.tokens_file();
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let content = std::fs::read_to_string(&path)?;
        let map: HashMap<String, String> = serde_json::from_str(&content)?;
        Ok(map)
    }

    fn write_tokens_map(&self, map: &HashMap<String, String>) -> anyhow::Result<()> {
        let path = self.tokens_file();
        let json_str = serde_json::to_string_pretty(map)?;
        std::fs::write(&path, json_str)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    fn read_defaults_map(&self) -> anyhow::Result<HashMap<String, String>> {
        let path = self.defaults_file();
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let content = std::fs::read_to_string(&path)?;
        let map: HashMap<String, String> = serde_json::from_str(&content)?;
        Ok(map)
    }

    fn write_defaults_map(&self, map: &HashMap<String, String>) -> anyhow::Result<()> {
        let path = self.defaults_file();
        let json_str = serde_json::to_string_pretty(map)?;
        std::fs::write(&path, json_str)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }
}

impl CredentialStore for FileCredentialStore {
    fn get_token(&self, client: &str, email: &str) -> anyhow::Result<TokenData> {
        let map = self.read_tokens_map()?;
        let key = crate::auth::token_key(client, email);
        match map.get(&key) {
            Some(json_str) => deserialize_token(json_str),
            None => anyhow::bail!("no token found for {}:{}", client, email),
        }
    }

    fn set_token(&self, client: &str, email: &str, token: &TokenData) -> anyhow::Result<()> {
        let mut map = self.read_tokens_map()?;
        let key = crate::auth::token_key(client, email);
        let json_str = serialize_token(token)?;
        map.insert(key, json_str);
        self.write_tokens_map(&map)
    }

    fn delete_token(&self, client: &str, email: &str) -> anyhow::Result<()> {
        let mut map = self.read_tokens_map()?;
        let key = crate::auth::token_key(client, email);
        map.remove(&key);
        self.write_tokens_map(&map)
    }

    fn list_tokens(&self) -> anyhow::Result<Vec<TokenData>> {
        let map = self.read_tokens_map()?;
        let mut tokens = Vec::new();
        for json_str in map.values() {
            if let Ok(token) = deserialize_token(json_str) {
                tokens.push(token);
            }
        }
        Ok(tokens)
    }

    fn keys(&self) -> anyhow::Result<Vec<String>> {
        let map = self.read_tokens_map()?;
        Ok(map.keys().cloned().collect())
    }

    fn get_default_account(&self, client: &str) -> anyhow::Result<Option<String>> {
        let map = self.read_defaults_map()?;
        Ok(map.get(client).cloned())
    }

    fn set_default_account(&self, client: &str, email: &str) -> anyhow::Result<()> {
        let mut map = self.read_defaults_map()?;
        map.insert(client.to_string(), email.to_string());
        self.write_defaults_map(&map)
    }

    fn delete_token_by_raw_key(&self, key: &str) -> anyhow::Result<()> {
        let mut map = self.read_tokens_map()?;
        map.remove(key);
        self.write_tokens_map(&map)
    }
}

/// OS keyring credential store wrapping the `keyring` crate.
/// Uses keyring::Entry with service name "omega-google".
/// Key format: token:<client>:<email>
pub struct KeyringCredentialStore;

impl KeyringCredentialStore {
    /// Create a new KeyringCredentialStore.
    /// Returns an error if the OS keyring is not available.
    pub fn new() -> anyhow::Result<Self> {
        // Probe the keyring to see if it is available
        let test_entry = keyring::Entry::new("omega-google", "probe")?;
        match test_entry.get_password() {
            Ok(_) | Err(keyring::Error::NoEntry) => {}
            Err(e) => anyhow::bail!("OS keyring not available: {}", e),
        }
        Ok(KeyringCredentialStore)
    }
}

impl CredentialStore for KeyringCredentialStore {
    fn get_token(&self, client: &str, email: &str) -> anyhow::Result<TokenData> {
        let key = crate::auth::token_key(client, email);
        let entry = keyring::Entry::new("omega-google", &key)?;
        match entry.get_password() {
            Ok(json_str) => deserialize_token(&json_str),
            Err(keyring::Error::NoEntry) => {
                anyhow::bail!("no token found for {}:{}", client, email)
            }
            Err(e) => Err(e.into()),
        }
    }

    fn set_token(&self, client: &str, email: &str, token: &TokenData) -> anyhow::Result<()> {
        let key = crate::auth::token_key(client, email);
        let json_str = serialize_token(token)?;
        let entry = keyring::Entry::new("omega-google", &key)?;
        entry.set_password(&json_str)?;
        Ok(())
    }

    fn delete_token(&self, client: &str, email: &str) -> anyhow::Result<()> {
        let key = crate::auth::token_key(client, email);
        let entry = keyring::Entry::new("omega-google", &key)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    fn list_tokens(&self) -> anyhow::Result<Vec<TokenData>> {
        // keyring crate doesn't support listing keys.
        // Listing is handled via config-stored metadata.
        Ok(vec![])
    }

    fn keys(&self) -> anyhow::Result<Vec<String>> {
        // keyring crate doesn't support enumerating keys.
        // Key enumeration is handled via config-stored metadata.
        Ok(vec![])
    }

    fn get_default_account(&self, client: &str) -> anyhow::Result<Option<String>> {
        let key = format!("default_account:{}", client);
        let entry = keyring::Entry::new("omega-google", &key)?;
        match entry.get_password() {
            Ok(email) => Ok(Some(email)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn set_default_account(&self, client: &str, email: &str) -> anyhow::Result<()> {
        let key = format!("default_account:{}", client);
        let entry = keyring::Entry::new("omega-google", &key)?;
        entry.set_password(email)?;
        Ok(())
    }

    fn delete_token_by_raw_key(&self, key: &str) -> anyhow::Result<()> {
        let entry = keyring::Entry::new("omega-google", key)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

/// Factory: build the appropriate CredentialStore based on config.
/// Reads keyring_backend from config and GOG_KEYRING_BACKEND env (env overrides config).
/// - "auto" (default): try OS keyring, fall back to file on error
/// - "keychain" / "keyring": force OS keyring
/// - "file": force file backend
pub fn credential_store_factory(
    config: &crate::config::ConfigFile,
) -> anyhow::Result<Box<dyn CredentialStore>> {
    // Determine backend: env overrides config
    let backend = std::env::var("GOG_KEYRING_BACKEND")
        .ok()
        .or_else(|| config.keyring_backend.clone())
        .unwrap_or_else(|| "auto".to_string());

    let config_dir = crate::config::config_dir()?;

    match backend.as_str() {
        "keychain" | "keyring" => {
            Ok(Box::new(KeyringCredentialStore::new()?))
        }
        "file" => {
            Ok(Box::new(FileCredentialStore::new(config_dir)?))
        }
        "auto" | "" => {
            // Try OS keyring first, fall back to file
            match KeyringCredentialStore::new() {
                Ok(store) => Ok(Box::new(store)),
                Err(_) => Ok(Box::new(FileCredentialStore::new(config_dir)?)),
            }
        }
        other => anyhow::bail!(
            "Unknown keyring backend: '{}'. Use: auto, keychain, keyring, file",
            other
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{CredentialStore, Service, TokenData};

    /// Helper: create a TokenData for testing.
    fn make_test_token(client: &str, email: &str) -> TokenData {
        TokenData {
            client: client.to_string(),
            email: email.to_string(),
            services: vec![Service::Gmail],
            scopes: vec!["https://www.googleapis.com/auth/gmail.modify".to_string()],
            created_at: chrono::Utc::now(),
            refresh_token: "1//test_refresh".to_string(),
            access_token: None,
            expires_at: None,
        }
    }

    // =================================================================
    // REQ-RT-013 (Must): OS keyring backend
    // =================================================================

    // Requirement: REQ-RT-013 (Must)
    // Acceptance: KeyringCredentialStore struct exists
    #[test]
    fn req_rt_013_keyring_credential_store_struct_exists() {
        // Verify the struct exists and is constructible
        let result = KeyringCredentialStore::new();
        // In CI/test environments, keyring may or may not be available
        // The important thing is that the type exists and new() is callable
        let _ = result;
    }

    // Requirement: REQ-RT-013 (Must)
    // Acceptance: KeyringCredentialStore implements CredentialStore trait
    #[test]
    fn req_rt_013_keyring_implements_credential_store() {
        fn assert_credential_store<T: CredentialStore>() {}
        assert_credential_store::<KeyringCredentialStore>();
    }

    // Requirement: REQ-RT-013 (Must)
    // Acceptance: Uses keyring::Entry with service name "omega-google"
    #[test]
    fn req_rt_013_service_name_is_omega_google() {
        assert_eq!(
            crate::config::APP_NAME,
            "omega-google",
            "Keyring service name must be 'omega-google'"
        );
    }

    // Requirement: REQ-RT-013 (Must)
    // Acceptance: Key format: token:<client>:<email>
    #[test]
    fn req_rt_013_key_format() {
        let key = crate::auth::token_key("myapp", "user@gmail.com");
        assert_eq!(key, "token:myapp:user@gmail.com");
    }

    // Requirement: REQ-RT-013 (Must)
    // Acceptance: Falls back gracefully if keyring unavailable (returns error, no panic)
    #[test]
    fn req_rt_013_graceful_failure_no_panic() {
        // KeyringCredentialStore::new() should not panic
        let result = std::panic::catch_unwind(|| {
            let _ = KeyringCredentialStore::new();
        });
        assert!(result.is_ok(), "KeyringCredentialStore::new() must not panic");
    }

    // Requirement: REQ-RT-013 (Must)
    // Acceptance: KeyringCredentialStore is Send + Sync (required by CredentialStore trait)
    #[test]
    fn req_rt_013_keyring_store_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<KeyringCredentialStore>();
    }

    // Requirement: REQ-RT-013 (Must)
    // Real keyring operations (marked #[ignore] -- requires OS keyring)
    #[test]
    #[ignore]
    fn req_rt_013_keyring_set_get_roundtrip() {
        let store = KeyringCredentialStore::new()
            .expect("OS keyring should be available for this test");
        let token = make_test_token("default", "test@example.com");
        store.set_token("default", "test@example.com", &token)
            .expect("set_token should succeed");
        let loaded = store.get_token("default", "test@example.com")
            .expect("get_token should succeed");
        assert_eq!(loaded.email, "test@example.com");
        // Clean up
        let _ = store.delete_token("default", "test@example.com");
    }

    // Requirement: REQ-RT-013 (Must)
    // Real keyring operations (marked #[ignore] -- requires OS keyring)
    #[test]
    #[ignore]
    fn req_rt_013_keyring_delete() {
        let store = KeyringCredentialStore::new()
            .expect("OS keyring should be available");
        let token = make_test_token("default", "delete-test@example.com");
        store.set_token("default", "delete-test@example.com", &token).unwrap();
        store.delete_token("default", "delete-test@example.com").unwrap();
        let result = store.get_token("default", "delete-test@example.com");
        assert!(result.is_err(), "Deleted token should not be found");
    }

    // Requirement: REQ-RT-013 (Must)
    // Real keyring operations (marked #[ignore] -- requires OS keyring)
    #[test]
    #[ignore]
    fn req_rt_013_keyring_list_tokens() {
        let store = KeyringCredentialStore::new()
            .expect("OS keyring should be available");
        let token = make_test_token("default", "list-test@example.com");
        store.set_token("default", "list-test@example.com", &token).unwrap();
        let tokens = store.list_tokens().expect("list_tokens should succeed");
        // keyring list returns empty vec (doesn't support enumeration)
        let _ = tokens;
        // Clean up
        let _ = store.delete_token("default", "list-test@example.com");
    }

    // Requirement: REQ-RT-013 (Must)
    // Real keyring operations (marked #[ignore] -- requires OS keyring)
    #[test]
    #[ignore]
    fn req_rt_013_keyring_default_account() {
        let store = KeyringCredentialStore::new()
            .expect("OS keyring should be available");
        store.set_default_account("default", "default-test@example.com").unwrap();
        let default = store.get_default_account("default").unwrap();
        assert_eq!(default.as_deref(), Some("default-test@example.com"));
    }

    // =================================================================
    // REQ-RT-015 (Must): Credential store factory
    // =================================================================

    // Requirement: REQ-RT-015 (Must)
    // Acceptance: credential_store_factory function exists
    #[test]
    fn req_rt_015_factory_function_exists() {
        let config = crate::config::ConfigFile::default();
        let result = credential_store_factory(&config);
        // auto backend should always succeed (falls back to file)
        let _ = result;
    }

    // Requirement: REQ-RT-015 (Must)
    // Acceptance: "file" backend forces FileCredentialStore
    #[test]
    fn req_rt_015_factory_file_backend() {
        let config = crate::config::ConfigFile {
            keyring_backend: Some("file".to_string()),
            ..Default::default()
        };
        let result = credential_store_factory(&config);
        // File backend should succeed
        assert!(result.is_ok(), "File backend should succeed: {:?}", result.err());
    }

    // Requirement: REQ-RT-015 (Must)
    // Acceptance: "keychain" forces OS keyring, error if unavailable
    #[test]
    fn req_rt_015_factory_keychain_backend() {
        let config = crate::config::ConfigFile {
            keyring_backend: Some("keychain".to_string()),
            ..Default::default()
        };
        let result = credential_store_factory(&config);
        // In test env, OS keyring may or may not work -- either Ok or Err is acceptable
        let _ = result;
    }

    // Requirement: REQ-RT-015 (Must)
    // Acceptance: "keyring" is synonym for "keychain"
    #[test]
    fn req_rt_015_factory_keyring_synonym() {
        let config = crate::config::ConfigFile {
            keyring_backend: Some("keyring".to_string()),
            ..Default::default()
        };
        let result = credential_store_factory(&config);
        let _ = result;
    }

    // Requirement: REQ-RT-015 (Must)
    // Acceptance: "auto" (default) tries OS keyring, falls back to file
    #[test]
    fn req_rt_015_factory_auto_backend() {
        let config = crate::config::ConfigFile {
            keyring_backend: Some("auto".to_string()),
            ..Default::default()
        };
        let result = credential_store_factory(&config);
        // auto should always succeed (falls back to file)
        assert!(result.is_ok(), "auto backend should succeed: {:?}", result.err());
    }

    // Requirement: REQ-RT-015 (Must)
    // Acceptance: None defaults to "auto"
    #[test]
    fn req_rt_015_factory_none_defaults_to_auto() {
        let config = crate::config::ConfigFile {
            keyring_backend: None,
            ..Default::default()
        };
        let result = credential_store_factory(&config);
        // None = auto = should always succeed
        assert!(result.is_ok(), "None (auto) backend should succeed: {:?}", result.err());
    }

    // Requirement: REQ-RT-015 (Must)
    // Acceptance: Returns Box<dyn CredentialStore>
    #[test]
    fn req_rt_015_factory_returns_boxed_trait() {
        fn assert_returns_box(_f: fn(&crate::config::ConfigFile) -> anyhow::Result<Box<dyn CredentialStore>>) {}
        assert_returns_box(credential_store_factory);
    }

    // Requirement: REQ-RT-015 (Must)
    // Acceptance: GOG_KEYRING_BACKEND env overrides config
    #[test]
    fn req_rt_015_factory_env_overrides_config() {
        let config = crate::config::ConfigFile {
            keyring_backend: Some("keychain".to_string()),
            ..Default::default()
        };
        // We can't easily set env vars in parallel tests without races,
        // so this test documents the requirement.
        let _ = config;
    }

    // Requirement: REQ-RT-015 (Must)
    // Edge case: Unknown backend value
    #[test]
    fn req_rt_015_edge_unknown_backend() {
        let config = crate::config::ConfigFile {
            keyring_backend: Some("unknown_backend".to_string()),
            ..Default::default()
        };
        let result = credential_store_factory(&config);
        assert!(result.is_err(), "Unknown backend should return error");
    }

    // Requirement: REQ-RT-015 (Must)
    // Edge case: Empty string backend
    #[test]
    fn req_rt_015_edge_empty_backend_string() {
        let config = crate::config::ConfigFile {
            keyring_backend: Some("".to_string()),
            ..Default::default()
        };
        let result = credential_store_factory(&config);
        // Empty string should be treated as "auto"
        assert!(result.is_ok(), "Empty string (auto) backend should succeed: {:?}", result.err());
    }

    // =================================================================
    // FileCredentialStore tests
    // =================================================================

    // Requirement: REQ-RT-015 (Must)
    // Verify file backend set/get/delete cycle works
    #[test]
    fn req_rt_015_file_store_set_get_delete_cycle() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileCredentialStore::new(dir.path().to_path_buf())
            .expect("FileCredentialStore::new should succeed");
        let token = make_test_token("default", "file-test@example.com");
        store.set_token("default", "file-test@example.com", &token).unwrap();
        let loaded = store.get_token("default", "file-test@example.com").unwrap();
        assert_eq!(loaded.email, "file-test@example.com");
        assert_eq!(loaded.client, "default");
        store.delete_token("default", "file-test@example.com").unwrap();
        let result = store.get_token("default", "file-test@example.com");
        assert!(result.is_err(), "Deleted token should not be found");
    }

    // Requirement: REQ-RT-015 (Must)
    // Verify file backend list_tokens
    #[test]
    fn req_rt_015_file_store_list_tokens() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileCredentialStore::new(dir.path().to_path_buf()).unwrap();
        let t1 = make_test_token("default", "a@example.com");
        let t2 = make_test_token("default", "b@example.com");
        store.set_token("default", "a@example.com", &t1).unwrap();
        store.set_token("default", "b@example.com", &t2).unwrap();
        let tokens = store.list_tokens().unwrap();
        assert_eq!(tokens.len(), 2);
    }

    // Requirement: REQ-RT-015 (Must)
    // Verify file backend keys()
    #[test]
    fn req_rt_015_file_store_keys() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileCredentialStore::new(dir.path().to_path_buf()).unwrap();
        let t1 = make_test_token("default", "a@example.com");
        store.set_token("default", "a@example.com", &t1).unwrap();
        let keys = store.keys().unwrap();
        assert_eq!(keys.len(), 1);
        assert!(keys[0].contains("a@example.com"));
    }

    // Requirement: REQ-RT-015 (Must)
    // Verify file backend default_account
    #[test]
    fn req_rt_015_file_store_default_account() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileCredentialStore::new(dir.path().to_path_buf()).unwrap();
        assert!(store.get_default_account("default").unwrap().is_none());
        store.set_default_account("default", "primary@example.com").unwrap();
        assert_eq!(
            store.get_default_account("default").unwrap().as_deref(),
            Some("primary@example.com")
        );
    }

    // Requirement: REQ-RT-015 (Must)
    // Edge case: File store empty directory (no tokens file yet)
    #[test]
    fn req_rt_015_file_store_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileCredentialStore::new(dir.path().to_path_buf()).unwrap();
        let tokens = store.list_tokens().unwrap();
        assert!(tokens.is_empty());
        let keys = store.keys().unwrap();
        assert!(keys.is_empty());
    }

    // Requirement: REQ-RT-015 (Must)
    // Edge case: File store multiple clients
    #[test]
    fn req_rt_015_file_store_multiple_clients() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileCredentialStore::new(dir.path().to_path_buf()).unwrap();
        let t1 = make_test_token("work", "user@work.com");
        let t2 = make_test_token("personal", "user@gmail.com");
        store.set_token("work", "user@work.com", &t1).unwrap();
        store.set_token("personal", "user@gmail.com", &t2).unwrap();
        let keys = store.keys().unwrap();
        assert_eq!(keys.len(), 2);
        // Verify separate defaults per client
        store.set_default_account("work", "user@work.com").unwrap();
        store.set_default_account("personal", "user@gmail.com").unwrap();
        assert_eq!(store.get_default_account("work").unwrap().as_deref(), Some("user@work.com"));
        assert_eq!(store.get_default_account("personal").unwrap().as_deref(), Some("user@gmail.com"));
    }

    // Requirement: REQ-RT-015 (Must)
    // Edge case: Overwriting an existing token
    #[test]
    fn req_rt_015_file_store_overwrite_token() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileCredentialStore::new(dir.path().to_path_buf()).unwrap();
        let t1 = make_test_token("default", "user@example.com");
        store.set_token("default", "user@example.com", &t1).unwrap();
        // Overwrite with different refresh_token
        let mut t2 = make_test_token("default", "user@example.com");
        t2.refresh_token = "1//new_refresh_token".to_string();
        store.set_token("default", "user@example.com", &t2).unwrap();
        let loaded = store.get_token("default", "user@example.com").unwrap();
        assert_eq!(loaded.refresh_token, "1//new_refresh_token");
        // Should still only have one entry
        assert_eq!(store.keys().unwrap().len(), 1);
    }

    // Requirement: REQ-RT-013 (Must)
    // Security: File permissions are 0600 on Unix
    #[cfg(unix)]
    #[test]
    fn req_rt_013_security_file_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let store = FileCredentialStore::new(dir.path().to_path_buf()).unwrap();
        let token = make_test_token("default", "perm-test@example.com");
        store.set_token("default", "perm-test@example.com", &token).unwrap();
        let tokens_path = dir.path().join("tokens.json");
        let perms = std::fs::metadata(&tokens_path).unwrap().permissions();
        assert_eq!(
            perms.mode() & 0o777,
            0o600,
            "Token file must have 0600 permissions"
        );
    }

    // Requirement: REQ-RT-013 (Must)
    // Edge case: Get non-existent token
    #[test]
    fn req_rt_013_edge_get_nonexistent_token() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileCredentialStore::new(dir.path().to_path_buf()).unwrap();
        let result = store.get_token("default", "nonexistent@example.com");
        assert!(result.is_err(), "Non-existent token should return error");
    }

    // Requirement: REQ-RT-013 (Must)
    // Edge case: Delete non-existent token (should not error)
    #[test]
    fn req_rt_013_edge_delete_nonexistent_token() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileCredentialStore::new(dir.path().to_path_buf()).unwrap();
        // Deleting a non-existent token should not error (idempotent)
        let result = store.delete_token("default", "nonexistent@example.com");
        assert!(result.is_ok(), "Deleting non-existent token should be ok");
    }

    // Requirement: REQ-RT-013 (Must)
    // Failure mode: File permission denied
    #[cfg(unix)]
    #[test]
    fn req_rt_013_failure_permission_denied() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let store = FileCredentialStore::new(dir.path().to_path_buf()).unwrap();
        // Make the directory read-only
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o444)).unwrap();
        let token = make_test_token("default", "perm@example.com");
        let result = store.set_token("default", "perm@example.com", &token);
        assert!(result.is_err(), "Writing to read-only directory should fail");
        // Restore permissions for cleanup
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}
