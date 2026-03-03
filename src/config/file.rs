// Config file I/O operations (atomic write, JSON5 read)
//
// The primary config file I/O logic lives in config/mod.rs:
//   - read_config() / read_config_from()
//   - write_config() / write_config_to()
//   - ensure_dir()
//
// This module re-exports those functions for architectural clarity.

pub use super::ensure_dir;
pub use super::read_config;
pub use super::read_config_from;
pub use super::write_config;
pub use super::write_config_to;
