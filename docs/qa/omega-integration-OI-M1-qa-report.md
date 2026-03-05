# QA Report: OMEGA Integration OI-M1 -- OMEGA Store Credentials

## Scope Validated
Modules validated: `src/auth/omega_store.rs`, `src/auth/keyring.rs` (credential_store_factory patch), `src/services/mod.rs` (bootstrap_service_context patch), `src/auth/mod.rs` (module declaration), CLI `auth status`/`auth list`/`auth remove` behavior with OMEGA_STORES_DIR set.

Requirements in scope: REQ-OI-001 through REQ-OI-007 (6 Must, 1 Should).

## Summary
**FAIL** -- One blocking Must requirement fails. REQ-OI-004 (bootstrap_service_context supports OMEGA store credentials) has a critical bug: the OMEGA store's `get_token()` always returns `access_token: None` and `created_at: Utc::now()`, which causes `needs_refresh()` to return `false` (since the token appears freshly created). As a result, the refresh step is skipped and all commands fail with "No access token available". Additionally, REQ-OI-006 (auth status shows OMEGA store mode) is not implemented in the CLI -- `handle_auth_status` does not print any OMEGA store indicator when OMEGA_STORES_DIR is active.

All 51 unit tests pass. The foundational data structures and CredentialStore implementation are correct. The issue is a logic mismatch between the OMEGA store's `get_token()` return values and the `needs_refresh()` heuristic in `bootstrap_service_context`.

## System Entrypoint
- **Build**: `cargo build` (Rust CLI, binary at `target/debug/omg-gog`)
- **Test**: `cargo test` (1,467 lib tests + integration tests across 24 test binaries)
- **Run**: `OMEGA_STORES_DIR=/path/to/dir cargo run -- <command>`
- **Environment**: macOS Darwin 25.2.0, Rust edition 2021

## Traceability Matrix Status

| Requirement ID | Priority | Has Tests | Tests Pass | Acceptance Met | Notes |
|---|---|---|---|---|---|
| REQ-OI-001 | Must | Yes (14 tests) | Yes | Yes | All 5 acceptance criteria verified: read valid JSON, use email as account, skip keyring, missing file error, malformed JSON error |
| REQ-OI-002 | Must | Yes (11 tests) | Yes | Yes | All 9 acceptance criteria verified: implements trait, get/set/delete/list/keys/default account all correct |
| REQ-OI-003 | Must | Yes (5 tests) | Yes | Yes | Factory check at top of function, returns OMEGA store, falls through when unset |
| REQ-OI-004 | Must | Yes (3 tests) | Yes | **No** | Tests verify client_credentials extraction works, but do NOT test the full bootstrap->refresh path. Live testing reveals `get_token()` returns `access_token: None` + `created_at: now()`, causing `needs_refresh()` to return false, so refresh never happens |
| REQ-OI-005 | Must | Yes (6 tests) | Yes | Yes | Atomic write (tmp+rename), 0600 permissions, field preservation all verified |
| REQ-OI-006 | Must | Yes (3 tests) | Yes | **No** | `is_omega_store_active()` works correctly (verified by unit tests + integration). But `handle_auth_status` in `src/cli/mod.rs` was NOT updated to print "Source: OMEGA store" -- it still shows "Keyring backend: auto" |
| REQ-OI-007 | Should | Yes (3 tests) | Yes | Partial | Missing file and invalid JSON errors are actionable with paths. Missing field error says "not valid JSON: missing field `X`" instead of spec's "missing required field: X" |

### Gaps Found
- **No e2e tests** for OMEGA store integration (no `OMEGA_STORES_DIR` references in `tests/` directory)
- **REQ-OI-004 tests** verify `client_credentials()` extraction but not the full bootstrap-to-refresh flow
- **REQ-OI-006 tests** verify `is_omega_store_active()` but not that `auth status` CLI output shows OMEGA mode
- **No test for `bootstrap_service_context`** with OMEGA store active (the needs_refresh logic gap was not caught by tests)
- Auth functionalities docs (`docs/functionalities/auth-functionalities.md`) do not mention OMEGA store

## Acceptance Criteria Results

### Must Requirements

#### REQ-OI-001: Read credentials from OMEGA store
- [x] When `OMEGA_STORES_DIR` is set, reads all fields from google.json -- PASS
- [x] Uses email field as current account -- PASS (verified: `auth list` shows `testuser@gmail.com` from store)
- [x] Skips keyring entirely -- PASS (factory returns OMEGA store before keyring logic)
- [x] Missing file produces actionable error -- PASS ("OMEGA_STORES_DIR is set but .../google.json not found")
- [x] Malformed JSON produces actionable error -- PASS ("...is not valid JSON: key must be a string at line 1 column 2")

#### REQ-OI-002: OmegaStoreCredentialStore implements CredentialStore
- [x] New struct in `src/auth/omega_store.rs` -- PASS
- [x] Implements all CredentialStore trait methods -- PASS (compile-time verified + runtime tested)
- [x] `get_token()` reads google.json, constructs TokenData -- PASS
- [x] `set_token()` writes back only refresh_token -- PASS (verified: other fields preserved after write)
- [x] `list_tokens()` returns single-element vec -- PASS
- [x] `keys()` returns single key -- PASS (format: `token:omega:{email}`)
- [x] `get_default_account()` returns email from google.json -- PASS
- [x] `set_default_account()` is a no-op -- PASS
- [x] `delete_token()` returns error -- PASS ("Cannot delete OMEGA store token via CLI")

#### REQ-OI-003: credential_store_factory detects OMEGA_STORES_DIR
- [x] Checks `OMEGA_STORES_DIR` before all other backend logic -- PASS (lines 466-473 of keyring.rs)
- [x] If set, returns `Box<OmegaStoreCredentialStore>` -- PASS
- [x] If not set, falls through to existing logic unchanged -- PASS
- [x] Empty string treated as "not set" -- PASS

#### REQ-OI-004: bootstrap_service_context supports OMEGA store credentials
- [x] When OMEGA store active, extracts ClientCredentials from google.json -- PASS (code correct at lines 132-137 of services/mod.rs)
- [ ] Token refresh uses these extracted credentials -- **FAIL**: Refresh never triggers because `get_token()` returns `created_at: Utc::now()` with `access_token: None` and `expires_at: None`. The `needs_refresh()` function sees a freshly-created token (age < 55 min) with no expiry, returns `false`, and the refresh step is skipped. The command then fails with "No access token available for {email}".
- [ ] After refresh, updated refresh_token is written back -- **FAIL**: Dependent on above; refresh never happens.
- [x] All other bootstrap steps work unchanged -- PASS (output mode, UI, circuit breaker all construct correctly)

**Root cause**: `OmegaStoreCredentialStore::get_token()` at line 135 sets `created_at: chrono::Utc::now()`. Since the OMEGA store does not persist access tokens, `access_token` is always `None`. The `needs_refresh()` heuristic interprets this as "freshly created token, no need to refresh" because `age < 55 minutes`.

**Suggested fix** (one of):
1. Set `created_at` to a past time (e.g., `chrono::Utc::now() - chrono::Duration::hours(2)`) so the heuristic triggers refresh
2. Set `expires_at` to `Some(past_time)` so the explicit expiry check triggers refresh
3. Add an `access_token.is_none()` check in `needs_refresh()` or in the bootstrap code

#### REQ-OI-005: Atomic write for OMEGA store updates
- [x] Writes use tmp file + rename pattern -- PASS (lines 97-118 of omega_store.rs: `.json.tmp` then `fs::rename`)
- [x] File permissions set to 0600 on Unix -- PASS (verified both by unit test and code inspection: line 113)
- [x] All existing fields preserved -- PASS (serde(flatten) + read-modify-write preserves extra fields)

#### REQ-OI-006: auth status shows OMEGA store mode
- [x] `is_omega_store_active()` correctly detects env var -- PASS (unit tests verify true/false/empty)
- [ ] `auth status` prints "Source: OMEGA store ($OMEGA_STORES_DIR/google.json)" -- **FAIL**: `handle_auth_status` in `src/cli/mod.rs` (lines 769-854) was NOT updated. It still shows "Keyring backend: auto" and "Credentials file: .../credentials.json" without any OMEGA store indicator.
- [x] Shows the email from the store -- PASS (Current account: testuser@gmail.com)
- [ ] Shows whether access token is cached -- **FAIL**: No access token status shown because the OMEGA store has no token caching; the "Token status: valid" shown is misleading (the token has no access_token, so it's not actually valid for API calls)

### Should Requirements

#### REQ-OI-007: OMEGA store error messages are actionable
- [x] Missing file: "OMEGA_STORES_DIR is set but {path}/google.json not found" -- PASS
- [x] Invalid JSON: "OMEGA_STORES_DIR is set but {path}/google.json is not valid JSON" -- PASS
- [x] Missing fields: error includes the field name via serde -- PASS (but says "not valid JSON: missing field `X`" rather than "missing required field: X")

## End-to-End Flow Results

| Flow | Steps | Result | Notes |
|---|---|---|---|
| auth list with OMEGA store | Set OMEGA_STORES_DIR, create google.json, run auth list | PASS | Shows `testuser@gmail.com omega Gmail,Calendar,...` |
| auth status with OMEGA store | Set OMEGA_STORES_DIR, create google.json, run auth status | PARTIAL | Shows correct email but no OMEGA store indicator |
| auth status --json with OMEGA store | Same as above with --json flag | PARTIAL | JSON output shows correct email but `keyring_backend: auto` |
| auth remove with OMEGA store | Set OMEGA_STORES_DIR, run auth remove --force | PASS | Correctly blocked: "Cannot delete OMEGA store token via CLI" |
| API command with OMEGA store | Set OMEGA_STORES_DIR, run calendar ls | **FAIL** | "No access token available" -- refresh never triggered |
| Forward compatibility | google.json with extra fields | PASS | Extra fields tolerated and preserved on write |
| Missing file error | OMEGA_STORES_DIR set to nonexistent path | PASS | Actionable error with path |
| Malformed JSON error | OMEGA_STORES_DIR with invalid JSON file | PASS | Actionable error with parsing detail |
| Missing field error | google.json missing refresh_token | PASS | Error mentions the specific field name |
| Empty env var fallback | OMEGA_STORES_DIR="" | PASS | Falls through to normal keyring |

## Exploratory Testing Findings

| # | What Was Tried | Expected | Actual | Severity |
|---|---|---|---|---|
| 1 | Run `calendar ls` with valid OMEGA store | Token refresh, then API call | "No access token available" error (exit 4) | **high** |
| 2 | `auth status` with OMEGA_STORES_DIR set | Output indicates OMEGA store source | Shows "Keyring backend: auto", no OMEGA indicator | **medium** |
| 3 | `auth status` with OMEGA_STORES_DIR pointing to nonexistent dir | Error message about missing file | Shows "Current account: (none)" silently (error swallowed by `.ok()`) | **medium** |
| 4 | `auth remove --force` on OMEGA store token | Clear rejection message | "Cannot delete OMEGA store token via CLI" (correct) | (pass) |
| 5 | google.json with extra/future fields after set_token | Extra fields preserved | Extra fields correctly preserved via `serde(flatten)` | (pass) |
| 6 | `auth list` shows OMEGA store account | Shows account with "omega" client name | Shows `testuser@gmail.com omega Gmail,...` correctly | (pass) |

## Failure Mode Validation

| Failure Scenario | Triggered | Detected | Recovered | Degraded OK | Notes |
|---|---|---|---|---|---|
| google.json missing | Yes | Yes | Yes (error message) | Yes | "OMEGA_STORES_DIR is set but .../google.json not found" |
| Malformed JSON | Yes | Yes | Yes (error message) | Yes | "...is not valid JSON: key must be a string..." |
| Missing field | Yes | Yes | Yes (error message) | Yes | "...not valid JSON: missing field `refresh_token`" |
| Write to read-only dir | Yes (unit test) | Yes | Yes (error returned) | Yes | set_token fails gracefully |
| Permission denied on read | Yes (unit test) | Yes | Yes (error returned) | Yes | Constructor or read fails gracefully |
| google.json is a directory | Yes (unit test) | Yes | Yes (error returned) | Yes | Read fails gracefully |
| Wrong field types in JSON | Yes (unit test) | Yes | Yes (error returned) | Yes | Serde deserialization fails |
| Token refresh not triggered | Yes | **No** | **No** | **No** | BLOCKING: `needs_refresh()` returns false for OMEGA store tokens |

## Security Validation

| Attack Surface | Test Performed | Result | Notes |
|---|---|---|---|
| Debug output leaks secrets | Formatted OmegaStoreData with Debug trait | PASS | `client_secret` and `refresh_token` show as `[REDACTED]` |
| File permissions after write | Checked permissions after set_token | PASS | File is 0600 after atomic write |
| Permission tightening | Started with 0644 file, ran set_token | PASS | Permissions tightened to 0600 |
| Delete token blocked | Ran `auth remove --force` | PASS | Returns error: "Cannot delete OMEGA store token via CLI" |
| Extra fields in JSON | Added `future_field`, `nested` JSON fields | PASS | Parsed without error, preserved on write |
| Path traversal via env var | N/A -- env var is trusted (set by OMEGA, not user input from network) | Out of Scope | Per architecture: "path comes from env var (trusted -- set by OMEGA)" |

## Specs/Docs Drift

| File | Documented Behavior | Actual Behavior | Severity |
|------|-------------------|-----------------|----------|
| specs/omega-integration-requirements.md (REQ-OI-007 AC-3) | "missing required field: {field}" | "not valid JSON: missing field `{field}`" (serde error message phrasing) | low |
| specs/omega-integration-requirements.md (REQ-OI-006 AC-1) | auth status prints "Source: OMEGA store ($OMEGA_STORES_DIR/google.json)" | auth status shows standard keyring output, no OMEGA indicator | medium |
| specs/omega-integration-architecture.md (Data Flow step 5) | "If refresh needed: [...] refresh_access_token() -> get new access_token" | Refresh never triggered for OMEGA store tokens due to needs_refresh() heuristic | high |
| specs/SPECS.md (milestone table) | OI-M1 status: "Planned" | OI-M1 is implemented (code exists, tests pass) | low |
| docs/functionalities/auth-functionalities.md | No mention of OMEGA store | OMEGA store is fully implemented in auth module | low |

## Blocking Issues (must fix before merge)

- **ISSUE-001**: `src/auth/omega_store.rs` line 135, `src/services/mod.rs` lines 131-152 -- OMEGA store `get_token()` sets `created_at: chrono::Utc::now()` and `access_token: None`, causing `needs_refresh()` to return `false`. Token refresh is never triggered, and all API commands fail with "No access token available for {email}". Every command that goes through `bootstrap_service_context()` is affected. The suggested fix is to set `created_at` to a time in the past (e.g., `Utc::now() - Duration::hours(2)`) or set `expires_at` to a past time, so the refresh heuristic triggers.

- **ISSUE-002**: `src/cli/mod.rs` lines 769-854 -- `handle_auth_status` was not updated for REQ-OI-006. When OMEGA_STORES_DIR is active, it should print "Source: OMEGA store ({path}/google.json)" and show the store's email. Currently it shows "Keyring backend: auto" and "Credentials file: .../credentials.json" with no indication that the OMEGA store is in use. The `is_omega_store_active()` function exists and works but is not called in this handler.

## Non-Blocking Observations

- **OBS-001**: `src/cli/mod.rs` line 795 -- `handle_auth_status` calls `credential_store_factory(&cfg).ok()` which swallows OMEGA store errors (e.g., missing google.json). When OMEGA_STORES_DIR is set but the file is missing, status silently shows "Current account: (none)" instead of reporting the configuration error. Suggest: propagate the error or show a warning when OMEGA_STORES_DIR is set but store initialization fails.

- **OBS-002**: `specs/omega-integration-requirements.md` REQ-OI-007 AC-3 -- Missing field error message says "not valid JSON: missing field `X`" because serde's deserialization error is caught by the JSON validation branch. The spec says "missing required field: {field}". The error is still actionable (names the field and path), but the phrasing differs from the spec. Suggest: add a separate validation step after serde parse to catch missing fields with the exact specified message, or update the spec to match the implementation.

- **OBS-003**: No e2e integration tests exist for the OMEGA store (no `OMEGA_STORES_DIR` references in the `tests/` directory). All testing is via unit tests in `omega_store.rs`. Adding at least one e2e test in `tests/e2e_test.rs` that sets `OMEGA_STORES_DIR` and runs `auth list` would increase confidence.

- **OBS-004**: `src/auth/omega_store.rs` line 131 -- `get_token()` uses `client: "omega"` as a hardcoded client name. This works because the OMEGA store ignores client/email parameters, but it could be confusing when displayed (e.g., in `auth list` output: `testuser@gmail.com omega Gmail,...`). The "omega" client name is not documented in the requirements.

## Modules Not Validated (if context limited)
- OI-M2 (Gmail Watch Commands) -- not in scope, not yet implemented
- OI-M3 (Calendar + Drive Watch) -- not in scope, not yet implemented
- OI-M4 (Webhook Serve) -- not in scope, not yet implemented

## Final Verdict

**FAIL** -- REQ-OI-004 (Must) fails due to the `needs_refresh()` / `get_token()` logic mismatch that prevents token refresh from ever triggering for OMEGA store tokens. REQ-OI-006 (Must) fails because `auth status` was not updated to show OMEGA store mode. These must be fixed before this work can be reviewed.

Specifically:
1. **ISSUE-001** (BLOCKING): Fix `get_token()` to ensure `needs_refresh()` returns `true` when `access_token` is `None`
2. **ISSUE-002** (BLOCKING): Update `handle_auth_status` to detect and display OMEGA store mode
