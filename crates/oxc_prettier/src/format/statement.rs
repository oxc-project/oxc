use oxc_allocator::Vec;

use crate::{doc::Doc, Prettier};

use super::Format;

pub(super) fn print_statement_sequence<'a, F: Format<'a>>(
    p: &mut Prettier<'a>,
    stmts: &Vec<'a, F>,
    remove_last_statement_hardline: bool,
) -> Vec<'a, Doc<'a>> {
    let mut parts = p.vec();
    for (index, stmt) in stmts.iter().enumerate() {
        let mut docs = stmt.format(p);

        if remove_last_statement_hardline && index == stmts.len() - 1 {
            match docs {
                Doc::Array(ref mut docs) | Doc::Group(ref mut docs) => {
                    if matches!(docs.last(), Some(Doc::Hardline)) {
                        docs.pop();
                    }
                }
                _ => {}
            }
        }

        parts.push(docs);
    }
    parts
}
