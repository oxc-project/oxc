use oxc_allocator::{Allocator, Box, String, Vec};
use oxc_span::{Atom, GetSpan, SourceType, Span};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

/// AST builder for creating AST nodes
pub struct AstBuilder<'a> {
    pub allocator: &'a Allocator,
}

impl<'a> AstBuilder<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator }
    }

    #[inline]
    pub fn alloc<T>(&self, value: T) -> Box<'a, T> {
        Box(self.allocator.alloc(value))
    }

    #[inline]
    pub fn new_vec<T>(&self) -> Vec<'a, T> {
        Vec::new_in(self.allocator)
    }

    #[inline]
    pub fn new_vec_with_capacity<T>(&self, capacity: usize) -> Vec<'a, T> {
        Vec::with_capacity_in(capacity, self.allocator)
    }

    #[inline]
    pub fn new_vec_single<T>(&self, value: T) -> Vec<'a, T> {
        let mut vec = self.new_vec_with_capacity(1);
        vec.push(value);
        vec
    }

    #[inline]
    pub fn new_str(&self, value: &str) -> &'a str {
        String::from_str_in(value, self.allocator).into_bump_str()
    }

    pub fn copy<T>(&self, src: &T) -> T {
        // SAFETY:
        // This should be safe as long as `src` is an reference from the allocator.
        // But honestly, I'm not really sure if this is safe.
        unsafe { std::mem::transmute_copy(src) }
    }

    pub fn alternative(&mut self, span: Span, elements: Vec<'a, Element<'a>>) -> Branch<'a> {
        Branch::Alternative(self.alloc(Alternative { span, elements }))
    }

    pub fn capturing_group(
        &mut self,
        span: Span,
        name: Option<Atom>,
        alternatives: Vec<'a, Alternative<'a>>,
        references: Vec<'a, Backreference<'a>>,
    ) -> Branch<'a> {
        Branch::CapturingGroup(self.alloc(CapturingGroup { span, name, alternatives, references }))
    }

    pub fn reg_exp_literal(
        &mut self,
        span: Span,
        flags: Flags,
        pattern: Pattern<'a>,
    ) -> RegExpLiteral<'a> {
        RegExpLiteral { span, pattern, flags }
    }
}
