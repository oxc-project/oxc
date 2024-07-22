use oxc_allocator::{Allocator, Box, Vec};
use oxc_diagnostics::{OxcDiagnostic, Result};

use crate::{
    ast,
    body_parser::{reader::Reader, state::State, unicode},
    options::ParserOptions,
    span::SpanFactory,
};

pub struct PatternParser<'a> {
    pub(super) allocator: &'a Allocator,
    pub(super) source_text: &'a str,
    pub(super) span_factory: SpanFactory,
    pub(super) reader: Reader<'a>,
    pub(super) state: State,
}

impl<'a> PatternParser<'a> {
    pub fn new(allocator: &'a Allocator, source_text: &'a str, options: ParserOptions) -> Self {
        let unicode_mode = options.unicode_flag || options.unicode_sets_flag;
        let unicode_sets_mode = options.unicode_sets_flag;

        Self {
            allocator,
            source_text,
            span_factory: SpanFactory::new(options.span_offset),
            reader: Reader::new(source_text, unicode_mode),
            state: State::new(unicode_mode, unicode_sets_mode),
        }
    }

    pub fn parse(&mut self) -> Result<ast::Pattern<'a>> {
        if self.source_text.is_empty() {
            return Err(OxcDiagnostic::error("Empty"));
        }

        Ok(ast::Pattern {
            span: self.span_factory.create(0, self.source_text.len()),
            body: ast::Disjunction {
                span: self.span_factory.create(0, self.source_text.len()),
                body: Vec::new_in(self.allocator),
            },
        })
    }
}
