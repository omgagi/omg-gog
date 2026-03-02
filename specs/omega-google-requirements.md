# Requirements: omega-google

## Scope

This document covers the complete requirements for omega-google, a Rust reimplementation of the Go CLI tool `gogcli`. It spans all 15 Google Workspace services, cross-cutting infrastructure (auth, config, HTTP, output), and supporting features (agent mode, shell completion, email tracking).

### Domains Affected
- New Rust project under `omega-google/`
- No existing code -- this is a greenfield implementation
- Reference: Go source at `/tmp/gogcli-source` (546 Go files, 272 command handlers, 311 test files)

### Source of Truth
- `/tmp/gogcli-source/docs/spec.md` -- Authoritative feature specification
- `/tmp/gogcli-source/internal/cmd/*.go` -- Exact command structures and flags
- `/tmp/gogcli-source/internal/googleauth/service.go` -- OAuth scope definitions per service
- `/tmp/gogcli-source/internal/config/config.go` -- Config file structure
- `/tmp/gogcli-source/internal/secrets/store.go` -- Keyring interface
- `/tmp/gogcli-source/internal/googleapi/transport.go` -- Retry/backoff logic

## Summary (plain language)

omega-google is a command-line tool written in Rust that lets developers and power users interact with 15 Google Workspace services (Gmail, Calendar, Drive, Docs, Sheets, Slides, Forms, Chat, Classroom, Tasks, Contacts, People, Groups, Keep, Apps Script) from the terminal. It supports multiple Google accounts, stores credentials securely in the OS keyring, outputs in JSON/plain/human-friendly formats, and is designed for scripting and LLM agent integration. It is a feature-complete port of the existing Go tool `gogcli`.

## User Stories

- As a developer, I want to search my Gmail from the terminal so that I can quickly find emails without switching to a browser.
- As a script author, I want JSON output from all commands so that I can pipe results to jq and other tools.
- As a CI pipeline operator, I want non-interactive mode with stable exit codes so that automation can branch on success/failure.
- As a power user with multiple Google accounts, I want to switch between accounts easily so that I can manage work and personal accounts.
- As a Workspace admin, I want to manage groups and delegate access so that I can automate user administration.
- As an LLM agent, I want a schema command that describes all commands and flags as JSON so that I can discover capabilities programmatically.
- As a developer, I want `nix build` to produce a single static binary so that I can distribute the tool easily.
- As a cautious user, I want dry-run mode for destructive operations so that I can preview what will happen before committing.

---

## Requirements

### Milestone 1: Project Scaffolding + Auth Infrastructure + Config

#### SCAFFOLD -- Project Structure and Build

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-SCAFFOLD-001 | Cargo workspace with binary crate `omega-google` | Must | - [ ] `cargo build` produces `omega-google` binary<br>- [ ] Project is structured under `omega-google/` directory<br>- [ ] Binary name is `omega-google` |
| REQ-SCAFFOLD-002 | `flake.nix` with dev shell providing Rust toolchain, openssl, pkg-config | Must | - [ ] `nix develop` enters a shell with `cargo`, `rustc`, `clippy`, `rustfmt` available<br>- [ ] OpenSSL and pkg-config are available for reqwest TLS |
| REQ-SCAFFOLD-003 | `nix build` produces a distributable binary | Must | - [ ] `nix build` succeeds and produces `result/bin/omega-google`<br>- [ ] Binary runs on the target platform without additional dependencies |
| REQ-SCAFFOLD-004 | Cargo dependencies established: clap, tokio, reqwest, serde, serde_json, keyring, oauth2, crossterm, thiserror, anyhow | Must | - [ ] All crates compile and pass basic smoke test<br>- [ ] `Cargo.lock` is committed |
| REQ-SCAFFOLD-005 | Module structure mirrors domain separation: cli, config, auth, http, output, ui, error | Must | - [ ] Each domain has its own Rust module<br>- [ ] Modules have clear public interfaces |

#### CLI -- Root CLI Flags and Structure

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-CLI-001 | Root flags: `--json` / `-j`, `--plain` / `-p`, `--color auto/always/never`, `--account` / `-a`, `--client`, `--verbose` / `-v`, `--dry-run` / `-n`, `--force` / `-y`, `--no-input`, `--select`, `--results-only`, `--enable-commands` | Must | - [ ] All flags parse correctly with clap<br>- [ ] `--json` and `--plain` are mutually exclusive (error if both set)<br>- [ ] `--color` defaults to `auto` |
| REQ-CLI-002 | Environment variable overrides: `GOG_COLOR`, `GOG_JSON`, `GOG_PLAIN`, `GOG_ACCOUNT`, `GOG_CLIENT`, `GOG_KEYRING_PASSWORD`, `GOG_KEYRING_BACKEND`, `GOG_TIMEZONE`, `GOG_ENABLE_COMMANDS`, `GOG_AUTO_JSON`, `GOG_CALENDAR_WEEKDAY` | Must | - [ ] Each env var is read and used as default when flag not set<br>- [ ] Flags override env vars<br>- [ ] `GOG_AUTO_JSON=1` enables JSON when stdout is not a TTY |
| REQ-CLI-003 | `--version` flag prints version, commit, and build date | Must | - [ ] `omega-google --version` outputs version string<br>- [ ] Version info is embedded at build time |
| REQ-CLI-004 | `omega-google version` command with JSON/plain/text output | Must | - [ ] `omega-google version --json` outputs `{"version":"...","commit":"...","date":"..."}`<br>- [ ] Text output prints human-readable version string |
| REQ-CLI-005 | `omega-google time now` command shows current time with optional `--timezone` | Must | - [ ] Default output shows local time in RFC3339<br>- [ ] `--timezone America/New_York` shows Eastern time<br>- [ ] JSON mode outputs structured time object |
| REQ-CLI-006 | Error output goes to stderr; data output goes to stdout | Must | - [ ] Progress messages, errors, and hints are written to stderr<br>- [ ] All data/results are written to stdout<br>- [ ] Stdout can be safely piped/redirected |
| REQ-CLI-007 | Stable exit codes matching gogcli conventions | Must | - [ ] Exit 0 = success<br>- [ ] Exit 1 = generic error<br>- [ ] Exit 2 = usage/parse error<br>- [ ] Exit 3 = empty results (with `--fail-empty`)<br>- [ ] Exit 4 = auth required<br>- [ ] Exit 5 = not found<br>- [ ] Exit 6 = permission denied<br>- [ ] Exit 7 = rate limited<br>- [ ] Exit 8 = retryable error<br>- [ ] Exit 10 = config error<br>- [ ] Exit 130 = cancelled (SIGINT) |
| REQ-CLI-008 | `SilenceUsage` equivalent: errors printed by the tool, not the CLI framework | Must | - [ ] Parse errors are formatted by omega-google, not raw clap output<br>- [ ] Error messages are colored when appropriate |
| REQ-CLI-009 | `--fields` flag rewritten to `--select` (desire path for agents) except in `calendar events` context | Should | - [ ] `--fields x,y` is silently rewritten to `--select x,y`<br>- [ ] `calendar events --fields` is NOT rewritten (passes to Calendar API) |

#### CONFIG -- Configuration Management

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-CONFIG-001 | Config file at `$CONFIG_DIR/omega-google/config.json` (JSON5 read, JSON write) | Must | - [ ] Config is read with JSON5 parser (comments, trailing commas)<br>- [ ] Config is written as standard JSON with mode 0600<br>- [ ] Atomic write via tmp file + rename |
| REQ-CONFIG-002 | Config structure: `keyring_backend`, `default_timezone`, `account_aliases`, `account_clients`, `client_domains` | Must | - [ ] All fields are optional (empty config is valid)<br>- [ ] Unknown fields are preserved (forward compatibility) |
| REQ-CONFIG-003 | `config get <key>` command | Must | - [ ] Prints value of the specified config key<br>- [ ] Returns error if key does not exist |
| REQ-CONFIG-004 | `config set <key> <value>` command | Must | - [ ] Sets the config key to the given value<br>- [ ] Creates config file if it does not exist<br>- [ ] Preserves existing keys |
| REQ-CONFIG-005 | `config unset <key>` command | Must | - [ ] Removes the specified key from config<br>- [ ] No error if key does not exist |
| REQ-CONFIG-006 | `config list` command (all key-value pairs) | Must | - [ ] Lists all config keys and values<br>- [ ] JSON mode outputs full config object |
| REQ-CONFIG-007 | `config keys` command (keys only) | Must | - [ ] Lists all valid config key names |
| REQ-CONFIG-008 | `config path` command (prints config file path) | Must | - [ ] Prints absolute path to config.json |
| REQ-CONFIG-009 | OAuth credential files at `$CONFIG_DIR/omega-google/credentials.json` (default) and `credentials-<client>.json` (named) | Must | - [ ] Files written with mode 0600<br>- [ ] Supports Google's `installed.client_id/client_secret` and `web.client_id/client_secret` JSON formats |

#### AUTH -- Authentication and Credential Management

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-AUTH-001 | `auth credentials <file>` stores OAuth client credentials | Must | - [ ] Reads credentials JSON from file or stdin (`-`)<br>- [ ] Validates `installed` or `web` format<br>- [ ] Writes to `credentials.json` (default client) or `credentials-<client>.json` (named) |
| REQ-AUTH-002 | `auth credentials list` lists stored credential files | Must | - [ ] Shows all stored credential files with client names<br>- [ ] JSON/plain/text output modes |
| REQ-AUTH-003 | `auth add <email>` performs OAuth2 flow and stores refresh token | Must | - [ ] Desktop OAuth redirect on ephemeral port<br>- [ ] Requests `access_type=offline` and `include_granted_scopes=true`<br>- [ ] Stores refresh token in OS keyring with key `token:<client>:<email>`<br>- [ ] Supports `--services` flag for scope selection<br>- [ ] Supports `--readonly` and `--drive-scope full/readonly/file` |
| REQ-AUTH-004 | `auth add --manual` browserless OAuth flow | Must | - [ ] Prints auth URL for manual navigation<br>- [ ] Accepts redirect URL paste from user<br>- [ ] Extracts authorization code and exchanges for tokens |
| REQ-AUTH-005 | `auth add --remote` two-step OAuth flow for headless servers | Must | - [ ] `--remote --step 1` prints auth URL with state<br>- [ ] `--remote --step 2 --auth-url <url>` exchanges URL with state validation<br>- [ ] State is cached in temporary file |
| REQ-AUTH-006 | `auth add --force-consent` forces consent prompt | Must | - [ ] Adds `prompt=consent` to OAuth URL<br>- [ ] Ensures new refresh token is issued |
| REQ-AUTH-007 | `auth remove <email>` deletes stored refresh token | Must | - [ ] Removes token from keyring<br>- [ ] Removes legacy key format if present<br>- [ ] Confirmation prompt unless `--force` |
| REQ-AUTH-008 | `auth list` lists all stored accounts | Must | - [ ] Shows email, client, services, scopes, created date<br>- [ ] JSON/plain/text output |
| REQ-AUTH-009 | `auth status` shows auth configuration summary | Must | - [ ] Shows config path, keyring backend, stored accounts<br>- [ ] Indicates if credentials are configured |
| REQ-AUTH-010 | `auth services` lists supported services with scopes | Must | - [ ] Shows all 15 services with OAuth scopes, APIs, and notes<br>- [ ] `--markdown` flag for markdown table output |
| REQ-AUTH-011 | `auth tokens list` and `auth tokens delete <email>` | Must | - [ ] `list` shows all keyring token keys<br>- [ ] `delete` removes specific token from keyring |
| REQ-AUTH-012 | `auth alias set/unset/list` manages account aliases | Must | - [ ] `set <alias> <email>` stores alias in config<br>- [ ] `unset <alias>` removes alias<br>- [ ] `list` shows all aliases<br>- [ ] Aliases resolve to emails in `--account` |
| REQ-AUTH-013 | OS keyring storage via `keyring` crate (macOS Keychain, Linux Secret Service) | Must | - [ ] Tokens stored in OS keyring with service name `omega-google`<br>- [ ] Key format: `token:<client>:<email>`<br>- [ ] Works on macOS (Keychain) and Linux (Secret Service) |
| REQ-AUTH-014 | File-based keyring fallback with password encryption | Must | - [ ] Falls back to encrypted file storage when OS keyring unavailable<br>- [ ] Directory: `$CONFIG_DIR/omega-google/keyring/`<br>- [ ] Password via TTY prompt or `GOG_KEYRING_PASSWORD` env var |
| REQ-AUTH-015 | `GOG_KEYRING_BACKEND` env and `keyring_backend` config support `auto/keychain/file` | Must | - [ ] `auto` = OS default with fallback<br>- [ ] `keychain` = force macOS Keychain<br>- [ ] `file` = force encrypted file backend<br>- [ ] Env var overrides config |
| REQ-AUTH-016 | Per-service OAuth scope mapping matching gogcli exactly | Must | - [ ] 15 services with correct default scopes<br>- [ ] `--readonly` variants for all applicable services<br>- [ ] `--drive-scope` variants (full/readonly/file) for Drive-dependent services<br>- [ ] Base scopes: `openid`, `email`, `userinfo.email` always included |
| REQ-AUTH-017 | Service account JWT auth for Keep and domain-wide delegation | Must | - [ ] `auth keep <email> --key <sa.json>` stores service account key<br>- [ ] `auth service-account set/unset/status` commands<br>- [ ] JWT token generation for domain-wide delegation |
| REQ-AUTH-018 | `auth keyring` command to view/change keyring backend | Should | - [ ] Shows current backend and source<br>- [ ] Allows switching backend |
| REQ-AUTH-019 | Account resolution order: `--account` flag > `GOG_ACCOUNT` env > keyring default > single stored token | Must | - [ ] Follows resolution order exactly<br>- [ ] `account_clients` config maps email to client<br>- [ ] `client_domains` config maps domain to client |
| REQ-AUTH-020 | Keyring timeout (5s) on Linux when D-Bus may be unresponsive | Should | - [ ] Timeout prevents indefinite hang<br>- [ ] Helpful error message with `GOG_KEYRING_BACKEND=file` suggestion<br>- [ ] Force file backend when no D-Bus session detected |

#### HTTP -- HTTP Client and Retry Logic

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-HTTP-001 | reqwest-based HTTP client with OAuth2 bearer token injection | Must | - [ ] Automatically adds `Authorization: Bearer <token>` header<br>- [ ] Token refresh on 401<br>- [ ] TLS 1.2+ enforced |
| REQ-HTTP-002 | Retry transport: exponential backoff with jitter for 429 responses | Must | - [ ] Max 3 retries for 429<br>- [ ] Base delay 1s with exponential growth (1s, 2s, 4s)<br>- [ ] Random jitter added (0-50% of base delay)<br>- [ ] Respects `Retry-After` header when present |
| REQ-HTTP-003 | Retry on 5xx server errors | Must | - [ ] Max 1 retry for 5xx<br>- [ ] 1s delay before retry<br>- [ ] No retry on 4xx (except 429) |
| REQ-HTTP-004 | Circuit breaker: open after 5 consecutive failures, reset after 30s | Must | - [ ] Tracks consecutive 5xx failures<br>- [ ] Opens circuit after threshold<br>- [ ] Returns `CircuitBreakerError` when open<br>- [ ] Resets after cooldown period<br>- [ ] Resets on any success |
| REQ-HTTP-005 | Request body replay for retries | Must | - [ ] Reads body into memory before first attempt<br>- [ ] Replays body on subsequent retry attempts<br>- [ ] Drains and closes response body before retry |
| REQ-HTTP-006 | Context cancellation support in retry sleep | Must | - [ ] Sleep can be interrupted by context cancellation<br>- [ ] Returns appropriate error on cancellation |

#### OUTPUT -- Output Formatting

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-OUTPUT-001 | Three output modes: JSON (`--json`), plain/TSV (`--plain`), human-friendly text (default) | Must | - [ ] JSON: structured JSON objects/arrays to stdout<br>- [ ] Plain: tab-separated values, no colors, stable for scripting<br>- [ ] Text: human-readable with alignment and colors |
| REQ-OUTPUT-002 | `--results-only` strips envelope fields (e.g., `nextPageToken`) from JSON output | Must | - [ ] In JSON mode, emits only the primary result array/object<br>- [ ] Drops pagination metadata |
| REQ-OUTPUT-003 | `--select field1,field2` filters JSON output to specified fields | Must | - [ ] Supports comma-separated field names<br>- [ ] Supports dot-path notation (e.g., `file.id`)<br>- [ ] Best-effort: missing fields silently omitted |
| REQ-OUTPUT-004 | `GOG_AUTO_JSON` enables automatic JSON mode when stdout is not a TTY | Must | - [ ] When env var is truthy and stdout is piped, default to JSON<br>- [ ] `--plain` can still override<br>- [ ] Only applies after parse phase |
| REQ-OUTPUT-005 | Colors disabled automatically for `--json` and `--plain` modes | Must | - [ ] No ANSI escape codes in JSON or plain output<br>- [ ] Colors only in human-friendly text mode |

#### UI -- Terminal User Interface

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-UI-001 | Colored terminal output using crossterm | Must | - [ ] Colors enabled when: TTY + `--color auto` + `NO_COLOR` not set, or `--color always`<br>- [ ] Colors disabled when: `--color never` or `NO_COLOR` set<br>- [ ] `GOG_COLOR` env var respected |
| REQ-UI-002 | Progress and hints written to stderr | Must | - [ ] All non-data output (progress, hints, warnings) goes to stderr<br>- [ ] `# Next page: --page <token>` hints on stderr |
| REQ-UI-003 | Error formatting with colored error messages | Must | - [ ] Errors formatted with context and colored prefix<br>- [ ] User-facing errors distinct from internal errors<br>- [ ] Google API errors include helpful messages |

---

### Milestone 2: Core Services (Gmail, Calendar, Drive)

#### GMAIL -- Gmail Service

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-GMAIL-001 | `gmail search <query>` -- search threads with Gmail query syntax | Must | - [ ] `--max`, `--page`, `--all`, `--fail-empty` flags<br>- [ ] `--oldest` shows first message date instead of last<br>- [ ] `--timezone` for date output<br>- [ ] JSON/plain/text output with thread ID, subject, date, snippet |
| REQ-GMAIL-002 | `gmail messages search <query>` -- search messages (not threads) | Must | - [ ] `--include-body` flag to include message body<br>- [ ] Pagination support<br>- [ ] JSON/plain/text output |
| REQ-GMAIL-003 | `gmail thread get <threadId>` -- get thread with all messages | Must | - [ ] Shows all messages in thread<br>- [ ] `--download` flag to download all attachments<br>- [ ] JSON/plain/text output |
| REQ-GMAIL-004 | `gmail thread modify <threadId>` -- add/remove labels on thread | Must | - [ ] `--add` and `--remove` flags for label IDs<br>- [ ] JSON/plain/text output |
| REQ-GMAIL-005 | `gmail thread attachments <threadId>` -- download thread attachments | Must | - [ ] Downloads all attachments from all messages<br>- [ ] `--out-dir` flag for output directory |
| REQ-GMAIL-006 | `gmail get <messageId>` -- get single message | Must | - [ ] `--format full/metadata/raw` flag<br>- [ ] `--headers` for specific header fields<br>- [ ] JSON/plain/text output |
| REQ-GMAIL-007 | `gmail attachment <messageId> <attachmentId>` -- download attachment | Must | - [ ] `--out` and `--name` flags for output path<br>- [ ] Binary download to file |
| REQ-GMAIL-008 | `gmail url <threadIds...>` -- print Gmail web URLs | Must | - [ ] Accepts multiple thread IDs<br>- [ ] Outputs `https://mail.google.com/mail/u/0/#all/<threadId>` |
| REQ-GMAIL-009 | `gmail labels list/get/create/modify/delete` -- label management | Must | - [ ] `list` shows all labels with ID, name, type<br>- [ ] `get <labelIdOrName>` resolves by name or ID<br>- [ ] `create <name>` creates a new label<br>- [ ] `modify <threadIds...> --add/--remove` batch label modification<br>- [ ] `delete <labelId>` deletes a label |
| REQ-GMAIL-010 | `gmail send` -- send email with full feature set | Must | - [ ] `--to`, `--subject`, `--body`, `--body-html` flags<br>- [ ] `--cc`, `--bcc` for additional recipients<br>- [ ] `--reply-to-message-id` for threading<br>- [ ] `--reply-to` for reply-to header<br>- [ ] `--attach <file>...` for attachments<br>- [ ] `--track` for open tracking integration |
| REQ-GMAIL-011 | `gmail drafts list/get/create/update/send/delete` -- draft management | Must | - [ ] Full CRUD for drafts<br>- [ ] Same compose flags as send (to, subject, body, cc, bcc, attach)<br>- [ ] `send <draftId>` sends a draft<br>- [ ] `--download` on get for attachments |
| REQ-GMAIL-012 | `gmail watch start/status/renew/stop/serve` -- Gmail push notifications | Must | - [ ] `start` initiates Pub/Sub watch with topic and label filters<br>- [ ] `status` shows current watch state<br>- [ ] `renew` refreshes the watch<br>- [ ] `stop` cancels the watch<br>- [ ] `serve` runs a local webhook server |
| REQ-GMAIL-013 | `gmail history --since <historyId>` -- history change list | Must | - [ ] Lists message changes since history ID<br>- [ ] Pagination support<br>- [ ] JSON/plain/text output |
| REQ-GMAIL-014 | `gmail batch delete/modify` -- batch operations on messages | Must | - [ ] `delete <messageIds...>` permanently deletes messages<br>- [ ] `modify <messageIds...> --add/--remove` batch label changes |
| REQ-GMAIL-015 | `gmail settings filters list/get/create/delete` -- email filter management | Must | - [ ] Full CRUD for Gmail filters<br>- [ ] Filter criteria: from, to, subject, query, negatedQuery, size, hasAttachment<br>- [ ] Filter actions: label add/remove, archive, star, forward, trash, etc. |
| REQ-GMAIL-016 | `gmail settings forwarding list/get/create/delete` -- forwarding addresses | Must | - [ ] Full CRUD for forwarding addresses |
| REQ-GMAIL-017 | `gmail settings sendas list/get/create/verify/delete/update` -- send-as aliases | Must | - [ ] Full CRUD plus verify for send-as addresses |
| REQ-GMAIL-018 | `gmail settings delegates list/get/add/remove` -- delegation | Must | - [ ] Full CRUD for delegate management |
| REQ-GMAIL-019 | `gmail settings vacation get/update` -- vacation responder | Must | - [ ] Get and set vacation responder with enable/disable, subject, body, date range |
| REQ-GMAIL-020 | `gmail settings autoforward get/update` -- auto-forwarding | Must | - [ ] Get and set auto-forwarding configuration |

#### CAL -- Calendar Service

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-CAL-001 | `calendar calendars` -- list all calendars | Must | - [ ] Shows ID, summary, access role, primary flag<br>- [ ] Pagination support<br>- [ ] JSON/plain/text output |
| REQ-CAL-002 | `calendar acl <calendarId>` -- list calendar ACL entries | Must | - [ ] Shows role, scope type, scope value<br>- [ ] JSON/plain/text output |
| REQ-CAL-003 | `calendar events` -- list events with rich filtering | Must | - [ ] `--cal` for calendar ID or name resolution<br>- [ ] `--calendars` for multiple calendars (CSV)<br>- [ ] `--all` for events from all calendars<br>- [ ] `--from`, `--to` with flexible date/time parsing (RFC3339, relative, weekday names)<br>- [ ] `--max`, `--page`, `--query`, `--weekday` flags<br>- [ ] `--fields` passes to Calendar API (NOT rewritten to `--select`) |
| REQ-CAL-004 | `calendar event <calendarId> <eventId>` -- get single event | Must | - [ ] Full event details<br>- [ ] JSON/plain/text output |
| REQ-CAL-005 | `calendar create` -- create event | Must | - [ ] `--summary`, `--from`, `--to`, `--description`, `--location`<br>- [ ] `--attendees` for comma-separated emails<br>- [ ] `--all-day` for date-only events<br>- [ ] `--event-type` for focus-time, OOO, working-location |
| REQ-CAL-006 | `calendar update <calendarId> <eventId>` -- update event | Must | - [ ] All fields from create as optional overrides<br>- [ ] `--add-attendee` to add without replacing |
| REQ-CAL-007 | `calendar delete <calendarId> <eventId>` -- delete event | Must | - [ ] Confirmation prompt unless `--force`<br>- [ ] Dry-run support |
| REQ-CAL-008 | `calendar freebusy <calendarIds> --from --to` -- free/busy query | Must | - [ ] Accepts multiple calendar IDs<br>- [ ] Shows busy time ranges<br>- [ ] JSON/plain/text output |
| REQ-CAL-009 | `calendar respond <calendarId> <eventId> --status accepted/declined/tentative` -- RSVP | Must | - [ ] `--send-updates all/none/externalOnly` flag |
| REQ-CAL-010 | `calendar search` -- search events across calendars | Must | - [ ] Full-text query across events<br>- [ ] Calendar selection via `--cal`, `--calendars`, `--all` |
| REQ-CAL-011 | `calendar time` -- show server time | Must | - [ ] Current server time in configured timezone |
| REQ-CAL-012 | `calendar users` -- list Workspace users (emails as calendar IDs) | Must | - [ ] Lists workspace users for calendar lookup |
| REQ-CAL-013 | `calendar team` -- show events for all members of a Google Group | Must | - [ ] Takes group email, resolves members, fetches calendars<br>- [ ] `--from`, `--to` for time range |
| REQ-CAL-014 | `calendar colors` -- show calendar color definitions | Must | - [ ] Lists all available calendar colors<br>- [ ] JSON/plain/text output |
| REQ-CAL-015 | `calendar conflicts` -- find scheduling conflicts | Must | - [ ] Finds overlapping events in a time range<br>- [ ] Multi-calendar support |
| REQ-CAL-016 | `calendar propose-time <calendarId> <eventId>` -- generate propose-time URL | Should | - [ ] Generates browser URL for proposing a new time<br>- [ ] Offline operation (no API call needed) |
| REQ-CAL-017 | `calendar focus-time` -- create Focus Time blocks | Should | - [ ] Creates event with type `focusTime`<br>- [ ] Appropriate visibility and status |
| REQ-CAL-018 | `calendar out-of-office` -- create OOO events | Should | - [ ] Creates event with type `outOfOffice`<br>- [ ] Full-day or time-ranged |
| REQ-CAL-019 | `calendar working-location` -- set working location | Should | - [ ] Creates event with type `workingLocation`<br>- [ ] Supports home/office/custom |
| REQ-CAL-020 | Flexible date/time parsing for all calendar commands | Must | - [ ] RFC3339, `YYYY-MM-DD`, `YYYY-MM-DDTHH:MM:SS`<br>- [ ] Relative: `now`, `today`, `tomorrow`, `yesterday`<br>- [ ] Weekday names: `monday`, `next friday`<br>- [ ] Duration support for `--since`: `24h`, `7d` |
| REQ-CAL-021 | Recurrence rule display and handling | Should | - [ ] Displays RRULE in human-readable format<br>- [ ] Handles recurring event instances |
| REQ-CAL-022 | Day-of-week enrichment for event listings (`--weekday` / `GOG_CALENDAR_WEEKDAY`) | Should | - [ ] Shows day of week alongside dates when enabled |

#### DRIVE -- Drive Service

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-DRIVE-001 | `drive ls` -- list files in a folder | Must | - [ ] Default: root folder<br>- [ ] `--parent ID` for specific folder<br>- [ ] `--max`, `--page`, `--query`, `--all-drives` flags<br>- [ ] Shows ID, name, mimeType, size, modified date |
| REQ-DRIVE-002 | `drive search <text>` -- full-text search across Drive | Must | - [ ] `--raw-query` for raw Drive query syntax<br>- [ ] `--max`, `--page`, `--all-drives`<br>- [ ] JSON/plain/text output |
| REQ-DRIVE-003 | `drive get <fileId>` -- get file metadata | Must | - [ ] Full file metadata<br>- [ ] JSON/plain/text output |
| REQ-DRIVE-004 | `drive download <fileId>` -- download file with format conversion | Must | - [ ] `--out PATH` for output location<br>- [ ] `--format` for Google Workspace files (pdf, docx, xlsx, pptx, csv, txt)<br>- [ ] Binary files download directly<br>- [ ] Google Docs/Sheets/Slides export via Drive API |
| REQ-DRIVE-005 | `drive upload <localPath>` -- upload file with optional conversion | Must | - [ ] `--name`, `--parent` flags<br>- [ ] `--convert` to convert to Google format<br>- [ ] `--convert-to doc/sheet/slides` for specific conversion |
| REQ-DRIVE-006 | `drive mkdir <name>` -- create folder | Must | - [ ] `--parent ID` flag<br>- [ ] Returns folder ID and link |
| REQ-DRIVE-007 | `drive delete <fileId>` -- trash or permanently delete | Must | - [ ] Default: move to trash<br>- [ ] `--permanent` for permanent deletion<br>- [ ] Confirmation prompt unless `--force` |
| REQ-DRIVE-008 | `drive move <fileId> --parent ID` -- move file to folder | Must | - [ ] Moves file from current parent to specified parent |
| REQ-DRIVE-009 | `drive rename <fileId> <newName>` -- rename file | Must | - [ ] Updates file name |
| REQ-DRIVE-010 | `drive share <fileId>` -- share with permissions | Must | - [ ] `--to anyone/user/domain`<br>- [ ] `--email`, `--domain`, `--role reader/writer`, `--discoverable`<br>- [ ] Creates permission on file |
| REQ-DRIVE-011 | `drive permissions <fileId>` -- list permissions | Must | - [ ] `--max`, `--page` pagination<br>- [ ] Shows role, type, email/domain |
| REQ-DRIVE-012 | `drive unshare <fileId> <permissionId>` -- remove permission | Must | - [ ] Deletes specific permission |
| REQ-DRIVE-013 | `drive url <fileIds...>` -- print web URLs | Must | - [ ] Generates `https://drive.google.com/open?id=<id>` for each file |
| REQ-DRIVE-014 | `drive drives` -- list shared drives | Must | - [ ] `--max`, `--page`, `--query` flags<br>- [ ] Shows shared drive names and IDs |
| REQ-DRIVE-015 | `drive copy <fileId>` -- copy a file | Must | - [ ] Creates a copy of the file<br>- [ ] `--name`, `--parent` flags |
| REQ-DRIVE-016 | `drive comments list/get/create/update/delete/reply` -- file comments | Should | - [ ] Full comment CRUD on Drive files<br>- [ ] Reply to comments<br>- [ ] `--include-quoted` for anchored text |
| REQ-DRIVE-017 | `--all-drives` defaults to true, negatable with `--no-all-drives` | Must | - [ ] Includes shared drives by default<br>- [ ] `--no-all-drives` restricts to My Drive only |

#### CLI -- Desire Path Aliases (M2)

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-CLI-010 | `send` -- alias for `gmail send` | Must | - [ ] All flags from `gmail send` available<br>- [ ] Identical behavior |
| REQ-CLI-011 | `ls` -- alias for `drive ls` | Must | - [ ] All flags from `drive ls` available |
| REQ-CLI-012 | `search` -- alias for `drive search` | Must | - [ ] All flags from `drive search` available |
| REQ-CLI-013 | `download` -- alias for `drive download` | Must | - [ ] All flags from `drive download` available |
| REQ-CLI-014 | `upload` -- alias for `drive upload` | Must | - [ ] All flags from `drive upload` available |
| REQ-CLI-015 | `login` -- alias for `auth add` | Must | - [ ] All flags from `auth add` available |
| REQ-CLI-016 | `logout` -- alias for `auth remove` | Must | - [ ] All flags from `auth remove` available |
| REQ-CLI-017 | `status` -- alias for `auth status` | Must | - [ ] All flags from `auth status` available |
| REQ-CLI-018 | `me` / `whoami` -- alias for `people me` | Must | - [ ] Shows current user profile |
| REQ-CLI-019 | `open <target>` -- offline URL generation for Google resource IDs | Must | - [ ] `--type auto/drive/folder/docs/sheets/slides/gmail-thread`<br>- [ ] Generates web URLs from IDs without API calls<br>- [ ] Supports URL canonicalization |

---

### Milestone 3: Productivity Services (Docs, Sheets, Slides, Forms)

#### DOCS -- Google Docs Service

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-DOCS-001 | `docs export <docId>` -- export as PDF/DOCX/TXT | Must | - [ ] `--format pdf/docx/txt` (default: pdf)<br>- [ ] `--out PATH` for output location<br>- [ ] Uses Drive API export |
| REQ-DOCS-002 | `docs info <docId>` -- get doc metadata | Must | - [ ] Shows ID, title, revision, mime type, web link |
| REQ-DOCS-003 | `docs create <title>` -- create new Google Doc | Must | - [ ] `--parent` for folder placement<br>- [ ] `--file` for importing markdown content<br>- [ ] Image extraction and insertion from markdown |
| REQ-DOCS-004 | `docs copy <docId> <title>` -- copy a document | Must | - [ ] `--parent` for destination folder |
| REQ-DOCS-005 | `docs cat <docId>` -- extract plain text | Must | - [ ] `--max-bytes` limit (default 2MB)<br>- [ ] `--tab` for specific tab<br>- [ ] `--all-tabs` for all tabs with headers<br>- [ ] `--raw` for raw API JSON response |
| REQ-DOCS-006 | `docs list-tabs <docId>` -- list document tabs | Should | - [ ] Shows tab ID, title, index, nesting |
| REQ-DOCS-007 | `docs comments list/get/add/reply/resolve/delete` -- comment management | Must | - [ ] Full CRUD for document comments<br>- [ ] Reply to comments<br>- [ ] Resolve (mark as done) |
| REQ-DOCS-008 | `docs write <docId>` -- write content (append or replace) | Must | - [ ] Positional content or `--file` or stdin<br>- [ ] `--replace` to replace all content<br>- [ ] `--markdown` to convert markdown to Google Docs formatting |
| REQ-DOCS-009 | `docs insert <docId>` -- insert text at specific position | Must | - [ ] `--index` for character position (default: 1)<br>- [ ] Content from argument, `--file`, or stdin |
| REQ-DOCS-010 | `docs delete <docId>` -- delete text range | Must | - [ ] `--start` and `--end` for character range |
| REQ-DOCS-011 | `docs find-replace <docId> <find> <replace>` -- find and replace text | Must | - [ ] `--match-case` flag<br>- [ ] Reports number of replacements |
| REQ-DOCS-012 | `docs edit <docId>` -- find/replace with named flags | Must | - [ ] `--find` and `--replace` flags<br>- [ ] `--match-case` (default: true) |
| REQ-DOCS-013 | `docs update <docId>` -- update content with format support | Must | - [ ] `--content` or `--content-file`<br>- [ ] `--format plain/markdown`<br>- [ ] `--append` flag<br>- [ ] Markdown formatting converts to Google Docs formatting requests |
| REQ-DOCS-014 | `docs sed <docId> <expression>` -- sed-like regex find/replace | Must | - [ ] `s/pattern/replacement/flags` syntax<br>- [ ] `-e` for multiple expressions<br>- [ ] `-f` for expressions from file<br>- [ ] Stdin support for expressions<br>- [ ] Regex support with backreferences |
| REQ-DOCS-015 | `docs clear <docId>` -- clear all document content | Must | - [ ] Removes all content from document body |
| REQ-DOCS-016 | Markdown-to-Docs formatting engine | Should | - [ ] Converts markdown headings, bold, italic, links to Docs API requests<br>- [ ] Table creation from markdown tables<br>- [ ] Image insertion support |

#### SHEETS -- Google Sheets Service

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-SHEETS-001 | `sheets get <spreadsheetId> <range>` -- read cell values | Must | - [ ] A1 notation for range (e.g., `Sheet1!A1:B10`)<br>- [ ] `--dimension ROWS/COLUMNS`<br>- [ ] `--render FORMATTED_VALUE/UNFORMATTED_VALUE/FORMULA`<br>- [ ] Shell escape handling for `!` in range |
| REQ-SHEETS-002 | `sheets update <spreadsheetId> <range>` -- write cell values | Must | - [ ] Values as args (pipe-separated cells, comma-separated rows) or `--values-json`<br>- [ ] `--input RAW/USER_ENTERED`<br>- [ ] `--copy-validation-from` for data validation<br>- [ ] Dry-run support |
| REQ-SHEETS-003 | `sheets append <spreadsheetId> <range>` -- append rows | Must | - [ ] Same value input as update<br>- [ ] `--insert OVERWRITE/INSERT_ROWS`<br>- [ ] `--copy-validation-from` for data validation |
| REQ-SHEETS-004 | `sheets insert <spreadsheetId> <sheet> <dimension> <start>` -- insert rows/columns | Must | - [ ] `dimension` = rows or cols<br>- [ ] `--count` for number to insert (default: 1)<br>- [ ] `--after` to insert after position |
| REQ-SHEETS-005 | `sheets clear <spreadsheetId> <range>` -- clear cell values | Must | - [ ] Clears values in specified range<br>- [ ] Dry-run support |
| REQ-SHEETS-006 | `sheets format <spreadsheetId> <range>` -- apply cell formatting | Must | - [ ] `--format-json` for Sheets API CellFormat JSON<br>- [ ] `--format-fields` for field mask |
| REQ-SHEETS-007 | `sheets notes <spreadsheetId> <range>` -- read cell notes | Should | - [ ] Shows notes for cells in range |
| REQ-SHEETS-008 | `sheets metadata <spreadsheetId>` -- get spreadsheet metadata | Must | - [ ] Shows title, locale, timezone, sheets with dimensions |
| REQ-SHEETS-009 | `sheets create <title>` -- create new spreadsheet | Must | - [ ] `--sheets` for comma-separated sheet names<br>- [ ] Dry-run support |
| REQ-SHEETS-010 | `sheets copy <spreadsheetId> <title>` -- copy spreadsheet | Must | - [ ] `--parent` for destination folder |
| REQ-SHEETS-011 | `sheets export <spreadsheetId>` -- export as PDF/XLSX/CSV | Must | - [ ] `--format pdf/xlsx/csv` (default: xlsx)<br>- [ ] `--out PATH` for output |
| REQ-SHEETS-012 | A1 notation parsing and validation | Must | - [ ] Handles `Sheet1!A1:B10`, `A:C`, `1:5`, etc.<br>- [ ] Shell escape handling (`\!` -> `!`) |

#### SLIDES -- Google Slides Service

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-SLIDES-001 | `slides export <presentationId>` -- export as PDF/PPTX | Must | - [ ] `--format pdf/pptx` (default: pptx)<br>- [ ] `--out PATH` for output |
| REQ-SLIDES-002 | `slides info <presentationId>` -- get presentation metadata | Must | - [ ] Shows ID, name, mime type, web link |
| REQ-SLIDES-003 | `slides create <title>` -- create new presentation | Must | - [ ] `--parent` for folder placement<br>- [ ] `--template` for copying from template |
| REQ-SLIDES-004 | `slides create-from-markdown` -- create presentation from markdown | Should | - [ ] `--content` or `--content-file` for markdown<br>- [ ] Parses markdown into slides<br>- [ ] `--parent` for folder placement |
| REQ-SLIDES-005 | `slides copy <presentationId> <title>` -- copy presentation | Must | - [ ] `--parent` for destination folder |
| REQ-SLIDES-006 | `slides list-slides <presentationId>` -- list all slides | Must | - [ ] Shows slide object IDs and indices |
| REQ-SLIDES-007 | `slides add-slide <presentationId>` -- add slide with image and notes | Must | - [ ] Full-bleed image support<br>- [ ] Speaker notes |
| REQ-SLIDES-008 | `slides delete-slide <presentationId> <slideId>` -- delete a slide | Must | - [ ] Removes slide by object ID |
| REQ-SLIDES-009 | `slides read-slide <presentationId> <slideId>` -- read slide content | Must | - [ ] Shows speaker notes, text elements, images |
| REQ-SLIDES-010 | `slides update-notes <presentationId> <slideId>` -- update speaker notes | Must | - [ ] Sets or replaces speaker notes text |
| REQ-SLIDES-011 | `slides replace-slide <presentationId> <slideId>` -- replace slide image | Should | - [ ] Replaces the image on an existing slide in-place |

#### FORMS -- Google Forms Service

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-FORMS-001 | `forms get <formId>` -- get form metadata | Must | - [ ] Shows ID, title, description, responder URI, edit URL |
| REQ-FORMS-002 | `forms create --title <title>` -- create a form | Must | - [ ] `--description` optional<br>- [ ] Dry-run support |
| REQ-FORMS-003 | `forms responses list <formId>` -- list form responses | Must | - [ ] `--max`, `--page`, `--filter` flags<br>- [ ] Shows response ID, submitted time, email |
| REQ-FORMS-004 | `forms responses get <formId> <responseId>` -- get single response | Must | - [ ] Full response details with answers |

---

### Milestone 4: Collaboration Services (Chat, Classroom, Tasks, Contacts, People)

#### CHAT -- Google Chat Service

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-CHAT-001 | `chat spaces list` -- list chat spaces | Must | - [ ] `--max`, `--page` pagination<br>- [ ] JSON/plain/text output |
| REQ-CHAT-002 | `chat spaces find <displayName>` -- find spaces by name | Must | - [ ] `--max` for result limit |
| REQ-CHAT-003 | `chat spaces create <displayName>` -- create a space | Must | - [ ] `--member email,...` for initial members |
| REQ-CHAT-004 | `chat messages list <space>` -- list messages in a space | Must | - [ ] `--max`, `--page`, `--order` flags<br>- [ ] `--thread THREAD` for thread filter<br>- [ ] `--unread` for unread messages only |
| REQ-CHAT-005 | `chat messages send <space> --text TEXT` -- send message | Must | - [ ] `--thread THREAD` for threading |
| REQ-CHAT-006 | `chat threads list <space>` -- list threads in a space | Must | - [ ] `--max`, `--page` pagination |
| REQ-CHAT-007 | `chat dm space <email>` -- get/create DM space | Must | - [ ] Returns DM space for the given email |
| REQ-CHAT-008 | `chat dm send <email> --text TEXT` -- send DM | Must | - [ ] `--thread` for threading<br>- [ ] Workspace-only feature |

#### CLASS -- Google Classroom Service

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-CLASS-001 | `classroom courses list/get/create/update/delete/archive/unarchive/join/leave/url` -- full course management | Must | - [ ] `list` with `--state`, `--max`, `--page` filters<br>- [ ] `get <courseId>` shows full course details<br>- [ ] `create --name NAME` with optional `--owner`, `--state`<br>- [ ] `update <courseId>` with `--name`, `--state` overrides<br>- [ ] `delete <courseId>` with confirmation<br>- [ ] `archive/unarchive <courseId>` state changes<br>- [ ] `join/leave <courseId>` with `--role student/teacher` and `--user me`<br>- [ ] `url <courseId...>` prints Classroom URLs |
| REQ-CLASS-002 | `classroom students list/get/add/remove` -- student roster management | Must | - [ ] `list <courseId>` with pagination<br>- [ ] `add <courseId> <userId>` with `--enrollment-code`<br>- [ ] `remove <courseId> <userId>` |
| REQ-CLASS-003 | `classroom teachers list/get/add/remove` -- teacher roster management | Must | - [ ] Same pattern as students |
| REQ-CLASS-004 | `classroom roster <courseId>` -- combined roster view | Must | - [ ] `--students`, `--teachers` filters |
| REQ-CLASS-005 | `classroom coursework list/get/create/update/delete/assignees` -- assignment management | Must | - [ ] `list <courseId>` with `--state`, `--topic`, `--scan-pages`, `--max`, `--page`<br>- [ ] `create <courseId> --title TITLE` with `--type ASSIGNMENT/...`<br>- [ ] `assignees <courseId> <courseworkId>` with `--mode`, `--add-student` |
| REQ-CLASS-006 | `classroom materials list/get/create/update/delete` -- course materials | Must | - [ ] Full CRUD for course materials<br>- [ ] Filtering by state and topic |
| REQ-CLASS-007 | `classroom submissions list/get/turn-in/reclaim/return/grade` -- submission management | Must | - [ ] `list <courseId> <courseworkId>` with `--state`<br>- [ ] `turn-in/reclaim/return` state transitions<br>- [ ] `grade` with `--draft N`, `--assigned N` scores |
| REQ-CLASS-008 | `classroom announcements list/get/create/update/delete/assignees` -- announcements | Must | - [ ] Full CRUD for announcements<br>- [ ] `assignees` for audience management |
| REQ-CLASS-009 | `classroom topics list/get/create/update/delete` -- topic management | Must | - [ ] Full CRUD for course topics |
| REQ-CLASS-010 | `classroom invitations list/get/create/accept/delete` -- course invitations | Must | - [ ] `create <courseId> <userId> --role STUDENT/TEACHER/OWNER`<br>- [ ] `accept <invitationId>` |
| REQ-CLASS-011 | `classroom guardians list/get/delete` -- guardian management | Must | - [ ] `list <studentId>`, `get <studentId> <guardianId>`<br>- [ ] `delete <studentId> <guardianId>` |
| REQ-CLASS-012 | `classroom guardian-invitations list/get/create` -- guardian invitations | Must | - [ ] `list <studentId>` with `--state`<br>- [ ] `create <studentId> --email EMAIL` |
| REQ-CLASS-013 | `classroom profile [userId]` -- user profile view | Must | - [ ] Shows name, email, photo URL |

#### TASKS -- Google Tasks Service

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-TASKS-001 | `tasks lists` -- list all task lists | Must | - [ ] `--max`, `--page` pagination |
| REQ-TASKS-002 | `tasks lists create <title>` -- create task list | Must | - [ ] Returns task list ID |
| REQ-TASKS-003 | `tasks list <tasklistId>` -- list tasks in a list | Must | - [ ] `--max`, `--page` pagination<br>- [ ] Shows title, status, due date, notes |
| REQ-TASKS-004 | `tasks get <tasklistId> <taskId>` -- get single task | Must | - [ ] Full task details |
| REQ-TASKS-005 | `tasks add <tasklistId>` -- add a task | Must | - [ ] `--title`, `--notes`, `--due` (RFC3339 or YYYY-MM-DD)<br>- [ ] `--repeat daily/weekly/monthly/yearly` with `--repeat-count`, `--repeat-until`<br>- [ ] `--parent`, `--previous` for ordering |
| REQ-TASKS-006 | `tasks update <tasklistId> <taskId>` -- update task | Must | - [ ] `--title`, `--notes`, `--due`, `--status needsAction/completed` |
| REQ-TASKS-007 | `tasks done <tasklistId> <taskId>` -- mark complete | Must | - [ ] Sets status to `completed` |
| REQ-TASKS-008 | `tasks undo <tasklistId> <taskId>` -- mark incomplete | Must | - [ ] Sets status to `needsAction` |
| REQ-TASKS-009 | `tasks delete <tasklistId> <taskId>` -- delete task | Must | - [ ] Confirmation unless `--force` |
| REQ-TASKS-010 | `tasks clear <tasklistId>` -- clear completed tasks | Must | - [ ] Removes all completed tasks from list |

#### CONTACTS -- Google Contacts Service

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-CONTACTS-001 | `contacts search <query>` -- search contacts | Must | - [ ] `--max` for result limit |
| REQ-CONTACTS-002 | `contacts list` -- list all contacts | Must | - [ ] `--max`, `--page` pagination |
| REQ-CONTACTS-003 | `contacts get <resourceName or email>` -- get single contact | Must | - [ ] Accepts `people/...` resource name or email |
| REQ-CONTACTS-004 | `contacts create` -- create a contact | Must | - [ ] `--given`, `--family`, `--email`, `--phone` flags |
| REQ-CONTACTS-005 | `contacts update <resourceName>` -- update a contact | Must | - [ ] `--given`, `--family`, `--email`, `--phone`, `--birthday`, `--notes` flags<br>- [ ] `--from-file PATH/-` for JSON update from file<br>- [ ] `--ignore-etag` to skip concurrency check |
| REQ-CONTACTS-006 | `contacts delete <resourceName>` -- delete a contact | Must | - [ ] Confirmation unless `--force` |
| REQ-CONTACTS-007 | `contacts directory list/search` -- Workspace directory | Must | - [ ] `list` with pagination<br>- [ ] `search <query>` with `--max` |
| REQ-CONTACTS-008 | `contacts other list/search` -- other contacts (read suggestions) | Must | - [ ] `list` with pagination<br>- [ ] `search <query>` with `--max` |

#### PEOPLE -- Google People Service

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-PEOPLE-001 | `people me` -- show authenticated user profile | Must | - [ ] Shows name, email, photo, locale |
| REQ-PEOPLE-002 | `people get <resourceName or userId>` -- get a person's profile | Must | - [ ] Accepts `people/...` or user ID |
| REQ-PEOPLE-003 | `people search <query>` -- search people | Must | - [ ] `--max`, `--page` pagination |
| REQ-PEOPLE-004 | `people relations` -- show user relations | Must | - [ ] Optional `<resourceName>` argument<br>- [ ] `--type TYPE` filter |

---

### Milestone 5: Admin/Workspace Services (Groups, Keep, Apps Script)

#### GROUPS -- Google Groups (Cloud Identity)

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-GROUPS-001 | `groups list` -- list groups user belongs to | Must | - [ ] `--max`, `--page`, `--all`, `--fail-empty` flags<br>- [ ] Uses Cloud Identity API transitive group search<br>- [ ] Shows group email, display name, relation type |
| REQ-GROUPS-002 | `groups members <groupEmail>` -- list group members | Must | - [ ] Looks up group by email, then lists memberships<br>- [ ] Shows email, role (OWNER/MANAGER/MEMBER), type<br>- [ ] `--max`, `--page`, `--all`, `--fail-empty` flags |
| REQ-GROUPS-003 | Helpful error messages for Cloud Identity issues | Should | - [ ] Consumer account rejection detected and explained<br>- [ ] API not enabled: link to enable page<br>- [ ] Insufficient scopes: guidance to re-auth |

#### KEEP -- Google Keep (Workspace Only)

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-KEEP-001 | `keep list` -- list notes | Must | - [ ] `--max`, `--page`, `--all`, `--fail-empty` flags<br>- [ ] `--filter` for Keep API filter expressions<br>- [ ] Shows name, title, updated time |
| REQ-KEEP-002 | `keep get <noteId>` -- get a note | Must | - [ ] Accepts note ID or `notes/...` format<br>- [ ] Shows title, body text, attachments |
| REQ-KEEP-003 | `keep search <query>` -- client-side text search | Must | - [ ] Fetches all notes, filters by title/body text match<br>- [ ] `--max` for fetch limit |
| REQ-KEEP-004 | `keep attachment <attachmentName>` -- download attachment | Must | - [ ] `--mime-type`, `--out` flags<br>- [ ] Downloads to specified path<br>- [ ] Dry-run support |
| REQ-KEEP-005 | Service account auth required for Keep | Must | - [ ] `--service-account` and `--impersonate` flags on keep command<br>- [ ] Falls back to stored service account keys<br>- [ ] Helpful error if no service account configured |

#### SCRIPT -- Google Apps Script

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-SCRIPT-001 | `appscript get <scriptId>` -- get project metadata | Must | - [ ] Shows script ID, title, parent ID, timestamps, editor URL<br>- [ ] Accepts URLs or IDs (normalizes via `normalizeGoogleID`) |
| REQ-SCRIPT-002 | `appscript content <scriptId>` -- get project source files | Must | - [ ] Lists files with name and type |
| REQ-SCRIPT-003 | `appscript run <scriptId> <function>` -- run a deployed function | Must | - [ ] `--params` for JSON array of parameters<br>- [ ] `--dev-mode` for running saved (not deployed) code<br>- [ ] Shows execution result or error details |
| REQ-SCRIPT-004 | `appscript create --title <title>` -- create a new project | Must | - [ ] `--parent-id` for binding to a Drive file<br>- [ ] Dry-run support |

---

### Milestone 6: Polish (Tracking, Completion, Allowlisting, Agent Mode, Tests)

#### TRACK -- Email Tracking

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-TRACK-001 | `gmail track setup` -- configure email tracking with Cloudflare Worker | Could | - [ ] `--worker-url`, `--worker-name`, `--tracking-key` flags<br>- [ ] Generates AES-GCM encryption key if not provided<br>- [ ] Optional Cloudflare Worker deployment<br>- [ ] Saves tracking config and secrets |
| REQ-TRACK-002 | `gmail track opens <trackingId>` -- query tracking opens | Could | - [ ] Shows open events with timestamp, IP, location<br>- [ ] Admin mode for all tracking data |
| REQ-TRACK-003 | `gmail track status` -- show tracking configuration | Could | - [ ] Shows enabled/disabled, worker URL, config path |
| REQ-TRACK-004 | `gmail send --track` -- tracking pixel injection | Could | - [ ] Generates tracking pixel URL with AES-GCM encrypted payload<br>- [ ] Injects 1x1 pixel HTML into email body<br>- [ ] Returns tracking ID in send output |

#### AGENT -- Agent Mode and Machine Interface

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-AGENT-001 | `agent exit-codes` / `exit-codes` command -- print exit code table | Must | - [ ] Lists all exit codes with names and values<br>- [ ] JSON/plain/text output |
| REQ-AGENT-002 | `schema` / `help-json` command -- machine-readable command tree | Must | - [ ] Outputs full CLI schema as JSON<br>- [ ] Includes commands, subcommands, flags, positionals, types, defaults<br>- [ ] `--include-hidden` flag<br>- [ ] Optional `<command>` argument for subtree |
| REQ-AGENT-003 | `--enable-commands` enforcement for sandboxed runs | Must | - [ ] Restricts CLI to comma-separated top-level commands<br>- [ ] Rejects non-allowed commands with error<br>- [ ] `GOG_ENABLE_COMMANDS` env var support |

#### CLI -- Shell Completion and Final Polish

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-CLI-020 | `completion <shell>` -- generate shell completion scripts | Must | - [ ] Supports `bash`, `zsh`, `fish`, `powershell`<br>- [ ] Outputs completion script to stdout |
| REQ-CLI-021 | `--dry-run` / `-n` support for all destructive operations | Must | - [ ] Prints intended action as JSON and exits successfully<br>- [ ] No API calls made in dry-run mode<br>- [ ] Available on: delete, send, create, update, share, etc. |
| REQ-CLI-022 | `--force` / `-y` skips all confirmation prompts | Must | - [ ] Applies to destructive commands (delete, remove, clear) |
| REQ-CLI-023 | `--no-input` fails instead of prompting | Must | - [ ] Returns error when prompt would be shown<br>- [ ] For CI/automation environments |
| REQ-CLI-024 | Pagination: `--max`, `--page`, `--all`, `--fail-empty` standard flags | Must | - [ ] Consistent across all list commands<br>- [ ] `--all` fetches all pages with loop detection<br>- [ ] `--fail-empty` exits with code 3 if no results |

#### OUTPUT -- CSV Output (M6)

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-OUTPUT-006 | CSV output mode where applicable | Could | - [ ] Activated by `--csv` or `--format csv` flag<br>- [ ] Proper CSV escaping and quoting |

---

## Acceptance Criteria (detailed)

### REQ-SCAFFOLD-001: Cargo Workspace
- [ ] Given a clean checkout, when `cargo build --release` is run, then the `omega-google` binary is produced
- [ ] Given the binary, when `omega-google --version` is run, then version info is printed
- [ ] Given the project structure, when inspected, then `Cargo.toml` uses workspace layout

### REQ-AUTH-003: OAuth2 Flow
- [ ] Given valid credentials, when `omega-google auth add user@gmail.com` is run, then a browser opens with OAuth consent
- [ ] Given user approves, when the OAuth redirect is received, then a refresh token is stored in the keyring
- [ ] Given `--services gmail,calendar`, when the flow completes, then only Gmail and Calendar scopes are requested
- [ ] Given `--readonly`, when the flow completes, then readonly scope variants are used

### REQ-HTTP-002: Retry on 429
- [ ] Given a 429 response with `Retry-After: 5`, when retried, then the client waits 5 seconds
- [ ] Given a 429 without `Retry-After`, when retried, then exponential backoff with jitter is used
- [ ] Given 3 consecutive 429s, when max retries exceeded, then the 429 response is returned to the caller
- [ ] Given a context cancellation during sleep, when cancelled, then an error is returned immediately

### REQ-CLI-007: Stable Exit Codes
- [ ] Given a 404 API error, when the command exits, then exit code is 5
- [ ] Given a 401 API error, when the command exits, then exit code is 4
- [ ] Given a circuit breaker open, when the command exits, then exit code is 8
- [ ] Given SIGINT, when the process is interrupted, then exit code is 130

### REQ-GMAIL-010: Send Email
- [ ] Given `--to a@b.com --subject Test --body Hello`, when send executes, then email is sent successfully
- [ ] Given `--attach file.pdf`, when send executes, then file is attached as MIME attachment
- [ ] Given `--reply-to-message-id <id>`, when send executes, then email is threaded correctly
- [ ] Given `--track`, when tracking is configured, then tracking pixel is injected

---

## Impact Analysis

### Existing Code Affected
This is a greenfield project. No existing code is affected.

### What Breaks If This Changes
N/A -- new project.

### Regression Risk Areas
- **OAuth scope mapping**: Must match gogcli exactly or APIs will reject requests
- **Keyring storage format**: Must be consistent across platforms for credential portability
- **Output format**: JSON schema must match gogcli for downstream script compatibility
- **Exit codes**: Must match gogcli for CI/automation pipeline compatibility
- **Date/time parsing**: Must accept all formats gogcli accepts

---

## Traceability Matrix

> Architecture Section references point to `omega-google-architecture.md`. Full per-requirement traceability is in the architecture spec's "Requirement Traceability" section.

| Requirement ID | Priority | Test IDs | Architecture Section | Implementation Module |
|---------------|----------|----------|---------------------|---------------------|
| REQ-SCAFFOLD-001 | Must | (filled by test-writer) | Module 1: main.rs; Project Structure | `Cargo.toml`, `src/main.rs` |
| REQ-SCAFFOLD-002 | Must | (filled by test-writer) | Nix Flake Structure (devShells) | `flake.nix` |
| REQ-SCAFFOLD-003 | Must | (filled by test-writer) | Nix Flake Structure (packages) | `flake.nix` |
| REQ-SCAFFOLD-004 | Must | (filled by test-writer) | External Dependencies | `Cargo.toml` |
| REQ-SCAFFOLD-005 | Must | (filled by test-writer) | Project Structure; Modules 2-11 | `src/cli/`, `src/config/`, `src/auth/`, `src/http/`, `src/output/`, `src/ui/`, `src/error/` |
| REQ-CLI-001 | Must | (filled by test-writer) | Module 2: cli/ (RootFlags) | `src/cli/root.rs` |
| REQ-CLI-002 | Must | (filled by test-writer) | Module 2: cli/ (env attrs) | `src/cli/root.rs` |
| REQ-CLI-003 | Must | (filled by test-writer) | Module 2: cli/ | `src/cli/version.rs` |
| REQ-CLI-004 | Must | (filled by test-writer) | Module 2: cli/ | `src/cli/version.rs` |
| REQ-CLI-005 | Must | (filled by test-writer) | Module 9: time/ | `src/cli/time_cmd.rs`, `src/time/mod.rs` |
| REQ-CLI-006 | Must | (filled by test-writer) | Module 7: ui/ | `src/ui/mod.rs` |
| REQ-CLI-007 | Must | (filled by test-writer) | Module 8: error/ (exit codes) | `src/error/exit.rs` |
| REQ-CLI-008 | Must | (filled by test-writer) | Module 8: error/ (formatting) | `src/error/mod.rs` |
| REQ-CLI-009 | Should | (filled by test-writer) | Module 2: cli/ (desire paths) | `src/cli/desire_paths.rs` |
| REQ-CONFIG-001 | Must | (filled by test-writer) | Module 3: config/ | `src/config/file.rs`, `src/config/paths.rs` |
| REQ-CONFIG-002 | Must | (filled by test-writer) | Module 3: config/ (ConfigFile) | `src/config/file.rs` |
| REQ-CONFIG-003 | Must | (filled by test-writer) | Module 3: config/ | `src/cli/config_cmd.rs`, `src/config/mod.rs` |
| REQ-CONFIG-004 | Must | (filled by test-writer) | Module 3: config/ | `src/cli/config_cmd.rs`, `src/config/mod.rs` |
| REQ-CONFIG-005 | Must | (filled by test-writer) | Module 3: config/ | `src/cli/config_cmd.rs`, `src/config/mod.rs` |
| REQ-CONFIG-006 | Must | (filled by test-writer) | Module 3: config/ | `src/cli/config_cmd.rs`, `src/config/mod.rs` |
| REQ-CONFIG-007 | Must | (filled by test-writer) | Module 3: config/ | `src/cli/config_cmd.rs`, `src/config/mod.rs` |
| REQ-CONFIG-008 | Must | (filled by test-writer) | Module 3: config/ (paths) | `src/cli/config_cmd.rs`, `src/config/paths.rs` |
| REQ-CONFIG-009 | Must | (filled by test-writer) | Module 3: config/ (credentials) | `src/config/credentials.rs` |
| REQ-AUTH-001 | Must | (filled by test-writer) | Module 4: auth/ | `src/cli/auth.rs`, `src/config/credentials.rs` |
| REQ-AUTH-002 | Must | (filled by test-writer) | Module 4: auth/ | `src/cli/auth.rs`, `src/config/credentials.rs` |
| REQ-AUTH-003 | Must | (filled by test-writer) | Module 4: auth/ (OAuth flow) | `src/auth/oauth.rs` |
| REQ-AUTH-004 | Must | (filled by test-writer) | Module 4: auth/ (manual flow) | `src/auth/oauth.rs` |
| REQ-AUTH-005 | Must | (filled by test-writer) | Module 4: auth/ (remote flow) | `src/auth/oauth.rs` |
| REQ-AUTH-006 | Must | (filled by test-writer) | Module 4: auth/ (force consent) | `src/auth/oauth.rs` |
| REQ-AUTH-007 | Must | (filled by test-writer) | Module 4: auth/ | `src/cli/auth.rs`, `src/auth/token.rs` |
| REQ-AUTH-008 | Must | (filled by test-writer) | Module 4: auth/ | `src/cli/auth.rs`, `src/auth/keyring.rs` |
| REQ-AUTH-009 | Must | (filled by test-writer) | Module 4: auth/ | `src/cli/auth.rs` |
| REQ-AUTH-010 | Must | (filled by test-writer) | Module 4: auth/ (scopes) | `src/cli/auth.rs`, `src/auth/scopes.rs` |
| REQ-AUTH-011 | Must | (filled by test-writer) | Module 4: auth/ (keyring) | `src/cli/auth.rs`, `src/auth/keyring.rs` |
| REQ-AUTH-012 | Must | (filled by test-writer) | Module 4: auth/ + Module 3: config/ | `src/cli/auth.rs`, `src/config/file.rs` |
| REQ-AUTH-013 | Must | (filled by test-writer) | Module 4: auth/ (KeyringStore) | `src/auth/keyring.rs` |
| REQ-AUTH-014 | Must | (filled by test-writer) | Module 4: auth/ (file fallback) | `src/auth/keyring.rs` |
| REQ-AUTH-015 | Must | (filled by test-writer) | Module 4: auth/ + Module 3: config/ | `src/auth/keyring.rs`, `src/config/file.rs` |
| REQ-AUTH-016 | Must | (filled by test-writer) | Module 4: auth/ (scope mapping) | `src/auth/scopes.rs` |
| REQ-AUTH-017 | Must | (filled by test-writer) | Module 4: auth/ (service account) | `src/auth/service_account.rs` |
| REQ-AUTH-018 | Should | (filled by test-writer) | Module 4: auth/ (keyring cmd) | `src/cli/auth.rs`, `src/auth/keyring.rs` |
| REQ-AUTH-019 | Must | (filled by test-writer) | Module 4: auth/ (account resolution) | `src/auth/account.rs` |
| REQ-AUTH-020 | Should | (filled by test-writer) | Module 4: auth/ (timeout) | `src/auth/keyring.rs` |
| REQ-HTTP-001 | Must | (filled by test-writer) | Module 5: http/ (ApiClient) | `src/http/client.rs` |
| REQ-HTTP-002 | Must | (filled by test-writer) | Module 5: http/ (retry 429) | `src/http/retry.rs` |
| REQ-HTTP-003 | Must | (filled by test-writer) | Module 5: http/ (retry 5xx) | `src/http/retry.rs` |
| REQ-HTTP-004 | Must | (filled by test-writer) | Module 5: http/ (circuit breaker) | `src/http/circuit_breaker.rs` |
| REQ-HTTP-005 | Must | (filled by test-writer) | Module 5: http/ (body replay) | `src/http/retry.rs` |
| REQ-HTTP-006 | Must | (filled by test-writer) | Module 5: http/ (cancellation) | `src/http/retry.rs` |
| REQ-OUTPUT-001 | Must | (filled by test-writer) | Module 6: output/ | `src/output/mode.rs`, `src/output/json.rs`, `src/output/plain.rs`, `src/output/text.rs` |
| REQ-OUTPUT-002 | Must | (filled by test-writer) | Module 6: output/ (results-only) | `src/output/transform.rs` |
| REQ-OUTPUT-003 | Must | (filled by test-writer) | Module 6: output/ (field select) | `src/output/transform.rs` |
| REQ-OUTPUT-004 | Must | (filled by test-writer) | Module 6: output/ + Module 2: cli/ | `src/output/mode.rs`, `src/cli/root.rs` |
| REQ-OUTPUT-005 | Must | (filled by test-writer) | Module 7: ui/ (color disable) | `src/ui/color.rs` |
| REQ-UI-001 | Must | (filled by test-writer) | Module 7: ui/ (color) | `src/ui/color.rs` |
| REQ-UI-002 | Must | (filled by test-writer) | Module 7: ui/ (stderr) | `src/ui/progress.rs` |
| REQ-UI-003 | Must | (filled by test-writer) | Module 8: error/ (API errors) | `src/error/api_error.rs` |
| REQ-GMAIL-001 through REQ-GMAIL-020 | Must | (filled by test-writer) | Module 10: services/gmail | `src/services/gmail/` (search, thread, message, labels, send, drafts, watch, history, batch, settings, mime) |
| REQ-CAL-001 through REQ-CAL-022 | Must/Should | (filled by test-writer) | Module 10: services/calendar; Module 9: time/ | `src/services/calendar/` (events, calendars, freebusy, respond, search, special, colors), `src/time/parse.rs` |
| REQ-DRIVE-001 through REQ-DRIVE-017 | Must/Should | (filled by test-writer) | Module 10: services/drive | `src/services/drive/` (list, files, folders, permissions, comments, drives) |
| REQ-CLI-010 through REQ-CLI-019 | Must | (filled by test-writer) | Module 2: cli/ (Command enum) | `src/cli/mod.rs` (desire path variants), `src/cli/open.rs` |
| REQ-DOCS-001 through REQ-DOCS-016 | Must/Should | (filled by test-writer) | Module 10: services/docs | `src/services/docs/` (export, content, edit, sedmat, markdown, comments), `src/services/export.rs` |
| REQ-SHEETS-001 through REQ-SHEETS-012 | Must/Should | (filled by test-writer) | Module 10: services/sheets | `src/services/sheets/` (read, write, format, structure, a1), `src/services/export.rs` |
| REQ-SLIDES-001 through REQ-SLIDES-011 | Must/Should | (filled by test-writer) | Module 10: services/slides | `src/services/slides/` (export, presentations, slides_ops, notes, markdown), `src/services/export.rs` |
| REQ-FORMS-001 through REQ-FORMS-004 | Must | (filled by test-writer) | Module 10: services/forms | `src/services/forms/` (forms, responses) |
| REQ-CHAT-001 through REQ-CHAT-008 | Must | (filled by test-writer) | Module 10: services/chat | `src/services/chat/` (spaces, messages, dm) |
| REQ-CLASS-001 through REQ-CLASS-013 | Must | (filled by test-writer) | Module 10: services/classroom | `src/services/classroom/` (courses, roster, coursework, materials, submissions, announcements, topics, invitations, guardians) |
| REQ-TASKS-001 through REQ-TASKS-010 | Must | (filled by test-writer) | Module 10: services/tasks | `src/services/tasks/` (tasklists, tasks_ops) |
| REQ-CONTACTS-001 through REQ-CONTACTS-008 | Must | (filled by test-writer) | Module 10: services/contacts | `src/services/contacts/` (contacts, directory, other) |
| REQ-PEOPLE-001 through REQ-PEOPLE-004 | Must | (filled by test-writer) | Module 10: services/people | `src/services/people/` (profile) |
| REQ-GROUPS-001 through REQ-GROUPS-003 | Must/Should | (filled by test-writer) | Module 10: services/groups; Module 8: error/ | `src/services/groups/` (groups), `src/error/api_error.rs` |
| REQ-KEEP-001 through REQ-KEEP-005 | Must | (filled by test-writer) | Module 10: services/keep; Module 4: auth/ | `src/services/keep/` (notes, attachments), `src/auth/service_account.rs` |
| REQ-SCRIPT-001 through REQ-SCRIPT-004 | Must | (filled by test-writer) | Module 10: services/appscript | `src/services/appscript/` (projects) |
| REQ-TRACK-001 through REQ-TRACK-004 | Could | (filled by test-writer) | Module 11: tracking/ | `src/tracking/` (pixel, config), `src/services/gmail/send.rs` |
| REQ-AGENT-001 through REQ-AGENT-003 | Must | (filled by test-writer) | Module 2: cli/ (agent) | `src/cli/agent.rs`, `src/cli/root.rs` |
| REQ-CLI-020 through REQ-CLI-024 | Must | (filled by test-writer) | Module 2: cli/; Module 7: ui/; Module 10: services/ | `src/cli/completion.rs`, `src/ui/prompt.rs`, `src/services/common.rs` |
| REQ-OUTPUT-006 | Could | (filled by test-writer) | Module 6: output/ | `src/output/csv.rs` |

---

## Specs Drift Detected

N/A -- this is a new project with no existing specs.

---

## Assumptions

| # | Assumption (technical) | Explanation (plain language) | Confirmed |
|---|----------------------|---------------------------|-----------|
| 1 | Binary name is `omega-google` (not `og`) | The CLI binary will be called `omega-google` as stated in the idea brief. The shorter alias `og` is listed as an open question. | No |
| 2 | Config directory uses `omega-google/` namespace (not `gogcli/`) | Fresh config directory, no migration from gogcli. | No |
| 3 | JSON5 reading is supported for config (using a Rust JSON5 crate) | The Go version uses JSON5 for config reading but writes standard JSON. The Rust version should do the same. | No |
| 4 | Environment variable prefix remains `GOG_` for compatibility | Even though the binary is renamed, env vars keep `GOG_` prefix so scripts work if users switch between tools. | No |
| 5 | All command handlers are async (tokio runtime) | Since reqwest is async and the tool is IO-bound, all command handlers should be async. | No |
| 6 | Google REST APIs are called directly via reqwest+serde (no generated clients) | As stated in the idea brief, raw REST API calls rather than generated client libraries. | Yes |
| 7 | The `keyring` Rust crate supports macOS Keychain and Linux Secret Service | Need to verify the Rust `keyring` crate has equivalent backend support to Go's `github.com/99designs/keyring`. | No |
| 8 | OAuth2 desktop redirect flow works on ephemeral ports in Rust | The `oauth2` Rust crate or custom implementation can listen on ephemeral ports for the redirect. | No |
| 9 | `sedmat` complexity is deferred to M3 but fully implemented | The sed-like document editing engine is complex (~15 Go files) but required for feature parity. | Yes |
| 10 | Service account key format is standard Google service account JSON | JWT signing uses the `jsonwebtoken` crate with RS256. | Yes |
| 11 | `clap` derive macros are used for CLI definition (not builder pattern) | Derive macros provide the most ergonomic and maintainable CLI definition. | No |
| 12 | Command aliases match gogcli exactly for backward script compatibility | e.g., `drv` for `drive`, `cal` for `calendar`, `mail`/`email` for `gmail` | No |
| 13 | The `GOG_AUTO_JSON` behavior applies AFTER argument parsing | JSON mode is enabled post-parse so `--plain` can still override | Yes |
| 14 | Keyring service name is `omega-google` (not `gogcli`) | Fresh keyring namespace, tokens are not shared with gogcli | No |

---

## Identified Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Rust `keyring` crate may not have file-based fallback with password encryption | High | Medium | Evaluate crate capabilities early in M1; may need custom file backend |
| Raw REST API surface area for 15 Google APIs is enormous | High | High | Define only used fields with `serde(flatten)` for pass-through; implement incrementally per milestone |
| Google API behavioral differences between Go client libraries and raw REST | Medium | Medium | Integration tests against real APIs; reference Google API Discovery Documents |
| `sedmat` regex engine in Rust may behave differently than Go's `regexp` | Medium | Medium | Use `regex` crate with careful testing; document any behavioral differences |
| OAuth2 ephemeral port listener may conflict on some systems | Low | Low | Allow port configuration; fall back to manual flow |
| JSON5 parsing crate quality in Rust | Low | Medium | Evaluate `json5` crate; fall back to standard JSON if unreliable |
| Keep API and Chat API have Workspace-only restrictions that are hard to test | Medium | High | Document testing prerequisites; mock-based tests for CI |
| Circuit breaker state is in-process only (no persistence) | Low | Low | Acceptable for CLI tool; process exits on error |
| Date/time parsing must match gogcli's flexible parser exactly | Medium | Medium | Build comprehensive test suite covering all accepted formats |
| Nix flake may need specific overrides for OpenSSL/ring on different platforms | Medium | Medium | Test on macOS and Linux; document known issues |

---

## Out of Scope (Won't -- this iteration)

| Item | Reason |
|------|--------|
| Backwards compatibility with gogcli config/tokens | Explicitly excluded -- fresh setup required |
| MCP server mode | This is a CLI, not a server |
| GUI or TUI dashboards | Terminal output only |
| Google Admin SDK beyond Groups/Keep | Out of scope for this tool |
| Google Cloud Platform APIs | Only Workspace/consumer APIs |
| Homebrew/AUR/apt packaging | Nix only for now |
| Windows CI testing | macOS and Linux first; Windows best-effort |
| Migration tooling from gogcli | No migration path; users re-authenticate |
| Config directory sharing with gogcli | Fresh namespace prevents conflicts |

---

## Requirement Statistics

| Category | Must | Should | Could | Won't | Total |
|----------|------|--------|-------|-------|-------|
| SCAFFOLD | 5 | 0 | 0 | 0 | 5 |
| CLI | 19 | 1 | 0 | 0 | 20 |
| CONFIG | 9 | 0 | 0 | 0 | 9 |
| AUTH | 18 | 2 | 0 | 0 | 20 |
| HTTP | 6 | 0 | 0 | 0 | 6 |
| OUTPUT | 5 | 0 | 1 | 0 | 6 |
| UI | 3 | 0 | 0 | 0 | 3 |
| GMAIL | 20 | 0 | 0 | 0 | 20 |
| CAL | 15 | 5 | 0 | 0 | 20 |
| DRIVE | 16 | 1 | 0 | 0 | 17 |
| DOCS | 14 | 2 | 0 | 0 | 16 |
| SHEETS | 10 | 1 | 0 | 0 | 11 |
| SLIDES | 8 | 2 | 0 | 0 | 10 |
| FORMS | 4 | 0 | 0 | 0 | 4 |
| CHAT | 8 | 0 | 0 | 0 | 8 |
| CLASS | 13 | 0 | 0 | 0 | 13 |
| TASKS | 10 | 0 | 0 | 0 | 10 |
| CONTACTS | 8 | 0 | 0 | 0 | 8 |
| PEOPLE | 4 | 0 | 0 | 0 | 4 |
| GROUPS | 2 | 1 | 0 | 0 | 3 |
| KEEP | 5 | 0 | 0 | 0 | 5 |
| SCRIPT | 4 | 0 | 0 | 0 | 4 |
| TRACK | 0 | 0 | 4 | 0 | 4 |
| AGENT | 3 | 0 | 0 | 0 | 3 |
| **Total** | **209** | **15** | **5** | **0** | **229** |
