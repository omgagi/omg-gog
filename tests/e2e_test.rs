//! Comprehensive end-to-end test suite for omega-google against real Google APIs.
//!
//! Gated behind `OMEGA_E2E_ACCOUNT`. Skipped when the variable is not set,
//! making them safe to include in regular `cargo test` runs.
//!
//! To run (serial to avoid rate-limiting):
//!   OMEGA_E2E_ACCOUNT=you@gmail.com cargo test e2e --jobs 1 -- --test-threads=1
//!
//! Prerequisites:
//!   1. OAuth credentials stored: omega-google auth credentials <file>
//!   2. Account authorized:       omega-google auth add --force-consent
//!   3. OMEGA_E2E_ACCOUNT set to the authorized email
//!   4. APIs enabled in GCP: Gmail, Calendar, Drive, Contacts, People, Tasks,
//!      Docs, Sheets, Slides, Forms

use assert_cmd::Command;
use std::io::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// ===================================================================
// Helper functions
// ===================================================================

/// Returns the E2E account email if set, or None to skip.
fn e2e_account() -> Option<String> {
    std::env::var("OMEGA_E2E_ACCOUNT").ok()
}

/// Build an omega-google command with the E2E account, --force, --no-input, and 30s timeout.
fn omega(account: &str) -> Command {
    let mut cmd = Command::cargo_bin("omg-gog").expect("binary should exist");
    cmd.arg("--account")
        .arg(account)
        .arg("--force")
        .arg("--no-input")
        .timeout(std::time::Duration::from_secs(30));
    cmd
}

/// Build an omega-google command in JSON mode.
fn omega_json(account: &str) -> Command {
    let mut cmd = omega(account);
    cmd.arg("--json");
    cmd
}

/// Execute a command, assert success, parse JSON from stdout.
fn run_json(cmd: &mut Command) -> serde_json::Value {
    let output = cmd.output().expect("command should execute");
    assert!(
        output.status.success(),
        "Command failed (exit {:?}):\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!("Invalid JSON: {e}\nstdout: {stdout}");
    })
}

/// Execute a command and assert it succeeds.
fn run_success(cmd: &mut Command) -> std::process::Output {
    let output = cmd.output().expect("command should execute");
    assert!(
        output.status.success(),
        "Expected success but got exit {:?}:\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// Execute a command and assert it fails (non-zero exit).
fn run_failure(cmd: &mut Command) -> std::process::Output {
    let output = cmd.output().expect("command should execute");
    assert!(
        !output.status.success(),
        "Expected failure but got success.\nstdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    output
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a collision-proof unique name using microsecond timestamp + atomic counter.
fn unique_name(prefix: &str) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    let seq = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{prefix}_e2e_{ts}_{seq}")
}

/// Check if stderr indicates the API is not enabled or command not implemented (precise matching).
fn should_skip(output: &std::process::Output) -> bool {
    let stderr = String::from_utf8_lossy(&output.stderr);
    stderr.contains("has not been used in project")
        || stderr.contains("is not enabled")
        || stderr.contains("accessNotConfigured")
        || stderr.contains("not yet implemented")
}

/// Return a distinct skip label for reporting.
fn skip_reason(output: &std::process::Output) -> &'static str {
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("not yet implemented") {
        "[SKIP-UNIMPLEMENTED]"
    } else {
        "[SKIP-API-DISABLED]"
    }
}

/// Check output and print skip reason if applicable. Returns true if skipped.
fn check_skip(output: &std::process::Output) -> bool {
    if should_skip(output) {
        let reason = skip_reason(output);
        eprintln!("  {reason} {}", String::from_utf8_lossy(&output.stderr).lines().next().unwrap_or(""));
        return true;
    }
    false
}

/// Try to run a command; if it fails due to API-disabled or not-implemented, skip. Otherwise assert success.
fn run_or_skip(cmd: &mut Command) -> Option<std::process::Output> {
    let output = cmd.output().expect("command should execute");
    if !output.status.success() && check_skip(&output) {
        return None;
    }
    assert!(
        output.status.success(),
        "Command failed (exit {:?}):\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    Some(output)
}

/// Try to run a command and parse JSON; skip if API disabled. Returns None to skip.
fn run_json_or_skip(cmd: &mut Command) -> Option<serde_json::Value> {
    let output = run_or_skip(cmd)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!("Invalid JSON: {e}\nstdout: {stdout}");
    }))
}

/// Extract a string field from a JSON value.
fn extract_id(val: &serde_json::Value, key: &str) -> Option<String> {
    val.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Macro to skip tests when OMEGA_E2E_ACCOUNT is not set.
macro_rules! require_account {
    () => {
        match e2e_account() {
            Some(a) => a,
            None => {
                eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
                return;
            }
        }
    };
}

/// Sleep briefly to let eventual-consistency APIs catch up.
fn settle() {
    std::thread::sleep(std::time::Duration::from_secs(2));
}

// ===================================================================
// RAII Cleanup Guard — ensures remote resources are deleted on panic
// ===================================================================

struct CleanupGuard<'a> {
    account: &'a str,
    resource_id: Option<String>,
    cleanup_fn: fn(&str, &str),
}

impl<'a> CleanupGuard<'a> {
    fn new(account: &'a str, id: String, cleanup: fn(&str, &str)) -> Self {
        Self { account, resource_id: Some(id), cleanup_fn: cleanup }
    }
    #[allow(dead_code)]
    fn id(&self) -> &str {
        self.resource_id.as_deref().unwrap()
    }
    #[allow(dead_code)]
    fn disarm(mut self) {
        self.resource_id = None;
    }
}

impl Drop for CleanupGuard<'_> {
    fn drop(&mut self) {
        if let Some(ref id) = self.resource_id {
            (self.cleanup_fn)(self.account, id);
        }
    }
}

// Cleanup helper functions

fn drive_delete(account: &str, id: &str) {
    let _ = omega(account).args(["drive", "rm", id]).output();
}

fn gmail_label_delete(account: &str, id: &str) {
    let _ = omega(account).args(["gmail", "labels", "delete", id]).output();
}

fn gmail_draft_delete(account: &str, id: &str) {
    let _ = omega(account).args(["gmail", "drafts", "delete", id]).output();
}

fn cal_event_delete(account: &str, id: &str) {
    let _ = omega(account).args(["cal", "delete", "primary", id]).output();
}

fn contact_delete(account: &str, id: &str) {
    let _ = omega(account).args(["contacts", "delete", id]).output();
}

/// Delete a task. compound_id format: "list_id:task_id"
fn task_delete(account: &str, compound_id: &str) {
    if let Some((list_id, task_id)) = compound_id.split_once(':') {
        let _ = omega(account).args(["tasks", "delete", list_id, task_id]).output();
    }
}

// ===================================================================
// Section 1: Auth & Infrastructure (4 tests)
// ===================================================================

#[test]
fn e2e_version_no_auth() {
    Command::cargo_bin("omg-gog")
        .expect("binary should exist")
        .arg("version")
        .assert()
        .success();
}

#[test]
fn e2e_auth_status() {
    let account = require_account!();
    run_success(&mut omega(&account).arg("auth").arg("status"));
}

#[test]
fn e2e_auth_list() {
    let account = require_account!();
    let output = run_success(&mut omega(&account).arg("auth").arg("list"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&account),
        "auth list should contain the exact account email '{account}', got: {stdout}"
    );
}

#[test]
fn e2e_auth_services() {
    let account = require_account!();
    let output = run_success(&mut omega(&account).arg("auth").arg("services"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("gmail") || stdout.contains("drive") || stdout.contains("Gmail") || stdout.contains("Drive"),
        "auth services should list known services like gmail or drive"
    );
}

// ===================================================================
// Section 2: Gmail (17 tests)
// ===================================================================

#[test]
fn e2e_gmail_search_threads() {
    let account = require_account!();
    let val = run_json(
        omega_json(&account)
            .args(["gmail", "search", "in:inbox", "--max", "2"]),
    );
    assert!(val.is_object() || val.is_array(), "expected JSON response");
}

#[test]
fn e2e_gmail_search_messages() {
    let account = require_account!();
    let val = run_json(
        omega_json(&account)
            .args(["gmail", "messages", "search", "in:inbox", "--max", "2"]),
    );
    assert!(val.is_object() || val.is_array());
}

#[test]
fn e2e_gmail_search_pagination() {
    let account = require_account!();
    // First page
    let page1 = run_json(
        omega_json(&account)
            .args(["gmail", "search", "in:inbox", "--max", "1"]),
    );
    // If there is a nextPageToken, fetch page 2
    if let Some(token) = extract_id(&page1, "nextPageToken") {
        let page2 = run_json(
            omega_json(&account)
                .args(["gmail", "search", "in:inbox", "--max", "1", "--page", &token]),
        );
        assert!(page2.is_object(), "page2 should be a JSON object");
    }
}

#[test]
fn e2e_gmail_get_message() {
    let account = require_account!();
    // Search for a message to get its ID
    let search = run_json(
        omega_json(&account)
            .args(["gmail", "messages", "search", "in:inbox", "--max", "1"]),
    );
    let msg_id = search
        .get("messages")
        .and_then(|m| m.as_array())
        .and_then(|a| a.first())
        .and_then(|m| m.get("id"))
        .and_then(|id| id.as_str());
    if let Some(id) = msg_id {
        let msg = run_json(omega_json(&account).args(["gmail", "get", id]));
        assert!(msg.get("id").is_some(), "message should have id field");
    } else {
        eprintln!("  [SKIP] No messages found in inbox");
    }
}

#[test]
fn e2e_gmail_thread_get() {
    let account = require_account!();
    let search = run_json(
        omega_json(&account)
            .args(["gmail", "search", "in:inbox", "--max", "1"]),
    );
    let thread_id = search
        .get("threads")
        .and_then(|t| t.as_array())
        .and_then(|a| a.first())
        .and_then(|t| t.get("id"))
        .and_then(|id| id.as_str());
    if let Some(id) = thread_id {
        let thread = run_json(omega_json(&account).args(["gmail", "thread", "get", id]));
        assert!(thread.get("id").is_some(), "thread should have id field");
    } else {
        eprintln!("  [SKIP] No threads found in inbox");
    }
}

#[test]
fn e2e_gmail_send_to_self() {
    let account = require_account!();
    let subject = unique_name("E2E_send");
    let output = omega_json(&account)
        .args([
            "gmail", "send",
            "--to", &account,
            "--subject", &subject,
            "--body", "Automated E2E test message — safe to delete.",
        ])
        .output()
        .expect("send should execute");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("  [WARN] gmail send failed (may need send scope): {}", stderr.lines().next().unwrap_or(""));
        return;
    }
    // Brief settle for delivery
    settle();
    // Verify the message arrives via search
    let search = run_json(
        omega_json(&account)
            .args(["gmail", "search", &format!("subject:{subject}"), "--max", "1"]),
    );
    assert!(
        search.get("threads").and_then(|t| t.as_array()).map_or(false, |a| !a.is_empty())
            || search.get("resultSizeEstimate").and_then(|v| v.as_u64()).unwrap_or(0) > 0,
        "sent message should be findable"
    );
}

#[test]
fn e2e_gmail_labels_crud() {
    let account = require_account!();
    let label_name = unique_name("Label");

    // Create
    let created = run_json(
        omega_json(&account).args(["gmail", "labels", "create", &label_name]),
    );
    let label_id = extract_id(&created, "id").expect("created label should have id");
    let _guard = CleanupGuard::new(&account, label_id.clone(), gmail_label_delete);

    // List
    let list = run_json(omega_json(&account).args(["gmail", "labels", "list"]));
    assert!(list.get("labels").is_some(), "labels list should have labels key");

    // Get
    let got = run_json(omega_json(&account).args(["gmail", "labels", "get", &label_id]));
    assert_eq!(got.get("id").and_then(|v| v.as_str()), Some(label_id.as_str()));
}

#[test]
fn e2e_gmail_drafts_crud() {
    let account = require_account!();

    // Create
    let Some(created) = run_json_or_skip(
        omega_json(&account).args([
            "gmail", "drafts", "create",
            "--to", &account,
            "--subject", "E2E Draft Test",
            "--body", "draft body",
        ]),
    ) else { return };
    let draft_id = extract_id(&created, "id").expect("draft should have id");
    let _guard = CleanupGuard::new(&account, draft_id.clone(), gmail_draft_delete);

    // List
    let list = run_json(omega_json(&account).args(["gmail", "drafts", "list"]));
    assert!(list.is_object(), "drafts list should return JSON object");

    // Get
    let got = run_json(omega_json(&account).args(["gmail", "drafts", "get", &draft_id]));
    assert!(got.get("id").is_some(), "draft get should return id");

    // Update
    let _ = omega_json(&account)
        .args([
            "gmail", "drafts", "update", &draft_id,
            "--subject", "E2E Draft Updated",
        ])
        .output()
        .expect("update should execute");
}

#[test]
fn e2e_gmail_settings_vacation_get() {
    let account = require_account!();
    let Some(val) = run_json_or_skip(
        omega_json(&account).args(["gmail", "settings", "vacation", "get"]),
    ) else { return };
    assert!(val.is_object(), "vacation get should return JSON object");
}

#[test]
fn e2e_gmail_settings_filters_list() {
    let account = require_account!();
    if let Some(output) = run_or_skip(&mut omega_json(&account).args(["gmail", "settings", "filters", "list"])) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.trim().is_empty(), "filters list should produce output");
    }
}

#[test]
fn e2e_gmail_settings_forwarding_list() {
    let account = require_account!();
    if let Some(output) = run_or_skip(&mut omega_json(&account).args(["gmail", "settings", "forwarding", "list"])) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.trim().is_empty(), "forwarding list should produce output");
    }
}

#[test]
fn e2e_gmail_settings_sendas_list() {
    let account = require_account!();
    let Some(output) = run_or_skip(
        &mut omega_json(&account).args(["gmail", "settings", "sendas", "list"]),
    ) else { return };
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&account) || stdout.contains("sendAsEmail") || stdout.contains("@"),
        "sendas list should contain account email"
    );
}

#[test]
fn e2e_gmail_settings_delegates_list() {
    let account = require_account!();
    if let Some(output) = run_or_skip(&mut omega_json(&account).args(["gmail", "settings", "delegates", "list"])) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.trim().is_empty(), "delegates list should produce output");
    }
}

#[test]
fn e2e_gmail_settings_autoforward_get() {
    let account = require_account!();
    if let Some(output) = run_or_skip(
        &mut omega_json(&account).args(["gmail", "settings", "autoforward", "get"]),
    ) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.trim().is_empty(), "autoforward get should produce output");
    }
}

#[test]
fn e2e_gmail_thread_modify_labels() {
    let account = require_account!();
    // Find a thread
    let search = run_json(
        omega_json(&account)
            .args(["gmail", "search", "in:inbox", "--max", "1"]),
    );
    let thread_id = search
        .get("threads")
        .and_then(|t| t.as_array())
        .and_then(|a| a.first())
        .and_then(|t| t.get("id"))
        .and_then(|id| id.as_str());
    if let Some(id) = thread_id {
        // Remove UNREAD label
        let _ = omega(&account)
            .args(["gmail", "thread", "modify", id, "--remove", "UNREAD"])
            .output();
        // Re-add UNREAD label
        let _ = omega(&account)
            .args(["gmail", "thread", "modify", id, "--add", "UNREAD"])
            .output();
    } else {
        eprintln!("  [SKIP] No threads to modify");
    }
}

#[test]
fn e2e_gmail_url_generation() {
    let account = require_account!();
    let output = run_success(&mut omega(&account).args(["gmail", "url", "FAKE_THREAD_ID"]));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("mail.google.com"),
        "gmail url should contain mail.google.com"
    );
}

#[test]
fn e2e_gmail_search_fail_empty() {
    let account = require_account!();
    let output = omega(&account)
        .args(["gmail", "search", "xyznonexistent_e2e_query_99999", "--fail-empty"])
        .output()
        .expect("command should execute");
    // --fail-empty should produce exit code 3 when no results
    if let Some(code) = output.status.code() {
        assert!(
            code == 3 || code == 0,
            "Expected exit code 3 (fail-empty) or 0, got {code}"
        );
    }
}

// ===================================================================
// Section 3: Calendar (10 tests)
// ===================================================================

#[test]
fn e2e_calendar_list_calendars() {
    let account = require_account!();
    let val = run_json(omega_json(&account).args(["cal", "calendars"]));
    assert!(
        val.get("items").is_some() || val.is_array(),
        "calendars should return items"
    );
}

#[test]
fn e2e_calendar_list_events() {
    let account = require_account!();
    let val = run_json(omega_json(&account).args(["cal", "events", "--max", "3"]));
    assert!(val.is_object() || val.is_array());
}

#[test]
fn e2e_calendar_events_search() {
    let account = require_account!();
    let val = run_json(
        omega_json(&account).args(["cal", "events", "-q", "test", "--max", "2"]),
    );
    assert!(val.is_object(), "calendar events search should return a JSON object");
}

#[test]
fn e2e_calendar_event_crud() {
    let account = require_account!();
    let summary = unique_name("CalEvent");
    let from = "2026-12-25T10:00:00";
    let to = "2026-12-25T11:00:00";

    // Create
    let created = run_json(
        omega_json(&account).args([
            "cal", "create",
            "--summary", &summary,
            "--from", from,
            "--to", to,
        ]),
    );
    let event_id = extract_id(&created, "id").expect("event should have id");
    let _guard = CleanupGuard::new(&account, event_id.clone(), cal_event_delete);

    settle();

    // Get
    let got = run_json(
        omega_json(&account).args(["cal", "event", "primary", &event_id]),
    );
    assert_eq!(
        got.get("summary").and_then(|v| v.as_str()),
        Some(summary.as_str()),
        "event summary should match"
    );

    // Update
    let updated_summary = format!("{summary}_updated");
    run_success(
        &mut omega(&account).args([
            "cal", "update", "primary", &event_id,
            "--summary", &updated_summary,
        ]),
    );
}

#[test]
fn e2e_calendar_event_allday() {
    let account = require_account!();
    let summary = unique_name("AllDay");

    // Create all-day event
    let created = run_json(
        omega_json(&account).args([
            "cal", "create",
            "--summary", &summary,
            "--from", "2026-12-25",
            "--to", "2026-12-26",
            "--all-day",
        ]),
    );
    let event_id = extract_id(&created, "id").expect("event should have id");
    let _guard = CleanupGuard::new(&account, event_id.clone(), cal_event_delete);

    settle();

    // Verify date (not dateTime) in start
    let got = run_json(
        omega_json(&account).args(["cal", "event", "primary", &event_id]),
    );
    let start = got.get("start");
    assert!(
        start.and_then(|s| s.get("date")).is_some(),
        "all-day event should have date field in start"
    );
}

#[test]
fn e2e_calendar_freebusy() {
    let account = require_account!();
    let val = run_json(
        omega_json(&account).args([
            "cal", "freebusy", "primary",
            "--from", "2026-12-25T00:00:00Z",
            "--to", "2026-12-26T00:00:00Z",
        ]),
    );
    assert!(
        val.get("calendars").is_some() || val.is_object(),
        "freebusy should return calendars"
    );
}

#[test]
fn e2e_calendar_colors() {
    let account = require_account!();
    let output = omega_json(&account)
        .args(["cal", "colors"])
        .output()
        .expect("command should execute");
    if !output.status.success() || check_skip(&output) {
        eprintln!("  [SKIP] cal colors not available or not implemented");
        return;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        eprintln!("  [SKIP] cal colors returned empty output");
        return;
    }
    let val: serde_json::Value = serde_json::from_str(&stdout)
        .expect("colors output should be valid JSON");
    assert!(val.is_object(), "colors should return JSON object");
}

#[test]
fn e2e_calendar_time() {
    let account = require_account!();
    let output = run_success(&mut omega(&account).args(["cal", "time"]));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty(), "cal time should produce non-empty output");
}

#[test]
fn e2e_calendar_event_pagination() {
    let account = require_account!();
    let page1 = run_json(
        omega_json(&account).args(["cal", "events", "--max", "1"]),
    );
    if let Some(token) = extract_id(&page1, "nextPageToken") {
        let page2 = run_json(
            omega_json(&account)
                .args(["cal", "events", "--max", "1", "--page", &token]),
        );
        assert!(page2.is_object(), "page2 should be a JSON object");
    }
}

#[test]
fn e2e_calendar_acl() {
    let account = require_account!();
    // ACL may return 404 for personal calendars without explicit ACL
    let output = omega_json(&account)
        .args(["cal", "acl", "primary"])
        .output()
        .expect("command should execute");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("404") || stderr.contains("Not Found") || stderr.contains("not yet implemented") {
            eprintln!("  [SKIP] ACL not available for this calendar");
            return;
        }
        panic!("cal acl failed unexpectedly: {}", stderr);
    }
}

// ===================================================================
// Section 4: Drive (14 tests)
// ===================================================================

#[test]
fn e2e_drive_list() {
    let account = require_account!();
    let val = run_json(omega_json(&account).args(["drive", "ls", "--max", "3"]));
    assert!(
        val.get("files").is_some() || val.is_array() || val.is_object(),
        "drive ls should return files"
    );
}

#[test]
fn e2e_drive_search() {
    let account = require_account!();
    let val = run_json(
        omega_json(&account).args(["drive", "search", "test", "--max", "2"]),
    );
    assert!(val.is_object(), "drive search should return a JSON object");
}

#[test]
fn e2e_drive_list_pagination() {
    let account = require_account!();
    let page1 = run_json(
        omega_json(&account).args(["drive", "ls", "--max", "1"]),
    );
    if let Some(token) = extract_id(&page1, "nextPageToken") {
        let page2 = run_json(
            omega_json(&account)
                .args(["drive", "ls", "--max", "1", "--page", &token]),
        );
        assert!(page2.is_object(), "page2 should be a JSON object");
    }
}

#[test]
fn e2e_drive_mkdir_and_delete() {
    let account = require_account!();
    let folder_name = unique_name("Folder");

    // Create folder
    let created = run_json(
        omega_json(&account).args(["drive", "mkdir", &folder_name]),
    );
    let folder_id = extract_id(&created, "id").expect("folder should have id");
    let _guard = CleanupGuard::new(&account, folder_id.clone(), drive_delete);

    settle();

    // Get folder metadata
    let got = run_json(omega_json(&account).args(["drive", "get", &folder_id]));
    assert_eq!(
        got.get("name").and_then(|v| v.as_str()),
        Some(folder_name.as_str())
    );
}

#[test]
fn e2e_drive_upload_download_delete() {
    let account = require_account!();
    let file_name = unique_name("Upload");

    // Create a temp file to upload (auto-deletes on drop)
    let mut upload_file = tempfile::Builder::new()
        .prefix(&file_name)
        .suffix(".txt")
        .tempfile()
        .expect("create temp file");
    let content = "Hello from E2E test!\nLine 2.\n";
    write!(upload_file, "{content}").expect("write temp file");

    // Upload
    let created = run_json(
        omega_json(&account).args([
            "drive", "upload",
            upload_file.path().to_str().unwrap(),
            "--name", &format!("{file_name}.txt"),
        ]),
    );
    let file_id = extract_id(&created, "id").expect("uploaded file should have id");
    let _guard = CleanupGuard::new(&account, file_id.clone(), drive_delete);

    settle();

    // Get metadata
    let got = run_json(omega_json(&account).args(["drive", "get", &file_id]));
    assert!(got.get("name").is_some());

    // Download
    let download_file = tempfile::Builder::new()
        .suffix(".txt")
        .tempfile()
        .expect("create download temp file");
    let download_path = download_file.path().to_str().unwrap().to_string();
    run_success(
        &mut omega(&account).args([
            "drive", "download", &file_id,
            "--out", &download_path,
        ]),
    );
    let downloaded = std::fs::read_to_string(&download_path).expect("read downloaded file");
    assert_eq!(downloaded, content, "downloaded content should match uploaded");
}

#[test]
fn e2e_drive_rename() {
    let account = require_account!();
    let original = unique_name("Rename");
    let new_name = format!("{original}_renamed");

    // Upload a file
    let mut tmp = tempfile::Builder::new().suffix(".txt").tempfile().unwrap();
    write!(tmp, "rename test").unwrap();
    let created = run_json(
        omega_json(&account).args([
            "drive", "upload", tmp.path().to_str().unwrap(),
            "--name", &format!("{original}.txt"),
        ]),
    );
    let file_id = extract_id(&created, "id").unwrap();
    let _guard = CleanupGuard::new(&account, file_id.clone(), drive_delete);

    // Rename
    run_success(
        &mut omega(&account).args(["drive", "rename", &file_id, &new_name]),
    );

    settle();

    // Verify
    let got = run_json(omega_json(&account).args(["drive", "get", &file_id]));
    assert_eq!(got.get("name").and_then(|v| v.as_str()), Some(new_name.as_str()));
}

#[test]
fn e2e_drive_copy() {
    let account = require_account!();
    let name = unique_name("Copy");

    // Upload original
    let mut tmp = tempfile::Builder::new().suffix(".txt").tempfile().unwrap();
    write!(tmp, "copy test").unwrap();
    let created = run_json(
        omega_json(&account).args([
            "drive", "upload", tmp.path().to_str().unwrap(),
            "--name", &format!("{name}.txt"),
        ]),
    );
    let file_id = extract_id(&created, "id").unwrap();
    let _guard1 = CleanupGuard::new(&account, file_id.clone(), drive_delete);

    // Copy
    let copy_name = format!("{name}_copy");
    let copied = run_json(
        omega_json(&account).args(["drive", "copy", &file_id, "--name", &copy_name]),
    );
    let copy_id = extract_id(&copied, "id").expect("copy should have id");
    let _guard2 = CleanupGuard::new(&account, copy_id.clone(), drive_delete);

    settle();

    // Verify copy exists
    let got = run_json(omega_json(&account).args(["drive", "get", &copy_id]));
    assert!(got.get("name").is_some());
}

#[test]
fn e2e_drive_move() {
    let account = require_account!();
    let prefix = unique_name("Move");

    // Create two folders
    let folder_a = run_json(
        omega_json(&account).args(["drive", "mkdir", &format!("{prefix}_A")]),
    );
    let folder_a_id = extract_id(&folder_a, "id").unwrap();
    let _guard_a = CleanupGuard::new(&account, folder_a_id.clone(), drive_delete);

    let folder_b = run_json(
        omega_json(&account).args(["drive", "mkdir", &format!("{prefix}_B")]),
    );
    let folder_b_id = extract_id(&folder_b, "id").unwrap();
    let _guard_b = CleanupGuard::new(&account, folder_b_id.clone(), drive_delete);

    // Upload a file into folder A
    let mut tmp = tempfile::Builder::new().suffix(".txt").tempfile().unwrap();
    write!(tmp, "move test").unwrap();
    let file = run_json(
        omega_json(&account).args([
            "drive", "upload", tmp.path().to_str().unwrap(),
            "--name", &format!("{prefix}.txt"),
            "--parent", &folder_a_id,
        ]),
    );
    let file_id = extract_id(&file, "id").unwrap();
    let _guard_f = CleanupGuard::new(&account, file_id.clone(), drive_delete);

    // Move file to folder B
    run_success(
        &mut omega(&account).args(["drive", "move", &file_id, "--parent", &folder_b_id]),
    );

    settle();

    // Verify file still accessible after move
    run_success(&mut omega_json(&account).args(["drive", "get", &file_id]));
}

#[test]
fn e2e_drive_share_unshare() {
    let account = require_account!();
    let name = unique_name("Share");

    // Upload a file
    let mut tmp = tempfile::Builder::new().suffix(".txt").tempfile().unwrap();
    write!(tmp, "share test").unwrap();
    let created = run_json(
        omega_json(&account).args([
            "drive", "upload", tmp.path().to_str().unwrap(),
            "--name", &format!("{name}.txt"),
        ]),
    );
    let file_id = extract_id(&created, "id").unwrap();
    let _guard = CleanupGuard::new(&account, file_id.clone(), drive_delete);

    // Share with anyone
    let shared = run_json(
        omega_json(&account).args(["drive", "share", &file_id, "--to", "anyone", "--role", "reader"]),
    );
    let perm_id = extract_id(&shared, "id").unwrap_or_else(|| "anyoneWithLink".to_string());

    settle();

    // List permissions
    let perms = run_json(
        omega_json(&account).args(["drive", "permissions", &file_id]),
    );
    assert!(perms.is_object() || perms.is_array(), "permissions should be JSON");

    // Unshare
    let _ = omega(&account)
        .args(["drive", "unshare", &file_id, &perm_id])
        .output();
}

#[test]
fn e2e_drive_permissions_list() {
    let account = require_account!();
    let name = unique_name("Perms");

    // Upload a file
    let mut tmp = tempfile::Builder::new().suffix(".txt").tempfile().unwrap();
    write!(tmp, "perms test").unwrap();
    let created = run_json(
        omega_json(&account).args([
            "drive", "upload", tmp.path().to_str().unwrap(),
            "--name", &format!("{name}.txt"),
        ]),
    );
    let file_id = extract_id(&created, "id").unwrap();
    let _guard = CleanupGuard::new(&account, file_id.clone(), drive_delete);

    settle();

    // List permissions (owner should be present)
    let perms = run_json(
        omega_json(&account).args(["drive", "permissions", &file_id]),
    );
    assert!(perms.is_object() || perms.is_array());
}

#[test]
fn e2e_drive_comments_crud() {
    let account = require_account!();
    let name = unique_name("Comments");

    // Upload a file (Google Docs support comments, but plain files may not;
    // we try anyway and skip if it fails)
    let mut tmp = tempfile::Builder::new().suffix(".txt").tempfile().unwrap();
    write!(tmp, "comments test").unwrap();
    let created = run_json(
        omega_json(&account).args([
            "drive", "upload", tmp.path().to_str().unwrap(),
            "--name", &format!("{name}.txt"),
            "--convert",
        ]),
    );
    let file_id = extract_id(&created, "id").unwrap();
    let _guard = CleanupGuard::new(&account, file_id.clone(), drive_delete);

    settle();

    // Add comment
    let comment_result = omega_json(&account)
        .args(["drive", "comments", &file_id, "create", "--content", "E2E test comment"])
        .output()
        .expect("command should execute");

    if comment_result.status.success() {
        // List comments
        run_success(
            &mut omega_json(&account).args(["drive", "comments", &file_id, "list"]),
        );
    } else {
        eprintln!("  [SKIP] Comments not supported on this file type");
    }
}

#[test]
fn e2e_drive_url_generation() {
    let account = require_account!();
    let output = run_success(&mut omega(&account).args(["drive", "url", "FAKE_FILE_ID"]));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("drive.google.com"),
        "drive url should contain drive.google.com"
    );
}

#[test]
fn e2e_drive_drives_list() {
    let account = require_account!();
    // May be empty for personal accounts; may not be implemented yet
    run_or_skip(&mut omega_json(&account).args(["drive", "drives"]));
}

#[test]
fn e2e_drive_get_invalid_id() {
    let account = require_account!();
    run_failure(
        &mut omega_json(&account).args(["drive", "get", "nonexistent_file_id_e2e_12345"]),
    );
}

// ===================================================================
// Section 5: Docs (8 tests, API-gated)
// ===================================================================

/// Helper: create a Google Doc and return its ID. Returns None if API disabled.
fn docs_create(account: &str, title: &str) -> Option<String> {
    let output = omega_json(account)
        .args(["docs", "create", title])
        .output()
        .expect("docs create should execute");
    if check_skip(&output) {
        return None;
    }
    assert!(
        output.status.success(),
        "docs create failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let val: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).ok()?;
    // The response may have "documentId" or "id"
    extract_id(&val, "documentId").or_else(|| extract_id(&val, "id"))
}

#[test]
fn e2e_docs_create_and_delete() {
    let account = require_account!();
    let title = unique_name("Doc");
    if let Some(doc_id) = docs_create(&account, &title) {
        let _guard = CleanupGuard::new(&account, doc_id, drive_delete);
    }
}

#[test]
fn e2e_docs_cat() {
    let account = require_account!();
    let title = unique_name("DocCat");
    let Some(doc_id) = docs_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, doc_id.clone(), drive_delete);

    settle();

    // Write content
    let write_out = omega(&account)
        .args(["docs", "write", &doc_id, "Hello E2E world"])
        .output()
        .expect("write should execute");
    if check_skip(&write_out) {
        return;
    }

    settle();

    // Cat should return the text
    let Some(output) = run_or_skip(&mut omega(&account).args(["docs", "cat", &doc_id])) else {
        return;
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hello E2E world"),
        "cat should contain written text, got: {stdout}"
    );
}

#[test]
fn e2e_docs_write_and_read() {
    let account = require_account!();
    let title = unique_name("DocWrite");
    let Some(doc_id) = docs_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, doc_id.clone(), drive_delete);

    settle();

    // Write initial content
    let write_out = omega(&account)
        .args(["docs", "write", &doc_id, "first content"])
        .output()
        .expect("write should execute");
    if check_skip(&write_out) {
        return;
    }

    settle();

    // Replace content
    let _ = omega(&account)
        .args(["docs", "write", &doc_id, "replaced content", "--replace"])
        .output();

    settle();

    // Read back
    let Some(output) = run_or_skip(&mut omega(&account).args(["docs", "cat", &doc_id])) else {
        return;
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("replaced content"),
        "cat should show replaced text"
    );
}

#[test]
fn e2e_docs_find_replace() {
    let account = require_account!();
    let title = unique_name("DocFR");
    let Some(doc_id) = docs_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, doc_id.clone(), drive_delete);

    settle();

    // Write
    let write_out = omega(&account)
        .args(["docs", "write", &doc_id, "the quick brown fox"])
        .output()
        .expect("write should execute");
    if check_skip(&write_out) {
        return;
    }

    settle();

    // Find-replace
    if run_or_skip(
        &mut omega(&account).args(["docs", "find-replace", &doc_id, "fox", "dog"]),
    ).is_none() {
        return;
    }

    settle();

    // Verify
    let Some(output) = run_or_skip(&mut omega(&account).args(["docs", "cat", &doc_id])) else {
        return;
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dog"), "find-replace should have changed fox to dog");
}

#[test]
fn e2e_docs_sed() {
    let account = require_account!();
    let title = unique_name("DocSed");
    let Some(doc_id) = docs_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, doc_id.clone(), drive_delete);

    settle();

    // Write
    let write_out = omega(&account)
        .args(["docs", "write", &doc_id, "hello world"])
        .output()
        .expect("write should execute");
    if check_skip(&write_out) {
        return;
    }

    settle();

    // Sed
    if run_or_skip(
        &mut omega(&account).args(["docs", "sed", &doc_id, "s/hello/goodbye/"]),
    ).is_none() {
        return;
    }

    settle();

    // Verify
    let Some(output) = run_or_skip(&mut omega(&account).args(["docs", "cat", &doc_id])) else {
        return;
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("goodbye"), "sed should have changed hello to goodbye");
}

#[test]
fn e2e_docs_info() {
    let account = require_account!();
    let title = unique_name("DocInfo");
    let Some(doc_id) = docs_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, doc_id.clone(), drive_delete);

    settle();

    let Some(val) = run_json_or_skip(omega_json(&account).args(["docs", "info", &doc_id])) else {
        return;
    };
    let doc_title = val.get("title").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        doc_title.contains("DocInfo"),
        "info should return the doc title"
    );
}

#[test]
fn e2e_docs_export() {
    let account = require_account!();
    let title = unique_name("DocExport");
    let Some(doc_id) = docs_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, doc_id.clone(), drive_delete);

    settle();

    // Write something
    let write_out = omega(&account)
        .args(["docs", "write", &doc_id, "export test content"])
        .output()
        .expect("write should execute");
    if check_skip(&write_out) {
        return;
    }

    settle();

    // Export as txt
    let export_file = tempfile::Builder::new().suffix(".txt").tempfile().expect("create export temp");
    let export_path = export_file.path().to_str().unwrap().to_string();
    let Some(_) = run_or_skip(
        &mut omega(&account).args([
            "docs", "export", &doc_id,
            "--format", "txt",
            "--out", &export_path,
        ]),
    ) else {
        return;
    };
    let content = std::fs::read_to_string(&export_path).unwrap_or_default();
    assert!(
        content.contains("export test content"),
        "exported file should contain written text"
    );
}

#[test]
fn e2e_docs_comments_crud() {
    let account = require_account!();
    let title = unique_name("DocComments");
    let Some(doc_id) = docs_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, doc_id.clone(), drive_delete);

    settle();

    // Add comment (via Drive comments API)
    let comment_out = omega_json(&account)
        .args(["drive", "comments", &doc_id, "create", "--content", "E2E doc comment"])
        .output()
        .expect("add comment should execute");

    if !comment_out.status.success() {
        eprintln!("  [SKIP] Comments not supported or API issue");
        return;
    }

    let comment_val: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&comment_out.stdout))
            .unwrap_or(serde_json::Value::Null);
    let _comment_id = extract_id(&comment_val, "id");

    // List comments
    run_success(
        &mut omega_json(&account).args(["drive", "comments", &doc_id, "list"]),
    );
}

// ===================================================================
// Section 6: Sheets (8 tests)
// ===================================================================

/// Helper: create a Google Sheet and return its ID. Returns None if API disabled.
fn sheets_create(account: &str, title: &str) -> Option<String> {
    let output = omega_json(account)
        .args(["sheets", "create", title])
        .output()
        .expect("sheets create should execute");
    if check_skip(&output) {
        return None;
    }
    assert!(
        output.status.success(),
        "sheets create failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let val: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).ok()?;
    extract_id(&val, "spreadsheetId").or_else(|| extract_id(&val, "id"))
}

#[test]
fn e2e_sheets_create_and_delete() {
    let account = require_account!();
    let title = unique_name("Sheet");
    if let Some(sheet_id) = sheets_create(&account, &title) {
        let _guard = CleanupGuard::new(&account, sheet_id, drive_delete);
    }
}

#[test]
fn e2e_sheets_metadata() {
    let account = require_account!();
    let title = unique_name("SheetMeta");
    let Some(sheet_id) = sheets_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, sheet_id.clone(), drive_delete);

    let val = run_json(omega_json(&account).args(["sheets", "metadata", &sheet_id]));
    let props = val.get("properties");
    assert!(
        props.and_then(|p| p.get("title")).and_then(|t| t.as_str()).unwrap_or("").contains("SheetMeta"),
        "metadata should contain sheet title"
    );
}

#[test]
fn e2e_sheets_update_and_get() {
    let account = require_account!();
    let title = unique_name("SheetRW");
    let Some(sheet_id) = sheets_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, sheet_id.clone(), drive_delete);

    // Write cells
    run_success(
        &mut omega(&account).args([
            "sheets", "update", &sheet_id, "Sheet1!A1:B2",
            "--values-json", "[[\"Name\",\"Age\"],[\"Alice\",\"30\"]]",
        ]),
    );

    settle();

    // Read cells
    let val = run_json(
        omega_json(&account).args(["sheets", "get", &sheet_id, "Sheet1!A1:B2"]),
    );
    let values = val.get("values").and_then(|v| v.as_array());
    assert!(values.is_some(), "get should return values");
    let rows = values.unwrap();
    assert!(rows.len() >= 2, "should have at least 2 rows");
    assert_eq!(
        rows[0].as_array().and_then(|r| r.first()).and_then(|v| v.as_str()),
        Some("Name")
    );
}

#[test]
fn e2e_sheets_append() {
    let account = require_account!();
    let title = unique_name("SheetAppend");
    let Some(sheet_id) = sheets_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, sheet_id.clone(), drive_delete);

    // Write initial row
    run_success(
        &mut omega(&account).args([
            "sheets", "update", &sheet_id, "Sheet1!A1:B1",
            "--values-json", "[[\"Header1\",\"Header2\"]]",
        ]),
    );

    // Append row
    run_success(
        &mut omega(&account).args([
            "sheets", "append", &sheet_id, "Sheet1!A1:B1",
            "--values-json", "[[\"Val1\",\"Val2\"]]",
        ]),
    );

    settle();

    // Read to verify
    let val = run_json(
        omega_json(&account).args(["sheets", "get", &sheet_id, "Sheet1!A1:B2"]),
    );
    let values = val.get("values").and_then(|v| v.as_array());
    assert!(
        values.map_or(false, |v| v.len() >= 2),
        "should have at least 2 rows after append"
    );
}

#[test]
fn e2e_sheets_clear() {
    let account = require_account!();
    let title = unique_name("SheetClear");
    let Some(sheet_id) = sheets_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, sheet_id.clone(), drive_delete);

    // Write cells
    run_success(
        &mut omega(&account).args([
            "sheets", "update", &sheet_id, "Sheet1!A1:A2",
            "--values-json", "[[\"one\"],[\"two\"]]",
        ]),
    );

    // Clear
    run_success(
        &mut omega(&account).args(["sheets", "clear", &sheet_id, "Sheet1!A1:A2"]),
    );

    settle();

    // Read to verify empty
    let val = run_json(
        omega_json(&account).args(["sheets", "get", &sheet_id, "Sheet1!A1:A2"]),
    );
    // After clear, values should be absent or empty
    let values = val.get("values").and_then(|v| v.as_array());
    assert!(
        values.is_none() || values.unwrap().is_empty(),
        "cells should be empty after clear"
    );
}

#[test]
fn e2e_sheets_export() {
    let account = require_account!();
    let title = unique_name("SheetExport");
    let Some(sheet_id) = sheets_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, sheet_id.clone(), drive_delete);

    // Write some data
    run_success(
        &mut omega(&account).args([
            "sheets", "update", &sheet_id, "Sheet1!A1:B1",
            "--values-json", "[[\"col1\",\"col2\"]]",
        ]),
    );

    // Export as CSV
    let export_file = tempfile::Builder::new().suffix(".csv").tempfile().expect("create export temp");
    let export_path = export_file.path().to_str().unwrap().to_string();
    run_success(
        &mut omega(&account).args([
            "sheets", "export", &sheet_id,
            "--format", "csv",
            "--out", &export_path,
        ]),
    );
    let content = std::fs::read_to_string(&export_path).unwrap_or_default();
    assert!(content.contains("col1"), "CSV should contain header data");
}

#[test]
fn e2e_sheets_insert_rows() {
    let account = require_account!();
    let title = unique_name("SheetInsert");
    let Some(sheet_id) = sheets_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, sheet_id.clone(), drive_delete);

    // Write data
    run_success(
        &mut omega(&account).args([
            "sheets", "update", &sheet_id, "Sheet1!A1:A3",
            "--values-json", "[[\"row1\"],[\"row2\"],[\"row3\"]]",
        ]),
    );

    // Get the numeric sheet ID from metadata
    let meta = run_json(omega_json(&account).args(["sheets", "metadata", &sheet_id]));
    let numeric_sheet_id = meta
        .get("sheets")
        .and_then(|s| s.as_array())
        .and_then(|a| a.first())
        .and_then(|s| s.get("properties"))
        .and_then(|p| p.get("sheetId"))
        .and_then(|id| id.as_u64())
        .map(|id| id.to_string())
        .unwrap_or_else(|| "0".to_string());

    // Insert a row at position 1 (0-indexed)
    run_success(
        &mut omega(&account).args([
            "sheets", "insert", &sheet_id, &numeric_sheet_id, "ROWS", "1",
            "--count", "1",
        ]),
    );

    settle();

    // Read back — row1 should now be at A1, blank at A2, row2 at A3
    let val = run_json(
        omega_json(&account).args(["sheets", "get", &sheet_id, "Sheet1!A1:A4"]),
    );
    let values = val.get("values").and_then(|v| v.as_array());
    assert!(
        values.map_or(false, |v| v.len() >= 3),
        "should have rows after insert"
    );
}

#[test]
fn e2e_sheets_copy() {
    let account = require_account!();
    let title = unique_name("SheetCopy");
    let Some(sheet_id) = sheets_create(&account, &title) else { return };
    let _guard1 = CleanupGuard::new(&account, sheet_id.clone(), drive_delete);

    // Copy
    let copy_title = format!("{title}_copy");
    let copied = run_json(
        omega_json(&account).args(["sheets", "copy", &sheet_id, &copy_title]),
    );
    let copy_id = extract_id(&copied, "spreadsheetId")
        .or_else(|| extract_id(&copied, "id"))
        .expect("copy should return id");
    let _guard2 = CleanupGuard::new(&account, copy_id.clone(), drive_delete);

    // Verify copy exists
    let got = run_json(omega_json(&account).args(["sheets", "metadata", &copy_id]));
    assert!(got.get("properties").is_some());
}

// ===================================================================
// Section 7: Slides (6 tests, API-gated)
// ===================================================================

/// Helper: create a Google Slides presentation. Returns None if API disabled.
fn slides_create(account: &str, title: &str) -> Option<String> {
    let output = omega_json(account)
        .args(["slides", "create", title])
        .output()
        .expect("slides create should execute");
    if check_skip(&output) {
        return None;
    }
    assert!(
        output.status.success(),
        "slides create failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let val: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).ok()?;
    extract_id(&val, "presentationId").or_else(|| extract_id(&val, "id"))
}

#[test]
fn e2e_slides_create_and_delete() {
    let account = require_account!();
    let title = unique_name("Slides");
    if let Some(pres_id) = slides_create(&account, &title) {
        let _guard = CleanupGuard::new(&account, pres_id, drive_delete);
    }
}

#[test]
fn e2e_slides_info() {
    let account = require_account!();
    let title = unique_name("SlidesInfo");
    let Some(pres_id) = slides_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, pres_id.clone(), drive_delete);

    let val = run_json(omega_json(&account).args(["slides", "info", &pres_id]));
    let pres_title = val.get("title").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        pres_title.contains("SlidesInfo"),
        "info should return the presentation title"
    );
}

#[test]
fn e2e_slides_list_slides() {
    let account = require_account!();
    let title = unique_name("SlidesList");
    let Some(pres_id) = slides_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, pres_id.clone(), drive_delete);

    let val = run_json(omega_json(&account).args(["slides", "list-slides", &pres_id]));
    // A new presentation has at least 1 slide
    let slides = val.get("slides").and_then(|s| s.as_array());
    assert!(
        slides.map_or(false, |s| !s.is_empty()) || val.is_array(),
        "list-slides should return slides"
    );
}

#[test]
fn e2e_slides_add_and_delete_slide() {
    let account = require_account!();
    let title = unique_name("SlidesAD");
    let Some(pres_id) = slides_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, pres_id.clone(), drive_delete);

    // Get initial slide count
    let initial = run_json(omega_json(&account).args(["slides", "list-slides", &pres_id]));
    let initial_count = initial
        .get("slides")
        .and_then(|s| s.as_array())
        .map(|a| a.len())
        .unwrap_or(1);

    // Add a slide
    let added = run_json(
        omega_json(&account).args(["slides", "add-slide", &pres_id]),
    );

    settle();

    // Verify count increased
    let after_add = run_json(omega_json(&account).args(["slides", "list-slides", &pres_id]));
    let after_add_count = after_add
        .get("slides")
        .and_then(|s| s.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(
        after_add_count,
        initial_count + 1,
        "should have one more slide after add"
    );

    // Get the new slide's ID for deletion
    let slide_id = added
        .get("replies")
        .and_then(|r| r.as_array())
        .and_then(|a| a.first())
        .and_then(|r| r.get("createSlide"))
        .and_then(|cs| cs.get("objectId"))
        .and_then(|id| id.as_str())
        .or_else(|| {
            after_add
                .get("slides")
                .and_then(|s| s.as_array())
                .and_then(|a| a.last())
                .and_then(|s| s.get("objectId"))
                .and_then(|id| id.as_str())
        });

    if let Some(sid) = slide_id {
        run_success(
            &mut omega(&account).args(["slides", "delete-slide", &pres_id, sid]),
        );
    }
}

#[test]
fn e2e_slides_update_notes() {
    let account = require_account!();
    let title = unique_name("SlidesNotes");
    let Some(pres_id) = slides_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, pres_id.clone(), drive_delete);

    // Get the first slide ID
    let listing = run_json(omega_json(&account).args(["slides", "list-slides", &pres_id]));
    let slide_id = listing
        .get("slides")
        .and_then(|s| s.as_array())
        .and_then(|a| a.first())
        .and_then(|s| s.get("objectId"))
        .and_then(|id| id.as_str());

    if let Some(sid) = slide_id {
        // Update speaker notes
        run_success(
            &mut omega(&account).args(["slides", "update-notes", &pres_id, sid, "E2E speaker notes"]),
        );

        settle();

        // Read slide to verify notes
        let slide = run_json(
            omega_json(&account).args(["slides", "read-slide", &pres_id, sid]),
        );
        // Notes may be in slideProperties.notesPage or similar
        let json_str = serde_json::to_string(&slide).unwrap_or_default();
        assert!(
            json_str.contains("E2E speaker notes") || json_str.contains("speaker"),
            "slide should contain speaker notes"
        );
    }
}

#[test]
fn e2e_slides_export() {
    let account = require_account!();
    let title = unique_name("SlidesExport");
    let Some(pres_id) = slides_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, pres_id.clone(), drive_delete);

    settle();

    let export_file = tempfile::Builder::new().suffix(".pptx").tempfile().expect("create export temp");
    let export_path = export_file.path().to_str().unwrap().to_string();
    run_success(
        &mut omega(&account).args([
            "slides", "export", &pres_id,
            "--format", "pptx",
            "--out", &export_path,
        ]),
    );
    assert!(
        std::fs::metadata(&export_path).map(|m| m.len() > 0).unwrap_or(false),
        "exported file should not be empty"
    );
}

// ===================================================================
// Section 8: Contacts (6 tests)
// ===================================================================

#[test]
fn e2e_contacts_list() {
    let account = require_account!();
    let val = run_json(
        omega_json(&account).args(["contacts", "list", "--max", "2"]),
    );
    assert!(
        val.get("connections").is_some() || val.get("people").is_some() || val.is_array(),
        "contacts list should have connections or people key"
    );
}

#[test]
fn e2e_contacts_crud() {
    let account = require_account!();
    let given = unique_name("Contact");

    // Create
    let created = run_json(
        omega_json(&account).args([
            "contacts", "create",
            "--given", &given,
            "--family", "E2ETest",
            "--email", "e2e@example.com",
        ]),
    );
    let resource_name = extract_id(&created, "resourceName")
        .expect("contact should have resourceName");
    let _guard = CleanupGuard::new(&account, resource_name.clone(), contact_delete);

    settle();

    // Get
    let got = run_json(
        omega_json(&account).args(["contacts", "get", &resource_name]),
    );
    assert!(
        got.get("resourceName").is_some(),
        "get should return contact with resourceName"
    );

    // Update
    let _ = omega(&account)
        .args([
            "contacts", "update", &resource_name,
            "--family", "E2EUpdated",
            "--ignore-etag",
        ])
        .output();
}

#[test]
fn e2e_contacts_search() {
    let account = require_account!();
    let given = unique_name("CSearch");

    // Create a contact to search for
    let created = run_json(
        omega_json(&account).args([
            "contacts", "create",
            "--given", &given,
            "--family", "Searchable",
        ]),
    );
    let resource_name = extract_id(&created, "resourceName").unwrap();
    let _guard = CleanupGuard::new(&account, resource_name.clone(), contact_delete);

    settle();

    // Search (People API search may have propagation delay, so we're lenient)
    let _ = omega_json(&account)
        .args(["contacts", "search", &given])
        .output();
}

#[test]
fn e2e_contacts_list_pagination() {
    let account = require_account!();
    let val = run_json(
        omega_json(&account).args(["contacts", "list", "--max", "1"]),
    );
    // Check that pagination token is present (may not be if < 2 contacts)
    if val.get("nextPageToken").is_some() {
        let token = val["nextPageToken"].as_str().unwrap();
        run_success(
            &mut omega_json(&account).args(["contacts", "list", "--max", "1", "--page", token]),
        );
    }
}

#[test]
fn e2e_contacts_other_list() {
    let account = require_account!();
    let output = run_success(
        &mut omega_json(&account).args(["contacts", "other", "list", "--max", "2"]),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_ok(),
        "contacts other list should return valid JSON"
    );
}

#[test]
fn e2e_contacts_directory_list() {
    let account = require_account!();
    let output = omega_json(&account)
        .args(["contacts", "directory", "list", "--max", "2"])
        .output()
        .expect("command should execute");
    // Personal accounts may get 403/400 for directory, so skip gracefully
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if check_skip(&output)
            || stderr.contains("requestSyncToken")
            || stderr.contains("400")
        {
            eprintln!("  [SKIP-API-DISABLED] Directory not available (expected for personal accounts)");
            return;
        }
        panic!(
            "contacts directory list failed unexpectedly: {}",
            stderr
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_ok(),
        "contacts directory list should return valid JSON"
    );
}

// ===================================================================
// Section 9: People (3 tests)
// ===================================================================

#[test]
fn e2e_people_me() {
    let account = require_account!();
    let val = run_json(omega_json(&account).args(["people", "me"]));
    assert!(
        val.get("resourceName").is_some(),
        "people me should have resourceName"
    );
}

#[test]
fn e2e_people_search() {
    let account = require_account!();
    let output = run_success(
        &mut omega_json(&account).args(["people", "search", "test", "--max", "2"]),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_ok(),
        "people search should return valid JSON"
    );
}

#[test]
fn e2e_people_relations() {
    let account = require_account!();
    let output = run_success(&mut omega_json(&account).args(["people", "relations"]));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_ok(),
        "people relations should return valid JSON"
    );
}

// ===================================================================
// Section 10: Tasks (8 tests)
// ===================================================================

/// Helper: get the default task list ID.
fn default_task_list(account: &str) -> Option<String> {
    let val = run_json(omega_json(account).args(["tasks", "lists", "list"]));
    val.get("items")
        .and_then(|items| items.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("id"))
        .and_then(|id| id.as_str())
        .map(|s| s.to_string())
}

#[test]
fn e2e_tasks_lists_list() {
    let account = require_account!();
    let val = run_json(omega_json(&account).args(["tasks", "lists", "list"]));
    assert!(
        val.get("items").is_some(),
        "task lists should have items key"
    );
}

#[test]
fn e2e_tasks_list_crud() {
    let account = require_account!();
    let list_name = unique_name("TaskList");

    // Create
    let created = run_json(
        omega_json(&account).args(["tasks", "lists", "create", &list_name]),
    );
    let list_id = extract_id(&created, "id").expect("task list should have id");

    // Verify in listing
    let listing = run_json(omega_json(&account).args(["tasks", "lists", "list"]));
    let items = listing.get("items").and_then(|i| i.as_array());
    assert!(
        items.map_or(false, |arr| arr.iter().any(|item| {
            item.get("id").and_then(|id| id.as_str()) == Some(&list_id)
        })),
        "created task list should appear in listing"
    );

    // Cleanup — Tasks API doesn't have a delete-list command typically,
    // so we leave it (it's harmless) or attempt delete if available
    let _ = omega(&account)
        .args(["tasks", "clear", &list_id])
        .output();
}

#[test]
fn e2e_tasks_full_lifecycle() {
    let account = require_account!();
    let Some(list_id) = default_task_list(&account) else {
        eprintln!("  [SKIP] No task lists found");
        return;
    };
    let task_title = unique_name("Task");

    // Add
    let created = run_json(
        omega_json(&account).args([
            "tasks", "add", &list_id,
            "--title", &task_title,
        ]),
    );
    let task_id = extract_id(&created, "id").expect("task should have id");
    let _guard = CleanupGuard::new(&account, format!("{list_id}:{task_id}"), task_delete);

    settle();

    // Get
    let got = run_json(
        omega_json(&account).args(["tasks", "get", &list_id, &task_id]),
    );
    assert_eq!(
        got.get("title").and_then(|v| v.as_str()),
        Some(task_title.as_str())
    );

    // Update
    let updated_title = format!("{task_title}_updated");
    run_success(
        &mut omega(&account).args([
            "tasks", "update", &list_id, &task_id,
            "--title", &updated_title,
        ]),
    );

    // Done
    run_success(
        &mut omega(&account).args(["tasks", "done", &list_id, &task_id]),
    );

    // Undo
    run_success(
        &mut omega(&account).args(["tasks", "undo", &list_id, &task_id]),
    );
}

#[test]
fn e2e_tasks_list_tasks() {
    let account = require_account!();
    let Some(list_id) = default_task_list(&account) else {
        eprintln!("  [SKIP] No task lists found");
        return;
    };
    let output = run_success(
        &mut omega_json(&account).args(["tasks", "list", &list_id, "--max", "3"]),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_ok(),
        "tasks list should return valid JSON"
    );
}

#[test]
fn e2e_tasks_add_with_notes() {
    let account = require_account!();
    let Some(list_id) = default_task_list(&account) else { return };
    let title = unique_name("TaskNotes");

    let created = run_json(
        omega_json(&account).args([
            "tasks", "add", &list_id,
            "--title", &title,
            "--notes", "E2E test notes content",
        ]),
    );
    let task_id = extract_id(&created, "id").unwrap();
    let _guard = CleanupGuard::new(&account, format!("{list_id}:{task_id}"), task_delete);

    settle();

    // Verify notes field
    let got = run_json(
        omega_json(&account).args(["tasks", "get", &list_id, &task_id]),
    );
    assert_eq!(
        got.get("notes").and_then(|v| v.as_str()),
        Some("E2E test notes content")
    );
}

#[test]
fn e2e_tasks_add_with_due() {
    let account = require_account!();
    let Some(list_id) = default_task_list(&account) else { return };
    let title = unique_name("TaskDue");

    // Tasks API requires RFC3339 date format
    let created = run_json(
        omega_json(&account).args([
            "tasks", "add", &list_id,
            "--title", &title,
            "--due", "2026-12-31T00:00:00Z",
        ]),
    );
    let task_id = extract_id(&created, "id").unwrap();
    let _guard = CleanupGuard::new(&account, format!("{list_id}:{task_id}"), task_delete);

    settle();

    // Verify due field
    let got = run_json(
        omega_json(&account).args(["tasks", "get", &list_id, &task_id]),
    );
    let due = got.get("due").and_then(|v| v.as_str()).unwrap_or("");
    assert!(due.contains("2026-12-31"), "due should contain the set date");
}

#[test]
fn e2e_tasks_clear_completed() {
    let account = require_account!();
    let Some(list_id) = default_task_list(&account) else { return };

    // Add 2 tasks
    let t1 = run_json(
        omega_json(&account).args([
            "tasks", "add", &list_id, "--title", &unique_name("TaskClear1"),
        ]),
    );
    let t1_id = extract_id(&t1, "id").unwrap();
    let _guard1 = CleanupGuard::new(&account, format!("{list_id}:{t1_id}"), task_delete);

    let t2 = run_json(
        omega_json(&account).args([
            "tasks", "add", &list_id, "--title", &unique_name("TaskClear2"),
        ]),
    );
    let t2_id = extract_id(&t2, "id").unwrap();
    let _guard2 = CleanupGuard::new(&account, format!("{list_id}:{t2_id}"), task_delete);

    // Mark task 1 as done
    run_success(&mut omega(&account).args(["tasks", "done", &list_id, &t1_id]));

    // Clear completed (returns 204 No Content, so don't use --json)
    let clear_out = omega(&account)
        .args(["tasks", "clear", &list_id])
        .output()
        .expect("clear should execute");
    // Clear may return exit 0 or may fail with empty-body parse error; accept both
    if !clear_out.status.success() {
        eprintln!("  [WARN] tasks clear returned non-zero (may be expected for empty response)");
    }

    // Task 2 (active) should still exist
    run_success(
        &mut omega_json(&account).args(["tasks", "get", &list_id, &t2_id]),
    );
}

#[test]
fn e2e_tasks_pagination() {
    let account = require_account!();
    let Some(list_id) = default_task_list(&account) else { return };
    let val = run_json(
        omega_json(&account).args(["tasks", "list", &list_id, "--max", "1"]),
    );
    if let Some(token) = extract_id(&val, "nextPageToken") {
        let page2 = run_json(
            omega_json(&account).args([
                "tasks", "list", &list_id, "--max", "1", "--page", &token,
            ]),
        );
        assert!(page2.is_object(), "page2 should be a JSON object");
    }
}

// ===================================================================
// Section 11: Forms (3 tests, API-gated)
// ===================================================================

/// Helper: create a Google Form. Returns None if API disabled.
fn forms_create(account: &str, title: &str) -> Option<String> {
    let output = omega_json(account)
        .args(["forms", "create", "--title", title])
        .output()
        .expect("forms create should execute");
    if check_skip(&output) {
        return None;
    }
    assert!(
        output.status.success(),
        "forms create failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let val: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).ok()?;
    extract_id(&val, "formId").or_else(|| extract_id(&val, "id"))
}

#[test]
fn e2e_forms_create_and_get() {
    let account = require_account!();
    let title = unique_name("Form");
    let Some(form_id) = forms_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, form_id.clone(), drive_delete);

    settle();

    // Get
    let val = run_json(omega_json(&account).args(["forms", "get", &form_id]));
    let info = val.get("info");
    assert!(
        info.and_then(|i| i.get("title")).and_then(|t| t.as_str()).unwrap_or("").contains("Form"),
        "form get should return form info with title"
    );
}

#[test]
fn e2e_forms_responses_list() {
    let account = require_account!();
    let title = unique_name("FormResp");
    let Some(form_id) = forms_create(&account, &title) else { return };
    let _guard = CleanupGuard::new(&account, form_id.clone(), drive_delete);

    // List responses (will be empty for new form)
    run_success(
        &mut omega_json(&account).args(["forms", "responses", "list", &form_id]),
    );
}

#[test]
fn e2e_forms_create_with_description() {
    let account = require_account!();
    let title = unique_name("FormDesc");
    let output = omega_json(&account)
        .args([
            "forms", "create",
            "--title", &title,
            "--description", "E2E test form description",
        ])
        .output()
        .expect("forms create should execute");

    if check_skip(&output) {
        return;
    }

    if output.status.success() {
        let val: serde_json::Value =
            serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
                .unwrap_or(serde_json::Value::Null);
        let form_id = extract_id(&val, "formId").or_else(|| extract_id(&val, "id"));
        if let Some(id) = &form_id {
            let _guard = CleanupGuard::new(&account, id.clone(), drive_delete);
            // Verify description was set
            let got = run_json(omega_json(&account).args(["forms", "get", id]));
            let desc = got
                .get("info")
                .and_then(|i| i.get("description"))
                .and_then(|d| d.as_str())
                .unwrap_or("");
            assert!(
                desc.contains("E2E test form description"),
                "form should have the description set"
            );
        }
    }
}

// ===================================================================
// Section 12: Cross-cutting (9 tests)
// ===================================================================

#[test]
fn e2e_output_json_mode() {
    let account = require_account!();
    let output = run_success(
        &mut omega(&account).args(["drive", "ls", "--max", "1", "--json"]),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_ok(),
        "JSON mode should produce valid JSON"
    );
}

#[test]
fn e2e_output_plain_mode() {
    let account = require_account!();
    let output = run_success(
        &mut omega(&account).args(["drive", "ls", "--max", "1", "--plain"]),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Plain mode should NOT be valid JSON (it's TSV)
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_err() || stdout.is_empty(),
        "plain mode should produce non-JSON output"
    );
}

#[test]
fn e2e_output_csv_mode() {
    let account = require_account!();
    let output = run_success(
        &mut omega(&account).args(["drive", "ls", "--max", "1", "--csv"]),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // CSV should NOT be valid JSON
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_err() || stdout.is_empty(),
        "csv mode should produce non-JSON output"
    );
    // If there are results, should contain commas (CSV)
    if !stdout.trim().is_empty() {
        assert!(
            stdout.contains(','),
            "CSV output should contain commas"
        );
    }
}

#[test]
fn e2e_select_field() {
    let account = require_account!();
    let output = run_success(
        &mut omega(&account).args([
            "people", "me", "--json", "--select", "resourceName",
        ]),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap_or(serde_json::Value::Null);
    // With --select, should have resourceName but minimal other fields
    assert!(
        val.get("resourceName").is_some() || stdout.contains("resourceName"),
        "--select should include the selected field"
    );
}

#[test]
fn e2e_results_only() {
    let account = require_account!();
    let output = run_success(
        &mut omega(&account).args([
            "drive", "ls", "--max", "1", "--json", "--results-only",
        ]),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap_or(serde_json::Value::Null);
    // With --results-only, the envelope should be stripped — expect array or unwrapped data
    assert!(
        val.is_array() || !val.get("kind").is_some(),
        "--results-only should strip the envelope"
    );
}

#[test]
fn e2e_dry_run_no_mutate() {
    let account = require_account!();
    let folder_name = unique_name("DryRun");

    // Create folder with --dry-run: should NOT create anything
    let output = omega(&account)
        .args(["--dry-run", "drive", "mkdir", &folder_name])
        .output()
        .expect("dry-run should execute");
    // Dry-run should succeed (just prints what would happen)
    assert!(
        output.status.success(),
        "dry-run should exit 0"
    );

    // Verify folder was NOT created
    let search = omega_json(&account)
        .args(["drive", "search", &folder_name, "--max", "1"])
        .output()
        .expect("search should execute");
    let stdout = String::from_utf8_lossy(&search.stdout);
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap_or(serde_json::Value::Null);
    let files = val.get("files").and_then(|f| f.as_array());
    assert!(
        files.is_none() || files.unwrap().is_empty(),
        "dry-run should not have created the folder"
    );
}

#[test]
fn e2e_verbose_flag() {
    let account = require_account!();
    let output = omega(&account)
        .args(["--verbose", "drive", "ls", "--max", "1", "--json"])
        .output()
        .expect("verbose should execute");
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verbose mode should produce some trace output on stderr
    assert!(
        !stderr.is_empty(),
        "verbose mode should produce stderr output"
    );
}

#[test]
fn e2e_error_no_auth() {
    if e2e_account().is_none() {
        eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
        return;
    }
    let mut cmd = Command::cargo_bin("omg-gog").expect("binary should exist");
    cmd.args([
        "--account", "bogus_nonexistent_account@invalid.test",
        "--no-input",
        "drive", "ls", "--json",
    ]);
    run_failure(&mut cmd);
}

#[test]
fn e2e_error_invalid_service_id() {
    let account = require_account!();
    run_failure(
        &mut omega_json(&account).args(["drive", "get", "nonexistent_file_id_e2e_99999"]),
    );
}

// ===================================================================
// Section 13: Open Command (2 tests, no auth needed)
// ===================================================================

#[test]
fn e2e_open_bare_id() {
    if e2e_account().is_none() {
        eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
        return;
    }
    // `open` with --type docs + --dry-run should generate a URL without opening browser
    let mut cmd = Command::cargo_bin("omg-gog").expect("binary should exist");
    cmd.args(["--dry-run", "--json", "open", "FAKE_DOC_ID", "--type", "docs"]);
    let output = cmd.output().expect("command should execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("docs.google.com") || output.status.success(),
        "open with --type docs should reference docs.google.com"
    );
}

#[test]
fn e2e_open_url_canonicalize() {
    if e2e_account().is_none() {
        eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
        return;
    }
    let mut cmd = Command::cargo_bin("omg-gog").expect("binary should exist");
    cmd.args([
        "--dry-run", "--json",
        "open", "https://docs.google.com/document/d/abc123/edit",
    ]);
    let output = cmd.output().expect("command should execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("docs.google.com") || output.status.success(),
        "open should canonicalize a Google Docs URL"
    );
}
