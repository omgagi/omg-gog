//! MIME message construction for sending emails.
//! Constructs RFC 2822 compliant messages with support for:
//! - Plain text and HTML bodies
//! - CC, BCC, Reply-To headers
//! - File attachments (base64 encoded)
//! - In-Reply-To / References for threading

use base64::Engine;

/// RFC 2047 encode a header value if it contains non-ASCII characters.
/// Uses Base64 variant: =?UTF-8?B?<base64>?=
/// Pure-ASCII strings are returned unchanged.
fn encode_rfc2047(value: &str) -> String {
    if value.is_ascii() {
        return value.to_string();
    }
    // RFC 2047 encoded-words have a 75-char line limit. Each encoded word:
    //   =?UTF-8?B?...?=  (prefix=10, suffix=2 → 12 overhead, leaving 63 for base64)
    // 63 base64 chars → 47 raw bytes per chunk. We split on UTF-8 char boundaries.
    const MAX_RAW_BYTES: usize = 45; // conservative to stay within 75-char limit
    let bytes = value.as_bytes();
    let mut words = Vec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        let mut end = (pos + MAX_RAW_BYTES).min(bytes.len());
        // Don't split in the middle of a UTF-8 character
        while end < bytes.len() && (bytes[end] & 0b1100_0000) == 0b1000_0000 {
            end -= 1;
        }
        if end == pos {
            // Edge case: single char wider than budget (shouldn't happen at 45 bytes)
            end = pos + 1;
            while end < bytes.len() && (bytes[end] & 0b1100_0000) == 0b1000_0000 {
                end += 1;
            }
        }
        let chunk = &bytes[pos..end];
        let b64 = base64::engine::general_purpose::STANDARD.encode(chunk);
        words.push(format!("=?UTF-8?B?{}?=", b64));
        pos = end;
    }
    words.join("\r\n ")
}

/// RFC 2047 encode a header value containing an address with optional display name.
/// Encodes only the display name portion, preserving the email in angle brackets.
fn encode_address_header(address: &str) -> String {
    // Pattern: "Display Name <email@example.com>" or just "email@example.com"
    if let Some(bracket_start) = address.rfind('<') {
        let display_name = address[..bracket_start].trim();
        let email_part = &address[bracket_start..]; // includes <...>
        if display_name.is_empty() || display_name.is_ascii() {
            return address.to_string();
        }
        format!("{} {}", encode_rfc2047(display_name), email_part)
    } else {
        // Plain email address, no display name to encode
        address.to_string()
    }
}

/// Encode a comma-separated list of addresses.
fn encode_address_list(addresses: &[String]) -> String {
    addresses
        .iter()
        .map(|a| encode_address_header(a))
        .collect::<Vec<_>>()
        .join(", ")
}

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

    headers.push(format!("From: {}", encode_address_header(&params.from)));
    if !params.to.is_empty() {
        headers.push(format!("To: {}", encode_address_list(&params.to)));
    }
    if !params.cc.is_empty() {
        headers.push(format!("Cc: {}", encode_address_list(&params.cc)));
    }
    if !params.bcc.is_empty() {
        headers.push(format!("Bcc: {}", encode_address_list(&params.bcc)));
    }
    if let Some(ref reply_to) = params.reply_to {
        headers.push(format!("Reply-To: {}", encode_address_header(reply_to)));
    }
    if let Some(ref in_reply_to) = params.in_reply_to {
        headers.push(format!("In-Reply-To: {}", in_reply_to));
    }
    if let Some(ref references) = params.references {
        headers.push(format!("References: {}", references));
    }
    headers.push(format!("Subject: {}", encode_rfc2047(&params.subject)));
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
        "docx" => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()
        }
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
        "pptx" => {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string()
        }
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

    // ---------------------------------------------------------------
    // REQ-GMAIL-010 (Must): RFC 2047 header encoding
    // ---------------------------------------------------------------

    #[test]
    fn rfc2047_ascii_passthrough() {
        assert_eq!(encode_rfc2047("Hello World"), "Hello World");
    }

    #[test]
    fn rfc2047_emoji_subject() {
        let encoded = encode_rfc2047("\u{1F44B} Hello");
        assert!(encoded.starts_with("=?UTF-8?B?"));
        assert!(encoded.ends_with("?="));
        // Decode to verify round-trip
        let b64 = &encoded[10..encoded.len() - 2];
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .unwrap();
        assert_eq!(std::str::from_utf8(&decoded).unwrap(), "\u{1F44B} Hello");
    }

    #[test]
    fn rfc2047_em_dash() {
        let encoded = encode_rfc2047("Hello \u{2014} World");
        assert!(encoded.contains("=?UTF-8?B?"));
        // Verify round-trip by decoding all encoded words
        let decoded = decode_rfc2047_words(&encoded);
        assert_eq!(decoded, "Hello \u{2014} World");
    }

    #[test]
    fn rfc2047_mixed_emoji_and_dash() {
        let subject = "\u{1F44B} Hello from OMEGA \u{2014} Sent autonomously via omg-gog";
        let encoded = encode_rfc2047(subject);
        assert!(
            !encoded.contains('\u{1F44B}'),
            "raw emoji must not appear in header"
        );
        assert!(
            !encoded.contains('\u{2014}'),
            "raw em-dash must not appear in header"
        );
        let decoded = decode_rfc2047_words(&encoded);
        assert_eq!(decoded, subject);
    }

    #[test]
    fn mime_message_encodes_non_ascii_subject() {
        let params = MimeMessageParams {
            from: "sender@example.com".to_string(),
            to: vec!["to@example.com".to_string()],
            subject: "\u{1F44B} Test".to_string(),
            body_text: Some("body".to_string()),
            ..Default::default()
        };
        let mime = build_mime_message(&params);
        assert!(
            mime.contains("Subject: =?UTF-8?B?"),
            "subject must be RFC 2047 encoded"
        );
        assert!(
            !mime.contains("Subject: \u{1F44B}"),
            "raw emoji must not be in Subject header"
        );
    }

    #[test]
    fn address_header_encodes_non_ascii_display_name() {
        let encoded = encode_address_header("Ren\u{00e9} <rene@example.com>");
        assert!(encoded.contains("=?UTF-8?B?"));
        assert!(encoded.contains("<rene@example.com>"));
    }

    #[test]
    fn address_header_ascii_passthrough() {
        let addr = "John Doe <john@example.com>";
        assert_eq!(encode_address_header(addr), addr);
    }

    /// Helper: decode RFC 2047 encoded words back to a string for test verification.
    fn decode_rfc2047_words(input: &str) -> String {
        let mut result = String::new();
        let mut remaining = input;
        while let Some(start) = remaining.find("=?UTF-8?B?") {
            // Text before encoded word (skip folding whitespace between encoded words)
            let prefix = &remaining[..start];
            if !result.is_empty() && prefix.trim().is_empty() {
                // Skip whitespace between consecutive encoded words per RFC 2047
            } else {
                result.push_str(prefix);
            }
            let after_prefix = &remaining[start + 10..];
            if let Some(end) = after_prefix.find("?=") {
                let b64 = &after_prefix[..end];
                if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64) {
                    if let Ok(s) = std::str::from_utf8(&bytes) {
                        result.push_str(s);
                    }
                }
                remaining = &after_prefix[end + 2..];
            } else {
                break;
            }
        }
        result.push_str(remaining);
        result
    }
}
