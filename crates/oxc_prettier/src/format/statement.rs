use oxc_allocator::Vec;
use oxc_ast::ast::Statement;

use crate::{
    doc::{Doc, DocBuilder, Group, Line},
    hardline, Prettier,
};
use oxc_span::GetSpan;

use super::Format;

pub(super) fn print_statement_sequence<'a>(
    p: &mut Prettier<'a>,
    stmts: &[Statement<'a>],
    remove_last_statement_hardline: bool,
    skip_empty_statement: bool,
) -> Vec<'a, Doc<'a>> {
    let mut parts = p.vec();
    let len = stmts.len();

    for (i, stmt) in stmts.iter().enumerate() {
        if skip_empty_statement && matches!(stmt, Statement::EmptyStatement(_)) {
            continue;
        }

        let mut docs = stmt.format(p);

        if remove_last_statement_hardline && i == len - 1 {
            match docs {
                Doc::Array(ref mut docs) | Doc::Group(Group { contents: ref mut docs, .. }) => {
                    if docs.last().is_some_and(
                        |doc| matches!(doc, Doc::Line(line) if *line == Line::hardline()),
                    ) {
                        docs.pop();
                    }
                }
                _ => {}
            }
        }

        parts.push(docs);

        if i < len - 1 && !matches!(stmts[i + 1], Statement::EmptyStatement(_)) {
            parts.extend(hardline!());

            if p.is_next_line_empty(stmt.span()) {
                parts.extend(hardline!());
            }
        }
    }

    parts
}
