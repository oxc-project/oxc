use std::collections::HashSet;

use oxc_allocator::Vec;
use oxc_ast::ast::Decorator;

pub struct ParserState<'a> {
    pub not_parenthesized_arrow: HashSet<u32>,

    pub decorators: Vec<'a, Decorator<'a>>,
}

impl<'a> ParserState<'a> {
    pub fn new(decorators: Vec<'a, Decorator<'a>>) -> Self {
        Self { not_parenthesized_arrow: HashSet::new(), decorators }
    }

    pub fn consume_decorators(&mut self) -> Vec<'a, Decorator<'a>> {
        todo!()
    }
}
