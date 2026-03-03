//! Markdown-to-slides parser and Slides API request generator.
//!
//! Parses markdown text into structured slide content, then converts
//! that content into Google Slides API batchUpdate requests.
//!
//! Conventions:
//! - `---` is the slide separator
//! - `# Title` starts a new slide (title slide layout)
//! - `## Title` starts a new slide (section header / title-and-body layout)
//! - Notes section: `<!-- notes: ... -->` or lines after `Notes:` at end of slide

/// Parsed content for a single slide.
#[derive(Debug, Clone, PartialEq)]
pub struct SlideContent {
    pub title: String,
    pub body: String,
    pub speaker_notes: Option<String>,
    pub layout: SlideLayout,
}

/// Layout type for a slide, determined by markdown heading level.
#[derive(Debug, Clone, PartialEq)]
pub enum SlideLayout {
    /// `# Title` -- large title slide, typically the first slide
    TitleSlide,
    /// `## Title` -- section header with body content
    TitleAndBody,
    /// No heading -- blank/content slide
    Blank,
}

/// Parse markdown text into a vector of slide content structures.
///
/// # Slide separation
/// Slides are separated by:
/// - `---` on its own line (with optional surrounding whitespace)
/// - `# Heading` or `## Heading` at the start of a line (starts a new slide)
///
/// # Title detection
/// - `# Title` -> `SlideLayout::TitleSlide`
/// - `## Title` -> `SlideLayout::TitleAndBody`
/// - No heading -> `SlideLayout::Blank`
///
/// # Speaker notes
/// Notes can be specified in two ways:
/// - HTML comment: `<!-- notes: your notes here -->`
/// - `Notes:` marker: all lines after a standalone `Notes:` line
pub fn parse_markdown_to_slides(markdown: &str) -> Vec<SlideContent> {
    if markdown.trim().is_empty() {
        return vec![];
    }

    // First split on --- separators
    let segments = split_on_separator(markdown);

    let mut all_slides = Vec::new();

    for segment in &segments {
        let trimmed = segment.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Within each segment, split on # headings
        let sub_slides = split_on_headings(trimmed);
        all_slides.extend(sub_slides);
    }

    all_slides
}

/// Split markdown text on `---` separators, returning raw text segments.
fn split_on_separator(markdown: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();

    for line in markdown.lines() {
        if line.trim() == "---" {
            segments.push(current);
            current = String::new();
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
    }
    segments.push(current);

    segments
}

/// Split a segment on `# ` or `## ` headings, producing one slide per heading.
/// Content before the first heading becomes a blank slide.
fn split_on_headings(segment: &str) -> Vec<SlideContent> {
    let mut slides: Vec<SlideContent> = Vec::new();
    let mut current_title = String::new();
    let mut current_layout = SlideLayout::Blank;
    let mut current_body_lines: Vec<String> = Vec::new();
    let mut has_heading = false;

    for line in segment.lines() {
        let trimmed = line.trim();

        // Check if this line is a heading (# or ##)
        if (trimmed.starts_with("# ") && !trimmed.starts_with("## ")) || trimmed.starts_with("## ")
        {
            // If we already have accumulated content, push it as a slide
            if has_heading || !current_body_lines.is_empty() {
                slides.push(finalize_slide(
                    current_title,
                    current_body_lines,
                    current_layout,
                ));
            }

            // Start a new slide with this heading
            if let Some(rest) = trimmed.strip_prefix("## ") {
                current_title = rest.trim().to_string();
                current_layout = SlideLayout::TitleAndBody;
            } else if let Some(rest) = trimmed.strip_prefix("# ") {
                current_title = rest.trim().to_string();
                current_layout = SlideLayout::TitleSlide;
            } else {
                unreachable!("outer guard ensures line starts with # or ##");
            }
            current_body_lines = Vec::new();
            has_heading = true;
        } else {
            current_body_lines.push(line.to_string());
        }
    }

    // Push the last accumulated slide
    if has_heading || !current_body_lines.is_empty() {
        slides.push(finalize_slide(
            current_title,
            current_body_lines,
            current_layout,
        ));
    }

    slides
}

/// Finalize a slide by processing body lines, extracting notes, and trimming.
fn finalize_slide(title: String, body_lines: Vec<String>, layout: SlideLayout) -> SlideContent {
    let raw_body = body_lines.join("\n");

    // Extract HTML comment notes
    let (content, html_notes) = extract_html_notes(&raw_body);

    // Extract Notes: marker notes
    let (final_body, marker_notes) = extract_marker_notes(&content);

    let notes = html_notes.or(marker_notes);

    // Trim leading/trailing blank lines from body
    let trimmed_body = trim_blank_lines(&final_body);

    SlideContent {
        title,
        body: trimmed_body,
        speaker_notes: notes,
        layout,
    }
}

/// Trim leading and trailing blank lines from text.
fn trim_blank_lines(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|l| !l.trim().is_empty())
        .unwrap_or(lines.len());
    let end = lines
        .iter()
        .rposition(|l| !l.trim().is_empty())
        .map(|i| i + 1)
        .unwrap_or(start);
    if start >= end {
        return String::new();
    }
    lines[start..end].join("\n")
}

/// Extract HTML-comment notes from content.
/// Returns (content_without_notes, extracted_notes).
fn extract_html_notes(content: &str) -> (String, Option<String>) {
    let notes_start = "<!-- notes:";
    let notes_end = "-->";

    if let Some(start_idx) = content.find(notes_start) {
        if let Some(end_rel) = content[start_idx..].find(notes_end) {
            let end_idx = start_idx + end_rel + notes_end.len();
            let notes_raw = &content[start_idx + notes_start.len()..start_idx + end_rel];
            let notes_text = notes_raw.trim().to_string();

            let mut cleaned = String::new();
            cleaned.push_str(&content[..start_idx]);
            cleaned.push_str(&content[end_idx..]);

            let notes = if notes_text.is_empty() {
                None
            } else {
                Some(notes_text)
            };

            return (cleaned, notes);
        }
    }

    (content.to_string(), None)
}

/// Extract Notes: marker section from content.
/// Returns (content_before_notes, extracted_notes).
fn extract_marker_notes(content: &str) -> (String, Option<String>) {
    let mut before = Vec::new();
    let mut notes_lines = Vec::new();
    let mut in_notes = false;

    for line in content.lines() {
        if !in_notes && line.trim() == "Notes:" {
            in_notes = true;
            continue;
        }
        if in_notes {
            notes_lines.push(line.to_string());
        } else {
            before.push(line.to_string());
        }
    }

    if notes_lines.is_empty() {
        (content.to_string(), None)
    } else {
        let joined = notes_lines.join("\n").trim().to_string();
        let notes = if joined.is_empty() {
            None
        } else {
            Some(joined)
        };
        (before.join("\n"), notes)
    }
}

/// Convert parsed slide content into a Google Slides API batchUpdate request body.
///
/// Returns a JSON value with a `requests` array containing:
/// - `createSlide` requests for each slide
/// - `createShape` requests for title and body text boxes
/// - `insertText` requests for title and body text
pub fn build_slides_from_markdown(slides: &[SlideContent]) -> serde_json::Value {
    let requests = slides_to_requests(slides);
    serde_json::json!({ "requests": requests })
}

/// Convert parsed slide content into Google Slides API batchUpdate requests.
///
/// For each slide, generates:
/// 1. A `createSlide` request
/// 2. `createShape` + `insertText` requests for title and body placeholders
pub fn slides_to_requests(slides: &[SlideContent]) -> Vec<serde_json::Value> {
    let mut requests = Vec::new();

    for (i, slide) in slides.iter().enumerate() {
        let slide_id = format!("slide_{}", i);

        // Determine layout preset
        let layout_preset = match slide.layout {
            SlideLayout::TitleSlide => "TITLE",
            SlideLayout::TitleAndBody => "TITLE_AND_BODY",
            SlideLayout::Blank => "BLANK",
        };

        // Create the slide
        let mut create_slide = serde_json::json!({
            "createSlide": {
                "objectId": slide_id,
                "insertionIndex": i,
                "slideLayoutReference": {
                    "predefinedLayout": layout_preset
                }
            }
        });

        // Add placeholder ID mappings for title and body
        let mut placeholder_mappings = Vec::new();

        if !slide.title.is_empty() {
            let title_id = format!("{}_title", slide_id);
            placeholder_mappings.push(serde_json::json!({
                "layoutPlaceholder": {
                    "type": "TITLE",
                    "index": 0
                },
                "objectId": title_id
            }));
        }

        if !slide.body.is_empty() {
            let body_id = format!("{}_body", slide_id);
            placeholder_mappings.push(serde_json::json!({
                "layoutPlaceholder": {
                    "type": "BODY",
                    "index": 0
                },
                "objectId": body_id
            }));
        }

        if !placeholder_mappings.is_empty() {
            create_slide["createSlide"]["placeholderIdMappings"] =
                serde_json::json!(placeholder_mappings);
        }

        requests.push(create_slide);

        // Insert title text
        if !slide.title.is_empty() {
            let title_id = format!("{}_title", slide_id);
            requests.push(serde_json::json!({
                "insertText": {
                    "objectId": title_id,
                    "text": slide.title,
                    "insertionIndex": 0
                }
            }));
        }

        // Insert body text
        if !slide.body.is_empty() {
            let body_id = format!("{}_body", slide_id);
            requests.push(serde_json::json!({
                "insertText": {
                    "objectId": body_id,
                    "text": slide.body,
                    "insertionIndex": 0
                }
            }));
        }

        // Insert speaker notes
        if let Some(ref notes) = slide.speaker_notes {
            let notes_id = format!("{}_notes", slide_id);
            let notes_reqs = super::notes::build_update_notes_request(&notes_id, notes);
            requests.extend(notes_reqs);
        }
    }

    requests
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-SLIDES-004 (Should): Slide separation by ---
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: Single slide (no separator)
    #[test]
    fn req_slides_004_single_slide_no_separator() {
        let md = "# Welcome\nThis is the first slide";
        let slides = parse_markdown_to_slides(md);
        assert_eq!(slides.len(), 1);
        assert_eq!(slides[0].title, "Welcome");
    }

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: Two content slides separated by --- (separator is just a boundary)
    #[test]
    fn req_slides_004_two_slides_separator() {
        let md = "# Slide One\nContent 1\n---\n## Slide Two\nContent 2";
        let slides = parse_markdown_to_slides(md);
        // --- is a boundary only, no blank slide inserted
        assert_eq!(slides.len(), 2);
        assert_eq!(slides[0].title, "Slide One");
        assert_eq!(slides[1].title, "Slide Two");
    }

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: Multiple slides with separators
    #[test]
    fn req_slides_004_three_slides() {
        let md = "# First\n---\n## Second\nBody\n---\nJust text";
        let slides = parse_markdown_to_slides(md);
        // --- is only a boundary, 3 content slides
        assert_eq!(slides.len(), 3);
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-004 (Should): Title slide detection
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: # heading -> TitleSlide layout
    #[test]
    fn req_slides_004_title_slide_detection() {
        let md = "# Main Title";
        let slides = parse_markdown_to_slides(md);
        assert_eq!(slides.len(), 1);
        assert_eq!(slides[0].layout, SlideLayout::TitleSlide);
        assert_eq!(slides[0].title, "Main Title");
    }

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: ## heading -> TitleAndBody layout
    #[test]
    fn req_slides_004_title_and_body_detection() {
        let md = "## Section Title\nSome body text";
        let slides = parse_markdown_to_slides(md);
        assert_eq!(slides.len(), 1);
        assert_eq!(slides[0].layout, SlideLayout::TitleAndBody);
        assert_eq!(slides[0].title, "Section Title");
    }

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: No heading -> Blank layout
    #[test]
    fn req_slides_004_blank_layout() {
        let md = "Just some text without heading";
        let slides = parse_markdown_to_slides(md);
        assert_eq!(slides.len(), 1);
        assert_eq!(slides[0].layout, SlideLayout::Blank);
        assert!(slides[0].title.is_empty());
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-004 (Should): Body content parsing
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: Body lines captured after title
    #[test]
    fn req_slides_004_body_content() {
        let md = "## My Slide\n- Point 1\n- Point 2\n- Point 3";
        let slides = parse_markdown_to_slides(md);
        assert!(slides[0].body.contains("- Point 1"));
        assert!(slides[0].body.contains("- Point 2"));
        assert!(slides[0].body.contains("- Point 3"));
    }

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: Multi-paragraph body
    #[test]
    fn req_slides_004_multi_paragraph_body() {
        let md = "## Title\nParagraph one\n\nParagraph two";
        let slides = parse_markdown_to_slides(md);
        assert!(slides[0].body.contains("Paragraph one"));
        assert!(slides[0].body.contains("Paragraph two"));
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-004 (Should): Speaker notes extraction
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: HTML comment notes extracted
    #[test]
    fn req_slides_004_html_comment_notes() {
        let md = "## Slide\nContent\n<!-- notes: These are my notes -->";
        let slides = parse_markdown_to_slides(md);
        assert_eq!(slides.len(), 1);
        assert_eq!(
            slides[0].speaker_notes,
            Some("These are my notes".to_string())
        );
    }

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: Notes: marker extracts notes
    #[test]
    fn req_slides_004_notes_marker() {
        let md = "## Slide\nContent\nNotes:\nThese are speaker notes";
        let slides = parse_markdown_to_slides(md);
        assert_eq!(slides.len(), 1);
        assert_eq!(
            slides[0].speaker_notes,
            Some("These are speaker notes".to_string())
        );
    }

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: Multiline Notes: section
    #[test]
    fn req_slides_004_multiline_notes() {
        let md = "## Slide\nBody\nNotes:\nLine 1\nLine 2";
        let slides = parse_markdown_to_slides(md);
        assert!(slides[0].speaker_notes.is_some());
        let notes = slides[0].speaker_notes.as_ref().unwrap();
        assert!(notes.contains("Line 1"));
        assert!(notes.contains("Line 2"));
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-004 (Should): Layout detection
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: Correct layout for mixed slides
    #[test]
    fn req_slides_004_mixed_layouts() {
        let md = "# Title Slide\n---\n## Content Slide\nBody\n---\nBlank content";
        let slides = parse_markdown_to_slides(md);
        // No blank separator slides: [TitleSlide, TitleAndBody, Blank(content)]
        assert_eq!(slides.len(), 3);
        assert_eq!(slides[0].layout, SlideLayout::TitleSlide);
        assert_eq!(slides[1].layout, SlideLayout::TitleAndBody);
        assert_eq!(slides[2].layout, SlideLayout::Blank); // content without heading
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-004 (Should): Multi-slide parsing
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: Full presentation parsing
    #[test]
    fn req_slides_004_full_presentation() {
        let md = "\
# Welcome to My Talk\n\
---\n\
## Agenda\n\
- Topic 1\n\
- Topic 2\n\
- Topic 3\n\
---\n\
## Details\n\
Here are the details\n\
<!-- notes: Remember to explain this clearly -->\n\
---\n\
# Thank You";
        let slides = parse_markdown_to_slides(md);
        // 4 content slides, --- is only a boundary
        assert_eq!(slides.len(), 4);
        assert_eq!(slides[0].title, "Welcome to My Talk");
        assert_eq!(slides[0].layout, SlideLayout::TitleSlide);
        assert_eq!(slides[1].title, "Agenda");
        assert!(slides[1].body.contains("- Topic 1"));
        assert_eq!(slides[2].title, "Details");
        assert_eq!(
            slides[2].speaker_notes,
            Some("Remember to explain this clearly".to_string())
        );
        assert_eq!(slides[3].title, "Thank You");
        assert_eq!(slides[3].layout, SlideLayout::TitleSlide);
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-004 (Should): Request generation
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: Generates createSlide requests
    #[test]
    fn req_slides_004_request_generation_create() {
        let slides = vec![SlideContent {
            title: "Hello".to_string(),
            body: String::new(),
            speaker_notes: None,
            layout: SlideLayout::TitleSlide,
        }];
        let reqs = slides_to_requests(&slides);
        // Should have createSlide + insertText for title
        assert!(reqs.len() >= 2);
        assert!(reqs[0].get("createSlide").is_some());
        assert_eq!(
            reqs[0]["createSlide"]["slideLayoutReference"]["predefinedLayout"],
            "TITLE"
        );
    }

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: Generates insertText for body
    #[test]
    fn req_slides_004_request_generation_body() {
        let slides = vec![SlideContent {
            title: "Slide".to_string(),
            body: "Line 1\nLine 2".to_string(),
            speaker_notes: None,
            layout: SlideLayout::TitleAndBody,
        }];
        let reqs = slides_to_requests(&slides);
        // createSlide + insertText(title) + insertText(body)
        assert_eq!(reqs.len(), 3);

        // Check body insertText
        let body_req = &reqs[2];
        assert!(body_req.get("insertText").is_some());
        assert_eq!(body_req["insertText"]["text"], "Line 1\nLine 2");
    }

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: Multiple slides generate sequential requests
    #[test]
    fn req_slides_004_request_generation_multiple() {
        let slides = vec![
            SlideContent {
                title: "First".to_string(),
                body: String::new(),
                speaker_notes: None,
                layout: SlideLayout::TitleSlide,
            },
            SlideContent {
                title: "Second".to_string(),
                body: "Content".to_string(),
                speaker_notes: None,
                layout: SlideLayout::TitleAndBody,
            },
        ];
        let reqs = slides_to_requests(&slides);
        // Slide 1: createSlide + insertText(title) = 2
        // Slide 2: createSlide + insertText(title) + insertText(body) = 3
        assert_eq!(reqs.len(), 5);

        // First slide ID
        assert_eq!(reqs[0]["createSlide"]["objectId"], "slide_0");
        // Second slide ID
        assert_eq!(reqs[2]["createSlide"]["objectId"], "slide_1");
    }

    // Requirement: REQ-SLIDES-004 (Should)
    // Acceptance: Blank layout generates BLANK preset
    #[test]
    fn req_slides_004_request_blank_layout() {
        let slides = vec![SlideContent {
            title: String::new(),
            body: "Some text".to_string(),
            speaker_notes: None,
            layout: SlideLayout::Blank,
        }];
        let reqs = slides_to_requests(&slides);
        assert_eq!(
            reqs[0]["createSlide"]["slideLayoutReference"]["predefinedLayout"],
            "BLANK"
        );
    }

    // ---------------------------------------------------------------
    // Edge cases
    // ---------------------------------------------------------------

    // Edge case: Empty markdown
    #[test]
    fn edge_case_empty_markdown() {
        let slides = parse_markdown_to_slides("");
        assert!(slides.is_empty());
    }

    // Edge case: Whitespace only
    #[test]
    fn edge_case_whitespace_only() {
        let slides = parse_markdown_to_slides("   \n  \n  ");
        assert!(slides.is_empty());
    }

    // Edge case: Only title, no body
    #[test]
    fn edge_case_title_only() {
        let md = "# Just a Title";
        let slides = parse_markdown_to_slides(md);
        assert_eq!(slides.len(), 1);
        assert_eq!(slides[0].title, "Just a Title");
        assert!(slides[0].body.is_empty());
    }

    // Edge case: Only separator
    #[test]
    fn edge_case_only_separator() {
        let md = "---";
        let slides = parse_markdown_to_slides(md);
        // --- is only a boundary; both segments are empty, so no slides
        assert!(slides.is_empty());
    }

    // Edge case: Separator with content on both sides
    #[test]
    fn edge_case_separator_both_sides() {
        let md = "Content A\n---\nContent B";
        let slides = parse_markdown_to_slides(md);
        // [Content A, Content B] -- no blank separator slide
        assert_eq!(slides.len(), 2);
    }

    // Edge case: Empty HTML notes comment
    #[test]
    fn edge_case_empty_html_notes() {
        let md = "## Slide\nContent\n<!-- notes:  -->";
        let slides = parse_markdown_to_slides(md);
        assert!(slides[0].speaker_notes.is_none());
    }
}
