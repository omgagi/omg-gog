//! Apps Script service integration tests.

use omega_google::services::appscript::scripts::*;
use omega_google::services::appscript::types::*;

// ---------------------------------------------------------------
// REQ-SCRIPT-001 (Must): Project deserialization from full JSON
// ---------------------------------------------------------------

// Requirement: REQ-SCRIPT-001 (Must)
// Acceptance: Full project structure from a realistic API response
#[test]
fn req_script_001_integration_full_project_from_api() {
    // REQ-SCRIPT-001
    let api_response = r#"{
        "scriptId": "1BxaNKVcfmBxQHLqm8MCl8RK6qlPnxzqHGCQ7aE3K-MUfvjJCYkXvJDn",
        "title": "My Automation Script",
        "parentId": "1abc123def456",
        "createTime": "2024-01-15T10:30:00Z",
        "updateTime": "2024-06-01T14:00:00Z",
        "creator": {"email": "user@example.com"},
        "lastModifyUser": {"email": "user@example.com"}
    }"#;

    let project: Project = serde_json::from_str(api_response).unwrap();

    assert_eq!(
        project.script_id,
        Some("1BxaNKVcfmBxQHLqm8MCl8RK6qlPnxzqHGCQ7aE3K-MUfvjJCYkXvJDn".to_string())
    );
    assert_eq!(project.title, Some("My Automation Script".to_string()));
    assert_eq!(project.parent_id, Some("1abc123def456".to_string()));
    assert_eq!(
        project.create_time,
        Some("2024-01-15T10:30:00Z".to_string())
    );
    assert_eq!(
        project.update_time,
        Some("2024-06-01T14:00:00Z".to_string())
    );

    // Unknown fields preserved via flatten
    assert!(project.extra.contains_key("creator"));
    assert!(project.extra.contains_key("lastModifyUser"));
}

// ---------------------------------------------------------------
// REQ-SCRIPT-002 (Must): Content deserialization from full JSON
// ---------------------------------------------------------------

// Requirement: REQ-SCRIPT-002 (Must)
// Acceptance: Full content structure from a realistic API response
#[test]
fn req_script_002_integration_full_content_from_api() {
    // REQ-SCRIPT-002
    let api_response = r#"{
        "scriptId": "abc123",
        "files": [
            {
                "name": "Code",
                "type": "SERVER_JS",
                "functionSet": {
                    "values": [
                        {"name": "onOpen"},
                        {"name": "processData"},
                        {"name": "sendReport"}
                    ]
                },
                "source": "function onOpen() {\n  var ui = SpreadsheetApp.getUi();\n  ui.createMenu('Custom').addItem('Run', 'processData').addToUi();\n}\n\nfunction processData() {\n  var sheet = SpreadsheetApp.getActive();\n  Logger.log('Processing...');\n}\n\nfunction sendReport() {\n  GmailApp.sendEmail('admin@example.com', 'Report', 'Done');\n}",
                "lastModifyUser": {"email": "dev@example.com"}
            },
            {
                "name": "Helpers",
                "type": "SERVER_JS",
                "functionSet": {
                    "values": [
                        {"name": "formatDate"},
                        {"name": "validateInput"}
                    ]
                },
                "source": "function formatDate(d) { return Utilities.formatDate(d, 'GMT', 'yyyy-MM-dd'); }\nfunction validateInput(x) { return x != null; }"
            },
            {
                "name": "appsscript",
                "type": "JSON",
                "source": "{\"timeZone\":\"America/New_York\",\"dependencies\":{},\"exceptionLogging\":\"STACKDRIVER\"}"
            }
        ]
    }"#;

    let content: Content = serde_json::from_str(api_response).unwrap();

    assert_eq!(content.script_id, Some("abc123".to_string()));
    assert_eq!(content.files.len(), 3);

    // First file: Code.gs
    let code = &content.files[0];
    assert_eq!(code.name, Some("Code".to_string()));
    assert_eq!(code.type_, Some("SERVER_JS".to_string()));
    assert!(code.source.as_ref().unwrap().contains("onOpen"));

    let func_set = code.function_set.as_ref().unwrap();
    assert_eq!(func_set.values.len(), 3);
    assert_eq!(func_set.values[0].name, Some("onOpen".to_string()));
    assert_eq!(func_set.values[1].name, Some("processData".to_string()));
    assert_eq!(func_set.values[2].name, Some("sendReport".to_string()));

    // Unknown fields preserved via flatten on ScriptFile
    assert!(code.extra.contains_key("lastModifyUser"));

    // Second file: Helpers
    let helpers = &content.files[1];
    assert_eq!(helpers.name, Some("Helpers".to_string()));
    let helper_funcs = helpers.function_set.as_ref().unwrap();
    assert_eq!(helper_funcs.values.len(), 2);

    // Third file: appsscript.json
    let manifest = &content.files[2];
    assert_eq!(manifest.type_, Some("JSON".to_string()));
    assert!(manifest.source.as_ref().unwrap().contains("timeZone"));
}

// ---------------------------------------------------------------
// REQ-SCRIPT-003 (Must): Execution types from realistic API
// ---------------------------------------------------------------

// Requirement: REQ-SCRIPT-003 (Must)
// Acceptance: Successful operation response
#[test]
fn req_script_003_integration_operation_success() {
    // REQ-SCRIPT-003
    let api_response = r#"{
        "done": true,
        "response": {
            "@type": "type.googleapis.com/google.apps.script.v1.ExecutionResponse",
            "result": {
                "status": "success",
                "processedRows": 150,
                "summary": "All data processed successfully"
            }
        }
    }"#;

    let op: Operation = serde_json::from_str(api_response).unwrap();
    assert_eq!(op.done, Some(true));
    assert!(op.error.is_none());

    let response = op.response.unwrap();
    assert_eq!(response["result"]["status"], "success");
    assert_eq!(response["result"]["processedRows"], 150);
}

// Requirement: REQ-SCRIPT-003 (Must)
// Acceptance: Failed operation response with error
#[test]
fn req_script_003_integration_operation_error() {
    // REQ-SCRIPT-003
    let api_response = r#"{
        "done": true,
        "error": {
            "errorMessage": "TypeError: Cannot read property 'getRange' of null",
            "errorType": "ScriptError",
            "scriptStackTraceElements": [
                {"function": "processData", "lineNumber": 15},
                {"function": "onOpen", "lineNumber": 3}
            ]
        }
    }"#;

    let op: Operation = serde_json::from_str(api_response).unwrap();
    assert_eq!(op.done, Some(true));
    assert!(op.response.is_none());

    let err = op.error.unwrap();
    assert!(err.error_message.as_ref().unwrap().contains("TypeError"));
    assert_eq!(err.error_type, Some("ScriptError".to_string()));
    assert_eq!(err.script_stack_trace_elements.len(), 2);
}

// ---------------------------------------------------------------
// REQ-SCRIPT-001 (Must): URL builder verification - project get
// ---------------------------------------------------------------

// Requirement: REQ-SCRIPT-001 (Must)
// Acceptance: Project get URL builds correctly
#[test]
fn req_script_001_integration_url_builder_project_get() {
    // REQ-SCRIPT-001
    let url = build_project_get_url("abc123def456");
    assert_eq!(
        url,
        "https://script.googleapis.com/v1/projects/abc123def456"
    );
}

// ---------------------------------------------------------------
// REQ-SCRIPT-002 (Must): URL builder verification - content get
// ---------------------------------------------------------------

// Requirement: REQ-SCRIPT-002 (Must)
// Acceptance: Content get URL builds correctly
#[test]
fn req_script_002_integration_url_builder_content_get() {
    // REQ-SCRIPT-002
    let url = build_content_get_url("abc123def456");
    assert_eq!(
        url,
        "https://script.googleapis.com/v1/projects/abc123def456/content"
    );
}

// ---------------------------------------------------------------
// REQ-SCRIPT-003 (Must): URL and body builder verification - run
// ---------------------------------------------------------------

// Requirement: REQ-SCRIPT-003 (Must)
// Acceptance: Run URL builds correctly
#[test]
fn req_script_003_integration_url_builder_run() {
    // REQ-SCRIPT-003
    let url = build_run_url("abc123def456");
    assert_eq!(
        url,
        "https://script.googleapis.com/v1/scripts/abc123def456:run"
    );
}

// Requirement: REQ-SCRIPT-003 (Must)
// Acceptance: Run body construction with various parameters
#[test]
fn req_script_003_integration_run_body_construction() {
    // REQ-SCRIPT-003
    // No params
    let body = build_run_body("processData", None, false).unwrap();
    assert_eq!(body["function"], "processData");
    assert_eq!(body["devMode"], false);

    // With array params in dev mode
    let body = build_run_body(
        "sendEmail",
        Some(r#"["user@example.com", "Subject", "Body"]"#),
        true,
    )
    .unwrap();
    assert_eq!(body["function"], "sendEmail");
    assert_eq!(body["devMode"], true);
    let params = body["parameters"].as_array().unwrap();
    assert_eq!(params.len(), 3);
    assert_eq!(params[0], "user@example.com");

    // Invalid JSON error
    let result = build_run_body("test", Some("not json"), false);
    assert!(result.is_err());
}

// ---------------------------------------------------------------
// REQ-SCRIPT-004 (Must): Project create URL and body
// ---------------------------------------------------------------

// Requirement: REQ-SCRIPT-004 (Must)
// Acceptance: Project create URL and body build correctly
#[test]
fn req_script_004_integration_project_create() {
    // REQ-SCRIPT-004
    let url = build_project_create_url();
    assert_eq!(url, "https://script.googleapis.com/v1/projects");

    // Without parent
    let body = build_project_create_body("My New Script", None);
    assert_eq!(body["title"], "My New Script");
    assert!(body.get("parentId").is_none());

    // With parent
    let body = build_project_create_body("Bound Script", Some("spreadsheet_id_123"));
    assert_eq!(body["title"], "Bound Script");
    assert_eq!(body["parentId"], "spreadsheet_id_123");
}

// ---------------------------------------------------------------
// REQ-SCRIPT-004 (Must): normalize_google_id
// ---------------------------------------------------------------

// Requirement: REQ-SCRIPT-004 (Must)
// Acceptance: normalize_google_id extracts IDs from various URL formats
#[test]
fn req_script_004_integration_normalize_google_id() {
    // REQ-SCRIPT-004
    // Bare ID
    assert_eq!(normalize_google_id("abc123def456"), "abc123def456");

    // Script editor URL
    assert_eq!(
        normalize_google_id("https://script.google.com/d/abc123def456/edit"),
        "abc123def456"
    );

    // Spreadsheet URL
    assert_eq!(
        normalize_google_id("https://docs.google.com/spreadsheets/d/SHEET_ID/edit#gid=0"),
        "SHEET_ID"
    );

    // Drive file URL
    assert_eq!(
        normalize_google_id("https://drive.google.com/file/d/FILE_ID/view"),
        "FILE_ID"
    );
}
