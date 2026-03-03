// Account resolution logic (flag > env > default)
//
// The primary account resolution logic lives in auth/mod.rs:
//   - resolve_account()
//   - parse_token_key()
//   - token_key()
//
// This module re-exports those functions for architectural clarity.

pub use super::legacy_token_key;
pub use super::parse_token_key;
pub use super::resolve_account;
pub use super::token_key;
