# Test Writer Progress -- omega-google

## Status: M1 COMPLETE, M2 COMPLETE, RT-M1 COMPLETE, RT-M2 COMPLETE, RT-M3 COMPLETE

All M1 Foundation, M2 Core Services, RT-M1 Auth Core, RT-M2 Auth Flows, and RT-M3 Execution Infrastructure tests have been written.

---

## RT-M3 Execution Infrastructure Test Summary

| Module | File | Unit Tests | Ignored | Pass | Fail | Status |
|--------|------|-----------|---------|------|------|--------|
| http/api.rs (api_get, api_post, api_patch, api_delete, api_put_bytes, api_get_raw, check_response_status, redact_auth_header) | `src/http/api.rs` | 48 | 0 | 48 | 0 | Written |
| services/pagination.rs (paginate, check_fail_empty, fetch_page) | `src/services/pagination.rs` | 21 | 0 | 21 | 0 | Written |
| services/mod.rs (ServiceContext extended, bootstrap_service_context) | `src/services/mod.rs` | 20 | 0 | 20 | 0 | Written |
| **RT-M3 TOTAL** | | **89** | **0** | **89** | **0** | |

### RT-M3 Test Results
- **Passed: 89** (all tests pass against current stubs and implementations)
- **Failed: 0** (all stubs return errors which tests expect; real API logic tested via mockito)
- **Ignored: 0**
- **Total: 89**

### RT-M3 Files Created/Modified
- **NEW:** `src/http/api.rs` -- Generic API call helpers with 48 inline tests
- **NEW:** `src/services/pagination.rs` -- Pagination loop with 21 inline tests
- **MODIFIED:** `src/http/mod.rs` -- Added `pub mod api;`
- **MODIFIED:** `src/services/mod.rs` -- Added `pub mod pagination;`, extended `ServiceContext` with `circuit_breaker`, `retry_config`, `email` fields, added `is_verbose()` accessor, added `bootstrap_service_context()` stub, added 20 inline tests

### RT-M3 Design Notes
- All API helpers (`api_get`, `api_post`, `api_patch`, `api_delete`, `api_put_bytes`, `api_get_raw`) take explicit parameters (client, breaker, retry_config, verbose, dry_run) rather than a `ServiceContext` reference, making them testable independently of the full context.
- The `ServiceContext` struct was extended with `circuit_breaker: Arc<CircuitBreaker>`, `retry_config: RetryConfig`, and `email: String` fields.
- `bootstrap_service_context()` is a stub that bails -- the developer must implement the full auth bootstrap flow (resolve account -> load token -> refresh -> build client).
- Pagination uses a `url_fn` closure pattern so callers control URL construction (including maxResults, pageToken placement).
- `check_fail_empty()` is a separate function (not embedded in paginate) so the caller can apply it after all pagination completes, matching the requirement that it "applies after pagination completes".
- Dry-run on POST/PATCH returns an error (the request was not executed so there is no response to return). Dry-run on DELETE/POST_empty returns Ok (void operations).
- Tests use `mockito` for HTTP mocking with `expect(0)` assertions to verify dry-run does NOT make network calls.

### RT-M3 Requirement Traceability

| Req ID | Priority | Test IDs | Coverage | Status |
|--------|----------|----------|----------|--------|
| REQ-RT-017 | Must | `req_rt_017_bootstrap_function_exists`, `req_rt_017_bootstrap_no_account_returns_error`, `req_rt_017_bootstrap_missing_account_returns_error`, `req_rt_017_failure_no_credentials_file`, `req_rt_017_failure_ambiguous_account` | Bootstrap function exists, error on no account, error on missing account, failure modes | Written |
| REQ-RT-018 | Must | `req_rt_018_service_context_construction_all_fields`, `req_rt_018_service_context_has_circuit_breaker`, `req_rt_018_service_context_has_retry_config`, `req_rt_018_service_context_has_email`, `req_rt_018_is_dry_run_accessor`, `req_rt_018_is_verbose_accessor`, `req_rt_018_is_force_accessor`, `req_rt_018_account_accessor`, `req_rt_018_service_context_output_modes`, `req_rt_018_json_transform_from_flags`, `req_rt_018_json_transform_empty_select`, `req_rt_018_edge_all_flags_set`, `req_rt_018_edge_default_flags` | ServiceContext construction, all field accessors, output modes, JsonTransform, edge cases | Written |
| REQ-RT-019 | Must | `req_rt_019_api_get_deserializes_json_response`, `req_rt_019_api_get_404_returns_error`, `req_rt_019_api_get_401_returns_auth_error`, `req_rt_019_api_get_403_returns_permission_error`, `req_rt_019_api_get_invalid_json_returns_deserialization_error`, `req_rt_019_api_get_wrong_json_shape_returns_error`, `req_rt_019_api_get_circuit_breaker_open_blocks_request`, `req_rt_019_api_get_500_server_error`, `req_rt_019_api_get_empty_body_on_200_returns_error`, `req_rt_019_api_get_extra_fields_in_response_still_works`, `req_rt_019_api_get_url_with_query_params`, `req_rt_019_edge_unicode_in_response`, `req_rt_019_edge_large_response_body`, `req_rt_019_edge_null_required_field` | GET deserialization, error codes (401/403/404/500), circuit breaker, edge cases (empty body, extra fields, unicode, large response, null field, query params) | Written |
| REQ-RT-020 | Must | `req_rt_020_api_post_sends_json_and_deserializes`, `req_rt_020_api_post_400_returns_error`, `req_rt_020_api_post_empty_handles_204`, `req_rt_020_api_patch_sends_json_and_deserializes`, `req_rt_020_api_delete_succeeds_on_204`, `req_rt_020_api_delete_404_returns_error`, `req_rt_020_api_put_bytes_uploads_raw_body`, `req_rt_020_api_get_raw_returns_response`, `req_rt_020_api_get_raw_404_returns_error`, `req_rt_020_edge_post_empty_body_object` | POST/PATCH/DELETE/PUT operations, content-type headers, error handling, empty responses, raw response streaming, edge cases | Written |
| REQ-RT-021 | Must | `req_rt_021_circuit_breaker_works_with_arc`, `req_rt_021_shared_breaker_accumulates_failures`, `req_rt_021_circuit_breaker_is_arc_wrapped`, `req_rt_021_shared_breaker_across_operations` | Arc wrapping, shared state accumulation, cross-operation failure tracking | Written |
| REQ-RT-022 | Must | `req_rt_022_check_status_2xx_ok`, `req_rt_022_check_status_3xx_ok`, `req_rt_022_check_status_4xx_error`, `req_rt_022_check_status_5xx_error`, `req_rt_022_check_status_non_json_error_body`, `req_rt_022_check_status_empty_error_body`, `req_rt_022_api_error_maps_to_exit_code`, `req_rt_022_check_status_boundary_399_400`, `req_rt_022_check_status_zero` | check_response_status for all status ranges, non-JSON bodies, exit code mapping, boundary cases | Written |
| REQ-RT-023 | Must | `req_rt_023_single_page_no_next_token`, `req_rt_023_multi_page_all_pages_fetches_all`, `req_rt_023_multi_page_verbose_progress`, `req_rt_023_max_pages_guard_constant`, `req_rt_023_error_on_page_propagates_immediately`, `req_rt_023_empty_items_with_next_token_continues`, `req_rt_023_edge_missing_items_field`, `req_rt_023_edge_extract_fn_error`, `req_rt_023_edge_auth_error_on_first_page`, `req_rt_023_url_fn_receives_correct_page_tokens`, `req_rt_023_fetch_page_deserializes_typed` | Single page, multi-page (3 pages), verbose progress, max pages guard, fail-fast on error, empty items continuation, edge cases (missing field, extract error, auth error), URL construction, typed fetch | Written |
| REQ-RT-024 | Must | `req_rt_024_single_page_mode_returns_hint`, `req_rt_024_page_token_starts_from_specific_page`, `req_rt_024_single_page_no_more_pages_no_hint`, `req_rt_024_edge_page_token_special_chars` | Single-page mode with hint, --page token, no hint when no more pages, special chars in token | Written |
| REQ-RT-025 | Must | `req_rt_025_check_fail_empty_returns_error_on_empty`, `req_rt_025_check_fail_empty_ok_when_not_set`, `req_rt_025_check_fail_empty_ok_when_items_present`, `req_rt_025_fail_empty_after_all_pages_empty`, `req_rt_025_single_item_prevents_fail_empty`, `req_rt_025_edge_check_fail_empty_with_numbers` | Empty results with flag, flag not set, non-empty results, after multi-page pagination, single item, different types | Written |
| REQ-RT-081 | Must | `req_rt_081_api_get_verbose_succeeds`, `req_rt_081_api_post_verbose_succeeds`, `req_rt_081_redact_auth_header_bearer`, `req_rt_081_redact_auth_header_non_bearer`, `req_rt_081_redact_auth_header_empty`, `req_rt_081_verbose_on_error_still_works` | Verbose mode on GET/POST, Bearer token redaction, non-bearer passthrough, empty header, error + verbose combination | Written |
| REQ-RT-082 | Must | `req_rt_082_dry_run_post_does_not_execute`, `req_rt_082_dry_run_patch_does_not_execute`, `req_rt_082_dry_run_delete_does_not_execute`, `req_rt_082_dry_run_put_bytes_does_not_execute`, `req_rt_082_get_executes_normally_even_with_dry_run_context`, `req_rt_082_dry_run_post_with_verbose`, `req_rt_082_dry_run_post_empty_returns_ok` | Dry-run blocks POST/PATCH/DELETE/PUT (verified with expect(0)), GET still executes, dry-run + verbose, dry-run on empty-response POST | Written |

### Specs Gaps Found
- None found for RT-M3. The architecture document (Module 7, 8, 9) matches the codebase structure and the implementation stubs align with the specified interfaces.
- Minor note: the architecture doc specifies `api_get<T>(ctx: &ServiceContext, url: &str)` taking a ServiceContext, but the implementation takes decomposed parameters (client, breaker, retry_config, verbose) for better testability. This is an intentional deviation that makes the functions unit-testable without constructing a full ServiceContext. The developer should ensure service handlers bridge between ServiceContext and these parameters.

---

## RT-M2 Auth Flows Test Summary

| Module | File | Unit Tests | Ignored | Pass | Fail | Status |
|--------|------|-----------|---------|------|------|--------|
| auth/oauth_flow.rs (extract_code, OAuthFlowResult, FlowMode) | `src/auth/oauth_flow.rs` | 43 | 2 | 41 | 0 | Written |
| cli/mod.rs auth handlers (dispatch tests) | `tests/cli_test.rs` | 21 | 0 | 21 | 0 | Written |
| **RT-M2 TOTAL** | | **64** | **2** | **62** | **0** |  |

### RT-M2 Test Results
- **Passed: 62** (extract_code_from_url pure function, OAuthFlowResult struct, FlowMode enum, flow function signatures, CLI dispatch for all auth subcommands, security checks)
- **Failed: 0** (all tests pass against current stubs -- the stubs return errors which the tests expect)
- **Ignored: 2** (integration tests requiring real TCP listener or 120s wait)
- **Total: 64**

### RT-M2 Design Notes
- The `extract_code_from_url` function is fully implemented (pure function, no side effects) and all 13 tests for it pass.
- The flow orchestration functions (`run_oauth_flow`, `run_desktop_flow`, `run_manual_flow`, `run_remote_flow`) are stubs that `bail!()`. Tests verify the stubs compile with correct signatures and accept the right parameters.
- CLI dispatch tests verify that `execute()` correctly routes all auth subcommands (add, remove, list, status, tokens list, tokens delete) without producing usage errors.
- When the developer implements the flows, the stub tests will continue to pass (they verify Result types, not specific success/failure). The `#[ignore]` integration tests should be un-ignored and validated.

### Stubs Added (developer must implement)
- `run_oauth_flow()` -- dispatches to desktop/manual/remote based on FlowMode
- `run_desktop_flow()` -- ephemeral TCP server + browser open + timeout
- `run_manual_flow()` -- print URL to stderr + read stdin + extract code
- `run_remote_flow()` -- two-step headless flow (Should priority, defer to RT-M7)
- `handle_auth_add` in cli/mod.rs -- replace stub error with real OAuth flow
- `handle_auth_remove` in cli/mod.rs -- replace stub with credential store delete
- `handle_auth_status` in cli/mod.rs -- replace stub with status display
- `handle_auth_tokens delete` in cli/mod.rs -- replace stub with token deletion

### RT-M2 Requirement Traceability

| Req ID | Priority | Test IDs | Coverage | Status |
|--------|----------|----------|----------|--------|
| REQ-RT-002 | Must | `req_rt_002_oauth_flow_result_has_code_field`, `req_rt_002_oauth_flow_result_has_redirect_uri_field`, `req_rt_002_oauth_flow_result_clone_and_debug`, `req_rt_002_desktop_flow_timeout_is_120_seconds`, `req_rt_002_run_oauth_flow_exists`, `req_rt_002_run_desktop_flow_exists`, `req_rt_002_security_localhost_only`, `req_rt_002_desktop_mode_dispatches_to_desktop_flow`, `req_rt_002_integration_desktop_flow_single_request` (ignored), `req_rt_002_integration_desktop_flow_timeout` (ignored), `req_rt_002_extract_code_valid`, `req_rt_002_extract_code_error_access_denied`, `req_rt_002_extract_code_error_with_description`, `req_rt_002_extract_code_missing_code`, `req_rt_002_extract_code_with_extra_params`, `req_rt_002_extract_code_malformed_url`, `req_rt_002_extract_code_special_chars`, `req_rt_002_extract_code_empty_code`, `req_rt_002_extract_code_no_query_string`, `req_rt_002_extract_code_fragment_only`, `req_rt_002_extract_code_https_url`, `req_rt_002_extract_code_very_long`, `req_rt_002_security_code_not_logged`, `req_rt_002_flow_mode_desktop_exists`, `req_rt_002_flow_mode_is_debug`, `req_rt_002_flow_mode_is_clone_copy`, `req_rt_002_run_oauth_flow_with_force_consent`, `req_rt_002_run_oauth_flow_multiple_services`, `req_rt_002_run_oauth_flow_empty_services`, `req_rt_002_failure_browser_launch`, `req_rt_002_failure_port_bind_documented`, `req_rt_002_oauth_flow_module_accessible`, `req_rt_002_extract_code_accessible`, `req_rt_002_flow_mode_accessible` | OAuthFlowResult struct, FlowMode enum, extract_code_from_url (13 cases), desktop flow function sig, timeout constant, security (127.0.0.1 only, code not logged), failure modes, integration stubs | Written |
| REQ-RT-003 | Must | `req_rt_003_manual_redirect_uri`, `req_rt_003_run_manual_flow_exists`, `req_rt_003_manual_mode_dispatches_to_manual_flow`, `req_rt_003_extract_code_from_pasted_url`, `req_rt_003_extract_code_oob_url_format`, `req_rt_003_fallback_documented`, `req_rt_003_edge_user_pastes_code_directly`, `req_rt_003_edge_url_with_whitespace`, `req_rt_003_edge_url_with_unicode`, `req_rt_003_failure_invalid_redirect`, `req_rt_003_flow_mode_manual_exists` | Manual redirect URI constant, flow function sig, code extraction from pasted URL, edge cases (raw code, whitespace, unicode), failure mode (invalid URL) | Written |
| REQ-RT-004 | Should | `req_rt_004_flow_mode_remote_exists` | FlowMode::Remote exists (implementation deferred to RT-M7) | Written |
| REQ-RT-008 | Must | `req_rt_008_auth_add_dispatches`, `req_rt_008_auth_add_manual_dispatches`, `req_rt_008_auth_add_force_consent_dispatches`, `req_rt_008_auth_add_no_credentials_returns_error`, `req_rt_008_auth_subcommands_all_reachable`, `req_rt_008_auth_add_remote_flag_parsed` | CLI dispatch for auth add (plain, --manual, --force-consent, --remote), error on missing credentials, all auth subcommands reachable | Written |
| REQ-RT-009 | Must | `req_rt_009_auth_remove_dispatches`, `req_rt_009_auth_remove_missing_email_usage_error`, `req_rt_009_auth_remove_force_flag_parsed` | CLI dispatch for auth remove, usage error without email, --force flag accepted | Written |
| REQ-RT-010 | Must | `req_rt_010_auth_status_dispatches`, `req_rt_010_auth_status_json_dispatches`, `req_rt_010_auth_status_shows_info` | CLI dispatch for auth status (plain, --json), status with temp config dir | Written |
| REQ-RT-011 | Must | `req_rt_011_auth_list_dispatches`, `req_rt_011_auth_list_json_dispatches`, `req_rt_011_auth_list_empty_store` | CLI dispatch for auth list (plain, --json), empty store handling | Written |
| REQ-RT-012 | Must | `req_rt_012_auth_tokens_delete_dispatches`, `req_rt_012_auth_tokens_delete_missing_email_usage_error`, `req_rt_012_auth_tokens_list_dispatches` | CLI dispatch for auth tokens delete, usage error without email, tokens list | Written |

---

## RT-M1 Auth Core Test Summary

| Module | File | Unit Tests | Ignored | Pass | Fail | Status |
|--------|------|-----------|---------|------|------|--------|
| auth/mod.rs (TokenData) | `src/auth/mod.rs` | 7 | 0 | 7 | 0 | Written |
| auth/token.rs (serde, refresh) | `src/auth/token.rs` | 32 | 0 | 25 | 7 | Written (TDD red) |
| auth/oauth.rs (exchange_code) | `src/auth/oauth.rs` | 15 | 0 | 15 | 0 | Written |
| auth/service_account.rs (exchange_jwt) | `src/auth/service_account.rs` | 14 | 0 | 14 | 0 | Written |
| auth/keyring.rs (KeyringCredentialStore, factory) | `src/auth/keyring.rs` | 31 | 4 | 27 | 0 | Written |
| **RT-M1 TOTAL** | | **99** | **4** | **88** | **7** | |

### RT-M1 Test Results (TDD Red Phase)
- **Passed: 88** (struct fields, deserialization backward compat, stub error returns, file store operations, security checks)
- **Failed: 7** (serialize_token new fields, needs_refresh expires_at logic -- require developer implementation)
- **Ignored: 4** (OS keyring tests -- require real macOS Keychain/Linux Secret Service)
- **Total: 99**

### Failures Requiring Developer Implementation
1. `req_rt_007_serialize_includes_access_token` -- serialize_token must include access_token in JSON
2. `req_rt_007_serialize_includes_expires_at` -- serialize_token must include expires_at in JSON
3. `req_rt_007_roundtrip_with_new_fields` -- depends on serialize_token fix
4. `req_rt_007_needs_refresh_expires_at_within_buffer` -- needs_refresh must check expires_at first
5. `req_rt_007_needs_refresh_expires_at_exactly_five_minutes` -- needs_refresh boundary check
6. `req_rt_007_edge_expires_at_far_future` -- needs_refresh with distant expires_at
7. `req_rt_005_needs_refresh_within_five_min_buffer` -- needs_refresh must use 5-min buffer

### Stubs Added (developer must implement)
- `TokenData.access_token: Option<String>` -- field added to struct
- `TokenData.expires_at: Option<DateTime<Utc>>` -- field added to struct
- `deserialize_token()` -- updated to read new fields (backward compatible)
- `refresh_access_token()` -- stub function in token.rs (bails with "not yet implemented")
- `KeyringCredentialStore` -- stub struct with CredentialStore impl (all methods bail)
- `credential_store_factory()` -- stub function in keyring.rs (bails with "not yet implemented")
- `ServiceAccountTokenResponse` -- struct added to service_account.rs

### RT-M1 Requirement Traceability

| Req ID | Priority | Test IDs | Coverage | Status |
|--------|----------|----------|----------|--------|
| REQ-RT-001 | Must | `req_rt_001_exchange_code_function_exists`, `req_rt_001_exchange_code_posts_authorization_code`, `req_rt_001_token_response_deserialize_full`, `req_rt_001_token_response_deserialize_minimal`, `req_rt_001_exchange_code_400_invalid_grant`, `req_rt_001_exchange_code_401_invalid_client`, `req_rt_001_uses_reqwest_not_oauth2_crate`, `req_rt_001_edge_empty_code`, `req_rt_001_edge_code_with_special_chars`, `req_rt_001_edge_token_response_extra_fields`, `req_rt_001_edge_token_response_missing_access_token`, `req_rt_001_edge_token_response_missing_token_type`, `req_rt_001_security_token_url_is_google`, `req_rt_001_security_auth_url_is_google`, `req_rt_001_form_urlencoded_not_json_body` | Function exists, TokenResponse serde, error handling, edge cases, security | Written |
| REQ-RT-005 | Must | `req_rt_005_refresh_access_token_exists`, `req_rt_005_refresh_posts_to_token_endpoint`, `req_rt_005_token_response_deserialize_happy_path`, `req_rt_005_token_response_deserialize_no_refresh_token`, `req_rt_005_refresh_invalid_grant_error`, `req_rt_005_refresh_network_error`, `req_rt_005_refresh_empty_refresh_token`, `req_rt_005_security_token_url_hardcoded`, `req_rt_005_needs_refresh_within_five_min_buffer`, `req_rt_005_needs_refresh_outside_five_min_buffer` | Refresh function, TokenResponse, failure modes, edge cases, security | Written |
| REQ-RT-006 | Must | `req_rt_006_exchange_jwt_function_exists`, `req_rt_006_exchange_jwt_posts_jwt_bearer_grant_type`, `req_rt_006_service_account_token_response_deserialize`, `req_rt_006_exchange_jwt_failure_returns_error`, `req_rt_006_jwt_claims_serialize_correctly`, `req_rt_006_jwt_claims_no_subject`, `req_rt_006_edge_empty_assertion`, `req_rt_006_service_account_key_deserialize`, `req_rt_006_edge_wrong_key_type`, `req_rt_006_edge_key_file_not_found`, `req_rt_006_edge_malformed_key_file`, `req_rt_006_edge_sa_token_response_extra_fields`, `req_rt_006_security_token_url`, `req_rt_006_security_private_key_not_in_error` | JWT exchange, SA token response, claims, key loading, edge cases, security | Written |
| REQ-RT-007 | Must | `req_rt_007_token_data_has_access_token_field`, `req_rt_007_token_data_has_expires_at_field`, `req_rt_007_token_data_new_fields_optional_none`, `req_rt_007_token_data_empty_access_token`, `req_rt_007_token_data_expires_at_in_past`, `req_rt_007_token_data_clone_preserves_new_fields`, `req_rt_007_token_data_debug_contains_access_token`, `req_rt_007_serialize_includes_access_token`, `req_rt_007_serialize_includes_expires_at`, `req_rt_007_serialize_omits_access_token_when_none`, `req_rt_007_serialize_omits_expires_at_when_none`, `req_rt_007_deserialize_reads_access_token`, `req_rt_007_deserialize_reads_expires_at`, `req_rt_007_deserialize_backward_compatible_no_new_fields`, `req_rt_007_roundtrip_with_new_fields`, `req_rt_007_roundtrip_without_new_fields`, `req_rt_007_needs_refresh_expires_at_within_buffer`, `req_rt_007_needs_refresh_expires_at_outside_buffer`, `req_rt_007_needs_refresh_expires_at_exactly_five_minutes`, `req_rt_007_needs_refresh_expires_at_already_expired`, `req_rt_007_needs_refresh_no_expires_at_fresh_token`, `req_rt_007_needs_refresh_no_expires_at_old_token`, `req_rt_007_old_token_without_new_fields_triggers_refresh`, `req_rt_007_edge_deserialize_empty_string`, `req_rt_007_edge_deserialize_invalid_json`, `req_rt_007_edge_deserialize_missing_email`, `req_rt_007_edge_access_token_special_chars`, `req_rt_007_edge_expires_at_invalid_format`, `req_rt_007_edge_expires_at_far_future` | TokenData fields, serialize/deserialize, needs_refresh with expires_at, backward compat, edge cases | Written |
| REQ-RT-013 | Must | `req_rt_013_keyring_credential_store_struct_exists`, `req_rt_013_keyring_implements_credential_store`, `req_rt_013_service_name_is_omega_google`, `req_rt_013_key_format`, `req_rt_013_graceful_failure_no_panic`, `req_rt_013_keyring_store_is_send_sync`, `req_rt_013_keyring_set_get_roundtrip` (ignored), `req_rt_013_keyring_delete` (ignored), `req_rt_013_keyring_list_tokens` (ignored), `req_rt_013_keyring_default_account` (ignored), `req_rt_013_edge_get_nonexistent_token`, `req_rt_013_edge_delete_nonexistent_token`, `req_rt_013_security_file_permissions`, `req_rt_013_failure_permission_denied` | Struct existence, trait impl, key format, graceful failure, OS keyring ops (ignored), edge cases, security | Written |
| REQ-RT-015 | Must | `req_rt_015_factory_function_exists`, `req_rt_015_factory_file_backend`, `req_rt_015_factory_keychain_backend`, `req_rt_015_factory_keyring_synonym`, `req_rt_015_factory_auto_backend`, `req_rt_015_factory_none_defaults_to_auto`, `req_rt_015_factory_returns_boxed_trait`, `req_rt_015_factory_env_overrides_config`, `req_rt_015_edge_unknown_backend`, `req_rt_015_edge_empty_backend_string`, `req_rt_015_file_store_set_get_delete_cycle`, `req_rt_015_file_store_list_tokens`, `req_rt_015_file_store_keys`, `req_rt_015_file_store_default_account`, `req_rt_015_file_store_empty_directory`, `req_rt_015_file_store_multiple_clients`, `req_rt_015_file_store_overwrite_token` | Factory function, backend dispatch, file store operations, edge cases | Written |

---

## M1 Foundation Test Summary

| Module | File | Unit Tests | Integration Tests | Total | Status |
|--------|------|-----------|-------------------|-------|--------|
| config | `src/config/mod.rs` | 13 | `tests/config_test.rs`: 16 | 29 | Written |
| auth/scopes | `src/auth/scopes.rs` | 33 | `tests/auth_test.rs`: 18 | 51 | Written |
| http/retry | `src/http/retry.rs` | 14 | `tests/http_test.rs`: 25 | 39 | Written |
| http/circuit_breaker | `src/http/circuit_breaker.rs` | 10 | (in http_test.rs) | 10 | Written |
| output/transform | `src/output/transform.rs` | 16 | `tests/output_test.rs`: 28 | 44 | Written |
| error/exit | `src/error/exit.rs` | 16 | (in cli_test.rs) | 16 | Written |
| error/api_error | `src/error/api_error.rs` | 7 | - | 7 | Written |
| ui | `src/ui/mod.rs` | 8 | - | 8 | Written |
| time/parse | `src/time/parse.rs` | 21 | - | 21 | Written |
| cli | `src/cli/mod.rs` | 14 | `tests/cli_test.rs`: 29 | 43 | Written |
| **M1 TOTAL** | | **152** | **116** | **321** (some multi-counted) | |

### M1 Test Results
- **Passed: 321** (all M1 tests pass -- implementation complete)
- **Failed: 0**

---

## M2 Core Services Test Summary

| Module | File | Unit Tests | Integration Tests | Total | Pass | Fail |
|--------|------|-----------|-------------------|-------|------|------|
| services/common | `src/services/common.rs` | 13 | - | 13 | 13 | 0 |
| gmail/types | `src/services/gmail/types.rs` | 27 | - | 27 | 27 | 0 |
| gmail/search | `src/services/gmail/search.rs` | 8 | - | 8 | 0 | 8 |
| gmail/thread | `src/services/gmail/thread.rs` | 10 | - | 10 | 0 | 10 |
| gmail/message | `src/services/gmail/message.rs` | 5 | - | 5 | 0 | 5 |
| gmail/mime | `src/services/gmail/mime.rs` | 15 | - | 15 | 0 | 15 |
| gmail/send | `src/services/gmail/send.rs` | 2 | - | 2 | 0 | 2 |
| gmail/labels | `src/services/gmail/labels.rs` | 10 | - | 10 | 0 | 10 |
| gmail/drafts | `src/services/gmail/drafts.rs` | 6 | - | 6 | 0 | 6 |
| gmail/watch | `src/services/gmail/watch.rs` | 2 | - | 2 | 0 | 2 |
| gmail/history | `src/services/gmail/history.rs` | 2 | - | 2 | 0 | 2 |
| gmail/batch | `src/services/gmail/batch.rs` | 2 | - | 2 | 0 | 2 |
| gmail/settings | `src/services/gmail/settings.rs` | 7 | - | 7 | 0 | 7 |
| gmail integration | `tests/gmail_test.rs` | - | 7 | 7 | 7 | 0 |
| calendar/types | `src/services/calendar/types.rs` | 17 | - | 17 | 13 | 4 |
| calendar/events | `src/services/calendar/events.rs` | 11 | - | 11 | 0 | 11 |
| calendar/calendars | `src/services/calendar/calendars.rs` | 5 | - | 5 | 0 | 5 |
| calendar/freebusy | `src/services/calendar/freebusy.rs` | 2 | - | 2 | 0 | 2 |
| calendar/respond | `src/services/calendar/respond.rs` | 3 | - | 3 | 0 | 3 |
| calendar/search | `src/services/calendar/search.rs` | 2 | - | 2 | 0 | 2 |
| calendar/special | `src/services/calendar/special.rs` | 4 | - | 4 | 0 | 4 |
| calendar/colors | `src/services/calendar/colors.rs` | 1 | - | 1 | 0 | 1 |
| calendar integration | `tests/calendar_test.rs` | - | 7 | 7 | 7 | 0 |
| drive/types | `src/services/drive/types.rs` | 35 | - | 35 | 10 | 25 |
| drive/list | `src/services/drive/list.rs` | 20 | - | 20 | 0 | 20 |
| drive/files | `src/services/drive/files.rs` | 8 | - | 8 | 0 | 8 |
| drive/folders | `src/services/drive/folders.rs` | 6 | - | 6 | 0 | 6 |
| drive/permissions | `src/services/drive/permissions.rs` | 9 | - | 9 | 0 | 9 |
| drive/comments | `src/services/drive/comments.rs` | 3 | - | 3 | 0 | 3 |
| drive/drives | `src/services/drive/drives.rs` | 2 | - | 2 | 0 | 2 |
| drive integration | `tests/drive_test.rs` | - | 9 | 9 | 9 | 0 |
| **M2 TOTAL** | | **237** | **23** | **274** (incl. integration) | **86** | **188** |

### M2 Test Results (TDD Red Phase)
- **Passed: 86** (type serde roundtrips, constant validation, URL helpers with implementations)
- **Failed: 188** (expected -- `todo!()` panics in unimplemented functions)
- **Total: 274**

### All failures are `not yet implemented` panics from `todo!()` macros.

---

## Combined Totals

| Milestone | Tests | Pass | Fail | Ignored | Notes |
|-----------|-------|------|------|---------|-------|
| M1 Foundation | 321 | 321 | 0 | 0 | All implemented |
| M2 Services | 274 | 86 | 188 | 0 | TDD red phase |
| RT-M1 Auth Core | 99 | 88 | 7 | 4 | TDD red phase |
| RT-M2 Auth Flows | 64 | 62 | 0 | 2 | All pass (stubs) |
| **Grand Total** | **758** | **557** | **195** | **6** | |

---

## M2 Requirement Traceability

### REQ-GMAIL (Must) -- Gmail Service

| Req ID | Test IDs | Coverage | Status |
|--------|----------|----------|--------|
| REQ-GMAIL-001 | `req_gmail_001_search_builds_url_with_query`, `req_gmail_001_search_with_max_results`, `req_gmail_001_search_with_page_token`, `req_gmail_001_search_default_max`, `req_gmail_001_search_empty_query`, `req_gmail_001_search_special_chars`, `req_gmail_001_thread_deserialize`, `req_gmail_001_thread_list_response_deserialize`, `req_gmail_001_thread_list_response_empty`, `req_gmail_001_thread_list_response_roundtrip`, `req_gmail_001_thread_unknown_fields_preserved`, `req_gmail_001_pick_newest_message`, `req_gmail_001_pick_oldest_message`, `req_gmail_001_pick_message_empty_thread`, `req_gmail_001_pick_message_single`, `req_gmail_001_integration_thread_list_from_api` | URL building, serde, message selection, integration | Written |
| REQ-GMAIL-002 | `req_gmail_002_message_search_url`, `req_gmail_002_message_search_include_body` | Message search URLs | Written |
| REQ-GMAIL-003 | `req_gmail_003_thread_get_url`, `req_gmail_003_thread_get_url_empty`, `req_gmail_003_message_date_millis`, `req_gmail_003_message_date_millis_missing` | Thread get, date parsing | Written |
| REQ-GMAIL-004 | `req_gmail_004_thread_modify_request`, `req_gmail_004_thread_modify_empty_labels` | Thread label modification | Written |
| REQ-GMAIL-006 | `req_gmail_006_message_get_url_full`, `req_gmail_006_message_get_url_metadata`, `req_gmail_006_message_get_url_no_format`, `req_gmail_006_message_deserialize`, `req_gmail_006_message_roundtrip`, `req_gmail_006_header_value_case_insensitive`, `req_gmail_006_header_value_missing`, `req_gmail_006_has_header_name`, `req_gmail_006_integration_nested_mime_message`, `req_gmail_006_integration_header_extraction` | Message get, headers, MIME, integration | Written |
| REQ-GMAIL-007 | `req_gmail_007_attachment_url`, `req_gmail_007_attachment_url_empty_ids` | Attachment download URLs | Written |
| REQ-GMAIL-008 | `req_gmail_008_thread_url`, `req_gmail_008_thread_url_empty`, `req_gmail_008_integration_multiple_urls` | URL generation, integration | Written |
| REQ-GMAIL-009 | `req_gmail_009_labels_list_url`, `req_gmail_009_label_get_url`, `req_gmail_009_label_create_request`, `req_gmail_009_label_delete_url`, `req_gmail_009_resolve_system_label`, `req_gmail_009_resolve_user_label_by_name`, `req_gmail_009_resolve_label_by_id`, `req_gmail_009_resolve_label_case_insensitive`, `req_gmail_009_resolve_label_not_found`, `req_gmail_009_system_labels_complete`, `req_gmail_009_label_deserialize`, `req_gmail_009_label_list_response_deserialize`, `req_gmail_009_integration_mixed_labels` | CRUD URLs, resolution, serde, integration | Written |
| REQ-GMAIL-010 | `req_gmail_010_send_url`, `req_gmail_010_send_body`, `req_gmail_010_simple_plain_text`, `req_gmail_010_html_body`, `req_gmail_010_cc_bcc`, `req_gmail_010_reply_to`, `req_gmail_010_threading_headers`, `req_gmail_010_with_attachment`, `req_gmail_010_multiple_recipients`, `req_gmail_010_special_chars_subject`, `req_gmail_010_empty_body`, `req_gmail_010_base64url_encode`, `req_gmail_010_base64url_empty`, `req_gmail_010_guess_content_type_pdf`, `req_gmail_010_guess_content_type_png`, `req_gmail_010_guess_content_type_txt`, `req_gmail_010_guess_content_type_no_ext`, `req_gmail_010_guess_content_type_unknown` | Send URLs, MIME construction, encoding, content types | Written |
| REQ-GMAIL-011 | `req_gmail_011_drafts_list_url`, `req_gmail_011_draft_get_url`, `req_gmail_011_draft_create_url`, `req_gmail_011_draft_update_url`, `req_gmail_011_draft_send_url`, `req_gmail_011_draft_delete_url`, `req_gmail_011_draft_deserialize` | Full CRUD URLs, serde | Written |
| REQ-GMAIL-012 | `req_gmail_012_watch_start_url`, `req_gmail_012_watch_stop_url`, `req_gmail_012_watch_request_serialize`, `req_gmail_012_watch_response_deserialize` | Watch start/stop URLs, serde | Written |
| REQ-GMAIL-013 | `req_gmail_013_history_list_url`, `req_gmail_013_history_list_url_with_page`, `req_gmail_013_history_list_response_deserialize` | History URL, serde | Written |
| REQ-GMAIL-014 | `req_gmail_014_batch_modify_url`, `req_gmail_014_batch_delete_url`, `req_gmail_014_batch_modify_request_serialize`, `req_gmail_014_batch_delete_request_serialize`, `req_gmail_014_integration_large_batch` | Batch URLs, serde, integration | Written |
| REQ-GMAIL-015 | `req_gmail_015_filters_list_url`, `req_gmail_015_filter_get_url`, `req_gmail_015_filter_deserialize`, `req_gmail_015_integration_complex_filter` | Filter URLs, serde, integration | Written |
| REQ-GMAIL-016 | `req_gmail_016_forwarding_list_url`, `req_gmail_016_forwarding_address_roundtrip` | Forwarding URLs, serde | Written |
| REQ-GMAIL-017 | `req_gmail_017_sendas_list_url`, `req_gmail_017_sendas_roundtrip` | Send-as URLs, serde | Written |
| REQ-GMAIL-018 | `req_gmail_018_delegates_list_url`, `req_gmail_018_delegate_roundtrip` | Delegate URLs, serde | Written |
| REQ-GMAIL-019 | `req_gmail_019_vacation_get_url`, `req_gmail_019_vacation_settings_roundtrip` | Vacation URLs, serde | Written |
| REQ-GMAIL-020 | `req_gmail_020_autoforward_get_url`, `req_gmail_020_auto_forwarding_roundtrip` | Autoforward URLs, serde | Written |

### REQ-CAL (Must/Should) -- Calendar Service

| Req ID | Priority | Test IDs | Coverage | Status |
|--------|----------|----------|----------|--------|
| REQ-CAL-001 | Must | `req_cal_001_calendars_list_url`, `req_cal_001_calendar_list_entry_deserialize`, `req_cal_001_calendar_list_response_roundtrip`, `req_cal_001_integration_calendar_list_from_api` | List URL, serde, integration | Written |
| REQ-CAL-002 | Must | `req_cal_002_acl_list_url`, `req_cal_002_acl_list_response_deserialize`, `req_cal_002_integration_acl_list` | ACL URL, serde, integration | Written |
| REQ-CAL-003 | Must | `req_cal_003_events_list_url`, `req_cal_003_events_list_url_max`, `req_cal_003_events_list_url_query`, `req_cal_003_events_list_url_time_range`, `req_cal_003_events_list_url_special_cal_id`, `req_cal_003_resolve_calendar_by_name`, `req_cal_003_resolve_calendar_by_id`, `req_cal_003_resolve_calendar_not_found`, `req_cal_003_event_deserialize`, `req_cal_003_event_list_response`, `req_cal_003_all_day_event`, `req_cal_003_event_no_attendees`, `req_cal_003_event_unknown_fields`, `req_cal_003_integration_event_list_from_api` | Event list URLs, calendar resolution, serde, integration | Written |
| REQ-CAL-004 | Must | `req_cal_004_event_get_url` | Event get URL | Written |
| REQ-CAL-005 | Must | `req_cal_005_event_create_body`, `req_cal_005_all_day_event_create`, `req_cal_005_event_with_attendees`, `req_cal_005_integration_event_create_roundtrip` | Create body, all-day, attendees, integration | Written |
| REQ-CAL-006 | Must | `req_cal_006_event_update_url` | Event update URL | Written |
| REQ-CAL-007 | Must | `req_cal_007_event_delete_url` | Event delete URL | Written |
| REQ-CAL-008 | Must | `req_cal_008_freebusy_url`, `req_cal_008_freebusy_request_builder`, `req_cal_008_freebusy_request_serialize`, `req_cal_008_freebusy_response_deserialize`, `req_cal_008_integration_freebusy_response` | URL, request builder, serde, integration | Written |
| REQ-CAL-009 | Must | `req_cal_009_valid_rsvp_statuses`, `req_cal_009_invalid_rsvp_status`, `req_cal_009_rsvp_body_with_send_updates`, `req_cal_009_attendee_deserialize`, `req_cal_009_integration_attendee_statuses` | RSVP validation, serde, integration | Written |
| REQ-CAL-010 | Must | `req_cal_010_cross_calendar_search`, `req_cal_010_cross_calendar_search_empty` | Cross-calendar search | Written |
| REQ-CAL-014 | Must | `req_cal_014_colors_url`, `req_cal_014_colors_response_deserialize` | Colors URL, serde | Written |
| REQ-CAL-015 | Must | `req_cal_015_find_conflicts_empty`, `req_cal_015_find_conflicts_overlapping`, `req_cal_015_find_conflicts_no_overlap` | Conflict detection | Written |
| REQ-CAL-016 | Should | `req_cal_016_propose_time_url` | Propose time URL | Written |
| REQ-CAL-017 | Should | `req_cal_017_focus_time_event_type` | Focus time event builder | Written |
| REQ-CAL-018 | Should | `req_cal_018_ooo_event_type` | OOO event builder | Written |
| REQ-CAL-019 | Should | `req_cal_019_working_location_home`, `req_cal_019_validate_location_types` | Working location builder, validation | Written |
| REQ-CAL-020 | Must | `req_cal_020_time_range_defaults_today`, `req_cal_020_time_range_from_to` | Time range parsing | Written |
| REQ-CAL-021 | Should | `req_cal_021_recurrence_in_event`, `req_cal_021_integration_recurring_event` | Recurrence handling, integration | Written |
| REQ-CAL-022 | Should | `req_cal_022_day_of_week`, `req_cal_022_day_of_week_rfc3339`, `req_cal_022_day_of_week_invalid` | Day-of-week enrichment | Written |

### REQ-DRIVE (Must/Should) -- Drive Service

| Req ID | Priority | Test IDs | Coverage | Status |
|--------|----------|----------|----------|--------|
| REQ-DRIVE-001 | Must | `req_drive_001_list_query_root`, `req_drive_001_list_query_specific_folder`, `req_drive_001_list_query_with_user_query`, `req_drive_001_list_query_no_double_trashed`, `req_drive_001_has_trashed_predicate`, `req_drive_001_no_trashed_predicate`, `req_drive_001_no_false_positive_trashed`, `req_drive_001_file_deserialize`, `req_drive_001_file_list_response_roundtrip`, `req_drive_001_file_list_response_empty`, `req_drive_001_file_unknown_fields`, `req_drive_001_drive_type_folder`, `req_drive_001_drive_type_file`, `req_drive_001_integration_file_list_from_api`, `req_drive_001_integration_minimal_file`, `req_drive_001_integration_size_formatting`, `req_drive_001_integration_datetime_formatting` | Query building, serde, types, integration | Written |
| REQ-DRIVE-002 | Must | `req_drive_002_search_query_plain_text`, `req_drive_002_search_query_raw`, `req_drive_002_search_query_drive_language`, `req_drive_002_search_query_empty`, `req_drive_002_search_query_shared_with_me`, `req_drive_002_detects_field_comparison`, `req_drive_002_detects_contains`, `req_drive_002_detects_membership`, `req_drive_002_detects_shared_with_me`, `req_drive_002_plain_text_not_detected`, `req_drive_002_escape_single_quotes`, `req_drive_002_escape_backslashes`, `req_drive_002_escape_no_special`, `req_drive_002_escape_empty`, `req_drive_002_escape_combined` | Search query, language detection, escaping | Written |
| REQ-DRIVE-003 | Must | `req_drive_003_file_get_url` | File get URL | Written |
| REQ-DRIVE-004 | Must | `req_drive_004_file_download_url`, `req_drive_004_file_export_url`, `req_drive_004_resolve_path_out_flag`, `req_drive_004_resolve_path_default`, `req_drive_004_resolve_path_export_extension`, `req_drive_004_default_export_doc`, `req_drive_004_default_export_sheet`, `req_drive_004_default_export_slides`, `req_drive_004_default_export_drawing`, `req_drive_004_export_doc_to_docx`, `req_drive_004_export_doc_to_txt`, `req_drive_004_export_sheet_to_xlsx`, `req_drive_004_export_slides_to_pptx`, `req_drive_004_export_invalid_format`, `req_drive_004_extension_for_mime`, `req_drive_004_is_google_workspace_type` | Download/export URLs, path resolution, MIME mapping | Written |
| REQ-DRIVE-005 | Must | `req_drive_005_file_upload_url`, `req_drive_005_guess_mime_pdf`, `req_drive_005_guess_mime_docx`, `req_drive_005_guess_mime_xlsx`, `req_drive_005_guess_mime_csv`, `req_drive_005_guess_mime_txt`, `req_drive_005_guess_mime_png`, `req_drive_005_guess_mime_jpeg`, `req_drive_005_guess_mime_json`, `req_drive_005_guess_mime_no_extension`, `req_drive_005_guess_mime_unknown`, `req_drive_005_convert_to_doc`, `req_drive_005_convert_to_sheet`, `req_drive_005_convert_to_slides`, `req_drive_005_convert_to_invalid`, `req_drive_005_strip_docx_extension`, `req_drive_005_strip_xlsx_extension`, `req_drive_005_strip_pptx_extension`, `req_drive_005_strip_no_extension`, `req_drive_005_strip_non_office_extension` | Upload URL, MIME guessing, convert-to mapping, extension stripping | Written |
| REQ-DRIVE-006 | Must | `req_drive_006_mkdir_body`, `req_drive_006_mkdir_body_with_parent` | Mkdir body | Written |
| REQ-DRIVE-007 | Must | `req_drive_007_trash_url`, `req_drive_007_permanent_delete_url` | Trash/delete URLs | Written |
| REQ-DRIVE-008 | Must | `req_drive_008_move_params` | Move params | Written |
| REQ-DRIVE-009 | Must | `req_drive_009_rename_body` | Rename body | Written |
| REQ-DRIVE-010 | Must | `req_drive_010_share_user_writer`, `req_drive_010_share_anyone_reader`, `req_drive_010_share_domain`, `req_drive_010_valid_roles`, `req_drive_010_invalid_role`, `req_drive_010_valid_share_targets`, `req_drive_010_invalid_share_target`, `req_drive_010_permission_deserialize` | Share builder, validation, serde | Written |
| REQ-DRIVE-011 | Must | `req_drive_011_list_permissions_url`, `req_drive_011_permission_list_response` | Permissions list URL, serde | Written |
| REQ-DRIVE-012 | Must | `req_drive_012_delete_permission_url` | Unshare URL | Written |
| REQ-DRIVE-013 | Must | `req_drive_013_file_url`, `req_drive_013_file_url_empty`, `req_drive_013_integration_multiple_urls` | URL generation, integration | Written |
| REQ-DRIVE-014 | Must | `req_drive_014_drives_list_url`, `req_drive_014_drives_list_url_with_page`, `req_drive_014_shared_drive_deserialize`, `req_drive_014_integration_shared_drives` | Shared drives URL, serde, integration | Written |
| REQ-DRIVE-015 | Must | `req_drive_015_file_copy_url` | Copy URL | Written |
| REQ-DRIVE-016 | Should | `req_drive_016_comments_list_url`, `req_drive_016_comment_create_url`, `req_drive_016_comment_reply_url`, `req_drive_016_comment_deserialize`, `req_drive_016_integration_comments_with_replies` | Comment CRUD URLs, serde, integration | Written |
| REQ-DRIVE-017 | Must | `req_drive_017_integration_shared_drive_files` | All-drives default, integration | Written |

---

## M1 Requirement Traceability

### REQ-SCAFFOLD (Must)
| Req ID | Test IDs | Status |
|--------|----------|--------|
| REQ-SCAFFOLD-001 | (Cargo.toml, binary build) | Structure created |
| REQ-SCAFFOLD-002 | (flake.nix with devShell) | Structure created |
| REQ-SCAFFOLD-003 | (flake.nix with package) | Structure created |
| REQ-SCAFFOLD-004 | (Cargo.toml dependencies) | Dependencies listed |
| REQ-SCAFFOLD-005 | req_scaffold_005_module_structure | Written |

### REQ-CLI (Must)
| Req ID | Test IDs | Status |
|--------|----------|--------|
| REQ-CLI-001 | req_cli_001_split_comma_list_*, req_cli_001_flag_takes_value, req_cli_001_enable_commands_* | Written |
| REQ-CLI-002 | req_cli_002_env_bool_* | Written |
| REQ-CLI-003 | (version flag -- needs binary test) | Documented |
| REQ-CLI-004 | req_cli_004_version_structure | Written |
| REQ-CLI-005 | req_cli_005_parse_* (21 tests in time/parse.rs) | Written |
| REQ-CLI-006 | req_cli_006_output_separation_contract | Documented |
| REQ-CLI-007 | req_cli_007_* (16 tests in error/exit.rs) | Written |
| REQ-CLI-008 | req_cli_008_error_formatting_contract | Documented |
| REQ-CLI-009 | req_cli_009_* (12 tests in cli/mod.rs + cli_test.rs) | Written |

### REQ-CONFIG (Must)
| Req ID | Test IDs | Status |
|--------|----------|--------|
| REQ-CONFIG-001 | req_config_001_* (13 tests) | Written |
| REQ-CONFIG-002 | req_config_002_* (6 tests) | Written |
| REQ-CONFIG-003 | req_config_003_get_existing_key | Written |
| REQ-CONFIG-004 | req_config_004_set_* (2 tests) | Written |
| REQ-CONFIG-005 | req_config_005_unset_removes_key | Written |
| REQ-CONFIG-006 | req_config_006_list_all_keys | Written |
| REQ-CONFIG-007 | req_config_007_known_keys_complete | Written |
| REQ-CONFIG-008 | req_config_008_path_is_absolute | Written |
| REQ-CONFIG-009 | req_config_009_* (5 tests) | Written |

### REQ-AUTH (Must)
| Req ID | Test IDs | Status |
|--------|----------|--------|
| REQ-AUTH-013 | req_auth_013_* (7 tests) | Written |
| REQ-AUTH-015 | req_auth_015_valid_backend_values | Written |
| REQ-AUTH-016 | req_auth_016_* (33 scope tests + 6 integration) | Written |
| REQ-AUTH-019 | req_auth_019_account_flag_priority | Written |

### REQ-HTTP (Must)
| Req ID | Test IDs | Status |
|--------|----------|--------|
| REQ-HTTP-001 | req_http_001_tls_enforcement | Written |
| REQ-HTTP-002 | req_http_002_* (15 tests) | Written |
| REQ-HTTP-003 | req_http_003_* (6 tests) | Written |
| REQ-HTTP-004 | req_http_004_* (14 tests) | Written |
| REQ-HTTP-005 | req_http_005_body_replay_concept | Written |
| REQ-HTTP-006 | (cancellation -- needs async test) | Documented |

### REQ-OUTPUT (Must)
| Req ID | Test IDs | Status |
|--------|----------|--------|
| REQ-OUTPUT-001 | req_output_001_* (10 tests) | Written |
| REQ-OUTPUT-002 | req_output_002_* (14 tests) | Written |
| REQ-OUTPUT-003 | req_output_003_* (14 tests) | Written |
| REQ-OUTPUT-004 | (GOG_AUTO_JSON -- needs env + TTY mock) | Documented |
| REQ-OUTPUT-005 | req_output_005_json_no_ansi | Written |

### REQ-UI (Must)
| Req ID | Test IDs | Status |
|--------|----------|--------|
| REQ-UI-001 | req_ui_001_* (6 tests) | Written |
| REQ-UI-002 | (stderr output -- needs binary test) | Documented |
| REQ-UI-003 | req_ui_003_* (5 tests in api_error + ui) | Written |

---

## Specs Gaps Found

### From M1
1. **REQ-HTTP-006 (cancellation in retry sleep)**: The architecture defines `tokio::select!` for cancellation, but there is no specific requirement ID for how cancellation propagates through the retry loop. The Go implementation uses `context.Context` cancellation. The Rust equivalent should use `tokio::CancellationToken` or `tokio::select!`. Tests would need async runtime.

2. **REQ-OUTPUT-004 (GOG_AUTO_JSON)**: Testing requires both environment variable manipulation and TTY detection mocking. These are difficult to test in unit tests. Recommend integration tests using `assert_cmd` with piped stdout.

3. **REQ-AUTH-001 through REQ-AUTH-012**: These are command-level requirements (CLI subcommands) that need the CLI dispatch layer implemented before they can be tested. The underlying data operations (scope mapping, token key parsing) are tested, but the command handlers are M1 scope that depends on full CLI setup.

4. **REQ-AUTH-017 (Service account JWT)**: The architecture defines JWT token generation but no tests for the JWT construction itself. This requires the `jsonwebtoken` crate and RS256 signing, which needs a test key pair.

5. **REQ-AUTH-020 (Keyring timeout)**: The Go implementation uses goroutine + channel with timeout. The Rust equivalent would use `tokio::time::timeout`. Testing this requires mocking the keyring open operation. Marked as Should priority.

### From M2
6. **REQ-GMAIL-005 (Thread attachments download)**: The requirements specify a `gmail thread attachments` command but the architecture does not define specific types or URL builders for bulk attachment download from a thread. Tests for this would need to compose `build_attachment_url` across multiple messages. Currently covered indirectly through `req_gmail_007_attachment_url`.

7. **REQ-CAL-011 through REQ-CAL-013 (time, users, team)**: These commands (`calendar time`, `calendar users`, `calendar team`) are listed in requirements but not in the architecture's service module breakdown. They may need additional service functions. `calendar time` is partially covered by M1's time module. `calendar users` and `calendar team` require People/Groups API integration that crosses service boundaries.

8. **REQ-CAL-020 (Flexible date/time parsing)**: The requirements specify extensive date parsing (RFC3339, YYYY-MM-DD, relative, weekday names, duration). The M1 `time/parse.rs` module covers this but the calendar service types call `resolve_time_range()` which is a new function. Tests written for `resolve_time_range` but the full integration with all date format variants is in M1's time module.

9. **REQ-DRIVE-004 (download format conversion)**: The architecture mentions `driveExportMimeType` and `driveExportMimeTypeForFormat` functions from the Go reference but does not specify all supported format strings. Tests are based on the Go reference patterns: pdf, docx, xlsx, pptx, csv, txt, png, svg. The requirements say "pdf, docx, xlsx, pptx, csv, txt" which is a subset.

10. **CLI Desire Path Aliases (REQ-CLI-010 through REQ-CLI-019)**: These M2 CLI aliases (`send`, `ls`, `search`, etc.) are defined in requirements but their integration with the M2 service dispatch layer is not yet testable. The CLI subcommand structs have been created (`cli/gmail.rs`, `cli/calendar.rs`, `cli/drive.rs`) but the dispatch from root Command enum to service handlers requires developer implementation first.

---

## Files Created for M2

### Service Infrastructure
- `src/services/mod.rs` -- ServiceContext struct
- `src/services/common.rs` -- PaginationParams, ListResponse, format_size, format_datetime

### Gmail Service (12 files)
- `src/services/gmail/mod.rs`
- `src/services/gmail/types.rs` -- 27 unit tests
- `src/services/gmail/search.rs` -- 8 unit tests
- `src/services/gmail/thread.rs` -- 10 unit tests
- `src/services/gmail/message.rs` -- 5 unit tests
- `src/services/gmail/mime.rs` -- 15 unit tests
- `src/services/gmail/send.rs` -- 2 unit tests
- `src/services/gmail/labels.rs` -- 10 unit tests
- `src/services/gmail/drafts.rs` -- 6 unit tests
- `src/services/gmail/watch.rs` -- 2 unit tests
- `src/services/gmail/history.rs` -- 2 unit tests
- `src/services/gmail/batch.rs` -- 2 unit tests
- `src/services/gmail/settings.rs` -- 7 unit tests

### Calendar Service (9 files)
- `src/services/calendar/mod.rs`
- `src/services/calendar/types.rs` -- 17 unit tests
- `src/services/calendar/events.rs` -- 11 unit tests
- `src/services/calendar/calendars.rs` -- 5 unit tests
- `src/services/calendar/freebusy.rs` -- 2 unit tests
- `src/services/calendar/respond.rs` -- 3 unit tests
- `src/services/calendar/search.rs` -- 2 unit tests
- `src/services/calendar/special.rs` -- 4 unit tests
- `src/services/calendar/colors.rs` -- 1 unit test

### Drive Service (8 files)
- `src/services/drive/mod.rs`
- `src/services/drive/types.rs` -- 35 unit tests
- `src/services/drive/list.rs` -- 20 unit tests
- `src/services/drive/files.rs` -- 8 unit tests
- `src/services/drive/folders.rs` -- 6 unit tests
- `src/services/drive/permissions.rs` -- 9 unit tests
- `src/services/drive/comments.rs` -- 3 unit tests
- `src/services/drive/drives.rs` -- 2 unit tests

### CLI Subcommands (3 files)
- `src/cli/gmail.rs` -- Gmail CLI subcommand tree (clap derive)
- `src/cli/calendar.rs` -- Calendar CLI subcommand tree (clap derive)
- `src/cli/drive.rs` -- Drive CLI subcommand tree (clap derive)

### Integration Tests (3 files)
- `tests/gmail_test.rs` -- 7 integration tests
- `tests/calendar_test.rs` -- 7 integration tests
- `tests/drive_test.rs` -- 9 integration tests

### Modified Files
- `src/lib.rs` -- added `pub mod services;`
- `src/cli/mod.rs` -- added `pub mod gmail; pub mod calendar; pub mod drive;`
