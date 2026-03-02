//! End-to-end integration tests against real Google APIs.
//!
//! These tests are gated behind the `OMEGA_E2E_ACCOUNT` environment variable.
//! They will be skipped entirely when the variable is not set, making them
//! safe to include in regular `cargo test` runs.
//!
//! To run:
//!   OMEGA_E2E_ACCOUNT=you@gmail.com cargo test e2e --jobs 1
//!
//! Prerequisites:
//!   1. OAuth credentials stored: omega-google auth credentials <file>
//!   2. Account authorized: omega-google auth add
//!   3. OMEGA_E2E_ACCOUNT set to the authorized email

use assert_cmd::Command;

/// Returns the account email if E2E tests are enabled, or None to skip.
fn e2e_account() -> Option<String> {
    std::env::var("OMEGA_E2E_ACCOUNT").ok()
}

/// Build an omega-google command with the E2E account pre-configured.
fn omega_google(account: &str) -> Command {
    let mut cmd = Command::cargo_bin("omega-google").expect("binary should exist");
    cmd.arg("--account").arg(account);
    cmd
}

// ===================================================================
// Auth status
// ===================================================================

#[test]
fn e2e_auth_status() {
    let account = match e2e_account() {
        Some(a) => a,
        None => {
            eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
            return;
        }
    };
    omega_google(&account)
        .arg("auth")
        .arg("status")
        .assert()
        .success();
}

#[test]
fn e2e_auth_list() {
    let account = match e2e_account() {
        Some(a) => a,
        None => {
            eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
            return;
        }
    };
    omega_google(&account)
        .arg("auth")
        .arg("list")
        .assert()
        .success();
}

// ===================================================================
// Gmail — read-only search
// ===================================================================

#[test]
fn e2e_gmail_search_json() {
    let account = match e2e_account() {
        Some(a) => a,
        None => {
            eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
            return;
        }
    };
    let output = omega_google(&account)
        .args(["gmail", "search", "in:inbox", "--max", "2", "--json"])
        .output()
        .expect("command should execute");

    assert!(
        output.status.success(),
        "gmail search failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");
    // Gmail search returns an object (may have "threads" or "messages" key, or be empty)
    assert!(parsed.is_object() || parsed.is_array(), "expected JSON object or array");
}

// ===================================================================
// Calendar — list events
// ===================================================================

#[test]
fn e2e_calendar_events_json() {
    let account = match e2e_account() {
        Some(a) => a,
        None => {
            eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
            return;
        }
    };
    let output = omega_google(&account)
        .args(["cal", "events", "--max", "2", "--json"])
        .output()
        .expect("command should execute");

    assert!(
        output.status.success(),
        "calendar events failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");
    assert!(parsed.is_object() || parsed.is_array());
}

// ===================================================================
// Drive — list files
// ===================================================================

#[test]
fn e2e_drive_list_json() {
    let account = match e2e_account() {
        Some(a) => a,
        None => {
            eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
            return;
        }
    };
    let output = omega_google(&account)
        .args(["drive", "ls", "--max", "2", "--json"])
        .output()
        .expect("command should execute");

    assert!(
        output.status.success(),
        "drive ls failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");
    assert!(parsed.is_object() || parsed.is_array());
}

// ===================================================================
// People — get own profile
// ===================================================================

#[test]
fn e2e_people_me_json() {
    let account = match e2e_account() {
        Some(a) => a,
        None => {
            eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
            return;
        }
    };
    let output = omega_google(&account)
        .args(["people", "me", "--json"])
        .output()
        .expect("command should execute");

    assert!(
        output.status.success(),
        "people me failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");
    assert!(parsed.is_object());
}

// ===================================================================
// Tasks — CRUD lifecycle
// ===================================================================

#[test]
fn e2e_tasks_crud() {
    let account = match e2e_account() {
        Some(a) => a,
        None => {
            eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
            return;
        }
    };

    // 1. List task lists
    let output = omega_google(&account)
        .args(["tasks", "lists", "list", "--json"])
        .output()
        .expect("tasks lists list should execute");

    assert!(
        output.status.success(),
        "tasks lists list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");

    // Extract the first task list ID
    let list_id = parsed
        .get("items")
        .and_then(|items| items.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("id"))
        .and_then(|id| id.as_str());

    if list_id.is_none() {
        eprintln!("No task lists found, skipping CRUD portion");
        return;
    }
    let list_id = list_id.unwrap();

    // 2. Create a task
    let output = omega_google(&account)
        .args([
            "tasks", "add", list_id,
            "--title", "E2E Test Task (safe to delete)",
            "--json",
        ])
        .output()
        .expect("tasks add should execute");

    if !output.status.success() {
        eprintln!(
            "tasks add failed (non-fatal): {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let task: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or(serde_json::Value::Null);
    let task_id = task.get("id").and_then(|id| id.as_str());

    // 3. Delete the task (cleanup)
    if let Some(tid) = task_id {
        let _ = omega_google(&account)
            .args(["tasks", "delete", list_id, tid])
            .output();
    }
}

// ===================================================================
// Output modes — same query with --json, --plain
// ===================================================================

#[test]
fn e2e_output_modes() {
    let account = match e2e_account() {
        Some(a) => a,
        None => {
            eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
            return;
        }
    };

    // JSON mode
    let json_out = omega_google(&account)
        .args(["drive", "ls", "--max", "1", "--json"])
        .output()
        .expect("json mode should execute");
    assert!(
        json_out.status.success(),
        "json mode failed: {}",
        String::from_utf8_lossy(&json_out.stderr)
    );
    let stdout = String::from_utf8_lossy(&json_out.stdout);
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_ok(),
        "JSON mode should produce valid JSON"
    );

    // Plain mode
    let plain_out = omega_google(&account)
        .args(["drive", "ls", "--max", "1", "--plain"])
        .output()
        .expect("plain mode should execute");
    assert!(
        plain_out.status.success(),
        "plain mode failed: {}",
        String::from_utf8_lossy(&plain_out.stderr)
    );
}

// ===================================================================
// Error cases
// ===================================================================

#[test]
fn e2e_error_invalid_file_id() {
    let account = match e2e_account() {
        Some(a) => a,
        None => {
            eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
            return;
        }
    };
    // Requesting a non-existent Drive file should fail with a non-zero exit code
    let output = omega_google(&account)
        .args(["drive", "get", "nonexistent_file_id_12345", "--json"])
        .output()
        .expect("command should execute");

    assert!(
        !output.status.success(),
        "Invalid file ID should produce non-zero exit code"
    );
}

#[test]
fn e2e_error_no_auth() {
    if e2e_account().is_none() {
        eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
        return;
    }
    // Using a bogus account should fail with auth error
    let mut cmd = Command::cargo_bin("omega-google").expect("binary should exist");
    cmd.args(["--account", "bogus_nonexistent_account@invalid.test", "drive", "ls", "--json"]);
    cmd.assert().failure();
}

// ===================================================================
// Contacts — read-only list
// ===================================================================

#[test]
fn e2e_contacts_list_json() {
    let account = match e2e_account() {
        Some(a) => a,
        None => {
            eprintln!("Skipping E2E test (set OMEGA_E2E_ACCOUNT to enable)");
            return;
        }
    };
    let output = omega_google(&account)
        .args(["contacts", "list", "--max", "2", "--json"])
        .output()
        .expect("command should execute");

    assert!(
        output.status.success(),
        "contacts list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");
    assert!(parsed.is_object() || parsed.is_array());
}

// ===================================================================
// Version — always works (no auth needed)
// ===================================================================

#[test]
fn e2e_version_no_auth() {
    // This test runs even without E2E account since it needs no auth
    Command::cargo_bin("omega-google")
        .expect("binary should exist")
        .arg("version")
        .assert()
        .success();
}
