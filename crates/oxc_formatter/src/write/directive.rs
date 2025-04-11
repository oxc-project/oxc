use oxc_allocator::Vec;
use oxc_ast::ast::*;

use super::FormatWrite;
use crate::{
    formatter::{Buffer, FormatResult, Formatter, prelude::*},
    write,
};

impl<'a> Format<'a> for Vec<'a, Directive<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.is_empty() {
            return Ok(());
        }
        let source_text = f.context().source_text();
        let mut join = f.join_nodes_with_hardline();
        for directive in self {
            join.entry(directive.span, source_text, directive);
        }
        join.finish()?;

        // TODO
        let need_extra_empty_line = false;
        if need_extra_empty_line { write!(f, empty_line()) } else { write!(f, hard_line_break()) }
    }
}

impl<'a> FormatWrite<'a> for Directive<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let source_text = f.context().source_text();
        write!(f, located_token_text(self.span, source_text))
    }
}
