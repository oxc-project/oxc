mod state;

use oxc_allocator::Allocator;
use oxc_diagnostics::{OxcDiagnostic, Result};
use oxc_span::Span;

use crate::{
    ast,
    ast_builder::AstBuilder,
    parser::{body_parser::state::ParserState, options::ParserOptions, reader::Reader},
};

pub struct PatternParser<'a> {
    source_text: &'a str,
    ast: AstBuilder<'a>,
    options: ParserOptions,
    reader: Reader<'a>,
    state: ParserState,
}

impl<'a> PatternParser<'a> {
    pub fn new(allocator: &'a Allocator, source_text: &'a str, options: ParserOptions) -> Self {
        Self {
            source_text,
            ast: AstBuilder::new(allocator),
            options,
            reader: Reader::new(),
            state: ParserState::default(),
        }
    }

    pub fn parse(&mut self) -> Result<ast::Pattern<'a>> {
        if self.source_text.is_empty() {
            return Err(OxcDiagnostic::error("Empty"));
        }

        let (start, end) = (0, self.source_text.len());
        self.reader.reset(self.source_text, start, end, self.state.unicode_mode);

        // const unicode = source.includes("u", flagStart)
        // const unicodeSets = source.includes("v", flagStart)
        // this.validatePatternInternal(source, start + 1, flagStart - 1, {
        //   unicode,
        //   unicodeSets,
        // })
        let pattern = self.ast.pattern(
            #[allow(clippy::cast_possible_truncation)]
            Span::new(0, self.source_text.len() as u32),
            self.ast.new_vec(),
        );

        Ok(pattern)
    }
}
