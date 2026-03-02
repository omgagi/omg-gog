# Developer Guide: omega-google

## Prerequisites

- Rust (latest stable, installed via rustup or Nix)
- Nix with flakes enabled (recommended for reproducible builds)
- OpenSSL + pkg-config (provided by Nix dev shell)

## Getting Started

### With Nix (recommended)

```bash
cd omega-google
nix develop          # Enter dev shell with all dependencies
cargo build          # Build debug binary
cargo test           # Run tests
cargo clippy         # Lint
cargo fmt --check    # Check formatting
```

### Without Nix

```bash
cd omega-google
# Ensure OpenSSL and pkg-config are installed:
# macOS: brew install openssl pkg-config
# Linux: apt install libssl-dev pkg-config (or equivalent)
cargo build
cargo test
```

### Nix Build (release binary)

```bash
nix build            # Produces result/bin/omega-google
./result/bin/omega-google --version
```

## Project Structure

```
omega-google/
  Cargo.toml          # Dependencies and build config
  flake.nix           # Nix flake (dev shell + package)
  src/
    main.rs            # Entry point (tokio runtime)
    lib.rs             # Library root (re-exports for testing)
    cli/               # clap command definitions and dispatch
    config/            # Config file management, paths
    auth/              # OAuth2, keyring, service accounts, scopes
    http/              # Authenticated HTTP client, retry, circuit breaker
    output/            # JSON/plain/text formatting, transforms
    ui/                # Terminal colors, progress, prompts
    error/             # Error types, exit codes, API error parsing
    time/              # Flexible date/time parsing
    services/          # 15 Google API implementations
      gmail/
      calendar/
      drive/
      docs/
      sheets/
      slides/
      forms/
      chat/
      classroom/
      tasks/
      contacts/
      people/
      groups/
      keep/
      appscript/
    tracking/          # Email tracking (M6)
  tests/
    integration/       # Integration tests (opt-in, requires credentials)
    cli_smoke.rs       # CLI binary invocation tests
```

## Module Overview

### cli/ -- Command Line Interface

All clap command definitions live here. Each Google service has its own file (e.g., `cli/gmail.rs`). The root module defines `RootFlags` and the top-level `Command` enum.

**Pattern**: Service command files define clap structs only. Business logic lives in `services/`. CLI files call service functions.

### config/ -- Configuration

Handles `$CONFIG_DIR/omega-google/config.json` (read with JSON5, write as JSON). Also manages credential files and path resolution.

### auth/ -- Authentication

OAuth2 flows (desktop, manual, remote two-step), keyring abstraction, service account JWT, per-service scope mapping, and account resolution.

### http/ -- HTTP Client

Builds an authenticated reqwest client with automatic token injection, retry on 429 (exponential backoff with jitter), retry on 5xx, and circuit breaker.

### output/ -- Output Formatting

Three modes: JSON, plain/TSV, and human-friendly text. Supports `--results-only` (strip envelope) and `--select` (field projection).

### services/ -- Google API Implementations

Each service is a submodule with:
- `types.rs` -- serde structs for API requests/responses
- Operation-specific files (e.g., `search.rs`, `send.rs`)

All service handlers receive a `ServiceContext` that bundles the authenticated client, output mode, and UI.

## Coding Conventions

### Error Handling

- Use `thiserror` for library-level errors (`OmegaError` enum with typed variants)
- Use `anyhow` for CLI-level error chaining where exact type does not matter
- Map errors to stable exit codes (see `error/exit.rs`)
- Never use `.unwrap()` in library code; `.expect()` only for provably safe cases

### API Types

- Use `#[serde(rename_all = "camelCase")]` to match Google API JSON conventions
- Use `#[serde(flatten)] pub extra: HashMap<String, serde_json::Value>` on response types for forward compatibility
- Use `Option<T>` for all non-required fields
- Use `#[serde(default)]` on collection fields

### Output

- Every command handler must support all three output modes (JSON/plain/text)
- JSON output uses `serde_json::to_writer_pretty` with `SetEscapeHTML(false)` equivalent
- Plain output is tab-separated, one record per line, no headers
- Text output uses colors (via crossterm) and alignment

### Async

- All command handlers are `async fn`
- Use `tokio::select!` for cancellable operations
- Use `Arc<Mutex<T>>` for shared mutable state (e.g., cached access tokens)

### Testing

- Unit tests: `#[cfg(test)] mod tests` in each module
- Mock HTTP: use `wiremock` for service-level tests
- CLI tests: use `assert_cmd` for binary invocation tests
- Integration tests: opt-in with `#[cfg(feature = "integration")]`, require real credentials

## Adding a New Google Service

1. Create `src/services/<service>/` directory with `mod.rs` and `types.rs`
2. Define API request/response types in `types.rs`
3. Implement operation functions in separate files
4. Create `src/cli/<service>.rs` with clap command structs
5. Add the service to `Command` enum in `src/cli/mod.rs`
6. Add scope mapping in `src/auth/scopes.rs`
7. Add tests

## Testing

### Unit Tests

```bash
cargo test                       # All unit tests
cargo test --lib                 # Library tests only
cargo test config::              # Tests in config module
cargo test auth::scopes          # Tests in auth::scopes module
```

### Integration Tests (requires credentials)

```bash
GOG_IT_ACCOUNT=you@gmail.com cargo test --features integration
```

### CLI Smoke Tests

```bash
cargo test --test cli_smoke
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `GOG_COLOR` | Color mode: auto/always/never |
| `GOG_JSON` | Default to JSON output |
| `GOG_PLAIN` | Default to plain output |
| `GOG_ACCOUNT` | Default Google account email |
| `GOG_CLIENT` | Default OAuth client name |
| `GOG_AUTO_JSON` | Auto-JSON when stdout is piped |
| `GOG_TIMEZONE` | Default output timezone (IANA name) |
| `GOG_ENABLE_COMMANDS` | Restrict available commands (comma-separated) |
| `GOG_KEYRING_PASSWORD` | Password for encrypted file keyring backend |
| `GOG_KEYRING_BACKEND` | Force keyring backend: auto/keychain/file |
| `GOG_CALENDAR_WEEKDAY` | Show day-of-week in calendar output |

## Milestones

| Milestone | Scope | Key Modules |
|-----------|-------|-------------|
| M1 | Scaffolding, auth, config, HTTP, output, UI | All infrastructure modules |
| M2 | Gmail, Calendar, Drive + desire paths | services/gmail, calendar, drive; cli/ service files |
| M3 | Docs (incl. sedmat), Sheets, Slides, Forms | services/docs, sheets, slides, forms |
| M4 | Chat, Classroom, Tasks, Contacts, People | services/chat, classroom, tasks, contacts, people |
| M5 | Groups, Keep, Apps Script | services/groups, keep, appscript |
| M6 | Tracking, completion, agent mode, full tests | tracking/, cli/completion.rs, cli/agent.rs |
