//! Document edit operations: batch update body builders for insert, delete, find-replace.

use super::DOCS_BASE_URL;

/// Build the batchUpdate URL for a document.
pub fn build_batch_update_url(doc_id: &str) -> String {
    format!("{}/documents/{}:batchUpdate", DOCS_BASE_URL, doc_id)
}

/// Build a batchUpdate request body to insert text at a given index.
pub fn build_insert_text_body(text: &str, index: i64) -> serde_json::Value {
    serde_json::json!({
        "requests": [
            {
                "insertText": {
                    "location": {
                        "index": index
                    },
                    "text": text
                }
            }
        ]
    })
}

/// Build a batchUpdate request body to delete a range of text.
pub fn build_delete_content_range_body(start_index: i64, end_index: i64) -> serde_json::Value {
    serde_json::json!({
        "requests": [
            {
                "deleteContentRange": {
                    "range": {
                        "startIndex": start_index,
                        "endIndex": end_index
                    }
                }
            }
        ]
    })
}

/// Build a batchUpdate request body for replaceAllText.
pub fn build_replace_all_text_body(find: &str, replace: &str, match_case: bool) -> serde_json::Value {
    serde_json::json!({
        "requests": [
            {
                "replaceAllText": {
                    "containsText": {
                        "text": find,
                        "matchCase": match_case
                    },
                    "replaceText": replace
                }
            }
        ]
    })
}

/// Build a batchUpdate request body to clear all document content.
/// This deletes from index 1 to end_index - 1.
pub fn build_clear_body(end_index: i64) -> serde_json::Value {
    if end_index <= 1 {
        // Document is already empty
        return serde_json::json!({"requests": []});
    }
    build_delete_content_range_body(1, end_index - 1)
}

/// Build a batchUpdate request body to replace all content.
/// First clears, then inserts new content.
pub fn build_replace_content_body(new_content: &str, end_index: i64) -> serde_json::Value {
    let mut requests = Vec::new();
    if end_index > 1 {
        requests.push(serde_json::json!({
            "deleteContentRange": {
                "range": {
                    "startIndex": 1,
                    "endIndex": end_index - 1
                }
            }
        }));
    }
    requests.push(serde_json::json!({
        "insertText": {
            "location": {
                "index": 1
            },
            "text": new_content
        }
    }));
    serde_json::json!({"requests": requests})
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ-DOCS-009
    #[test]
    fn req_docs_009_insert_text_body() {
        let body = build_insert_text_body("Hello", 1);
        let req = &body["requests"][0]["insertText"];
        assert_eq!(req["text"], "Hello");
        assert_eq!(req["location"]["index"], 1);
    }

    // REQ-DOCS-010
    #[test]
    fn req_docs_010_delete_range_body() {
        let body = build_delete_content_range_body(5, 10);
        let req = &body["requests"][0]["deleteContentRange"]["range"];
        assert_eq!(req["startIndex"], 5);
        assert_eq!(req["endIndex"], 10);
    }

    // REQ-DOCS-011
    #[test]
    fn req_docs_011_replace_all_text_body() {
        let body = build_replace_all_text_body("old", "new", true);
        let req = &body["requests"][0]["replaceAllText"];
        assert_eq!(req["containsText"]["text"], "old");
        assert_eq!(req["replaceText"], "new");
        assert_eq!(req["containsText"]["matchCase"], true);
    }

    // REQ-DOCS-015
    #[test]
    fn req_docs_015_clear_body() {
        let body = build_clear_body(100);
        let req = &body["requests"][0]["deleteContentRange"]["range"];
        assert_eq!(req["startIndex"], 1);
        assert_eq!(req["endIndex"], 99);
    }

    #[test]
    fn req_docs_015_clear_empty_doc() {
        let body = build_clear_body(1);
        let reqs = body["requests"].as_array().unwrap();
        assert!(reqs.is_empty());
    }

    #[test]
    fn test_batch_update_url() {
        let url = build_batch_update_url("doc123");
        assert!(url.contains("documents/doc123:batchUpdate"));
    }

    #[test]
    fn test_replace_content_body() {
        let body = build_replace_content_body("new content", 50);
        let reqs = body["requests"].as_array().unwrap();
        assert_eq!(reqs.len(), 2);
    }
}
