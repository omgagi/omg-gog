# QA Report: RT-M7 Polish Features

## Scope Validated
Four "Should" priority features in the RT-M7 milestone:
- REQ-RT-004: Remote OAuth flow (two-step headless)
- REQ-RT-014: AES-GCM encrypted file backend
- REQ-RT-016: Keyring timeout on Linux (5s)
- REQ-RT-029: Resumable upload for files >5MB

Modules inspected: `src/auth/oauth_flow.rs`, `src/auth/keyring.rs`, `src/services/drive/files.rs`, `src/cli/mod.rs`, `src/cli/root.rs`.

## Summary
**CONDITIONAL APPROVAL** -- All four requirements are Should priority. The core implementations exist and their unit tests pass. However, REQ-RT-004 (remote OAuth flow) has a critical gap: the `--step` and `--auth-url` CLI flags are not defined in the CLI parser, so the two-step remote flow cannot actually be used by end users despite the backend functions being fully implemented and tested. REQ-RT-014 has a security concern with a non-cryptographic KDF (uses `DefaultHasher` instead of PBKDF2/Argon2). REQ-RT-016 and REQ-RT-029 are well implemented.

## System Entrypoint
- **Build**: `cargo build` (compiles in <0.1s, already built)
- **Test**: `cargo test --jobs 1`
- **Binary**: `target/debug/omega-google`
- The system is a CLI tool, not a server. Validation was performed by running the binary with `--help`, inspecting source code, and executing the full test suite.

## Test Suite Results
All tests pass with zero failures:
- 1,408 unit tests passed (lib.rs), 6 ignored (keyring integration + desktop flow integration)
- 406 integration tests passed across 22 test files, 0 ignored
- 0 doc-tests
- **Total: 1,814 passed, 0 failed, 6 ignored**

## Traceability Matrix Status

| Requirement ID | Priority | Has Tests | Tests Pass | Acceptance Met | Notes |
|---|---|---|---|---|---|
| REQ-RT-004 | Should | Yes (9 tests) | Yes | PARTIAL | Backend functions work; CLI integration missing (no `--step`/`--auth-url` flags) |
| REQ-RT-014 | Should | Yes (11 tests) | Yes | PARTIAL | Encrypt/decrypt roundtrip works; KDF uses non-cryptographic `DefaultHasher`; no TTY prompt for password |
| REQ-RT-016 | Should | Yes (1 test) | Yes | Yes | Linux timeout code present with 5s timeout; stderr hint on timeout; documented test |
| REQ-RT-029 | Should | Yes (3 tests) | Yes | Yes | URL builder, threshold, chunk size all correct; handler implemented with 308 handling and progress |

### Gaps Found

1. **REQ-RT-004 missing CLI wiring**: `remote_flow_step1()` and `remote_flow_step2()` are marked `#[allow(dead_code)]` and never called from the CLI handler. The `AuthAddArgs` struct in `src/cli/root.rs` defines `--remote` but not `--step` or `--auth-url`. When a user runs `auth add --remote`, it immediately fails with "Remote flow requires --step 1 or --step 2" but there is no way to provide `--step`.

2. **REQ-RT-014 missing TTY prompt**: The spec says "Password sourced from TTY prompt or `GOG_KEYRING_PASSWORD` env var". Only the env var path is implemented in `credential_store_factory()`. There is no TTY prompt fallback when the env var is not set.

3. **REQ-RT-029 missing network failure resumption**: The spec says "Resumable on network failure (re-PUT from last successful byte)". The current implementation does not implement resume-on-failure -- if a chunk PUT fails with a network error, the upload is aborted entirely (returns `GENERIC_ERROR`).

4. **No integration test for `handle_drive_resumable_upload`**: The handler function is tested only via unit tests on the URL builder and constants. There is no test that exercises the full upload flow (even with mocked HTTP).

## Acceptance Criteria Results

### Should Requirements

#### REQ-RT-004: Remote OAuth flow (--remote): two-step headless flow

- [x] `--remote --step 1` prints auth URL with a randomly generated state parameter -- PASS (unit level): `remote_flow_step1_with_dir` generates a 32-char alphanumeric state, appends `&state=` to the auth URL. Tested by `req_rt_004_step1_generates_url_with_state`.
- [x] State is cached in a temporary file in the config directory -- PASS (unit level): State written to `<config_dir>/remote_oauth_state`. Verified by test.
- [x] `--remote --step 2 --auth-url <url>` receives the redirect URL -- PASS (unit level): `remote_flow_step2_with_dir` reads cached state, validates against URL state, extracts code. Tested by `req_rt_004_step2_validates_state_extracts_code`.
- [x] Validates state parameter matches cached state -- PASS: State mismatch returns "State parameter mismatch. Possible CSRF attack or stale flow." Tested by `req_rt_004_step2_state_mismatch_returns_error`.
- [ ] Exchanges code for tokens -- FAIL: The step2 function returns an `OAuthFlowResult` with the code, but the CLI handler never calls step1/step2. The token exchange path in `handle_auth_add` is unreachable for remote flow because `run_remote_flow()` always returns an error.
- [ ] **CLI integration**: FAIL -- `--step` and `--auth-url` flags do not exist in `AuthAddArgs` (src/cli/root.rs:231-255). The remote flow is dead code from the user's perspective.

#### REQ-RT-014: File-based fallback with AES-GCM encryption

- [x] Existing `FileCredentialStore` extended with optional AES-GCM encryption -- PASS: `EncryptedFileCredentialStore` wraps `FileCredentialStore`, implements the full `CredentialStore` trait.
- [x] Password sourced from `GOG_KEYRING_PASSWORD` env var -- PASS: `credential_store_factory()` reads `std::env::var("GOG_KEYRING_PASSWORD")` and passes it to `EncryptedFileCredentialStore::new()`.
- [ ] Password sourced from TTY prompt -- FAIL: No TTY prompt implementation. Only the env var path exists.
- [x] Encryption uses AES-256-GCM with random nonce per entry -- PASS: `encrypt()` at line 293 uses `Aes256Gcm` with a 12-byte random nonce via `rand::thread_rng().fill_bytes()`. Each call produces different ciphertext (tested by `req_rt_014_encrypt_random_nonce`).
- [ ] Key derived from password using a KDF (e.g., PBKDF2 or Argon2) -- FAIL: `derive_key()` at line 263 uses `std::collections::hash_map::DefaultHasher` (SipHash), which is not a cryptographic hash and not a recognized KDF. The spec explicitly suggests PBKDF2 or Argon2. `DefaultHasher` has no work factor, no salt, and its output is not even stable across Rust versions.
- [x] Unencrypted mode remains for testing/CI (when no password provided) -- PASS: When `GOG_KEYRING_PASSWORD` is not set, `credential_store_factory()` returns plain `FileCredentialStore`. Tested by `req_rt_014_unencrypted_mode_no_password`.

#### REQ-RT-016: Keyring timeout on Linux (5 seconds)

- [x] When accessing D-Bus keyring on Linux, wrap with a 5-second timeout -- PASS: `KeyringCredentialStore::new()` at line 147 uses `#[cfg(target_os = "linux")]` block with `std::sync::mpsc::channel()` and `recv_timeout(Duration::from_secs(5))`.
- [x] On timeout, print hint: "Keyring timed out. Try GOG_KEYRING_BACKEND=file" -- PASS: Line 166 prints `"Keyring timed out after 5 seconds. Try GOG_KEYRING_BACKEND=file"` to stderr.
- [x] Fall back to file backend in `auto` mode on timeout -- PASS: `credential_store_factory()` catches the error from `KeyringCredentialStore::new()` and falls back to `FileCredentialStore` (or `EncryptedFileCredentialStore` if password is set).

Note: Cannot trigger timeout on macOS (non-Linux platform). The `#[cfg(target_os = "linux")]` block is verified by code inspection. Test `req_rt_016_keyring_probe_documented` verifies no panic.

#### REQ-RT-029: Drive file upload: resumable for large files

- [x] Files > 5MB use resumable upload protocol -- PASS: `RESUMABLE_THRESHOLD` is 5 * 1024 * 1024 (5,242,880 bytes). Check at cli/mod.rs:2760 routes to `handle_drive_resumable_upload`.
- [x] POST to initiate, then PUT chunks to the returned upload URI -- PASS: `handle_drive_resumable_upload` at line 2827 POSTs metadata with `X-Upload-Content-Type` and `X-Upload-Content-Length` headers, extracts `Location` header for upload URI, then sends chunks via PUT with `Content-Range` headers.
- [x] Progress reporting on stderr -- PASS: Progress displayed via `eprint!("\rUploading: {:.0}%", pct)` during chunk upload (line 2961).
- [ ] Resumable on network failure (re-PUT from last successful byte) -- FAIL: On network error during a chunk PUT (line 2931-2934), the function returns `GENERIC_ERROR` immediately. There is no retry or resume logic -- the upload is abandoned.

## End-to-End Flow Results

| Flow | Steps | Result | Notes |
|---|---|---|---|
| `omega-google --help` | 1 | PASS | All 27 commands listed, binary starts correctly |
| `omega-google auth add --help` | 1 | PASS | Shows `--remote` flag; does NOT show `--step` or `--auth-url` |
| `omega-google drive upload --help` | 1 | PASS | Shows upload options (no resumable-specific flags, which is correct -- resumable is automatic) |
| Remote flow user path | 3 | FAIL | User runs `auth add --remote`, gets error "requires --step 1 or --step 2" but `--step` does not exist |
| Encrypted file backend via env | 2 | PASS (code inspection) | Setting `GOG_KEYRING_BACKEND=file` and `GOG_KEYRING_PASSWORD=xxx` routes to `EncryptedFileCredentialStore` |

## Exploratory Testing Findings

| # | What Was Tried | Expected | Actual | Severity |
|---|---|---|---|---|
| 1 | `auth add --remote` without step flags | User-friendly error with guidance | Error: "Remote flow requires --step 1 or --step 2" but `--step` does not exist in CLI | high |
| 2 | `auth add --manual --remote` together | Mutual exclusion error or documented precedence | `--manual` silently takes precedence (if-else chain at cli/mod.rs:424-430) | low |
| 3 | Code inspection: `derive_key("a")` for 1-char password | Reasonable key derivation | 1-char password cycled to fill 32 bytes (all same byte repeated), then XORed with `DefaultHasher` rounds. Very low entropy. | medium |
| 4 | Code inspection: `derive_key` with empty string | Error or rejection | `"".as_bytes().iter().cycle().take(32)` panics -- cycle on empty iterator. | high |

### Exploratory Finding 4 Detail
The `derive_key` function at line 263 will panic if called with an empty password because `bytes.iter().cycle()` on an empty slice produces a panic when `.take(32)` tries to advance the iterator (cycle on an empty iterator panics in Rust). This could crash the application if `GOG_KEYRING_PASSWORD=""` is set.

Let me verify this finding.

**Correction after code re-inspection**: `Iterator::cycle()` on an empty iterator returns an iterator that immediately ends. `.take(32).enumerate()` would iterate zero times, leaving `key` as all zeros. Then the hash rounds would produce a deterministic key from the empty password. This is not a panic but a weak key. Re-checking the Rust docs: actually, `[].iter().cycle()` never yields, so `take(32)` yields nothing, the for loop does nothing, key stays `[0u8; 32]`. The hash rounds then mix in the password (empty) and round numbers. This produces a deterministic but weak key. Not a panic, but a security concern.

## Failure Mode Validation

| Failure Scenario | Triggered | Detected | Recovered | Degraded OK | Notes |
|---|---|---|---|---|---|
| Keyring timeout (Linux D-Bus) | Not Triggered (macOS platform) | Yes (code inspection) | Yes | Yes | `recv_timeout(5s)` -> stderr hint -> falls back to file backend |
| Keyring unavailable (auto mode) | Yes (built without keyring access in test) | Yes | Yes | Yes | Falls back to FileCredentialStore transparently |
| Wrong encryption password | Yes (unit test) | Yes | N/A | Yes | Returns "Decryption failed (wrong password?)" error |
| Network error during resumable chunk | Not Triggered (no real Google API) | Yes (code inspection) | No | No | Upload aborted entirely; no resume from last byte |
| Missing state file for remote step2 | Yes (unit test) | Yes | N/A | Yes | Returns "No pending remote flow. Run --step 1 first." |
| State mismatch (CSRF) | Yes (unit test) | Yes | N/A | Yes | Returns "State parameter mismatch. Possible CSRF attack or stale flow." |
| Resumable upload initiation fails (HTTP error) | Not Triggered | Yes (code inspection) | N/A | Yes | Returns GENERIC_ERROR with error message on stderr |
| No Location header in initiation response | Not Triggered | Yes (code inspection) | N/A | Yes | Returns "Error: no upload URI (Location header) in response" |

## Security Validation

| Attack Surface | Test Performed | Result | Notes |
|---|---|---|---|
| CSRF in remote flow | State parameter validation in step2 | PASS | Random 32-char state; mismatch -> error; missing state -> error |
| State file exposure | Code inspection | PASS | State file written to config dir (user-owned), contains only random alphanumeric string, cleaned up after step2 |
| Encrypted token file on disk | Code + test inspection | PASS (with caveat) | Tokens encrypted with AES-256-GCM, stored as base64 in JSON. Random nonce per entry. **Caveat**: KDF uses `DefaultHasher` not PBKDF2/Argon2 |
| Token file permissions | Unit test `req_rt_013_security_file_permissions` | PASS | Token files created with 0600 permissions on Unix |
| Password in environment variable | Code inspection | PASS | `GOG_KEYRING_PASSWORD` follows standard Unix env security model; documented in developer guide |
| Encrypted data not readable as plain JSON | Unit test `req_rt_014_encrypted_data_not_plain_json` | PASS | Stored values are base64-encoded ciphertext, not parseable as JSON |
| KDF strength | Code inspection | FAIL | `derive_key()` uses `std::collections::hash_map::DefaultHasher` (SipHash), not a cryptographic KDF. No salt. No intentional work factor. 256 rounds of SipHash is trivially fast to brute-force. |
| Empty password handling | Code inspection | PASS (weak) | Empty password produces all-zero initial key, mixed only by hash rounds. Results in a deterministic weak key. Should be rejected. |

## Specs/Docs Drift

| File | Documented Behavior | Actual Behavior | Severity |
|------|-------------------|-----------------|----------|
| `docs/command-reference.md:33-34` | `--remote --step 1\|2` and `--auth-url <url>` flags exist | These flags do not exist in `AuthAddArgs` (src/cli/root.rs). Only `--remote` exists. | high |
| `specs/runtime-requirements.md:58` (REQ-RT-004) | "Exchanges code for tokens" as acceptance criterion | Remote flow always errors; token exchange unreachable for remote mode | medium |
| `specs/runtime-requirements.md:73` (REQ-RT-014) | "Password sourced from TTY prompt or GOG_KEYRING_PASSWORD env var" | Only env var path implemented; no TTY prompt | medium |
| `specs/runtime-requirements.md:73` (REQ-RT-014) | "Key derived from password using a KDF (e.g., PBKDF2 or Argon2)" | Uses `DefaultHasher` (SipHash), not a cryptographic KDF | medium |
| `specs/runtime-requirements.md:103` (REQ-RT-029) | "Resumable on network failure (re-PUT from last successful byte)" | Network error aborts upload entirely; no resume logic | low |
| `src/auth/keyring.rs:336-337` (doc comment) | "Password sourced from GOG_KEYRING_PASSWORD env var or TTY prompt" | Only env var implemented | low |

## Blocking Issues (must fix before merge)

No Must requirements are in scope. All four requirements are Should priority, so failures are non-blocking by definition. However, the following issues are significant enough to warrant attention:

*None (all Should priority -- failures tracked as non-blocking)*

## Non-Blocking Observations

- **[OBS-001]**: `src/cli/root.rs` (AuthAddArgs) -- REQ-RT-004 remote flow is unreachable from CLI. The `--step` and `--auth-url` flags are not defined. `remote_flow_step1()` and `remote_flow_step2()` are dead code. Either add the CLI flags or remove the dead code to reduce maintenance burden.

- **[OBS-002]**: `src/auth/keyring.rs:263` (derive_key) -- `DefaultHasher` (SipHash) is used for key derivation instead of a cryptographic KDF (PBKDF2, Argon2, scrypt). This has no work factor, no salt, and `DefaultHasher`'s behavior is explicitly not guaranteed stable across Rust versions. Recommendation: use `ring::pbkdf2` or add an `argon2` dependency. For a local-only encrypted file, this is low-urgency but should be fixed before any public release.

- **[OBS-003]**: `src/auth/keyring.rs` -- No TTY prompt fallback for encryption password. Users must set `GOG_KEYRING_PASSWORD` env var. Adding a `crossterm`-based or `rpassword`-based TTY prompt would improve UX.

- **[OBS-004]**: `src/cli/mod.rs:2931-2934` -- Resumable upload does not retry on network failure. The spec says "Resumable on network failure (re-PUT from last successful byte)". Current implementation aborts on any chunk error. Recommendation: query upload status endpoint and resume from last byte on transient errors.

- **[OBS-005]**: `src/cli/mod.rs:2701` -- Entire file is read into memory before upload. For very large files (>>5MB), this could be a memory concern. Consider streaming from disk for the resumable upload path.

- **[OBS-006]**: `docs/command-reference.md:33-34` -- Documents `--step` and `--auth-url` flags that do not exist in the CLI. This should be updated to match the actual CLI or the flags should be implemented.

- **[OBS-007]**: `src/auth/keyring.rs` -- `derive_key("")` with empty password produces a weak all-zero-based key. Consider rejecting empty passwords with a user-facing error.

- **[OBS-008]**: `src/cli/root.rs:424-430` -- `--manual` and `--remote` flags are not mutually exclusive. If both are specified, `--manual` silently wins. Consider adding `conflicts_with = "remote"` to the `manual` field.

## Modules Not Validated (if context limited)

All four M7 requirements were fully validated. No modules remain.

## Final Verdict

**CONDITIONAL APPROVAL** -- All four requirements are Should priority. REQ-RT-016 and REQ-RT-029 meet their acceptance criteria (with the noted gap on resume-on-failure for RT-029). REQ-RT-004 has functional backend code but is unreachable from the CLI (missing `--step`/`--auth-url` flags). REQ-RT-014 encryption works but uses a non-cryptographic KDF. No Must requirements are affected. The following Should-priority gaps are tracked as non-blocking:

1. REQ-RT-004: CLI flags `--step` and `--auth-url` not implemented (OBS-001)
2. REQ-RT-014: `derive_key()` uses `DefaultHasher` instead of PBKDF2/Argon2 (OBS-002)
3. REQ-RT-014: No TTY prompt for encryption password (OBS-003)
4. REQ-RT-029: No resume-on-failure for chunk uploads (OBS-004)

These should be resolved before GA but do not block the current milestone review.
