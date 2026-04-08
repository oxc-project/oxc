//! Lightweight span-preserving JSON parser for linting.
//!
//! Unlike `serde_json`, this parser preserves the exact source spans for every
//! key, value, property, object, and array in the JSON document. This enables
//! precise diagnostics and autofixes for JSON lint rules.

use oxc_span::Span;

// ── AST ──────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum JsonValue<'a> {
    Object(JsonObject<'a>),
    Array(JsonArray<'a>),
    /// (unescaped value, span of the full quoted string including quotes)
    String(&'a str, Span),
    /// (raw text, span)
    Number(&'a str, Span),
    Boolean(bool, Span),
    Null(Span),
}

impl JsonValue<'_> {
    pub fn span(&self) -> Span {
        match self {
            Self::Object(obj) => obj.span,
            Self::Array(arr) => arr.span,
            Self::String(_, span)
            | Self::Number(_, span)
            | Self::Boolean(_, span)
            | Self::Null(span) => *span,
        }
    }

    pub fn as_object(&self) -> Option<&JsonObject<'_>> {
        if let Self::Object(obj) = self { Some(obj) } else { None }
    }

    pub fn as_array(&self) -> Option<&JsonArray<'_>> {
        if let Self::Array(arr) = self { Some(arr) } else { None }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(s, _) = self { Some(s) } else { None }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Boolean(b, _) = self { Some(*b) } else { None }
    }
}

#[derive(Debug)]
pub struct JsonProperty<'a> {
    /// The key string (without quotes).
    pub key: &'a str,
    /// Span of the full quoted key including quotes.
    pub key_span: Span,
    /// The value.
    pub value: JsonValue<'a>,
    /// Full span from key start to value end.
    pub span: Span,
}

#[derive(Debug)]
pub struct JsonObject<'a> {
    pub properties: Vec<JsonProperty<'a>>,
    pub span: Span,
}

impl<'a> JsonObject<'a> {
    pub fn get(&self, key: &str) -> Option<&JsonValue<'a>> {
        self.properties.iter().find(|p| p.key == key).map(|p| &p.value)
    }

    pub fn get_property(&self, key: &str) -> Option<&JsonProperty<'a>> {
        self.properties.iter().find(|p| p.key == key)
    }
}

#[derive(Debug)]
pub struct JsonArray<'a> {
    pub elements: Vec<JsonValue<'a>>,
    pub span: Span,
}

// ── Errors ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct JsonParseError {
    pub message: String,
    pub span: Span,
}

// ── Result ───────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct JsonParseResult<'a> {
    pub root: Option<JsonValue<'a>>,
    pub errors: Vec<JsonParseError>,
}

// ── Parser ───────────────────────────────────────────────────────────────────

pub fn parse_json(source: &str) -> JsonParseResult<'_> {
    let mut parser = Parser::new(source);
    let root = parser.parse_value();
    parser.skip_whitespace();
    if parser.pos < parser.source.len() && parser.errors.is_empty() {
        parser.errors.push(JsonParseError {
            message: "Unexpected content after JSON value".to_string(),
            span: Span::new(parser.pos as u32, (parser.pos + 1) as u32),
        });
    }
    JsonParseResult { root, errors: parser.errors }
}

struct Parser<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    errors: Vec<JsonParseError>,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, bytes: source.as_bytes(), pos: 0, errors: Vec::new() }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let b = self.bytes.get(self.pos).copied()?;
        self.pos += 1;
        Some(b)
    }

    fn expect(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            self.errors.push(JsonParseError {
                message: format!(
                    "Expected '{}', found {}",
                    expected as char,
                    self.peek().map_or("end of input".to_string(), |b| format!("'{}'", b as char))
                ),
                span: Span::new(self.pos as u32, (self.pos + 1).min(self.source.len()) as u32),
            });
            false
        }
    }

    fn parse_value(&mut self) -> Option<JsonValue<'a>> {
        self.skip_whitespace();
        match self.peek()? {
            b'{' => self.parse_object().map(JsonValue::Object),
            b'[' => self.parse_array().map(JsonValue::Array),
            b'"' => self.parse_string().map(|(s, span)| JsonValue::String(s, span)),
            b't' | b'f' => self.parse_boolean(),
            b'n' => self.parse_null(),
            b'-' | b'0'..=b'9' => self.parse_number(),
            c => {
                self.errors.push(JsonParseError {
                    message: format!("Unexpected character '{}'", c as char),
                    span: Span::new(self.pos as u32, (self.pos + 1) as u32),
                });
                None
            }
        }
    }

    fn parse_object(&mut self) -> Option<JsonObject<'a>> {
        let start = self.pos;
        self.advance(); // skip '{'
        self.skip_whitespace();

        let mut properties = Vec::new();

        if self.peek() == Some(b'}') {
            self.advance();
            return Some(JsonObject { properties, span: Span::new(start as u32, self.pos as u32) });
        }

        loop {
            self.skip_whitespace();
            let prop_start = self.pos;

            // Parse key
            if self.peek() != Some(b'"') {
                self.errors.push(JsonParseError {
                    message: "Expected string key".to_string(),
                    span: Span::new(self.pos as u32, (self.pos + 1).min(self.source.len()) as u32),
                });
                return None;
            }
            let (key, key_span) = self.parse_string()?;

            self.skip_whitespace();
            if !self.expect(b':') {
                return None;
            }

            // Parse value
            let value = self.parse_value()?;
            let prop_end = self.pos;

            properties.push(JsonProperty {
                key,
                key_span,
                span: Span::new(prop_start as u32, prop_end as u32),
                value,
            });

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.advance();
                }
                Some(b'}') => {
                    self.advance();
                    break;
                }
                _ => {
                    self.errors.push(JsonParseError {
                        message: "Expected ',' or '}'".to_string(),
                        span: Span::new(
                            self.pos as u32,
                            (self.pos + 1).min(self.source.len()) as u32,
                        ),
                    });
                    return None;
                }
            }
        }

        Some(JsonObject { properties, span: Span::new(start as u32, self.pos as u32) })
    }

    fn parse_array(&mut self) -> Option<JsonArray<'a>> {
        let start = self.pos;
        self.advance(); // skip '['
        self.skip_whitespace();

        let mut elements = Vec::new();

        if self.peek() == Some(b']') {
            self.advance();
            return Some(JsonArray { elements, span: Span::new(start as u32, self.pos as u32) });
        }

        loop {
            let value = self.parse_value()?;
            elements.push(value);

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.advance();
                }
                Some(b']') => {
                    self.advance();
                    break;
                }
                _ => {
                    self.errors.push(JsonParseError {
                        message: "Expected ',' or ']'".to_string(),
                        span: Span::new(
                            self.pos as u32,
                            (self.pos + 1).min(self.source.len()) as u32,
                        ),
                    });
                    return None;
                }
            }
        }

        Some(JsonArray { elements, span: Span::new(start as u32, self.pos as u32) })
    }

    fn parse_string(&mut self) -> Option<(&'a str, Span)> {
        let start = self.pos;
        self.advance(); // skip opening '"'

        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'"' => {
                    let value = &self.source[start + 1..self.pos];
                    self.pos += 1; // skip closing '"'
                    let span = Span::new(start as u32, self.pos as u32);
                    return Some((value, span));
                }
                b'\\' => {
                    self.pos += 1; // skip backslash
                    if self.pos < self.bytes.len() {
                        // Skip the escaped character
                        if self.bytes[self.pos] == b'u' {
                            // \uXXXX — skip 4 hex digits
                            self.pos += 1;
                            for _ in 0..4 {
                                if self.pos < self.bytes.len() {
                                    self.pos += 1;
                                }
                            }
                        } else {
                            self.pos += 1;
                        }
                    }
                }
                _ => {
                    self.pos += 1;
                }
            }
        }

        self.errors.push(JsonParseError {
            message: "Unterminated string".to_string(),
            span: Span::new(start as u32, self.source.len() as u32),
        });
        None
    }

    fn parse_number(&mut self) -> Option<JsonValue<'a>> {
        let start = self.pos;

        if self.peek() == Some(b'-') {
            self.advance();
        }

        // Integer part
        match self.peek() {
            Some(b'0') => {
                self.advance();
            }
            Some(b'1'..=b'9') => {
                self.advance();
                while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
            }
            _ => {
                self.errors.push(JsonParseError {
                    message: "Invalid number".to_string(),
                    span: Span::new(start as u32, (self.pos + 1).min(self.source.len()) as u32),
                });
                return None;
            }
        }

        // Fraction
        if self.peek() == Some(b'.') {
            self.advance();
            if !self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.errors.push(JsonParseError {
                    message: "Expected digit after decimal point".to_string(),
                    span: Span::new(self.pos as u32, (self.pos + 1).min(self.source.len()) as u32),
                });
                return None;
            }
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }

        // Exponent
        if self.peek().is_some_and(|b| b == b'e' || b == b'E') {
            self.advance();
            if self.peek().is_some_and(|b| b == b'+' || b == b'-') {
                self.advance();
            }
            if !self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.errors.push(JsonParseError {
                    message: "Expected digit in exponent".to_string(),
                    span: Span::new(self.pos as u32, (self.pos + 1).min(self.source.len()) as u32),
                });
                return None;
            }
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }

        let raw = &self.source[start..self.pos];
        let span = Span::new(start as u32, self.pos as u32);
        Some(JsonValue::Number(raw, span))
    }

    fn parse_boolean(&mut self) -> Option<JsonValue<'a>> {
        let start = self.pos;
        if self.source[self.pos..].starts_with("true") {
            self.pos += 4;
            Some(JsonValue::Boolean(true, Span::new(start as u32, self.pos as u32)))
        } else if self.source[self.pos..].starts_with("false") {
            self.pos += 5;
            Some(JsonValue::Boolean(false, Span::new(start as u32, self.pos as u32)))
        } else {
            self.errors.push(JsonParseError {
                message: "Invalid boolean".to_string(),
                span: Span::new(start as u32, (start + 1) as u32),
            });
            None
        }
    }

    fn parse_null(&mut self) -> Option<JsonValue<'a>> {
        let start = self.pos;
        if self.source[self.pos..].starts_with("null") {
            self.pos += 4;
            Some(JsonValue::Null(Span::new(start as u32, self.pos as u32)))
        } else {
            self.errors.push(JsonParseError {
                message: "Invalid null".to_string(),
                span: Span::new(start as u32, (start + 1) as u32),
            });
            None
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_object() {
        let result = parse_json("{}");
        assert!(result.errors.is_empty());
        let root = result.root.unwrap();
        let obj = root.as_object().unwrap();
        assert!(obj.properties.is_empty());
        assert_eq!(obj.span, Span::new(0, 2));
    }

    #[test]
    fn test_simple_object() {
        let src = r#"{"name": "test", "version": 1}"#;
        let result = parse_json(src);
        assert!(result.errors.is_empty());
        let root = result.root.unwrap();
        let obj = root.as_object().unwrap();
        assert_eq!(obj.properties.len(), 2);
        assert_eq!(obj.properties[0].key, "name");
        assert_eq!(obj.properties[0].key_span, Span::new(1, 7)); // "name"
        assert_eq!(obj.properties[0].value.as_str(), Some("test"));
        assert_eq!(obj.properties[1].key, "version");
    }

    #[test]
    fn test_nested_object() {
        let src = r#"{"a": {"b": true}}"#;
        let result = parse_json(src);
        assert!(result.errors.is_empty());
        let root = result.root.unwrap();
        let outer = root.as_object().unwrap();
        let inner = outer.properties[0].value.as_object().unwrap();
        assert_eq!(inner.properties[0].key, "b");
        assert_eq!(inner.properties[0].value.as_bool(), Some(true));
    }

    #[test]
    fn test_array() {
        let src = r#"[1, "two", true, null]"#;
        let result = parse_json(src);
        assert!(result.errors.is_empty());
        let root = result.root.unwrap();
        let arr = root.as_array().unwrap();
        assert_eq!(arr.elements.len(), 4);
    }

    #[test]
    fn test_number_spans() {
        let src = "42";
        let result = parse_json(src);
        assert!(result.errors.is_empty());
        if let JsonValue::Number(raw, span) = result.root.unwrap() {
            assert_eq!(raw, "42");
            assert_eq!(span, Span::new(0, 2));
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn test_string_with_escapes() {
        let src = r#""hello \"world\"""#;
        let result = parse_json(src);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_parse_error() {
        let result = parse_json("{invalid}");
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_duplicate_keys() {
        let src = r#"{"a": 1, "a": 2}"#;
        let result = parse_json(src);
        assert!(result.errors.is_empty()); // parser allows duplicate keys
        let root = result.root.unwrap();
        let obj = root.as_object().unwrap();
        assert_eq!(obj.properties.len(), 2);
        assert_eq!(obj.properties[0].key, "a");
        assert_eq!(obj.properties[1].key, "a");
    }

    #[test]
    fn test_empty_input() {
        let result = parse_json("");
        assert!(result.root.is_none());
    }

    #[test]
    fn test_whitespace_only() {
        let result = parse_json("  \n\t  ");
        assert!(result.root.is_none());
    }

    #[test]
    fn test_negative_number() {
        let result = parse_json("-3.14e2");
        assert!(result.errors.is_empty());
        if let JsonValue::Number(raw, span) = result.root.unwrap() {
            assert_eq!(raw, "-3.14e2");
            assert_eq!(span, Span::new(0, 7));
        } else {
            panic!("expected number");
        }
    }
}
