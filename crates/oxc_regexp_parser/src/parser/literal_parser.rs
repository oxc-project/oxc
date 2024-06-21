use oxc_allocator::Allocator;
use oxc_diagnostics::{OxcDiagnostic, Result};
use oxc_syntax::identifier::is_line_terminator;

use crate::{
    ast,
    ast_builder::AstBuilder,
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
    ast: AstBuilder<'a>,
    span_factory: SpanFactory,
}

impl<'a> Parser<'a> {
    pub fn new(allocator: &'a Allocator, source_text: &'a str, options: ParserOptions) -> Self {
        Self {
            allocator,
            source_text,
            options,
            ast: AstBuilder::new(allocator),
            span_factory: SpanFactory::new(options.span_offset),
        }
    }

    // NOTE: Should return `ParserReturn { (empty)literal, errors }`?
    pub fn parse(self) -> Result<ast::RegExpLiteral<'a>> {
        let flag_start_idx = is_valid_reg_exp_literal(self.source_text)?;

        let flags = FlagsParser::new(
            self.allocator,
            &self.source_text[flag_start_idx..],
            #[allow(clippy::cast_possible_truncation)]
            self.options.with_span_offset(self.options.span_offset + flag_start_idx as u32),
        )
        .parse()?;

        // TODO: Pass these flags to `PatternParser`
        flags.unicode;
        flags.unicode_sets;
        let pattern = PatternParser::new(
            self.allocator,
            &self.source_text[1..flag_start_idx - 1],
            self.options.with_span_offset(self.options.span_offset + 1),
        )
        .parse()?;

        let span = self.span_factory.new_with_offset(0, self.source_text.len());
        Ok(self.ast.reg_exp_literal(span, pattern, flags))
    }
}

/// ```
/// / RegularExpressionBody / RegularExpressionFlags
/// ```
/// https://tc39.es/ecma262/#sec-literals-regular-expression-literals
fn is_valid_reg_exp_literal(source_text: &str) -> Result<usize> {
    let mut reader = Reader::new();
    reader.reset(source_text, 0, source_text.len(), false);

    if !reader.eat('/') {
        return Err(OxcDiagnostic::error("Unexpected character"));
    };

    // For `RegularExpressionFirstChar` check
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
