//! Docs service integration tests.

use omega_google::services::docs::content::*;
use omega_google::services::docs::markdown::*;
use omega_google::services::docs::sedmat::*;
use omega_google::services::docs::types::*;

// ---------------------------------------------------------------
// REQ-DOCS-002 (Must): Document deserialization from realistic API response
// ---------------------------------------------------------------

// Requirement: REQ-DOCS-002 (Must)
// Acceptance: Full document structure from a realistic Docs API response
#[test]
fn req_docs_002_integration_full_document_from_api() {
    let api_response = r#"{
        "documentId": "1a2b3c4d5e",
        "title": "Q1 2024 Strategy Document",
        "revisionId": "ALm37BWuZnVRhxYh8",
        "body": {
            "content": [
                {
                    "startIndex": 1,
                    "endIndex": 2,
                    "sectionBreak": {
                        "sectionStyle": {
                            "columnSeparatorStyle": "NONE",
                            "contentDirection": "LEFT_TO_RIGHT"
                        }
                    }
                },
                {
                    "startIndex": 2,
                    "endIndex": 25,
                    "paragraph": {
                        "elements": [
                            {
                                "startIndex": 2,
                                "endIndex": 25,
                                "textRun": {
                                    "content": "Strategic Objectives\n",
                                    "textStyle": {
                                        "bold": true,
                                        "fontSize": {"magnitude": 18.0, "unit": "PT"}
                                    }
                                }
                            }
                        ],
                        "paragraphStyle": {
                            "namedStyleType": "HEADING_1",
                            "headingId": "h.abc123"
                        }
                    }
                },
                {
                    "startIndex": 25,
                    "endIndex": 80,
                    "paragraph": {
                        "elements": [
                            {
                                "startIndex": 25,
                                "endIndex": 55,
                                "textRun": {
                                    "content": "Our primary goal is to ",
                                    "textStyle": {}
                                }
                            },
                            {
                                "startIndex": 55,
                                "endIndex": 80,
                                "textRun": {
                                    "content": "increase revenue by 20%.\n",
                                    "textStyle": {
                                        "bold": true,
                                        "italic": true
                                    }
                                }
                            }
                        ],
                        "paragraphStyle": {
                            "namedStyleType": "NORMAL_TEXT"
                        }
                    }
                }
            ]
        },
        "tabs": [
            {
                "tabProperties": {
                    "tabId": "t.0",
                    "title": "Main",
                    "index": 0
                },
                "documentTab": {
                    "body": {
                        "content": [
                            {
                                "startIndex": 1,
                                "endIndex": 15,
                                "paragraph": {
                                    "elements": [
                                        {
                                            "startIndex": 1,
                                            "endIndex": 15,
                                            "textRun": {
                                                "content": "Tab content.\n"
                                            }
                                        }
                                    ]
                                }
                            }
                        ]
                    }
                }
            }
        ],
        "documentStyle": {
            "marginTop": {"magnitude": 72, "unit": "PT"},
            "marginBottom": {"magnitude": 72, "unit": "PT"},
            "marginLeft": {"magnitude": 72, "unit": "PT"},
            "marginRight": {"magnitude": 72, "unit": "PT"}
        }
    }"#;

    let doc: Document = serde_json::from_str(api_response).unwrap();

    // Verify document metadata
    assert_eq!(doc.document_id, Some("1a2b3c4d5e".to_string()));
    assert_eq!(doc.title, Some("Q1 2024 Strategy Document".to_string()));
    assert_eq!(doc.revision_id, Some("ALm37BWuZnVRhxYh8".to_string()));

    // Verify body content
    let body = doc.body.as_ref().unwrap();
    assert_eq!(body.content.len(), 3);

    // First element is a section break
    assert!(body.content[0].section_break.is_some());
    assert!(body.content[0].paragraph.is_none());

    // Second element is the heading
    let heading = body.content[1].paragraph.as_ref().unwrap();
    assert_eq!(heading.elements.len(), 1);
    let tr = heading.elements[0].text_run.as_ref().unwrap();
    assert_eq!(tr.content, Some("Strategic Objectives\n".to_string()));
    let style = tr.text_style.as_ref().unwrap();
    assert_eq!(style.bold, Some(true));
    let ps = heading.paragraph_style.as_ref().unwrap();
    assert_eq!(ps.named_style_type, Some("HEADING_1".to_string()));

    // Third element has two text runs with different styling
    let para = body.content[2].paragraph.as_ref().unwrap();
    assert_eq!(para.elements.len(), 2);
    let run2 = para.elements[1].text_run.as_ref().unwrap();
    let run2_style = run2.text_style.as_ref().unwrap();
    assert_eq!(run2_style.bold, Some(true));
    assert_eq!(run2_style.italic, Some(true));

    // Verify tabs
    assert_eq!(doc.tabs.len(), 1);
    let tab = &doc.tabs[0];
    let tab_props = tab.tab_properties.as_ref().unwrap();
    assert_eq!(tab_props.tab_id, Some("t.0".to_string()));
    assert_eq!(tab_props.title, Some("Main".to_string()));

    // Verify document style
    assert!(doc.document_style.is_some());
}

// ---------------------------------------------------------------
// REQ-DOCS-005 (Must): Plain text extraction from real document structure
// ---------------------------------------------------------------

// Requirement: REQ-DOCS-005 (Must)
// Acceptance: Extract plain text from a realistic document body
#[test]
fn req_docs_005_integration_plain_text_from_api() {
    let body_json = r#"{
        "content": [
            {
                "startIndex": 1,
                "endIndex": 2,
                "sectionBreak": {
                    "sectionStyle": {}
                }
            },
            {
                "startIndex": 2,
                "endIndex": 15,
                "paragraph": {
                    "elements": [
                        {
                            "startIndex": 2,
                            "endIndex": 15,
                            "textRun": {
                                "content": "Hello World!\n",
                                "textStyle": {"bold": true}
                            }
                        }
                    ],
                    "paragraphStyle": {
                        "namedStyleType": "HEADING_1"
                    }
                }
            },
            {
                "startIndex": 15,
                "endIndex": 40,
                "paragraph": {
                    "elements": [
                        {
                            "startIndex": 15,
                            "endIndex": 25,
                            "textRun": {
                                "content": "This is a ",
                                "textStyle": {}
                            }
                        },
                        {
                            "startIndex": 25,
                            "endIndex": 34,
                            "textRun": {
                                "content": "paragraph",
                                "textStyle": {"italic": true}
                            }
                        },
                        {
                            "startIndex": 34,
                            "endIndex": 40,
                            "textRun": {
                                "content": " text.\n",
                                "textStyle": {}
                            }
                        }
                    ]
                }
            }
        ]
    }"#;

    let body: Body = serde_json::from_str(body_json).unwrap();
    let text = extract_plain_text(&body);

    // Section break contributes no text
    // Heading contributes "Hello World!\n"
    // Paragraph contributes "This is a paragraph text.\n"
    assert_eq!(text, "Hello World!\nThis is a paragraph text.\n");
}

// Requirement: REQ-DOCS-005 (Must)
// Acceptance: Tab text extraction
#[test]
fn req_docs_005_integration_tab_text_extraction() {
    let tab_json = r#"{
        "tabProperties": {
            "tabId": "tab_1",
            "title": "Notes",
            "index": 1
        },
        "documentTab": {
            "body": {
                "content": [
                    {
                        "startIndex": 1,
                        "endIndex": 20,
                        "paragraph": {
                            "elements": [
                                {
                                    "startIndex": 1,
                                    "endIndex": 20,
                                    "textRun": {
                                        "content": "Meeting notes here\n"
                                    }
                                }
                            ]
                        }
                    }
                ]
            }
        }
    }"#;

    let tab: Tab = serde_json::from_str(tab_json).unwrap();
    let text = extract_tab_text(&tab);
    assert_eq!(text, "Meeting notes here\n");
}

// ---------------------------------------------------------------
// REQ-DOCS-014 (Must): Sed expression end-to-end parsing
// ---------------------------------------------------------------

// Requirement: REQ-DOCS-014 (Must)
// Acceptance: Parse sed expression and verify all components
#[test]
fn req_docs_014_integration_sed_expression_e2e() {
    // Basic substitution
    let expr = parse_sed_expression("s/Hello World/Goodbye World/g").unwrap();
    assert_eq!(expr.find, "Hello World");
    assert_eq!(expr.replace, "Goodbye World");
    assert!(expr.global);
    assert!(!expr.case_insensitive);

    // Case-insensitive global replacement
    let expr2 = parse_sed_expression("s/TODO/DONE/gi").unwrap();
    assert_eq!(expr2.find, "TODO");
    assert_eq!(expr2.replace, "DONE");
    assert!(expr2.global);
    assert!(expr2.case_insensitive);

    // Custom delimiter with special characters in find
    let expr3 = parse_sed_expression("s|http://old.com|https://new.com|g").unwrap();
    assert_eq!(expr3.find, "http://old.com");
    assert_eq!(expr3.replace, "https://new.com");
    assert!(expr3.global);

    // Delete (empty replace)
    let expr4 = parse_sed_expression("s/REMOVE_ME//g").unwrap();
    assert_eq!(expr4.find, "REMOVE_ME");
    assert_eq!(expr4.replace, "");
    assert!(expr4.global);
}

// Requirement: REQ-DOCS-014 (Must)
// Acceptance: Parse multiple expressions from file content
#[test]
fn req_docs_014_integration_sed_file_parsing() {
    let file_content = r#"# Style corrections
s/colour/color/g
s/analyse/analyze/gi

# URL updates
s|http://old.example.com|https://new.example.com|g
s|http://staging.example.com|https://prod.example.com|g
"#;

    let exprs = parse_sed_file(file_content).unwrap();
    assert_eq!(exprs.len(), 4);

    assert_eq!(exprs[0].find, "colour");
    assert_eq!(exprs[0].replace, "color");
    assert!(exprs[0].global);
    assert!(!exprs[0].case_insensitive);

    assert_eq!(exprs[1].find, "analyse");
    assert_eq!(exprs[1].replace, "analyze");
    assert!(exprs[1].global);
    assert!(exprs[1].case_insensitive);

    assert_eq!(exprs[2].find, "http://old.example.com");
    assert_eq!(exprs[2].replace, "https://new.example.com");
    assert!(exprs[2].global);

    assert_eq!(exprs[3].find, "http://staging.example.com");
    assert_eq!(exprs[3].replace, "https://prod.example.com");
}

// ---------------------------------------------------------------
// REQ-DOCS-016 (Should): Markdown conversion integration test
// ---------------------------------------------------------------

// Requirement: REQ-DOCS-016 (Should)
// Acceptance: Convert a realistic Docs API body to Markdown
#[test]
fn req_docs_016_integration_markdown_conversion() {
    let body_json = r#"{
        "content": [
            {
                "startIndex": 1,
                "endIndex": 10,
                "paragraph": {
                    "elements": [
                        {
                            "startIndex": 1,
                            "endIndex": 10,
                            "textRun": {
                                "content": "My Title",
                                "textStyle": {}
                            }
                        }
                    ],
                    "paragraphStyle": {
                        "namedStyleType": "HEADING_1"
                    }
                }
            },
            {
                "startIndex": 10,
                "endIndex": 30,
                "paragraph": {
                    "elements": [
                        {
                            "startIndex": 10,
                            "endIndex": 25,
                            "textRun": {
                                "content": "Some normal text",
                                "textStyle": {}
                            }
                        },
                        {
                            "startIndex": 25,
                            "endIndex": 30,
                            "textRun": {
                                "content": " and ",
                                "textStyle": {}
                            }
                        }
                    ]
                }
            },
            {
                "startIndex": 30,
                "endIndex": 50,
                "paragraph": {
                    "elements": [
                        {
                            "startIndex": 30,
                            "endIndex": 40,
                            "textRun": {
                                "content": "bold text",
                                "textStyle": {
                                    "bold": true
                                }
                            }
                        },
                        {
                            "startIndex": 40,
                            "endIndex": 50,
                            "textRun": {
                                "content": " follows.\n",
                                "textStyle": {}
                            }
                        }
                    ]
                }
            },
            {
                "startIndex": 50,
                "endIndex": 65,
                "paragraph": {
                    "elements": [
                        {
                            "startIndex": 50,
                            "endIndex": 65,
                            "textRun": {
                                "content": "Sub-section",
                                "textStyle": {}
                            }
                        }
                    ],
                    "paragraphStyle": {
                        "namedStyleType": "HEADING_2"
                    }
                }
            }
        ]
    }"#;

    let body: Body = serde_json::from_str(body_json).unwrap();
    let md = body_to_markdown(&body);

    // Verify heading conversion
    assert!(md.contains("# My Title"), "Should contain H1 heading");
    assert!(md.contains("## Sub-section"), "Should contain H2 heading");
    // Verify bold text conversion
    assert!(md.contains("**bold text**"), "Should contain bold text");
    // Verify normal text preserved
    assert!(
        md.contains("Some normal text"),
        "Should contain normal text"
    );
}

// Requirement: REQ-DOCS-016 (Should)
// Acceptance: Markdown with mixed formatting
#[test]
fn req_docs_016_integration_markdown_bold_italic() {
    let body_json = r#"{
        "content": [
            {
                "paragraph": {
                    "elements": [
                        {
                            "textRun": {
                                "content": "emphasis",
                                "textStyle": {
                                    "bold": true,
                                    "italic": true
                                }
                            }
                        }
                    ]
                }
            }
        ]
    }"#;

    let body: Body = serde_json::from_str(body_json).unwrap();
    let md = body_to_markdown(&body);
    assert!(
        md.contains("***emphasis***"),
        "Should contain bold+italic (***) text, got: {}",
        md
    );
}

// ---------------------------------------------------------------
// REQ-DOCS-002 (Must): Comment round-trip with realistic data
// ---------------------------------------------------------------

// Requirement: REQ-DOCS-002 (Must)
// Acceptance: Comment list from realistic API response
#[test]
fn req_docs_002_integration_comments_from_api() {
    let api_response = r#"[
        {
            "id": "c_1001",
            "content": "Please review the introduction section for clarity.",
            "author": {
                "displayName": "Sarah Chen",
                "emailAddress": "sarah.chen@company.com"
            },
            "createdTime": "2024-02-15T09:30:00.000Z",
            "modifiedTime": "2024-02-15T10:00:00.000Z",
            "resolved": false,
            "replies": [
                {
                    "id": "r_2001",
                    "content": "I've updated the intro. Please take another look.",
                    "author": {
                        "displayName": "James Liu",
                        "emailAddress": "james.liu@company.com"
                    },
                    "createdTime": "2024-02-15T14:00:00.000Z"
                },
                {
                    "id": "r_2002",
                    "content": "Looks good now, thank you!",
                    "author": {
                        "displayName": "Sarah Chen",
                        "emailAddress": "sarah.chen@company.com"
                    },
                    "createdTime": "2024-02-15T15:30:00.000Z"
                }
            ]
        },
        {
            "id": "c_1002",
            "content": "The budget figures need updating for Q2.",
            "author": {
                "displayName": "Mark Johnson",
                "emailAddress": "mark.johnson@company.com"
            },
            "createdTime": "2024-02-16T11:00:00.000Z",
            "resolved": true,
            "replies": []
        }
    ]"#;

    let comments: Vec<Comment> = serde_json::from_str(api_response).unwrap();
    assert_eq!(comments.len(), 2);

    // First comment with replies
    assert_eq!(comments[0].id, Some("c_1001".to_string()));
    assert_eq!(comments[0].resolved, Some(false));
    assert_eq!(comments[0].replies.len(), 2);
    assert_eq!(
        comments[0].replies[0].content,
        Some("I've updated the intro. Please take another look.".to_string())
    );

    // Second comment resolved
    assert_eq!(comments[1].id, Some("c_1002".to_string()));
    assert_eq!(comments[1].resolved, Some(true));
    assert!(comments[1].replies.is_empty());
}

// ---------------------------------------------------------------
// REQ-DOCS-005 (Must): Table text extraction integration
// ---------------------------------------------------------------

// Requirement: REQ-DOCS-005 (Must)
// Acceptance: Text extracted from a table within a document body
#[test]
fn req_docs_005_integration_table_text_extraction() {
    let body_json = r#"{
        "content": [
            {
                "startIndex": 1,
                "endIndex": 100,
                "table": {
                    "rows": 2,
                    "columns": 2,
                    "tableRows": [
                        {
                            "tableCells": [
                                {
                                    "content": [
                                        {
                                            "paragraph": {
                                                "elements": [
                                                    {"textRun": {"content": "Name\n"}}
                                                ]
                                            }
                                        }
                                    ]
                                },
                                {
                                    "content": [
                                        {
                                            "paragraph": {
                                                "elements": [
                                                    {"textRun": {"content": "Value\n"}}
                                                ]
                                            }
                                        }
                                    ]
                                }
                            ]
                        },
                        {
                            "tableCells": [
                                {
                                    "content": [
                                        {
                                            "paragraph": {
                                                "elements": [
                                                    {"textRun": {"content": "Alice\n"}}
                                                ]
                                            }
                                        }
                                    ]
                                },
                                {
                                    "content": [
                                        {
                                            "paragraph": {
                                                "elements": [
                                                    {"textRun": {"content": "42\n"}}
                                                ]
                                            }
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            }
        ]
    }"#;

    let body: Body = serde_json::from_str(body_json).unwrap();
    let text = extract_plain_text(&body);
    assert!(text.contains("Name"));
    assert!(text.contains("Value"));
    assert!(text.contains("Alice"));
    assert!(text.contains("42"));
}
