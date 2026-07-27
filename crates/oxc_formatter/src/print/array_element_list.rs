use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    FormatTrailingCommas,
    ast_nodes::AstNode,
    formatter::{
        Buffer, Format, GroupId, JsFormatContext, JsFormatter,
        prelude::*,
        separated::FormatSeparatedIter,
        trivia::{DanglingIndentMode, FormatDanglingComments},
    },
    options::ArrayLinePattern,
    utils::array::write_array_node,
    write,
};

pub struct ArrayElementList<'a, 'b> {
    elements: &'b AstNode<'a, ArenaVec<'a, ArrayExpressionElement<'a>>>,
    group_id: Option<GroupId>,
    /// When `true`, always use `OnePerLine` layout regardless of the fill heuristic.
    force_one_per_line: bool,
}

impl<'a, 'b> ArrayElementList<'a, 'b> {
    pub fn new(
        elements: &'b AstNode<'a, ArenaVec<'a, ArrayExpressionElement<'a>>>,
        group_id: GroupId,
    ) -> Self {
        Self { elements, group_id: Some(group_id), force_one_per_line: false }
    }

    pub fn with_force_one_per_line(mut self, force: bool) -> Self {
        self.force_one_per_line = force;
        self
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for ArrayElementList<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        // A configured line pattern applies to any array printed across
        // multiple lines, however it came to break. Holes and comments need
        // `write_array_node`'s special handling, so they opt out
        let line_pattern = f
            .options()
            .array_line_pattern
            .as_ref()
            .filter(|_| can_use_line_pattern(self.elements.parent().span(), self.elements, f));

        let layout = if let Some(pattern) = line_pattern {
            ArrayLayout::Pattern(pattern.clone())
        } else if self.force_one_per_line {
            ArrayLayout::OnePerLine
        } else if can_concisely_print_array_list(self.elements.parent().span(), self.elements, f) {
            ArrayLayout::Fill
        } else {
            ArrayLayout::OnePerLine
        };

        match layout {
            ArrayLayout::Pattern(pattern) => {
                let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());

                let mut line_index = 0;
                let mut written_in_line = 0;

                // Using format_separated is valid in this case as the line
                // pattern is not used when the array contains holes.
                // Pattern boundaries are soft so a flat array stays on one
                // line; elements within a line never break on their own
                for (index, element) in FormatSeparatedIter::new(self.elements.iter(), ",")
                    .with_trailing_separator(trailing_separator)
                    .with_group_id(self.group_id)
                    .enumerate()
                {
                    if index > 0 {
                        if written_in_line >= pattern.elements_for_line(line_index) {
                            write!(f, soft_line_break_or_space());
                            line_index += 1;
                            written_in_line = 0;
                        } else {
                            write!(f, space());
                        }
                    }
                    write!(f, [element]);
                    written_in_line += 1;
                }
            }
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
            }
            ArrayLayout::OnePerLine => write_array_node(
                self.elements.len(),
                self.elements.iter().map(|e| if e.is_elision() { None } else { Some(e) }),
                f,
            ),
        }

        // Comments that no element consumed as a trailing comment
        // (e.g. after a trailing hole: `[,, /* comment */]`)
        // would otherwise escape the brackets; print them right before the `]` like Prettier
        let dangling_comments =
            f.context().comments().comments_before(self.elements.parent().span().end);
        write!(
            f,
            FormatDanglingComments::Comments {
                comments: dangling_comments,
                indent: DanglingIndentMode::None
            }
        );
    }
}

#[derive(Clone, Debug)]
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

    /// Prints a fixed number of elements per line, following the configured
    /// repeating pattern (e.g. `"2 1"`).
    /// ```javascript
    /// [
    ///     1, 2,
    ///     3,
    ///     4, 5,
    /// ]
    /// ```
    Pattern(ArrayLinePattern),
}

/// A configured line pattern replaces the one-per-line layout only when the
/// array has no holes (which `format_separated` cannot print) and no comments
/// (which need their own lines).
fn can_use_line_pattern(
    array_expression_span: Span,
    list: &[ArrayExpressionElement<'_>],
    f: &JsFormatter<'_, '_>,
) -> bool {
    !list.iter().any(ArrayExpressionElement::is_elision)
        && f.comments()
            .comments_in_range(array_expression_span.start, array_expression_span.end)
            .is_empty()
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

    let mut comments_iter = comments.comments_before_iter(array_expression_span.end);

    for item in list {
        match item {
            ArrayExpressionElement::NumericLiteral(_) => {}
            ArrayExpressionElement::UnaryExpression(unary_expr) => {
                let signed = unary_expr.operator.is_arithmetic();
                let argument = &unary_expr.argument;

                if !signed
                    || !matches!(argument, Expression::NumericLiteral(_))
                    || has_comment_inside_unary(&mut comments_iter, unary_expr.span)
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
    // ```

    !comments
        .comments_before_iter(array_expression_span.end)
        .any(|comment| comment.is_line() && !comment.preceded_by_newline())
}

// ```js
// - (/* comment */ 1)
//    ^^^^^^^^^^^^ // This is a unary expression with a comment inside
// ```
fn has_comment_inside_unary<'a>(
    comments_iter: &mut impl Iterator<Item = &'a Comment>,
    unary_expr_span: Span,
) -> bool {
    // `comments_iter` is avoid repeatedly iterating over the same comments from the start
    for comment in comments_iter.by_ref() {
        if comment.span.start > unary_expr_span.start {
            return unary_expr_span.contains_inclusive(comment.span);
        }
    }
    false
}
