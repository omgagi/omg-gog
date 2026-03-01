//! MIME message construction for sending emails.
//! Constructs RFC 2822 compliant messages with support for:
//! - Plain text and HTML bodies
//! - CC, BCC, Reply-To headers
//! - File attachments (base64 encoded)
//! - In-Reply-To / References for threading

use base64::Engine;

/// Parameters for constructing a MIME email message.
#[derive(Debug, Clone, Default)]
pub struct MimeMessageParams {
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub reply_to: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Option<String>,
    pub attachments: Vec<MimeAttachment>,
}

/// An attachment to include in the MIME message.
#[derive(Debug, Clone)]
pub struct MimeAttachment {
    pub filename: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

/// Build a complete RFC 2822 MIME message from parameters.
/// Returns the raw message as a string suitable for base64url encoding and sending via Gmail API.
pub fn build_mime_message(params: &MimeMessageParams) -> String {
    let mut headers = Vec::new();

    headers.push(format!("From: {}", params.from));
    if !params.to.is_empty() {
        headers.push(format!("To: {}", params.to.join(", ")));
    }
    if !params.cc.is_empty() {
        headers.push(format!("Cc: {}", params.cc.join(", ")));
    }
    if !params.bcc.is_empty() {
        headers.push(format!("Bcc: {}", params.bcc.join(", ")));
    }
    if let Some(ref reply_to) = params.reply_to {
        headers.push(format!("Reply-To: {}", reply_to));
    }
    if let Some(ref in_reply_to) = params.in_reply_to {
        headers.push(format!("In-Reply-To: {}", in_reply_to));
    }
    if let Some(ref references) = params.references {
        headers.push(format!("References: {}", references));
    }
    headers.push(format!("Subject: {}", params.subject));
    headers.push("MIME-Version: 1.0".to_string());

    let has_attachments = !params.attachments.is_empty();
    let has_html = params.body_html.is_some();
    let has_text = params.body_text.is_some();

    if has_attachments {
        let boundary = generate_boundary();
        headers.push(format!(
            "Content-Type: multipart/mixed; boundary=\"{}\"",
            boundary
        ));
        let mut body = String::new();
        body.push_str(&headers.join("\r\n"));
        body.push_str("\r\n\r\n");

        // Text body part
        if let Some(ref text) = params.body_text {
            body.push_str(&format!("--{}\r\n", boundary));
            body.push_str("Content-Type: text/plain; charset=\"UTF-8\"\r\n\r\n");
            body.push_str(text);
            body.push_str("\r\n");
        } else if let Some(ref html) = params.body_html {
            body.push_str(&format!("--{}\r\n", boundary));
            body.push_str("Content-Type: text/html; charset=\"UTF-8\"\r\n\r\n");
            body.push_str(html);
            body.push_str("\r\n");
        }

        // Attachment parts
        for att in &params.attachments {
            body.push_str(&format!("--{}\r\n", boundary));
            body.push_str(&format!(
                "Content-Type: {}; name=\"{}\"\r\n",
                att.content_type, att.filename
            ));
            body.push_str("Content-Transfer-Encoding: base64\r\n");
            body.push_str(&format!(
                "Content-Disposition: attachment; filename=\"{}\"\r\n\r\n",
                att.filename
            ));
            let encoded = base64::engine::general_purpose::STANDARD.encode(&att.data);
            body.push_str(&encoded);
            body.push_str("\r\n");
        }

        body.push_str(&format!("--{}--\r\n", boundary));
        body
    } else if has_html && has_text {
        let boundary = generate_boundary();
        headers.push(format!(
            "Content-Type: multipart/alternative; boundary=\"{}\"",
            boundary
        ));
        let mut body = String::new();
        body.push_str(&headers.join("\r\n"));
        body.push_str("\r\n\r\n");

        body.push_str(&format!("--{}\r\n", boundary));
        body.push_str("Content-Type: text/plain; charset=\"UTF-8\"\r\n\r\n");
        body.push_str(params.body_text.as_ref().unwrap());
        body.push_str("\r\n");

        body.push_str(&format!("--{}\r\n", boundary));
        body.push_str("Content-Type: text/html; charset=\"UTF-8\"\r\n\r\n");
        body.push_str(params.body_html.as_ref().unwrap());
        body.push_str("\r\n");

        body.push_str(&format!("--{}--\r\n", boundary));
        body
    } else if has_html {
        headers.push("Content-Type: text/html; charset=\"UTF-8\"".to_string());
        let mut body = headers.join("\r\n");
        body.push_str("\r\n\r\n");
        body.push_str(params.body_html.as_ref().unwrap());
        body
    } else {
        headers.push("Content-Type: text/plain; charset=\"UTF-8\"".to_string());
        let mut body = headers.join("\r\n");
        body.push_str("\r\n\r\n");
        if let Some(ref text) = params.body_text {
            body.push_str(text);
        }
        body
    }
}

/// Generate a MIME boundary string.
pub fn generate_boundary() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random: u64 = rng.gen();
    format!("boundary_{:016x}", random)
}

/// Encode data as base64url (Gmail API format, no padding).
pub fn base64url_encode(data: &[u8]) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    Engine::encode(&URL_SAFE_NO_PAD, data)
}

/// Guess the MIME content type from a file extension.
pub fn guess_content_type(filename: &str) -> String {
    if !filename.contains('.') {
        return "application/octet-stream".to_string();
    }
    let ext = filename.rsplit('.').next().unwrap_or("");
    match ext.to_lowercase().as_str() {
        "pdf" => "application/pdf".to_string(),
        "txt" => "text/plain".to_string(),
        "html" | "htm" => "text/html".to_string(),
        "csv" => "text/csv".to_string(),
        "png" => "image/png".to_string(),
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "gif" => "image/gif".to_string(),
        "svg" => "image/svg+xml".to_string(),
        "json" => "application/json".to_string(),
        "xml" => "application/xml".to_string(),
        "zip" => "application/zip".to_string(),
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string(),
        "mp4" => "video/mp4".to_string(),
        "mp3" => "audio/mpeg".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-GMAIL-010 (Must): MIME message construction
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-010 (Must)
    // Acceptance: Simple plain text email
    #[test]
    fn req_gmail_010_simple_plain_text() {
        let params = MimeMessageParams {
            from: "sender@example.com".to_string(),
            to: vec!["recipient@example.com".to_string()],
            subject: "Test Subject".to_string(),
            body_text: Some("Hello, World!".to_string()),
            ..Default::default()
        };
        let mime = build_mime_message(&params);
        assert!(mime.contains("From: sender@example.com"));
        assert!(mime.contains("To: recipient@example.com"));
        assert!(mime.contains("Subject: Test Subject"));
        assert!(mime.contains("Hello, World!"));
    }

    // Requirement: REQ-GMAIL-010 (Must)
    // Acceptance: Email with HTML body
    #[test]
    fn req_gmail_010_html_body() {
        let params = MimeMessageParams {
            from: "sender@example.com".to_string(),
            to: vec!["recipient@example.com".to_string()],
            subject: "HTML Test".to_string(),
            body_html: Some("<h1>Hello</h1>".to_string()),
            ..Default::default()
        };
        let mime = build_mime_message(&params);
        assert!(mime.contains("Content-Type: text/html"));
        assert!(mime.contains("<h1>Hello</h1>"));
    }

    // Requirement: REQ-GMAIL-010 (Must)
    // Acceptance: Email with CC and BCC
    #[test]
    fn req_gmail_010_cc_bcc() {
        let params = MimeMessageParams {
            from: "sender@example.com".to_string(),
            to: vec!["to@example.com".to_string()],
            cc: vec!["cc@example.com".to_string()],
            bcc: vec!["bcc@example.com".to_string()],
            subject: "CC/BCC Test".to_string(),
            body_text: Some("test".to_string()),
            ..Default::default()
        };
        let mime = build_mime_message(&params);
        assert!(mime.contains("Cc: cc@example.com"));
        assert!(mime.contains("Bcc: bcc@example.com"));
    }

    // Requirement: REQ-GMAIL-010 (Must)
    // Acceptance: Reply-To header
    #[test]
    fn req_gmail_010_reply_to() {
        let params = MimeMessageParams {
            from: "sender@example.com".to_string(),
            to: vec!["to@example.com".to_string()],
            subject: "Reply Test".to_string(),
            body_text: Some("test".to_string()),
            reply_to: Some("reply@example.com".to_string()),
            ..Default::default()
        };
        let mime = build_mime_message(&params);
        assert!(mime.contains("Reply-To: reply@example.com"));
    }

    // Requirement: REQ-GMAIL-010 (Must)
    // Acceptance: Threading via In-Reply-To and References
    #[test]
    fn req_gmail_010_threading_headers() {
        let params = MimeMessageParams {
            from: "sender@example.com".to_string(),
            to: vec!["to@example.com".to_string()],
            subject: "Re: Original".to_string(),
            body_text: Some("test".to_string()),
            in_reply_to: Some("<original@example.com>".to_string()),
            references: Some("<original@example.com>".to_string()),
            ..Default::default()
        };
        let mime = build_mime_message(&params);
        assert!(mime.contains("In-Reply-To: <original@example.com>"));
        assert!(mime.contains("References: <original@example.com>"));
    }

    // Requirement: REQ-GMAIL-010 (Must)
    // Acceptance: Attachment included in MIME
    #[test]
    fn req_gmail_010_with_attachment() {
        let params = MimeMessageParams {
            from: "sender@example.com".to_string(),
            to: vec!["to@example.com".to_string()],
            subject: "Attachment Test".to_string(),
            body_text: Some("See attached.".to_string()),
            attachments: vec![MimeAttachment {
                filename: "test.pdf".to_string(),
                content_type: "application/pdf".to_string(),
                data: vec![0x25, 0x50, 0x44, 0x46], // %PDF
            }],
            ..Default::default()
        };
        let mime = build_mime_message(&params);
        assert!(mime.contains("test.pdf"));
        assert!(mime.contains("application/pdf"));
    }

    // Requirement: REQ-GMAIL-010 (Must)
    // Acceptance: Multiple recipients separated by comma
    #[test]
    fn req_gmail_010_multiple_recipients() {
        let params = MimeMessageParams {
            from: "sender@example.com".to_string(),
            to: vec!["a@example.com".to_string(), "b@example.com".to_string()],
            subject: "Multi".to_string(),
            body_text: Some("test".to_string()),
            ..Default::default()
        };
        let mime = build_mime_message(&params);
        assert!(mime.contains("a@example.com"));
        assert!(mime.contains("b@example.com"));
    }

    // Requirement: REQ-GMAIL-010 (Must)
    // Edge case: Subject with special characters
    #[test]
    fn req_gmail_010_special_chars_subject() {
        let params = MimeMessageParams {
            from: "sender@example.com".to_string(),
            to: vec!["to@example.com".to_string()],
            subject: "Test: Hello & Goodbye \"quotes\"".to_string(),
            body_text: Some("test".to_string()),
            ..Default::default()
        };
        let mime = build_mime_message(&params);
        assert!(mime.contains("Subject:"));
    }

    // Requirement: REQ-GMAIL-010 (Must)
    // Edge case: Empty body
    #[test]
    fn req_gmail_010_empty_body() {
        let params = MimeMessageParams {
            from: "sender@example.com".to_string(),
            to: vec!["to@example.com".to_string()],
            subject: "Empty".to_string(),
            ..Default::default()
        };
        let mime = build_mime_message(&params);
        assert!(mime.contains("From:"));
        assert!(mime.contains("Subject:"));
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-010 (Must): base64url encoding
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-010 (Must)
    // Acceptance: base64url encoding for Gmail API
    #[test]
    fn req_gmail_010_base64url_encode() {
        let encoded = base64url_encode(b"Hello, World!");
        // base64url should not have + / or padding =
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
        assert!(!encoded.contains('='));
    }

    // Requirement: REQ-GMAIL-010 (Must)
    // Edge case: Empty data
    #[test]
    fn req_gmail_010_base64url_empty() {
        let encoded = base64url_encode(b"");
        assert_eq!(encoded, "");
    }

    // ---------------------------------------------------------------
    // REQ-GMAIL-010 (Must): Content type guessing
    // ---------------------------------------------------------------

    // Requirement: REQ-GMAIL-010 (Must)
    // Acceptance: Guesses common MIME types from extensions
    #[test]
    fn req_gmail_010_guess_content_type_pdf() {
        assert_eq!(guess_content_type("document.pdf"), "application/pdf");
    }

    #[test]
    fn req_gmail_010_guess_content_type_png() {
        assert_eq!(guess_content_type("image.png"), "image/png");
    }

    #[test]
    fn req_gmail_010_guess_content_type_txt() {
        assert_eq!(guess_content_type("readme.txt"), "text/plain");
    }

    #[test]
    fn req_gmail_010_guess_content_type_unknown() {
        assert_eq!(guess_content_type("file.xyz"), "application/octet-stream");
    }

    #[test]
    fn req_gmail_010_guess_content_type_no_ext() {
        assert_eq!(guess_content_type("Makefile"), "application/octet-stream");
    }
}
