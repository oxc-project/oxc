use oxc_allocator::Allocator;
use oxc_diagnostics::Result;

use crate::{
    ast, body_parser::PatternParser, diagnostics, flag_parser::FlagsParser, options::ParserOptions,
    span::SpanFactory,
};

/// LiteralParser
pub struct Parser<'a> {
    allocator: &'a Allocator,
    source_text: &'a str,
    options: ParserOptions,
    span_factory: SpanFactory,
}

impl<'a> Parser<'a> {
    pub fn new(allocator: &'a Allocator, source_text: &'a str, options: ParserOptions) -> Self {
        Self {
            allocator,
            source_text,
            options,
            span_factory: SpanFactory::new(options.span_offset),
        }
    }

    pub fn parse(self) -> Result<ast::RegularExpression<'a>> {
        // Precheck if the source text is a valid regular expression literal
        // If valid, parse the pattern and flags with returned span offsets
        let (body_start_offset, body_end_offset, flag_start_offset) =
            parse_reg_exp_literal(self.source_text, &self.span_factory)?;

        // Parse flags first to know if unicode mode is enabled or not
        let flags = FlagsParser::new(
            self.allocator,
            &self.source_text[flag_start_offset..],
            #[allow(clippy::cast_possible_truncation)]
            self.options.with_span_offset(self.options.span_offset + flag_start_offset as u32),
        )
        .parse()?;

        // Then parse the pattern with the flags
        let pattern_options = match (flags.unicode, flags.unicode_sets) {
            (true, false) => self.options.with_unicode_mode(),
            (_, true) => self.options.with_unicode_sets_mode(),
            _ => self.options,
        };

        let pattern = PatternParser::new(
            self.allocator,
            &self.source_text[body_start_offset..body_end_offset],
            #[allow(clippy::cast_possible_truncation)]
            pattern_options.with_span_offset(self.options.span_offset + body_start_offset as u32),
        )
        .parse()?;

        Ok(ast::RegularExpression {
            span: self.span_factory.create(0, self.source_text.len()),
            pattern,
            flags,
        })
    }
}

/// Check passed source text is a valid regular expression literal.
/// ```
/// / RegularExpressionBody / RegularExpressionFlags
/// ```
/// Returns `(body_start_offset, body_end_offset, flag_start_offset)`.
fn parse_reg_exp_literal(
    source_text: &str,
    span_factory: &SpanFactory,
) -> Result<(usize, usize, usize)> {
    let mut offset = 0;
    let mut chars = source_text.chars().peekable();

    let Some('/') = chars.next() else {
        return Err(diagnostics::unexpected_literal_char(span_factory.create(offset, offset)));
    };
    offset += 1; // '/'

    let body_start = offset;

    let mut in_escape = false;
    let mut in_character_class = false;
    loop {
        match chars.peek() {
            // Line terminators are not allowed
            Some('\u{a}' | '\u{d}' | '\u{2028}' | '\u{2029}') | None => {
                return Err(diagnostics::unterminated_literal(
                    span_factory.create(body_start, offset),
                    if in_character_class { "character class" } else { "regular expression" },
                ));
            }
            Some(&ch) => {
                if in_escape {
                    in_escape = false;
                } else if ch == '\\' {
                    in_escape = true;
                } else if ch == '[' {
                    in_character_class = true;
                } else if ch == ']' {
                    in_character_class = false;
                } else if ch == '/' && !in_character_class
                    // `*` is not allowed as `RegularExpressionFirstChar`
                    || offset == body_start && ch == '*'
                {
                    break;
                }

                offset += ch.len_utf8();
            }
        }

        chars.next();
    }

    let Some('/') = chars.next() else {
        return Err(diagnostics::unexpected_literal_char(span_factory.create(offset, offset)));
    };
    let body_end = offset;

    if body_end == body_start {
        return Err(diagnostics::empty_literal(span_factory.create(0, body_end + 1)));
    }

    Ok((body_start, body_end, body_end + 1))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_valid_reg_exp_literal() {
        for literal_text in [
            "/(?:)/",
            "/abc/",
            "/abcd/igsmv",
            r"/\w+/u",
            r"/foo\/bar|baz/i",
            "/[a-z]/",
            "/正規表現/u",
            "/あっち👈🏻/i",
            "/👈🏻こっち/u",
        ] {
            let (body_start_offset, body_end_offset, flag_start_offset) =
                parse_reg_exp_literal(literal_text, &SpanFactory::new(0))
                    .unwrap_or_else(|_| panic!("{literal_text} should be parsed"));

            let body_text = &literal_text[body_start_offset..body_end_offset];
            let flag_text = &literal_text[flag_start_offset..];
            assert_eq!(format!("/{body_text}/{flag_text}",), literal_text);
        }
    }

    #[test]
    fn parse_invalid_reg_exp_literal() {
        for literal_text in
            ["", "foo", ":(", "a\nb", "/", "/x", "/y\nz/", "/1[\n]/", "//", "///", "/*abc/", "/\\/"]
        {
            assert!(parse_reg_exp_literal(literal_text, &SpanFactory::new(0)).is_err());
        }
    }
}
