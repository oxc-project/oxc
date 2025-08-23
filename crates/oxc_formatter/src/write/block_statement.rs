use oxc_allocator::{Address, GetAddress};
use oxc_ast::{AstKind, ast::*};
use oxc_span::GetSpan;

use super::FormatWrite;
use crate::{
    format_args,
    formatter::{
        Buffer, FormatResult, Formatter,
        prelude::*,
        trivia::{DanglingIndentMode, FormatDanglingComments},
    },
    generated::ast_nodes::{AstNode, AstNodes},
    write,
};

impl<'a> FormatWrite<'a> for AstNode<'a, BlockStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "{")?;

        let comments_before_catch_clause = if let AstNodes::CatchClause(catch) = self.parent {
            f.context().get_cached_element(&catch.span)
        } else {
            None
        };
        let has_comments_before_catch_clause = comments_before_catch_clause.is_some();
        // See reason in `[AstNode<'a, CatchClause<'a>>::write]`
        let formatted_comments_before_catch_clause = format_once(|f| {
            if let Some(comments) = comments_before_catch_clause {
                f.write_element(comments)
            } else {
                Ok(())
            }
        });

        if is_empty_block(&self.body, f) {
            // `if (a) /* comment */ {}`
            // should be formatted like:
            // `if (a) { /* comment */ }`
            //
            // Some comments are not inside the block, but we need to print them inside the block.
            if has_comments_before_catch_clause
                || f.context().comments().has_comments_before(self.span.end)
            {
                write!(
                    f,
                    block_indent(&format_args!(
                        &formatted_comments_before_catch_clause,
                        format_dangling_comments(self.span)
                    ))
                )?;
            } else if is_non_collapsible(self.parent) {
                write!(f, hard_line_break())?;
            }
        } else {
            write!(
                f,
                block_indent(&format_args!(&formatted_comments_before_catch_clause, self.body()))
            )?;
        }
        write!(f, "}")
    }
}

pub fn is_empty_block(block: &[Statement<'_>], f: &Formatter<'_, '_>) -> bool {
    block.is_empty()
        || block.iter().all(|s| {
            matches!(s, Statement::EmptyStatement(_))
            // TODO: it seems removing `has_comments` doesn't break anything, needs to check further
            // && !f.comments().has_comments(s.span())
            // && !f.comments().is_suppressed(s.span())
        })
}

/// Formatting of curly braces for an:
/// * empty block: same line `{}`,
/// * empty block that is the 'bons' or 'alt' of an if statement: two lines `{\n}`
/// * non empty block: put each stmt on its own line: `{\nstmt1;\nstmt2;\n}`
/// * non empty block with comments (trailing comments on {, or leading comments on })
fn is_non_collapsible(parent: &AstNodes<'_>) -> bool {
    match parent {
        AstNodes::FunctionBody(_)
        | AstNodes::ForStatement(_)
        | AstNodes::WhileStatement(_)
        | AstNodes::DoWhileStatement(_)
        | AstNodes::TSModuleDeclaration(_) => false,
        AstNodes::CatchClause(catch) => {
            // prettier collapse the catch block when it don't have `finalizer`, insert a new line when it has `finalizer`
            matches!(catch.parent, AstNodes::TryStatement(try_stmt) if try_stmt.finalizer().is_some())
        }
        _ => true,
    }
}
