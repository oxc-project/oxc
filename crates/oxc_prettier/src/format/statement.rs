use oxc_allocator::Vec;

use crate::{doc::Doc, hardline, Prettier};
use oxc_span::GetSpan;

use super::Format;
use crate::util::is_next_line_empty;

pub(super) fn print_statement_sequence<'a, F: Format<'a> + GetSpan>(
    p: &mut Prettier<'a>,
    stmts: &Vec<'a, F>,
    remove_last_statement_hardline: bool,
) -> Vec<'a, Doc<'a>> {
    let mut parts = p.vec();

    for (i, stmt) in stmts.iter().enumerate() {
        let mut docs = stmt.format(p);

        if remove_last_statement_hardline && i == stmts.len() - 1 {
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

        if i < stmts.len() - 1 {
            parts.push(hardline!());

            if is_next_line_empty(p.source_text, stmt.span()) {
                parts.push(hardline!());
            }
        }
    }

    parts
}
