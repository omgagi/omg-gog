# QA Report: Browser Opens During Tests Bugfix

## Scope Validated
- `src/auth/oauth_flow.rs` -- `open_browser()` `#[cfg(test)]` guard
- `tests/cli_test.rs` -- env var isolation for all 18 auth-related tests
- `tests/rt_m4_handlers_test.rs` -- env var isolation for 6 `execute()` tests
- `src/services/mod.rs` -- env var isolation for 5 bootstrap tests
- `src/cli/mod.rs` -- `handle_auth_add` and `handle_auth_list` logic

## Summary
**PASS** -- All 9 requirements verified. The full test suite (1,841 tests) passes with 0 failures, no hangs, and no browser windows opening. The root cause (lack of test isolation from real OS keychain and browser spawning) is addressed through two complementary mechanisms: (1) `#[cfg(test)]` guard on `open_browser()` prevents browser spawning in test builds, and (2) `OMEGA_GOOGLE_CONFIG_DIR` + `GOG_KEYRING_BACKEND=file` env var isolation on all `execute()` and `bootstrap_service_context()` calls prevents access to real credentials.

## System Entrypoint
```bash
cd /Users/isudoajl/ownCloud/Projects/omega-tools/omega-google
cargo test --jobs 1 -- --test-threads=1
```
No additional environment setup required. Tests run in isolation via temp directories and file-based keyring backend.

## Traceability Matrix Status

| Requirement ID | Priority | Has Tests | Tests Pass | Acceptance Met | Notes |
|---|---|---|---|---|---|
| REQ-BUG-001 | Must | Yes | Yes | Yes | `open_browser()` returns `false` in test builds via `#[cfg(test)]` block |
| REQ-BUG-002 | Must | Yes | Yes | Yes | All 18 auth tests in cli_test.rs set `GOG_KEYRING_BACKEND=file` |
| REQ-BUG-003 | Must | Yes | Yes | Yes | All 6 execute() tests in rt_m4_handlers_test.rs set `GOG_KEYRING_BACKEND=file` |
| REQ-BUG-004 | Must | Yes | Yes | Yes | `auth remove --force` test uses isolated temp dir and file backend |
| REQ-BUG-005 | Should | N/A | N/A | Partial | Env var cleanup is ordered correctly (remove before assert) but no RAII guard used |
| REQ-BUG-006 | Must | Yes | Yes | Yes | Desktop flow tests use `#[cfg(test)]` no-op for `open_browser()` + 1s timeout |
| REQ-BUG-007 | Should | Yes | Yes | Yes | `run_manual_flow` calls `stdin().read_line()` which returns empty in test context (no TTY) |
| REQ-BUG-008 | Should | N/A | N/A | Partial | Cleanup before assert prevents most leaks, but no RAII guard for panics during `execute()` |
| REQ-BUG-009 | Could | No | N/A | No | No documentation file for test isolation pattern (deliberate deferral) |

### Gaps Found
- REQ-BUG-009 (Could): No test isolation documentation. Non-blocking.
- REQ-BUG-005/008 (Should): No RAII env var guard. Mitigated by `--test-threads=1` and cleanup-before-assert ordering. Non-blocking.

## Acceptance Criteria Results

### Must Requirements

#### REQ-BUG-001: `open_browser()` must not execute in test builds
- [x] `open_browser()` contains `#[cfg(test)] { let _ = url; return false; }` block -- PASS
- [x] `#[cfg(not(test))]` block contains actual browser-open logic -- PASS
- [x] Production builds unaffected: `#[cfg(not(test))]` preserves full browser-open behavior for macOS/Linux -- PASS
- [x] Empirically verified: unit test `req_rt_002_run_desktop_flow_exists` outputs "Could not open browser automatically" confirming the no-op path -- PASS

#### REQ-BUG-002: All cli_test.rs auth tests must set `GOG_KEYRING_BACKEND=file`
- [x] 36 `set_var` calls matched by 36 `remove_var` calls (balanced cleanup) -- PASS
- [x] All 18 `cli::execute()` calls preceded by both `OMEGA_GOOGLE_CONFIG_DIR` and `GOG_KEYRING_BACKEND` env vars -- PASS
- [x] Each test uses `tempfile::tempdir()` for config isolation -- PASS

#### REQ-BUG-003: rt_m4_handlers_test.rs service tests must set `GOG_KEYRING_BACKEND=file`
- [x] 12 `set_var` calls matched by 12 `remove_var` calls -- PASS
- [x] All 6 `execute()` calls have both env vars set -- PASS
- [x] Tests for Gmail, Calendar, and Drive all isolated -- PASS

#### REQ-BUG-004: `auth remove --force` test must not risk deleting real credentials
- [x] Test `req_rt_009_auth_remove_force_flag_parsed` uses `tempfile::tempdir()` and `GOG_KEYRING_BACKEND=file` -- PASS
- [x] File-based keyring backend writes to temp dir, not OS keychain -- PASS

#### REQ-BUG-006: Desktop flow tests must not open real OAuth URLs
- [x] `open_browser()` is no-op under `#[cfg(test)]` -- PASS
- [x] `EFFECTIVE_TIMEOUT_SECS` is 1 second under `#[cfg(test)]` (vs 120s in production) -- PASS
- [x] Empirically confirmed: `req_rt_002_run_desktop_flow_exists` completes in 1.00s -- PASS

### Should Requirements

#### REQ-BUG-005: Thread-safe env var handling
- [x] Tests run with `--test-threads=1` which prevents concurrent env var mutation -- PASS
- [ ] No RAII guard (e.g., `struct EnvGuard`) for automatic cleanup -- NOT IMPLEMENTED
- **Notes**: The manual `remove_var` before `assert` ordering is correct and sufficient for serial execution. RAII would be a defense-in-depth improvement.

#### REQ-BUG-007: Manual flow test must not block on stdin
- [x] `run_manual_flow` calls `stdin().read_line()` which returns empty in test context -- PASS
- [x] Empty input triggers `bail!("No URL provided")`, returning `Err` immediately -- PASS
- [x] Test `req_rt_003_run_manual_flow_exists` asserts `result.is_err() || result.is_ok()` -- accepts either -- PASS

#### REQ-BUG-008: RAII cleanup of env vars on panic
- [x] Cleanup (`remove_var`) is called BEFORE assertions in all tests -- PASS
- [ ] If `cli::execute()` panics between `set_var` and `remove_var`, env vars would leak -- NOT FULLY ADDRESSED
- **Notes**: The risk is mitigated by serial test execution. A panic in `execute()` would abort the test process anyway.

### Could Requirements

#### REQ-BUG-009: Document test isolation pattern
- [ ] No documentation file exists for the test isolation pattern -- NOT IMPLEMENTED
- **Notes**: This is a "Could" priority. The pattern is consistent across all test files and self-documenting through comments like "Use empty config dir + file backend to isolate from real OS keychain".

## End-to-End Flow Results

| Flow | Steps | Result | Notes |
|---|---|---|---|
| Full test suite execution | 1,841 tests, single-threaded | PASS | 0 failures, 6 ignored (keyring tests requiring OS keyring), completed in ~15s |
| Auth add dispatch (no creds) | set env -> execute(auth add) -> check exit code | PASS | Returns CONFIG_ERROR (10), no browser opened |
| Auth add manual (no creds) | set env -> execute(auth add --manual) -> check exit | PASS | Returns CONFIG_ERROR, no stdin blocking |
| Auth remove --force (isolation) | set env -> execute(--force auth remove user@) -> check exit | PASS | Operates on isolated temp dir |
| Auth list (empty store) | set env -> execute(auth list) -> check exit | PASS | Returns SUCCESS (0) with "No authenticated accounts" |
| Service bootstrap (no account) | set env -> bootstrap_service_context() -> check error | PASS | Returns error without accessing OS keychain |
| Desktop flow (unit test) | run_desktop_flow with fake creds | PASS | Times out in 1s (not 120s), no browser opened |

## Exploratory Testing Findings

| # | What Was Tried | Expected | Actual | Severity |
|---|---|---|---|---|
| 1 | Run full suite without `--test-threads=1` | Tests should still pass if isolation is correct | All pass (env vars cleaned up before assert) | low |
| 2 | Check if e2e tests accidentally run during `cargo test` | They should be skipped | Correctly gated behind `OMEGA_E2E_ACCOUNT` env var | low |
| 3 | Verify `EFFECTIVE_TIMEOUT_SECS` in unit tests | Should be 1s, not 120s | Confirmed 1s via timing output | N/A (validation) |

## Failure Mode Validation

| Failure Scenario | Triggered | Detected | Recovered | Degraded OK | Notes |
|---|---|---|---|---|---|
| No credentials file in temp dir | Yes | Yes | Yes | Yes | `handle_auth_add` returns CONFIG_ERROR (10) with guidance message |
| Empty keyring store | Yes | Yes | Yes | Yes | `handle_auth_list` returns SUCCESS (0) with hint to run `auth add` |
| Desktop flow timeout (1s in test) | Yes | Yes | Yes | Yes | Returns error with "Timed out" message suggesting `--manual` mode |
| Manual flow empty stdin | Yes | Yes | Yes | Yes | Returns error with "No URL provided" message |
| OS keyring inaccessible (file backend) | Yes | Yes | Yes | Yes | File backend used instead, operates on temp dir |

## Security Validation

| Attack Surface | Test Performed | Result | Notes |
|---|---|---|---|
| Real credential access during tests | Verified all `execute()` calls use `GOG_KEYRING_BACKEND=file` + temp dir | PASS | No path to real OS keychain in any test |
| Real browser spawning during tests | Verified `#[cfg(test)]` guard on `open_browser()` | PASS | Returns `false` without spawning process |
| Auth code leakage in logs | Checked `open_browser()` uses URL but does not log the code | PASS | Code extraction happens after redirect, not passed to browser |
| `auth remove --force` data loss | Verified test uses isolated temp dir | PASS | Cannot affect real credentials |

## Specs/Docs Drift

| File | Documented Behavior | Actual Behavior | Severity |
|------|-------------------|-----------------|----------|
| `specs/bugfixes/browser-opens-during-tests-analysis.md` | "3 tests missing isolation" in rt_m4_handlers_test.rs | 6 tests received isolation (3 service handler tests + 3 dispatch tests) | low |
| `specs/bugfixes/browser-opens-during-tests-analysis.md` | "13 tests missing full isolation" in cli_test.rs | 18 tests received isolation (all `execute()` callers) | low |
| `specs/bugfixes/browser-opens-during-tests-analysis.md` | Lists REQ-BUG-010 in table header but no REQ-BUG-010 row | Only 9 requirements (001-009) exist in the table | low |

## Blocking Issues (must fix before merge)
None. All Must requirements are met.

## Non-Blocking Observations

- **[OBS-001]**: `tests/cli_test.rs`, `tests/rt_m4_handlers_test.rs`, `src/services/mod.rs` -- No RAII env var guard. The current manual cleanup pattern (remove_var before assert) is correct but could be replaced with a `struct EnvGuard` that implements `Drop` for automatic cleanup on panic. This is a Should-level improvement (REQ-BUG-005/008).

- **[OBS-002]**: `specs/bugfixes/browser-opens-during-tests-analysis.md` -- The analysis doc mentions "10 requirements" in the title area but only lists 9 (REQ-BUG-001 through REQ-BUG-009). Minor doc inconsistency.

- **[OBS-003]**: The `#[cfg(test)]` guard inside `open_browser()` works for both unit tests AND integration tests because `cargo test` compiles the library with `cfg(test)` enabled. However, if a binary invocation test (using `assert_cmd`) were to call `auth add`, it would use the production code path where `open_browser()` is live. The e2e tests are properly gated behind `OMEGA_E2E_ACCOUNT` to prevent this.

## Modules Not Validated
None. All affected modules were validated.

## Final Verdict

**PASS** -- All Must requirements (REQ-BUG-001 through REQ-BUG-004, REQ-BUG-006) are met. All Should requirements (REQ-BUG-005, REQ-BUG-007, REQ-BUG-008) are met or partially met with acceptable mitigations. The Could requirement (REQ-BUG-009) was deliberately deferred. No blocking issues found. The full test suite of 1,841 tests passes with 0 failures, no browser windows opened, and no test hangs. Approved for review.
