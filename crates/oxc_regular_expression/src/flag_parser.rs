use oxc_allocator::Allocator;
use oxc_diagnostics::Result;
use rustc_hash::FxHashSet;

use crate::{ast, diagnostics, options::ParserOptions, span::SpanFactory};

pub struct FlagsParser<'a> {
    source_text: &'a str,
    // options: ParserOptions,
    span_factory: SpanFactory,
}

impl<'a> FlagsParser<'a> {
    pub fn new(_allocator: &'a Allocator, source_text: &'a str, options: ParserOptions) -> Self {
        Self {
            source_text,
            // options,
            span_factory: SpanFactory::new(options.span_offset),
        }
    }

    pub fn parse(&mut self) -> Result<ast::Flags> {
        let span = self.span_factory.create(0, self.source_text.len());
        let mut global = false;
        let mut ignore_case = false;
        let mut multiline = false;
        let mut unicode = false;
        let mut sticky = false;
        let mut dot_all = false;
        let mut has_indices = false;
        let mut unicode_sets = false;

        let mut existing_flags = FxHashSet::default();
        for (idx, c) in self.source_text.char_indices() {
            if !existing_flags.insert(c) {
                return Err(diagnostics::duplicated_flag(self.span_factory.create(idx, idx)));
            }

            match c {
                'g' => global = true,
                'i' => ignore_case = true,
                'm' => multiline = true,
                'u' => unicode = true,
                'y' => sticky = true,
                's' => dot_all = true,
                'd' => has_indices = true,
                'v' => unicode_sets = true,
                _ => return Err(diagnostics::unknown_flag(self.span_factory.create(idx, idx))),
            }
        }

        if unicode && unicode_sets {
            return Err(diagnostics::invalid_unicode_flags(span));
        }

        Ok(ast::Flags {
            span,
            global,
            ignore_case,
            multiline,
            unicode,
            sticky,
            dot_all,
            has_indices,
            unicode_sets,
        })
    }
}
