use std::ops::Deref;

use oxc_ast::ast::ArrayPattern;
use oxc_span::{GetSpan, Span};

use crate::{
    Format, FormatResult,
    ast_nodes::{AstNode, AstNodes},
    formatter::{Formatter, prelude::*, trivia::format_dangling_comments},
    utils::{
        array::write_array_node,
        format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
    },
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

impl GetSpan for FormatArrayPattern<'_, '_> {
    fn span(&self) -> Span {
        // `[a, b]: [a, b]`
        //  ^^^^^^^^^^^^^^ ArrayPattern's span covers the type annotation if exists,
        //  ^^^^^^ but we want the span to cover only the pattern itself, otherwise,
        //         the comments of type annotation will be treated as dangling comments
        //         of ArrayPattern.
        if let AstNodes::FormalParameter(param) = self.parent
            && let Some(ty) = &param.pattern.type_annotation
        {
            Span::new(self.span.start, ty.span.start)
        } else {
            self.span
        }
    }
}

impl<'a> Format<'a> for FormatArrayPattern<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "[")?;

        if self.elements.is_empty() && self.rest.is_none() {
            write!(f, [format_dangling_comments(self.span()).with_block_indent()])?;
        } else {
            write!(
                f,
                group(&soft_block_indent(&format_once(|f| {
                    let has_element = !self.elements.is_empty();
                    if has_element {
                        write_array_node(
                            self.elements.len() + usize::from(self.rest.is_some()),
                            self.elements().iter().map(AstNode::as_ref),
                            f,
                        )?;
                    }
                    if let Some(rest) = self.rest() {
                        write!(f, [has_element.then_some(soft_line_break_or_space()), rest]);
                    }
                    Ok(())
                })))
            )?;
        }

        write!(f, "]")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ArrayPattern<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if matches!(self.parent, AstNodes::FormalParameter(param) if param.pattern.type_annotation.is_some())
        {
            FormatNodeWithoutTrailingComments(&FormatArrayPattern(self)).fmt(f)
        } else {
            FormatArrayPattern(self).fmt(f)
        }
    }
}
