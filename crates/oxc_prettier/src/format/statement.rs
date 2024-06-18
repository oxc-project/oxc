use oxc_allocator::Vec;
use oxc_ast::ast::Statement;
use oxc_span::GetSpan;

use super::Format;
use crate::{
    doc::{Doc, DocBuilder},
    hardline, Prettier,
};

pub(super) fn print_statement_sequence<'a>(
    p: &mut Prettier<'a>,
    stmts: &[Statement<'a>],
    remove_last_statement_hardline: bool,
    skip_empty_statement: bool,
) -> Vec<'a, Doc<'a>> {
    let mut parts = p.vec();

    let last_statement_span =
        stmts.iter().rev().find(|s| !matches!(s, Statement::EmptyStatement(_))).map(GetSpan::span);

    for stmt in stmts {
        if matches!(stmt, Statement::EmptyStatement(_)) {
            continue;
        }

        parts.push(stmt.format(p));

        if Some(stmt.span()) != last_statement_span {
            parts.extend(hardline!());
            if p.is_next_line_empty(stmt.span()) {
                parts.extend(hardline!());
            }
        }
    }

    parts
}
