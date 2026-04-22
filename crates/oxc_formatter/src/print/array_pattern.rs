use std::ops::Deref;

use oxc_ast::ast::ArrayPattern;
use oxc_span::GetSpan;

use crate::{
    Format,
    ast_nodes::AstNode,
    formatter::{Formatter, prelude::*, trivia::format_dangling_comments},
    utils::array::write_array_node,
    write,
};

use super::FormatWrite;

struct FormatArrayPattern<'a, 'b>(&'b AstNode<'a, ArrayPattern<'a>>);

impl<'a> Deref for FormatArrayPattern<'a, '_> {
    type Target = AstNode<'a, ArrayPattern<'a>>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Format<'a> for FormatArrayPattern<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, "[");

        if self.elements.is_empty() && self.rest.is_none() {
            write!(f, [format_dangling_comments(self.span()).with_block_indent()]);
        } else {
            write!(
                f,
                group(&soft_block_indent(&format_with(|f| {
                    let has_element = !self.elements.is_empty();
                    if has_element {
                        write_array_node(
                            self.elements.len() + usize::from(self.rest.is_some()),
                            self.elements().iter().map(AstNode::as_ref),
                            f,
                        );
                    }
                    if let Some(rest) = self.rest() {
                        write!(f, [has_element.then_some(soft_line_break_or_space()), rest]);
                    }
                })))
            );
        }

        write!(f, "]");
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ArrayPattern<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        FormatArrayPattern(self).fmt(f);
    }
}
