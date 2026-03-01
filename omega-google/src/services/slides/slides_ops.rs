//! Slide-level operations: add, delete, replace image, extract text/notes.
//! These produce batchUpdate request objects for the Slides API.

use super::types::{Page, PageElement, PageSize};

/// Build a createSlide batchUpdate request.
///
/// - `layout_id`: Optional layout object ID to use for the new slide.
/// - `insertion_index`: Optional zero-based index at which to insert the slide.
pub fn build_add_slide_request(
    layout_id: Option<&str>,
    insertion_index: Option<i32>,
) -> serde_json::Value {
    let mut create_slide = serde_json::Map::new();

    if let Some(idx) = insertion_index {
        create_slide.insert(
            "insertionIndex".to_string(),
            serde_json::json!(idx),
        );
    }

    if let Some(lid) = layout_id {
        create_slide.insert(
            "slideLayoutReference".to_string(),
            serde_json::json!({ "layoutId": lid }),
        );
    }

    serde_json::json!({
        "createSlide": create_slide
    })
}

/// Build a deleteObject batchUpdate request to remove a slide.
pub fn build_delete_slide_request(slide_id: &str) -> serde_json::Value {
    serde_json::json!({
        "deleteObject": {
            "objectId": slide_id
        }
    })
}

/// Build a createImage batchUpdate request for adding an image to a slide.
///
/// If `page_size` is provided, the image is sized to fill the entire slide
/// (full-bleed) with a transform positioning it at the origin.
pub fn build_replace_image_request(
    slide_id: &str,
    image_url: &str,
    page_size: Option<&PageSize>,
) -> serde_json::Value {
    let mut request = serde_json::json!({
        "createImage": {
            "url": image_url,
            "elementProperties": {
                "pageObjectId": slide_id
            }
        }
    });

    if let Some(ps) = page_size {
        let mut size_obj = serde_json::Map::new();
        if let Some(ref w) = ps.width {
            let mut width_obj = serde_json::Map::new();
            if let Some(mag) = w.magnitude {
                width_obj.insert("magnitude".to_string(), serde_json::json!(mag));
            }
            if let Some(ref unit) = w.unit {
                width_obj.insert("unit".to_string(), serde_json::json!(unit));
            }
            size_obj.insert("width".to_string(), serde_json::Value::Object(width_obj));
        }
        if let Some(ref h) = ps.height {
            let mut height_obj = serde_json::Map::new();
            if let Some(mag) = h.magnitude {
                height_obj.insert("magnitude".to_string(), serde_json::json!(mag));
            }
            if let Some(ref unit) = h.unit {
                height_obj.insert("unit".to_string(), serde_json::json!(unit));
            }
            size_obj.insert("height".to_string(), serde_json::Value::Object(height_obj));
        }

        let ep = request["createImage"]["elementProperties"]
            .as_object_mut()
            .unwrap();
        ep.insert("size".to_string(), serde_json::Value::Object(size_obj));

        // Set transform to position at origin (0,0) for full-bleed
        ep.insert(
            "transform".to_string(),
            serde_json::json!({
                "scaleX": 1.0,
                "scaleY": 1.0,
                "translateX": 0.0,
                "translateY": 0.0,
                "unit": "EMU"
            }),
        );
    }

    request
}

/// Extract all text content from a slice of page elements.
///
/// Iterates over all page elements, extracts text runs from shapes,
/// and concatenates them into a single string.
pub fn extract_slide_text(elements: &[PageElement]) -> String {
    let mut text = String::new();
    for elem in elements {
        if let Some(ref shape) = elem.shape {
            if let Some(ref tc) = shape.text {
                for te in &tc.text_elements {
                    if let Some(ref tr) = te.text_run {
                        if let Some(ref content) = tr.content {
                            text.push_str(content);
                        }
                    }
                }
            }
        }
    }
    text
}

/// Extract speaker notes text from a slide page.
///
/// Looks for the notes page via `slide_properties.notes_page` or the
/// top-level `notes_page` field, then extracts text from the shape
/// identified by `speaker_notes_object_id`.
pub fn extract_speaker_notes(page: &Page) -> Option<String> {
    // Try slide_properties.notes_page first, then top-level notes_page
    let notes_page = page
        .slide_properties
        .as_ref()
        .and_then(|sp| sp.notes_page.as_ref())
        .or_else(|| page.notes_page.as_ref());

    let notes_page = notes_page?;

    let speaker_notes_id = notes_page
        .notes_properties
        .as_ref()
        .and_then(|np| np.speaker_notes_object_id.as_deref());

    let mut text = String::new();
    for elem in &notes_page.page_elements {
        // If we have a speaker_notes_object_id, only extract from that element;
        // otherwise extract from all shapes on the notes page
        let matches = match speaker_notes_id {
            Some(id) => elem.object_id.as_deref() == Some(id),
            None => true,
        };

        if matches {
            if let Some(ref shape) = elem.shape {
                if let Some(ref tc) = shape.text {
                    for te in &tc.text_elements {
                        if let Some(ref tr) = te.text_run {
                            if let Some(ref content) = tr.content {
                                text.push_str(content);
                            }
                        }
                    }
                }
            }
        }
    }

    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::*;
    use std::collections::HashMap;

    // ---------------------------------------------------------------
    // REQ-SLIDES-007 (Must): Add slide request
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-007 (Must)
    // Acceptance: Add slide with no options
    #[test]
    fn req_slides_007_add_slide_default() {
        let req = build_add_slide_request(None, None);
        assert!(req.get("createSlide").is_some());
    }

    // Requirement: REQ-SLIDES-007 (Must)
    // Acceptance: Add slide with layout
    #[test]
    fn req_slides_007_add_slide_with_layout() {
        let req = build_add_slide_request(Some("layout_abc"), None);
        let cs = &req["createSlide"];
        assert_eq!(cs["slideLayoutReference"]["layoutId"], "layout_abc");
    }

    // Requirement: REQ-SLIDES-007 (Must)
    // Acceptance: Add slide at specific index
    #[test]
    fn req_slides_007_add_slide_at_index() {
        let req = build_add_slide_request(None, Some(2));
        let cs = &req["createSlide"];
        assert_eq!(cs["insertionIndex"], 2);
    }

    // Requirement: REQ-SLIDES-007 (Must)
    // Acceptance: Add slide with both layout and index
    #[test]
    fn req_slides_007_add_slide_full() {
        let req = build_add_slide_request(Some("layout_xyz"), Some(5));
        let cs = &req["createSlide"];
        assert_eq!(cs["slideLayoutReference"]["layoutId"], "layout_xyz");
        assert_eq!(cs["insertionIndex"], 5);
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-008 (Must): Delete slide request
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-008 (Must)
    // Acceptance: Delete request has correct object ID
    #[test]
    fn req_slides_008_delete_slide() {
        let req = build_delete_slide_request("slide_123");
        assert_eq!(req["deleteObject"]["objectId"], "slide_123");
    }

    // Requirement: REQ-SLIDES-008 (Must)
    // Edge case: Delete with different ID
    #[test]
    fn req_slides_008_delete_slide_different_id() {
        let req = build_delete_slide_request("g12345_p0");
        assert_eq!(req["deleteObject"]["objectId"], "g12345_p0");
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-009 (Must): Text extraction from slides
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-009 (Must)
    // Acceptance: Extract text from page with shapes
    #[test]
    fn req_slides_009_extract_text() {
        let page = Page {
            object_id: Some("slide1".to_string()),
            page_type: Some("SLIDE".to_string()),
            page_elements: vec![PageElement {
                object_id: Some("shape1".to_string()),
                size: None,
                transform: None,
                title: None,
                description: None,
                shape: Some(Shape {
                    shape_type: Some("TEXT_BOX".to_string()),
                    text: Some(TextContent {
                        text_elements: vec![
                            TextElement {
                                start_index: Some(0),
                                end_index: Some(6),
                                paragraph_marker: None,
                                text_run: Some(TextRun {
                                    content: Some("Hello ".to_string()),
                                    style: None,
                                    extra: HashMap::new(),
                                }),
                                extra: HashMap::new(),
                            },
                            TextElement {
                                start_index: Some(6),
                                end_index: Some(11),
                                paragraph_marker: None,
                                text_run: Some(TextRun {
                                    content: Some("World".to_string()),
                                    style: None,
                                    extra: HashMap::new(),
                                }),
                                extra: HashMap::new(),
                            },
                        ],
                        extra: HashMap::new(),
                    }),
                    placeholder: None,
                    extra: HashMap::new(),
                }),
                image: None,
                table: None,
                extra: HashMap::new(),
            }],
            slide_properties: None,
            layout_properties: None,
            notes_page: None,
            extra: HashMap::new(),
        };
        let text = extract_slide_text(&page.page_elements);
        assert_eq!(text, "Hello World");
    }

    // Requirement: REQ-SLIDES-009 (Must)
    // Acceptance: Extract text from empty page
    #[test]
    fn req_slides_009_extract_text_empty_page() {
        let page = Page {
            object_id: Some("slide1".to_string()),
            page_type: Some("SLIDE".to_string()),
            page_elements: vec![],
            slide_properties: None,
            layout_properties: None,
            notes_page: None,
            extra: HashMap::new(),
        };
        let text = extract_slide_text(&page.page_elements);
        assert_eq!(text, "");
    }

    // Requirement: REQ-SLIDES-009 (Must)
    // Acceptance: Extract text from page with image elements (no text)
    #[test]
    fn req_slides_009_extract_text_image_only() {
        let page = Page {
            object_id: Some("slide1".to_string()),
            page_type: Some("SLIDE".to_string()),
            page_elements: vec![PageElement {
                object_id: Some("img1".to_string()),
                size: None,
                transform: None,
                title: None,
                description: None,
                shape: None,
                image: Some(Image {
                    content_url: Some("https://example.com/img.png".to_string()),
                    source_url: None,
                    extra: HashMap::new(),
                }),
                table: None,
                extra: HashMap::new(),
            }],
            slide_properties: None,
            layout_properties: None,
            notes_page: None,
            extra: HashMap::new(),
        };
        let text = extract_slide_text(&page.page_elements);
        assert_eq!(text, "");
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-011 (Should): Replace image request
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-011 (Should)
    // Acceptance: Replace image without page size
    #[test]
    fn req_slides_011_replace_image_no_size() {
        let req = build_replace_image_request("slide_1", "https://example.com/img.png", None);
        assert_eq!(req["createImage"]["url"], "https://example.com/img.png");
        assert_eq!(
            req["createImage"]["elementProperties"]["pageObjectId"],
            "slide_1"
        );
    }

    // Requirement: REQ-SLIDES-011 (Should)
    // Acceptance: Replace image with page size (full-bleed)
    #[test]
    fn req_slides_011_replace_image_full_bleed() {
        let ps = PageSize {
            width: Some(Dimension {
                magnitude: Some(9144000.0),
                unit: Some("EMU".to_string()),
            }),
            height: Some(Dimension {
                magnitude: Some(6858000.0),
                unit: Some("EMU".to_string()),
            }),
            extra: HashMap::new(),
        };
        let req =
            build_replace_image_request("slide_1", "https://example.com/img.png", Some(&ps));
        let ep = &req["createImage"]["elementProperties"];
        assert_eq!(ep["pageObjectId"], "slide_1");
        assert_eq!(ep["size"]["width"]["magnitude"], 9144000.0);
        assert_eq!(ep["size"]["height"]["magnitude"], 6858000.0);
        assert_eq!(ep["transform"]["translateX"], 0.0);
        assert_eq!(ep["transform"]["translateY"], 0.0);
    }

    // ---------------------------------------------------------------
    // Speaker notes extraction tests
    // ---------------------------------------------------------------

    // Acceptance: Extract speaker notes from slide
    #[test]
    fn extract_speaker_notes_present() {
        let page = Page {
            object_id: Some("slide1".to_string()),
            page_type: Some("SLIDE".to_string()),
            page_elements: vec![],
            slide_properties: Some(SlideProperties {
                layout_object_id: None,
                master_object_id: None,
                notes_page: Some(Box::new(NotesPage {
                    object_id: Some("notes1".to_string()),
                    page_type: Some("NOTES".to_string()),
                    page_elements: vec![PageElement {
                        object_id: Some("notesShape1".to_string()),
                        size: None,
                        transform: None,
                        title: None,
                        description: None,
                        shape: Some(Shape {
                            shape_type: Some("TEXT_BOX".to_string()),
                            text: Some(TextContent {
                                text_elements: vec![TextElement {
                                    start_index: Some(0),
                                    end_index: Some(15),
                                    paragraph_marker: None,
                                    text_run: Some(TextRun {
                                        content: Some("Speaker's notes".to_string()),
                                        style: None,
                                        extra: HashMap::new(),
                                    }),
                                    extra: HashMap::new(),
                                }],
                                extra: HashMap::new(),
                            }),
                            placeholder: None,
                            extra: HashMap::new(),
                        }),
                        image: None,
                        table: None,
                        extra: HashMap::new(),
                    }],
                    notes_properties: Some(NotesProperties {
                        speaker_notes_object_id: Some("notesShape1".to_string()),
                        extra: HashMap::new(),
                    }),
                    extra: HashMap::new(),
                })),
                extra: HashMap::new(),
            }),
            layout_properties: None,
            notes_page: None,
            extra: HashMap::new(),
        };
        let notes = extract_speaker_notes(&page);
        assert_eq!(notes, Some("Speaker's notes".to_string()));
    }

    // Acceptance: No notes page returns None
    #[test]
    fn extract_speaker_notes_absent() {
        let page = Page {
            object_id: Some("slide1".to_string()),
            page_type: Some("SLIDE".to_string()),
            page_elements: vec![],
            slide_properties: None,
            layout_properties: None,
            notes_page: None,
            extra: HashMap::new(),
        };
        let notes = extract_speaker_notes(&page);
        assert!(notes.is_none());
    }

    // Acceptance: Notes page with empty text returns None
    #[test]
    fn extract_speaker_notes_empty_text() {
        let page = Page {
            object_id: Some("slide1".to_string()),
            page_type: Some("SLIDE".to_string()),
            page_elements: vec![],
            slide_properties: Some(SlideProperties {
                layout_object_id: None,
                master_object_id: None,
                notes_page: Some(Box::new(NotesPage {
                    object_id: Some("notes1".to_string()),
                    page_type: Some("NOTES".to_string()),
                    page_elements: vec![],
                    notes_properties: Some(NotesProperties {
                        speaker_notes_object_id: Some("notesShape1".to_string()),
                        extra: HashMap::new(),
                    }),
                    extra: HashMap::new(),
                })),
                extra: HashMap::new(),
            }),
            layout_properties: None,
            notes_page: None,
            extra: HashMap::new(),
        };
        let notes = extract_speaker_notes(&page);
        assert!(notes.is_none());
    }
}
