//! Sed-like expression parser for document find/replace operations.
//!
//! Supports sed s-command syntax: s/find/replace/flags
//! Flags: g (global/match all), i (case-insensitive)

/// A parsed sed expression.
#[derive(Debug, Clone, PartialEq)]
pub struct SedExpression {
    pub find: String,
    pub replace: String,
    pub global: bool,
    pub case_insensitive: bool,
}

/// Parse a sed-like expression string.
///
/// Accepted formats:
/// - `s/find/replace/`
/// - `s/find/replace/g`
/// - `s/find/replace/gi`
/// - `s/find/replace/ig`
/// - `s|find|replace|flags` (any delimiter)
///
/// The delimiter is the character immediately after 's'.
pub fn parse_sed_expression(expr: &str) -> Result<SedExpression, String> {
    let expr = expr.trim();
    if expr.is_empty() {
        return Err("empty sed expression".to_string());
    }

    if !expr.starts_with('s') {
        return Err(format!(
            "sed expression must start with 's', got: '{}'",
            expr.chars().next().unwrap_or(' ')
        ));
    }

    if expr.len() < 2 {
        return Err("sed expression too short".to_string());
    }

    let delimiter = expr.chars().nth(1).unwrap();
    let delim_byte_len = delimiter.len_utf8();
    let rest = &expr[1 + delim_byte_len..];

    // Split by delimiter
    let parts: Vec<&str> = rest.split(delimiter).collect();

    if parts.len() < 2 {
        return Err(format!(
            "sed expression must have at least find and replace parts separated by '{}'",
            delimiter
        ));
    }

    let find = parts[0].to_string();
    let replace = parts[1].to_string();

    if find.is_empty() {
        return Err("find pattern cannot be empty".to_string());
    }

    // Parse flags (if present)
    let flags = if parts.len() >= 3 { parts[2] } else { "" };
    let global = flags.contains('g');
    let case_insensitive = flags.contains('i');

    Ok(SedExpression {
        find,
        replace,
        global,
        case_insensitive,
    })
}

/// Parse multiple sed expressions from lines (as from a file).
pub fn parse_sed_file(content: &str) -> Result<Vec<SedExpression>, String> {
    let mut exprs = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue; // skip empty lines and comments
        }
        match parse_sed_expression(trimmed) {
            Ok(expr) => exprs.push(expr),
            Err(e) => return Err(format!("line {}: {}", line_num + 1, e)),
        }
    }
    Ok(exprs)
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_parse_basic() {
        let expr = parse_sed_expression("s/hello/world/").unwrap();
        assert_eq!(expr.find, "hello");
        assert_eq!(expr.replace, "world");
        assert!(!expr.global);
        assert!(!expr.case_insensitive);
    }

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_parse_global() {
        let expr = parse_sed_expression("s/hello/world/g").unwrap();
        assert!(expr.global);
        assert!(!expr.case_insensitive);
    }

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_parse_case_insensitive() {
        let expr = parse_sed_expression("s/hello/world/i").unwrap();
        assert!(!expr.global);
        assert!(expr.case_insensitive);
    }

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_parse_both_flags() {
        let expr = parse_sed_expression("s/hello/world/gi").unwrap();
        assert!(expr.global);
        assert!(expr.case_insensitive);
    }

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_parse_flags_reversed() {
        let expr = parse_sed_expression("s/hello/world/ig").unwrap();
        assert!(expr.global);
        assert!(expr.case_insensitive);
    }

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_custom_delimiter() {
        let expr = parse_sed_expression("s|hello|world|g").unwrap();
        assert_eq!(expr.find, "hello");
        assert_eq!(expr.replace, "world");
        assert!(expr.global);
    }

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_hash_delimiter() {
        let expr = parse_sed_expression("s#old#new#").unwrap();
        assert_eq!(expr.find, "old");
        assert_eq!(expr.replace, "new");
    }

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_empty_replace() {
        let expr = parse_sed_expression("s/hello//").unwrap();
        assert_eq!(expr.find, "hello");
        assert_eq!(expr.replace, "");
    }

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_no_trailing_delimiter() {
        let expr = parse_sed_expression("s/hello/world").unwrap();
        assert_eq!(expr.find, "hello");
        assert_eq!(expr.replace, "world");
    }

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_empty_find_error() {
        let result = parse_sed_expression("s//world/");
        assert!(result.is_err());
    }

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_empty_input_error() {
        assert!(parse_sed_expression("").is_err());
    }

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_no_s_prefix_error() {
        assert!(parse_sed_expression("/hello/world/").is_err());
    }

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_parse_file_basic() {
        let content = "s/hello/world/g\ns/foo/bar/\n";
        let exprs = parse_sed_file(content).unwrap();
        assert_eq!(exprs.len(), 2);
        assert_eq!(exprs[0].find, "hello");
        assert_eq!(exprs[1].find, "foo");
    }

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_parse_file_skips_comments() {
        let content = "# This is a comment\ns/a/b/\n\n# Another comment\ns/c/d/\n";
        let exprs = parse_sed_file(content).unwrap();
        assert_eq!(exprs.len(), 2);
    }

    // REQ-DOCS-014
    #[test]
    fn req_docs_014_parse_file_empty() {
        let content = "# Just comments\n\n";
        let exprs = parse_sed_file(content).unwrap();
        assert!(exprs.is_empty());
    }

    // REQ-DOCS-014
    #[test]
    fn test_multibyte_delimiter() {
        // This should not panic
        let result = parse_sed_expression("s\u{00e9}find\u{00e9}replace\u{00e9}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        assert_eq!(expr.find, "find");
        assert_eq!(expr.replace, "replace");
    }
}
