#![allow(clippy::unused_self, clippy::fn_params_excessive_bools, clippy::too_many_arguments)]

use oxc_allocator::{Allocator, Vec};
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

    #[inline]
    pub fn new_vec<T>(self) -> Vec<'a, T> {
        Vec::new_in(self.allocator)
    }

    pub fn reg_exp_literal(
        self,
        span: Span,
        pattern: ast::Pattern<'a>,
        flags: ast::Flags,
    ) -> ast::RegExpLiteral<'a> {
        ast::RegExpLiteral { span, pattern, flags }
    }

    pub fn pattern(
        self,
        span: Span,
        alternatives: Vec<'a, ast::Alternative<'a>>,
    ) -> ast::Pattern<'a> {
        ast::Pattern { span, alternatives }
    }

    pub fn flags(
        self,
        span: Span,
        global: bool,
        ignore_case: bool,
        multiline: bool,
        unicode: bool,
        sticky: bool,
        dot_all: bool,
        has_indices: bool,
        unicode_sets: bool,
    ) -> ast::Flags {
        ast::Flags {
            span,
            global,
            ignore_case,
            multiline,
            unicode,
            sticky,
            dot_all,
            has_indices,
            unicode_sets,
        }
    }
}
