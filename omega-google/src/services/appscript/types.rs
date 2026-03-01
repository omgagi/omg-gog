//! Google Apps Script API request/response types.
//! All types use camelCase serde rename and flatten for forward compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------
// Project types
// ---------------------------------------------------------------

/// An Apps Script project.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub script_id: Option<String>,
    pub title: Option<String>,
    pub parent_id: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Content types
// ---------------------------------------------------------------

/// The content (source files) of an Apps Script project.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub script_id: Option<String>,
    #[serde(default)]
    pub files: Vec<ScriptFile>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A file within an Apps Script project.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptFile {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub function_set: Option<FunctionSet>,
    pub source: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A set of functions in a script file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionSet {
    #[serde(default)]
    pub values: Vec<Function>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A function within a script file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Function {
    pub name: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Execution types
// ---------------------------------------------------------------

/// The response from a script execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResponse {
    pub result: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// An error from a script execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionError {
    #[serde(default)]
    pub script_stack_trace_elements: Vec<serde_json::Value>,
    pub error_message: Option<String>,
    pub error_type: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// An operation from the API (wraps execution response or error).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub done: Option<bool>,
    pub response: Option<serde_json::Value>,
    pub error: Option<ExecutionError>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-SCRIPT-001 (Must): Project type deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-SCRIPT-001 (Must)
    // Acceptance: Project type deserializes from Apps Script API JSON
    #[test]
    fn req_script_001_project_deserialize() {
        // REQ-SCRIPT-001
        let json_str = r#"{
            "scriptId": "abc123def456",
            "title": "My Script",
            "parentId": "doc_parent_id",
            "createTime": "2024-01-15T10:30:00Z",
            "updateTime": "2024-03-20T14:00:00Z"
        }"#;
        let project: Project = serde_json::from_str(json_str).unwrap();
        assert_eq!(project.script_id, Some("abc123def456".to_string()));
        assert_eq!(project.title, Some("My Script".to_string()));
        assert_eq!(project.parent_id, Some("doc_parent_id".to_string()));
        assert_eq!(project.create_time, Some("2024-01-15T10:30:00Z".to_string()));
        assert_eq!(project.update_time, Some("2024-03-20T14:00:00Z".to_string()));
    }

    // Requirement: REQ-SCRIPT-001 (Must)
    // Edge case: Project with unknown fields preserved via flatten
    #[test]
    fn req_script_001_project_unknown_fields_preserved() {
        // REQ-SCRIPT-001
        let json_str = r#"{
            "scriptId": "xxx",
            "futureField": "some_value"
        }"#;
        let project: Project = serde_json::from_str(json_str).unwrap();
        assert_eq!(project.script_id, Some("xxx".to_string()));
        assert!(project.extra.contains_key("futureField"));
    }

    // Requirement: REQ-SCRIPT-001 (Must)
    // Acceptance: Project round-trip serialization
    #[test]
    fn req_script_001_project_roundtrip() {
        // REQ-SCRIPT-001
        let project = Project {
            script_id: Some("abc".to_string()),
            title: Some("Test".to_string()),
            parent_id: None,
            create_time: None,
            update_time: None,
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&project).unwrap();
        let parsed: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.script_id, Some("abc".to_string()));
        assert_eq!(parsed.title, Some("Test".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-SCRIPT-002 (Must): Content and ScriptFile type deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-SCRIPT-002 (Must)
    // Acceptance: Content type deserializes with files
    #[test]
    fn req_script_002_content_deserialize() {
        // REQ-SCRIPT-002
        let json_str = r#"{
            "scriptId": "abc123",
            "files": [
                {
                    "name": "Code",
                    "type": "SERVER_JS",
                    "functionSet": {
                        "values": [
                            {"name": "myFunction"},
                            {"name": "onOpen"}
                        ]
                    },
                    "source": "function myFunction() {\n  Logger.log('Hello');\n}"
                },
                {
                    "name": "appsscript",
                    "type": "JSON",
                    "source": "{\"timeZone\":\"America/New_York\"}"
                }
            ]
        }"#;
        let content: Content = serde_json::from_str(json_str).unwrap();
        assert_eq!(content.script_id, Some("abc123".to_string()));
        assert_eq!(content.files.len(), 2);

        let code_file = &content.files[0];
        assert_eq!(code_file.name, Some("Code".to_string()));
        assert_eq!(code_file.type_, Some("SERVER_JS".to_string()));
        assert!(code_file.source.as_ref().unwrap().contains("myFunction"));

        let func_set = code_file.function_set.as_ref().unwrap();
        assert_eq!(func_set.values.len(), 2);
        assert_eq!(func_set.values[0].name, Some("myFunction".to_string()));

        let json_file = &content.files[1];
        assert_eq!(json_file.type_, Some("JSON".to_string()));
    }

    // Requirement: REQ-SCRIPT-002 (Must)
    // Edge case: Content with empty files
    #[test]
    fn req_script_002_content_empty_files() {
        // REQ-SCRIPT-002
        let json_str = r#"{"scriptId": "abc"}"#;
        let content: Content = serde_json::from_str(json_str).unwrap();
        assert!(content.files.is_empty());
    }

    // Requirement: REQ-SCRIPT-002 (Must)
    // Acceptance: ScriptFile type rename works
    #[test]
    fn req_script_002_script_file_type_rename() {
        // REQ-SCRIPT-002
        let json_str = r#"{
            "name": "Code",
            "type": "SERVER_JS"
        }"#;
        let file: ScriptFile = serde_json::from_str(json_str).unwrap();
        assert_eq!(file.type_, Some("SERVER_JS".to_string()));

        // Round-trip: should serialize back with "type"
        let serialized = serde_json::to_value(&file).unwrap();
        assert_eq!(serialized["type"], "SERVER_JS");
    }

    // ---------------------------------------------------------------
    // REQ-SCRIPT-003 (Must): Execution types
    // ---------------------------------------------------------------

    // Requirement: REQ-SCRIPT-003 (Must)
    // Acceptance: ExecutionResponse deserializes
    #[test]
    fn req_script_003_execution_response_deserialize() {
        // REQ-SCRIPT-003
        let json_str = r#"{
            "result": {"key": "value", "count": 42}
        }"#;
        let resp: ExecutionResponse = serde_json::from_str(json_str).unwrap();
        let result = resp.result.unwrap();
        assert_eq!(result["key"], "value");
        assert_eq!(result["count"], 42);
    }

    // Requirement: REQ-SCRIPT-003 (Must)
    // Acceptance: ExecutionError deserializes
    #[test]
    fn req_script_003_execution_error_deserialize() {
        // REQ-SCRIPT-003
        let json_str = r#"{
            "scriptStackTraceElements": [
                {"function": "myFunction", "lineNumber": 5}
            ],
            "errorMessage": "ReferenceError: x is not defined",
            "errorType": "ScriptError"
        }"#;
        let err: ExecutionError = serde_json::from_str(json_str).unwrap();
        assert_eq!(err.error_message, Some("ReferenceError: x is not defined".to_string()));
        assert_eq!(err.error_type, Some("ScriptError".to_string()));
        assert_eq!(err.script_stack_trace_elements.len(), 1);
    }

    // Requirement: REQ-SCRIPT-003 (Must)
    // Acceptance: Operation deserializes with response
    #[test]
    fn req_script_003_operation_with_response() {
        // REQ-SCRIPT-003
        let json_str = r#"{
            "done": true,
            "response": {
                "@type": "type.googleapis.com/google.apps.script.v1.ExecutionResponse",
                "result": "Hello World"
            }
        }"#;
        let op: Operation = serde_json::from_str(json_str).unwrap();
        assert_eq!(op.done, Some(true));
        assert!(op.response.is_some());
        assert!(op.error.is_none());
    }

    // Requirement: REQ-SCRIPT-003 (Must)
    // Acceptance: Operation deserializes with error
    #[test]
    fn req_script_003_operation_with_error() {
        // REQ-SCRIPT-003
        let json_str = r#"{
            "done": true,
            "error": {
                "errorMessage": "Script error",
                "errorType": "ScriptError",
                "scriptStackTraceElements": []
            }
        }"#;
        let op: Operation = serde_json::from_str(json_str).unwrap();
        assert_eq!(op.done, Some(true));
        assert!(op.error.is_some());
        assert_eq!(op.error.unwrap().error_message, Some("Script error".to_string()));
    }
}
