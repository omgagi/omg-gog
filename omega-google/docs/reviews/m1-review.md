# Code Review: omega-google M1 Foundation

## Verdict: APPROVED WITH COMMENTS

2 Critical, 5 Major, 7 Minor, 5 Nit findings.

## Critical Findings

### C-1. Circuit breaker never recovers after cooldown
- **File:** `src/http/circuit_breaker.rs` lines 50-64
- **Problem:** `is_open()` checks if cooldown has elapsed but still returns `true`. No half-open state, so circuit stays open forever.
- **Requirement violated:** REQ-HTTP-004
- **Fix:** Implement half-open state: after cooldown, allow one probe request.

### C-2. Root flags missing env var bindings, short flags, aliases, and conflicts_with
- **File:** `src/cli/root.rs` lines 22-71
- **Problem:** Architecture spec defines many clap attributes (env, short, aliases, conflicts_with) not present in code.
- **Requirements violated:** REQ-CLI-001, REQ-CLI-002
- **Fix:** Add all clap attributes to match architecture spec.

## Major Findings

### M-1. `write_config_to` panics instead of returning Result
- **File:** `src/config/mod.rs` lines 94-118
- **Fix:** Return `anyhow::Result<()>` and use `?` operator.

### M-2. OsString arguments silently dropped on non-UTF-8
- **File:** `src/cli/mod.rs` lines 14-17
- **Fix:** Use `to_string_lossy()` or report error.

### M-3. Account resolution uses `OMEGA_GOOGLE_ACCOUNT` instead of spec's `GOG_ACCOUNT`
- **File:** `src/auth/mod.rs` lines 170-175, 194
- **Fix:** Change to `GOG_ACCOUNT`.

### M-4. Credential file naming doesn't match spec
- **File:** `src/config/mod.rs` lines 122-146
- **Fix:** Use `credentials.json` / `credentials-{client}.json`.

### M-5. `notes` in both META_KEYS and KNOWN_RESULT_KEYS
- **File:** `src/output/transform.rs` lines 15 and 34
- **Fix:** Remove `notes` from META_KEYS.

## Minor Findings

### m-1. 12+ `.unwrap()` on serde serialization in CLI dispatch
### m-2. Mutex `.unwrap()` in circuit breaker panics on poisoned lock
### m-3. `build_auth_url` omits `include_granted_scopes=true`
### m-4. `is_duration()` returns false positives
### m-5. Test calls to `write_json`/`write_plain` ignore Result
### m-6. `.and_hms_opt(0,0,0).unwrap()` should use `.expect()`
### m-7. `config set` only supports 2 of 5 config keys
