mod state;

use oxc_allocator::Allocator;
use oxc_diagnostics::{OxcDiagnostic, Result};

use crate::{
    ast,
    ast_builder::AstBuilder,
    parser::{
        body_parser::state::ParserState, options::ParserOptions, reader::Reader, span::SpanFactory,
    },
};

pub struct PatternParser<'a> {
    source_text: &'a str,
    // options: ParserOptions,
    ast: AstBuilder<'a>,
    span_factory: SpanFactory,
    reader: Reader<'a>,
    state: ParserState,
}

impl<'a> PatternParser<'a> {
    pub fn new(allocator: &'a Allocator, source_text: &'a str, options: ParserOptions) -> Self {
        Self {
            source_text,
            // options,
            ast: AstBuilder::new(allocator),
            span_factory: SpanFactory::new(options.span_offset),
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

        let pattern = self.ast.pattern(
            self.span_factory.new_with_offset(0, self.source_text.len()),
            self.ast.new_vec(),
        );
        // const unicode = source.includes("u", flagStart)
        // const unicodeSets = source.includes("v", flagStart)
        // const mode = this._parseFlagsOptionToMode(uFlagOrFlags, end);

        // this._unicodeMode = mode.unicodeMode;
        // this._nFlag = mode.nFlag;
        // this._unicodeSetsMode = mode.unicodeSetsMode;
        // this.reset(source, start, end);
        // this.consumePattern();

        // if (
        //   !this._nFlag &&
        //   !this._groupSpecifiers.isEmpty()
        // ) {
        //   this._nFlag = true;
        //   this.rewind(start);
        //   this.consumePattern();
        // }

        Ok(pattern)
    }
}
