# Code Review: M2 — Gmail, Calendar, Drive Services

## Verdict: CHANGES REQUESTED

3 Critical, 6 Major, 10 Minor, 4 Nit findings.

## Critical Findings

### C-1 [Bug] `search` desire path alias maps to `gmail search` instead of `drive search`
- **File:** `src/cli/mod.rs`
- **Requirement:** REQ-CLI-012 specifies `search` -> `drive search`
- **Fix:** Change `"search" => Some(("gmail", "search"))` to `Some(("drive", "search"))`

### C-2 [API] Calendar RSVP `sendUpdates` placed in JSON body instead of query parameter
- **File:** `src/services/calendar/respond.rs`
- **Fix:** Return `sendUpdates` separately so handler adds it as URL query param. Add user email to attendee object.

### C-3 [Bug] `find_conflicts` uses string comparison for datetime overlap
- **File:** `src/services/calendar/events.rs`
- **Fix:** Parse datetimes to `chrono::DateTime<Utc>` before comparison.

## Major Findings

### M-1 `_include_body` parameter ignored in message search
### M-2 Silent date parse fallback masks user errors
### M-3 Missing CLI command variants for Must-priority Gmail settings
### M-4 URL parameters not URL-encoded across Calendar/Gmail
### M-5 GMAIL_BASE_URL defined 7 times
### M-6 18 production unwrap() calls in CLI dispatch
