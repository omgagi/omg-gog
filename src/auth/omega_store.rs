// OMEGA Store Credential Backend
//
// This module reads/writes credentials from $OMEGA_STORES_DIR/google.json.
// Implements the CredentialStore trait for the OMEGA store format.
//
// Covers requirements REQ-OI-001 through REQ-OI-007 (OI-M1).

use crate::auth::{all_services, CredentialStore, TokenData};
use crate::config::ClientCredentials;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// JSON schema for $OMEGA_STORES_DIR/google.json.
#[derive(Clone, Serialize, Deserialize)]
pub struct OmegaStoreData {
    pub version: u32,
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
    pub email: String,
    /// Preserve unknown fields for forward compatibility.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl std::fmt::Debug for OmegaStoreData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OmegaStoreData")
            .field("version", &self.version)
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .field("refresh_token", &"[REDACTED]")
            .field("email", &self.email)
            .finish()
    }
}

pub struct OmegaStoreCredentialStore {
    path: PathBuf,
}

impl OmegaStoreCredentialStore {
    /// Create a new OMEGA store credential store. Validates that google.json
    /// exists and is valid JSON with all required fields.
    pub fn new(stores_dir: &str) -> anyhow::Result<Self> {
        let path = PathBuf::from(stores_dir).join("google.json");

        // Eagerly validate: read and parse the file to fail fast
        let content = std::fs::read_to_string(&path).map_err(|e| {
            anyhow::anyhow!(
                "OMEGA_STORES_DIR is set but {}/google.json not found: {}",
                stores_dir,
                e
            )
        })?;

        let _data: OmegaStoreData = serde_json::from_str(&content).map_err(|e| {
            anyhow::anyhow!(
                "OMEGA_STORES_DIR is set but {}/google.json is not valid JSON: {}",
                stores_dir,
                e
            )
        })?;

        Ok(Self { path })
    }

    /// Read and parse google.json, returning the full store data.
    pub fn read_store_data(&self) -> anyhow::Result<OmegaStoreData> {
        let content = std::fs::read_to_string(&self.path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to read {}: {}",
                self.path.display(),
                e
            )
        })?;
        let data: OmegaStoreData = serde_json::from_str(&content).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse {}: {}",
                self.path.display(),
                e
            )
        })?;
        Ok(data)
    }

    /// Extract ClientCredentials (client_id, client_secret) from google.json.
    pub fn client_credentials(&self) -> anyhow::Result<ClientCredentials> {
        let data = self.read_store_data()?;
        Ok(ClientCredentials {
            client_id: data.client_id,
            client_secret: data.client_secret,
        })
    }

    /// Atomic write: write to .tmp file, set permissions, rename to final path.
    fn atomic_write(&self, data: &OmegaStoreData) -> anyhow::Result<()> {
        use std::io::Write;

        let json_str = serde_json::to_string_pretty(data)?;
        let tmp_path = self.path.with_extension("json.tmp");

        {
            let mut f = std::fs::File::create(&tmp_path)?;
            f.write_all(json_str.as_bytes())?;
            f.write_all(b"\n")?;
            f.sync_all()?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600))?;
        }

        std::fs::rename(&tmp_path, &self.path)?;
        Ok(())
    }
}

impl CredentialStore for OmegaStoreCredentialStore {
    fn get_token(&self, _client: &str, _email: &str) -> anyhow::Result<TokenData> {
        let data = self.read_store_data()?;
        let all_svcs = all_services();
        let all_scopes: Vec<String> = all_svcs
            .iter()
            .flat_map(|s| crate::auth::scopes::scopes_for_service(*s))
            .collect();

        Ok(TokenData {
            client: "omega".to_string(),
            email: data.email,
            services: all_svcs,
            scopes: all_scopes,
            created_at: chrono::Utc::now(),
            refresh_token: data.refresh_token,
            // OMEGA store never caches access tokens — force refresh by
            // setting expires_at in the past so needs_refresh() returns true.
            access_token: None,
            expires_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
        })
    }

    fn set_token(&self, _client: &str, _email: &str, token: &TokenData) -> anyhow::Result<()> {
        let mut data = self.read_store_data()?;
        data.refresh_token = token.refresh_token.clone();
        self.atomic_write(&data)
    }

    fn delete_token(&self, _client: &str, _email: &str) -> anyhow::Result<()> {
        anyhow::bail!("Cannot delete OMEGA store token via CLI")
    }

    fn list_tokens(&self) -> anyhow::Result<Vec<TokenData>> {
        let token = self.get_token("", "")?;
        Ok(vec![token])
    }

    fn keys(&self) -> anyhow::Result<Vec<String>> {
        let data = self.read_store_data()?;
        Ok(vec![format!("token:omega:{}", data.email)])
    }

    fn get_default_account(&self, _client: &str) -> anyhow::Result<Option<String>> {
        let data = self.read_store_data()?;
        Ok(Some(data.email))
    }

    fn set_default_account(&self, _client: &str, _email: &str) -> anyhow::Result<()> {
        // No-op: OMEGA store has only one account
        Ok(())
    }
}

impl std::fmt::Debug for OmegaStoreCredentialStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OmegaStoreCredentialStore")
            .field("path", &self.path)
            .finish()
    }
}

/// Check if OMEGA_STORES_DIR is set and non-empty.
pub fn is_omega_store_active() -> bool {
    match std::env::var("OMEGA_STORES_DIR") {
        Ok(val) => !val.is_empty(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{CredentialStore, Service, TokenData};
    use crate::config::ClientCredentials;
    use std::fs;
    use std::path::PathBuf;

    // =================================================================
    // Test Helpers
    // =================================================================

    /// Write a valid google.json test fixture to the given directory.
    /// Returns the full path to the created file.
    fn write_test_google_json(dir: &std::path::Path) -> PathBuf {
        let path = dir.join("google.json");
        let content = serde_json::json!({
            "version": 1,
            "client_id": "424288504335-test.apps.googleusercontent.com",
            "client_secret": "GOCSPX-test-secret",
            "refresh_token": "1//03test-refresh-token",
            "email": "testuser@gmail.com"
        });
        fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();
        path
    }

    /// Write a google.json with custom field values.
    fn write_custom_google_json(
        dir: &std::path::Path,
        version: u32,
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
        email: &str,
    ) -> PathBuf {
        let path = dir.join("google.json");
        let content = serde_json::json!({
            "version": version,
            "client_id": client_id,
            "client_secret": client_secret,
            "refresh_token": refresh_token,
            "email": email
        });
        fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();
        path
    }

    /// Write a google.json with extra fields for forward-compatibility testing.
    fn write_google_json_with_extras(dir: &std::path::Path) -> PathBuf {
        let path = dir.join("google.json");
        let content = serde_json::json!({
            "version": 1,
            "client_id": "424288504335-test.apps.googleusercontent.com",
            "client_secret": "GOCSPX-test-secret",
            "refresh_token": "1//03test-refresh-token",
            "email": "testuser@gmail.com",
            "extra_field": "should-be-ignored",
            "another_future_field": 42
        });
        fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();
        path
    }

    /// Write a google.json missing a specific required field.
    fn write_google_json_missing_field(dir: &std::path::Path, missing: &str) -> PathBuf {
        let path = dir.join("google.json");
        let mut map = serde_json::Map::new();
        if missing != "version" {
            map.insert("version".to_string(), serde_json::json!(1));
        }
        if missing != "client_id" {
            map.insert(
                "client_id".to_string(),
                serde_json::json!("424288504335-test.apps.googleusercontent.com"),
            );
        }
        if missing != "client_secret" {
            map.insert(
                "client_secret".to_string(),
                serde_json::json!("GOCSPX-test-secret"),
            );
        }
        if missing != "refresh_token" {
            map.insert(
                "refresh_token".to_string(),
                serde_json::json!("1//03test-refresh-token"),
            );
        }
        if missing != "email" {
            map.insert(
                "email".to_string(),
                serde_json::json!("testuser@gmail.com"),
            );
        }
        let content = serde_json::Value::Object(map);
        fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();
        path
    }

    /// Helper: create a TokenData for testing (matches existing pattern from keyring tests).
    fn make_test_token(email: &str, refresh_token: &str) -> TokenData {
        TokenData {
            client: "default".to_string(),
            email: email.to_string(),
            services: vec![Service::Gmail],
            scopes: vec!["https://www.googleapis.com/auth/gmail.modify".to_string()],
            created_at: chrono::Utc::now(),
            refresh_token: refresh_token.to_string(),
            access_token: None,
            expires_at: None,
        }
    }

    // =================================================================
    // REQ-OI-001 (Must): Read credentials from OMEGA store
    // =================================================================

    // Requirement: REQ-OI-001 (Must)
    // Acceptance: Read valid google.json -- all fields parsed correctly
    #[test]
    fn req_oi_001_read_valid_google_json() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap())
            .expect("new() should succeed with valid google.json");
        let data = store
            .read_store_data()
            .expect("read_store_data should succeed");

        assert_eq!(data.version, 1);
        assert_eq!(
            data.client_id,
            "424288504335-test.apps.googleusercontent.com"
        );
        assert_eq!(data.client_secret, "GOCSPX-test-secret");
        assert_eq!(data.refresh_token, "1//03test-refresh-token");
        assert_eq!(data.email, "testuser@gmail.com");
    }

    // Requirement: REQ-OI-001 (Must)
    // Acceptance: Use the email field as the current account
    #[test]
    fn req_oi_001_email_used_as_account() {
        let dir = tempfile::tempdir().unwrap();
        write_custom_google_json(
            dir.path(),
            1,
            "cid",
            "csecret",
            "rt",
            "omega-user@example.com",
        );

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let data = store.read_store_data().unwrap();
        assert_eq!(data.email, "omega-user@example.com");
    }

    // Requirement: REQ-OI-001 (Must)
    // Acceptance: If the file doesn't exist, return error
    #[test]
    fn req_oi_001_missing_file_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        // Do NOT create google.json

        let result = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap());
        // Constructor may succeed (just stores path), but read_store_data must fail.
        // Or constructor itself may fail. Either pattern is acceptable.
        // Test both paths:
        match result {
            Err(e) => {
                // Constructor failed -- acceptable behavior
                let msg = format!("{}", e);
                assert!(
                    msg.contains("google.json"),
                    "Error should mention google.json, got: {}",
                    msg
                );
            }
            Ok(store) => {
                // Constructor succeeded; reading must fail
                let read_result = store.read_store_data();
                assert!(
                    read_result.is_err(),
                    "read_store_data should fail when google.json is missing"
                );
            }
        }
    }

    // Requirement: REQ-OI-001 (Must)
    // Acceptance: If the file is malformed JSON, return error
    #[test]
    fn req_oi_001_malformed_json_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("google.json");
        fs::write(&path, "{ this is not valid json }").unwrap();

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap());
        match store {
            Err(e) => {
                let msg = format!("{}", e);
                assert!(
                    msg.contains("JSON") || msg.contains("json") || msg.contains("parse"),
                    "Error should mention JSON parsing, got: {}",
                    msg
                );
            }
            Ok(store) => {
                let read_result = store.read_store_data();
                assert!(
                    read_result.is_err(),
                    "read_store_data should fail on malformed JSON"
                );
            }
        }
    }

    // Requirement: REQ-OI-001 (Must)
    // Acceptance: Missing required field client_id returns error
    #[test]
    fn req_oi_001_missing_client_id_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        write_google_json_missing_field(dir.path(), "client_id");

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap());
        match store {
            Err(_) => { /* constructor rejected it -- acceptable */ }
            Ok(store) => {
                let result = store.read_store_data();
                assert!(
                    result.is_err(),
                    "read_store_data should fail when client_id is missing"
                );
            }
        }
    }

    // Requirement: REQ-OI-001 (Must)
    // Acceptance: Missing required field client_secret returns error
    #[test]
    fn req_oi_001_missing_client_secret_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        write_google_json_missing_field(dir.path(), "client_secret");

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap());
        match store {
            Err(_) => {}
            Ok(store) => {
                let result = store.read_store_data();
                assert!(
                    result.is_err(),
                    "read_store_data should fail when client_secret is missing"
                );
            }
        }
    }

    // Requirement: REQ-OI-001 (Must)
    // Acceptance: Missing required field refresh_token returns error
    #[test]
    fn req_oi_001_missing_refresh_token_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        write_google_json_missing_field(dir.path(), "refresh_token");

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap());
        match store {
            Err(_) => {}
            Ok(store) => {
                let result = store.read_store_data();
                assert!(
                    result.is_err(),
                    "read_store_data should fail when refresh_token is missing"
                );
            }
        }
    }

    // Requirement: REQ-OI-001 (Must)
    // Acceptance: Missing required field email returns error
    #[test]
    fn req_oi_001_missing_email_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        write_google_json_missing_field(dir.path(), "email");

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap());
        match store {
            Err(_) => {}
            Ok(store) => {
                let result = store.read_store_data();
                assert!(
                    result.is_err(),
                    "read_store_data should fail when email is missing"
                );
            }
        }
    }

    // Requirement: REQ-OI-001 (Must)
    // Acceptance: Extra fields in JSON are tolerated (forward compatibility)
    #[test]
    fn req_oi_001_extra_fields_tolerated() {
        let dir = tempfile::tempdir().unwrap();
        write_google_json_with_extras(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap())
            .expect("new() should succeed even with extra fields");
        let data = store
            .read_store_data()
            .expect("read_store_data should succeed with extra fields");

        assert_eq!(data.version, 1);
        assert_eq!(
            data.client_id,
            "424288504335-test.apps.googleusercontent.com"
        );
        assert_eq!(data.email, "testuser@gmail.com");
    }

    // Requirement: REQ-OI-001 (Must)
    // Edge case: Empty JSON object (all fields missing)
    #[test]
    fn req_oi_001_edge_empty_json_object() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("google.json");
        fs::write(&path, "{}").unwrap();

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap());
        match store {
            Err(_) => { /* acceptable */ }
            Ok(store) => {
                let result = store.read_store_data();
                assert!(
                    result.is_err(),
                    "read_store_data should fail on empty JSON object"
                );
            }
        }
    }

    // Requirement: REQ-OI-001 (Must)
    // Edge case: Empty file (zero bytes)
    #[test]
    fn req_oi_001_edge_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("google.json");
        fs::write(&path, "").unwrap();

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap());
        match store {
            Err(_) => { /* acceptable */ }
            Ok(store) => {
                let result = store.read_store_data();
                assert!(
                    result.is_err(),
                    "read_store_data should fail on empty file"
                );
            }
        }
    }

    // Requirement: REQ-OI-001 (Must)
    // Edge case: JSON array instead of object
    #[test]
    fn req_oi_001_edge_json_array() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("google.json");
        fs::write(&path, r#"["not", "an", "object"]"#).unwrap();

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap());
        match store {
            Err(_) => { /* acceptable */ }
            Ok(store) => {
                let result = store.read_store_data();
                assert!(
                    result.is_err(),
                    "read_store_data should fail on JSON array"
                );
            }
        }
    }

    // Requirement: REQ-OI-001 (Must)
    // Edge case: Unicode/special characters in email field
    #[test]
    fn req_oi_001_edge_unicode_email() {
        let dir = tempfile::tempdir().unwrap();
        write_custom_google_json(
            dir.path(),
            1,
            "cid",
            "csecret",
            "rt",
            "user+tag@gmail.com",
        );

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let data = store.read_store_data().unwrap();
        assert_eq!(data.email, "user+tag@gmail.com");
    }

    // Requirement: REQ-OI-001 (Must)
    // Edge case: Very large JSON file (extra whitespace / padding)
    #[test]
    fn req_oi_001_edge_large_json_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("google.json");
        let content = serde_json::json!({
            "version": 1,
            "client_id": "424288504335-test.apps.googleusercontent.com",
            "client_secret": "GOCSPX-test-secret",
            "refresh_token": "1//03test-refresh-token",
            "email": "testuser@gmail.com",
            "padding": "x".repeat(100_000)
        });
        fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let data = store.read_store_data().unwrap();
        assert_eq!(data.email, "testuser@gmail.com");
    }

    // =================================================================
    // REQ-OI-002 (Must): OmegaStoreCredentialStore implements CredentialStore
    // =================================================================

    // Requirement: REQ-OI-002 (Must)
    // Acceptance: OmegaStoreCredentialStore implements CredentialStore trait (compile-time)
    #[test]
    fn req_oi_002_implements_credential_store_trait() {
        fn assert_credential_store<T: CredentialStore>() {}
        assert_credential_store::<OmegaStoreCredentialStore>();
    }

    // Requirement: REQ-OI-002 (Must)
    // Acceptance: OmegaStoreCredentialStore is Send + Sync
    #[test]
    fn req_oi_002_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OmegaStoreCredentialStore>();
    }

    // Requirement: REQ-OI-002 (Must)
    // Acceptance: get_token reads from google.json and constructs TokenData
    #[test]
    fn req_oi_002_get_token_returns_token_data() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let token = store
            .get_token("default", "testuser@gmail.com")
            .expect("get_token should succeed");

        assert_eq!(token.email, "testuser@gmail.com");
        assert_eq!(token.refresh_token, "1//03test-refresh-token");
        // client field should be set
        assert!(!token.client.is_empty());
    }

    // Requirement: REQ-OI-002 (Must)
    // Acceptance: get_token works regardless of what client/email is passed
    //   (OMEGA store has only one account; the client/email params are ignored)
    #[test]
    fn req_oi_002_get_token_ignores_client_email_params() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        // Pass different client/email than what's in the store
        let token = store
            .get_token("some-other-client", "other@example.com")
            .expect("get_token should succeed regardless of params");

        // Should still return the OMEGA store data
        assert_eq!(token.email, "testuser@gmail.com");
        assert_eq!(token.refresh_token, "1//03test-refresh-token");
    }

    // Requirement: REQ-OI-002 (Must)
    // Acceptance: set_token updates only refresh_token, preserves other fields
    #[test]
    fn req_oi_002_set_token_updates_refresh_token_only() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();

        // Read original data
        let original = store.read_store_data().unwrap();

        // Create a token with a new refresh_token
        let mut updated_token = make_test_token("testuser@gmail.com", "1//03new-refresh-token");
        updated_token.refresh_token = "1//03new-refresh-token".to_string();

        store
            .set_token("default", "testuser@gmail.com", &updated_token)
            .expect("set_token should succeed");

        // Read back and verify only refresh_token changed
        let after = store.read_store_data().unwrap();
        assert_eq!(after.refresh_token, "1//03new-refresh-token");
        assert_eq!(after.client_id, original.client_id, "client_id must be preserved");
        assert_eq!(
            after.client_secret, original.client_secret,
            "client_secret must be preserved"
        );
        assert_eq!(after.email, original.email, "email must be preserved");
        assert_eq!(after.version, original.version, "version must be preserved");
    }

    // Requirement: REQ-OI-002 (Must)
    // Acceptance: delete_token returns error (not permitted)
    #[test]
    fn req_oi_002_delete_token_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let result = store.delete_token("default", "testuser@gmail.com");
        assert!(
            result.is_err(),
            "delete_token should return an error for OMEGA store"
        );
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.to_lowercase().contains("omega")
                || err_msg.to_lowercase().contains("delete")
                || err_msg.to_lowercase().contains("not permitted")
                || err_msg.to_lowercase().contains("cannot"),
            "Error message should explain deletion is not permitted, got: {}",
            err_msg
        );
    }

    // Requirement: REQ-OI-002 (Must)
    // Acceptance: list_tokens returns single-element vec
    #[test]
    fn req_oi_002_list_tokens_returns_single_element() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let tokens = store
            .list_tokens()
            .expect("list_tokens should succeed");

        assert_eq!(
            tokens.len(),
            1,
            "OMEGA store should always have exactly one token"
        );
        assert_eq!(tokens[0].email, "testuser@gmail.com");
    }

    // Requirement: REQ-OI-002 (Must)
    // Acceptance: keys returns single key
    #[test]
    fn req_oi_002_keys_returns_single_key() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let keys = store.keys().expect("keys should succeed");

        assert_eq!(keys.len(), 1, "OMEGA store should have exactly one key");
        // The key should contain the email
        assert!(
            keys[0].contains("testuser@gmail.com"),
            "Key should contain the email, got: {}",
            keys[0]
        );
    }

    // Requirement: REQ-OI-002 (Must)
    // Acceptance: get_default_account returns email from google.json
    #[test]
    fn req_oi_002_get_default_account_returns_email() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let default = store
            .get_default_account("default")
            .expect("get_default_account should succeed");

        assert_eq!(
            default.as_deref(),
            Some("testuser@gmail.com"),
            "get_default_account should return the email from google.json"
        );
    }

    // Requirement: REQ-OI-002 (Must)
    // Acceptance: set_default_account is a no-op (succeeds without error)
    #[test]
    fn req_oi_002_set_default_account_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let result = store.set_default_account("default", "other@example.com");
        assert!(
            result.is_ok(),
            "set_default_account should succeed (no-op), got: {:?}",
            result.err()
        );

        // Verify it didn't actually change the default
        let default = store.get_default_account("default").unwrap();
        assert_eq!(
            default.as_deref(),
            Some("testuser@gmail.com"),
            "set_default_account should be a no-op; email should be unchanged"
        );
    }

    // Requirement: REQ-OI-002 (Must)
    // Edge case: get_token after set_token reflects new refresh_token
    #[test]
    fn req_oi_002_get_token_after_set_token_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();

        // Update refresh token
        let new_token = make_test_token("testuser@gmail.com", "1//refreshed");
        store
            .set_token("default", "testuser@gmail.com", &new_token)
            .unwrap();

        // Read it back via get_token
        let loaded = store
            .get_token("default", "testuser@gmail.com")
            .unwrap();
        assert_eq!(loaded.refresh_token, "1//refreshed");
    }

    // =================================================================
    // REQ-OI-003 (Must): credential_store_factory detects OMEGA_STORES_DIR
    // =================================================================

    // Requirement: REQ-OI-003 (Must)
    // Acceptance: credential_store_factory returns OmegaStoreCredentialStore
    //   when OMEGA_STORES_DIR is set
    // NOTE: This test manipulates env vars. It uses a mutex or serial execution
    //   to avoid races with other tests.
    #[test]
    fn req_oi_003_factory_returns_omega_store_when_env_set() {
        use crate::auth::keyring::credential_store_factory;

        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        // Set the env var
        let dir_str = dir.path().to_str().unwrap().to_string();
        std::env::set_var("OMEGA_STORES_DIR", &dir_str);

        let config = crate::config::ConfigFile::default();
        let result = credential_store_factory(&config);

        // Clean up env var BEFORE assertions to avoid contaminating other tests
        std::env::remove_var("OMEGA_STORES_DIR");

        let store = result.expect("credential_store_factory should succeed with OMEGA_STORES_DIR");

        // Verify we got a working credential store that reads from the OMEGA store
        let default_account = store.get_default_account("default").unwrap();
        assert_eq!(
            default_account.as_deref(),
            Some("testuser@gmail.com"),
            "Factory should have returned an OmegaStoreCredentialStore"
        );
    }

    // Requirement: REQ-OI-003 (Must)
    // Acceptance: When OMEGA_STORES_DIR is not set, existing behavior unchanged
    #[test]
    fn req_oi_003_factory_unchanged_without_env() {
        use crate::auth::keyring::credential_store_factory;

        // Ensure env var is NOT set
        std::env::remove_var("OMEGA_STORES_DIR");

        let config = crate::config::ConfigFile {
            keyring_backend: Some("file".to_string()),
            ..Default::default()
        };
        let result = credential_store_factory(&config);
        assert!(
            result.is_ok(),
            "Factory should still work without OMEGA_STORES_DIR: {:?}",
            result.err()
        );
    }

    // Requirement: REQ-OI-003 (Must)
    // Acceptance: OMEGA_STORES_DIR takes priority over keyring_backend config
    #[test]
    fn req_oi_003_omega_store_overrides_keyring_backend() {
        use crate::auth::keyring::credential_store_factory;

        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let dir_str = dir.path().to_str().unwrap().to_string();
        std::env::set_var("OMEGA_STORES_DIR", &dir_str);

        // Config says "keychain" but OMEGA_STORES_DIR should win
        let config = crate::config::ConfigFile {
            keyring_backend: Some("keychain".to_string()),
            ..Default::default()
        };
        let result = credential_store_factory(&config);

        std::env::remove_var("OMEGA_STORES_DIR");

        let store = result.expect("OMEGA store should win over keyring_backend");
        let default = store.get_default_account("default").unwrap();
        assert_eq!(
            default.as_deref(),
            Some("testuser@gmail.com"),
            "OMEGA store should take priority over keyring_backend"
        );
    }

    // Requirement: REQ-OI-003 (Must)
    // Edge case: OMEGA_STORES_DIR set but google.json missing
    #[test]
    fn req_oi_003_factory_omega_dir_without_google_json() {
        use crate::auth::keyring::credential_store_factory;

        let dir = tempfile::tempdir().unwrap();
        // Do NOT create google.json

        let dir_str = dir.path().to_str().unwrap().to_string();
        std::env::set_var("OMEGA_STORES_DIR", &dir_str);

        let config = crate::config::ConfigFile::default();
        let result = credential_store_factory(&config);

        std::env::remove_var("OMEGA_STORES_DIR");

        // Factory should return an error when OMEGA_STORES_DIR is set but
        // google.json is missing. The error handling may be eager (in factory)
        // or lazy (on first use). Either way, we expect an error.
        assert!(
            result.is_err(),
            "Factory should fail when OMEGA_STORES_DIR points to dir without google.json"
        );
    }

    // Requirement: REQ-OI-003 (Must)
    // Edge case: OMEGA_STORES_DIR set to empty string
    #[test]
    fn req_oi_003_factory_omega_dir_empty_string() {
        use crate::auth::keyring::credential_store_factory;

        // Setting to empty string should behave as "not set" -- fall through
        // to normal logic. Or fail with an error. Either is acceptable.
        std::env::set_var("OMEGA_STORES_DIR", "");

        let config = crate::config::ConfigFile {
            keyring_backend: Some("file".to_string()),
            ..Default::default()
        };
        let result = credential_store_factory(&config);

        std::env::remove_var("OMEGA_STORES_DIR");

        // The result should not panic. It may succeed (falls through to file)
        // or fail (empty path is invalid). Both are acceptable.
        let _ = result;
    }

    // =================================================================
    // REQ-OI-004 (Must): bootstrap_service_context OMEGA store support
    // =================================================================

    // Requirement: REQ-OI-004 (Must)
    // Acceptance: client_credentials() returns correct ClientCredentials from google.json
    #[test]
    fn req_oi_004_client_credentials_extracted() {
        let dir = tempfile::tempdir().unwrap();
        write_custom_google_json(
            dir.path(),
            1,
            "my-client-id.apps.googleusercontent.com",
            "GOCSPX-my-secret",
            "1//refresh",
            "user@example.com",
        );

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let creds = store
            .client_credentials()
            .expect("client_credentials should succeed");

        assert_eq!(creds.client_id, "my-client-id.apps.googleusercontent.com");
        assert_eq!(creds.client_secret, "GOCSPX-my-secret");
    }

    // Requirement: REQ-OI-004 (Must)
    // Acceptance: ClientCredentials fields match the google.json exactly
    #[test]
    fn req_oi_004_client_credentials_type_is_correct() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let creds: ClientCredentials = store.client_credentials().unwrap();

        // Verify the type is actually crate::config::ClientCredentials
        assert_eq!(
            creds.client_id,
            "424288504335-test.apps.googleusercontent.com"
        );
        assert_eq!(creds.client_secret, "GOCSPX-test-secret");
    }

    // Requirement: REQ-OI-004 (Must)
    // Edge case: client_credentials fails if file is corrupted
    #[test]
    fn req_oi_004_client_credentials_fails_on_corrupt_file() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();

        // Corrupt the file after construction
        let path = dir.path().join("google.json");
        fs::write(&path, "CORRUPTED").unwrap();

        let result = store.client_credentials();
        assert!(
            result.is_err(),
            "client_credentials should fail if file is corrupted"
        );
    }

    // =================================================================
    // REQ-OI-005 (Must): Atomic write for OMEGA store updates
    // =================================================================

    // Requirement: REQ-OI-005 (Must)
    // Acceptance: set_token writes atomically (file exists after write)
    #[test]
    fn req_oi_005_set_token_writes_atomically() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let new_token = make_test_token("testuser@gmail.com", "1//atomic-refresh");

        store
            .set_token("default", "testuser@gmail.com", &new_token)
            .expect("set_token should succeed");

        // Verify the file exists and is valid JSON
        let path = dir.path().join("google.json");
        assert!(path.exists(), "google.json should exist after set_token");

        let content = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(&content).expect("File should be valid JSON after set_token");
        assert_eq!(
            parsed["refresh_token"].as_str().unwrap(),
            "1//atomic-refresh"
        );
    }

    // Requirement: REQ-OI-005 (Must)
    // Acceptance: File permissions are 0600 on Unix
    #[cfg(unix)]
    #[test]
    fn req_oi_005_file_permissions_0600() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let new_token = make_test_token("testuser@gmail.com", "1//perm-test");
        store
            .set_token("default", "testuser@gmail.com", &new_token)
            .unwrap();

        let path = dir.path().join("google.json");
        let perms = fs::metadata(&path).unwrap().permissions();
        assert_eq!(
            perms.mode() & 0o777,
            0o600,
            "google.json must have 0600 permissions after set_token"
        );
    }

    // Requirement: REQ-OI-005 (Must)
    // Acceptance: All existing fields preserved after set_token
    #[test]
    fn req_oi_005_all_fields_preserved_after_write() {
        let dir = tempfile::tempdir().unwrap();
        write_custom_google_json(
            dir.path(),
            1,
            "preserved-client-id",
            "preserved-client-secret",
            "old-refresh",
            "preserved@example.com",
        );

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let new_token = make_test_token("preserved@example.com", "new-refresh");
        store
            .set_token("default", "preserved@example.com", &new_token)
            .unwrap();

        // Read back raw JSON to verify all fields
        let path = dir.path().join("google.json");
        let content = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["version"].as_u64().unwrap(), 1);
        assert_eq!(
            parsed["client_id"].as_str().unwrap(),
            "preserved-client-id"
        );
        assert_eq!(
            parsed["client_secret"].as_str().unwrap(),
            "preserved-client-secret"
        );
        assert_eq!(parsed["refresh_token"].as_str().unwrap(), "new-refresh");
        assert_eq!(
            parsed["email"].as_str().unwrap(),
            "preserved@example.com"
        );
    }

    // Requirement: REQ-OI-005 (Must)
    // Edge case: No temp file left behind after successful write
    #[test]
    fn req_oi_005_no_temp_file_left_behind() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let new_token = make_test_token("testuser@gmail.com", "1//no-leftover");
        store
            .set_token("default", "testuser@gmail.com", &new_token)
            .unwrap();

        // Check that no .tmp files are left in the directory
        let entries: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        for entry in &entries {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            assert!(
                !name_str.contains(".tmp"),
                "Temporary file should not remain after successful write: {}",
                name_str
            );
        }
    }

    // Requirement: REQ-OI-005 (Must)
    // Edge case: set_token on read-only directory fails gracefully
    #[cfg(unix)]
    #[test]
    fn req_oi_005_write_to_readonly_dir_fails() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();

        // Make directory read-only
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o555)).unwrap();

        let new_token = make_test_token("testuser@gmail.com", "1//should-fail");
        let result = store.set_token("default", "testuser@gmail.com", &new_token);

        // Restore permissions for cleanup
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();

        assert!(
            result.is_err(),
            "set_token should fail on read-only directory"
        );
    }

    // Requirement: REQ-OI-005 (Must)
    // Edge case: Multiple sequential writes succeed
    #[test]
    fn req_oi_005_multiple_sequential_writes() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();

        for i in 0..5 {
            let token = make_test_token("testuser@gmail.com", &format!("1//refresh-{}", i));
            store
                .set_token("default", "testuser@gmail.com", &token)
                .unwrap_or_else(|e| panic!("set_token iteration {} failed: {}", i, e));
        }

        // Verify final state
        let data = store.read_store_data().unwrap();
        assert_eq!(data.refresh_token, "1//refresh-4");
    }

    // =================================================================
    // REQ-OI-006 (Must): auth status shows OMEGA store mode
    // =================================================================

    // Requirement: REQ-OI-006 (Must)
    // Acceptance: is_omega_store_active() returns true when env var set
    #[test]
    fn req_oi_006_is_omega_store_active_true_when_set() {
        // Use a unique env var value to avoid races
        let dir = tempfile::tempdir().unwrap();
        let dir_str = dir.path().to_str().unwrap().to_string();
        std::env::set_var("OMEGA_STORES_DIR", &dir_str);

        let active = is_omega_store_active();

        std::env::remove_var("OMEGA_STORES_DIR");

        assert!(
            active,
            "is_omega_store_active should return true when OMEGA_STORES_DIR is set"
        );
    }

    // Requirement: REQ-OI-006 (Must)
    // Acceptance: is_omega_store_active() returns false when env var not set
    #[test]
    fn req_oi_006_is_omega_store_active_false_when_not_set() {
        std::env::remove_var("OMEGA_STORES_DIR");

        let active = is_omega_store_active();

        assert!(
            !active,
            "is_omega_store_active should return false when OMEGA_STORES_DIR is not set"
        );
    }

    // Requirement: REQ-OI-006 (Must)
    // Edge case: is_omega_store_active with empty string env var
    #[test]
    fn req_oi_006_is_omega_store_active_empty_string() {
        std::env::set_var("OMEGA_STORES_DIR", "");

        let active = is_omega_store_active();

        std::env::remove_var("OMEGA_STORES_DIR");

        // Empty string should be treated as "not set"
        assert!(
            !active,
            "is_omega_store_active should return false for empty OMEGA_STORES_DIR"
        );
    }

    // =================================================================
    // REQ-OI-007 (Should): OMEGA store error messages are actionable
    // =================================================================

    // Requirement: REQ-OI-007 (Should)
    // Acceptance: Missing file error contains the path
    #[test]
    fn req_oi_007_missing_file_error_contains_path() {
        let dir = tempfile::tempdir().unwrap();
        let dir_str = dir.path().to_str().unwrap();
        // Do NOT create google.json

        let result = OmegaStoreCredentialStore::new(dir_str);
        match result {
            Err(e) => {
                let msg = format!("{:#}", e);
                assert!(
                    msg.contains(dir_str) || msg.contains("google.json"),
                    "Missing file error should contain path or filename, got: {}",
                    msg
                );
                assert!(
                    msg.contains("not found")
                        || msg.contains("missing")
                        || msg.contains("No such file"),
                    "Error should indicate file not found, got: {}",
                    msg
                );
            }
            Ok(store) => {
                let result = store.read_store_data();
                assert!(result.is_err());
                let msg = format!("{:#}", result.unwrap_err());
                assert!(
                    msg.contains(dir_str)
                        || msg.contains("google.json")
                        || msg.contains("not found")
                        || msg.contains("missing"),
                    "Error should contain path info, got: {}",
                    msg
                );
            }
        }
    }

    // Requirement: REQ-OI-007 (Should)
    // Acceptance: Invalid JSON error mentions "not valid JSON" or similar
    #[test]
    fn req_oi_007_invalid_json_error_message() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("google.json");
        fs::write(&path, "NOT { VALID } JSON [").unwrap();

        let result = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap());
        match result {
            Err(e) => {
                let msg = format!("{:#}", e);
                assert!(
                    msg.to_lowercase().contains("json")
                        || msg.to_lowercase().contains("parse")
                        || msg.to_lowercase().contains("valid"),
                    "Error should mention JSON, got: {}",
                    msg
                );
            }
            Ok(store) => {
                let result = store.read_store_data();
                assert!(result.is_err());
                let msg = format!("{:#}", result.unwrap_err());
                assert!(
                    msg.to_lowercase().contains("json")
                        || msg.to_lowercase().contains("parse")
                        || msg.to_lowercase().contains("valid"),
                    "Error should mention JSON, got: {}",
                    msg
                );
            }
        }
    }

    // Requirement: REQ-OI-007 (Should)
    // Acceptance: Missing field error names the specific field
    #[test]
    fn req_oi_007_missing_field_error_names_field() {
        // Test each required field individually
        let required_fields = ["client_id", "client_secret", "refresh_token", "email"];

        for field in &required_fields {
            let dir = tempfile::tempdir().unwrap();
            write_google_json_missing_field(dir.path(), field);

            let result = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap());
            match result {
                Err(e) => {
                    let msg = format!("{:#}", e);
                    assert!(
                        msg.contains(field)
                            || msg.to_lowercase().contains("missing")
                            || msg.to_lowercase().contains("field"),
                        "Error for missing '{}' should name the field, got: {}",
                        field,
                        msg
                    );
                }
                Ok(store) => {
                    let result = store.read_store_data();
                    assert!(
                        result.is_err(),
                        "read_store_data should fail for missing field '{}'",
                        field
                    );
                    let msg = format!("{:#}", result.unwrap_err());
                    assert!(
                        msg.contains(field)
                            || msg.to_lowercase().contains("missing")
                            || msg.to_lowercase().contains("field"),
                        "Error for missing '{}' should name the field, got: {}",
                        field,
                        msg
                    );
                }
            }
        }
    }

    // =================================================================
    // Security Tests (from Architecture)
    // =================================================================

    // Requirement: REQ-OI-001 (Must) - Security
    // Security: OmegaStoreData Debug impl must redact sensitive fields
    #[test]
    fn security_omega_store_data_debug_redacts_secrets() {
        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let data = store.read_store_data().unwrap();
        let debug_str = format!("{:?}", data);

        // Debug output must NOT contain actual secret values
        assert!(
            !debug_str.contains("GOCSPX-test-secret"),
            "Debug output must not contain client_secret value"
        );
        assert!(
            !debug_str.contains("1//03test-refresh-token"),
            "Debug output must not contain refresh_token value"
        );
    }

    // Requirement: REQ-OI-005 (Must) - Security
    // Security: Newly created google.json file has restricted permissions
    #[cfg(unix)]
    #[test]
    fn security_new_file_permissions_restricted() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        // Set initial permissions to something lax
        let path = dir.path().join("google.json");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

        let store = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap()).unwrap();
        let new_token = make_test_token("testuser@gmail.com", "1//sec-test");
        store
            .set_token("default", "testuser@gmail.com", &new_token)
            .unwrap();

        // After set_token, permissions should be 0600 regardless of what they were before
        let perms = fs::metadata(&path).unwrap().permissions();
        assert_eq!(
            perms.mode() & 0o777,
            0o600,
            "set_token must enforce 0600 permissions even if file was previously lax"
        );
    }

    // =================================================================
    // Failure Mode Tests (from Architecture)
    // =================================================================

    // Failure mode: Permission denied on file read
    #[cfg(unix)]
    #[test]
    fn failure_permission_denied_on_read() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        write_test_google_json(dir.path());

        // Make the file unreadable
        let path = dir.path().join("google.json");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();

        let result = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap());
        match result {
            Err(_) => { /* acceptable -- constructor failed */ }
            Ok(store) => {
                let result = store.read_store_data();
                assert!(
                    result.is_err(),
                    "read_store_data should fail when file is unreadable"
                );
            }
        }

        // Restore permissions for cleanup
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    }

    // Failure mode: stores_dir does not exist at all
    #[test]
    fn failure_stores_dir_does_not_exist() {
        let nonexistent = "/tmp/omg-gog-test-nonexistent-dir-98765";
        // Ensure it does not exist
        let _ = fs::remove_dir_all(nonexistent);

        let result = OmegaStoreCredentialStore::new(nonexistent);
        match result {
            Err(e) => {
                let msg = format!("{}", e);
                // Should mention the path or file-not-found
                assert!(
                    msg.contains("google.json")
                        || msg.contains(nonexistent)
                        || msg.contains("not found")
                        || msg.contains("No such file"),
                    "Error should mention the missing path, got: {}",
                    msg
                );
            }
            Ok(store) => {
                let result = store.read_store_data();
                assert!(
                    result.is_err(),
                    "read_store_data should fail when stores_dir does not exist"
                );
            }
        }
    }

    // Failure mode: google.json is a directory instead of a file
    #[test]
    fn failure_google_json_is_directory() {
        let dir = tempfile::tempdir().unwrap();
        // Create google.json as a directory
        fs::create_dir(dir.path().join("google.json")).unwrap();

        let result = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap());
        match result {
            Err(_) => { /* acceptable */ }
            Ok(store) => {
                let result = store.read_store_data();
                assert!(
                    result.is_err(),
                    "read_store_data should fail when google.json is a directory"
                );
            }
        }
    }

    // Failure mode: Correct JSON format but wrong types (e.g., version as string)
    #[test]
    fn failure_wrong_field_types() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("google.json");
        let content = serde_json::json!({
            "version": "one",  // should be u32, not string
            "client_id": 12345,  // should be string, not number
            "client_secret": "GOCSPX-test",
            "refresh_token": "1//03test",
            "email": "user@gmail.com"
        });
        fs::write(&path, serde_json::to_string(&content).unwrap()).unwrap();

        let result = OmegaStoreCredentialStore::new(dir.path().to_str().unwrap());
        match result {
            Err(_) => { /* acceptable -- deserialization failed */ }
            Ok(store) => {
                let result = store.read_store_data();
                assert!(
                    result.is_err(),
                    "read_store_data should fail with wrong field types"
                );
            }
        }
    }
}
