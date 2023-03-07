use std::collections::HashSet;

use oxc_allocator::{Allocator, Vec};
use oxc_ast::ast::Decorator;

pub struct ParserState<'a> {
    allocator: &'a Allocator,

    pub not_parenthesized_arrow: HashSet<u32>,

    pub decorators: Vec<'a, Decorator<'a>>,
}

impl<'a> ParserState<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            not_parenthesized_arrow: HashSet::new(),
            decorators: Vec::new_in(allocator.allocator()),
        }
    }

    pub fn consume_decorators(&mut self) -> Vec<'a, Decorator<'a>> {
        std::mem::replace(&mut self.decorators, Vec::new_in(self.allocator.allocator()))
    }
}
