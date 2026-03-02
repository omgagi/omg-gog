# QA Progress -- omega-google

## Status: RT-M1 COMPLETE, RT-M2 COMPLETE, RT-M3 COMPLETE, RT-M4 COMPLETE, RT-M5 COMPLETE, RT-M6 COMPLETE, RT-M7 COMPLETE, BUGFIX-BROWSER COMPLETE

### Bugfix: Browser Opens During Tests
- **Date**: 2026-03-02
- **Report**: `docs/qa/browser-opens-during-tests-qa-report.md`
- **Verdict**: PASS
- **Tests**: 1,841 total (1,409 lib + 432 integration), 0 failures, 6 ignored
- **Key findings**:
  - REQ-BUG-001: `open_browser()` guarded with `#[cfg(test)]` no-op -- PASS
  - REQ-BUG-002: All 18 cli_test.rs auth tests isolated with env vars -- PASS
  - REQ-BUG-003: All 6 rt_m4_handlers_test.rs execute() tests isolated -- PASS
  - REQ-BUG-004: `auth remove --force` test uses isolated temp dir -- PASS
  - REQ-BUG-005: Thread-safe env vars -- partial (manual cleanup, no RAII guard)
  - REQ-BUG-006: Desktop flow uses 1s timeout in tests -- PASS
  - REQ-BUG-007: Manual flow stdin returns empty in test context -- PASS
  - REQ-BUG-008: Cleanup before assert prevents most leaks -- partial
  - REQ-BUG-009: No documentation file (Could, deferred) -- not implemented
  - set_var/remove_var balanced in all files: 36/36, 12/12, 10/10

---

### RT-M7 Polish Features QA Validation
- **Date**: 2026-03-01
- **Report**: `docs/qa/rt-m7-qa-report.md`
- **Verdict**: CONDITIONAL APPROVAL
- **Tests**: 1814 total (1408 lib + 406 integration), 0 failures, 6 ignored
- **M7-specific tests**: 9 REQ-RT-004 + 11 REQ-RT-014 + 1 REQ-RT-016 + 3 REQ-RT-029 = 24 tests
- **Key findings**:
  - REQ-RT-004 (remote OAuth): Backend functions work (step1/step2 with state/CSRF) but CLI flags `--step` and `--auth-url` not implemented -- dead code from user perspective
  - REQ-RT-014 (encrypted file): AES-256-GCM encrypt/decrypt roundtrip works, random nonce per entry, credential_store_factory integration correct -- but KDF uses `DefaultHasher` (non-cryptographic, no salt, no work factor) instead of PBKDF2/Argon2; no TTY prompt for password
  - REQ-RT-016 (keyring timeout): Linux-specific 5s timeout with stderr hint correctly implemented -- PASS (verified by code inspection, not triggerable on macOS)
  - REQ-RT-029 (resumable upload): URL builder, threshold (5MB), chunk size (256KB), handler with 308 handling and progress -- PASS; but no resume-on-failure (upload aborted on network error)
  - docs/command-reference.md documents `--step` and `--auth-url` flags that do not exist -- docs drift

---

### RT-M6 Extended Service Handlers QA Validation
- **Date**: 2026-03-01
- **Report**: `docs/qa/rt-m6-qa-report.md`
- **Verdict**: CONDITIONAL APPROVAL
- **Tests**: 1385 lib (6 ignored) + all integration test binaries, 0 failures
- **M6-specific tests**: 10 appscript + 8 chat + 8 classroom + 9 contacts + 21 docs + 7 forms + 7 groups + 7 keep + 9 people + 8 sheets + 6 slides + 6 tasks = 106 integration tests
- **Key findings**:
  - All 12 services fully wired with async bootstrap/dispatch pattern -- PASS
  - All 12 services visible in CLI help output -- PASS
  - dispatch_command uses .await for all 12 services -- PASS
  - All sub-handlers use api_get/api_post/api_patch/api_delete with correct params -- PASS
  - All mutating operations pass ctx.is_dry_run() -- PASS
  - All handlers use URL builders from services/*/ (no inline URLs) -- PASS
  - ctx.write_output for single items, ctx.write_paginated for lists -- PASS
  - Bootstrap error handling inconsistent: 4 handlers use codes::AUTH_REQUIRED vs map_error_to_exit_code -- non-blocking, minor
  - 5 destructive operations missing force/no_input guard -- non-blocking, minor
  - Traceability table not backfilled with test IDs -- non-blocking, low

---

### RT-M5 File I/O QA Validation
- **Date**: 2026-03-01
- **Report**: `docs/qa/rt-m5-qa-report.md`
- **Verdict**: CONDITIONAL APPROVAL
- **Tests**: 1799 total (1380 lib + 419 integration), 0 failures, 6 ignored (auth-related), clippy clean
- **M5-specific tests**: 34 integration (rt_m5_fileio_test.rs) + 49 unit (services::export::tests) + file/type tests
- **Key findings**:
  - Drive binary download uses bounded-memory streaming (bytes_stream + chunked write) -- PASS
  - Drive export correctly URL-encodes MIME type in export URL -- PASS
  - Upload multipart body construction follows RFC 2046 -- PASS
  - Gmail base64url decode handles text, binary, and empty inputs -- PASS
  - Shared export module covers all Google Workspace types (Doc, Sheet, Slides, Drawing) -- PASS
  - api_post_bytes and api_get_raw properly integrate with retry middleware and circuit breaker -- PASS
  - --convert/--convert-to flags parsed but ignored in upload handler -- non-blocking, medium
  - Progress hint is post-completion only (no incremental during streaming) -- non-blocking, low
  - Partial download files not cleaned up on error -- non-blocking, low
  - Static multipart boundary string -- non-blocking, low

---

### RT-M3 Execution Infrastructure QA Validation
- **Date**: 2026-03-01
- **Report**: `docs/qa/rt-m3-qa-report.md`
- **Verdict**: PASS
- **Tests**: 1331 lib (6 ignored), 0 failures, clippy clean (0 warnings)
- **M3-specific tests**: 89 (48 api.rs + 21 pagination.rs + 20 mod.rs)
- **Key findings**:
  - All api_get/post/patch/delete use execute_with_retry with circuit breaker -- PASS
  - Verbose logging redacts Bearer tokens via redact_auth_header -- PASS
  - Dry-run blocks POST/PATCH/DELETE/PUT but allows GET (no dry_run param on api_get) -- PASS
  - Pagination fetches all pages with all_pages=true, single page by default -- PASS
  - check_fail_empty returns error on empty results (exit code 3 mapping exists) -- PASS
  - bootstrap_service_context returns error for missing account/token -- PASS (stub)
  - ServiceContext has all required fields (client, output_mode, json_transform, ui, flags, circuit_breaker, retry_config, email) -- PASS
  - Architecture spec divergence: api functions take individual params instead of ServiceContext (intentional, more testable) -- non-blocking observation
  - Verbose logging does not log request headers (only method, URL, body size, status) -- non-blocking, low
  - redact_auth_header exists but is not called in verbose code paths -- non-blocking, low

---

### RT-M2 Auth Flows QA Validation
- **Date**: 2026-03-01
- **Report**: `docs/qa/rt-m2-qa-report.md`
- **Verdict**: CONDITIONAL APPROVAL
- **Tests**: 1242 lib (6 ignored), 76 integration, 0 failures, clippy clean
- **Key findings**:
  - OAuth desktop flow binds to 127.0.0.1 only (security PASS)
  - extract_code_from_url handles all edge cases (13 test scenarios)
  - Manual flow uses correct OOB redirect URI
  - All auth CLI handlers dispatch and return correct exit codes
  - TokenData Debug redacts access_token and refresh_token
  - auth remove does not block for confirmation (non-blocking, medium)
  - auth add missing --services/--readonly/--drive-scope flags (non-blocking, medium)
  - auth status missing token-level detail fields (non-blocking, low)
  - auth list missing default account indicator (non-blocking, low)
  - 7 previously failing RT-M1 tests now pass (developer fixed)

---

### RT-M1 Auth Core QA Validation
- **Date**: 2026-03-01
- **Report**: `docs/qa/rt-m1-qa-report.md`
- **Verdict**: CONDITIONAL APPROVAL
- **Tests**: 95 passed, 0 failed, 4 ignored (OS keyring)
- **Clippy**: 0 warnings
- **Key findings**:
  - REQ-RT-007 (TokenData extension) fully implemented and correct
  - REQ-RT-001, RT-005, RT-006, RT-013, RT-015 have stubs with test infrastructure
  - exchange_code and exchange_jwt signatures do not match architecture
  - TokenData Debug output exposes sensitive tokens (latent risk)
  - FileCredentialStore is fully functional with proper security (0600 perms)
