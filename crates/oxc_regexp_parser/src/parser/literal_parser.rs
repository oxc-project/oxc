use oxc_allocator::Allocator;
use oxc_diagnostics::{OxcDiagnostic, Result};
use oxc_syntax::identifier::is_line_terminator;

use crate::{
    ast,
    parser::{
        body_parser::PatternParser, flag_parser::FlagsParser, options::ParserOptions,
        reader::Reader, span::SpanFactory,
    },
};

// LiteralParser
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

    pub fn parse(self) -> Result<ast::RegExpLiteral<'a>> {
        // Precheck if the source text is a valid regular expression literal
        let flag_start_idx = is_valid_reg_exp_literal(self.source_text)?;

        // If valid, parse flags first
        let flags = FlagsParser::new(
            self.allocator,
            &self.source_text[flag_start_idx..],
            #[allow(clippy::cast_possible_truncation)]
            self.options.with_span_offset(self.options.span_offset + flag_start_idx as u32),
        )
        .parse()?;

        // Then parse the pattern with the flags
        let unicode_mode = flags.unicode || flags.unicode_sets;
        let unicode_sets_mode = flags.unicode_sets;

        let pattern = PatternParser::new(
            self.allocator,
            &self.source_text[1..flag_start_idx - 1],
            self.options
                .with_span_offset(self.options.span_offset + 1)
                .with_modes(unicode_mode, unicode_sets_mode),
        )
        .parse()?;

        Ok(ast::RegExpLiteral {
            span: self.span_factory.create(0, self.source_text.len()),
            pattern,
            flags,
        })
    }
}

/// ```
/// / RegularExpressionBody / RegularExpressionFlags
/// ```
/// <https://tc39.es/ecma262/#sec-literals-regular-expression-literals>
fn is_valid_reg_exp_literal(source_text: &str) -> Result<usize> {
    let mut reader = Reader::new(
        source_text,
        false, // We don't care Unicode or UTF-16 here
    );

    if !reader.eat('/') {
        return Err(OxcDiagnostic::error("Unexpected character"));
    };

    let body_start_idx = reader.idx;
    let mut in_escape = false;
    let mut in_character_class = false;
    loop {
        match reader.c1 {
            None => {
                let kind =
                    if in_character_class { "character class" } else { "regular expression" };
                return Err(OxcDiagnostic::error(format!("Unterminated {kind}")));
            }
            Some(c) if is_line_terminator(c) => {
                let kind =
                    if in_character_class { "character class" } else { "regular expression" };
                return Err(OxcDiagnostic::error(format!("Unterminated {kind}")));
            }
            Some(c) => {
                if in_escape {
                    in_escape = false;
                } else if c == '\\' {
                    in_escape = true;
                } else if c == '[' {
                    in_character_class = true;
                } else if c == ']' {
                    in_character_class = false;
                } else if c == '/' && !in_character_class
                    // `*` is not allowed as `RegularExpressionFirstChar`
                    // https://tc39.es/ecma262/#prod-RegularExpressionBody
                    || c == '*' && reader.idx == body_start_idx
                {
                    break;
                }

                reader.advance();
            }
        }
    }

    if reader.idx == body_start_idx {
        return Err(OxcDiagnostic::error("Empty"));
    }

    if !reader.eat('/') {
        return Err(OxcDiagnostic::error("Unexpected character"));
    };

    // flag start
    Ok(reader.idx)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_valid_reg_exp_literal() {
        assert_eq!(is_valid_reg_exp_literal("/abc/").unwrap(), 5);
        assert_eq!(is_valid_reg_exp_literal("/abcd/i").unwrap(), 6);
        assert_eq!(is_valid_reg_exp_literal("/正規表現/u").unwrap(), 6);
        assert_eq!(is_valid_reg_exp_literal("/👈🏻/i").unwrap(), 4);

        assert!(is_valid_reg_exp_literal("/").is_err());
        assert!(is_valid_reg_exp_literal("//").is_err());
        assert!(is_valid_reg_exp_literal("///").is_err());
        assert!(is_valid_reg_exp_literal("/*abc/").is_err());
        assert!(is_valid_reg_exp_literal("/\\/").is_err());
    }
}
