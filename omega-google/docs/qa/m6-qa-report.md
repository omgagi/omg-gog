# QA Report: M6 (Polish)

## Scope Validated
- REQ-CLI-019: `open <target>` -- offline URL generation for Google resource IDs
- REQ-CLI-020: `completion <shell>` -- shell completion scripts (bash/zsh/fish/powershell)
- REQ-AGENT-001: `exit-codes` command -- print exit code table
- REQ-AGENT-002: `schema` / `help-json` -- machine-readable CLI command tree as JSON
- REQ-AGENT-003: `--enable-commands` enforcement for sandboxed runs
- REQ-OUTPUT-006: CSV output mode (Could priority)

## Summary
**PASS** -- All Must and Should requirements are met. All Could requirements that were in scope are implemented and working. No blocking issues found. The system builds cleanly, all 1106 unit tests pass, and all end-to-end CLI validations produce correct output.

## System Entrypoint
- **Build**: `cargo build` in `/Users/isudoajl/ownCloud/Projects/omega-tools/omega-google/`
- **Run**: `cargo run -- <subcommand>` (binary: `target/debug/omega-google`)
- **Tests**: `cargo test` (1106 tests across lib + 20 integration test files)
- **No special environment setup required** -- M6 features are offline/local only

## Traceability Matrix Status

| Requirement ID | Priority | Has Tests | Tests Pass | Acceptance Met | Notes |
|---|---|---|---|---|---|
| REQ-CLI-019 | Must | Yes (26 unit tests) | Yes | Yes | All URL types, auto-detection, canonicalization, edge cases |
| REQ-CLI-020 | Must | Yes (9 unit tests) | Yes | Yes | All 4 shells generate non-empty completions |
| REQ-AGENT-001 | Must | Yes (3 unit tests) | Yes | Yes | Full exit code table with JSON/plain/text/CSV output |
| REQ-AGENT-002 | Must | Yes (11 unit tests) | Yes | Yes | Full schema, subtree, aliases, --include-hidden, case-insensitive |
| REQ-AGENT-003 | Must | Yes (3 unit tests in cli/mod.rs) | Yes | Yes | --enable-commands and GOG_ENABLE_COMMANDS work correctly |
| REQ-OUTPUT-006 | Could | Yes (12 unit tests) | Yes | Yes | CSV mode, escaping, quoting, resolve_mode_full |

### Gaps Found
- No M6-specific integration tests in `tests/` directory -- all M6 tests are inline unit tests in source files. This is acceptable given the nature of the features (offline, no API calls), but integration tests using `assert_cmd` for binary invocation would strengthen coverage.
- REQ-OUTPUT-006 specifies "`--csv` or `--format csv` flag" but only `--csv` is implemented. `--format csv` is not available. Since this is a Could-priority requirement and `--csv` works, this is a minor gap.

## Acceptance Criteria Results

### Must Requirements

#### REQ-CLI-019: `open <target>` -- offline URL generation
- [x] `--type auto/drive/folder/docs/sheets/slides/gmail-thread` -- PASS: All types supported, including aliases (file, dir, doc, document, sheet, spreadsheet, slide, presentation, gmail, thread)
- [x] Generates web URLs from IDs without API calls -- PASS: All URL types produce correct Google URLs offline
- [x] Supports URL canonicalization -- PASS: Full URLs are parsed, IDs extracted, and canonical URLs produced. Gmail inbox URLs canonicalized to `#all/`. Query params stripped.
- [x] Auto-detection from URL -- PASS: Drive files, folders, Docs, Sheets, Slides, Gmail thread URLs all auto-detected
- [x] `-t` shorthand for `--type` -- PASS: Works correctly
- [x] `browse` alias -- PASS: `omega-google browse abc123` works
- [x] JSON output mode -- PASS: Returns `{"url": "...", "target": "..."}` with `--json`
- [x] Invalid type returns error with exit code 2 -- PASS
- [x] Missing target argument returns usage error -- PASS

#### REQ-CLI-020: `completion <shell>` -- shell completion scripts
- [x] Supports `bash` -- PASS: Produces valid bash completion script referencing `omega-google`
- [x] Supports `zsh` -- PASS: Produces valid zsh completion script with `#compdef omega-google`
- [x] Supports `fish` -- PASS: Produces valid fish completion script
- [x] Supports `powershell` -- PASS: Produces valid PowerShell completion script with `Register-ArgumentCompleter`
- [x] Outputs completion script to stdout -- PASS: All output goes to stdout
- [x] Invalid shell returns error with exit code 2 -- PASS
- [x] Missing shell argument returns usage error -- PASS

#### REQ-AGENT-001: `exit-codes` command
- [x] Lists all exit codes with names and values -- PASS: 11 exit codes listed (0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 130)
- [x] JSON output -- PASS: Array of `{code, name, description}` objects
- [x] Plain output -- PASS: Tab-separated `code\tname\tdescription`
- [x] Text output -- PASS: Formatted table with headers
- [x] CSV output -- PASS: `code,name,description` with header row
- [x] Available as `exit-codes` top-level and `agent exit-codes` -- PASS

#### REQ-AGENT-002: `schema` / `help-json` command
- [x] Outputs full CLI schema as JSON -- PASS: Full command tree with name, about, args, subcommands
- [x] Includes commands, subcommands, flags, positionals, types, defaults -- PASS: Each arg has name, type (bool/string), required, default, help, hidden, global
- [x] `--include-hidden` flag -- PASS: Accepted and processed
- [x] Optional `<command>` argument for subtree -- PASS: `schema gmail` returns Gmail subtree only
- [x] `help-json` alias -- PASS: `omega-google help-json` works identically
- [x] Unknown command returns error JSON -- PASS: Returns `{"error": "unknown command: '...'"}`
- [x] Case-insensitive lookup -- PASS: `schema Gmail` finds `gmail`
- [x] Alias lookup -- PASS: `schema cal` finds `calendar`
- [x] `--plain` mode outputs command names only -- PASS
- [x] Schema includes aliases for commands -- PASS: Calendar shows `cal` alias
- [x] Schema includes global args -- PASS: json, plain, csv, verbose, account, etc.

#### REQ-AGENT-003: `--enable-commands` enforcement
- [x] Restricts CLI to comma-separated top-level commands -- PASS: `--enable-commands gmail,calendar` blocks `open`
- [x] Rejects non-allowed commands with error -- PASS: Exit code 2 with descriptive error
- [x] `GOG_ENABLE_COMMANDS` env var support -- PASS: `GOG_ENABLE_COMMANDS=gmail cargo run -- open abc123` correctly blocked

### Could Requirements

#### REQ-OUTPUT-006: CSV output mode
- [x] Activated by `--csv` flag -- PASS: `--csv` resolves to `OutputMode::Csv`
- [x] Proper CSV escaping and quoting -- PASS: Commas, quotes (doubled), newlines all properly escaped
- [x] Mutually exclusive with `--json` and `--plain` -- PASS: Conflicts correctly enforced by clap
- [x] `GOG_CSV` env var support -- PASS: Defined in clap argument
- [ ] `--format csv` alternative -- NOT IMPLEMENTED: Only `--csv` flag is available (see Non-Blocking Observations)

## End-to-End Flow Results

| Flow | Steps | Result | Notes |
|---|---|---|---|
| Open bare ID (auto) | `open abc123` | PASS | Produces Drive file URL |
| Open bare ID (typed) | `open abc123 --type docs` | PASS | Produces Docs URL |
| Open with URL auto-detect | `open https://docs.google.com/document/d/ID/edit` | PASS | Canonicalizes URL |
| Open with URL canonicalization | `open https://drive.google.com/file/d/ID/view?usp=sharing` | PASS | Strips query params |
| Gmail inbox URL canonicalization | `open https://mail.google.com/mail/u/0/#inbox/ID` | PASS | Rewrites to `#all/ID` |
| Completion for all shells | `completion bash/zsh/fish/powershell` | PASS | Non-empty, valid scripts |
| Exit codes in all formats | `exit-codes` / `--json` / `--plain` / `--csv` | PASS | All 4 output modes work |
| Schema full tree | `schema` | PASS | Complete CLI tree as JSON |
| Schema subtree | `schema gmail` | PASS | Gmail subtree only |
| Schema alias | `help-json` | PASS | Identical to `schema` |
| Agent subcommands | `agent exit-codes` / `agent schema gmail` | PASS | Delegates correctly |
| Enable-commands blocking | `--enable-commands gmail open abc123` | PASS | Blocked with exit 2 |
| Enable-commands allowing | `--enable-commands open open abc123` | PASS | Allowed |
| Mutually exclusive flags | `--json --csv exit-codes` | PASS | Clap error, exit 2 |

## Exploratory Testing Findings

| # | What Was Tried | Expected | Actual | Severity |
|---|---|---|---|---|
| 1 | `open ""` (empty string target) | Error or warning about empty ID | Produces `https://drive.google.com/file/d//view` with exit 0 | low |
| 2 | `open "  spaces  "` (whitespace target) | Error or trimmed ID | Produces `https://drive.google.com/file/d/  spaces  /view` with exit 0 | low |
| 3 | `open "https://example.com/something"` (non-Google URL) | Error about unrecognized URL | Falls through to bare-ID path, produces `https://drive.google.com/file/d/https://example.com/something/view` | low |
| 4 | `schema nonexistent` (unknown command) | Error or non-zero exit code | Outputs `{"error": "..."}` but exits with code 0, not an error exit | low |

### Exploratory Finding 1
- **Tried**: `cargo run -- open ""` (empty string as target)
- **Expected**: An error message indicating the target ID is empty, or at minimum the URL should not have an empty path segment
- **Actual**: Produces `https://drive.google.com/file/d//view` and exits with code 0
- **Severity**: low
- **Reproducible**: yes
- **Note**: While technically not a crash, a user accidentally passing an empty string gets a malformed URL. A validation check on the target would improve UX.

### Exploratory Finding 2
- **Tried**: `cargo run -- open "  spaces  "` (whitespace-containing target)
- **Expected**: Target should be trimmed, or an error about invalid characters
- **Actual**: Produces `https://drive.google.com/file/d/  spaces  /view` with spaces in URL
- **Severity**: low
- **Reproducible**: yes
- **Note**: Google resource IDs never contain spaces. Trimming or rejecting whitespace-only/whitespace-containing targets would be a nice hardening.

### Exploratory Finding 3
- **Tried**: `cargo run -- open "https://example.com/something"` (non-Google URL)
- **Expected**: Error about unrecognized URL domain
- **Actual**: Falls through URL detection, treats the entire URL as a bare ID, produces `https://drive.google.com/file/d/https://example.com/something/view`
- **Severity**: low
- **Reproducible**: yes
- **Note**: The code correctly returns None from `detect_from_url` for non-Google URLs, but then treats the entire URL string as a bare ID. Could benefit from a warning when the target starts with `http` but is not recognized.

### Exploratory Finding 4
- **Tried**: `cargo run -- schema nonexistent`
- **Expected**: Error exit code (non-zero) when the requested command does not exist
- **Actual**: Outputs `{"error": "unknown command: 'nonexistent'"}` but exits with code 0
- **Severity**: low
- **Reproducible**: yes
- **Note**: The error is communicated via JSON structure but the exit code does not reflect the failure. An agent consuming this output would need to check the JSON for an "error" key rather than relying on the exit code.

## Failure Mode Validation

| Failure Scenario | Triggered | Detected | Recovered | Degraded OK | Notes |
|---|---|---|---|---|---|
| Invalid resource type | Yes | Yes | N/A | Yes | Descriptive error message, exit code 2 |
| Invalid shell type | Yes | Yes | N/A | Yes | Descriptive error message, exit code 2 |
| Missing required argument | Yes | Yes | N/A | Yes | Clap usage error, exit code 2 |
| Blocked command (--enable-commands) | Yes | Yes | N/A | Yes | Clear error message, exit code 2 |
| Mutually exclusive output flags | Yes | Yes | N/A | Yes | Clap conflict error, exit code 2 |
| Unknown schema command | Yes | Partial | N/A | Yes | Error in JSON but exit 0 (see Finding 4) |

## Security Validation

| Attack Surface | Test Performed | Result | Notes |
|---|---|---|---|
| Input injection via target | Sent `<script>alert(1)</script>` as target ID | PASS | Treated as literal string in URL, no execution context |
| Path traversal via target | Sent `../../etc/passwd` as target ID | PASS | Treated as literal string, no file system access |
| Shell injection via completion | Completion scripts generated by clap_complete library | PASS (Out of Scope) | Scripts are generated by a well-tested library; shell injection would require a clap_complete vulnerability |
| Enable-commands bypass | Tested case sensitivity and whitespace | PASS | Case-insensitive matching, whitespace trimmed |

## Specs/Docs Drift

| File | Documented Behavior | Actual Behavior | Severity |
|------|-------------------|-----------------|----------|
| `docs/command-reference.md` (Global Flags table) | `--csv` flag not listed in Global Flags table | `--csv` flag exists and works, with `GOG_CSV` env var | medium |
| `specs/omega-google-requirements.md` (REQ-OUTPUT-006) | "Activated by `--csv` or `--format csv` flag" | Only `--csv` is implemented; `--format csv` is not available | low |
| `specs/omega-google-requirements.md` (traceability) | "Module 6: output/" maps to `src/output/csv.rs` | CSV functionality is in `src/output/mod.rs`, no `csv.rs` file exists | low |

## Blocking Issues (must fix before merge)

None.

## Non-Blocking Observations

- **[OBS-001]**: `src/cli/open.rs` -- Empty string target produces malformed URL (`/file/d//view`). Consider adding validation to reject empty or whitespace-only targets.
- **[OBS-002]**: `src/cli/open.rs` -- Non-Google URLs (e.g., `https://example.com/...`) are treated as bare IDs rather than producing a warning. Consider warning when input starts with `http` but is not a recognized Google URL.
- **[OBS-003]**: `src/cli/mod.rs` -- `schema nonexistent` exits with code 0 despite returning an error JSON object. Consider returning a non-zero exit code when the requested command is not found.
- **[OBS-004]**: `docs/command-reference.md` -- The `--csv` global flag and `GOG_CSV` env var should be added to the Global Flags table.
- **[OBS-005]**: `specs/omega-google-requirements.md` -- The traceability matrix references `src/output/csv.rs` but CSV functionality is in `src/output/mod.rs`. The spec should be updated.
- **[OBS-006]**: No M6-specific integration tests exist in `tests/`. While inline unit tests provide good coverage, `assert_cmd`-based integration tests would validate the binary invocation path end-to-end.
- **[OBS-007]**: `src/cli/open.rs` -- Whitespace in target IDs is not trimmed or rejected. Google resource IDs never contain spaces.

## Modules Not Validated (if context limited)

All M6 modules were fully validated. No modules remain.

## Test Count Summary

| Module | Unit Tests | All Pass |
|--------|-----------|----------|
| `cli::open` (REQ-CLI-019) | 26 | Yes |
| `cli::completion` (REQ-CLI-020) | 9 | Yes |
| `cli::agent` (REQ-AGENT-001, REQ-AGENT-002) | 14 | Yes |
| `output::tests` (REQ-OUTPUT-006) | 12 | Yes |
| `cli::mod` (REQ-AGENT-003) | 3 (enable_commands) | Yes |
| **Total M6-related** | **64** | **Yes** |
| **Full test suite** | **1106 + integration** | **Yes (0 failures)** |

## Final Verdict

**PASS** -- All Must requirements (REQ-CLI-019, REQ-CLI-020, REQ-AGENT-001, REQ-AGENT-002, REQ-AGENT-003) are met. The Could requirement (REQ-OUTPUT-006) is implemented and working. No blocking issues. Seven non-blocking observations noted for future improvement. Approved for review.
