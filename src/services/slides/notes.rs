//! Speaker notes operations for Google Slides.
//! Produces batchUpdate request objects for updating speaker notes.

use super::types::Page;

/// Build batchUpdate requests to update speaker notes text.
///
/// Returns a vector of two requests:
/// 1. `deleteText` -- removes all existing text from the notes shape
/// 2. `insertText` -- inserts the new text
///
/// The `notes_object_id` is the object ID of the speaker notes shape
/// (from `NotesProperties.speaker_notes_object_id`).
pub fn build_update_notes_request(notes_object_id: &str, text: &str) -> Vec<serde_json::Value> {
    vec![
        // First, delete all existing text
        serde_json::json!({
            "deleteText": {
                "objectId": notes_object_id,
                "textRange": {
                    "type": "ALL"
                }
            }
        }),
        // Then insert new text
        serde_json::json!({
            "insertText": {
                "objectId": notes_object_id,
                "text": text,
                "insertionIndex": 0
            }
        }),
    ]
}

/// Find the speaker notes shape object ID from a slide's page structure.
///
/// Looks for the notes page in `slide_properties.notes_page` or the
/// top-level `notes_page` field, then returns the `speaker_notes_object_id`
/// from `notes_properties`.
pub fn find_notes_object_id(page: &Page) -> Option<String> {
    // Try slide_properties.notes_page first
    if let Some(ref sp) = page.slide_properties {
        if let Some(ref np) = sp.notes_page {
            if let Some(ref props) = np.notes_properties {
                if let Some(ref id) = props.speaker_notes_object_id {
                    return Some(id.clone());
                }
            }
        }
    }

    // Fall back to top-level notes_page
    if let Some(ref np) = page.notes_page {
        if let Some(ref props) = np.notes_properties {
            if let Some(ref id) = props.speaker_notes_object_id {
                return Some(id.clone());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::*;
    use std::collections::HashMap;

    // ---------------------------------------------------------------
    // REQ-SLIDES-010 (Must): Update notes request
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-010 (Must)
    // Acceptance: Update notes produces deleteText and insertText
    #[test]
    fn req_slides_010_update_notes_request() {
        let reqs = build_update_notes_request("notesShape1", "New speaker notes");
        assert_eq!(reqs.len(), 2);

        // First request: deleteText
        assert!(reqs[0].get("deleteText").is_some());
        assert_eq!(reqs[0]["deleteText"]["objectId"], "notesShape1");
        assert_eq!(reqs[0]["deleteText"]["textRange"]["type"], "ALL");

        // Second request: insertText
        assert!(reqs[1].get("insertText").is_some());
        assert_eq!(reqs[1]["insertText"]["objectId"], "notesShape1");
        assert_eq!(reqs[1]["insertText"]["text"], "New speaker notes");
        assert_eq!(reqs[1]["insertText"]["insertionIndex"], 0);
    }

    // Requirement: REQ-SLIDES-010 (Must)
    // Acceptance: Update notes with empty text
    #[test]
    fn req_slides_010_update_notes_empty() {
        let reqs = build_update_notes_request("notesShape1", "");
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[1]["insertText"]["text"], "");
    }

    // Requirement: REQ-SLIDES-010 (Must)
    // Acceptance: Update notes with multiline text
    #[test]
    fn req_slides_010_update_notes_multiline() {
        let reqs = build_update_notes_request("ns1", "Line 1\nLine 2\nLine 3");
        assert_eq!(reqs[1]["insertText"]["text"], "Line 1\nLine 2\nLine 3");
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-010 (Must): Find notes object ID
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-010 (Must)
    // Acceptance: Find notes ID from slide_properties.notes_page
    #[test]
    fn req_slides_010_find_notes_id_from_slide_properties() {
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
        assert_eq!(find_notes_object_id(&page), Some("notesShape1".to_string()));
    }

    // Requirement: REQ-SLIDES-010 (Must)
    // Acceptance: Find notes ID from top-level notes_page
    #[test]
    fn req_slides_010_find_notes_id_from_top_level() {
        let page = Page {
            object_id: Some("slide1".to_string()),
            page_type: Some("SLIDE".to_string()),
            page_elements: vec![],
            slide_properties: None,
            layout_properties: None,
            notes_page: Some(Box::new(NotesPage {
                object_id: Some("notes2".to_string()),
                page_type: Some("NOTES".to_string()),
                page_elements: vec![],
                notes_properties: Some(NotesProperties {
                    speaker_notes_object_id: Some("notesShape2".to_string()),
                    extra: HashMap::new(),
                }),
                extra: HashMap::new(),
            })),
            extra: HashMap::new(),
        };
        assert_eq!(find_notes_object_id(&page), Some("notesShape2".to_string()));
    }

    // Edge case: No notes page at all
    #[test]
    fn find_notes_id_none() {
        let page = Page {
            object_id: Some("slide1".to_string()),
            page_type: Some("SLIDE".to_string()),
            page_elements: vec![],
            slide_properties: None,
            layout_properties: None,
            notes_page: None,
            extra: HashMap::new(),
        };
        assert_eq!(find_notes_object_id(&page), None);
    }

    // Edge case: Notes page exists but no notes_properties
    #[test]
    fn find_notes_id_no_properties() {
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
                    notes_properties: None,
                    extra: HashMap::new(),
                })),
                extra: HashMap::new(),
            }),
            layout_properties: None,
            notes_page: None,
            extra: HashMap::new(),
        };
        assert_eq!(find_notes_object_id(&page), None);
    }
}
