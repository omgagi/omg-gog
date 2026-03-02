# QA Report: RT-M1 Auth Core

## Scope Validated
- `src/auth/mod.rs` -- TokenData struct extension (access_token, expires_at fields)
- `src/auth/token.rs` -- serialize/deserialize with new fields, needs_refresh logic, refresh_access_token stub
- `src/auth/oauth.rs` -- exchange_code stub, TokenResponse struct, auth URL constants
- `src/auth/service_account.rs` -- exchange_jwt stub, ServiceAccountTokenResponse, JWT claims, key loading
- `src/auth/keyring.rs` -- KeyringCredentialStore stub, FileCredentialStore (full), credential_store_factory stub

## Summary
**CONDITIONAL APPROVAL** -- All Must requirements have their test infrastructure in place and all 95 RT-M1 tests pass (4 ignored for OS keyring). However, four of the six Must requirements (REQ-RT-001, REQ-RT-005, REQ-RT-006, REQ-RT-013, REQ-RT-015) contain stub functions that bail with "not yet implemented". The stubs compile, match the expected trait/type signatures (with noted exceptions for exchange_code and exchange_jwt), and all tests pass -- but the functions do not perform real work yet. The implemented portions (REQ-RT-007: TokenData extension, serialization, needs_refresh logic) are fully functional and correct. Two function signatures (exchange_code, exchange_jwt) do not match the architecture and will need breaking changes when implemented.

## System Entrypoint
- **Build**: `cargo build` (Rust project, Cargo.toml at project root)
- **Test suite**: `cargo test --lib` (1201 tests total, 95 RT-M1 specific via `cargo test --lib req_rt_`)
- **Lint**: `cargo clippy -- -D warnings`
- **Environment**: No special setup required. Tests use tempfile for filesystem operations and mockito for HTTP mocks.

## Traceability Matrix Status

| Requirement ID | Priority | Has Tests | Tests Pass | Acceptance Met | Notes |
|---|---|---|---|---|---|
| REQ-RT-001 | Must | Yes (15 tests) | Yes (15/15) | PARTIAL | exchange_code is a stub that bails. TokenResponse deserialization works. Signature missing http_client param per architecture. |
| REQ-RT-005 | Must | Yes (10 tests) | Yes (10/10) | PARTIAL | refresh_access_token is a stub that bails. needs_refresh logic is fully implemented. TokenResponse deserialization works. |
| REQ-RT-006 | Must | Yes (14 tests) | Yes (14/14) | PARTIAL | exchange_jwt is a stub that bails. ServiceAccountTokenResponse deserialization works. JWT claims + key loading work. Signature missing http_client param and returns String instead of ServiceAccountTokenResponse per architecture. |
| REQ-RT-007 | Must | Yes (29 tests) | Yes (29/29) | YES | TokenData extended with access_token and expires_at. Serialization/deserialization handles new fields. Backward compatible. needs_refresh uses expires_at with 5-min buffer. |
| REQ-RT-013 | Must | Yes (14 tests, 4 ignored) | Yes (10/10, 4 ignored) | PARTIAL | KeyringCredentialStore struct exists, implements CredentialStore trait, is Send+Sync. All methods are stubs. FileCredentialStore fully works. OS keyring tests correctly ignored (require real keychain). |
| REQ-RT-015 | Must | Yes (17 tests) | Yes (17/17) | PARTIAL | credential_store_factory is a stub that bails. FileCredentialStore set/get/delete/list/keys/defaults all work correctly. Factory signature and return type are correct. |

### Gaps Found
- **Signature mismatch (REQ-RT-001)**: `exchange_code` is `(creds, code, redirect_uri)` but architecture specifies `(http_client, creds, code, redirect_uri)`. Tests document this gap and include commented-out code for the correct signature.
- **Signature mismatch (REQ-RT-006)**: `exchange_jwt` is `(assertion) -> Result<String>` but architecture specifies `(http_client, assertion) -> Result<ServiceAccountTokenResponse>`. Tests document this gap.
- **No tests for actual HTTP exchange**: The mockito-based tests for exchange_code and exchange_jwt create mock servers but cannot exercise them because the stubs don't make HTTP calls. The mocks are correctly set up for when the developer implements the functions.
- **credential_store_factory dispatch logic untested**: Since the factory is a stub, tests like `req_rt_015_factory_file_backend` and `req_rt_015_factory_auto_backend` only verify the function exists and returns an error. They don't verify correct dispatch behavior.

## Acceptance Criteria Results

### Must Requirements

#### REQ-RT-001: OAuth code exchange
- [x] exchange_code function exists and compiles -- PASS (stub function present)
- [ ] POST to token endpoint with grant_type=authorization_code -- FAIL: stub bails immediately, no HTTP call made
- [x] TokenResponse struct deserializes correctly from Google's response format -- PASS
- [ ] Returns error with Google's error message on failure (400, 401) -- FAIL: stub returns generic "not yet implemented" error, not Google's error
- [x] TOKEN_URL hardcoded to https://oauth2.googleapis.com/token -- PASS
- [x] AUTH_URL hardcoded to https://accounts.google.com/o/oauth2/v2/auth -- PASS
- [ ] Signature includes http_client parameter per architecture -- FAIL: current signature is `(creds, code, redirect_uri)`, missing `http_client`

#### REQ-RT-005: Token refresh
- [x] refresh_access_token function exists with correct signature -- PASS (matches architecture: http_client, creds, refresh_token)
- [ ] POSTs to token endpoint with grant_type=refresh_token -- FAIL: stub bails immediately
- [x] needs_refresh checks expires_at with 5-min buffer -- PASS
- [x] needs_refresh falls back to created_at heuristic when expires_at is None -- PASS
- [ ] Updates stored access_token and expires_at in credential store on success -- FAIL: not implemented
- [x] TOKEN_URL hardcoded -- PASS

#### REQ-RT-006: Service account JWT exchange
- [x] exchange_jwt function exists -- PASS (stub present)
- [ ] POSTs to token endpoint with grant_type=jwt-bearer -- FAIL: stub bails immediately
- [x] ServiceAccountTokenResponse struct deserializes correctly -- PASS
- [x] JwtClaims serialize correctly (including skip_serializing_if for sub) -- PASS
- [x] load_service_account_key validates key type -- PASS
- [ ] exchange_jwt signature matches architecture (http_client param, returns ServiceAccountTokenResponse) -- FAIL: current is `(assertion) -> Result<String>`
- [x] Private key not leaked in error messages -- PASS

#### REQ-RT-007: Access token caching in TokenData
- [x] TokenData has access_token: Option<String> -- PASS
- [x] TokenData has expires_at: Option<DateTime<Utc>> -- PASS
- [x] New fields are Option (can be None) -- PASS
- [x] serialize_token includes access_token when Some -- PASS
- [x] serialize_token includes expires_at as RFC3339 when Some -- PASS
- [x] serialize_token omits access_token when None -- PASS
- [x] serialize_token omits expires_at when None -- PASS
- [x] deserialize_token reads access_token from JSON -- PASS
- [x] deserialize_token reads expires_at from JSON -- PASS
- [x] Backward compatible: old data without new fields deserializes to None -- PASS
- [x] Round-trip: serialize then deserialize preserves new fields -- PASS
- [x] Round-trip: serialize then deserialize preserves None fields -- PASS
- [x] needs_refresh uses expires_at when present (5-min buffer) -- PASS
- [x] needs_refresh falls back to created_at when expires_at is None -- PASS
- [x] Boundary: exactly 5 minutes triggers refresh -- PASS
- [x] Already expired token triggers refresh -- PASS
- [x] Invalid expires_at format handled gracefully (defaults to None) -- PASS
- [x] Far-future expires_at does not trigger refresh -- PASS
- [x] Clone preserves new fields -- PASS

#### REQ-RT-013: OS keyring backend
- [x] KeyringCredentialStore struct exists -- PASS
- [x] KeyringCredentialStore implements CredentialStore trait -- PASS
- [x] KeyringCredentialStore is Send + Sync -- PASS
- [x] Service name is "omega-google" (APP_NAME constant) -- PASS
- [x] Key format: token:<client>:<email> -- PASS
- [x] Graceful failure: new() returns Err, does not panic -- PASS
- [ ] get_token/set_token/delete_token/list_tokens/keys operations -- FAIL: all methods are stubs that bail
- [x] FileCredentialStore (fallback) works correctly for all operations -- PASS
- [x] File permissions are 0600 on Unix -- PASS

#### REQ-RT-015: Credential store factory
- [x] credential_store_factory function exists with correct signature -- PASS
- [x] Returns Box<dyn CredentialStore> -- PASS (compile-time verified)
- [ ] "file" backend returns FileCredentialStore -- FAIL: stub bails
- [ ] "keychain"/"keyring" forces OS keyring -- FAIL: stub bails
- [ ] "auto" tries OS keyring, falls back to file -- FAIL: stub bails
- [ ] None defaults to "auto" -- FAIL: stub bails
- [ ] GOG_KEYRING_BACKEND env overrides config -- FAIL: stub bails
- [x] FileCredentialStore set/get/delete/list/keys cycle works -- PASS
- [x] FileCredentialStore default account operations work -- PASS
- [x] FileCredentialStore multiple clients isolated correctly -- PASS
- [x] FileCredentialStore overwrite preserves single entry -- PASS
- [x] FileCredentialStore empty directory returns empty results -- PASS
- [x] Unknown backend returns error -- PASS (stub bails, which is correct)

## End-to-End Flow Results

| Flow | Steps | Result | Notes |
|---|---|---|---|
| Token serialize/deserialize round-trip (with new fields) | Serialize TokenData -> JSON -> Deserialize -> compare | PASS | access_token, expires_at, and all original fields preserved with <1s tolerance |
| Token serialize/deserialize round-trip (without new fields) | Serialize TokenData with None fields -> JSON -> Deserialize -> verify None | PASS | None fields correctly omitted from JSON and restored as None |
| Backward compatibility: old token format | Deserialize JSON without access_token/expires_at -> verify None | PASS | No error, fields default to None |
| needs_refresh + token lifecycle | Create token with various expires_at values -> check needs_refresh | PASS | 5-min buffer works correctly at all boundaries |
| FileCredentialStore full lifecycle | Create store -> set token -> get token -> list tokens -> delete -> verify gone | PASS | Complete CRUD cycle works, permissions set to 0600 |
| FileCredentialStore multi-client | Set tokens for different clients -> verify isolation -> verify separate defaults | PASS | Clients correctly isolated |
| OAuth code exchange (stub) | Call exchange_code with any args | PASS (as stub) | Returns error "not yet implemented" -- no actual HTTP exchange |
| JWT exchange (stub) | Call exchange_jwt with any args | PASS (as stub) | Returns error "not yet implemented" -- no actual HTTP exchange |
| Token refresh (stub) | Call refresh_access_token with any args | PASS (as stub) | Returns error "not yet implemented" -- no actual HTTP exchange |

## Exploratory Testing Findings

| # | What Was Tried | Expected | Actual | Severity |
|---|---|---|---|---|
| 1 | TokenData with Debug derive exposes refresh_token and access_token in debug output | Sensitive fields should be redacted in Debug output per architecture security note | Debug output includes full refresh_token and access_token values. Test `req_rt_007_token_data_debug_contains_access_token` documents this. | medium |
| 2 | deserialize_token with invalid RFC3339 expires_at | Should either fail or default to None | Defaults to None (graceful). This is correct behavior per the architecture. | low (positive finding) |
| 3 | deserialize_token with missing email field | Should return error | Returns error "missing 'email' in token data". Clear error message. | low (positive finding) |
| 4 | deserialize_token with missing refresh_token field | Could be problematic | Defaults to empty string via `.unwrap_or("")`. This means a token without a refresh_token silently gets an empty string. | medium |
| 5 | exchange_code signature does not include http_client | Architecture requires http_client parameter for testability | Current stub uses `(_creds, _code, _redirect_uri)`. Tests are written against current stub, with comments for future update. | high |
| 6 | exchange_jwt signature returns String instead of ServiceAccountTokenResponse | Architecture requires returning typed response | Current stub uses `(_assertion) -> Result<String>`. This will be a breaking change when implemented. | high |
| 7 | credential_store_factory takes _config but ignores GOG_KEYRING_BACKEND env | Architecture requires env override of config | Stub bails immediately, so env override is not tested. | medium |
| 8 | FileCredentialStore write_tokens_map serializes tokens as HashMap<String, String> where values are JSON strings | Nested JSON strings could cause confusion | Works correctly, but stored format has JSON within JSON (token keys map to JSON-encoded token data strings). This is valid but means the tokens.json file has escaped JSON. | low |

## Failure Mode Validation

| Failure Scenario | Triggered | Detected | Recovered | Degraded OK | Notes |
|---|---|---|---|---|---|
| Old token missing new fields | Yes | Yes | Yes | Yes | Deserializes to None for access_token/expires_at, needs_refresh falls back to created_at heuristic. Correct per architecture. |
| File permission denied (write) | Yes | Yes | N/A | N/A | Returns clear error. Test `req_rt_013_failure_permission_denied` passes on Unix. |
| Non-existent token get | Yes | Yes | N/A | N/A | Returns error "no token found for ..." |
| Delete non-existent token | Yes | Yes | N/A | Yes | No error (idempotent). Correct behavior. |
| Invalid JSON deserialization | Yes | Yes | N/A | N/A | Returns serde error |
| Empty string deserialization | Yes | Yes | N/A | N/A | Returns serde error |
| Missing email field | Yes | Yes | N/A | N/A | Returns "missing 'email' in token data" |
| Service account wrong key type | Yes | Yes | N/A | N/A | Returns "expected key type 'service_account', got 'wrong'" |
| Service account key file not found | Yes | Yes | N/A | N/A | Returns OS file-not-found error |
| KeyringCredentialStore unavailable | Yes | Yes | N/A | N/A | new() returns Err, does not panic. |
| Refresh token revoked (invalid_grant) | Not Triggered | N/A | N/A | N/A | refresh_access_token is a stub; cannot test actual HTTP error handling |
| Token endpoint unreachable | Not Triggered | N/A | N/A | N/A | refresh_access_token is a stub; test exists but only verifies stub error |
| Clock skew | Not Triggered (untestable) | N/A | N/A | N/A | Would require mocking Utc::now() which is not injectable |

## Security Validation

| Attack Surface | Test Performed | Result | Notes |
|---|---|---|---|
| Token URL hardcoded | Verified TOKEN_URL constant in oauth.rs and service_account.rs | PASS | Both hardcoded to `https://oauth2.googleapis.com/token` with HTTPS |
| Auth URL hardcoded | Verified AUTH_URL constant in oauth.rs | PASS | Hardcoded to `https://accounts.google.com/o/oauth2/v2/auth` with HTTPS |
| Private key in error messages | Created service account key with wrong type, checked error output | PASS | Error message is "expected key type 'service_account', got 'wrong'" -- no private key data leaked |
| Token file permissions | Created file store, wrote token, checked Unix permissions | PASS | tokens.json and defaults.json both get 0600 permissions |
| Refresh token in stdout | Verified no println!/print! calls in src/auth/ | PASS | No print statements in any auth module |
| Debug output sensitivity | TokenData derives Debug, which includes refresh_token and access_token | PARTIAL FAIL | Debug output exposes sensitive tokens. Architecture notes say these "must never appear in stdout, logs, or error messages." While no code currently uses Debug on TokenData in production paths, the derived Debug is a latent risk. |
| TokenResponse extra fields | Sent JSON with unknown fields to TokenResponse deserialization | PASS | Extra fields silently ignored (serde default). No deny_unknown_fields. |
| Base scopes always included | Verified BASE_SCOPES constant contains openid, email, userinfo.email | PASS | All three present |

## Specs/Docs Drift

| File | Documented Behavior | Actual Behavior | Severity |
|------|-------------------|-----------------|----------|
| specs/runtime-architecture.md | `exchange_code` signature: `(http_client: &reqwest::Client, creds: &ClientCredentials, code: &str, redirect_uri: &str)` | Actual signature: `(_creds: &ClientCredentials, _code: &str, _redirect_uri: &str)` -- missing http_client | high |
| specs/runtime-architecture.md | `exchange_jwt` signature: `(http_client: &reqwest::Client, assertion: &str) -> Result<ServiceAccountTokenResponse>` | Actual signature: `(_assertion: &str) -> Result<String>` -- missing http_client, wrong return type | high |
| specs/runtime-architecture.md | TokenData uses `#[serde(skip_serializing_if = "Option::is_none")]` and `#[serde(default)]` on new fields | TokenData does not derive Serialize/Deserialize; uses manual serialize_token/deserialize_token functions instead. This achieves the same effect but through different means. | low |
| specs/runtime-architecture.md | credential_store_factory reads GOG_KEYRING_BACKEND env and dispatches | Factory is a stub that bails immediately | medium (expected -- not yet implemented) |
| specs/runtime-architecture.md | KeyringCredentialStore maintains a keyring-index.json for key enumeration | No index file logic exists in the stub | medium (expected -- not yet implemented) |
| specs/runtime-requirements.md | REQ-RT-005 says "If refresh fails with invalid_grant, returns exit code 4 with re-auth message" | refresh_access_token is a stub, no error code mapping | medium (expected -- not yet implemented) |

## Blocking Issues (must fix before merge)

There are no issues that prevent merging RT-M1 as-is, because the milestone is explicitly structured as a test-first (TDD) phase where stubs are acceptable. However, the following items MUST be addressed before the stubs are replaced with implementations:

- **[ISSUE-001]**: `src/auth/oauth.rs:55` -- exchange_code signature must be updated to include `http_client: &reqwest::Client` as first parameter before implementation. Tests in oauth.rs and token.rs that call this function will need updating simultaneously.

- **[ISSUE-002]**: `src/auth/service_account.rs:81` -- exchange_jwt signature must be updated to include `http_client: &reqwest::Client` as first parameter and return `Result<ServiceAccountTokenResponse>` instead of `Result<String>` before implementation. Tests will need updating simultaneously.

## Non-Blocking Observations

- **[OBS-001]**: `src/auth/mod.rs:58` -- TokenData derives Debug which exposes refresh_token and access_token in debug output. Consider implementing a custom Debug that redacts sensitive fields (e.g., shows "ya29.***" or "[REDACTED]"). The architecture security section states these "must never appear in stdout, logs, or error messages."

- **[OBS-002]**: `src/auth/token.rs:68` -- deserialize_token defaults missing refresh_token to empty string via `.unwrap_or("")`. This silently creates a token with an empty refresh_token rather than failing. Consider making refresh_token required in deserialization (return error if missing), or at minimum documenting that an empty refresh_token is valid for service account tokens.

- **[OBS-003]**: `src/auth/keyring.rs:174` -- credential_store_factory currently ignores the `_config` parameter entirely (it's a stub). When implemented, ensure GOG_KEYRING_BACKEND env var takes precedence over config per architecture spec.

- **[OBS-004]**: Test `req_rt_015_factory_env_overrides_config` is essentially a no-op test -- it creates a config but never calls the factory with it, citing parallel test env var race conditions. When the factory is implemented, this test should be made functional (consider serial test attribute or temp env scoping).

- **[OBS-005]**: Several mockito-based tests (e.g., `req_rt_001_exchange_code_posts_authorization_code`, `req_rt_006_exchange_jwt_posts_jwt_bearer_grant_type`) create mock servers but never hit them because the stub functions don't make HTTP calls. These tests will become meaningful once the stubs are replaced. Currently they silently pass without testing anything.

- **[OBS-006]**: The `ServiceAccountTokenResponse` struct does not ignore unknown fields (no explicit `deny_unknown_fields`, which is correct), but test `req_rt_006_edge_sa_token_response_extra_fields` verifies this. Good.

## Test Execution Summary

| Test Suite | Total | Passed | Failed | Ignored |
|---|---|---|---|---|
| All library tests (`cargo test --lib`) | 1205 | 1201 | 0 | 4 |
| RT-M1 tests only (`cargo test --lib req_rt_`) | 99 | 95 | 0 | 4 |
| Clippy (`cargo clippy -- -D warnings`) | N/A | PASS | 0 warnings | N/A |

### Ignored Tests (all expected)
1. `req_rt_013_keyring_set_get_roundtrip` -- requires OS keychain
2. `req_rt_013_keyring_delete` -- requires OS keychain
3. `req_rt_013_keyring_list_tokens` -- requires OS keychain
4. `req_rt_013_keyring_default_account` -- requires OS keychain

## Modules Not Validated
None -- all RT-M1 modules were validated within this report.

## Final Verdict

**CONDITIONAL APPROVAL** -- The fully implemented portion of RT-M1 (REQ-RT-007: TokenData extension, serialization/deserialization, needs_refresh logic) is correct, thorough, and meets all acceptance criteria. The remaining requirements (REQ-RT-001, REQ-RT-005, REQ-RT-006, REQ-RT-013, REQ-RT-015) have proper test infrastructure, correct type definitions, and working supporting structures (TokenResponse, ServiceAccountTokenResponse, FileCredentialStore), but their core functions remain stubs. This is consistent with the documented TDD red-phase approach.

Before these stubs are implemented, two blocking signature mismatches must be addressed:
1. `exchange_code` needs `http_client` parameter added
2. `exchange_jwt` needs `http_client` parameter added and return type changed to `ServiceAccountTokenResponse`

All 95 RT-M1 tests pass. Clippy reports zero warnings. No security vulnerabilities found in implemented code. File permission security is correctly enforced (0600).
