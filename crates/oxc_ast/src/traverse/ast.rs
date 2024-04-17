#![allow(dead_code)] // just for now
use crate::ast;
use oxc_allocator::Allocator;

/// AST builder for creating AST nodes for traversable AST
pub struct TraversableAstBuilder<'a> {
    pub allocator: &'a Allocator,
}

impl<'a> TraversableAstBuilder<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator }
    }
}

pub use ast::traversable_stringliteral::TraversableStringLiteral;
