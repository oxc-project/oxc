use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Expand, FormatTrailingCommas,
    formatter::{
        Buffer, Comments, Format, FormatResult, Formatter, GroupId, prelude::*,
        separated::FormatSeparatedIter,
    },
    generated::ast_nodes::AstNode,
    write,
};

use super::utils::array::write_array_node;

pub struct ArrayElementList<'a, 'b> {
    elements: &'b AstNode<'a, Vec<'a, ArrayExpressionElement<'a>>>,
    group_id: Option<GroupId>,
}

impl<'a, 'b> ArrayElementList<'a, 'b> {
    pub fn new(
        elements: &'b AstNode<'a, Vec<'a, ArrayExpressionElement<'a>>>,
        group_id: GroupId,
    ) -> Self {
        Self { elements, group_id: Some(group_id) }
    }
}

impl<'a> Format<'a> for ArrayElementList<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let expand_lists = f.context().options().expand == Expand::Always;
        let layout = if expand_lists {
            ArrayLayout::OnePerLine
        } else if can_concisely_print_array_list(self.elements, f.context().comments()) {
            ArrayLayout::Fill
        } else {
            ArrayLayout::OnePerLine
        };

        match layout {
            ArrayLayout::Fill => {
                let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());

                let mut filler = f.fill();

                // Using format_separated is valid in this case as can_print_fill does not allow holes
                for (element, formatted) in self.elements.iter().zip(
                    FormatSeparatedIter::new(self.elements.iter(), ",")
                        .with_trailing_separator(trailing_separator)
                        .with_group_id(self.group_id),
                ) {
                    filler.entry(
                        &format_once(|f| {
                            if get_lines_before(element.span()) > 1 {
                                write!(f, empty_line())
                            } else if f
                                .comments()
                                .has_leading_own_line_comment(element.span().start)
                            {
                                write!(f, hard_line_break())
                            } else {
                                write!(f, soft_line_break_or_space())
                            }
                        }),
                        &formatted,
                    );
                }

                filler.finish()
            }
            ArrayLayout::OnePerLine => write_array_node(self.elements, f),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum ArrayLayout {
    /// Tries to fit as many array elements on a single line as possible.
    ///
    /// ```javascript
    /// [
    ///     1, 2, 3,
    ///     5, 6,
    /// ]
    /// ```
    Fill,

    /// Prints every element on a single line if the whole array expression exceeds the line width, or any
    /// of its elements gets printed in *expanded* mode.
    /// ```javascript
    /// [
    ///     a.b(),
    ///     4,
    ///     3,
    /// ]
    /// ```
    OnePerLine,
}

/// Returns true if the provided JsArrayElementList could
/// be "fill-printed" instead of breaking each element on
/// a different line.
///
/// The underlying logic only allows lists of literal expressions
/// with 10 or less characters, potentially wrapped in a "short"
/// unary expression (+, -, ~ or !)
pub fn can_concisely_print_array_list(
    list: &Vec<'_, ArrayExpressionElement<'_>>,
    comments: &Comments,
) -> bool {
    if list.is_empty() {
        return false;
    }

    list.iter().all(|item| {
        let end = match &item {
            ArrayExpressionElement::NumericLiteral(literal) => literal.span.end,
            ArrayExpressionElement::UnaryExpression(expr) => {
                let signed = expr.operator.is_arithmetic();
                let argument = &expr.argument;

                match argument {
                    Expression::NumericLiteral(literal) => {
                        if signed && !comments.has_comments(literal.span) {
                            literal.span.end
                        } else {
                            return false;
                        }
                    }
                    _ => return false,
                }
            }
            _ => return false,
        };

        // Does not have a line comment ending on the same line
        // ```javascript
        // [ a // not this
        //  b];
        //
        // [
        //   // This is fine
        //   thats
        // ]
        // ```
        !comments
            .trailing_comments(end)
            .iter()
            .filter(|comment| comment.kind().is_line())
            .any(|comment| comment.lines_before() == 0)
    })
}
