use oxc_allocator::Vec;
use oxc_ast::ast::Statement;
use oxc_span::GetSpan;

use crate::{hardline, ir::Doc, Format, Prettier};

pub fn print_statement_sequence<'a>(
    p: &mut Prettier<'a>,
    stmts: &[Statement<'a>],
) -> Vec<'a, Doc<'a>> {
    let mut parts = Vec::new_in(p.allocator);

    let last_statement_span =
        stmts.iter().rev().find(|s| !matches!(s, Statement::EmptyStatement(_))).map(GetSpan::span);

    for stmt in stmts {
        if matches!(stmt, Statement::EmptyStatement(_)) {
            continue;
        }

        parts.push(stmt.format(p));

        if Some(stmt.span()) != last_statement_span {
            parts.push(hardline!(p));

            if p.is_next_line_empty(stmt.span()) {
                parts.push(hardline!(p));
            }
        }
    }

    parts
}
