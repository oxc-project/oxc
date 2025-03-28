use oxc_allocator::Vec;
use oxc_ast::{AstKind, ast::*};
use oxc_span::GetSpan;

use crate::{
    formatter::{Buffer, Format, FormatResult, Formatter, prelude::*},
    write,
};

use super::FormatWrite;

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
