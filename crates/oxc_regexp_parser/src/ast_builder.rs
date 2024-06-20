use oxc_allocator::Allocator;
use oxc_span::Span;

use crate::ast;

#[derive(Copy, Clone)]
pub struct AstBuilder<'a> {
    pub allocator: &'a Allocator,
}

impl<'a> AstBuilder<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator }
    }

    pub fn reg_exp_literal(
        &mut self,
        span: Span,
        pattern: ast::Pattern<'a>,
        flags: ast::Flags,
    ) -> ast::RegExpLiteral<'a> {
        ast::RegExpLiteral { span, pattern, flags }
    }

    pub fn flags(&mut self, span: Span) -> ast::Flags {
        ast::Flags {
            span,
            // TODO: From arguments
            dot_all: false,
            global: false,
            has_indices: false,
            ignore_case: false,
            multiline: false,
            sticky: false,
            unicode: false,
            unicode_sets: false,
        }
    }
}
