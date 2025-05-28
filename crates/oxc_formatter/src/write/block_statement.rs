use oxc_allocator::{Address, GetAddress};
use oxc_ast::{AstKind, ast::*};
use oxc_span::GetSpan;

use super::FormatWrite;
use crate::{
    formatter::{Buffer, FormatResult, Formatter, prelude::*},
    write,
};

impl<'a> FormatWrite<'a> for BlockStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "{")?;
        if is_empty_block(self, f) {
            let comments = f.comments();
            let has_dangling_comments = comments.has_dangling_comments(self.span);
            if has_dangling_comments {
                write!(f, [format_dangling_comments(self.span).with_block_indent()])?;
            } else if is_non_collapsible(f.parent_kind_of(Address::from_ptr(self)), f) {
                write!(f, hard_line_break())?;
            }
        } else {
            write!(f, block_indent(&self.body))?;
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
/// * empty block that is the 'cons' or 'alt' of an if statement: two lines `{\n}`
/// * non empty block: put each stmt on its own line: `{\nstmt1;\nstmt2;\n}`
/// * non empty block with comments (trailing comments on {, or leading comments on })
fn is_non_collapsible(parent_kind: AstKind<'_>, f: &Formatter<'_, '_>) -> bool {
    match parent_kind {
        AstKind::FunctionBody(_)
        | AstKind::ForStatement(_)
        | AstKind::WhileStatement(_)
        | AstKind::DoWhileStatement(_)
        | AstKind::TSModuleDeclaration(_) => false,
        AstKind::CatchClause(_) => {
            // prettier collapse the catch block when it don't have `finalizer`, insert a new line when it has `finalizer`
            matches!(f.parent_kind_of(parent_kind.address()), AstKind::TryStatement(try_stmt) if try_stmt.finalizer.is_some())
        }
        _ => true,
    }
}
