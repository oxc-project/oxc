use std::collections::HashSet;

use oxc_allocator::Vec;
use oxc_ast::ast::Decorator;

#[derive(Default)]
pub struct ParserState<'a> {
    pub not_parenthesized_arrow: HashSet<u32>,

    pub decorators: Option<Vec<'a, Decorator<'a>>>,
}

impl<'a> ParserState<'a> {
    pub fn consume_decorators(&mut self) -> Option<Vec<'a, Decorator<'a>>> {
        self.decorators.take()
    }
}
