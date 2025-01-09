use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstKind};

use crate::{array, format::print::statement, hardline, indent, ir::Doc, text, Format, Prettier};

pub fn print_block<'a>(
    p: &mut Prettier<'a>,
    stmts: &[Statement<'a>],
    directives: Option<&[Directive<'a>]>,
) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);

    parts.push(text!("{"));
    if let Some(doc) = print_block_body(p, stmts, directives) {
        parts.push(indent!(p, [hardline!(p), doc]));
        parts.push(hardline!(p));
    } else {
        let parent = p.parent_kind();
        let parent_parent = p.parent_parent_kind();

        if (parent_parent.is_none()
            || parent_parent.is_some_and(|p| !matches!(p, AstKind::ObjectProperty(_))))
            && !(matches!(
                parent,
                AstKind::FunctionBody(_)
                    | AstKind::ArrowFunctionExpression(_)
                    | AstKind::ObjectExpression(_)
                    | AstKind::Function(_)
                    | AstKind::ForStatement(_)
                    | AstKind::WhileStatement(_)
                    | AstKind::DoWhileStatement(_)
                    | AstKind::MethodDefinition(_)
                    | AstKind::PropertyDefinition(_)
            ) || (matches!(parent, AstKind::CatchClause(_))
                && !matches!(p.parent_parent_kind(), Some(AstKind::TryStatement(stmt)) if stmt.finalizer.is_some()))
                || matches!(p.current_kind(), AstKind::StaticBlock(_)))
        {
            parts.push(hardline!(p));
        }
    }
    parts.push(text!("}"));

    array!(p, parts)
}

/// For `Program` only
pub fn print_block_body<'a>(
    p: &mut Prettier<'a>,
    stmts: &[Statement<'a>],
    directives: Option<&[Directive<'a>]>,
) -> Option<Doc<'a>> {
    let has_directives = directives.is_some_and(|directives| !directives.is_empty());
    let has_body = stmts.iter().any(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));

    if !has_body && !has_directives {
        return None;
    }

    let mut parts = Vec::new_in(p.allocator);

    if has_directives {
        if let Some(directives) = directives {
            // `statement::print_statement_sequence()` equivalent for directives
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

            if has_body {
                parts.push(hardline!(p));
                if p.is_next_line_empty(last_directive.span) {
                    parts.push(hardline!(p));
                }
            }
        }
    }

    if has_body {
        parts.extend(statement::print_statement_sequence(p, stmts));
    }

    Some(array!(p, parts))
}
