//! Markdown conversion for Google Docs content.
//!
//! Converts a Docs API document body to Markdown format.

use super::types::{Body, StructuralElement, Paragraph, ParagraphElement, TextRun, ParagraphStyle};

/// Convert a document body to Markdown.
pub fn body_to_markdown(body: &Body) -> String {
    let mut md = String::new();
    for elem in &body.content {
        md.push_str(&structural_element_to_markdown(elem));
    }
    md
}

/// Convert a single structural element to Markdown.
fn structural_element_to_markdown(elem: &StructuralElement) -> String {
    if let Some(ref para) = elem.paragraph {
        return paragraph_to_markdown(para);
    }
    if let Some(ref table) = elem.table {
        let mut md = String::new();
        for (row_idx, row) in table.table_rows.iter().enumerate() {
            let mut cells = Vec::new();
            for cell in &row.table_cells {
                let cell_text: String = cell
                    .content
                    .iter()
                    .map(structural_element_to_markdown)
                    .collect::<String>()
                    .trim()
                    .to_string();
                cells.push(cell_text);
            }
            md.push_str("| ");
            md.push_str(&cells.join(" | "));
            md.push_str(" |\n");

            // After header row, emit separator
            if row_idx == 0 {
                md.push_str("| ");
                md.push_str(&cells.iter().map(|_| "---").collect::<Vec<_>>().join(" | "));
                md.push_str(" |\n");
            }
        }
        return md;
    }
    String::new()
}

/// Convert a paragraph to Markdown.
fn paragraph_to_markdown(para: &Paragraph) -> String {
    let mut text = String::new();

    // Determine heading prefix from paragraph style
    let prefix = match para.paragraph_style.as_ref() {
        Some(style) => heading_prefix(style),
        None => String::new(),
    };

    for pe in &para.elements {
        text.push_str(&paragraph_element_to_markdown(pe));
    }

    if !prefix.is_empty() && !text.trim().is_empty() {
        let trimmed = text.trim_end_matches('\n');
        format!("{}{}\n", prefix, trimmed)
    } else {
        text
    }
}

/// Get Markdown heading prefix from paragraph style.
fn heading_prefix(style: &ParagraphStyle) -> String {
    match style.named_style_type.as_deref() {
        Some("HEADING_1") => "# ".to_string(),
        Some("HEADING_2") => "## ".to_string(),
        Some("HEADING_3") => "### ".to_string(),
        Some("HEADING_4") => "#### ".to_string(),
        Some("HEADING_5") => "##### ".to_string(),
        Some("HEADING_6") => "###### ".to_string(),
        _ => String::new(),
    }
}

/// Convert a paragraph element to Markdown.
fn paragraph_element_to_markdown(pe: &ParagraphElement) -> String {
    if let Some(ref text_run) = pe.text_run {
        return text_run_to_markdown(text_run);
    }
    String::new()
}

/// Convert a text run to Markdown, applying bold/italic styling.
fn text_run_to_markdown(tr: &TextRun) -> String {
    let content = match &tr.content {
        Some(c) => c.clone(),
        None => return String::new(),
    };

    if content.trim().is_empty() {
        return content;
    }

    let style = match &tr.text_style {
        Some(s) => s,
        None => return content,
    };

    let bold = style.bold.unwrap_or(false);
    let italic = style.italic.unwrap_or(false);

    let mut result = content.clone();
    let trimmed = result.trim().to_string();
    let leading = &result[..result.len() - result.trim_start().len()];
    let trailing = &result[result.trim_end().len()..];

    if bold && italic {
        result = format!("{}***{}***{}", leading, trimmed, trailing);
    } else if bold {
        result = format!("{}**{}**{}", leading, trimmed, trailing);
    } else if italic {
        result = format!("{}*{}*{}", leading, trimmed, trailing);
    }

    // Handle links
    if let Some(ref link) = style.link {
        if let Some(ref url) = link.url {
            let display = result.trim().to_string();
            result = format!("[{}]({})", display, url);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::*;
    use std::collections::HashMap;

    fn make_text_run(text: &str, bold: bool, italic: bool) -> TextRun {
        TextRun {
            content: Some(text.to_string()),
            text_style: Some(TextStyle {
                bold: Some(bold),
                italic: Some(italic),
                underline: None,
                strikethrough: None,
                font_size: None,
                foreground_color: None,
                link: None,
                extra: HashMap::new(),
            }),
            extra: HashMap::new(),
        }
    }

    fn make_paragraph_element_styled(text: &str, bold: bool, italic: bool) -> ParagraphElement {
        ParagraphElement {
            start_index: None,
            end_index: None,
            text_run: Some(make_text_run(text, bold, italic)),
            inline_object_element: None,
            extra: HashMap::new(),
        }
    }

    fn make_heading_paragraph(text: &str, heading: &str) -> StructuralElement {
        StructuralElement {
            start_index: None,
            end_index: None,
            paragraph: Some(Paragraph {
                elements: vec![ParagraphElement {
                    start_index: None,
                    end_index: None,
                    text_run: Some(TextRun {
                        content: Some(text.to_string()),
                        text_style: None,
                        extra: HashMap::new(),
                    }),
                    inline_object_element: None,
                    extra: HashMap::new(),
                }],
                paragraph_style: Some(ParagraphStyle {
                    named_style_type: Some(heading.to_string()),
                    heading_id: None,
                    extra: HashMap::new(),
                }),
                extra: HashMap::new(),
            }),
            section_break: None,
            table: None,
            table_of_contents: None,
            extra: HashMap::new(),
        }
    }

    // REQ-DOCS-016
    #[test]
    fn req_docs_016_plain_text_to_markdown() {
        let body = Body {
            content: vec![StructuralElement {
                start_index: None,
                end_index: None,
                paragraph: Some(Paragraph {
                    elements: vec![ParagraphElement {
                        start_index: None,
                        end_index: None,
                        text_run: Some(TextRun {
                            content: Some("Hello World\n".to_string()),
                            text_style: None,
                            extra: HashMap::new(),
                        }),
                        inline_object_element: None,
                        extra: HashMap::new(),
                    }],
                    paragraph_style: None,
                    extra: HashMap::new(),
                }),
                section_break: None,
                table: None,
                table_of_contents: None,
                extra: HashMap::new(),
            }],
            extra: HashMap::new(),
        };
        let md = body_to_markdown(&body);
        assert_eq!(md, "Hello World\n");
    }

    // REQ-DOCS-016
    #[test]
    fn req_docs_016_heading_to_markdown() {
        let body = Body {
            content: vec![make_heading_paragraph("Title\n", "HEADING_1")],
            extra: HashMap::new(),
        };
        let md = body_to_markdown(&body);
        assert!(md.starts_with("# Title"));
    }

    // REQ-DOCS-016
    #[test]
    fn req_docs_016_heading_levels() {
        for (level, prefix) in [
            ("HEADING_1", "# "),
            ("HEADING_2", "## "),
            ("HEADING_3", "### "),
        ] {
            let body = Body {
                content: vec![make_heading_paragraph("Test", level)],
                extra: HashMap::new(),
            };
            let md = body_to_markdown(&body);
            assert!(md.starts_with(prefix), "Level {} should start with '{}'", level, prefix);
        }
    }

    // REQ-DOCS-016
    #[test]
    fn req_docs_016_bold_text() {
        let body = Body {
            content: vec![StructuralElement {
                start_index: None,
                end_index: None,
                paragraph: Some(Paragraph {
                    elements: vec![make_paragraph_element_styled("Bold text", true, false)],
                    paragraph_style: None,
                    extra: HashMap::new(),
                }),
                section_break: None,
                table: None,
                table_of_contents: None,
                extra: HashMap::new(),
            }],
            extra: HashMap::new(),
        };
        let md = body_to_markdown(&body);
        assert!(md.contains("**Bold text**"));
    }

    // REQ-DOCS-016
    #[test]
    fn req_docs_016_italic_text() {
        let body = Body {
            content: vec![StructuralElement {
                start_index: None,
                end_index: None,
                paragraph: Some(Paragraph {
                    elements: vec![make_paragraph_element_styled("Italic text", false, true)],
                    paragraph_style: None,
                    extra: HashMap::new(),
                }),
                section_break: None,
                table: None,
                table_of_contents: None,
                extra: HashMap::new(),
            }],
            extra: HashMap::new(),
        };
        let md = body_to_markdown(&body);
        assert!(md.contains("*Italic text*"));
    }

    // REQ-DOCS-016
    #[test]
    fn req_docs_016_empty_body() {
        let body = Body {
            content: vec![],
            extra: HashMap::new(),
        };
        let md = body_to_markdown(&body);
        assert!(md.is_empty());
    }

    // REQ-DOCS-016
    #[test]
    fn req_docs_016_table_separator_row() {
        let body = Body {
            content: vec![StructuralElement {
                start_index: None,
                end_index: None,
                paragraph: None,
                section_break: None,
                table: Some(Table {
                    rows: Some(2),
                    columns: Some(2),
                    table_rows: vec![
                        TableRow {
                            table_cells: vec![
                                TableCell {
                                    content: vec![StructuralElement {
                                        start_index: None,
                                        end_index: None,
                                        paragraph: Some(Paragraph {
                                            elements: vec![ParagraphElement {
                                                start_index: None,
                                                end_index: None,
                                                text_run: Some(TextRun {
                                                    content: Some("Name\n".to_string()),
                                                    text_style: None,
                                                    extra: HashMap::new(),
                                                }),
                                                inline_object_element: None,
                                                extra: HashMap::new(),
                                            }],
                                            paragraph_style: None,
                                            extra: HashMap::new(),
                                        }),
                                        section_break: None,
                                        table: None,
                                        table_of_contents: None,
                                        extra: HashMap::new(),
                                    }],
                                    extra: HashMap::new(),
                                },
                                TableCell {
                                    content: vec![StructuralElement {
                                        start_index: None,
                                        end_index: None,
                                        paragraph: Some(Paragraph {
                                            elements: vec![ParagraphElement {
                                                start_index: None,
                                                end_index: None,
                                                text_run: Some(TextRun {
                                                    content: Some("Value\n".to_string()),
                                                    text_style: None,
                                                    extra: HashMap::new(),
                                                }),
                                                inline_object_element: None,
                                                extra: HashMap::new(),
                                            }],
                                            paragraph_style: None,
                                            extra: HashMap::new(),
                                        }),
                                        section_break: None,
                                        table: None,
                                        table_of_contents: None,
                                        extra: HashMap::new(),
                                    }],
                                    extra: HashMap::new(),
                                },
                            ],
                            extra: HashMap::new(),
                        },
                        TableRow {
                            table_cells: vec![
                                TableCell {
                                    content: vec![StructuralElement {
                                        start_index: None,
                                        end_index: None,
                                        paragraph: Some(Paragraph {
                                            elements: vec![ParagraphElement {
                                                start_index: None,
                                                end_index: None,
                                                text_run: Some(TextRun {
                                                    content: Some("Alice\n".to_string()),
                                                    text_style: None,
                                                    extra: HashMap::new(),
                                                }),
                                                inline_object_element: None,
                                                extra: HashMap::new(),
                                            }],
                                            paragraph_style: None,
                                            extra: HashMap::new(),
                                        }),
                                        section_break: None,
                                        table: None,
                                        table_of_contents: None,
                                        extra: HashMap::new(),
                                    }],
                                    extra: HashMap::new(),
                                },
                                TableCell {
                                    content: vec![StructuralElement {
                                        start_index: None,
                                        end_index: None,
                                        paragraph: Some(Paragraph {
                                            elements: vec![ParagraphElement {
                                                start_index: None,
                                                end_index: None,
                                                text_run: Some(TextRun {
                                                    content: Some("42\n".to_string()),
                                                    text_style: None,
                                                    extra: HashMap::new(),
                                                }),
                                                inline_object_element: None,
                                                extra: HashMap::new(),
                                            }],
                                            paragraph_style: None,
                                            extra: HashMap::new(),
                                        }),
                                        section_break: None,
                                        table: None,
                                        table_of_contents: None,
                                        extra: HashMap::new(),
                                    }],
                                    extra: HashMap::new(),
                                },
                            ],
                            extra: HashMap::new(),
                        },
                    ],
                    extra: HashMap::new(),
                }),
                table_of_contents: None,
                extra: HashMap::new(),
            }],
            extra: HashMap::new(),
        };
        let md = body_to_markdown(&body);
        // Should have header row, separator, and data row
        let lines: Vec<&str> = md.lines().collect();
        assert_eq!(lines.len(), 3, "Expected 3 lines (header, separator, data), got: {:?}", lines);
        assert!(lines[0].contains("Name"));
        assert!(lines[1].contains("---"));
        assert!(lines[2].contains("Alice"));
    }
}
