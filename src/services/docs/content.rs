//! Document content retrieval: URL builders and text extraction.

use super::types::{Body, StructuralElement, Tab};
use super::DOCS_BASE_URL;

/// Build URL for getting a document by ID.
pub fn build_doc_get_url(doc_id: &str) -> String {
    format!("{}/documents/{}", DOCS_BASE_URL, doc_id)
}

/// Build URL for getting a document with optional tabs content.
pub fn build_doc_get_url_with_tabs(doc_id: &str, include_tabs_content: bool) -> String {
    if include_tabs_content {
        format!(
            "{}/documents/{}?includeTabsContent=true",
            DOCS_BASE_URL, doc_id
        )
    } else {
        build_doc_get_url(doc_id)
    }
}

/// Extract plain text from a document body.
/// Walks the StructuralElement tree and collects all text_run content.
pub fn extract_plain_text(body: &Body) -> String {
    extract_plain_text_from_elements(&body.content)
}

/// Extract plain text from a slice of structural elements.
pub fn extract_plain_text_from_elements(elements: &[StructuralElement]) -> String {
    let mut text = String::new();
    for elem in elements {
        if let Some(ref para) = elem.paragraph {
            for pe in &para.elements {
                if let Some(ref text_run) = pe.text_run {
                    if let Some(ref content) = text_run.content {
                        text.push_str(content);
                    }
                }
            }
        }
        if let Some(ref table) = elem.table {
            for row in &table.table_rows {
                for cell in &row.table_cells {
                    text.push_str(&extract_plain_text_from_elements(&cell.content));
                }
            }
        }
    }
    text
}

/// Extract plain text from a tab.
/// Returns the text from the tab's document_tab body, or empty string if absent.
pub fn extract_tab_text(tab: &Tab) -> String {
    if let Some(ref doc_tab) = tab.document_tab {
        if let Some(ref body) = doc_tab.body {
            return extract_plain_text(body);
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::*;
    use std::collections::HashMap;

    // ---------------------------------------------------------------
    // REQ-DOCS-005: URL building
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Doc get URL includes document ID
    #[test]
    fn req_docs_005_build_doc_get_url() {
        let url = build_doc_get_url("doc123");
        assert_eq!(url, "https://docs.googleapis.com/v1/documents/doc123");
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Doc get URL with empty ID
    #[test]
    fn req_docs_005_build_doc_get_url_empty() {
        let url = build_doc_get_url("");
        assert_eq!(url, "https://docs.googleapis.com/v1/documents/");
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Doc get URL with tabs content
    #[test]
    fn req_docs_005_build_doc_get_url_with_tabs_true() {
        let url = build_doc_get_url_with_tabs("doc123", true);
        assert!(url.contains("documents/doc123"));
        assert!(url.contains("includeTabsContent=true"));
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Doc get URL without tabs content
    #[test]
    fn req_docs_005_build_doc_get_url_with_tabs_false() {
        let url = build_doc_get_url_with_tabs("doc123", false);
        assert!(url.contains("documents/doc123"));
        assert!(!url.contains("includeTabsContent"));
    }

    // ---------------------------------------------------------------
    // REQ-DOCS-005: Plain text extraction
    // ---------------------------------------------------------------

    fn make_text_run(text: &str) -> TextRun {
        TextRun {
            content: Some(text.to_string()),
            text_style: None,
            extra: HashMap::new(),
        }
    }

    fn make_paragraph_element(text: &str) -> ParagraphElement {
        ParagraphElement {
            start_index: None,
            end_index: None,
            text_run: Some(make_text_run(text)),
            inline_object_element: None,
            extra: HashMap::new(),
        }
    }

    fn make_paragraph(texts: &[&str]) -> Paragraph {
        Paragraph {
            elements: texts.iter().map(|t| make_paragraph_element(t)).collect(),
            paragraph_style: None,
            extra: HashMap::new(),
        }
    }

    fn make_structural_element(texts: &[&str]) -> StructuralElement {
        StructuralElement {
            start_index: None,
            end_index: None,
            paragraph: Some(make_paragraph(texts)),
            section_break: None,
            table: None,
            table_of_contents: None,
            extra: HashMap::new(),
        }
    }

    fn make_body(elements: Vec<StructuralElement>) -> Body {
        Body {
            content: elements,
            extra: HashMap::new(),
        }
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Extract text from single paragraph
    #[test]
    fn req_docs_005_extract_single_paragraph() {
        let body = make_body(vec![make_structural_element(&["Hello World\n"])]);
        let text = extract_plain_text(&body);
        assert_eq!(text, "Hello World\n");
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Extract text from multiple paragraphs
    #[test]
    fn req_docs_005_extract_multiple_paragraphs() {
        let body = make_body(vec![
            make_structural_element(&["Hello "]),
            make_structural_element(&["World"]),
        ]);
        let text = extract_plain_text(&body);
        assert_eq!(text, "Hello World");
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Extract text from paragraph with multiple runs
    #[test]
    fn req_docs_005_extract_multiple_runs() {
        let body = make_body(vec![make_structural_element(&["Hello ", "World", "!\n"])]);
        let text = extract_plain_text(&body);
        assert_eq!(text, "Hello World!\n");
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Empty body returns empty string
    #[test]
    fn req_docs_005_extract_empty_body() {
        let body = make_body(vec![]);
        let text = extract_plain_text(&body);
        assert_eq!(text, "");
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Structural element with no paragraph (e.g. section break) skipped
    #[test]
    fn req_docs_005_extract_skip_section_break() {
        let section_break = StructuralElement {
            start_index: None,
            end_index: None,
            paragraph: None,
            section_break: Some(serde_json::json!({})),
            table: None,
            table_of_contents: None,
            extra: HashMap::new(),
        };
        let body = make_body(vec![
            make_structural_element(&["Before\n"]),
            section_break,
            make_structural_element(&["After\n"]),
        ]);
        let text = extract_plain_text(&body);
        assert_eq!(text, "Before\nAfter\n");
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: ParagraphElement with no text_run (inline object) skipped
    #[test]
    fn req_docs_005_extract_skip_inline_object() {
        let inline_elem = ParagraphElement {
            start_index: None,
            end_index: None,
            text_run: None,
            inline_object_element: Some(serde_json::json!({"inlineObjectId": "obj1"})),
            extra: HashMap::new(),
        };
        let para = Paragraph {
            elements: vec![
                make_paragraph_element("Hello "),
                inline_elem,
                make_paragraph_element("World"),
            ],
            paragraph_style: None,
            extra: HashMap::new(),
        };
        let elem = StructuralElement {
            start_index: None,
            end_index: None,
            paragraph: Some(para),
            section_break: None,
            table: None,
            table_of_contents: None,
            extra: HashMap::new(),
        };
        let body = make_body(vec![elem]);
        let text = extract_plain_text(&body);
        assert_eq!(text, "Hello World");
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: TextRun with None content treated as empty
    #[test]
    fn req_docs_005_extract_none_content() {
        let text_run = TextRun {
            content: None,
            text_style: None,
            extra: HashMap::new(),
        };
        let pe = ParagraphElement {
            start_index: None,
            end_index: None,
            text_run: Some(text_run),
            inline_object_element: None,
            extra: HashMap::new(),
        };
        let para = Paragraph {
            elements: vec![pe],
            paragraph_style: None,
            extra: HashMap::new(),
        };
        let elem = StructuralElement {
            start_index: None,
            end_index: None,
            paragraph: Some(para),
            section_break: None,
            table: None,
            table_of_contents: None,
            extra: HashMap::new(),
        };
        let body = make_body(vec![elem]);
        let text = extract_plain_text(&body);
        assert_eq!(text, "");
    }

    // ---------------------------------------------------------------
    // REQ-DOCS-005: Table text extraction
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Extract text from table cells
    #[test]
    fn req_docs_005_extract_table_text() {
        let cell1 = TableCell {
            content: vec![make_structural_element(&["Cell 1\n"])],
            extra: HashMap::new(),
        };
        let cell2 = TableCell {
            content: vec![make_structural_element(&["Cell 2\n"])],
            extra: HashMap::new(),
        };
        let row = TableRow {
            table_cells: vec![cell1, cell2],
            extra: HashMap::new(),
        };
        let table = Table {
            rows: Some(1),
            columns: Some(2),
            table_rows: vec![row],
            extra: HashMap::new(),
        };
        let elem = StructuralElement {
            start_index: None,
            end_index: None,
            paragraph: None,
            section_break: None,
            table: Some(table),
            table_of_contents: None,
            extra: HashMap::new(),
        };
        let body = make_body(vec![elem]);
        let text = extract_plain_text(&body);
        assert_eq!(text, "Cell 1\nCell 2\n");
    }

    // ---------------------------------------------------------------
    // REQ-DOCS-005: Tab text extraction
    // ---------------------------------------------------------------

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Extract text from tab
    #[test]
    fn req_docs_005_extract_tab_text() {
        let body = make_body(vec![make_structural_element(&["Tab content\n"])]);
        let tab = Tab {
            tab_properties: Some(TabProperties {
                tab_id: Some("t1".to_string()),
                title: Some("Tab 1".to_string()),
                index: Some(0),
                extra: HashMap::new(),
            }),
            document_tab: Some(DocumentTab {
                body: Some(body),
                extra: HashMap::new(),
            }),
            extra: HashMap::new(),
        };
        let text = extract_tab_text(&tab);
        assert_eq!(text, "Tab content\n");
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Tab with no document_tab returns empty
    #[test]
    fn req_docs_005_extract_tab_no_document_tab() {
        let tab = Tab {
            tab_properties: None,
            document_tab: None,
            extra: HashMap::new(),
        };
        let text = extract_tab_text(&tab);
        assert_eq!(text, "");
    }

    // Requirement: REQ-DOCS-005 (Must)
    // Acceptance: Tab with document_tab but no body returns empty
    #[test]
    fn req_docs_005_extract_tab_no_body() {
        let tab = Tab {
            tab_properties: None,
            document_tab: Some(DocumentTab {
                body: None,
                extra: HashMap::new(),
            }),
            extra: HashMap::new(),
        };
        let text = extract_tab_text(&tab);
        assert_eq!(text, "");
    }
}
