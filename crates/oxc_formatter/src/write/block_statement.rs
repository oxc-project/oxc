use oxc_ast::{AstKind, ast::*};

use super::FormatWrite;
use crate::{
    formatter::{Buffer, FormatResult, Formatter, prelude::*},
    write,
};

impl<'a> FormatWrite<'a> for BlockStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "{")?;

        if is_empty_block(self) {
            let comments = f.comments();
            let has_dangling_comments = comments.has_dangling_comments(self.span);
            if has_dangling_comments {
            } else if is_non_collapsible(f) {
                write!(f, hard_line_break())?;
            }
        } else {
            write!(f, "{")?;
        }

        write!(f, "}")
    }
}

fn is_empty_block(block: &BlockStatement<'_>) -> bool {
    block.body.is_empty()
}

fn is_non_collapsible(_f: &Formatter<'_, '_>) -> bool {
    true
}
