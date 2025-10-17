use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    Expand, Format, FormatResult, FormatTrailingCommas,
    ast_nodes::AstNode,
    formatter::{Formatter, prelude::*, trivia::format_dangling_comments},
    write,
};

use super::FormatWrite;

impl<'a> Format<'a> for AstNode<'a, Vec<'a, TSTupleElement<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());
        f.join_nodes_with_soft_line()
            .entries_with_trailing_separator(self.iter(), ",", trailing_separator)
            .finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTupleType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "[")?;

        let element_types = self.element_types();
        if element_types.is_empty() {
            write!(f, [format_dangling_comments(self.span).with_block_indent()])?;
        } else {
            let should_expand = f.options().expand == Expand::Always;

            write!(f, [group(&soft_block_indent(&element_types)).should_expand(should_expand)])?;
        }

        write!(f, "]")
    }
}
