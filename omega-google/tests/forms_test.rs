//! Forms service integration tests.

use omega_google::services::forms::types::*;
use std::collections::HashMap;

// ---------------------------------------------------------------
// REQ-FORMS-001 (Must): Form from realistic API response
// ---------------------------------------------------------------

// Requirement: REQ-FORMS-001 (Must)
// Acceptance: Full form structure from a realistic Forms API response
#[test]
fn req_forms_001_integration_form_from_api() {
    let api_response = r#"{
        "formId": "1FAIpQLSf_abc123def456",
        "info": {
            "title": "Customer Satisfaction Survey",
            "description": "Please take a moment to rate your experience with our service.",
            "documentTitle": "Customer Satisfaction Survey"
        },
        "settings": {
            "quizSettings": {
                "isQuiz": false
            }
        },
        "revisionId": "00000042",
        "responderUri": "https://docs.google.com/forms/d/e/1FAIpQLSf_abc123def456/viewform",
        "linkedSheetId": "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgVE2upms",
        "items": [
            {
                "itemId": "item_001",
                "title": "How would you rate our service?",
                "description": "1 = Poor, 5 = Excellent",
                "questionItem": {
                    "question": {
                        "questionId": "q_001",
                        "required": true,
                        "scaleQuestion": {
                            "low": 1,
                            "high": 5,
                            "lowLabel": "Poor",
                            "highLabel": "Excellent"
                        }
                    }
                }
            },
            {
                "itemId": "item_002",
                "title": "Additional Comments",
                "questionItem": {
                    "question": {
                        "questionId": "q_002",
                        "required": false,
                        "textQuestion": {
                            "paragraph": true
                        }
                    }
                }
            },
            {
                "itemId": "item_003",
                "title": "Contact Information",
                "pageBreakItem": {}
            },
            {
                "itemId": "item_004",
                "title": "Your Email",
                "questionItem": {
                    "question": {
                        "questionId": "q_003",
                        "required": true,
                        "textQuestion": {}
                    }
                }
            }
        ]
    }"#;

    let form: Form = serde_json::from_str(api_response).unwrap();

    // Verify form metadata
    assert_eq!(form.form_id, Some("1FAIpQLSf_abc123def456".to_string()));
    assert_eq!(form.revision_id, Some("00000042".to_string()));
    assert!(form.responder_uri.as_ref().unwrap().contains("viewform"));
    assert_eq!(form.linked_sheet_id, Some("1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgVE2upms".to_string()));

    // Verify form info
    let info = form.info.as_ref().unwrap();
    assert_eq!(info.title, Some("Customer Satisfaction Survey".to_string()));
    assert_eq!(info.description, Some("Please take a moment to rate your experience with our service.".to_string()));
    assert_eq!(info.document_title, Some("Customer Satisfaction Survey".to_string()));

    // Verify settings
    assert!(form.settings.is_some());

    // Verify items
    assert_eq!(form.items.len(), 4);

    // Scale question
    assert_eq!(form.items[0].item_id, Some("item_001".to_string()));
    assert_eq!(form.items[0].title, Some("How would you rate our service?".to_string()));
    assert!(form.items[0].question_item.is_some());

    // Text question
    assert_eq!(form.items[1].title, Some("Additional Comments".to_string()));

    // Page break
    assert_eq!(form.items[2].title, Some("Contact Information".to_string()));
    assert!(form.items[2].page_break_item.is_some());

    // Email question
    assert_eq!(form.items[3].title, Some("Your Email".to_string()));
}

// Requirement: REQ-FORMS-001 (Must)
// Acceptance: Minimal form deserializes
#[test]
fn req_forms_001_integration_minimal_form() {
    let api_response = r#"{
        "formId": "min_form_id",
        "info": {
            "title": "Quick Poll"
        },
        "items": []
    }"#;

    let form: Form = serde_json::from_str(api_response).unwrap();
    assert_eq!(form.form_id, Some("min_form_id".to_string()));
    let info = form.info.as_ref().unwrap();
    assert_eq!(info.title, Some("Quick Poll".to_string()));
    assert!(form.items.is_empty());
}

// ---------------------------------------------------------------
// REQ-FORMS-003 (Must): Form response list from API response
// ---------------------------------------------------------------

// Requirement: REQ-FORMS-003 (Must)
// Acceptance: Full form response list from a realistic API response
#[test]
fn req_forms_003_integration_response_list_from_api() {
    let api_response = r#"{
        "responses": [
            {
                "responseId": "ACYDBNhW_abc123",
                "createTime": "2024-02-15T09:30:00.000Z",
                "lastSubmittedTime": "2024-02-15T09:32:00.000Z",
                "respondentEmail": "alice@example.com",
                "answers": {
                    "q_001": {
                        "questionId": "q_001",
                        "textAnswers": {
                            "answers": [
                                {"value": "5"}
                            ]
                        }
                    },
                    "q_002": {
                        "questionId": "q_002",
                        "textAnswers": {
                            "answers": [
                                {"value": "Great service, very responsive support team!"}
                            ]
                        }
                    },
                    "q_003": {
                        "questionId": "q_003",
                        "textAnswers": {
                            "answers": [
                                {"value": "alice@example.com"}
                            ]
                        }
                    }
                }
            },
            {
                "responseId": "ACYDBNhW_def456",
                "createTime": "2024-02-15T14:00:00.000Z",
                "lastSubmittedTime": "2024-02-15T14:05:00.000Z",
                "respondentEmail": "bob@example.com",
                "totalScore": 8.0,
                "answers": {
                    "q_001": {
                        "questionId": "q_001",
                        "textAnswers": {
                            "answers": [
                                {"value": "3"}
                            ]
                        }
                    },
                    "q_003": {
                        "questionId": "q_003",
                        "textAnswers": {
                            "answers": [
                                {"value": "bob@example.com"}
                            ]
                        }
                    }
                }
            },
            {
                "responseId": "ACYDBNhW_ghi789",
                "createTime": "2024-02-16T10:00:00.000Z",
                "lastSubmittedTime": "2024-02-16T10:01:00.000Z",
                "answers": {
                    "q_001": {
                        "questionId": "q_001",
                        "textAnswers": {
                            "answers": [
                                {"value": "4"}
                            ]
                        }
                    }
                }
            }
        ],
        "nextPageToken": "page_token_abc"
    }"#;

    let list: FormResponseList = serde_json::from_str(api_response).unwrap();

    // Verify pagination
    assert_eq!(list.responses.len(), 3);
    assert_eq!(list.next_page_token, Some("page_token_abc".to_string()));

    // First response: complete
    let r1 = &list.responses[0];
    assert_eq!(r1.response_id, Some("ACYDBNhW_abc123".to_string()));
    assert_eq!(r1.respondent_email, Some("alice@example.com".to_string()));
    assert_eq!(r1.answers.len(), 3);

    let a1 = r1.answers.get("q_001").unwrap();
    assert_eq!(a1.question_id, Some("q_001".to_string()));
    let ta1 = a1.text_answers.as_ref().unwrap();
    assert_eq!(ta1.answers[0].value, Some("5".to_string()));

    let a2 = r1.answers.get("q_002").unwrap();
    let ta2 = a2.text_answers.as_ref().unwrap();
    assert!(ta2.answers[0].value.as_ref().unwrap().contains("Great service"));

    // Second response: has score, skipped q_002
    let r2 = &list.responses[1];
    assert_eq!(r2.respondent_email, Some("bob@example.com".to_string()));
    assert_eq!(r2.total_score, Some(8.0));
    assert_eq!(r2.answers.len(), 2);
    assert!(!r2.answers.contains_key("q_002"));

    // Third response: anonymous (no email)
    let r3 = &list.responses[2];
    assert!(r3.respondent_email.is_none());
    assert_eq!(r3.answers.len(), 1);
}

// Requirement: REQ-FORMS-003 (Must)
// Acceptance: Empty response list
#[test]
fn req_forms_003_integration_empty_response_list() {
    let api_response = r#"{"responses": []}"#;

    let list: FormResponseList = serde_json::from_str(api_response).unwrap();
    assert!(list.responses.is_empty());
    assert!(list.next_page_token.is_none());
}
