use oxc_ast::ast::*;
use oxc_span::GetSpan;

use super::FormatWrite;
use crate::{
    formatter::{Buffer, FormatResult, Formatter, prelude::*},
    write,
};

impl<'a> FormatWrite<'a> for BlockStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [text("{")])?;

        if is_empty_block(self) {
            let comments = f.comments();
            let has_dangling_comments = comments.has_dangling_comments(self.span);
            if has_dangling_comments {
            } else if is_non_collapsible() {
                write!(f, [hard_line_break()])?;
            }
        } else {
            write!(f, [text("{")])?;
        }

        write!(f, [text("}")])
    }
}

fn is_empty_block(block: &BlockStatement<'_>) -> bool {
    true
}

fn is_non_collapsible() -> bool {
    false
}
