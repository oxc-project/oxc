use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{Format, Prettier, hardline, ir::Doc};

pub fn print_statement_sequence<'a>(
    p: &mut Prettier<'a>,
    stmts: &[Statement<'a>],
) -> Vec<'a, Doc<'a>> {
    let mut parts = Vec::new_in(p.allocator);

    let last_statement_span =
        stmts.iter().rev().find(|s| !matches!(s, Statement::EmptyStatement(_))).map(GetSpan::span);

    for stmt in stmts {
        // Skip printing `EmptyStatement` nodes to avoid leaving stray semicolons lying around
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

pub fn print_directives<'a>(
    p: &mut Prettier<'a>,
    directives: &[Directive<'a>],
    has_body_or_dangling_comments: bool,
) -> Vec<'a, Doc<'a>> {
    let mut parts = Vec::new_in(p.allocator);

    let mut last_directive = &directives[0];
    for (idx, directive) in directives.iter().enumerate() {
        parts.push(directive.format(p));
        if idx != directives.len() - 1 {
            parts.push(hardline!(p));
            if p.is_next_line_empty(directive.span) {
                parts.push(hardline!(p));
            }
        }

        last_directive = directive;
    }

    if has_body_or_dangling_comments {
        parts.push(hardline!(p));
        if p.is_next_line_empty(last_directive.span) {
            parts.push(hardline!(p));
        }
    }

    parts
}
