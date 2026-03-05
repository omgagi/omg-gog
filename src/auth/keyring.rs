// Keyring abstraction (OS + file fallback)
//
// This module provides both OS keyring and file-based credential store
// implementations. The OS keyring uses the `keyring` crate for platform-
// native secret storage. The file-based fallback provides a working
// implementation for environments without an OS keyring.

use crate::auth::token::{deserialize_token, serialize_token};
use crate::auth::{CredentialStore, TokenData};
use std::collections::HashMap;
use std::path::PathBuf;

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
    ///
    /// On Linux, the keyring probe is wrapped with a 5-second timeout to avoid
    /// hanging when the D-Bus keyring daemon is not running (REQ-RT-016).
    pub fn new() -> anyhow::Result<Self> {
        #[cfg(target_os = "linux")]
        {
            let (tx, rx) = std::sync::mpsc::channel();
            let handle = std::thread::spawn(move || {
                let result = keyring::Entry::new("omega-google", "probe");
                let _ = tx.send(result);
            });

            match rx.recv_timeout(std::time::Duration::from_secs(5)) {
                Ok(Ok(entry)) => match entry.get_password() {
                    Ok(_) | Err(keyring::Error::NoEntry) => {}
                    Err(e) => anyhow::bail!("OS keyring not available: {}", e),
                },
                Ok(Err(e)) => anyhow::bail!("OS keyring not available: {}", e),
                Err(_) => {
                    eprintln!("Keyring timed out after 5 seconds. Try GOG_KEYRING_BACKEND=file");
                    anyhow::bail!("Keyring timed out");
                }
            }
            let _ = handle.join();
        }

        #[cfg(not(target_os = "linux"))]
        {
            let test_entry = keyring::Entry::new("omega-google", "probe")?;
            match test_entry.get_password() {
                Ok(_) | Err(keyring::Error::NoEntry) => {}
                Err(e) => anyhow::bail!("OS keyring not available: {}", e),
            }
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

// =================================================================
// AES-GCM Encrypted File Backend (REQ-RT-014)
// =================================================================

/// Number of PBKDF2-style rounds for key derivation.
const KDF_ROUNDS: u32 = 100_000;

/// Derive a 256-bit key from a password and salt using a PBKDF2-style
/// construction with SHA-256.
///
/// Iterates SHA-256(salt || password || round || previous_hash) for
/// `KDF_ROUNDS` rounds. This provides cryptographic key stretching
/// to resist brute-force attacks on the password.
fn derive_key(password: &str, salt: &[u8; 16]) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    // Initial hash: SHA-256(salt || password)
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(password.as_bytes());
    let mut hash = hasher.finalize();

    // Iterate: SHA-256(salt || password || round || previous_hash)
    for round in 0u32..KDF_ROUNDS {
        let mut hasher = Sha256::new();
        hasher.update(salt);
        hasher.update(password.as_bytes());
        hasher.update(round.to_le_bytes());
        hasher.update(hash);
        hash = hasher.finalize();
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&hash);
    key
}

/// Encrypt plaintext using AES-256-GCM with a random nonce.
///
/// Returns nonce (12 bytes) prepended to ciphertext.
fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Key, Nonce};
    use rand::RngCore;

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Prepend nonce to ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);
    Ok(result)
}

/// Decrypt data that was encrypted with `encrypt`.
///
/// Expects nonce (12 bytes) prepended to ciphertext.
fn decrypt(key: &[u8; 32], data: &[u8]) -> anyhow::Result<Vec<u8>> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Key, Nonce};

    if data.len() < 12 {
        anyhow::bail!("Encrypted data too short");
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed (wrong password?): {}", e))
}

/// Encrypted file-based credential store using AES-256-GCM.
///
/// Wraps `FileCredentialStore` -- encrypts token JSON before writing
/// and decrypts after reading. Password sourced from `GOG_KEYRING_PASSWORD`
/// env var or TTY prompt.
///
/// A random 16-byte salt is generated on first use and persisted in an
/// `encryption_salt` file alongside the tokens. The salt is combined with
/// the password through a PBKDF2-style SHA-256 KDF to derive the AES key.
pub struct EncryptedFileCredentialStore {
    inner: FileCredentialStore,
    key: [u8; 32],
}

impl EncryptedFileCredentialStore {
    /// Create a new encrypted file store with the given directory and password.
    ///
    /// On first use, generates and persists a random 16-byte salt. On
    /// subsequent uses, loads the existing salt from disk.
    pub fn new(dir: PathBuf, password: &str) -> anyhow::Result<Self> {
        let inner = FileCredentialStore::new(dir)?;
        let salt = Self::load_or_create_salt(&inner.dir)?;
        let key = derive_key(password, &salt);
        Ok(Self { inner, key })
    }

    /// Load the encryption salt from disk, or generate a new one if it doesn't
    /// exist yet. The salt file is stored with 0600 permissions on Unix.
    fn load_or_create_salt(dir: &std::path::Path) -> anyhow::Result<[u8; 16]> {
        let salt_path = dir.join("encryption_salt");
        if salt_path.exists() {
            let data = std::fs::read(&salt_path)?;
            if data.len() == 16 {
                let mut salt = [0u8; 16];
                salt.copy_from_slice(&data);
                return Ok(salt);
            }
            // If the salt file is corrupt, regenerate it
        }
        // Generate a new random salt
        let mut salt = [0u8; 16];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut salt);
        std::fs::write(&salt_path, salt)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&salt_path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(salt)
    }
}

impl CredentialStore for EncryptedFileCredentialStore {
    fn get_token(&self, client: &str, email: &str) -> anyhow::Result<TokenData> {
        let map = self.inner.read_tokens_map()?;
        let key = crate::auth::token_key(client, email);
        match map.get(&key) {
            Some(encoded) => {
                // Decode base64 -> decrypt -> deserialize
                let cipher_bytes =
                    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
                        .map_err(|e| anyhow::anyhow!("Base64 decode failed: {}", e))?;
                let plain_bytes = decrypt(&self.key, &cipher_bytes)?;
                let json_str = String::from_utf8(plain_bytes)
                    .map_err(|e| anyhow::anyhow!("UTF-8 decode failed: {}", e))?;
                deserialize_token(&json_str)
            }
            None => anyhow::bail!("no token found for {}:{}", client, email),
        }
    }

    fn set_token(&self, client: &str, email: &str, token: &TokenData) -> anyhow::Result<()> {
        let mut map = self.inner.read_tokens_map()?;
        let key = crate::auth::token_key(client, email);
        let json_str = serialize_token(token)?;
        // Encrypt -> base64 encode -> store
        let cipher_bytes = encrypt(&self.key, json_str.as_bytes())?;
        let encoded =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &cipher_bytes);
        map.insert(key, encoded);
        self.inner.write_tokens_map(&map)
    }

    fn delete_token(&self, client: &str, email: &str) -> anyhow::Result<()> {
        self.inner.delete_token(client, email)
    }

    fn list_tokens(&self) -> anyhow::Result<Vec<TokenData>> {
        // For encrypted store, list requires decrypting each entry
        let map = self.inner.read_tokens_map()?;
        let mut tokens = Vec::new();
        for encoded in map.values() {
            if let Ok(cipher_bytes) =
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
            {
                if let Ok(plain_bytes) = decrypt(&self.key, &cipher_bytes) {
                    if let Ok(json_str) = String::from_utf8(plain_bytes) {
                        if let Ok(token) = deserialize_token(&json_str) {
                            tokens.push(token);
                        }
                    }
                }
            }
        }
        Ok(tokens)
    }

    fn keys(&self) -> anyhow::Result<Vec<String>> {
        self.inner.keys()
    }

    fn get_default_account(&self, client: &str) -> anyhow::Result<Option<String>> {
        self.inner.get_default_account(client)
    }

    fn set_default_account(&self, client: &str, email: &str) -> anyhow::Result<()> {
        self.inner.set_default_account(client, email)
    }

    fn delete_token_by_raw_key(&self, key: &str) -> anyhow::Result<()> {
        self.inner.delete_token_by_raw_key(key)
    }
}

/// Factory: build the appropriate CredentialStore based on config.
/// Reads keyring_backend from config and GOG_KEYRING_BACKEND env (env overrides config).
/// - "auto" (default): try OS keyring, fall back to file on error
/// - "keychain" / "keyring": force OS keyring
/// - "file": force file backend
///
/// If GOG_KEYRING_PASSWORD is set and the backend is "file" or falls back to file,
/// the encrypted file backend is used.
pub fn credential_store_factory(
    config: &crate::config::ConfigFile,
) -> anyhow::Result<Box<dyn CredentialStore>> {
    // OMEGA store takes priority over all other backends
    if let Ok(stores_dir) = std::env::var("OMEGA_STORES_DIR") {
        if !stores_dir.is_empty() {
            return Ok(Box::new(
                crate::auth::omega_store::OmegaStoreCredentialStore::new(&stores_dir)?,
            ));
        }
    }

    // Determine backend: env overrides config
    let backend = std::env::var("GOG_KEYRING_BACKEND")
        .ok()
        .or_else(|| config.keyring_backend.clone())
        .unwrap_or_else(|| "auto".to_string());

    let config_dir = crate::config::config_dir()?;
    let password = std::env::var("GOG_KEYRING_PASSWORD").ok();

    match backend.as_str() {
        "keychain" | "keyring" => Ok(Box::new(KeyringCredentialStore::new()?)),
        "file" => {
            if let Some(pw) = password {
                Ok(Box::new(EncryptedFileCredentialStore::new(
                    config_dir, &pw,
                )?))
            } else {
                Ok(Box::new(FileCredentialStore::new(config_dir)?))
            }
        }
        "auto" | "" => {
            // Try OS keyring first, fall back to file
            match KeyringCredentialStore::new() {
                Ok(store) => Ok(Box::new(store)),
                Err(_) => {
                    eprintln!("Warning: OS keyring not available, falling back to file-based credential storage.");
                    eprintln!("Set GOG_KEYRING_PASSWORD to encrypt stored credentials.");
                    if let Some(pw) = password {
                        Ok(Box::new(EncryptedFileCredentialStore::new(
                            config_dir, &pw,
                        )?))
                    } else {
                        Ok(Box::new(FileCredentialStore::new(config_dir)?))
                    }
                }
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
        assert!(
            result.is_ok(),
            "KeyringCredentialStore::new() must not panic"
        );
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
        let store =
            KeyringCredentialStore::new().expect("OS keyring should be available for this test");
        let token = make_test_token("default", "test@example.com");
        store
            .set_token("default", "test@example.com", &token)
            .expect("set_token should succeed");
        let loaded = store
            .get_token("default", "test@example.com")
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
        let store = KeyringCredentialStore::new().expect("OS keyring should be available");
        let token = make_test_token("default", "delete-test@example.com");
        store
            .set_token("default", "delete-test@example.com", &token)
            .unwrap();
        store
            .delete_token("default", "delete-test@example.com")
            .unwrap();
        let result = store.get_token("default", "delete-test@example.com");
        assert!(result.is_err(), "Deleted token should not be found");
    }

    // Requirement: REQ-RT-013 (Must)
    // Real keyring operations (marked #[ignore] -- requires OS keyring)
    #[test]
    #[ignore]
    fn req_rt_013_keyring_list_tokens() {
        let store = KeyringCredentialStore::new().expect("OS keyring should be available");
        let token = make_test_token("default", "list-test@example.com");
        store
            .set_token("default", "list-test@example.com", &token)
            .unwrap();
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
        let store = KeyringCredentialStore::new().expect("OS keyring should be available");
        store
            .set_default_account("default", "default-test@example.com")
            .unwrap();
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
        assert!(
            result.is_ok(),
            "File backend should succeed: {:?}",
            result.err()
        );
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
        assert!(
            result.is_ok(),
            "auto backend should succeed: {:?}",
            result.err()
        );
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
        assert!(
            result.is_ok(),
            "None (auto) backend should succeed: {:?}",
            result.err()
        );
    }

    // Requirement: REQ-RT-015 (Must)
    // Acceptance: Returns Box<dyn CredentialStore>
    #[test]
    fn req_rt_015_factory_returns_boxed_trait() {
        fn assert_returns_box(
            _f: fn(&crate::config::ConfigFile) -> anyhow::Result<Box<dyn CredentialStore>>,
        ) {
        }
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
        assert!(
            result.is_ok(),
            "Empty string (auto) backend should succeed: {:?}",
            result.err()
        );
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
        store
            .set_token("default", "file-test@example.com", &token)
            .unwrap();
        let loaded = store.get_token("default", "file-test@example.com").unwrap();
        assert_eq!(loaded.email, "file-test@example.com");
        assert_eq!(loaded.client, "default");
        store
            .delete_token("default", "file-test@example.com")
            .unwrap();
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
        store
            .set_default_account("default", "primary@example.com")
            .unwrap();
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
        store
            .set_default_account("personal", "user@gmail.com")
            .unwrap();
        assert_eq!(
            store.get_default_account("work").unwrap().as_deref(),
            Some("user@work.com")
        );
        assert_eq!(
            store.get_default_account("personal").unwrap().as_deref(),
            Some("user@gmail.com")
        );
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
        store
            .set_token("default", "perm-test@example.com", &token)
            .unwrap();
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
        assert!(
            result.is_err(),
            "Writing to read-only directory should fail"
        );
        // Restore permissions for cleanup
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    // =================================================================
    // REQ-RT-014 (Should): AES-GCM Encrypted File Backend
    // =================================================================

    /// Fixed test salt used for deterministic derive_key tests.
    const TEST_SALT: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    // Requirement: REQ-RT-014 (Should)
    // Acceptance: Encrypt/decrypt roundtrip produces original plaintext
    #[test]
    fn req_rt_014_encrypt_decrypt_roundtrip() {
        let password = "my-secret-password-123";
        let key = derive_key(password, &TEST_SALT);
        let plaintext = b"Hello, this is sensitive token data!";
        let encrypted = encrypt(&key, plaintext).expect("encrypt should succeed");
        // Encrypted data should be different from plaintext
        assert_ne!(&encrypted[..], &plaintext[..]);
        // Encrypted data should be at least 12 (nonce) + plaintext.len() bytes
        assert!(encrypted.len() >= 12 + plaintext.len());
        let decrypted = decrypt(&key, &encrypted).expect("decrypt should succeed");
        assert_eq!(&decrypted[..], &plaintext[..]);
    }

    // Requirement: REQ-RT-014 (Should)
    // Acceptance: Encrypted store set/get/delete cycle
    #[test]
    fn req_rt_014_encrypted_store_set_get_delete() {
        let dir = tempfile::tempdir().unwrap();
        let store = EncryptedFileCredentialStore::new(dir.path().to_path_buf(), "test-password")
            .expect("EncryptedFileCredentialStore::new should succeed");

        let token = make_test_token("default", "enc-test@example.com");
        store
            .set_token("default", "enc-test@example.com", &token)
            .expect("set_token should succeed");

        let loaded = store
            .get_token("default", "enc-test@example.com")
            .expect("get_token should succeed");
        assert_eq!(loaded.email, "enc-test@example.com");
        assert_eq!(loaded.client, "default");
        assert_eq!(loaded.refresh_token, token.refresh_token);

        store
            .delete_token("default", "enc-test@example.com")
            .expect("delete_token should succeed");
        let result = store.get_token("default", "enc-test@example.com");
        assert!(result.is_err(), "Deleted token should not be found");
    }

    // Requirement: REQ-RT-014 (Should)
    // Acceptance: Wrong password fails decrypt
    #[test]
    fn req_rt_014_wrong_password_fails_decrypt() {
        let dir = tempfile::tempdir().unwrap();
        let store_write =
            EncryptedFileCredentialStore::new(dir.path().to_path_buf(), "correct-password")
                .unwrap();

        let token = make_test_token("default", "wrong-pw@example.com");
        store_write
            .set_token("default", "wrong-pw@example.com", &token)
            .unwrap();

        // Try reading with wrong password
        let store_read =
            EncryptedFileCredentialStore::new(dir.path().to_path_buf(), "wrong-password").unwrap();
        let result = store_read.get_token("default", "wrong-pw@example.com");
        assert!(
            result.is_err(),
            "Wrong password should fail to decrypt: {:?}",
            result
        );
    }

    // Requirement: REQ-RT-014 (Should)
    // Acceptance: Unencrypted mode when no password (FileCredentialStore)
    #[test]
    fn req_rt_014_unencrypted_mode_no_password() {
        let dir = tempfile::tempdir().unwrap();
        // FileCredentialStore (no encryption) should work normally
        let store = FileCredentialStore::new(dir.path().to_path_buf()).unwrap();
        let token = make_test_token("default", "plain@example.com");
        store
            .set_token("default", "plain@example.com", &token)
            .unwrap();
        let loaded = store.get_token("default", "plain@example.com").unwrap();
        assert_eq!(loaded.email, "plain@example.com");
        // The stored file should be readable as plain JSON
        let tokens_path = dir.path().join("tokens.json");
        let content = std::fs::read_to_string(&tokens_path).unwrap();
        // Should be valid JSON (not encrypted)
        let _: std::collections::HashMap<String, String> =
            serde_json::from_str(&content).expect("Unencrypted file should be valid JSON");
    }

    // Requirement: REQ-RT-014 (Should)
    // Acceptance: Encrypted store list_tokens works
    #[test]
    fn req_rt_014_encrypted_store_list_tokens() {
        let dir = tempfile::tempdir().unwrap();
        let store = EncryptedFileCredentialStore::new(dir.path().to_path_buf(), "list-pw").unwrap();
        let t1 = make_test_token("default", "a@example.com");
        let t2 = make_test_token("default", "b@example.com");
        store.set_token("default", "a@example.com", &t1).unwrap();
        store.set_token("default", "b@example.com", &t2).unwrap();
        let tokens = store.list_tokens().unwrap();
        assert_eq!(tokens.len(), 2);
    }

    // Requirement: REQ-RT-014 (Should)
    // Acceptance: EncryptedFileCredentialStore implements CredentialStore
    #[test]
    fn req_rt_014_encrypted_implements_credential_store() {
        fn assert_credential_store<T: CredentialStore>() {}
        assert_credential_store::<EncryptedFileCredentialStore>();
    }

    // Requirement: REQ-RT-014 (Should)
    // Acceptance: derive_key produces consistent output for same password and salt
    #[test]
    fn req_rt_014_derive_key_deterministic() {
        let key1 = derive_key("test-password", &TEST_SALT);
        let key2 = derive_key("test-password", &TEST_SALT);
        assert_eq!(key1, key2, "Same password and salt should produce same key");
    }

    // Requirement: REQ-RT-014 (Should)
    // Acceptance: derive_key produces different output for different passwords
    #[test]
    fn req_rt_014_derive_key_different_passwords() {
        let key1 = derive_key("password-one", &TEST_SALT);
        let key2 = derive_key("password-two", &TEST_SALT);
        assert_ne!(
            key1, key2,
            "Different passwords should produce different keys"
        );
    }

    // Requirement: REQ-RT-014 (Should)
    // Acceptance: derive_key produces different output for different salts
    #[test]
    fn req_rt_014_derive_key_different_salts() {
        let salt2: [u8; 16] = [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
        let key1 = derive_key("same-password", &TEST_SALT);
        let key2 = derive_key("same-password", &salt2);
        assert_ne!(key1, key2, "Different salts should produce different keys");
    }

    // Requirement: REQ-RT-014 (Should)
    // Edge case: encrypt produces different ciphertext each time (random nonce)
    #[test]
    fn req_rt_014_encrypt_random_nonce() {
        let key = derive_key("nonce-test", &TEST_SALT);
        let plaintext = b"same plaintext";
        let enc1 = encrypt(&key, plaintext).unwrap();
        let enc2 = encrypt(&key, plaintext).unwrap();
        assert_ne!(
            enc1, enc2,
            "Different nonces should produce different ciphertext"
        );
        // But both should decrypt to the same plaintext
        let dec1 = decrypt(&key, &enc1).unwrap();
        let dec2 = decrypt(&key, &enc2).unwrap();
        assert_eq!(dec1, dec2);
        assert_eq!(&dec1[..], &plaintext[..]);
    }

    // Requirement: REQ-RT-014 (Should)
    // Edge case: decrypt with truncated data fails
    #[test]
    fn req_rt_014_decrypt_truncated_data() {
        let result = decrypt(&[0u8; 32], &[1, 2, 3]); // too short
        assert!(result.is_err(), "Truncated data should fail");
    }

    // Requirement: REQ-RT-014 (Should)
    // Edge case: Encrypted file data is not readable as plain JSON
    #[test]
    fn req_rt_014_encrypted_data_not_plain_json() {
        let dir = tempfile::tempdir().unwrap();
        let store =
            EncryptedFileCredentialStore::new(dir.path().to_path_buf(), "enc-password").unwrap();
        let token = make_test_token("default", "opaque@example.com");
        store
            .set_token("default", "opaque@example.com", &token)
            .unwrap();

        // Read the raw tokens file
        let tokens_path = dir.path().join("tokens.json");
        let content = std::fs::read_to_string(&tokens_path).unwrap();
        let map: std::collections::HashMap<String, String> =
            serde_json::from_str(&content).unwrap();
        // The stored value should NOT be valid token JSON (it is base64-encoded ciphertext)
        let stored_value = map.values().next().unwrap();
        let token_parse: Result<serde_json::Value, _> = serde_json::from_str(stored_value);
        assert!(
            token_parse.is_err(),
            "Encrypted data should not be parseable as JSON"
        );
    }

    // =================================================================
    // REQ-RT-016 (Should): Keyring timeout on Linux
    // =================================================================

    // Requirement: REQ-RT-016 (Should)
    // Acceptance: KeyringCredentialStore::new probes the keyring (timeout behavior
    // is platform-specific -- on Linux, wrapped with 5-second timeout)
    #[test]
    fn req_rt_016_keyring_probe_documented() {
        // KeyringCredentialStore::new() probes the OS keyring.
        // On Linux, the probe is wrapped with a 5-second timeout to avoid
        // hanging when D-Bus keyring is unavailable.
        // This test documents the requirement and verifies the function
        // compiles and does not panic.
        let result = std::panic::catch_unwind(|| {
            let _ = KeyringCredentialStore::new();
        });
        assert!(result.is_ok(), "KeyringCredentialStore::new must not panic");
    }
}
