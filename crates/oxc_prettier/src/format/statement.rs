use oxc_allocator::Vec;

use crate::{doc::Doc, Prettier};

use super::Format;

impl<'a> Prettier<'a> {
    pub(super) fn print_statement_sequence<F: Format<'a>>(
        &mut self,
        stmts: &Vec<'a, F>,
    ) -> Vec<'a, Doc<'a>> {
        let mut parts = self.vec();
        for stmt in stmts {
            parts.push(stmt.format(self));
        }
        parts
    }
}
