use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_formatter_core::{Format, GroupId};
use oxc_span::GetSpan;

use crate::{
    FormatTrailingCommas,
    ast_nodes::AstNode,
    formatter::{JsFormatContext, JsFormatter, prelude::*, separated::FormatSeparatedIter},
    utils::array::{write_array_node, write_unclaimed_comments},
    write,
};

pub struct ArrayElementList<'a, 'b> {
    elements: &'b AstNode<'b, 'a, ArenaVec<'a, ArrayExpressionElement<'a>>>,
    group_id: Option<GroupId>,
}

impl<'a, 'b> ArrayElementList<'a, 'b> {
    pub fn new(
        elements: &'b AstNode<'b, 'a, ArenaVec<'a, ArrayExpressionElement<'a>>>,
        group_id: GroupId,
    ) -> Self {
        Self { elements, group_id: Some(group_id) }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for ArrayElementList<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let array_span = self.elements.parent().span();
        let layout = if can_concisely_print_array_list(array_span, self.elements, f) {
            ArrayLayout::Fill
        } else {
            ArrayLayout::OnePerLine
        };

        match layout {
            ArrayLayout::Fill => {
                let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());

                let mut filler = f.fill();

                // Using format_separated is valid in this case as can_print_fill does not allow holes
                for element in FormatSeparatedIter::new(self.elements.iter(), ",")
                    .with_trailing_separator(trailing_separator)
                    .with_group_id(self.group_id)
                {
                    filler.entry(
                        &format_with(|f| {
                            if f.lines_before(element.span()) > 1 {
                                write!(f, empty_line());
                            } else if f
                                .comments()
                                .has_leading_own_line_comment(element.span().start)
                            {
                                write!(f, hard_line_break());
                            } else {
                                write!(f, soft_line_break_or_space());
                            }
                        }),
                        &element,
                    );
                }

                filler.finish();
                write_unclaimed_comments(array_span.end - 1, f);
            }
            ArrayLayout::OnePerLine => write_array_node(
                array_span,
                None,
                self.elements.iter().map(|e| if e.is_elision() { None } else { Some(e) }),
                f,
            ),
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
    array_expression_span: Span,
    list: &[ArrayExpressionElement<'_>],
    f: &JsFormatter<'_, '_>,
) -> bool {
    if list.is_empty() {
        return false;
    }

    let comments = f.comments();

    for item in list {
        match item {
            ArrayExpressionElement::NumericLiteral(_) => {}
            ArrayExpressionElement::UnaryExpression(unary_expr) => {
                let signed = unary_expr.operator.is_arithmetic();
                let argument = &unary_expr.argument;

                if !signed
                    || !matches!(argument, Expression::NumericLiteral(_))
                    // `-(/* comment */ 1)`
                    || comments.has_any_comment_in_range(unary_expr.span.start, unary_expr.span.end)
                {
                    return false;
                }
            }
            _ => return false,
        }
    }

    // Does not have a line comment ending on the same line
    // ```javascript
    // [ a // not this
    //  b];
    //
    // [
    //   // This is fine
    //   thats
    // ]
    //
    // [ // Also fine, it moves to its own line
    //   a,
    // ]
    // ```

    !comments
        .all_comments_in_range(list[0].span().start, array_expression_span.end)
        .any(|comment| comment.is_line() && !comment.preceded_by_newline())
}
