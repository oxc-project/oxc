use oxc_allocator::Vec;
use oxc_ast::{AstKind, ast::*};

use crate::{Format, Prettier, array, format::print::statement, hardline, indent, ir::Doc, text};

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
        let current_kind = p.current_kind();
        let parent_kind = p.parent_kind();
        let parent_parent_kind = p.parent_parent_kind();

        if !(matches!(
            parent_kind,
            AstKind::ArrowFunctionExpression(_)
                | AstKind::Function(_)
                | AstKind::MethodDefinition(_)
                | AstKind::ObjectProperty(_) // For object method
                | AstKind::ForStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::DoWhileStatement(_)
                | AstKind::TSModuleDeclaration(_)
        ) || (matches!(parent_kind, AstKind::CatchClause(_))
            && !matches!(parent_parent_kind, Some(AstKind::TryStatement(stmt)) if stmt.finalizer.is_some()))
            || matches!(current_kind, AstKind::StaticBlock(_)))
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
    let has_directives = directives.is_some_and(|d| !d.is_empty());
    let has_body = stmts.iter().any(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
    let has_dangling_comments = false; // TODO: Dangling comments

    if !has_body && !has_directives && !has_dangling_comments {
        return None;
    }

    let mut parts = Vec::new_in(p.allocator);

    if has_directives {
        if let Some(directives) = directives {
            parts.extend(statement::print_directives(
                p,
                directives,
                has_body || has_dangling_comments,
            ));
        }
    }

    if has_body {
        parts.extend(statement::print_statement_sequence(p, stmts));
    }

    if has_dangling_comments {
        // TODO: Dangling comments
    }

    Some(array!(p, parts))
}
