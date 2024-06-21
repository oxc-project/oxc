use oxc_allocator::Allocator;
use oxc_diagnostics::{OxcDiagnostic, Result};
use rustc_hash::FxHashSet;

use crate::{
    ast,
    ast_builder::AstBuilder,
    parser::{options::ParserOptions, span::SpanFactory},
};

pub struct FlagsParser<'a> {
    source_text: &'a str,
    options: ParserOptions,
    ast: AstBuilder<'a>,
    span_factory: SpanFactory,
}

impl<'a> FlagsParser<'a> {
    pub fn new(allocator: &'a Allocator, source_text: &'a str, options: ParserOptions) -> Self {
        Self {
            source_text,
            options,
            ast: AstBuilder::new(allocator),
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
                'u' if 2015 <= self.options.ecma_version => unicode = true,
                'y' if 2015 <= self.options.ecma_version => sticky = true,
                's' if 2018 <= self.options.ecma_version => dot_all = true,
                'd' if 2022 <= self.options.ecma_version => has_indices = true,
                'v' if 2024 <= self.options.ecma_version => unicode_sets = true,
                _ => return Err(OxcDiagnostic::error(format!("Invalid flag `{c}`"))),
            }
        }

        Ok(self.ast.flags(
            self.span_factory.new_with_offset(0, self.source_text.len()),
            global,
            ignore_case,
            multiline,
            unicode,
            sticky,
            dot_all,
            has_indices,
            unicode_sets,
        ))
    }
}
