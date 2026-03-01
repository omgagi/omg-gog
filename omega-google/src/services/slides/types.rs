//! Google Slides API request/response types.
//! All types use camelCase serde rename and flatten for forward compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------
// Presentation
// ---------------------------------------------------------------

/// A Google Slides presentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    pub presentation_id: Option<String>,
    pub title: Option<String>,
    #[serde(default)]
    pub slides: Vec<Page>,
    #[serde(default)]
    pub masters: Vec<Page>,
    #[serde(default)]
    pub layouts: Vec<Page>,
    pub page_size: Option<PageSize>,
    pub locale: Option<String>,
    pub revision_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Page / Slide
// ---------------------------------------------------------------

/// A page in a presentation (slide, master, layout, or notes page).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub object_id: Option<String>,
    pub page_type: Option<String>,
    #[serde(default)]
    pub page_elements: Vec<PageElement>,
    pub slide_properties: Option<SlideProperties>,
    pub layout_properties: Option<serde_json::Value>,
    pub notes_page: Option<Box<NotesPage>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// PageElement
// ---------------------------------------------------------------

/// An element on a page (shape, image, table, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageElement {
    pub object_id: Option<String>,
    pub size: Option<Size>,
    pub transform: Option<Transform>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub shape: Option<Shape>,
    pub image: Option<Image>,
    pub table: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Shape / Text
// ---------------------------------------------------------------

/// A shape on a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shape {
    pub shape_type: Option<String>,
    pub text: Option<TextContent>,
    pub placeholder: Option<Placeholder>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// The text content within a shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextContent {
    #[serde(default)]
    pub text_elements: Vec<TextElement>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A text element (paragraph marker or text run).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextElement {
    pub start_index: Option<i64>,
    pub end_index: Option<i64>,
    pub paragraph_marker: Option<ParagraphMarker>,
    pub text_run: Option<TextRun>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A run of text with styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRun {
    pub content: Option<String>,
    pub style: Option<TextStyle>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Text styling information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextStyle {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub font_size: Option<Dimension>,
    pub foreground_color: Option<serde_json::Value>,
    pub link: Option<Link>,
    pub font_family: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Slide Properties / Notes
// ---------------------------------------------------------------

/// Properties of a slide page.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlideProperties {
    pub layout_object_id: Option<String>,
    pub master_object_id: Option<String>,
    pub notes_page: Option<Box<NotesPage>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A notes page attached to a slide.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesPage {
    pub object_id: Option<String>,
    pub page_type: Option<String>,
    #[serde(default)]
    pub page_elements: Vec<PageElement>,
    pub notes_properties: Option<NotesProperties>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Properties specific to a notes page.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesProperties {
    pub speaker_notes_object_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Dimensions / Layout
// ---------------------------------------------------------------

/// Page size with width and height dimensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageSize {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A dimension value with magnitude and unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dimension {
    pub magnitude: Option<f64>,
    pub unit: Option<String>,
}

/// A 2D affine transform.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transform {
    pub scale_x: Option<f64>,
    pub scale_y: Option<f64>,
    pub translate_x: Option<f64>,
    pub translate_y: Option<f64>,
    pub unit: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Size with width and height.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Size {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
}

// ---------------------------------------------------------------
// Image
// ---------------------------------------------------------------

/// An image on a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub content_url: Option<String>,
    pub source_url: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Misc types
// ---------------------------------------------------------------

/// Speaker notes text container (convenience type).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerNotesText {
    #[serde(default)]
    pub text_elements: Vec<TextElement>,
}

/// A paragraph marker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphMarker {
    pub style: Option<serde_json::Value>,
    pub bullet: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A link to a URL or slide.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub url: Option<String>,
    pub relative_link: Option<String>,
    pub slide_index: Option<i32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A placeholder reference on a shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Placeholder {
    pub r#type: Option<String>,
    pub index: Option<i32>,
    pub parent_object_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response from a batchUpdate request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateResponse {
    pub presentation_id: Option<String>,
    #[serde(default)]
    pub replies: Vec<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---------------------------------------------------------------
    // REQ-SLIDES-002 (Must): Presentation metadata deserialization
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-002 (Must)
    // Acceptance: Presentation deserializes from Slides API JSON
    #[test]
    fn req_slides_002_presentation_deserialize() {
        let json_str = r#"{
            "presentationId": "pres123",
            "title": "My Presentation",
            "slides": [],
            "masters": [],
            "layouts": [],
            "locale": "en_US",
            "revisionId": "rev_abc"
        }"#;
        let pres: Presentation = serde_json::from_str(json_str).unwrap();
        assert_eq!(pres.presentation_id, Some("pres123".to_string()));
        assert_eq!(pres.title, Some("My Presentation".to_string()));
        assert!(pres.slides.is_empty());
        assert_eq!(pres.locale, Some("en_US".to_string()));
        assert_eq!(pres.revision_id, Some("rev_abc".to_string()));
    }

    // Requirement: REQ-SLIDES-002 (Must)
    // Acceptance: Presentation round-trip serialization
    #[test]
    fn req_slides_002_presentation_roundtrip() {
        let pres = Presentation {
            presentation_id: Some("p1".to_string()),
            title: Some("Test".to_string()),
            slides: vec![],
            masters: vec![],
            layouts: vec![],
            page_size: None,
            locale: Some("en".to_string()),
            revision_id: None,
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&pres).unwrap();
        let parsed: Presentation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.presentation_id, Some("p1".to_string()));
        assert_eq!(parsed.title, Some("Test".to_string()));
    }

    // Requirement: REQ-SLIDES-002 (Must)
    // Edge case: Unknown fields preserved via flatten
    #[test]
    fn req_slides_002_presentation_unknown_fields() {
        let json_str = r#"{
            "presentationId": "p1",
            "title": "Test",
            "customField": "preserved"
        }"#;
        let pres: Presentation = serde_json::from_str(json_str).unwrap();
        assert!(pres.extra.contains_key("customField"));
    }

    // Requirement: REQ-SLIDES-002 (Must)
    // Acceptance: Presentation with page size
    #[test]
    fn req_slides_002_presentation_with_page_size() {
        let json_str = r#"{
            "presentationId": "p1",
            "title": "Sized",
            "pageSize": {
                "width": {"magnitude": 9144000, "unit": "EMU"},
                "height": {"magnitude": 6858000, "unit": "EMU"}
            }
        }"#;
        let pres: Presentation = serde_json::from_str(json_str).unwrap();
        let ps = pres.page_size.unwrap();
        let w = ps.width.unwrap();
        assert_eq!(w.magnitude, Some(9144000.0));
        assert_eq!(w.unit, Some("EMU".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-006 (Must): Page/slide types
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-006 (Must)
    // Acceptance: Page deserializes with slide properties
    #[test]
    fn req_slides_006_page_deserialize() {
        let json_str = r#"{
            "objectId": "slide1",
            "pageType": "SLIDE",
            "pageElements": [],
            "slideProperties": {
                "layoutObjectId": "layout1",
                "masterObjectId": "master1"
            }
        }"#;
        let page: Page = serde_json::from_str(json_str).unwrap();
        assert_eq!(page.object_id, Some("slide1".to_string()));
        assert_eq!(page.page_type, Some("SLIDE".to_string()));
        let sp = page.slide_properties.unwrap();
        assert_eq!(sp.layout_object_id, Some("layout1".to_string()));
        assert_eq!(sp.master_object_id, Some("master1".to_string()));
    }

    // Requirement: REQ-SLIDES-006 (Must)
    // Acceptance: Page round-trip
    #[test]
    fn req_slides_006_page_roundtrip() {
        let page = Page {
            object_id: Some("s1".to_string()),
            page_type: Some("SLIDE".to_string()),
            page_elements: vec![],
            slide_properties: None,
            layout_properties: None,
            notes_page: None,
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&page).unwrap();
        let parsed: Page = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.object_id, Some("s1".to_string()));
    }

    // Requirement: REQ-SLIDES-006 (Must)
    // Acceptance: Page with notes page
    #[test]
    fn req_slides_006_page_with_notes() {
        let json_str = r#"{
            "objectId": "slide1",
            "pageType": "SLIDE",
            "pageElements": [],
            "slideProperties": {
                "notesPage": {
                    "objectId": "notes1",
                    "pageType": "NOTES",
                    "pageElements": [],
                    "notesProperties": {
                        "speakerNotesObjectId": "notesShape1"
                    }
                }
            }
        }"#;
        let page: Page = serde_json::from_str(json_str).unwrap();
        let sp = page.slide_properties.unwrap();
        let notes = sp.notes_page.unwrap();
        assert_eq!(notes.object_id, Some("notes1".to_string()));
        let np = notes.notes_properties.unwrap();
        assert_eq!(np.speaker_notes_object_id, Some("notesShape1".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-SLIDES-009 (Must): PageElement structure
    // ---------------------------------------------------------------

    // Requirement: REQ-SLIDES-009 (Must)
    // Acceptance: PageElement with shape deserializes
    #[test]
    fn req_slides_009_page_element_shape() {
        let json_str = r#"{
            "objectId": "elem1",
            "shape": {
                "shapeType": "TEXT_BOX",
                "text": {
                    "textElements": [
                        {
                            "startIndex": 0,
                            "endIndex": 5,
                            "textRun": {
                                "content": "Hello",
                                "style": {
                                    "bold": true,
                                    "italic": false,
                                    "fontFamily": "Arial"
                                }
                            }
                        }
                    ]
                }
            }
        }"#;
        let elem: PageElement = serde_json::from_str(json_str).unwrap();
        assert_eq!(elem.object_id, Some("elem1".to_string()));
        let shape = elem.shape.unwrap();
        assert_eq!(shape.shape_type, Some("TEXT_BOX".to_string()));
        let text = shape.text.unwrap();
        assert_eq!(text.text_elements.len(), 1);
        let tr = text.text_elements[0].text_run.as_ref().unwrap();
        assert_eq!(tr.content, Some("Hello".to_string()));
        let style = tr.style.as_ref().unwrap();
        assert_eq!(style.bold, Some(true));
        assert_eq!(style.font_family, Some("Arial".to_string()));
    }

    // Requirement: REQ-SLIDES-009 (Must)
    // Acceptance: PageElement with image deserializes
    #[test]
    fn req_slides_009_page_element_image() {
        let json_str = r#"{
            "objectId": "img1",
            "image": {
                "contentUrl": "https://example.com/image.png",
                "sourceUrl": "https://source.example.com/original.png"
            }
        }"#;
        let elem: PageElement = serde_json::from_str(json_str).unwrap();
        let image = elem.image.unwrap();
        assert_eq!(image.content_url, Some("https://example.com/image.png".to_string()));
        assert_eq!(image.source_url, Some("https://source.example.com/original.png".to_string()));
    }

    // Requirement: REQ-SLIDES-009 (Must)
    // Acceptance: PageElement with transform and size
    #[test]
    fn req_slides_009_page_element_transform() {
        let json_str = r#"{
            "objectId": "elem1",
            "size": {
                "width": {"magnitude": 100.0, "unit": "PT"},
                "height": {"magnitude": 50.0, "unit": "PT"}
            },
            "transform": {
                "scaleX": 1.0,
                "scaleY": 1.0,
                "translateX": 10.0,
                "translateY": 20.0,
                "unit": "PT"
            }
        }"#;
        let elem: PageElement = serde_json::from_str(json_str).unwrap();
        let size = elem.size.unwrap();
        assert_eq!(size.width.as_ref().unwrap().magnitude, Some(100.0));
        let transform = elem.transform.unwrap();
        assert_eq!(transform.scale_x, Some(1.0));
        assert_eq!(transform.translate_x, Some(10.0));
    }

    // ---------------------------------------------------------------
    // Additional round-trip tests
    // ---------------------------------------------------------------

    // TextContent round-trip
    #[test]
    fn text_content_roundtrip() {
        let tc = TextContent {
            text_elements: vec![TextElement {
                start_index: Some(0),
                end_index: Some(3),
                paragraph_marker: None,
                text_run: Some(TextRun {
                    content: Some("Hi!".to_string()),
                    style: None,
                    extra: HashMap::new(),
                }),
                extra: HashMap::new(),
            }],
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&tc).unwrap();
        let parsed: TextContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.text_elements.len(), 1);
        assert_eq!(
            parsed.text_elements[0].text_run.as_ref().unwrap().content,
            Some("Hi!".to_string())
        );
    }

    // BatchUpdateResponse deserialization
    #[test]
    fn batch_update_response_deserialize() {
        let json_str = r#"{
            "presentationId": "pres123",
            "replies": [
                {"createSlide": {"objectId": "newSlide1"}},
                {}
            ]
        }"#;
        let resp: BatchUpdateResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.presentation_id, Some("pres123".to_string()));
        assert_eq!(resp.replies.len(), 2);
    }

    // Link type round-trip
    #[test]
    fn link_roundtrip() {
        let link = Link {
            url: Some("https://example.com".to_string()),
            relative_link: None,
            slide_index: Some(2),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&link).unwrap();
        let parsed: Link = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.url, Some("https://example.com".to_string()));
        assert_eq!(parsed.slide_index, Some(2));
    }

    // Placeholder type round-trip
    #[test]
    fn placeholder_roundtrip() {
        let ph = Placeholder {
            r#type: Some("TITLE".to_string()),
            index: Some(0),
            parent_object_id: Some("layout1".to_string()),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&ph).unwrap();
        let parsed: Placeholder = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.r#type, Some("TITLE".to_string()));
        assert_eq!(parsed.index, Some(0));
    }

    // Dimension type round-trip
    #[test]
    fn dimension_roundtrip() {
        let dim = Dimension {
            magnitude: Some(72.0),
            unit: Some("PT".to_string()),
        };
        let json = serde_json::to_string(&dim).unwrap();
        let parsed: Dimension = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.magnitude, Some(72.0));
        assert_eq!(parsed.unit, Some("PT".to_string()));
    }
}
