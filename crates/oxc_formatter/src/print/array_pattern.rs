use std::ops::Deref;

use oxc_ast::ast::ArrayPattern;
use oxc_span::GetSpan;

use crate::{
    Format,
    ast_nodes::AstNode,
    formatter::{prelude::*, trivia::format_dangling_comments},
    utils::array::write_array_node,
    write,
};

use super::FormatWrite;

struct FormatArrayPattern<'a, 'b>(&'b AstNode<'b, 'a, ArrayPattern<'a>>);

impl<'a, 'b> Deref for FormatArrayPattern<'a, 'b> {
    type Target = AstNode<'b, 'a, ArrayPattern<'a>>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatArrayPattern<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        write!(f, "[");

        if self.elements.is_empty() && self.rest.is_none() {
            write!(f, [format_dangling_comments(self.span()).with_soft_block_indent()]);
        } else {
            write!(
                f,
                group(&soft_block_indent(&format_with(|f| {
                    let has_element = !self.elements.is_empty();
                    if has_element {
                        write_array_node(
                            self.span(),
                            self.rest.as_ref().map(|rest| rest.span.start),
                            self.elements().iter().map(|element| element.as_ref()),
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

impl<'a> FormatWrite<'a> for AstNode<'_, 'a, ArrayPattern<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        FormatArrayPattern(self).fmt(f);
    }
}
