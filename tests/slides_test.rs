//! Slides service integration tests.

use omega_google::services::slides::types::*;
use omega_google::services::slides::markdown::*;

// ---------------------------------------------------------------
// REQ-SLIDES-002 (Must): Presentation from realistic API response
// ---------------------------------------------------------------

// Requirement: REQ-SLIDES-002 (Must)
// Acceptance: Full presentation structure from Slides API
#[test]
fn req_slides_002_integration_presentation_from_api() {
    let api_response = r#"{
        "presentationId": "1a2b3c4d5e_pres",
        "title": "Q1 2024 Business Review",
        "locale": "en_US",
        "revisionId": "ALm37BWuZnVRhxYh8_pres",
        "pageSize": {
            "width": {"magnitude": 9144000, "unit": "EMU"},
            "height": {"magnitude": 6858000, "unit": "EMU"}
        },
        "slides": [
            {
                "objectId": "slide_001",
                "pageType": "SLIDE",
                "pageElements": [
                    {
                        "objectId": "title_001",
                        "shape": {
                            "shapeType": "TEXT_BOX",
                            "text": {
                                "textElements": [
                                    {
                                        "startIndex": 0,
                                        "endIndex": 0,
                                        "paragraphMarker": {
                                            "style": {}
                                        }
                                    },
                                    {
                                        "startIndex": 0,
                                        "endIndex": 25,
                                        "textRun": {
                                            "content": "Q1 2024 Business Review\n",
                                            "style": {
                                                "bold": true,
                                                "fontSize": {"magnitude": 28, "unit": "PT"},
                                                "fontFamily": "Google Sans"
                                            }
                                        }
                                    }
                                ]
                            },
                            "placeholder": {
                                "type": "TITLE",
                                "index": 0
                            }
                        }
                    }
                ],
                "slideProperties": {
                    "layoutObjectId": "layout_blank",
                    "masterObjectId": "master_default"
                }
            },
            {
                "objectId": "slide_002",
                "pageType": "SLIDE",
                "pageElements": [
                    {
                        "objectId": "body_002",
                        "shape": {
                            "shapeType": "TEXT_BOX",
                            "text": {
                                "textElements": [
                                    {
                                        "startIndex": 0,
                                        "endIndex": 15,
                                        "textRun": {
                                            "content": "Revenue grew by 15% year-over-year.\n",
                                            "style": {}
                                        }
                                    }
                                ]
                            }
                        }
                    },
                    {
                        "objectId": "chart_002",
                        "image": {
                            "contentUrl": "https://lh3.googleusercontent.com/chart_001",
                            "sourceUrl": "https://docs.google.com/spreadsheets/d/abc/chart"
                        },
                        "size": {
                            "width": {"magnitude": 5000000, "unit": "EMU"},
                            "height": {"magnitude": 3000000, "unit": "EMU"}
                        },
                        "transform": {
                            "scaleX": 1.0,
                            "scaleY": 1.0,
                            "translateX": 2000000,
                            "translateY": 2000000,
                            "unit": "EMU"
                        }
                    }
                ],
                "slideProperties": {
                    "layoutObjectId": "layout_content",
                    "masterObjectId": "master_default",
                    "notesPage": {
                        "objectId": "notes_002",
                        "pageType": "NOTES",
                        "pageElements": [
                            {
                                "objectId": "notes_shape_002",
                                "shape": {
                                    "shapeType": "TEXT_BOX",
                                    "text": {
                                        "textElements": [
                                            {
                                                "textRun": {
                                                    "content": "Mention the partnership deal.\n"
                                                }
                                            }
                                        ]
                                    }
                                }
                            }
                        ],
                        "notesProperties": {
                            "speakerNotesObjectId": "notes_shape_002"
                        }
                    }
                }
            }
        ],
        "masters": [
            {
                "objectId": "master_default",
                "pageType": "MASTER",
                "pageElements": []
            }
        ],
        "layouts": [
            {
                "objectId": "layout_blank",
                "pageType": "LAYOUT",
                "pageElements": [],
                "layoutProperties": {"name": "Blank"}
            },
            {
                "objectId": "layout_content",
                "pageType": "LAYOUT",
                "pageElements": [],
                "layoutProperties": {"name": "Title and Content"}
            }
        ]
    }"#;

    let pres: Presentation = serde_json::from_str(api_response).unwrap();

    // Verify presentation metadata
    assert_eq!(pres.presentation_id, Some("1a2b3c4d5e_pres".to_string()));
    assert_eq!(pres.title, Some("Q1 2024 Business Review".to_string()));
    assert_eq!(pres.locale, Some("en_US".to_string()));
    assert_eq!(pres.revision_id, Some("ALm37BWuZnVRhxYh8_pres".to_string()));

    // Verify page size
    let ps = pres.page_size.as_ref().unwrap();
    assert_eq!(ps.width.as_ref().unwrap().magnitude, Some(9144000.0));
    assert_eq!(ps.width.as_ref().unwrap().unit, Some("EMU".to_string()));

    // Verify slides
    assert_eq!(pres.slides.len(), 2);

    // First slide: title slide
    let s1 = &pres.slides[0];
    assert_eq!(s1.object_id, Some("slide_001".to_string()));
    assert_eq!(s1.page_type, Some("SLIDE".to_string()));
    assert_eq!(s1.page_elements.len(), 1);

    let title_shape = s1.page_elements[0].shape.as_ref().unwrap();
    assert_eq!(title_shape.shape_type, Some("TEXT_BOX".to_string()));
    let placeholder = title_shape.placeholder.as_ref().unwrap();
    assert_eq!(placeholder.r#type, Some("TITLE".to_string()));

    let sp1 = s1.slide_properties.as_ref().unwrap();
    assert_eq!(sp1.layout_object_id, Some("layout_blank".to_string()));

    // Second slide: content with image and notes
    let s2 = &pres.slides[1];
    assert_eq!(s2.page_elements.len(), 2);

    // Image element
    let img = s2.page_elements[1].image.as_ref().unwrap();
    assert!(img.content_url.as_ref().unwrap().contains("chart_001"));

    // Size and transform
    let size = s2.page_elements[1].size.as_ref().unwrap();
    assert_eq!(size.width.as_ref().unwrap().magnitude, Some(5000000.0));

    let transform = s2.page_elements[1].transform.as_ref().unwrap();
    assert_eq!(transform.scale_x, Some(1.0));
    assert_eq!(transform.translate_x, Some(2000000.0));

    // Notes page
    let sp2 = s2.slide_properties.as_ref().unwrap();
    let notes = sp2.notes_page.as_ref().unwrap();
    assert_eq!(notes.object_id, Some("notes_002".to_string()));
    let notes_props = notes.notes_properties.as_ref().unwrap();
    assert_eq!(notes_props.speaker_notes_object_id, Some("notes_shape_002".to_string()));

    // Verify masters and layouts
    assert_eq!(pres.masters.len(), 1);
    assert_eq!(pres.layouts.len(), 2);
}

// ---------------------------------------------------------------
// REQ-SLIDES-004 (Must): Markdown-to-slides end-to-end
// ---------------------------------------------------------------

// Requirement: REQ-SLIDES-004 (Must)
// Acceptance: Parse realistic Markdown content into slides
#[test]
fn req_slides_004_integration_markdown_to_slides() {
    let markdown = r#"# Company Overview

Founded in 2015, we have grown to 500+ employees.

# Revenue Growth

- Q1: $10M
- Q2: $12M
- Q3: $15M
- Q4: $18M

Total annual revenue: $55M

# Goals for 2025

1. Expand into European markets
2. Launch mobile application
3. Achieve profitability

---

# Appendix

Additional data and references."#;

    let slides = parse_markdown_to_slides(markdown);

    // Should produce 4 slides (4 headings, --- is only a boundary)
    assert_eq!(slides.len(), 4);

    // First slide: Company Overview
    assert_eq!(slides[0].title, "Company Overview");
    assert!(slides[0].body.contains("Founded in 2015"));

    // Second slide: Revenue Growth
    assert_eq!(slides[1].title, "Revenue Growth");
    assert!(slides[1].body.contains("Q1: $10M"));
    assert!(slides[1].body.contains("Total annual revenue"));

    // Third slide: Goals
    assert_eq!(slides[2].title, "Goals for 2025");
    assert!(slides[2].body.contains("European markets"));

    // Fourth: Appendix (--- is only a boundary, no blank slides)
    assert_eq!(slides[3].title, "Appendix");
    assert!(slides[3].body.contains("Additional data"));
}

// Requirement: REQ-SLIDES-004 (Must)
// Acceptance: Build batchUpdate requests from markdown slides
#[test]
fn req_slides_004_integration_build_requests_from_markdown() {
    let markdown = "# Title Slide\nIntroduction\n# Content Slide\nDetails here";
    let slides = parse_markdown_to_slides(markdown);
    assert_eq!(slides.len(), 2);

    let body = build_slides_from_markdown(&slides);
    let reqs = body["requests"].as_array().unwrap();

    // Should have requests for creating slides, shapes, and inserting text
    assert!(reqs.len() >= 6, "Expected at least 6 requests, got {}", reqs.len());

    // First request should be createSlide
    assert!(reqs[0]["createSlide"].is_object(), "First request should be createSlide");
}

// Requirement: REQ-SLIDES-004 (Must)
// Acceptance: Empty markdown produces no slides
#[test]
fn req_slides_004_integration_empty_markdown() {
    let slides = parse_markdown_to_slides("");
    assert!(slides.is_empty());

    let body = build_slides_from_markdown(&slides);
    let reqs = body["requests"].as_array().unwrap();
    assert!(reqs.is_empty());
}

// ---------------------------------------------------------------
// REQ-SLIDES-002 (Must): Minimal presentation
// ---------------------------------------------------------------

// Requirement: REQ-SLIDES-002 (Must)
// Acceptance: Minimal presentation deserializes
#[test]
fn req_slides_002_integration_minimal_presentation() {
    let api_response = r#"{
        "presentationId": "pres_minimal"
    }"#;

    let pres: Presentation = serde_json::from_str(api_response).unwrap();
    assert_eq!(pres.presentation_id, Some("pres_minimal".to_string()));
    assert!(pres.title.is_none());
    assert!(pres.slides.is_empty());
    assert!(pres.masters.is_empty());
    assert!(pres.layouts.is_empty());
    assert!(pres.page_size.is_none());
}

// ---------------------------------------------------------------
// REQ-SLIDES-009 (Must): PageElement text extraction
// ---------------------------------------------------------------

// Requirement: REQ-SLIDES-009 (Must)
// Acceptance: Extract text from slide page elements
#[test]
fn req_slides_009_integration_text_extraction() {
    let slide_json = r#"{
        "objectId": "slide_test",
        "pageType": "SLIDE",
        "pageElements": [
            {
                "objectId": "shape1",
                "shape": {
                    "shapeType": "TEXT_BOX",
                    "text": {
                        "textElements": [
                            {
                                "startIndex": 0,
                                "endIndex": 5,
                                "textRun": {
                                    "content": "Hello"
                                }
                            },
                            {
                                "startIndex": 5,
                                "endIndex": 11,
                                "textRun": {
                                    "content": " World"
                                }
                            }
                        ]
                    }
                }
            },
            {
                "objectId": "shape2",
                "shape": {
                    "shapeType": "TEXT_BOX",
                    "text": {
                        "textElements": [
                            {
                                "textRun": {
                                    "content": "\nSecond shape"
                                }
                            }
                        ]
                    }
                }
            }
        ]
    }"#;

    let page: Page = serde_json::from_str(slide_json).unwrap();

    use omega_google::services::slides::slides_ops::extract_slide_text;
    let text = extract_slide_text(&page.page_elements);
    assert!(text.contains("Hello"));
    assert!(text.contains("World"));
    assert!(text.contains("Second shape"));
}
