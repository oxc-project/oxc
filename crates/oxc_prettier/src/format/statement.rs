use oxc_allocator::Vec;
use oxc_ast::ast::Statement;

use crate::{
    doc::{Doc, DocBuilder, Group},
    hardline, Prettier,
};
use oxc_span::GetSpan;

use super::Format;

pub(super) fn print_statement_sequence<'a>(
    p: &mut Prettier<'a>,
    stmts: &[Statement<'a>],
    remove_last_statement_hardline: bool,
) -> Vec<'a, Doc<'a>> {
    let mut parts = p.vec();
    let mut len = stmts.len();

    for (i, stmt) in stmts.iter().enumerate() {
        if i < len - 1 && matches!(stmts[i + 1], Statement::EmptyStatement(_)) {
            len -= 1;
        }

        if matches!(stmt, Statement::EmptyStatement(_)) {
            continue;
        }

        let mut docs = stmt.format(p);

        if remove_last_statement_hardline && i == len - 1 {
            match docs {
                Doc::Array(ref mut docs) | Doc::Group(Group { contents: ref mut docs, .. }) => {
                    if matches!(docs.last(), Some(Doc::Hardline)) {
                        docs.pop();
                    }
                }
                _ => {}
            }
        }

        parts.push(docs);

        if i < len - 1 {
            parts.push(hardline!());

            if p.is_next_line_empty(stmt.span().end) {
                parts.push(hardline!());
            }
        }
    }

    parts
}
