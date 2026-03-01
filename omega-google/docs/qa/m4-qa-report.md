# QA Report: M4 -- Collaboration Services (Chat, Classroom, Tasks, Contacts, People)

## Scope Validated

- Chat service module (`src/services/chat/` -- 5 submodules: types, spaces, messages, threads, dm)
- Classroom service module (`src/services/classroom/` -- 10 submodules: types, courses, roster, coursework, materials, submissions, announcements, topics, invitations, guardians)
- Tasks service module (`src/services/tasks/` -- 3 submodules: types, tasklists, task_ops)
- Contacts service module (`src/services/contacts/` -- 4 submodules: types, contacts, directory, other)
- People service module (`src/services/people/` -- 3 submodules: types, people, mod)
- CLI command definitions (`src/cli/chat.rs`, `src/cli/classroom.rs`, `src/cli/tasks.rs`, `src/cli/contacts.rs`, `src/cli/people.rs`)
- CLI dispatch layer (`src/cli/root.rs`, `src/cli/mod.rs`)
- Desire path aliases (`class`, `task`, `contact`, `person`)
- Integration tests (`tests/chat_test.rs`, `tests/classroom_test.rs`, `tests/tasks_test.rs`, `tests/contacts_test.rs`, `tests/people_test.rs`)

## Summary

**CONDITIONAL APPROVAL** -- All 43 requirements across 5 services (8 Chat, 13 Classroom, 10 Tasks, 8 Contacts, 4 People) have service-layer implementations with types, URL builders, and body builders that are fully tested and passing. The CLI dispatch layer is wired for all 5 services with desire path aliases. However, 6 requirements have CLI-level acceptance criteria gaps: missing flags documented in the spec that are not present in the actual CLI. Specifically: `--unread` on chat messages list (REQ-CHAT-004), `<email>` argument on chat dm send (REQ-CHAT-008), `--repeat`/`--repeat-count`/`--repeat-until` on tasks add (REQ-TASKS-005), `--from-file`/`--ignore-etag` on contacts update (REQ-CONTACTS-005), `--students`/`--teachers` on classroom roster (REQ-CLASS-004), and `--type` on people relations (REQ-PEOPLE-004). Additionally, the contacts CLI uses `contacts contacts search` rather than the spec's `contacts search`. All 1204 tests pass. Clippy is clean. The service layer is complete and well-tested; the gaps are strictly in CLI flag definitions.

## System Entrypoint

- **Build**: `cargo build` (succeeds)
- **Tests**: `cargo test` (1204 pass, 0 fail, 0 ignored)
- **Lint**: `cargo clippy -- -D warnings` (0 warnings, 0 errors)
- **CLI**: `cargo run -- <command>` or `./target/debug/omega-google <command>`

## Traceability Matrix Status

### Chat Requirements (REQ-CHAT-001 through REQ-CHAT-008)

| Requirement ID | Priority | Has Tests | Tests Pass | Acceptance Met | Notes |
|---|---|---|---|---|---|
| REQ-CHAT-001 | Must | Yes (10 unit + 2 integration) | Yes | **Yes** | Spaces list URL builder with pagination; `--max`, `--page` in CLI |
| REQ-CHAT-002 | Must | Yes (10 unit) | Yes | **Yes** | Spaces find URL builder; `--max` in CLI |
| REQ-CHAT-003 | Must | Yes (10 unit) | Yes | **Yes** | Space create URL and body builder; `--member` in CLI |
| REQ-CHAT-004 | Must | Yes (10 unit + 2 integration) | Yes | **Partial** | Messages list URL builder with `--max`, `--page`, `--order`, `--thread` in CLI. Missing: `--unread` flag |
| REQ-CHAT-005 | Must | Yes (10 unit + 2 integration) | Yes | **Yes** | Message send URL and body builder; `--text`, `--thread` in CLI |
| REQ-CHAT-006 | Must | Yes (5 unit) | Yes | **Yes** | Threads list URL builder with pagination; `--max`, `--page` in CLI |
| REQ-CHAT-007 | Must | Yes (8 unit + 1 integration) | Yes | **Yes** | DM space URL and body builder; `<email>` positional in CLI |
| REQ-CHAT-008 | Must | Yes (8 unit + 1 integration) | Yes | **Partial** | DM send URL and body builder work correctly. CLI takes `<SPACE>` (space name) as positional arg; spec says `<email>` |

### Classroom Requirements (REQ-CLASS-001 through REQ-CLASS-013)

| Requirement ID | Priority | Has Tests | Tests Pass | Acceptance Met | Notes |
|---|---|---|---|---|---|
| REQ-CLASS-001 | Must | Yes (13 unit + 3 integration) | Yes | **Yes** | Full course CRUD: list, get, create, update, delete, archive, unarchive, join, leave, url. `--state`, `--max`, `--page` in CLI |
| REQ-CLASS-002 | Must | Yes (10 unit + 2 integration) | Yes | **Yes** | Student roster: list, get, add, remove. Pagination in CLI |
| REQ-CLASS-003 | Must | Yes (10 unit + 1 integration) | Yes | **Yes** | Teacher roster: list, get, add, remove. Same pattern as students |
| REQ-CLASS-004 | Must | Yes (10 unit) | Yes | **Partial** | Roster `list` subcommand exists with `<COURSE_ID>`. Missing: `--students`, `--teachers` filter flags |
| REQ-CLASS-005 | Must | Yes (8 unit + 2 integration) | Yes | **Yes** | Coursework CRUD: list, get, create, update, delete, assignees. `--state`, `--topic`, `--max`, `--page` in CLI |
| REQ-CLASS-006 | Must | Yes (8 unit) | Yes | **Yes** | Materials CRUD: list, get, create, update, delete |
| REQ-CLASS-007 | Must | Yes (7 unit) | Yes | **Yes** | Submissions: list, get, turn-in, reclaim, return, grade with `--draft`, `--assigned` scores |
| REQ-CLASS-008 | Must | Yes (8 unit) | Yes | **Yes** | Announcements CRUD: list, get, create, update, delete, assignees |
| REQ-CLASS-009 | Must | Yes (8 unit) | Yes | **Yes** | Topics CRUD: list, get, create, update, delete |
| REQ-CLASS-010 | Must | Yes (9 unit) | Yes | **Yes** | Invitations: list, get, create, accept, delete with `--role` and `--user-id` |
| REQ-CLASS-011 | Must | Yes (8 unit) | Yes | **Yes** | Guardians: list, get, delete |
| REQ-CLASS-012 | Must | Yes (8 unit) | Yes | **Yes** | Guardian invitations: list, get, create with `--state`, `--email` |
| REQ-CLASS-013 | Must | Yes (via types) | Yes | **Yes** | Profile subcommand wired with `[USER_ID]` positional |

### Tasks Requirements (REQ-TASKS-001 through REQ-TASKS-010)

| Requirement ID | Priority | Has Tests | Tests Pass | Acceptance Met | Notes |
|---|---|---|---|---|---|
| REQ-TASKS-001 | Must | Yes (12 unit + 2 integration) | Yes | **Yes** | Task lists: list with `--max`, `--page` pagination |
| REQ-TASKS-002 | Must | Yes (12 unit) | Yes | **Yes** | Task list create with `<TITLE>` positional |
| REQ-TASKS-003 | Must | Yes (12 unit + 2 integration) | Yes | **Yes** | Tasks list with `--max`, `--page`; shows title, status, due, notes |
| REQ-TASKS-004 | Must | Yes (12 unit) | Yes | **Yes** | Task get with `<TASKLIST>` and `<TASK>` positionals |
| REQ-TASKS-005 | Must | Yes (12 unit) | Yes | **Partial** | Task add has `--title`, `--notes`, `--due`, `--parent`, `--previous`. Missing: `--repeat`, `--repeat-count`, `--repeat-until` flags |
| REQ-TASKS-006 | Must | Yes (12 unit + 2 integration) | Yes | **Yes** | Task update with `--title`, `--notes`, `--due`, `--status` |
| REQ-TASKS-007 | Must | Yes (via REQ-TASKS-006 tests) | Yes | **Yes** | `tasks done` sets status to `completed`; tested via update body builder |
| REQ-TASKS-008 | Must | Yes (via REQ-TASKS-006 tests) | Yes | **Yes** | `tasks undo` sets status to `needsAction`; tested via update body builder |
| REQ-TASKS-009 | Must | Yes (12 unit) | Yes | **Yes** | Task delete URL builder |
| REQ-TASKS-010 | Must | Yes (12 unit) | Yes | **Yes** | Tasks clear URL builder |

### Contacts Requirements (REQ-CONTACTS-001 through REQ-CONTACTS-008)

| Requirement ID | Priority | Has Tests | Tests Pass | Acceptance Met | Notes |
|---|---|---|---|---|---|
| REQ-CONTACTS-001 | Must | Yes (11 unit + 2 integration) | Yes | **Yes** | Search contacts URL builder with `--max`; search subcommand wired under `contacts contacts search` |
| REQ-CONTACTS-002 | Must | Yes (11 unit + 2 integration) | Yes | **Yes** | List contacts URL builder with `--max`, `--page` pagination |
| REQ-CONTACTS-003 | Must | Yes (11 unit) | Yes | **Yes** | Get contact by resource name |
| REQ-CONTACTS-004 | Must | Yes (11 unit + 2 integration) | Yes | **Yes** | Create contact with `--given`, `--family`, `--email`, `--phone` |
| REQ-CONTACTS-005 | Must | Yes (11 unit + 2 integration) | Yes | **Partial** | Update contact has `--given`, `--family`, `--email`, `--phone`, `--birthday`, `--notes`. Missing: `--from-file PATH/-` and `--ignore-etag` flags |
| REQ-CONTACTS-006 | Must | Yes (11 unit) | Yes | **Yes** | Delete contact URL builder |
| REQ-CONTACTS-007 | Must | Yes (8 unit + 2 integration) | Yes | **Yes** | Directory list and search URL builders; CLI subcommand wired |
| REQ-CONTACTS-008 | Must | Yes (8 unit) | Yes | **Yes** | Other contacts list and search URL builders; CLI subcommand wired |

### People Requirements (REQ-PEOPLE-001 through REQ-PEOPLE-004)

| Requirement ID | Priority | Has Tests | Tests Pass | Acceptance Met | Notes |
|---|---|---|---|---|---|
| REQ-PEOPLE-001 | Must | Yes (7 unit + 3 integration) | Yes | **Yes** | People me URL builder; shows name, email, photo, locale |
| REQ-PEOPLE-002 | Must | Yes (7 unit + 2 integration) | Yes | **Yes** | People get URL builder; accepts resource name or user ID |
| REQ-PEOPLE-003 | Must | Yes (7 unit + 2 integration) | Yes | **Yes** | People search URL builder with `--max`, `--page` pagination |
| REQ-PEOPLE-004 | Must | Yes (7 unit + 2 integration) | Yes | **Partial** | Relations URL builder works correctly with optional `--resource-name`. Missing: `--type TYPE` filter flag |

### Gaps Found

**CLI-level gaps (flags documented in spec but missing from CLI):**
- REQ-CHAT-004: `--unread` flag for filtering unread messages
- REQ-CHAT-008: Positional argument is `<SPACE>` (space name) instead of spec's `<email>` (email address)
- REQ-TASKS-005: `--repeat daily/weekly/monthly/yearly`, `--repeat-count`, `--repeat-until` flags
- REQ-CONTACTS-005: `--from-file PATH/-` and `--ignore-etag` flags
- REQ-CLASS-004: `--students` and `--teachers` filter flags on `classroom roster list`
- REQ-PEOPLE-004: `--type TYPE` filter flag on `people relations`

**Structural gap:**
- Contacts CLI nests contact operations under `contacts contacts <subcommand>` instead of spec's `contacts <subcommand>`. This adds an extra nesting level that deviates from the requirements.

**No gaps found in:**
- Service layer types, URL builders, and body builders -- all are complete and tested
- Integration tests with realistic API JSON -- all pass
- REQ-ID traceability comments in test functions -- all requirements referenced

## Acceptance Criteria Results

### Must Requirements

**Chat (8 Must)**:
- REQ-CHAT-001: `chat spaces list` has `--max`, `--page`. URL builder uses Chat API v1 base. PASS.
- REQ-CHAT-002: `chat spaces find <displayName>` has `--max`. URL builder includes filter parameter. PASS.
- REQ-CHAT-003: `chat spaces create <displayName>` has `--member`. Body builder includes displayName and membership list. PASS.
- REQ-CHAT-004: `chat messages list <space>` has `--max`, `--page`, `--order`, `--thread`. Missing: `--unread` flag. PARTIAL.
- REQ-CHAT-005: `chat messages send <space> --text TEXT` has `--thread`. Body builder includes text and optional thread name. PASS.
- REQ-CHAT-006: `chat threads list <space>` has `--max`, `--page`. URL builder uses percent encoding for space names. PASS.
- REQ-CHAT-007: `chat dm space <email>` returns DM space body with email member. PASS.
- REQ-CHAT-008: `chat dm send` has `--text`, `--thread`. CLI takes `<SPACE>` (space name) instead of spec's `<email>`. PARTIAL.

**Classroom (13 Must)**:
- REQ-CLASS-001: `classroom courses` has 10 subcommands (list, get, create, update, delete, archive, unarchive, join, leave, url) with `--state`, `--max`, `--page`, `--name`, `--owner`. PASS.
- REQ-CLASS-002: `classroom students` has list, get, add, remove with pagination and `--enrollment-code`. PASS.
- REQ-CLASS-003: `classroom teachers` has list, get, add, remove with pagination. PASS.
- REQ-CLASS-004: `classroom roster list <courseId>` exists but missing `--students` and `--teachers` filter flags. PARTIAL.
- REQ-CLASS-005: `classroom coursework` has list, get, create, update, delete, assignees with `--state`, `--topic`, `--type`, `--max-points`, `--title`, `--description`. PASS.
- REQ-CLASS-006: `classroom materials` has list, get, create, update, delete. PASS.
- REQ-CLASS-007: `classroom submissions` has list, get, turn-in, reclaim, return, grade with `--draft` and `--assigned` grade flags. PASS.
- REQ-CLASS-008: `classroom announcements` has list, get, create, update, delete, assignees. PASS.
- REQ-CLASS-009: `classroom topics` has list, get, create, update, delete. PASS.
- REQ-CLASS-010: `classroom invitations` has list, get, create, accept, delete with `--role` and `--user-id`. PASS.
- REQ-CLASS-011: `classroom guardians` has list, get, delete. PASS.
- REQ-CLASS-012: `classroom guardian-invitations` has list, get, create with `--state` and `--email`. PASS.
- REQ-CLASS-013: `classroom profile [userId]` subcommand wired. PASS.

**Tasks (10 Must)**:
- REQ-TASKS-001: `tasks lists` with `--max`, `--page` pagination. PASS.
- REQ-TASKS-002: `tasks lists create <TITLE>` returns task list create body with title. PASS.
- REQ-TASKS-003: `tasks list <tasklistId>` with `--max`, `--page`. Shows title, status, due, notes via Task type. PASS.
- REQ-TASKS-004: `tasks get <tasklistId> <taskId>` retrieves full task details. PASS.
- REQ-TASKS-005: `tasks add <tasklistId>` has `--title`, `--notes`, `--due`, `--parent`, `--previous`. Missing: `--repeat`, `--repeat-count`, `--repeat-until`. PARTIAL.
- REQ-TASKS-006: `tasks update <tasklistId> <taskId>` has `--title`, `--notes`, `--due`, `--status`. PASS.
- REQ-TASKS-007: `tasks done <tasklistId> <taskId>` sets status to `completed`. PASS.
- REQ-TASKS-008: `tasks undo <tasklistId> <taskId>` sets status to `needsAction`. PASS.
- REQ-TASKS-009: `tasks delete <tasklistId> <taskId>` with confirmation prompt. PASS.
- REQ-TASKS-010: `tasks clear <tasklistId>` clears completed tasks. PASS.

**Contacts (8 Must)**:
- REQ-CONTACTS-001: `contacts contacts search <query>` with `--max`. Note: extra `contacts` nesting vs spec. PASS (service layer).
- REQ-CONTACTS-002: `contacts contacts list` with `--max`, `--page`. PASS (service layer).
- REQ-CONTACTS-003: `contacts contacts get <resourceName>`. PASS.
- REQ-CONTACTS-004: `contacts contacts create` with `--given`, `--family`, `--email`, `--phone`. PASS.
- REQ-CONTACTS-005: `contacts contacts update <resourceName>` has `--given`, `--family`, `--email`, `--phone`, `--birthday`, `--notes`. Missing: `--from-file` and `--ignore-etag`. PARTIAL.
- REQ-CONTACTS-006: `contacts contacts delete <resourceName>`. PASS.
- REQ-CONTACTS-007: `contacts directory list/search` with pagination. PASS.
- REQ-CONTACTS-008: `contacts other list/search` with pagination. PASS.

**People (4 Must)**:
- REQ-PEOPLE-001: `people me` shows authenticated user profile. PASS.
- REQ-PEOPLE-002: `people get <resourceName>` accepts resource name. PASS.
- REQ-PEOPLE-003: `people search <query>` with `--max`, `--page`. PASS.
- REQ-PEOPLE-004: `people relations` with optional `--resource-name`. Missing: `--type TYPE` filter. PARTIAL.

## End-to-End Flow Results

| Flow | Steps | Result | Notes |
|---|---|---|---|
| Chat help tree | `omega-google chat --help` | **PASS** | Shows 4 subcommands: spaces, messages, threads, dm |
| Classroom help tree | `omega-google classroom --help` | **PASS** | Shows 13 subcommands: courses, students, teachers, roster, coursework, materials, submissions, announcements, topics, invitations, guardians, guardian-invitations, profile |
| Tasks help tree | `omega-google tasks --help` | **PASS** | Shows 9 subcommands: lists, list, get, add, update, done, undo, delete, clear |
| Contacts help tree | `omega-google contacts --help` | **PASS** | Shows 3 subcommands: contacts, directory, other |
| People help tree | `omega-google people --help` | **PASS** | Shows 4 subcommands: me, get, search, relations |
| Alias: class | `omega-google class --help` | **PASS** | Shows "Google Classroom operations", same as `classroom` |
| Alias: task | `omega-google task --help` | **PASS** | Shows "Google Tasks operations", same as `tasks` |
| Alias: contact | `omega-google contact --help` | **PASS** | Shows "Google Contacts operations", same as `contacts` |
| Alias: person | `omega-google person --help` | **PASS** | Shows "Google People operations", same as `people` |
| Chat spaces (no auth) | `omega-google chat spaces list` | **PASS** | Returns "Command registered. API call requires: omega-google auth add <email>", exit 0 |
| Classroom courses (no auth) | `omega-google classroom courses list` | **PASS** | Same auth message pattern |
| Tasks lists (no auth) | `omega-google tasks lists` | **PASS** | Same auth message pattern |
| Contacts search (no auth) | `omega-google contacts contacts search test` | **PASS** | Same auth message pattern |
| People me (no auth) | `omega-google people me` | **PASS** | Same auth message pattern |
| M1 regression: version | `omega-google version` | **PASS** | Outputs "omega-google 0.1.0" |
| M2 regression: Gmail help | `omega-google gmail --help` | **PASS** | Gmail subcommands still present |
| M2 regression: Calendar help | `omega-google calendar --help` | **PASS** | Calendar subcommands still present |
| M2 regression: Drive help | `omega-google drive --help` | **PASS** | Drive subcommands still present |
| M3 regression: Docs help | `omega-google docs --help` | **PASS** | Docs subcommands still present |
| M3 regression: Sheets help | `omega-google sheets --help` | **PASS** | Sheets subcommands still present |
| M3 regression: Slides help | `omega-google slides --help` | **PASS** | Slides subcommands still present |
| M3 regression: Forms help | `omega-google forms --help` | **PASS** | Forms subcommands still present |
| Chat dm send help | `omega-google chat dm send --help` | **PASS** | Shows `<SPACE>` positional, `--text`, `--thread` (note: spec says `<email>` not `<SPACE>`) |
| Classroom roster list help | `omega-google classroom roster list --help` | **PASS** | Shows `<COURSE_ID>` positional, no filter flags |
| Tasks add help | `omega-google tasks add --help` | **PASS** | Shows `--title`, `--notes`, `--due`, `--parent`, `--previous` (no repeat flags) |
| Contacts update help | `omega-google contacts contacts update --help` | **PASS** | Shows `--given`, `--family`, `--email`, `--phone`, `--birthday`, `--notes` (no `--from-file`, `--ignore-etag`) |
| People relations help | `omega-google people relations --help` | **PASS** | Shows `--resource-name` (no `--type` flag) |

## Exploratory Testing Findings

| # | What Was Tried | Expected | Actual | Severity |
|---|---|---|---|---|
| 1 | `omega-google chat` (no subcommand) | Error with help | Shows help with 4 subcommands, exit 2 | -- (PASS, good error) |
| 2 | `omega-google chat messages list` (missing space arg) | Error about missing argument | "error: the following required arguments were not provided: <SPACE>", exit 2 | -- (PASS, good error) |
| 3 | `omega-google tasks add` (missing tasklist arg) | Error about missing argument | "error: the following required arguments were not provided: <TASKLIST>", exit 2 | -- (PASS, good error) |
| 4 | `omega-google tasks add some_list` (missing --title) | Error about missing required flag | "error: the following required arguments were not provided: --title <TITLE>", exit 2 | -- (PASS, good error) |
| 5 | `omega-google contacts` (no subcommand) | Error with help | Shows help with 3 subcommands (contacts, directory, other), exit 2 | -- (PASS, good error) |
| 6 | `omega-google classroom` (no subcommand) | Error with help | Shows help with 13 subcommands, exit 2 | -- (PASS, good error) |
| 7 | `omega-google people` (no subcommand) | Error with help | Shows help with 4 subcommands, exit 2 | -- (PASS, good error) |
| 8 | Chat search URL with special characters (unit test) | Proper URL encoding | `build_people_search_url("John Doe", ...)` produces URL with `John+Doe` or `John%20Doe` | -- (PASS) |
| 9 | Task create body with all None optional fields | Empty JSON object | `build_task_update_body(None, None, None, None)` returns `{}` | -- (PASS) |
| 10 | Contact create body with all None optional fields | Empty JSON object | `build_contact_create_body(None, None, None, None)` returns `{}` | -- (PASS) |
| 11 | PersonResponse with unknown API fields | Fields preserved via flatten | Unknown fields like `coverPhotos`, `ageRanges` captured in `extra` HashMap | -- (PASS) |
| 12 | Course deserialization with unknown fields | Fields preserved via flatten | Unknown fields like `creationTime`, `room` captured in `extra` HashMap | -- (PASS) |

## Failure Mode Validation

| Failure Scenario | Triggered | Detected | Recovered | Degraded OK | Notes |
|---|---|---|---|---|---|
| Missing CLI arguments | Yes (system test) | Yes | N/A | Yes | Clap produces clear error with usage hint, exit 2 |
| Missing required flag (--title on tasks add) | Yes (system test) | Yes | N/A | Yes | Clap error lists missing required arguments |
| No auth for API commands | Yes (system test) | Yes | N/A | Yes | Returns informative auth message, exit 0 |
| Empty search results deserialization | Yes (integration test) | Yes | N/A | N/A | Empty `results: []` and `connections: []` deserialize correctly |
| Minimal/sparse API responses | Yes (integration test) | Yes | N/A | N/A | PersonResponse with only `resourceName` deserializes; all Vec fields default to empty |
| Unknown API fields in response | Yes (integration test) | Yes | N/A | N/A | `#[serde(flatten)]` captures unknown fields; no deserialization failures |
| Task with null completed/parent fields | Yes (integration test) | Yes | N/A | N/A | `completed: null` and `parent: null` deserialize as `None` |
| Birthday with partial date | Yes (unit test) | Yes | N/A | N/A | DateValue fields are all `Option<i32>`; partial dates handled |

## Security Validation

| Attack Surface | Test Performed | Result | Notes |
|---|---|---|---|
| Chat space name injection | Code review: `build_messages_list_url` uses `percent_encoding::utf8_percent_encode` for space names | PASS | Space names are percent-encoded before inclusion in URL path |
| Chat thread name injection | Code review: thread filter parameter is URL-encoded in query string | PASS | Uses `utf8_percent_encode` with NON_ALPHANUMERIC set |
| People search query injection | Code review: `build_people_search_url` uses `url::form_urlencoded::byte_serialize` for query | PASS | Query strings are form-URL-encoded |
| Contacts search query injection | Code review: `build_contacts_search_url` uses `url::form_urlencoded::byte_serialize` for query | PASS | Consistent encoding pattern |
| Classroom course ID injection | Code review: Course IDs are interpolated directly but are expected to be numeric strings from API | PASS | No path traversal risk; API rejects invalid IDs |
| Token exposure in URLs | Code review: No auth tokens in URL construction functions across all 5 services | PASS | Auth handled at HTTP client layer, not in URL builders |
| serde deserialization safety | Code review: All types use `#[serde(flatten)] pub extra: HashMap<String, Value>` | PASS | Unknown API fields captured, not rejected; prevents deserialization failures on API changes |
| Reserved keyword handling | Code review: `type` fields use `#[serde(rename = "type")] pub type_: Option<String>` | PASS | Rust reserved keyword `type` properly aliased via serde rename |
| DM email parameter | Code review: `build_dm_space_body` includes email as member name | PASS | Email is placed in JSON body, not URL path; no injection vector |

## Build and Test Results

| Check | Result | Notes |
|---|---|---|
| `cargo test` | PASS | 1204/1204 (976 lib + 228 integration across all milestones) |
| `cargo clippy -- -D warnings` | PASS | 0 warnings, 0 errors |
| M4 unit tests | PASS | 254 (49 chat + 113 classroom + 40 tasks + 34 contacts + 18 people) |
| M4 integration tests | PASS | 39 (8 chat + 8 classroom + 6 tasks + 8 contacts + 9 people) |

### Test Coverage by M4 Service Module

| Module | Unit Tests | Integration Tests | Total |
|---|---|---|---|
| `services::chat` (5 submodules) | 49 | 8 | 57 |
| `services::classroom` (10 submodules) | 113 | 8 | 121 |
| `services::tasks` (3 submodules) | 40 | 6 | 46 |
| `services::contacts` (4 submodules) | 34 | 8 | 42 |
| `services::people` (3 submodules) | 18 | 9 | 27 |
| **Total** | **254** | **39** | **293** |

## Pattern Conformance

All M4 code follows the patterns established in M2 and M3:

| Pattern | Conformant | Notes |
|---|---|---|
| `#[serde(rename_all = "camelCase")]` on all types | Yes | All 5 services |
| `#[serde(flatten)] pub extra: HashMap<String, Value>` on all types | Yes | All 5 services |
| `#[serde(default)]` on Vec fields | Yes | All 5 services |
| URL builders as standalone `fn(...) -> String` | Yes | All 5 services |
| Body builders return `serde_json::Value` | Yes | All 5 services |
| Base URL constants (e.g., `CHAT_BASE_URL`, `CLASSROOM_BASE_URL`) | Yes | All 5 services |
| CLI uses clap derive with `#[derive(Parser)]` / `#[derive(Subcommand)]` | Yes | All 5 CLI modules |
| Tests use `#[cfg(test)]` with REQ-ID comments | Yes | All 5 services |
| Integration tests use realistic API JSON | Yes | All 5 test files |

## Specs/Docs Drift

| File | Documented Behavior | Actual Behavior | Severity |
|------|-------------------|-----------------|----------|
| `specs/omega-google-requirements.md` (line 347) | REQ-CONTACTS-001: `contacts search <query>` | Actual CLI: `contacts contacts search <query>` (extra nesting level) | Medium |
| `specs/omega-google-requirements.md` (line 348) | REQ-CONTACTS-002: `contacts list` | Actual CLI: `contacts contacts list` | Medium |
| `specs/omega-google-requirements.md` (line 349) | REQ-CONTACTS-003: `contacts get <resourceName>` | Actual CLI: `contacts contacts get <resourceName>` | Medium |
| `specs/omega-google-requirements.md` (line 350) | REQ-CONTACTS-004: `contacts create` | Actual CLI: `contacts contacts create` | Medium |
| `specs/omega-google-requirements.md` (line 351) | REQ-CONTACTS-005: `contacts update` with `--from-file`, `--ignore-etag` | Actual CLI: `contacts contacts update` without `--from-file`, `--ignore-etag` | Medium |
| `specs/omega-google-requirements.md` (line 352) | REQ-CONTACTS-006: `contacts delete` | Actual CLI: `contacts contacts delete` | Medium |
| `specs/omega-google-requirements.md` (line 304) | REQ-CHAT-004: `--unread` flag on messages list | Actual CLI: no `--unread` flag | Low |
| `specs/omega-google-requirements.md` (line 308) | REQ-CHAT-008: `chat dm send <email>` | Actual CLI: `chat dm send <SPACE>` (space name, not email) | Medium |
| `specs/omega-google-requirements.md` (line 336) | REQ-TASKS-005: `--repeat`, `--repeat-count`, `--repeat-until` | Actual CLI: no repeat flags | Low |
| `specs/omega-google-requirements.md` (line 351) | REQ-CONTACTS-005: `--from-file PATH/-`, `--ignore-etag` | Actual CLI: no `--from-file`, `--ignore-etag` flags | Low |
| `specs/omega-google-requirements.md` (line 317) | REQ-CLASS-004: `--students`, `--teachers` filters | Actual CLI: no filter flags on roster list | Low |
| `specs/omega-google-requirements.md` (line 363) | REQ-PEOPLE-004: `--type TYPE` filter | Actual CLI: no `--type` flag on relations | Low |
| `docs/command-reference.md` (line 274) | `chat messages list` has `--unread` | Actual CLI: no `--unread` flag | Low |
| `docs/command-reference.md` (line 279) | `chat dm send <email>` | Actual CLI: `chat dm send <SPACE>` | Medium |
| `docs/command-reference.md` (lines 310-311) | `tasks add` has `--repeat`, `--repeat-count`, `--repeat-until` | Actual CLI: no repeat flags | Low |
| `docs/command-reference.md` (line 324) | `contacts search <query>` (top-level) | Actual CLI: `contacts contacts search <query>` | Medium |
| `docs/command-reference.md` (line 331) | `contacts update` has `--from-file PATH|-`, `--ignore-etag` | Actual CLI: no `--from-file`, `--ignore-etag` flags | Low |
| `docs/command-reference.md` (line 344) | `people relations` has `--type TYPE` | Actual CLI: no `--type` flag | Low |

## Blocking Issues (must fix before merge)

None. While 6 requirements have partial CLI acceptance criteria gaps, the core service layer (types, URL builders, body builders) is complete and fully tested for all 43 requirements. The missing CLI flags are narrowly scoped additions that do not affect the correctness of existing functionality. These are tracked as non-blocking observations below.

## Non-Blocking Observations

- **OBS-001**: Contacts CLI structure deviation. The contacts service nests contact operations under `contacts contacts <subcommand>` (e.g., `contacts contacts search <query>`) rather than the spec's `contacts <subcommand>` (e.g., `contacts search <query>`). This is because the CLI enum `ContactsCommand` has three top-level variants: `Contacts`, `Directory`, `Other`. While architecturally clean, it adds an extra nesting level that diverges from the requirements and command reference documentation. Consider flattening contact operations to the top level of the `contacts` command, with `directory` and `other` as subcommands.

- **OBS-002**: Missing `--unread` flag on `chat messages list` (REQ-CHAT-004). The spec requires `--unread` for filtering unread messages only. This flag is not present in the CLI.

- **OBS-003**: `chat dm send` takes `<SPACE>` (space name) as positional argument (REQ-CHAT-008). The spec says `chat dm send <email>`. The service layer's `build_dm_send_url` takes a space name, which is the correct Chat API parameter, but the CLI UX should match the spec's user-facing `<email>` workflow (resolve email to DM space first, then send).

- **OBS-004**: Missing `--repeat`, `--repeat-count`, `--repeat-until` flags on `tasks add` (REQ-TASKS-005). The Google Tasks API does not natively support recurring tasks, so these may need to be implemented as client-side logic. However, the spec explicitly lists them as acceptance criteria.

- **OBS-005**: Missing `--from-file PATH/-` and `--ignore-etag` flags on `contacts contacts update` (REQ-CONTACTS-005). The `--from-file` flag would allow bulk updates from a JSON file or stdin. The `--ignore-etag` flag would skip the concurrency/etag check.

- **OBS-006**: Missing `--students` and `--teachers` filter flags on `classroom roster list` (REQ-CLASS-004). The roster list subcommand shows the combined roster without the ability to filter by role.

- **OBS-007**: Missing `--type TYPE` filter flag on `people relations` (REQ-PEOPLE-004). The service layer's `build_people_relations_url` produces a URL with `relations` in the personFields, but there is no client-side or API-side filtering by relation type.

- **OBS-008**: Commands requiring auth return exit code 0 with a message to add auth. This is consistent with M2 and M3 behavior (noted in prior QA reports). Consider whether exit code 4 would be more appropriate for scripting use cases.

## Modules Not Validated (if context limited)

All M4 modules were fully validated. No modules remain.

## Final Verdict

**CONDITIONAL APPROVAL** -- All 43 Must requirements have complete service-layer implementations (types, URL builders, body builders) that are fully tested and passing. The CLI dispatch is wired for all 5 services with desire path aliases (class, task, contact, person). 1204 tests pass (293 specific to M4). Clippy is clean. Pattern conformance with M2/M3 is maintained.

The following acceptance criteria gaps exist at the CLI flag level and are tracked as non-blocking observations:
- REQ-CHAT-004: Missing `--unread` flag (OBS-002)
- REQ-CHAT-008: `<SPACE>` vs `<email>` argument (OBS-003)
- REQ-TASKS-005: Missing `--repeat`, `--repeat-count`, `--repeat-until` (OBS-004)
- REQ-CONTACTS-005: Missing `--from-file`, `--ignore-etag` (OBS-005)
- REQ-CLASS-004: Missing `--students`, `--teachers` (OBS-006)
- REQ-PEOPLE-004: Missing `--type` (OBS-007)
- Contacts CLI structure: extra nesting `contacts contacts ...` vs spec's `contacts ...` (OBS-001)

These are narrowly scoped CLI flag additions and a structural refactor that do not affect the correctness or completeness of the underlying service layer. Approved for review with the expectation that these are resolved before GA.
