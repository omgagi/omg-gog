# omega-google Functionalities Index

## Summary Statistics

| Metric | Count |
|--------|-------|
| Google API Services | 15 |
| Top-Level CLI Commands | 24 |
| CLI Handler Functions | ~214 |
| Service Functions (URL/body builders) | ~308 |
| Serde Types (structs/enums) | ~211 |
| Infrastructure Modules | 8 |
| Inline Unit Tests | ~1,333 |
| Integration Tests | ~441 |
| **Total Tests** | **~1,774** |
| Total Source Files | ~95 |

## Module Index

| # | Domain | File | CLI Commands | Types | Functions | Description |
|---|--------|------|-------------|-------|-----------|-------------|
| 1 | [Auth](auth-functionalities.md) | `src/auth/` | 11 | 14 | 23 | OAuth, tokens, credential backends, service accounts |
| 2 | [Config](config-functionalities.md) | `src/config/` | 7 | 2 | 14 | Config file management, credential paths |
| 3 | [Gmail](gmail-functionalities.md) | `src/services/gmail/` | 30+ | 31 | 44 | Gmail threads, messages, labels, drafts, settings, watch |
| 4 | [Calendar](calendar-functionalities.md) | `src/services/calendar/` | 18 | 18 | 25 | Events, calendars, freebusy, RSVP, special events |
| 5 | [Drive](drive-functionalities.md) | `src/services/drive/` | 16 | 11 | 37 | Files, folders, permissions, upload, comments |
| 6 | [Docs](docs-functionalities.md) | `src/services/docs/` | 14 | 21 | 28 | Document content, export, edit, sed, markdown |
| 7 | [Sheets](sheets-functionalities.md) | `src/services/sheets/` | 11 | 18 | 24 | Read, write, format, structure, A1 notation |
| 8 | [Slides](slides-functionalities.md) | `src/services/slides/` | 11 | 22 | 20 | Presentations, slides, notes, markdown, export |
| 9 | [Forms](forms-functionalities.md) | `src/services/forms/` | 5 | 8 | 6 | Forms metadata, responses |
| 10 | [Chat](chat-functionalities.md) | `src/services/chat/` | 10 | 9 | 12 | Spaces, messages, threads, DMs |
| 11 | [Classroom](classroom-functionalities.md) | `src/services/classroom/` | 60+ | 24 | 64 | Courses, roster, coursework, submissions, topics, guardians |
| 12 | [Tasks](tasks-functionalities.md) | `src/services/tasks/` | 11 | 5 | 11 | Task lists, task CRUD, done/undo |
| 13 | [Contacts](contacts-functionalities.md) | `src/services/contacts/` | 12 | 10 | 12 | Contacts CRUD, directory, other contacts |
| 14 | [People](people-functionalities.md) | `src/services/people/` | 4 | 8 | 4 | People profile, search, relations |
| 15 | [Groups](groups-functionalities.md) | `src/services/groups/` | 2 | 7 | 3 | Group listing, membership |
| 16 | [Keep](keep-functionalities.md) | `src/services/keep/` | 4 | 8 | 4 | Notes, search, attachments |
| 17 | [Apps Script](appscript-functionalities.md) | `src/services/appscript/` | 4 | 8 | 7 | Projects, content, run functions |
| 18 | [Infrastructure](infrastructure-functionalities.md) | `src/http/`, `src/output/`, `src/error/`, `src/time/`, `src/ui/`, `src/services/` | -- | 12 | 70+ | HTTP retry/circuit-breaker, output formatting, errors, time parsing, UI, pagination |
| 19 | [Utilities](utilities-functionalities.md) | `src/cli/` | 7 | 6 | 15 | open, completion, exit-codes, schema, agent, desire-paths |

## Architecture Diagram

```
main.rs
  -> cli::execute()
    -> cli::root::Cli (parse with clap)
    -> cli::dispatch_command()
      -> services::bootstrap_service_context()
        -> config::read_config()
        -> auth::keyring::credential_store_factory()
        -> auth::resolve_account()
        -> auth::token::needs_refresh() -> token::refresh_access_token()
        -> http::client::build_authenticated_client()
        -> output::resolve_mode_full()
        -> ui::Ui::new()
      -> handle_* functions
        -> services::ServiceContext.write_output()
          -> output::write_json() | output::write_plain() | output::write_csv()
            -> output::transform::unwrap_primary() / select_fields()
        -> http::api::api_get/post/patch/delete()
          -> http::middleware::execute_with_retry()
            -> http::retry::calculate_backoff()
            -> http::circuit_breaker::CircuitBreaker
        -> services::{service}::build_*_url() / build_*_body()
```

## Source Layout

```
src/
  main.rs                   -- Entry point
  lib.rs                    -- Module tree
  cli/                      -- CLI parsing + dispatch + handlers
  auth/                     -- OAuth, tokens, credential stores
  config/                   -- Config file management
  http/                     -- HTTP client, retry, circuit breaker
  output/                   -- JSON/Plain/CSV/Text formatting
  ui/                       -- Color, progress, prompts
  error/                    -- Exit codes, error types
  time/                     -- Date/time parsing
  services/                 -- Google API service implementations
    mod.rs                  -- ServiceContext, bootstrap, write_output
    common.rs               -- Pagination params, list response
    pagination.rs           -- Generic pagination loop
    export.rs               -- MIME type mapping
    gmail/                  -- 13 modules
    calendar/               -- 9 modules
    drive/                  -- 8 modules
    docs/                   -- 8 modules
    sheets/                 -- 7 modules
    slides/                 -- 7 modules
    forms/                  -- 4 modules
    chat/                   -- 5 modules
    classroom/              -- 11 modules
    tasks/                  -- 3 modules
    contacts/               -- 4 modules
    people/                 -- 3 modules
    groups/                 -- 3 modules
    keep/                   -- 4 modules
    appscript/              -- 3 modules
```

## Top-Level CLI Commands

| # | Command | Aliases | Type | Description |
|---|---------|---------|------|-------------|
| 1 | `auth` | -- | Management | Authentication and account management |
| 2 | `config` | -- | Management | Configuration management |
| 3 | `version` | -- | Utility | Print version information |
| 4 | `time` | -- | Utility | Date/time utilities |
| 5 | `gmail` | -- | Service | Gmail operations |
| 6 | `calendar` | `cal` | Service | Google Calendar operations |
| 7 | `drive` | -- | Service | Google Drive operations |
| 8 | `docs` | `doc` | Service | Google Docs operations |
| 9 | `sheets` | `sheet` | Service | Google Sheets operations |
| 10 | `slides` | `slide` | Service | Google Slides operations |
| 11 | `forms` | `form` | Service | Google Forms operations |
| 12 | `chat` | -- | Service | Google Chat operations |
| 13 | `classroom` | `class` | Service | Google Classroom operations |
| 14 | `tasks` | `task` | Service | Google Tasks operations |
| 15 | `contacts` | `contact` | Service | Google Contacts operations |
| 16 | `people` | `person` | Service | Google People operations |
| 17 | `groups` | `group` | Service | Google Groups operations |
| 18 | `keep` | -- | Service | Google Keep operations |
| 19 | `appscript` | `script`, `apps-script` | Service | Google Apps Script operations |
| 20 | `open` | `browse` | Utility | Open Google resource in browser |
| 21 | `completion` | -- | Utility | Generate shell completions |
| 22 | `exit-codes` | -- | Utility | Print exit code table |
| 23 | `schema` | `help-json` | Utility | Machine-readable CLI schema JSON |
| 24 | `agent` | -- | Utility | Agent-oriented commands |

## Global Flags

| Flag | Short | Env Var | Description |
|------|-------|---------|-------------|
| `--json` | `-j` | `GOG_JSON` | Output JSON |
| `--plain` | `-p` | `GOG_PLAIN` | Output plain tab-separated values |
| `--csv` | -- | `GOG_CSV` | Output CSV |
| `--color` | -- | `GOG_COLOR` | Color mode: auto, always, never |
| `--account` | `-a` | `GOG_ACCOUNT` | Account email or alias |
| `--client` | -- | `GOG_CLIENT` | OAuth client name |
| `--verbose` | `-v` | `GOG_VERBOSE` | Verbose output |
| `--dry-run` | `-n` | `GOG_DRY_RUN` | Dry run mode |
| `--force` | `-y` | -- | Skip confirmation prompts |
| `--no-input` | -- | `GOG_NO_INPUT` | Disable interactive prompts |
| `--select` | -- | -- | Select specific fields (dot-path) |
| `--results-only` | -- | -- | Strip envelope, return primary result |
| `--enable-commands` | -- | `GOG_ENABLE_COMMANDS` | Restrict available commands |

## Dead Code / Potential Issues

1. **Thin re-export modules**: `src/cli/exit_codes.rs`, `src/cli/desire_paths.rs`, `src/auth/account.rs`, `src/output/mode.rs` — structural placeholders with no unique logic.
2. **Unused scope options**: `handle_auth_add` parses `--readonly` and `--drive-scope` flags but they may not be wired to the OAuth flow yet.
3. **OutputMode::Text vs Plain**: Both exist; Text is TTY-colored, Plain is TSV. They share the same code path in `write_output()`.
