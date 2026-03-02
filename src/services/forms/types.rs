//! Google Forms API request/response types.
//! All types use camelCase serde rename and flatten for forward compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------
// Form types
// ---------------------------------------------------------------

/// A Google Form resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Form {
    pub form_id: Option<String>,
    pub info: Option<FormInfo>,
    pub settings: Option<serde_json::Value>,
    #[serde(default)]
    pub items: Vec<FormItem>,
    pub revision_id: Option<String>,
    pub responder_uri: Option<String>,
    pub linked_sheet_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Form metadata (title, description, document_title).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormInfo {
    pub title: Option<String>,
    pub description: Option<String>,
    pub document_title: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A form item (question, section, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormItem {
    pub item_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub question_item: Option<serde_json::Value>,
    pub question_group_item: Option<serde_json::Value>,
    pub page_break_item: Option<serde_json::Value>,
    pub text_item: Option<serde_json::Value>,
    pub image_item: Option<serde_json::Value>,
    pub video_item: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Response types
// ---------------------------------------------------------------

/// A form response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormResponse {
    pub response_id: Option<String>,
    pub create_time: Option<String>,
    pub last_submitted_time: Option<String>,
    pub respondent_email: Option<String>,
    pub total_score: Option<f64>,
    #[serde(default)]
    pub answers: HashMap<String, Answer>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// An answer to a form question.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Answer {
    pub question_id: Option<String>,
    pub text_answers: Option<TextAnswers>,
    pub grade: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Text answers for a question.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextAnswers {
    #[serde(default)]
    pub answers: Vec<TextAnswer>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A single text answer value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextAnswer {
    pub value: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response list from the Forms API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormResponseList {
    #[serde(default)]
    pub responses: Vec<FormResponse>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ-FORMS-001
    #[test]
    fn req_forms_001_form_deserialize() {
        let json_str = r#"{
            "formId": "form123",
            "info": {
                "title": "My Survey",
                "description": "A test survey",
                "documentTitle": "Survey Doc"
            },
            "revisionId": "rev_abc",
            "responderUri": "https://docs.google.com/forms/d/form123/viewform",
            "items": [
                {
                    "itemId": "item1",
                    "title": "What is your name?",
                    "questionItem": {
                        "question": {
                            "questionId": "q1",
                            "textQuestion": {}
                        }
                    }
                }
            ]
        }"#;
        let form: Form = serde_json::from_str(json_str).unwrap();
        assert_eq!(form.form_id, Some("form123".to_string()));
        let info = form.info.unwrap();
        assert_eq!(info.title, Some("My Survey".to_string()));
        assert_eq!(info.description, Some("A test survey".to_string()));
        assert_eq!(form.items.len(), 1);
        assert_eq!(form.items[0].title, Some("What is your name?".to_string()));
    }

    // REQ-FORMS-001
    #[test]
    fn req_forms_001_form_roundtrip() {
        let form = Form {
            form_id: Some("f1".to_string()),
            info: Some(FormInfo {
                title: Some("Test".to_string()),
                description: None,
                document_title: None,
                extra: HashMap::new(),
            }),
            settings: None,
            items: vec![],
            revision_id: None,
            responder_uri: None,
            linked_sheet_id: None,
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&form).unwrap();
        let parsed: Form = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.form_id, Some("f1".to_string()));
    }

    // REQ-FORMS-001
    #[test]
    fn req_forms_001_form_unknown_fields() {
        let json_str = r#"{
            "formId": "f1",
            "newFeatureField": "preserved"
        }"#;
        let form: Form = serde_json::from_str(json_str).unwrap();
        assert!(form.extra.contains_key("newFeatureField"));
    }

    // REQ-FORMS-001
    #[test]
    fn req_forms_001_form_minimal() {
        let json_str = r#"{"formId": "f1"}"#;
        let form: Form = serde_json::from_str(json_str).unwrap();
        assert_eq!(form.form_id, Some("f1".to_string()));
        assert!(form.info.is_none());
        assert!(form.items.is_empty());
    }

    // REQ-FORMS-003
    #[test]
    fn req_forms_003_form_response_deserialize() {
        let json_str = r#"{
            "responseId": "resp123",
            "createTime": "2024-01-15T14:30:00.000Z",
            "lastSubmittedTime": "2024-01-15T14:31:00.000Z",
            "respondentEmail": "user@example.com",
            "answers": {
                "q1": {
                    "questionId": "q1",
                    "textAnswers": {
                        "answers": [
                            {"value": "Alice"}
                        ]
                    }
                }
            }
        }"#;
        let resp: FormResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.response_id, Some("resp123".to_string()));
        assert_eq!(resp.respondent_email, Some("user@example.com".to_string()));
        let answer = resp.answers.get("q1").unwrap();
        assert_eq!(answer.question_id, Some("q1".to_string()));
        let text_answers = answer.text_answers.as_ref().unwrap();
        assert_eq!(text_answers.answers[0].value, Some("Alice".to_string()));
    }

    // REQ-FORMS-003
    #[test]
    fn req_forms_003_response_list_deserialize() {
        let json_str = r#"{
            "responses": [
                {
                    "responseId": "r1",
                    "createTime": "2024-01-15T10:00:00Z",
                    "answers": {}
                },
                {
                    "responseId": "r2",
                    "createTime": "2024-01-15T11:00:00Z",
                    "answers": {}
                }
            ],
            "nextPageToken": "page2"
        }"#;
        let list: FormResponseList = serde_json::from_str(json_str).unwrap();
        assert_eq!(list.responses.len(), 2);
        assert_eq!(list.next_page_token, Some("page2".to_string()));
    }

    // REQ-FORMS-003
    #[test]
    fn req_forms_003_response_list_empty() {
        let json_str = r#"{"responses": []}"#;
        let list: FormResponseList = serde_json::from_str(json_str).unwrap();
        assert!(list.responses.is_empty());
        assert!(list.next_page_token.is_none());
    }

    // REQ-FORMS-001
    #[test]
    fn req_forms_001_form_item_types() {
        let json_str = r#"{
            "itemId": "item1",
            "title": "Section Header",
            "pageBreakItem": {}
        }"#;
        let item: FormItem = serde_json::from_str(json_str).unwrap();
        assert_eq!(item.item_id, Some("item1".to_string()));
        assert!(item.page_break_item.is_some());
        assert!(item.question_item.is_none());
    }
}
