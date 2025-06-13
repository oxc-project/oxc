use oxc_allocator::{Address, GetAddress};
use oxc_ast::{AstKind, ast::*};
use oxc_span::GetSpan;

use super::FormatWrite;
use crate::{
    formatter::{Buffer, FormatResult, Formatter, prelude::*},
    generated::ast_nodes::{AstNode, AstNodes},
    write,
};

impl<'a> FormatWrite<'a> for AstNode<'a, BlockStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "{")?;
        if is_empty_block(self, f) {
            let comments = f.comments();
            let has_dangling_comments = comments.has_dangling_comments(self.span());
            if has_dangling_comments {
                write!(f, [format_dangling_comments(self.span()).with_block_indent()])?;
            } else if is_non_collapsible(self.parent) {
                write!(f, hard_line_break())?;
            }
        } else {
            write!(f, block_indent(&self.body()))?;
        }
        write!(f, "}")
    }
}

fn is_empty_block(block: &BlockStatement<'_>, f: &Formatter<'_, '_>) -> bool {
    block.body.is_empty()
        || block.body.iter().all(|s| {
            matches!(s, Statement::EmptyStatement(_))
                && !f.comments().has_comments(s.span())
                && !f.comments().is_suppressed(s.span())
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
