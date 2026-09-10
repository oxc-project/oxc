//! Island locator for Ember `.gjs`/`.gts`: the `<template>` … `</template>` tags that are
//! the non-JavaScript part of an otherwise ordinary module.
//!
//! The grammar is narrow, as verified against `content-tag` (the reference preprocessor):
//!
//! - the open tag is exactly `<template>`; attributes and self-closing forms are errors
//! - the first `</template>` always closes it, even inside an attribute, a Handlebars
//!   string, or a comment, so the body needs no Handlebars knowledge to delimit
//! - tags do not nest
//!
//! What the scan does need is JavaScript lexical state, so that a `<template>` inside a
//! string, comment, template literal, or regular expression is not mistaken for a tag.
//!
//! # Accuracy and the caller's obligation
//!
//! The regular-expression heuristic below is the usual one and is not exact. It does not
//! need to be, because the caller must verify every island against the parsed AST and skip
//! formatting the file when one does not line up. Both directions of a wrong scan then
//! degrade to "file left alone" rather than to corrupted output:
//!
//! - a region found where there is none fails that check, because the span lands inside a
//!   regex or string token instead of on a placeholder
//! - a real tag missed leaves `<template>` in the text handed to the parser, which fails

use oxc_formatter::OpaqueRegion;
use oxc_span::Span;

/// Language identifier the dispatcher routes a template tag to.
/// The whole tag is handed over, wrapper included; see `embed::dispatcher`.
const LANGUAGE: &str = "ember-template-tag";

const OPEN: &[u8] = b"<template>";
const CLOSE: &[u8] = b"</template>";

/// Locate every template tag in `source`, in order and non-overlapping.
pub fn locate(source: &str) -> Vec<OpaqueRegion<'static>> {
    scan(source)
        .into_iter()
        .map(|region| OpaqueRegion { span: region, language: LANGUAGE })
        .collect()
}

/// The spans of every template tag, each covering `<template>` through `</template>`.
fn scan(source: &str) -> Vec<Span> {
    Scanner {
        bytes: source.as_bytes(),
        pos: 0,
        expression_allowed: true,
        brace_depth: 0,
        template_depths: vec![],
    }
    .run()
}

struct Scanner<'s> {
    bytes: &'s [u8],
    pos: usize,
    /// Whether the position can begin an expression.
    ///
    /// Drives both halves of the same ambiguity: `/` is a regex here and a division
    /// otherwise, and `<` opens a template tag here and is a comparison otherwise
    /// (`a <template> b` is two comparisons on a variable named `template`).
    expression_allowed: bool,
    /// Depth of `{` nesting within the current context.
    brace_depth: u32,
    /// Brace depth saved on entering each `${`, innermost last. Empty means "not inside a
    /// template literal substitution".
    template_depths: Vec<u32>,
}

impl Scanner<'_> {
    fn run(mut self) -> Vec<Span> {
        let mut regions = Vec::new();

        while self.pos < self.bytes.len() {
            let byte = self.bytes[self.pos];
            match byte {
                b'"' | b'\'' => self.skip_quoted(byte),
                b'`' => self.skip_template_literal(),
                b'/' => match self.peek(1) {
                    Some(b'/') => self.skip_line_comment(),
                    Some(b'*') => self.skip_block_comment(),
                    _ => {
                        if self.expression_allowed {
                            self.skip_regex();
                        } else {
                            self.pos += 1;
                        }
                        self.expression_allowed = true;
                    }
                },
                b'{' => {
                    self.brace_depth += 1;
                    self.pos += 1;
                    self.expression_allowed = true;
                }
                b'}' => {
                    self.pos += 1;
                    self.expression_allowed = true;
                    // A `}` at depth zero closes a `${` and returns to its template literal.
                    if self.brace_depth == 0
                        && let Some(saved) = self.template_depths.pop()
                    {
                        self.brace_depth = saved;
                        self.resume_template_literal();
                    } else {
                        self.brace_depth = self.brace_depth.saturating_sub(1);
                    }
                }
                b'<' if self.expression_allowed && self.starts_with(OPEN) => {
                    if let Some(span) = self.take_template_region() {
                        regions.push(span);
                    } else {
                        // No closing tag: not a template tag after all. Step past the `<`
                        // so a later tag in the same file is still found.
                        self.pos += 1;
                    }
                }
                _ => self.consume_code_byte(byte),
            }
        }

        regions
    }

    /// Consume one byte of ordinary code, updating [`Self::expression_allowed`].
    fn consume_code_byte(&mut self, byte: u8) {
        if byte.is_ascii_whitespace() {
            self.pos += 1;
            return;
        }
        if is_identifier_byte(byte) {
            let start = self.pos;
            while self.pos < self.bytes.len() && is_identifier_byte(self.bytes[self.pos]) {
                self.pos += 1;
            }
            // A word ends an expression unless it is a keyword that introduces one.
            let word = &self.bytes[start..self.pos];
            self.expression_allowed =
                word.first().is_some_and(u8::is_ascii_digit) || EXPRESSION_KEYWORDS.contains(&word);
            return;
        }
        // `)` and `]` close an expression; every other punctuation opens one.
        // `if (x) /re/.test(y)` is the known miss, and is safe (see module docs).
        self.expression_allowed = !matches!(byte, b')' | b']');
        self.pos += 1;
    }

    /// At an `OPEN` tag: take the region through the first `CLOSE`, if there is one.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "oxc spans are u32; a source that long is rejected long before this"
    )]
    fn take_template_region(&mut self) -> Option<Span> {
        let outer_start = self.pos;
        let inner_end =
            find(&self.bytes[outer_start + OPEN.len()..], CLOSE)? + outer_start + OPEN.len();
        let outer_end = inner_end + CLOSE.len();

        self.pos = outer_end;
        // A tag is a value, but the common next position is a fresh statement or class
        // member (semicolons are inserted), where another tag may legally begin. Staying
        // permissive can only over-report, which the caller's AST check rejects, whereas
        // being strict would silently miss the second tag in a class body.
        self.expression_allowed = true;
        Some(Span::new(outer_start as u32, outer_end as u32))
    }

    fn skip_quoted(&mut self, quote: u8) {
        self.pos += 1;
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'\\' => self.pos += 2,
                b if b == quote => {
                    self.pos += 1;
                    break;
                }
                // An unterminated string cannot span a line; bail so the rest still scans.
                b'\n' => break,
                _ => self.pos += 1,
            }
        }
        self.expression_allowed = false;
    }

    fn skip_template_literal(&mut self) {
        self.pos += 1;
        self.resume_template_literal();
    }

    /// Scan template-literal text from [`Self::pos`], stopping at the closing backtick or
    /// at a `${`, which hands control back to the code loop.
    fn resume_template_literal(&mut self) {
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'\\' => self.pos += 2,
                b'`' => {
                    self.pos += 1;
                    self.expression_allowed = false;
                    return;
                }
                b'$' if self.peek(1) == Some(b'{') => {
                    self.template_depths.push(self.brace_depth);
                    self.brace_depth = 0;
                    self.pos += 2;
                    self.expression_allowed = true;
                    return;
                }
                _ => self.pos += 1,
            }
        }
        self.expression_allowed = false;
    }

    fn skip_line_comment(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
            self.pos += 1;
        }
    }

    fn skip_block_comment(&mut self) {
        self.pos += 2;
        while self.pos < self.bytes.len() {
            if self.bytes[self.pos] == b'*' && self.peek(1) == Some(b'/') {
                self.pos += 2;
                return;
            }
            self.pos += 1;
        }
    }

    /// Skip a regex literal, honouring escapes and `[...]` classes (where `/` is literal).
    fn skip_regex(&mut self) {
        self.pos += 1;
        let mut in_class = false;
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'\\' => self.pos += 2,
                b'[' => {
                    in_class = true;
                    self.pos += 1;
                }
                b']' => {
                    in_class = false;
                    self.pos += 1;
                }
                b'/' if !in_class => {
                    self.pos += 1;
                    // Flags.
                    while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_alphabetic()
                    {
                        self.pos += 1;
                    }
                    return;
                }
                // An unterminated regex cannot span a line.
                b'\n' => return,
                _ => self.pos += 1,
            }
        }
    }

    fn peek(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos + offset).copied()
    }

    fn starts_with(&self, needle: &[u8]) -> bool {
        self.bytes[self.pos..].starts_with(needle)
    }
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$' || byte >= 0x80
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

/// Words after which a `/` is a regex and a `<` can open a template tag.
/// Every other word ends an expression.
static EXPRESSION_KEYWORDS: &[&[u8]] = &[
    b"await",
    b"case",
    b"default",
    b"delete",
    b"do",
    b"else",
    b"in",
    b"instanceof",
    b"new",
    b"of",
    b"return",
    b"typeof",
    b"void",
    b"yield",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The scanned tags as `(whole tag, body)` text pairs, which read better in assertions
    /// than raw offsets.
    fn scan_text(source: &str) -> Vec<(&str, &str)> {
        scan(source)
            .into_iter()
            .map(|span| {
                let tag = &source[span.start as usize..span.end as usize];
                let body = &tag[OPEN.len()..tag.len() - CLOSE.len()];
                (tag, body)
            })
            .collect()
    }

    #[test]
    fn finds_tags_in_every_position() {
        // Top level, class body, and expression position: the three places a tag is legal.
        assert_eq!(scan_text("<template>x</template>"), [("<template>x</template>", "x")]);
        assert_eq!(
            scan_text("class A { <template>x</template> }"),
            [("<template>x</template>", "x")]
        );
        assert_eq!(
            scan_text("const a = <template>x</template>;"),
            [("<template>x</template>", "x")]
        );
        assert_eq!(
            scan_text("export default <template>x</template>;"),
            [("<template>x</template>", "x")]
        );
    }

    #[test]
    fn finds_multiple_tags() {
        let source = "<template>a</template>\nconst z = 1;\n<template>b</template>";
        assert_eq!(
            scan_text(source),
            [("<template>a</template>", "a"), ("<template>b</template>", "b")]
        );
    }

    #[test]
    fn preserves_body_verbatim() {
        // The body is Handlebars and must not be interpreted while delimiting.
        let source = "<template>\n  <div class=\"a\">{{x}}</div>\n</template>";
        assert_eq!(scan_text(source)[0].1, "\n  <div class=\"a\">{{x}}</div>\n");
    }

    #[test]
    fn first_close_tag_wins() {
        // Verified against content-tag: a `</template>` inside the body still closes,
        // which is why the scan needs no Handlebars knowledge.
        let source = r#"<template><div title="</template>"></div></template>"#;
        assert_eq!(scan_text(source)[0].0, r#"<template><div title="</template>"#);
    }

    #[test]
    fn ignores_tags_in_js_strings_and_comments() {
        for source in [
            r#"const s = "<template>x</template>";"#,
            r"const s = '<template>x</template>';",
            "// <template>x</template>\nconst a = 1;",
            "/* <template>x</template> */\nconst a = 1;",
            "/*\n * <template>x</template>\n */\nconst a = 1;",
        ] {
            assert!(scan(source).is_empty(), "should find no tag in: {source}");
        }
    }

    #[test]
    fn ignores_tags_in_template_literals() {
        for source in [
            "const s = `<template>x</template>`;",
            "const s = `${a}<template>x</template>`;",
            "const s = `${ `${ `<template>x</template>` }` }`;",
            // Braces inside a substitution must not end it early.
            "const s = `${ { a: 1 } }<template>x</template>`;",
        ] {
            assert!(scan(source).is_empty(), "should find no tag in: {source}");
        }
    }

    #[test]
    fn finds_tags_after_template_literals() {
        // The substitution must hand control back so a later real tag is still found.
        let source = "const s = `${ a }`;\n<template>x</template>";
        assert_eq!(scan_text(source), [("<template>x</template>", "x")]);
    }

    #[test]
    fn ignores_tags_in_regex_literals() {
        let source = r"const r = /<template>x<\/template>/;";
        assert!(scan(source).is_empty());
        // A `/` in a character class is literal, so the regex does not end early.
        let source = r"const r = /[/]<template>/;";
        assert!(scan(source).is_empty());
    }

    #[test]
    fn treats_division_as_division() {
        // `a / b` is division, so the `<template>` after it is a real tag rather than
        // regex contents.
        let source = "const q = a / b;\n<template>x</template>";
        assert_eq!(scan_text(source), [("<template>x</template>", "x")]);
    }

    #[test]
    fn ignores_comparison_against_a_variable_named_template() {
        // `a <template> b` is two comparisons, not a tag, because `<` there cannot begin
        // an expression. Nothing closes it either, so it is doubly rejected.
        assert!(scan("const c = a <template> b;").is_empty());
    }

    #[test]
    fn ignores_an_unclosed_tag() {
        assert!(scan("<template>x").is_empty());
    }

    #[test]
    fn handles_empty_and_trivial_sources() {
        assert!(scan("").is_empty());
        assert_eq!(scan_text("<template></template>"), [("<template></template>", "")]);
    }

    #[test]
    fn tags_are_ordered_and_disjoint() {
        // Two tags in one class body: accepted by content-tag, and the second is only
        // found if a tag leaves the scanner able to start another expression.
        let source = "class A { <template>a</template>\n<template>b</template> }";
        let spans = scan(source);
        assert_eq!(spans.len(), 2);
        assert!(spans[0].end <= spans[1].start);
    }

    #[test]
    fn locate_reports_islands_with_the_dispatch_language() {
        let islands = locate("const a = 1;\n<template>x</template>");
        assert_eq!(islands.len(), 1);
        assert_eq!(islands[0].language, LANGUAGE);
        assert_eq!(islands[0].span, scan("const a = 1;\n<template>x</template>")[0]);
    }

    #[test]
    fn does_not_panic_on_multibyte_or_truncated_input() {
        // Byte scanning must not split a multi-byte character or run off the end.
        for source in [
            "const s = \"日本語\";\n<template>絵文字 🎉</template>",
            "<template>\u{2028}x</template>",
            "const s = \"unterminated",
            "/* unterminated",
            "`unterminated ${",
            "/unterminated",
            "<template",
        ] {
            let _ = scan(source);
        }
    }
}
