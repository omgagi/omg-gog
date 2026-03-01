# Idea Brief: omega-google — Google Workspace CLI in Rust

## One-Line Summary

A clean, idiomatic Rust reimplementation of gogcli: a fast, script-friendly CLI for 15 Google Workspace services with JSON-first output, multiple accounts, least-privilege OAuth, and OS keyring credential storage.

## Problem Statement

The existing gogcli project (github.com/steipete/gogcli) provides comprehensive Google Workspace CLI access across 15 services in Go. The user wants this same capability reimplemented in Rust for the omega-tools project, gaining Rust's safety guarantees, single-binary distribution, and integration with the broader omega-tools ecosystem. This is a full-scope reimplementation, not a wrapper or partial port.

## Current State

The Go implementation at `/tmp/gogcli-source` is mature and feature-complete:
- **546 Go files**, approximately 117K lines of Go code
- **272 command handler functions** across 87 source files
- **311 test files** with comprehensive coverage
- **12 internal packages**: `cmd`, `config`, `googleauth`, `googleapi`, `secrets`, `outfmt`, `ui`, `errfmt`, `input`, `timeparse`, `tracking`, `authclient`
- **15 Google service integrations** with per-service OAuth scope mappings (including readonly variants and drive-scope modes)
- Detailed spec at `docs/spec.md` documenting every command, flag, scope, and design decision

The Go version uses Google's official typed Go client libraries (`google.golang.org/api`). The Rust version will use raw REST API calls instead.

## Proposed Solution

Build `omega-google`, a Rust binary that provides feature-equivalent CLI access to all 15 Google Workspace services. The implementation will be idiomatic Rust (not a line-by-line transpile), using established Rust crates and hitting Google's REST APIs directly via reqwest + serde.

### Binary Name

`omega-google` (or `og` as a short alias, to be decided during implementation)

### Google Services (all 15)

| Service | Google API | Key Operations |
|---------|-----------|----------------|
| Gmail | Gmail API v1 | Search, send, threads, labels, drafts, filters, delegation, vacation, watch (Pub/Sub), history, attachments, batch, email tracking |
| Calendar | Calendar API v3 | Events CRUD, free/busy, conflicts, search, respond/RSVP, team calendars, focus time, OOO, working location, recurrence, colors, propose time |
| Drive | Drive API v3 | List, search, upload, download, mkdir, delete, move, rename, share, permissions, comments, shared drives, URL generation |
| Docs | Docs API v1 + Drive API v3 | Export (PDF/DOCX), create/copy, text extraction, **sedmat** (sed-like document editing with Markdown formatting, images, tables, brace patterns) |
| Slides | Slides API v1 + Drive API v3 | Export (PDF/PPTX), create/copy, list/add/replace/delete slides, update notes, markdown support |
| Sheets | Sheets API v4 + Drive API v3 | Read/write/update cells, insert rows/cols, format cells, notes, create sheets, export, A1 notation parsing, append validation |
| Forms | Forms API v1 | Create/get forms, inspect responses |
| Apps Script | Apps Script API v1 | Create/get projects, inspect content, run functions |
| Chat | Chat API v1 | List/find/create spaces, messages (filter by thread/unread), send DMs (Workspace-only) |
| Classroom | Classroom API v1 | Courses CRUD, roster management, coursework/materials, submissions, announcements, topics, invitations, guardians, profiles |
| Contacts | People API v1 | Search/create/update/delete contacts, Workspace directory, other contacts, JSON update |
| Tasks | Tasks API v1 | Tasklists, tasks CRUD, done/undo, clear, repeat schedules, due dates |
| People | People API v1 | Profile info (me/get/search/relations) |
| Groups | Cloud Identity API v1 | List groups, view members (Workspace-only) |
| Keep | Keep API v1 | List/get/search notes, download attachments (Workspace-only, service account + domain-wide delegation) |

### Cross-Cutting Features

- **Multiple accounts** with aliases and per-account OAuth client selection
- **Output modes**: JSON (`--json`), plain/TSV (`--plain`), human-friendly colored text (default)
- **Auto-JSON mode**: defaults to JSON when stdout is piped/non-TTY (`GOG_AUTO_JSON`)
- **Field selection**: `--select` / `--results-only` for JSON filtering
- **Least-privilege auth**: `--readonly`, `--drive-scope full|readonly|file`
- **Service accounts**: domain-wide delegation for Workspace APIs
- **Command allowlisting**: `--enable-commands` for sandboxed/agent runs
- **Dry-run mode**: `--dry-run` for previewing destructive operations
- **Shell completion**: bash, zsh, fish, powershell
- **Desire paths**: action-first aliases (`send`, `ls`, `search`, `download`, `upload`, `login`, `logout`, `status`, `me`, `whoami`)
- **Email tracking**: tracking pixel insertion via Cloudflare Worker backend
- **Retry/backoff**: exponential backoff with jitter for 429s, retry on 5xx, circuit breaker
- **Date/time parsing**: flexible input (RFC3339, relative dates, weekday names, durations)

## Target Users

- **Primary**: Developers and power users who interact with Google Workspace programmatically — scripting, automation, CI/CD pipelines, and LLM/agent tool use
- **Secondary**: System administrators managing Google Workspace (service accounts, domain-wide delegation, groups)

## Success Criteria

1. Feature parity with gogcli: every command and flag documented in the source `docs/spec.md` has an equivalent in omega-google
2. All 15 Google services are functional with correct OAuth scope handling
3. JSON, plain, and human-friendly output modes work identically
4. Multi-account management and credential storage work on macOS, Linux, and (ideally) Windows
5. The binary is distributable as a single static executable via Nix
6. Comprehensive Rust test suite covering the same intent as the Go tests
7. All commands produce output parseable by the same scripts that consume gogcli output

## MVP Scope (Milestone 1 — Minimum Viable Product)

The smallest useful version is M1 + M2: project scaffolding, authentication, config management, and the three core Google services (Gmail, Calendar, Drive). This alone provides the most-used functionality.

## Milestones

### M1: Project Scaffolding + Nix + Auth Infrastructure + Config

**Goal**: A buildable Rust project with working authentication, config management, and the foundation all services will build on.

**Deliverables**:
- Cargo workspace structure under `omega-google/`
- `flake.nix` with dev shell (Rust toolchain, openssl, pkg-config) AND `nix build` support
- CLI skeleton with clap: root flags (`--json`, `--plain`, `--color`, `--account`, `--client`, `--verbose`, `--dry-run`, `--force`, `--no-input`, `--select`, `--results-only`, `--enable-commands`)
- Config module: read/write `config.json` (JSON5 support), paths, account aliases, client mapping, domain mapping
- Credential storage: OS keyring via `keyring` crate (macOS Keychain, Linux Secret Service, file fallback with password)
- OAuth2 flow: desktop redirect (ephemeral port), manual/browserless flow, remote 2-step flow, service account JWT
- Token management: store/retrieve/delete refresh tokens, per-client key namespacing (`token:<client>:<email>`)
- HTTP client module: reqwest-based client with OAuth2 token injection, retry transport (429 exponential backoff with jitter, 5xx retry), circuit breaker, TLS 1.2+ enforcement
- Output formatting module: JSON/plain/text modes, field selection, results-only filtering
- UI module: colored terminal output (crossterm), stderr for progress/errors, `NO_COLOR` support
- Error formatting module
- `auth` command group: `credentials`, `add`, `remove`, `list`, `status`, `services`, `tokens list/delete`, `alias set/unset/list`, `keep` (service account), `keyring`
- `config` command group: `get`, `set`, `unset`, `list`, `keys`, `path`
- `version` command
- `time now` command
- Unit tests for config, auth, output formatting, retry logic

### M2: Core Services (Gmail, Calendar, Drive)

**Goal**: The three most-used Google services, fully functional.

**Deliverables**:
- **Gmail**: search, messages, thread get/modify, get message, attachment download, URL, labels (list/get/create/modify/delete), send (with CC/BCC/reply/attachments), drafts (list/get/create/update/send/delete), watch (start/status/renew/stop/serve), history, batch modify, filters (list/get/create/delete), forwarding, send-as, delegates, vacation, auto-forward
- **Calendar**: calendars list, ACL, events list/get/create/update/delete, free/busy, respond/RSVP, search, time, users, team, colors, conflicts, propose-time, focus-time, OOO, working-location, recurrence handling, attendee management, event-type support, day-of-week enrichment, flexible date/time parsing
- **Drive**: ls, search, get, download (with format conversion for Google Workspace files), upload (with conversion), mkdir, delete (with permanent option), move, rename, share, permissions list/remove, URL, drives (shared drives), comments (list/get/create/resolve/delete), copy
- **Desire path aliases**: `send`, `ls`, `search`, `download`, `upload`, `login`, `logout`, `status`, `me`, `whoami`
- REST API type definitions (request/response structs) for Gmail v1, Calendar v3, Drive v3
- Integration test scaffolding (opt-in, requires real credentials)

### M3: Productivity Services (Docs, Sheets, Slides, Forms)

**Goal**: Document manipulation and data services.

**Deliverables**:
- **Docs**: export (PDF/DOCX/text), create, copy, info, text extraction, comments (list/get/create/resolve/delete), edit (append/prepend/insert/replace/delete), **sedmat** (full sed-like editing engine: s/pattern/replacement/flags, delete, append, insert, image insertion, table create/operations, brace pattern matching, Markdown formatting, dry-run mode, retry on quota)
- **Sheets**: read, write, update, append, insert rows/cols, format cells (bold/italic/color/borders/alignment/number-format), notes (read/write), create, A1 notation parsing, validation
- **Slides**: export (PDF/PPTX), create, copy, list slides, add slide, replace slide, delete slide, read slide, update notes, markdown support, formatter
- **Forms**: create, get, responses list
- REST API type definitions for Docs v1, Sheets v4, Slides v1, Forms v1
- Export-via-Drive shared module (used by Docs, Sheets, Slides)

### M4: Collaboration Services (Chat, Classroom, Tasks, Contacts, People)

**Goal**: Communication and people-oriented services.

**Deliverables**:
- **Chat**: spaces (list/find/create), messages (list/send, thread filter, unread filter), threads list, DM (space/send)
- **Classroom**: courses (list/get/create/update/delete/archive/unarchive/join/leave/url), students/teachers (list/get/add/remove), roster, coursework (list/get/create/update/delete/assignees), materials (list/get/create/update/delete), submissions (list/get/turn-in/reclaim/return/grade), announcements (list/get/create/update/delete/assignees), topics (list/get/create/update/delete), invitations (list/get/create/accept/delete), guardians (list/get/delete), guardian-invitations (list/get/create), profile
- **Tasks**: tasklists (list/create), tasks (list/get/add/update/done/undo/delete/clear), repeat schedules, due date handling
- **Contacts**: search, list, get, create, update (including JSON update from file), delete, directory (list/search), other contacts (list/search)
- **People**: me, get, search, relations
- REST API type definitions for Chat v1, Classroom v1, Tasks v1, People v1, Cloud Identity v1

### M5: Admin/Workspace Services (Groups, Keep, Apps Script)

**Goal**: Workspace-specific and admin services.

**Deliverables**:
- **Groups**: list, members (Workspace-only, Cloud Identity API)
- **Keep**: list, get, search notes, download attachments (Workspace-only, requires service account + domain-wide delegation)
- **Apps Script**: create, get, content, run (Apps Script API)
- REST API type definitions for Keep v1, Apps Script v1, Cloud Identity v1

### M6: Polish (Tracking, Completion, Allowlisting, Full Test Coverage)

**Goal**: Feature completeness and production readiness.

**Deliverables**:
- **Email tracking**: pixel generation (AES-GCM encryption), tracking setup, status checking, Cloudflare Worker integration
- **Shell completion**: bash, zsh, fish, powershell generation
- **Command allowlisting**: `--enable-commands` enforcement
- **Agent mode**: `agent exit-codes`, schema/help-json command, desire path argument rewriting
- **Open command**: offline URL generation for Google resource IDs
- **Google ID resolution**: name-to-ID resolution for calendars, labels, etc.
- **CSV output mode** (where applicable)
- **Comprehensive test suite**: unit tests for all modules, command-level tests with mock HTTP, edge case coverage matching Go test intent
- **CI setup**: GitHub Actions workflow (cargo fmt, clippy, test)
- **Documentation**: README, usage examples

## Explicitly Out of Scope

- **Backwards compatibility with gogcli config/tokens** — no migration tooling; fresh setup required
- **MCP server mode** — this is a CLI, not a server (matches gogcli's explicit non-goal)
- **GUI or TUI** — terminal output only, no interactive dashboards
- **Google Admin SDK** — admin-level Workspace management beyond Groups/Keep
- **Google Cloud Platform APIs** — only Workspace/consumer APIs
- **Homebrew/AUR packaging** — Nix only for now; other packaging later
- **Windows CI testing** — target macOS and Linux first; Windows is best-effort

## Key Decisions Made

- **Idiomatic Rust, not line-by-line transpile**: The Rust version will follow Rust conventions (traits, enums, error handling with `thiserror`/`anyhow`, builder patterns) rather than mirroring Go's struct-embedding and error-return patterns
- **Raw REST API calls**: Using reqwest + serde directly against Google's REST APIs rather than generated client libraries. Full control, avoids dependency on generated code quality, consistent handling across all 15 APIs
- **Popular crates, not stdlib-only**: clap (CLI), tokio (async), reqwest (HTTP), serde/serde_json (serialization), keyring (credentials), oauth2 (auth), crossterm (terminal)
- **Nix for dev + build**: `flake.nix` provides both a dev shell and reproducible `nix build` output
- **Fresh tests**: Idiomatic Rust tests covering the same behavioral intent as the Go tests, not transpiled test code
- **Full port**: All 15 services, all features, no service cuts — but structured in 6 milestones so each milestone produces a usable tool
- **Same command structure**: Command tree mirrors gogcli (`auth`, `gmail`, `calendar`, `drive`, etc.) for user familiarity

## Open Questions

- **Binary name**: `omega-google` as the binary name, or something shorter like `og`?
- **Config directory compatibility**: Should `omega-google` use the same config directory as gogcli or a separate directory?
- **Async everywhere**: Should command handlers be async (natural fit with tokio + reqwest) or use blocking wrappers where simple?
- **JSON5 support in Rust**: Keep JSON5 for config or switch to standard JSON or TOML?
- **sedmat complexity**: Deprioritize or simplify the sed-like document editing engine?
- **Cloudflare Worker for email tracking**: Include or treat as optional/separate?

## Constraints

- **Technology**: Rust (latest stable), Nix (flakes), popular crates (clap, tokio, reqwest, serde, keyring, oauth2, crossterm)
- **Scale**: Single-user CLI tool; no server-side scaling concerns
- **Integration**: Google Workspace REST APIs (15 services), OS keyring, optional Cloudflare Worker
- **Source reference**: Complete Go source at `/tmp/gogcli-source` with spec at `/tmp/gogcli-source/docs/spec.md`

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Raw REST API surface area (15 APIs) | Start with fields actually used, expand as needed. Use `serde(flatten)` for pass-through |
| Google API behavior parity | Integration tests against real APIs; reference Discovery Documents |
| Keyring cross-platform reliability | File-based fallback with password encryption |
| sedmat complexity (~15 Go files) | Defer to M3, write extensive tests |
| Keep/Chat API restrictions | Defer to M5, document testing prerequisites |
| Service account JWT signing | Use mature `jsonwebtoken` crate |

## Key Source Files Reference

- `/tmp/gogcli-source/docs/spec.md` — Authoritative spec (458 lines)
- `/tmp/gogcli-source/internal/cmd/root.go` — CLI structure, root flags
- `/tmp/gogcli-source/internal/googleauth/service.go` — All 15 service definitions with OAuth scopes
- `/tmp/gogcli-source/internal/googleapi/client.go` — OAuth client setup
- `/tmp/gogcli-source/internal/googleapi/transport.go` — Retry logic
- `/tmp/gogcli-source/internal/googleapi/service_account.go` — Service account JWT auth
- `/tmp/gogcli-source/internal/secrets/store.go` — Keyring interface
- `/tmp/gogcli-source/internal/config/config.go` — Config file structure
