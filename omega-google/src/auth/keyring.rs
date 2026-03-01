// Keyring abstraction (OS + file fallback)
//
// This module provides a file-based credential store implementation.
// The OS keyring integration (via the `keyring` crate) is stubbed
// because it requires actual OS keychain access. The file-based
// fallback provides a working implementation for all environments.

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
}
