# Bugfix Analysis: Browser Opens During Tests

## Bug
Running `cargo test` opens browser windows with Google OAuth error pages and accesses real OS keychain credentials.

## Root Cause
1. `open_browser()` in `src/auth/oauth_flow.rs:332` has no `#[cfg(test)]` guard
2. Tests calling `cli::execute` with auth commands lack `GOG_KEYRING_BACKEND=file` isolation
3. OS Keychain backend is used even when `OMEGA_GOOGLE_CONFIG_DIR` is set to a temp dir

## Affected Files
- `src/auth/oauth_flow.rs` — 7 tests open browser or block stdin
- `tests/cli_test.rs` — 13 tests missing full isolation
- `tests/rt_m4_handlers_test.rs` — 3 tests missing isolation
- `src/services/mod.rs` — 4 tests (already fixed)

## Requirements

| ID | Requirement | Priority |
|----|------------|----------|
| REQ-BUG-001 | `open_browser()` must not execute in test builds | Must |
| REQ-BUG-002 | All cli_test.rs auth tests must set `GOG_KEYRING_BACKEND=file` | Must |
| REQ-BUG-003 | rt_m4_handlers_test.rs service tests must set `GOG_KEYRING_BACKEND=file` | Must |
| REQ-BUG-004 | `auth remove --force` test must not risk deleting real credentials | Must |
| REQ-BUG-005 | Thread-safe env var handling (serial or RAII) | Should |
| REQ-BUG-006 | Desktop flow tests must not open real OAuth URLs | Must |
| REQ-BUG-007 | Manual flow test must not block on stdin | Should |
| REQ-BUG-008 | RAII cleanup of env vars on panic | Should |
| REQ-BUG-009 | Document test isolation pattern | Could |

## Fix Order
1. REQ-BUG-001 + REQ-BUG-006: Guard `open_browser()` with `#[cfg(test)]`
2. REQ-BUG-004: Fix `auth remove --force` test (data loss risk)
3. REQ-BUG-002: Add isolation to 13 cli_test.rs tests
4. REQ-BUG-003: Add isolation to 3 rt_m4_handlers_test.rs tests
5. REQ-BUG-005 + REQ-BUG-008: Thread-safe env var handling
6. REQ-BUG-009: Document the pattern
