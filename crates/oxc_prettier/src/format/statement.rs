use oxc_allocator::Vec;

use crate::{doc::Doc, Prettier};

use super::Format;

pub(super) fn print_statement_sequence<'a, F: Format<'a>>(
    p: &mut Prettier<'a>,
    stmts: &Vec<'a, F>,
) -> Vec<'a, Doc<'a>> {
    let mut parts = p.vec();
    for stmt in stmts {
        parts.push(stmt.format(p));
    }
    parts
}
