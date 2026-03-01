// Platform-specific config/keyring path resolution
//
// The primary path resolution logic lives in config/mod.rs:
//   - config_dir()
//   - config_path()
//   - service_account_path()
//
// This module re-exports those functions for architectural clarity.

pub use super::config_dir;
pub use super::config_path;
pub use super::service_account_path;
