use oxc_allocator::{Allocator, Vec};
use oxc_diagnostics::{OxcDiagnostic, Result};

use crate::{
    ast,
    parser::{
        body_parser::{reader::Reader, state::ParserState},
        options::ParserOptions,
        span::SpanFactory,
    },
};

pub struct PatternParser<'a> {
    allocator: &'a Allocator,
    source_text: &'a str,
    span_factory: SpanFactory,
    reader: Reader<'a>,
    _state: ParserState,
}

impl<'a> PatternParser<'a> {
    pub fn new(allocator: &'a Allocator, source_text: &'a str, options: ParserOptions) -> Self {
        Self {
            allocator,
            source_text,
            span_factory: SpanFactory::new(options.span_offset),
            reader: Reader::new(source_text, options.unicode_mode),
            _state: ParserState,
        }
    }

    pub fn parse(&mut self) -> Result<ast::Pattern<'a>> {
        if self.source_text.is_empty() {
            return Err(OxcDiagnostic::error("Empty"));
        }

        // TODO: ...
        self.reader.position();
        self.reader.eat1('a');
        self.reader.eat2('a', 'b');
        self.reader.eat3('a', 'b', 'c');
        self.reader.rewind(0);
        self.reader.position();

        let pattern = ast::Pattern {
            span: self.span_factory.create(0, self.source_text.len()),
            alternatives: Vec::new_in(self.allocator),
        };

        Ok(pattern)
    }
}
