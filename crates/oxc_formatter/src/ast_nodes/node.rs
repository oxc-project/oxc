use core::fmt;
use std::ops::Deref;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_span::{GetSpan, Span};

use super::AstNodes;

pub struct AstNode<'a, T> {
    pub(super) inner: &'a T,
    pub parent: &'a AstNodes<'a>,
    pub(super) allocator: &'a Allocator,
    pub(super) following_span: Option<Span>,
}

impl<T: fmt::Debug> fmt::Debug for AstNode<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AstNode")
            .field("inner", &self.inner)
            .field("parent", &self.parent.debug_name())
            .field("following_span", &self.following_span)
            .finish()
    }
}

impl<'a, T> Deref for AstNode<'a, T> {
    type Target = T;

    fn deref(&self) -> &'a Self::Target {
        self.inner
    }
}

impl<'a, T> AsRef<T> for AstNode<'a, T> {
    fn as_ref(&self) -> &'a T {
        self.inner
    }
}

impl<'a, T> AstNode<'a, Option<T>> {
    pub fn as_ref(&self) -> Option<&'a AstNode<'a, T>> {
        self.allocator
            .alloc(self.inner.as_ref().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_span: self.following_span,
            }))
            .as_ref()
    }
}

impl<T: GetSpan> GetSpan for AstNode<'_, T> {
    fn span(&self) -> Span {
        self.inner.span()
    }
}

impl<T: GetSpan> GetSpan for &AstNode<'_, T> {
    fn span(&self) -> Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, Program<'a>> {
    pub fn new(inner: &'a Program<'a>, parent: &'a AstNodes<'a>, allocator: &'a Allocator) -> Self {
        AstNode { inner, parent, allocator, following_span: None }
    }
}
