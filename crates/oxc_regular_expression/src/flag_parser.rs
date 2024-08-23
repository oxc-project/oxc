use oxc_allocator::Allocator;
use oxc_diagnostics::{OxcDiagnostic, Result};
use rustc_hash::FxHashSet;

use crate::{ast, options::ParserOptions, span::SpanFactory};

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
        let mut existing_flags = FxHashSet::default();

        let mut global = false;
        let mut ignore_case = false;
        let mut multiline = false;
        let mut unicode = false;
        let mut sticky = false;
        let mut dot_all = false;
        let mut has_indices = false;
        let mut unicode_sets = false;

        for c in self.source_text.chars() {
            if !existing_flags.insert(c) {
                return Err(OxcDiagnostic::error(format!("Duplicated flag `{c}`")));
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
                _ => return Err(OxcDiagnostic::error(format!("Invalid flag `{c}`"))),
            }
        }

        // This should be a `SyntaxError`
        if unicode && unicode_sets {
            return Err(OxcDiagnostic::error("Invalid regular expression flags"));
        }

        Ok(ast::Flags {
            span: self.span_factory.create(0, self.source_text.len()),
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
