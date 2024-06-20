use oxc_allocator::Allocator;
use oxc_diagnostics::{OxcDiagnostic, Result};
use oxc_span::Span;

use crate::{ast, ast_builder::AstBuilder, state::ParserState};

#[derive(Clone, Copy)]
struct ParserOptions {
    /// The flag to disable Annex B syntax.
    /// Default: false
    strict: bool,
    /// ECMAScript version.
    /// - `2015` added `u` and `y` flags
    /// - `2018` added `s` flag, Named Capturing Group, Lookbehind Assertion,
    ///   and Unicode Property Escape
    /// - `2019`, `2020`, and `2021` added more valid Unicode Property Escapes
    /// - `2022` added `d` flag
    /// - `2023` added more valid Unicode Property Escapes
    /// - `2024` added `v` flag
    /// - `2025` added duplicate named capturing groups
    /// Default: 2025
    ecma_version: u32, // TODO: Enum?
}
impl Default for ParserOptions {
    fn default() -> Self {
        Self { strict: false, ecma_version: 2025 }
    }
}

pub struct Parser<'a> {
    allocator: &'a Allocator,
    source_text: &'a str,
    options: ParserOptions,
}

impl<'a> Parser<'a> {
    pub fn new(allocator: &'a Allocator, source_text: &'a str) -> Self {
        let options = ParserOptions::default();
        Self { allocator, source_text, options }
    }

    #[must_use]
    pub fn with_strict(mut self, is_strict: bool) -> Self {
        self.options.strict = is_strict;
        self
    }
    #[must_use]
    pub fn with_ecma_version(mut self, ecma_version: u32) -> Self {
        self.options.ecma_version = ecma_version;
        self
    }

    pub fn parse(self) -> Result<ast::RegExpLiteral<'a>> {
        let mut parser = ParserImpl::new(self.allocator, self.source_text, self.options);
        parser.parse()
    }
}

struct ParserImpl<'a> {
    // lexer,
    errors: Vec<OxcDiagnostic>,
    source_text: &'a str,
    state: ParserState,
    ast: AstBuilder<'a>,
}

impl<'a> ParserImpl<'a> {
    fn new(allocator: &'a Allocator, source_text: &'a str, options: ParserOptions) -> Self {
        Self {
            errors: vec![],
            source_text,
            state: ParserState::new(options.strict, options.ecma_version),
            ast: AstBuilder::new(allocator),
        }
    }

    fn parse(&mut self) -> Result<ast::RegExpLiteral<'a>> {
        let pattern = todo!("parse_pattern()");
        let flags = todo!("parse_flags()");

        // this._srcCtx = { source, start, end, kind: "literal" }
        // this._unicodeSetsMode = this._unicodeMode = this._nFlag = false
        // this.reset(source, start, end)

        // if (this.eat(SOLIDUS) && this.eatRegExpBody() && this.eat(SOLIDUS)) {
        //     const flagStart = this.index
        //     const unicode = source.includes("u", flagStart)
        //     const unicodeSets = source.includes("v", flagStart)
        //     this.validateFlagsInternal(source, flagStart, end)
        //     this.validatePatternInternal(source, start + 1, flagStart - 1, {
        //         unicode,
        //         unicodeSets,
        //     })
        // } else {
        //     const c = String.fromCodePoint(this.currentCodePoint)
        //     this.raise(`Unexpected character '${c}'`)
        // }

        let span = Span::new(0, self.source_text.len() as u32);
        Ok(self.ast.reg_exp_literal(span, pattern, flags))
    }

    fn parse_pattern(&mut self) -> Result<ast::Pattern<'a>> {
        todo!()
    }
    fn parse_flags(&mut self) -> Result<ast::Flags> {
        todo!()
    }
}
