#![allow(clippy::missing_errors_doc)]

mod ast_impl;
mod diagnostics;
mod options;
mod parser;
mod surrogate_pair;

mod generated {
    mod derive_clone_in;
    mod derive_content_eq;
    mod derive_content_hash;
    #[cfg(feature = "serialize")]
    mod derive_estree;
}

pub mod ast;
pub use crate::{
    ast_impl::visit,
    options::Options,
    parser::{ConstructorParser, LiteralParser},
};

// LEGACY APIS TO BE REMOVED SOON! ============================================

#[derive(Clone, Copy, Debug, Default)]
pub struct ParserOptions {
    pub span_offset: u32,
    pub unicode_mode: bool,
    pub unicode_sets_mode: bool,
    pub parse_string_literal: bool,
}

impl ParserOptions {
    #[must_use]
    pub fn with_span_offset(self, span_offset: u32) -> Self {
        ParserOptions { span_offset, ..self }
    }

    #[must_use]
    pub fn with_flags(self, flags: &str) -> Self {
        let (mut unicode_mode, mut unicode_sets_mode) = (false, false);
        for ch in flags.chars() {
            if ch == 'u' {
                unicode_mode = true;
            }
            if ch == 'v' {
                unicode_mode = true;
                unicode_sets_mode = true;
            }
        }

        ParserOptions { unicode_mode, unicode_sets_mode, ..self }
    }

    #[must_use]
    pub fn with_parse_string_literal(self) -> Self {
        ParserOptions { parse_string_literal: true, ..self }
    }
}

pub struct Parser<'a> {
    allocator: &'a oxc_allocator::Allocator,
    source_text: &'a str,
    options: ParserOptions,
}

impl<'a> Parser<'a> {
    pub fn new(
        allocator: &'a oxc_allocator::Allocator,
        source_text: &'a str,
        options: ParserOptions,
    ) -> Self {
        Self { allocator, source_text, options }
    }

    pub fn parse(self) -> oxc_diagnostics::Result<crate::ast::Pattern<'a>> {
        let ParserOptions { unicode_mode, unicode_sets_mode, span_offset, parse_string_literal } =
            self.options;

        let options = Options {
            pattern_span_offset: span_offset,
            flags_span_offset: 0, // Never be used
        };

        if parse_string_literal {
            #[allow(clippy::match_same_arms)]
            let flags_text = match (unicode_mode, unicode_sets_mode) {
                (true, false) => Some("'u'"),
                (false, true) => Some("'v'"),
                (true, true) => Some("'v'"), // Do not validate this here
                (false, false) => None,
            };
            ConstructorParser::new(self.allocator, self.source_text, flags_text, options).parse()
        } else {
            #[allow(clippy::match_same_arms)]
            let flags_text = match (unicode_mode, unicode_sets_mode) {
                (true, false) => Some("u"),
                (false, true) => Some("v"),
                (true, true) => Some("v"), // Do not validate this here
                (false, false) => None,
            };
            LiteralParser::new(self.allocator, self.source_text, flags_text, options).parse()
        }
    }
}
