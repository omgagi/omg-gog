//! Google Docs API request/response types.
//! All types use camelCase serde rename and flatten for forward compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------
// Document types
// ---------------------------------------------------------------

/// A Google Docs document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub document_id: Option<String>,
    pub title: Option<String>,
    pub body: Option<Body>,
    #[serde(default)]
    pub tabs: Vec<Tab>,
    pub revision_id: Option<String>,
    pub document_style: Option<serde_json::Value>,
    pub named_styles: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// The body of a document or tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    #[serde(default)]
    pub content: Vec<StructuralElement>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A structural element in the document body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralElement {
    pub start_index: Option<i64>,
    pub end_index: Option<i64>,
    pub paragraph: Option<Paragraph>,
    pub section_break: Option<serde_json::Value>,
    pub table: Option<Table>,
    pub table_of_contents: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A paragraph in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paragraph {
    #[serde(default)]
    pub elements: Vec<ParagraphElement>,
    pub paragraph_style: Option<ParagraphStyle>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// An element within a paragraph.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphElement {
    pub start_index: Option<i64>,
    pub end_index: Option<i64>,
    pub text_run: Option<TextRun>,
    pub inline_object_element: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A run of text with consistent styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRun {
    pub content: Option<String>,
    pub text_style: Option<TextStyle>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Text styling properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextStyle {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strikethrough: Option<bool>,
    pub font_size: Option<Dimension>,
    pub foreground_color: Option<serde_json::Value>,
    pub link: Option<Link>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A dimension with magnitude and unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dimension {
    pub magnitude: f64,
    pub unit: String,
}

/// A link within text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub url: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Tab types
// ---------------------------------------------------------------

/// A document tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub tab_properties: Option<TabProperties>,
    pub document_tab: Option<DocumentTab>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Properties of a tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabProperties {
    pub tab_id: Option<String>,
    pub title: Option<String>,
    pub index: Option<i32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// The content of a document tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentTab {
    pub body: Option<Body>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Comment types
// ---------------------------------------------------------------

/// A document comment (via Drive API).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: Option<String>,
    pub content: Option<String>,
    pub author: Option<Author>,
    pub created_time: Option<String>,
    pub modified_time: Option<String>,
    pub resolved: Option<bool>,
    #[serde(default)]
    pub replies: Vec<Reply>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A comment or reply author.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub display_name: Option<String>,
    pub email_address: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A reply to a comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reply {
    pub id: Option<String>,
    pub content: Option<String>,
    pub author: Option<Author>,
    pub created_time: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Batch update response types
// ---------------------------------------------------------------

/// Response for a replaceAllText operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceAllTextResponse {
    pub occurrences_changed: Option<i32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Table types
// ---------------------------------------------------------------

/// A table in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    pub rows: Option<i32>,
    pub columns: Option<i32>,
    #[serde(default)]
    pub table_rows: Vec<TableRow>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A row in a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRow {
    #[serde(default)]
    pub table_cells: Vec<TableCell>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A cell in a table row.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableCell {
    #[serde(default)]
    pub content: Vec<StructuralElement>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Paragraph style properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphStyle {
    pub named_style_type: Option<String>,
    pub heading_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-DOCS-002: Document metadata deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-002 (Must)
    // Acceptance: Document metadata deserializes from Docs API JSON
    #[test]
    fn req_docs_002_document_deserialize() {
        let json_str = r#"{
            "documentId": "doc123",
            "title": "My Document",
            "revisionId": "rev456",
            "body": {"content": []},
            "tabs": []
        }"#;
        let doc: Document = serde_json::from_str(json_str).unwrap();
        assert_eq!(doc.document_id, Some("doc123".to_string()));
        assert_eq!(doc.title, Some("My Document".to_string()));
        assert_eq!(doc.revision_id, Some("rev456".to_string()));
        assert!(doc.body.is_some());
    }

    // Requirement: REQ-DOCS-002 (Must)
    // Acceptance: Document round-trip serialization
    #[test]
    fn req_docs_002_document_roundtrip() {
        let doc = Document {
            document_id: Some("d1".to_string()),
            title: Some("Test".to_string()),
            body: Some(Body {
                content: vec![],
                extra: HashMap::new(),
            }),
            tabs: vec![],
            revision_id: Some("r1".to_string()),
            document_style: None,
            named_styles: None,
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&doc).unwrap();
        let parsed: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.document_id, Some("d1".to_string()));
        assert_eq!(parsed.title, Some("Test".to_string()));
    }

    // Requirement: REQ-DOCS-002 (Must)
    // Acceptance: Document with unknown fields preserved via flatten
    #[test]
    fn req_docs_002_document_unknown_fields() {
        let json_str = r#"{
            "documentId": "d1",
            "title": "Test",
            "body": {"content": []},
            "unknownField": "preserved"
        }"#;
        let doc: Document = serde_json::from_str(json_str).unwrap();
        assert!(doc.extra.contains_key("unknownField"));
    }

    // Requirement: REQ-DOCS-002 (Must)
    // Acceptance: Document with optional fields absent
    #[test]
    fn req_docs_002_document_minimal() {
        let json_str = r#"{"documentId": "d1"}"#;
        let doc: Document = serde_json::from_str(json_str).unwrap();
        assert_eq!(doc.document_id, Some("d1".to_string()));
        assert!(doc.title.is_none());
        assert!(doc.body.is_none());
        assert!(doc.tabs.is_empty());
        assert!(doc.revision_id.is_none());
    }

    // Requirement: REQ-DOCS-002 (Must)
    // Acceptance: Document with document_style and named_styles
    #[test]
    fn req_docs_002_document_with_styles() {
        let json_str = r#"{
            "documentId": "d1",
            "documentStyle": {"marginTop": {"magnitude": 72, "unit": "PT"}},
            "namedStyles": {"styles": []}
        }"#;
        let doc: Document = serde_json::from_str(json_str).unwrap();
        assert!(doc.document_style.is_some());
        assert!(doc.named_styles.is_some());
    }

    // ---------------------------------------------------------------
    // REQ-DOCS-005: Body/StructuralElement parsing, text extraction
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Body with structural elements deserializes
    #[test]
    fn req_docs_005_body_deserialize() {
        let json_str = r#"{
            "content": [
                {
                    "startIndex": 1,
                    "endIndex": 12,
                    "paragraph": {
                        "elements": [
                            {
                                "startIndex": 1,
                                "endIndex": 12,
                                "textRun": {
                                    "content": "Hello World",
                                    "textStyle": {}
                                }
                            }
                        ]
                    }
                }
            ]
        }"#;
        let body: Body = serde_json::from_str(json_str).unwrap();
        assert_eq!(body.content.len(), 1);
        let elem = &body.content[0];
        assert_eq!(elem.start_index, Some(1));
        assert_eq!(elem.end_index, Some(12));
        let para = elem.paragraph.as_ref().unwrap();
        assert_eq!(para.elements.len(), 1);
        let text_run = para.elements[0].text_run.as_ref().unwrap();
        assert_eq!(text_run.content, Some("Hello World".to_string()));
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: StructuralElement with section break
    #[test]
    fn req_docs_005_structural_element_section_break() {
        let json_str = r#"{
            "startIndex": 0,
            "endIndex": 1,
            "sectionBreak": {"sectionStyle": {}}
        }"#;
        let elem: StructuralElement = serde_json::from_str(json_str).unwrap();
        assert!(elem.section_break.is_some());
        assert!(elem.paragraph.is_none());
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Multiple paragraphs in body
    #[test]
    fn req_docs_005_multiple_paragraphs() {
        let json_str = r#"{
            "content": [
                {
                    "startIndex": 1,
                    "endIndex": 6,
                    "paragraph": {
                        "elements": [
                            {
                                "startIndex": 1,
                                "endIndex": 6,
                                "textRun": {"content": "Hello"}
                            }
                        ]
                    }
                },
                {
                    "startIndex": 6,
                    "endIndex": 12,
                    "paragraph": {
                        "elements": [
                            {
                                "startIndex": 6,
                                "endIndex": 12,
                                "textRun": {"content": "World!"}
                            }
                        ]
                    }
                }
            ]
        }"#;
        let body: Body = serde_json::from_str(json_str).unwrap();
        assert_eq!(body.content.len(), 2);
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: TextRun with full text style
    #[test]
    fn req_docs_005_text_run_with_style() {
        let json_str = r#"{
            "content": "Bold text",
            "textStyle": {
                "bold": true,
                "italic": false,
                "underline": false,
                "strikethrough": false,
                "fontSize": {"magnitude": 12.0, "unit": "PT"},
                "link": {"url": "https://example.com"}
            }
        }"#;
        let text_run: TextRun = serde_json::from_str(json_str).unwrap();
        assert_eq!(text_run.content, Some("Bold text".to_string()));
        let style = text_run.text_style.unwrap();
        assert_eq!(style.bold, Some(true));
        assert_eq!(style.italic, Some(false));
        let font_size = style.font_size.unwrap();
        assert_eq!(font_size.magnitude, 12.0);
        assert_eq!(font_size.unit, "PT");
        let link = style.link.unwrap();
        assert_eq!(link.url, Some("https://example.com".to_string()));
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: ParagraphElement with inline object
    #[test]
    fn req_docs_005_paragraph_element_inline_object() {
        let json_str = r#"{
            "startIndex": 5,
            "endIndex": 6,
            "inlineObjectElement": {"inlineObjectId": "obj123"}
        }"#;
        let elem: ParagraphElement = serde_json::from_str(json_str).unwrap();
        assert!(elem.inline_object_element.is_some());
        assert!(elem.text_run.is_none());
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Empty body content
    #[test]
    fn req_docs_005_empty_body() {
        let json_str = r#"{"content": []}"#;
        let body: Body = serde_json::from_str(json_str).unwrap();
        assert!(body.content.is_empty());
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Body with missing content defaults to empty
    #[test]
    fn req_docs_005_body_missing_content() {
        let json_str = r#"{}"#;
        let body: Body = serde_json::from_str(json_str).unwrap();
        assert!(body.content.is_empty());
    }

    // ---------------------------------------------------------------
    // REQ-DOCS-005: Table types
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Table with rows and cells deserializes
    #[test]
    fn req_docs_005_table_deserialize() {
        let json_str = r#"{
            "rows": 2,
            "columns": 2,
            "tableRows": [
                {
                    "tableCells": [
                        {"content": [{"startIndex": 1, "endIndex": 5, "paragraph": {"elements": [{"textRun": {"content": "Cell"}}]}}]},
                        {"content": []}
                    ]
                },
                {
                    "tableCells": [
                        {"content": []},
                        {"content": []}
                    ]
                }
            ]
        }"#;
        let table: Table = serde_json::from_str(json_str).unwrap();
        assert_eq!(table.rows, Some(2));
        assert_eq!(table.columns, Some(2));
        assert_eq!(table.table_rows.len(), 2);
        assert_eq!(table.table_rows[0].table_cells.len(), 2);
    }

    // ---------------------------------------------------------------
    // REQ-DOCS-006: Tab properties deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-006 (Should)
    // Acceptance: Tab with properties deserializes
    #[test]
    fn req_docs_006_tab_deserialize() {
        let json_str = r#"{
            "tabProperties": {
                "tabId": "tab1",
                "title": "Main Tab",
                "index": 0
            },
            "documentTab": {
                "body": {"content": []}
            }
        }"#;
        let tab: Tab = serde_json::from_str(json_str).unwrap();
        let props = tab.tab_properties.unwrap();
        assert_eq!(props.tab_id, Some("tab1".to_string()));
        assert_eq!(props.title, Some("Main Tab".to_string()));
        assert_eq!(props.index, Some(0));
        assert!(tab.document_tab.is_some());
    }

    // Requirement: REQ-DOCS-006 (Should)
    // Acceptance: Tab properties round-trip
    #[test]
    fn req_docs_006_tab_properties_roundtrip() {
        let props = TabProperties {
            tab_id: Some("t1".to_string()),
            title: Some("Tab Title".to_string()),
            index: Some(1),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&props).unwrap();
        let parsed: TabProperties = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.tab_id, Some("t1".to_string()));
        assert_eq!(parsed.title, Some("Tab Title".to_string()));
        assert_eq!(parsed.index, Some(1));
    }

    // Requirement: REQ-DOCS-006 (Should)
    // Acceptance: Multiple tabs in document
    #[test]
    fn req_docs_006_document_with_multiple_tabs() {
        let json_str = r#"{
            "documentId": "d1",
            "title": "Multi-tab",
            "tabs": [
                {"tabProperties": {"tabId": "t1", "title": "Tab 1", "index": 0}},
                {"tabProperties": {"tabId": "t2", "title": "Tab 2", "index": 1}}
            ]
        }"#;
        let doc: Document = serde_json::from_str(json_str).unwrap();
        assert_eq!(doc.tabs.len(), 2);
    }

    // ---------------------------------------------------------------
    // REQ-DOCS-007: Comment types deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: Comment deserializes from Drive API JSON
    #[test]
    fn req_docs_007_comment_deserialize() {
        let json_str = r#"{
            "id": "comment1",
            "content": "This needs review",
            "author": {
                "displayName": "Alice Smith",
                "emailAddress": "alice@example.com"
            },
            "createdTime": "2024-01-15T14:30:00.000Z",
            "modifiedTime": "2024-01-15T15:00:00.000Z",
            "resolved": false,
            "replies": []
        }"#;
        let comment: Comment = serde_json::from_str(json_str).unwrap();
        assert_eq!(comment.id, Some("comment1".to_string()));
        assert_eq!(comment.content, Some("This needs review".to_string()));
        assert_eq!(comment.resolved, Some(false));
        let author = comment.author.unwrap();
        assert_eq!(author.display_name, Some("Alice Smith".to_string()));
        assert_eq!(author.email_address, Some("alice@example.com".to_string()));
    }

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: Comment with replies
    #[test]
    fn req_docs_007_comment_with_replies() {
        let json_str = r#"{
            "id": "c1",
            "content": "Original comment",
            "replies": [
                {
                    "id": "r1",
                    "content": "Reply to comment",
                    "author": {"displayName": "Bob"},
                    "createdTime": "2024-01-16T10:00:00.000Z"
                },
                {
                    "id": "r2",
                    "content": "Another reply",
                    "createdTime": "2024-01-16T11:00:00.000Z"
                }
            ]
        }"#;
        let comment: Comment = serde_json::from_str(json_str).unwrap();
        assert_eq!(comment.replies.len(), 2);
        assert_eq!(comment.replies[0].id, Some("r1".to_string()));
        assert_eq!(
            comment.replies[0].content,
            Some("Reply to comment".to_string())
        );
    }

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: Comment round-trip serialization
    #[test]
    fn req_docs_007_comment_roundtrip() {
        let comment = Comment {
            id: Some("c1".to_string()),
            content: Some("Test comment".to_string()),
            author: Some(Author {
                display_name: Some("Test User".to_string()),
                email_address: Some("test@example.com".to_string()),
                extra: HashMap::new(),
            }),
            created_time: Some("2024-01-15T14:30:00.000Z".to_string()),
            modified_time: None,
            resolved: Some(false),
            replies: vec![],
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&comment).unwrap();
        let parsed: Comment = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, Some("c1".to_string()));
        assert_eq!(parsed.content, Some("Test comment".to_string()));
    }

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: ReplaceAllTextResponse deserializes
    #[test]
    fn req_docs_007_replace_all_text_response() {
        let json_str = r#"{"occurrencesChanged": 5}"#;
        let resp: ReplaceAllTextResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.occurrences_changed, Some(5));
    }

    // Requirement: REQ-DOCS-007 (Must)
    // Acceptance: Reply with unknown fields preserved
    #[test]
    fn req_docs_007_reply_unknown_fields() {
        let json_str = r#"{
            "id": "r1",
            "content": "test",
            "newField": "preserved"
        }"#;
        let reply: Reply = serde_json::from_str(json_str).unwrap();
        assert!(reply.extra.contains_key("newField"));
    }

    // ---------------------------------------------------------------
    // REQ-DOCS-005: ParagraphStyle
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: ParagraphStyle with heading
    #[test]
    fn req_docs_005_paragraph_style_heading() {
        let json_str = r#"{
            "namedStyleType": "HEADING_1",
            "headingId": "h.abc123"
        }"#;
        let style: ParagraphStyle = serde_json::from_str(json_str).unwrap();
        assert_eq!(style.named_style_type, Some("HEADING_1".to_string()));
        assert_eq!(style.heading_id, Some("h.abc123".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-DOCS-005: Dimension type
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Dimension deserializes
    #[test]
    fn req_docs_005_dimension_deserialize() {
        let json_str = r#"{"magnitude": 14.5, "unit": "PT"}"#;
        let dim: Dimension = serde_json::from_str(json_str).unwrap();
        assert_eq!(dim.magnitude, 14.5);
        assert_eq!(dim.unit, "PT");
    }

    // ---------------------------------------------------------------
    // REQ-DOCS-005: Link type
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Link deserializes
    #[test]
    fn req_docs_005_link_deserialize() {
        let json_str = r#"{"url": "https://example.com"}"#;
        let link: Link = serde_json::from_str(json_str).unwrap();
        assert_eq!(link.url, Some("https://example.com".to_string()));
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Link with unknown fields preserved
    #[test]
    fn req_docs_005_link_unknown_fields() {
        let json_str = r#"{"url": "https://example.com", "bookmarkId": "bm1"}"#;
        let link: Link = serde_json::from_str(json_str).unwrap();
        assert!(link.extra.contains_key("bookmarkId"));
    }
}
