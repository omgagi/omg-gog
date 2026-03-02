# QA Report: RT-M6 Extended Service Handlers

## Scope Validated

Validation of 12 extended service handlers converted from sync stubs to async handlers with auth bootstrap and full subcommand dispatch: Docs, Sheets, Slides, Forms, Chat, Tasks, Classroom, Contacts, People, Groups, Keep, AppScript.

Requirements in scope: REQ-RT-069 through REQ-RT-080.

## Summary

**CONDITIONAL APPROVAL** -- All Must requirements are met (all Could and Should handlers are fully implemented and wired). The implementation follows the established pattern from RT-M4 with high fidelity. All 1,385 unit tests pass (6 ignored as expected for live-resource integration tests), the CLI compiles and shows all 12 services in help output, and every handler uses the async bootstrap/dispatch pattern. Two categories of non-blocking issues were found: (1) inconsistent error handling in the bootstrap block across 4 M6 handlers that use `codes::AUTH_REQUIRED` instead of `map_error_to_exit_code(&e)`, and (2) missing force/no-input guards on some destructive operations in Classroom and Docs.

## System Entrypoint

- **Build**: `cargo build` (completes successfully)
- **Run**: `./target/debug/omega-google --help` (all 12 services visible)
- **Test**: `cargo test --jobs 1` (1,385 passed, 0 failed, 6 ignored)

## Traceability Matrix Status

| Requirement ID | Priority | Has Tests | Tests Pass | Acceptance Met | Notes |
|---|---|---|---|---|---|
| REQ-RT-069 | Should | Yes | Yes | Yes | Docs: 16 subcommands wired (export, info, create, copy, cat, list-tabs, comments x6, write, insert, delete, find-replace, update, edit, sed, clear). Export uses shared Drive export logic. Tests in docs_test.rs. |
| REQ-RT-070 | Should | Yes | Yes | Yes | Sheets: 11 subcommands wired (get, update, append, insert, clear, format, notes, metadata, create, copy, export). A1 parsing used. Tests in sheets_test.rs (8 tests). |
| REQ-RT-071 | Should | Yes | Yes | Yes | Slides: 11 subcommands wired (export, info, create, create-from-markdown, copy, list-slides, add-slide, delete-slide, read-slide, update-notes, replace-slide). Export uses shared Drive export logic. Tests in slides_test.rs (6 tests). |
| REQ-RT-072 | Should | Yes | Yes | Yes | Forms: 4 subcommands wired (get, create, responses list, responses get). Uses existing 3 URL builders. Tests in forms_test.rs (7 tests). |
| REQ-RT-073 | Should | Yes | Yes | Yes | Chat: 7 subcommands wired across 4 groups (spaces list/find/create, messages list/send, threads list, dm space/send). Tests in chat_test.rs (8 tests). |
| REQ-RT-074 | Could | Yes | Yes | Yes | Classroom: 40+ subcommands wired across 11 groups (courses, students, teachers, roster, coursework, materials, submissions, announcements, topics, invitations, guardians, guardian-invitations, profile). Tests in classroom_test.rs (8 tests). |
| REQ-RT-075 | Should | Yes | Yes | Yes | Tasks: 9 subcommands wired (lists list/create, list, get, add, update, done, undo, delete, clear). Tests in tasks_test.rs (6 tests). |
| REQ-RT-076 | Should | Yes | Yes | Yes | Contacts: 10 subcommands wired (search, list, get, create, update, delete, directory list/search, other list/search). Tests in contacts_test.rs (9 tests). |
| REQ-RT-077 | Should | Yes | Yes | Yes | People: 4 subcommands wired (me, get, search, relations). Tests in people_test.rs (9 tests). |
| REQ-RT-078 | Could | Yes | Yes | Yes | Groups: 2 subcommands wired (list, members) with pagination support. Tests in groups_test.rs (7 tests). |
| REQ-RT-079 | Could | Yes | Yes | Yes | Keep: 4 subcommands wired (list, get, search, attachment). Client-side search implementation. Tests in keep_test.rs (7 tests). |
| REQ-RT-080 | Could | Yes | Yes | Yes | AppScript: 4 subcommands wired (get, content, run, create). Google ID normalization from URLs. Tests in appscript_test.rs (10 tests). |

### Gaps Found

- No requirement-to-test gap: all 12 requirements have corresponding test files with appropriate coverage.
- Traceability matrix in runtime-requirements.md shows "(filled by test-writer)" for REQ-RT-069, REQ-RT-070, REQ-RT-072, REQ-RT-073, REQ-RT-074, REQ-RT-075, REQ-RT-076, REQ-RT-077, REQ-RT-078, REQ-RT-079, REQ-RT-080 -- the test IDs were not backfilled into the traceability table, though tests do exist.

## Acceptance Criteria Results

### Should Requirements

#### REQ-RT-069: Docs service handlers
- [x] Wire up all Docs subcommands using existing URL/body builders -- PASS: 16 subcommands dispatched from `handle_docs`, all using builders from `services/docs/content`, `services/docs/edit`, `services/docs/export`, `services/docs/comments`, `services/docs/sedmat`
- [x] Export uses shared Drive export logic (REQ-RT-031) -- PASS: `handle_docs_export` imports from `services::drive::files::{build_file_export_url, build_file_get_url, resolve_download_path}`

#### REQ-RT-070: Sheets service handlers
- [x] Wire up all Sheets subcommands -- PASS: 11 subcommands dispatched, using builders from `services/sheets/read`, `services/sheets/write`, `services/sheets/format`, `services/sheets/structure`, `services/sheets/a1`
- [x] A1 notation parsing already exists -- PASS: `handle_sheets_format` calls `parse_a1`, `clean_range` used in get/update/append/clear/format/notes

#### REQ-RT-071: Slides service handlers
- [x] Wire up all Slides subcommands -- PASS: 11 subcommands dispatched, using builders from `services/slides/presentations`, `services/slides/export`, `services/slides/slides_ops`, `services/slides/notes`, `services/slides/markdown`
- [x] Export uses shared Drive export logic -- PASS: `handle_slides_export` imports from `services::drive::files::{build_file_export_url, build_file_get_url, resolve_download_path}`

#### REQ-RT-072: Forms service handlers
- [x] Wire up Forms subcommands -- PASS: 4 subcommands dispatched (get, create, responses list, responses get)
- [x] 3 URL builders exist -- PASS: uses `build_form_get_url`, `build_form_create_url`, `build_response_get_url`, plus `build_responses_list_url_with_options` and `build_form_create_body`

#### REQ-RT-073: Chat service handlers
- [x] Wire up Chat subcommands -- PASS: 7 subcommands across 4 groups, using builders from `services/chat/spaces`, `services/chat/messages`, `services/chat/threads`, `services/chat/dm`

#### REQ-RT-075: Tasks service handlers
- [x] Wire up all Tasks subcommands -- PASS: 9 subcommands dispatched, using builders from `services/tasks/tasklists`, `services/tasks/task_ops`
- [x] 11 URL builders exist -- PASS: all referenced in handler code

#### REQ-RT-076: Contacts service handlers
- [x] Wire up all Contacts subcommands -- PASS: 10 subcommands dispatched, using builders from `services/contacts/contacts`, `services/contacts/directory`
- [x] 12 URL builders exist -- PASS: all referenced in handler code

#### REQ-RT-077: People service handlers
- [x] Wire up People subcommands -- PASS: 4 subcommands dispatched (me, get, search, relations), using builders from `services/people/people`
- [x] 4 URL builders exist -- PASS: `build_people_me_url`, `build_people_get_url`, `build_people_search_url`, `build_people_relations_url`

### Could Requirements

#### REQ-RT-074: Classroom service handlers
- [x] Wire up all Classroom subcommands -- PASS: 40+ subcommands across 11 command groups
- [x] 61 URL builders exist across 9 modules -- PASS: uses builders from `services/classroom/courses`, `services/classroom/roster`, `services/classroom/coursework`, `services/classroom/materials`, `services/classroom/submissions`, `services/classroom/announcements`, `services/classroom/topics`, `services/classroom/invitations`, `services/classroom/guardians`

#### REQ-RT-078: Groups service handlers
- [x] Wire up Groups subcommands -- PASS: 2 subcommands with full pagination support using `services::pagination::paginate`
- [x] 3 URL builders exist -- PASS: `build_groups_list_url`, `build_group_lookup_url`, `build_members_list_url`

#### REQ-RT-079: Keep service handlers
- [x] Wire up Keep subcommands -- PASS: 4 subcommands (list, get, search, attachment) with full pagination support
- [x] 4 URL builders exist -- PASS: `build_notes_list_url`, `build_note_get_url`, `build_notes_search`, `build_attachment_download_url`

#### REQ-RT-080: Apps Script service handlers
- [x] Wire up Apps Script subcommands -- PASS: 4 subcommands (get, content, run, create) with Google ID normalization from various URL formats
- [x] 6 URL builders exist -- PASS: `build_project_get_url`, `build_content_get_url`, `build_run_url`, `build_run_body`, `build_project_create_url`, `build_project_create_body`

## Pattern Consistency Validation

### 1. Async Function Signature
All 12 service handlers use `async fn handle_<service>(args: ..., flags: &root::RootFlags) -> i32` -- PASS

### 2. Bootstrap Service Context
All 12 handlers call `crate::services::bootstrap_service_context(flags).await` at the top -- PASS

Note: Classroom has an offline early-return for the `courses url` subcommand before bootstrap (line 5622-5646). This is correct behavior -- URL generation does not require auth.

### 3. Dispatch Table with .await
All 12 services are dispatched with `.await` in `dispatch_command` (lines 144-155) -- PASS

### 4. Sub-handler API Method Usage
Verified across all handlers:
- Read operations use `api_get` -- PASS
- Create operations use `api_post` -- PASS
- Update operations use `api_patch` or `api_put_bytes` -- PASS
- Delete operations use `api_delete` -- PASS
- All mutating operations (api_post, api_patch, api_put_bytes) pass `ctx.is_dry_run()` -- PASS

### 5. Error Handling with map_error_to_exit_code
All sub-handler error branches use `map_error_to_exit_code(&e)` -- PASS

Bootstrap error handling is split (see finding below).

### 6. Output via ctx.write_output / ctx.write_paginated
- Single-item responses use `ctx.write_output(&result)` -- PASS
- List responses with pagination use `ctx.write_paginated(&result, next_token.as_deref())` -- PASS

### 7. URL Builder Usage (No Inline URLs)
Grep for `googleapis.com` in cli/mod.rs returns only one hit at line 575 (OAuth userinfo endpoint, pre-M6). All 12 M6 handlers use URL builders from `src/services/*/` -- PASS

### 8. Dry-Run Support
All mutating operations (create, update, delete, write, insert, find-replace, clear, copy, format, sed, edit) pass `ctx.is_dry_run()` to `api_post`, `api_patch`, `api_put_bytes`, or `api_delete` -- PASS

Manual dry-run gates (early return before API call) are present in export handlers (docs export, slides export, sheets export, slides create-from-markdown, keep search) -- PASS

### 9. Force/No-Input Guard on Delete Operations

**Services with proper force/no-input guard:**
- Tasks delete (line 7696) -- PASS
- Contacts delete (line 7931) -- PASS
- Classroom courses delete (line 5864) -- PASS
- Classroom coursework delete (line 6410) -- PASS
- Classroom materials delete (line 6592) -- PASS
- Classroom announcements delete (line 6896) -- PASS
- Classroom topics delete (line 7071) -- PASS
- Classroom guardians delete (line 7284) -- PASS

**Services missing force/no-input guard (non-blocking finding):**
- Docs comments delete (line 3401) -- no guard
- Classroom invitations delete (line 7205) -- no guard
- Classroom students remove (line 6073) -- no guard
- Classroom teachers remove (line 6174) -- no guard
- Slides delete-slide (line 4908) -- no guard (uses api_post with batchUpdate, not api_delete, but is still destructive)

## Test Results

```
Unit tests (src/lib.rs): 1385 passed, 0 failed, 6 ignored
Integration tests:
  appscript_test.rs:  10 passed
  chat_test.rs:        8 passed
  classroom_test.rs:   8 passed
  contacts_test.rs:    9 passed
  docs_test.rs:       21 passed
  forms_test.rs:       7 passed
  groups_test.rs:      7 passed
  keep_test.rs:        7 passed
  people_test.rs:      9 passed
  sheets_test.rs:      8 passed
  slides_test.rs:      6 passed
  tasks_test.rs:       6 passed

  cli_test.rs:        55 passed
  rt_m4_handlers_test: 113 passed
  rt_m5_fileio_test:   38 passed
  (and more from auth, config, drive, gmail, calendar, http, output)
```

All existing M1-M6 and RT-M1-M5 tests continue to pass. No regressions detected.

## Exploratory Testing Findings

| # | What Was Tried | Expected | Actual | Severity |
|---|---|---|---|---|
| 1 | Run `omega-google docs --help` | All subcommands listed | All 16 subcommands visible including sed, clear, write, insert | low (positive) |
| 2 | Run `omega-google chat --help` | Subcommand groups listed | 4 groups (spaces, messages, threads, dm) visible | low (positive) |
| 3 | Run `omega-google appscript --help` | All subcommands listed | 4 subcommands (get, content, run, create) visible | low (positive) |
| 4 | Run `omega-google classroom --help` | All subcommand groups listed | Groups visible including courses, students, teachers, roster, coursework, materials, submissions, announcements, topics, invitations, guardians, guardian-invitations, profile | low (positive) |
| 5 | Verify CLI shows all 12 services in main help | All 12 M6 services in help | Confirmed: docs, sheets, slides, forms, chat, classroom, tasks, contacts, people, groups, keep, appscript all present | low (positive) |

## Failure Mode Validation

| Failure Scenario | Triggered | Detected | Recovered | Degraded OK | Notes |
|---|---|---|---|---|---|
| Auth not configured (no token) | Not Triggered (requires keyring state) | N/A | N/A | N/A | Bootstrap returns error; handler returns exit code. Pattern validated via code inspection. |
| API returns 4xx/5xx | Not Triggered (requires live API) | N/A | N/A | N/A | All handlers use `map_error_to_exit_code(&e)` which maps API errors to appropriate exit codes. Pattern validated via code inspection and RT-M4 tests. |
| Dry-run mode | Verified via code inspection | Yes | N/A | Yes | All mutating operations check `ctx.is_dry_run()` and skip the API call, printing `[dry-run]` message instead. |

## Security Validation

| Attack Surface | Test Performed | Result | Notes |
|---|---|---|---|
| URL injection via CLI args | Code inspection: all user-supplied IDs go through URL builders that percent-encode | PASS | No raw string interpolation into URLs in handler code |
| Credential exposure in errors | Code inspection: error messages use `eprintln!("Error: {}", e)` with anyhow errors, not raw token data | PASS | No token/credential data in error output |
| Destructive operations without confirmation | Code inspection of delete handlers | PASS (with observations) | Most delete ops have force/no-input guards; 5 minor gaps noted as non-blocking |

## Specs/Docs Drift

| File | Documented Behavior | Actual Behavior | Severity |
|------|-------------------|-----------------|----------|
| specs/runtime-requirements.md | Traceability matrix for REQ-RT-069-080 says "(filled by test-writer)" for test IDs | Test files exist (e.g., `tests/chat_test.rs`, `tests/classroom_test.rs`) but test IDs not backfilled into the traceability table | low |
| specs/runtime-requirements.md | REQ-RT-069 lists subcommands as "export, create, copy, info, edit, comments" | Actual implementation includes 16 subcommands: export, info, create, copy, cat, list-tabs, comments (x6), write, insert, delete, find-replace, update, edit, sed, clear | low (positive drift -- more functionality than documented) |
| specs/runtime-requirements.md | REQ-RT-074 says "61 URL builders exist across 9 modules" | Classroom has 11 command groups (courses, students, teachers, roster, coursework, materials, submissions, announcements, topics, invitations, guardians, guardian-invitations, profile) which is more than the "9 modules" mentioned | low (positive drift) |

## Blocking Issues (must fix before merge)

None. All Must and Should requirements are met.

## Non-Blocking Observations

- **[OBS-001]**: `src/cli/mod.rs` lines 5166, 8073, 8217, 8377, 8609 -- Four M6 handlers (Forms, People, Groups, Keep, AppScript) use `codes::AUTH_REQUIRED` in the bootstrap error path instead of `map_error_to_exit_code(&e)`. This matches the pre-M6 pattern (Gmail, Calendar, Drive also use `codes::AUTH_REQUIRED`), but 7 other M6 handlers (Docs, Sheets, Slides, Chat, Classroom, Tasks, Contacts) use `map_error_to_exit_code(&e)` which is more general and correct for non-auth failures during bootstrap. Recommend standardizing all bootstrap error paths to `map_error_to_exit_code(&e)` in a follow-up.

- **[OBS-002]**: `src/cli/mod.rs` lines 3401 (Docs comments delete), 7205 (Classroom invitations delete), 6073 (Classroom students remove), 6174 (Classroom teachers remove), 4908 (Slides delete-slide) -- These destructive operations lack the `is_force()/no_input` guard that other delete operations have. The dry-run flag is still passed to the API helpers, so the `--dry-run` flag works, but the interactive confirmation prompt / `--force` requirement for `--no-input` mode is missing.

- **[OBS-003]**: `specs/runtime-requirements.md` traceability table -- Test IDs for REQ-RT-069 through REQ-RT-080 (except REQ-RT-071 which has some) still show "(filled by test-writer)". The tests exist but the matrix was not updated. Low priority documentation update.

## Modules Not Validated (if context limited)

None -- all 12 services were fully validated.

## Final Verdict

**CONDITIONAL APPROVAL** -- All Must requirements are met (N/A for this scope -- all requirements are Should or Could priority, and all are fully implemented). All Should requirements (REQ-RT-069 through REQ-RT-073, REQ-RT-075 through REQ-RT-077) pass. All Could requirements (REQ-RT-074, REQ-RT-078, REQ-RT-079, REQ-RT-080) are implemented and pass. No blocking issues. The following non-blocking observations are tracked for follow-up:

1. Bootstrap error handling inconsistency across 4 handlers (OBS-001)
2. Missing force/no-input guards on 5 destructive operations (OBS-002)
3. Traceability table not backfilled with test IDs (OBS-003)

Approved for review with the expectation that OBS-001 and OBS-002 are resolved before GA.
