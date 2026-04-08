//! Lightweight span-preserving CSS parser for linting.
//!
//! Parses plain CSS into a tree of stylesheet nodes with exact source spans.
//! Designed for lint rules, not for full CSS spec compliance. Handles:
//! - Qualified rules (selector + declaration block)
//! - At-rules (@media, @import, etc.)
//! - Declarations (property: value)
//! - Comments (/* ... */)

use oxc_span::Span;

// ── AST ──────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct CssStylesheet<'a> {
    pub rules: Vec<CssRule<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub enum CssRule<'a> {
    QualifiedRule(CssQualifiedRule<'a>),
    AtRule(CssAtRule<'a>),
    Comment(CssComment<'a>),
}

impl CssRule<'_> {
    pub fn span(&self) -> Span {
        match self {
            Self::QualifiedRule(r) => r.span,
            Self::AtRule(r) => r.span,
            Self::Comment(c) => c.span,
        }
    }
}

#[derive(Debug)]
pub struct CssQualifiedRule<'a> {
    /// The raw selector text (e.g., `.foo > bar`).
    pub selector: &'a str,
    pub selector_span: Span,
    pub block: CssDeclarationBlock<'a>,
    pub span: Span,
}

#[derive(Debug)]
pub struct CssAtRule<'a> {
    /// The at-rule name without `@` (e.g., `media`, `import`).
    pub name: &'a str,
    pub name_span: Span,
    /// The prelude text between the name and `{` or `;`.
    pub prelude: &'a str,
    pub prelude_span: Span,
    /// Body block, if present (at-rules like @import have no block).
    pub block: Option<CssRuleBlock<'a>>,
    pub span: Span,
}

/// A block that can contain nested rules (e.g., @media body).
#[derive(Debug)]
pub struct CssRuleBlock<'a> {
    pub rules: Vec<CssRule<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct CssDeclarationBlock<'a> {
    pub declarations: Vec<CssDeclaration<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct CssDeclaration<'a> {
    pub property: &'a str,
    pub property_span: Span,
    pub value: &'a str,
    pub value_span: Span,
    pub important: bool,
    pub span: Span,
}

#[derive(Debug)]
pub struct CssComment<'a> {
    pub text: &'a str,
    pub span: Span,
}

// ── Errors ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CssParseError {
    pub message: String,
    pub span: Span,
}

// ── Result ───────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct CssParseResult<'a> {
    pub stylesheet: Option<CssStylesheet<'a>>,
    pub errors: Vec<CssParseError>,
}

// ── Parser ───────────────────────────────────────────────────────────────────

pub fn parse_css(source: &str) -> CssParseResult<'_> {
    let mut parser = Parser::new(source);
    let rules = parser.parse_rule_list();
    let span = Span::new(0, source.len() as u32);
    CssParseResult { stylesheet: Some(CssStylesheet { rules, span }), errors: parser.errors }
}

struct Parser<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    errors: Vec<CssParseError>,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, bytes: source.as_bytes(), pos: 0, errors: Vec::new() }
    }

    fn parse_rule_list(&mut self) -> Vec<CssRule<'a>> {
        let mut rules = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos >= self.bytes.len() {
                break;
            }
            // Stop if we hit a closing brace (end of containing block)
            if self.peek() == Some(b'}') {
                break;
            }
            let before = self.pos;
            if let Some(rule) = self.parse_rule() {
                rules.push(rule);
            } else if self.pos == before {
                // No progress — skip one byte to avoid infinite loop
                self.pos += 1;
            }
        }
        rules
    }

    fn parse_rule(&mut self) -> Option<CssRule<'a>> {
        self.skip_whitespace();
        if self.pos >= self.bytes.len() {
            return None;
        }

        // Comment
        if self.starts_with(b"/*") {
            return self.parse_comment().map(CssRule::Comment);
        }

        // At-rule
        if self.peek() == Some(b'@') {
            return self.parse_at_rule().map(CssRule::AtRule);
        }

        // Closing brace (end of nested block)
        if self.peek() == Some(b'}') {
            return None;
        }

        // Qualified rule
        self.parse_qualified_rule().map(CssRule::QualifiedRule)
    }

    fn parse_comment(&mut self) -> Option<CssComment<'a>> {
        let start = self.pos;
        self.pos += 2; // skip /*
        let text_start = self.pos;
        loop {
            if self.pos + 1 >= self.bytes.len() {
                self.errors.push(CssParseError {
                    message: "Unterminated comment".to_string(),
                    span: Span::new(start as u32, self.bytes.len() as u32),
                });
                let text = &self.source[text_start..self.bytes.len()];
                self.pos = self.bytes.len();
                return Some(CssComment {
                    text,
                    span: Span::new(start as u32, self.bytes.len() as u32),
                });
            }
            if self.bytes[self.pos] == b'*' && self.bytes[self.pos + 1] == b'/' {
                let text = &self.source[text_start..self.pos];
                self.pos += 2; // skip */
                return Some(CssComment { text, span: Span::new(start as u32, self.pos as u32) });
            }
            self.pos += 1;
        }
    }

    fn parse_at_rule(&mut self) -> Option<CssAtRule<'a>> {
        let start = self.pos;
        self.pos += 1; // skip @

        // Parse name
        let name_start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_alphanumeric()
            || self.pos < self.bytes.len() && self.bytes[self.pos] == b'-'
        {
            self.pos += 1;
        }
        let name = &self.source[name_start..self.pos];
        let name_span = Span::new((start) as u32, self.pos as u32); // includes @

        // Parse prelude (everything until { or ;)
        self.skip_whitespace();
        let prelude_start = self.pos;
        let mut depth = 0u32;
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'{' if depth == 0 => break,
                b';' if depth == 0 => break,
                b'(' => {
                    depth += 1;
                    self.pos += 1;
                }
                b')' => {
                    depth = depth.saturating_sub(1);
                    self.pos += 1;
                }
                b'\'' | b'"' => self.skip_string(),
                _ => self.pos += 1,
            }
        }
        let prelude_end = self.pos;
        let prelude = self.source[prelude_start..prelude_end].trim();
        let prelude_span = Span::new(prelude_start as u32, prelude_end as u32);

        if self.pos >= self.bytes.len() {
            return Some(CssAtRule {
                name,
                name_span,
                prelude,
                prelude_span,
                block: None,
                span: Span::new(start as u32, self.pos as u32),
            });
        }

        // Semicolon-terminated at-rule (e.g., @import)
        if self.bytes[self.pos] == b';' {
            self.pos += 1;
            return Some(CssAtRule {
                name,
                name_span,
                prelude,
                prelude_span,
                block: None,
                span: Span::new(start as u32, self.pos as u32),
            });
        }

        // Block at-rule (e.g., @media)
        let block = self.parse_rule_block();
        Some(CssAtRule {
            name,
            name_span,
            prelude,
            prelude_span,
            block: Some(block),
            span: Span::new(start as u32, self.pos as u32),
        })
    }

    fn parse_rule_block(&mut self) -> CssRuleBlock<'a> {
        let block_start = self.pos;
        self.pos += 1; // skip {
        let rules = self.parse_rule_list();
        self.skip_whitespace();
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'}' {
            self.pos += 1;
        }
        CssRuleBlock { rules, span: Span::new(block_start as u32, self.pos as u32) }
    }

    fn parse_qualified_rule(&mut self) -> Option<CssQualifiedRule<'a>> {
        let start = self.pos;

        // Parse selector (everything until {)
        let selector_start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b'{' {
            match self.bytes[self.pos] {
                b'\'' | b'"' => self.skip_string(),
                _ => self.pos += 1,
            }
        }
        let selector_end = self.pos;
        let selector = self.source[selector_start..selector_end].trim();
        let selector_span = Span::new(selector_start as u32, selector_end as u32);

        if self.pos >= self.bytes.len() {
            self.errors.push(CssParseError {
                message: "Expected '{' after selector".to_string(),
                span: Span::new(start as u32, self.pos as u32),
            });
            return None;
        }

        let block = self.parse_declaration_block();

        Some(CssQualifiedRule {
            selector,
            selector_span,
            block,
            span: Span::new(start as u32, self.pos as u32),
        })
    }

    fn parse_declaration_block(&mut self) -> CssDeclarationBlock<'a> {
        let block_start = self.pos;
        self.pos += 1; // skip {
        let mut declarations = Vec::new();

        loop {
            self.skip_whitespace();
            if self.pos >= self.bytes.len() || self.bytes[self.pos] == b'}' {
                break;
            }

            // Skip comments inside blocks
            if self.starts_with(b"/*") {
                self.parse_comment();
                continue;
            }

            if let Some(decl) = self.parse_declaration() {
                declarations.push(decl);
            }
        }

        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'}' {
            self.pos += 1;
        }

        CssDeclarationBlock { declarations, span: Span::new(block_start as u32, self.pos as u32) }
    }

    fn parse_declaration(&mut self) -> Option<CssDeclaration<'a>> {
        let start = self.pos;

        // Parse property name
        let prop_start = self.pos;
        while self.pos < self.bytes.len()
            && self.bytes[self.pos] != b':'
            && self.bytes[self.pos] != b'}'
            && self.bytes[self.pos] != b';'
        {
            self.pos += 1;
        }
        let property = self.source[prop_start..self.pos].trim();
        let property_span = Span::new(prop_start as u32, self.pos as u32);

        if property.is_empty() || self.pos >= self.bytes.len() || self.bytes[self.pos] != b':' {
            // Not a valid declaration — skip to next ; or }
            while self.pos < self.bytes.len()
                && self.bytes[self.pos] != b';'
                && self.bytes[self.pos] != b'}'
            {
                self.pos += 1;
            }
            if self.pos < self.bytes.len() && self.bytes[self.pos] == b';' {
                self.pos += 1;
            }
            return None;
        }

        self.pos += 1; // skip :
        self.skip_whitespace();

        // Parse value (everything until ; or })
        let value_start = self.pos;
        let mut depth = 0u32;
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b';' if depth == 0 => break,
                b'}' if depth == 0 => break,
                b'(' => {
                    depth += 1;
                    self.pos += 1;
                }
                b')' => {
                    depth = depth.saturating_sub(1);
                    self.pos += 1;
                }
                b'\'' | b'"' => self.skip_string(),
                _ => self.pos += 1,
            }
        }
        let value_end = self.pos;
        let raw_value = self.source[value_start..value_end].trim();
        let value_span = Span::new(value_start as u32, value_end as u32);

        let important = raw_value.ends_with("!important") || raw_value.ends_with("! important");
        let value = if important {
            raw_value.trim_end_matches("!important").trim_end_matches("! important").trim()
        } else {
            raw_value
        };

        if self.pos < self.bytes.len() && self.bytes[self.pos] == b';' {
            self.pos += 1;
        }

        Some(CssDeclaration {
            property,
            property_span,
            value,
            value_span,
            important,
            span: Span::new(start as u32, self.pos as u32),
        })
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn skip_string(&mut self) {
        let quote = self.bytes[self.pos];
        self.pos += 1;
        while self.pos < self.bytes.len() {
            if self.bytes[self.pos] == b'\\' {
                self.pos += 2;
                continue;
            }
            if self.bytes[self.pos] == quote {
                self.pos += 1;
                return;
            }
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn starts_with(&self, prefix: &[u8]) -> bool {
        self.bytes[self.pos..].starts_with(prefix)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_rule() {
        let css = "body { color: red; }";
        let result = parse_css(css);
        assert!(result.errors.is_empty());
        let sheet = result.stylesheet.unwrap();
        assert_eq!(sheet.rules.len(), 1);
        if let CssRule::QualifiedRule(rule) = &sheet.rules[0] {
            assert_eq!(rule.selector, "body");
            assert_eq!(rule.block.declarations.len(), 1);
            assert_eq!(rule.block.declarations[0].property, "color");
            assert_eq!(rule.block.declarations[0].value, "red");
            assert!(!rule.block.declarations[0].important);
        } else {
            panic!("Expected qualified rule");
        }
    }

    #[test]
    fn parse_important() {
        let css = ".x { color: red !important; }";
        let result = parse_css(css);
        assert!(result.errors.is_empty());
        let sheet = result.stylesheet.unwrap();
        if let CssRule::QualifiedRule(rule) = &sheet.rules[0] {
            assert_eq!(rule.block.declarations[0].value, "red");
            assert!(rule.block.declarations[0].important);
        } else {
            panic!("Expected qualified rule");
        }
    }

    #[test]
    fn parse_multiple_declarations() {
        let css = "h1 { color: blue; font-size: 16px; margin: 0; }";
        let result = parse_css(css);
        assert!(result.errors.is_empty());
        let sheet = result.stylesheet.unwrap();
        if let CssRule::QualifiedRule(rule) = &sheet.rules[0] {
            assert_eq!(rule.block.declarations.len(), 3);
            assert_eq!(rule.block.declarations[0].property, "color");
            assert_eq!(rule.block.declarations[1].property, "font-size");
            assert_eq!(rule.block.declarations[2].property, "margin");
        } else {
            panic!("Expected qualified rule");
        }
    }

    #[test]
    fn parse_at_rule_import() {
        let css = "@import url('styles.css');";
        let result = parse_css(css);
        assert!(result.errors.is_empty());
        let sheet = result.stylesheet.unwrap();
        if let CssRule::AtRule(rule) = &sheet.rules[0] {
            assert_eq!(rule.name, "import");
            assert!(rule.prelude.contains("styles.css"));
            assert!(rule.block.is_none());
        } else {
            panic!("Expected at-rule");
        }
    }

    #[test]
    fn parse_at_rule_media() {
        let css = "@media (max-width: 768px) { body { color: red; } }";
        let result = parse_css(css);
        assert!(result.errors.is_empty());
        let sheet = result.stylesheet.unwrap();
        if let CssRule::AtRule(rule) = &sheet.rules[0] {
            assert_eq!(rule.name, "media");
            assert!(rule.block.is_some());
            let block = rule.block.as_ref().unwrap();
            assert_eq!(block.rules.len(), 1);
        } else {
            panic!("Expected at-rule");
        }
    }

    #[test]
    fn parse_comment() {
        let css = "/* hello */ body { color: red; }";
        let result = parse_css(css);
        assert!(result.errors.is_empty());
        let sheet = result.stylesheet.unwrap();
        assert_eq!(sheet.rules.len(), 2);
        if let CssRule::Comment(c) = &sheet.rules[0] {
            assert_eq!(c.text, " hello ");
        } else {
            panic!("Expected comment");
        }
    }

    #[test]
    fn parse_duplicate_properties() {
        let css = ".a { color: red; color: blue; }";
        let result = parse_css(css);
        assert!(result.errors.is_empty());
        let sheet = result.stylesheet.unwrap();
        if let CssRule::QualifiedRule(rule) = &sheet.rules[0] {
            assert_eq!(rule.block.declarations.len(), 2);
            assert_eq!(rule.block.declarations[0].property, "color");
            assert_eq!(rule.block.declarations[1].property, "color");
        } else {
            panic!("Expected qualified rule");
        }
    }

    #[test]
    fn parse_empty_block() {
        let css = ".empty { }";
        let result = parse_css(css);
        assert!(result.errors.is_empty());
        let sheet = result.stylesheet.unwrap();
        if let CssRule::QualifiedRule(rule) = &sheet.rules[0] {
            assert!(rule.block.declarations.is_empty());
        } else {
            panic!("Expected qualified rule");
        }
    }
}
