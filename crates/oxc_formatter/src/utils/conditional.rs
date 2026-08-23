use std::ops::Deref;

use oxc_ast::ast::*;
use oxc_formatter_core::FormatElement;
use oxc_span::{GetSpan, Span};

use crate::{
    Format,
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        JsFormatter,
        prelude::*,
        trivia::{
            DanglingIndentMode, FormatDanglingComments, FormatLeadingComments,
            FormatTrailingComments,
        },
    },
    parentheses::NeedsParentheses,
    utils::assignment_like::{AssignmentLikeLayout, is_lone_short_argument},
    utils::format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
    write,
};

#[derive(Clone, Copy)]
pub enum ConditionalLike<'a, 'b> {
    ConditionalExpression(&'b AstNode<'a, ConditionalExpression<'a>>),
    TSConditionalType(&'b AstNode<'a, TSConditionalType<'a>>),
}

impl<'a> ConditionalLike<'a, '_> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            ConditionalLike::ConditionalExpression(expr) => expr.span,
            ConditionalLike::TSConditionalType(ty) => ty.span,
        }
    }

    #[inline]
    fn parent(&self) -> &AstNodes<'a> {
        match self {
            ConditionalLike::ConditionalExpression(expr) => expr.parent(),
            ConditionalLike::TSConditionalType(ty) => ty.parent(),
        }
    }

    #[inline]
    fn is_conditional_expression(&self) -> bool {
        matches!(self, ConditionalLike::ConditionalExpression(_))
    }
}

/// Layout information for a conditional expression to determine formatting strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalLayout {
    /// This conditional isn't a child of another conditional.
    ///
    /// ```javascript
    /// return a ? b : c;
    /// ```
    Root {
        /// Whether this is part of a JSX conditional chain
        jsx_chain: bool,
    },
    /// Conditional that is the `test` of another conditional.
    ///
    /// ```javascript
    /// (
    ///     a              // <-- Note the extra indent here
    ///         ? b
    ///         : c
    ///  )
    ///     ? d
    ///     : e;
    /// ```
    NestedTest,
    /// Conditional that is the `consequent` of another conditional.
    ///
    /// ```javascript
    /// condition1
    ///     ? condition2
    ///         ? consequent2 // <-- consequent and alternate gets indented
    ///         : alternate2
    ///     : alternate1;
    /// ```
    NestedConsequent,
    /// Conditional that is the `alternate` of another conditional.
    ///
    /// The `test` condition of a nested alternated is aligned with the parent's `:`.
    ///
    /// ```javascript
    /// outerCondition
    ///     ? consequent
    ///     : nestedAlternate +
    ///       binary + // <- notice how the content is aligned to the `: `
    ///     ? consequentOfnestedAlternate
    ///     : alternateOfNestedAlternate;
    /// ```
    NestedAlternate,
}

impl ConditionalLayout {
    #[inline]
    fn is_root(self) -> bool {
        matches!(self, Self::Root { .. })
    }

    #[inline]
    fn is_nested_test(self) -> bool {
        matches!(self, Self::NestedTest)
    }

    #[inline]
    fn is_nested_alternate(self) -> bool {
        matches!(self, Self::NestedAlternate)
    }

    #[inline]
    fn is_jsx_chain(self) -> bool {
        matches!(self, Self::Root { jsx_chain: true })
    }
}

fn format_trailing_comments<'a>(
    mut start: u32,
    end: u32,
    operator: u8,
    f: &mut JsFormatter<'_, 'a>,
) {
    let mut get_comments = |f: &mut JsFormatter<'_, 'a>| -> &'a [Comment] {
        let comments = f.context().comments().unprinted_comments();
        if comments.is_empty() {
            return &[];
        }

        let source_text = f.context().source_text();
        let mut index_before_operator = None;
        for (index, comment) in comments.iter().enumerate() {
            // This comment is after the `end` position, so we stop here and return the comments before this comment
            if comment.span.end > end {
                return &comments[..index_before_operator.unwrap_or(index)];
            }

            // `a /* c1 */ /* c2 */ ? b : c`
            //   ^        ^        ^
            //   |        |        |
            //   |        |        |
            //  these are the gaps between comments
            // If this comment is in a new line, we stop here and return the comments before this comment
            if source_text.contains_newline_between(start, comment.span.start) {
                return &comments[..index];
            }
            // If this comment is a line comment or an end of line comment, so we stop here and return the comments with this comment
            else if comment.is_line() || comment.followed_by_newline() {
                return &comments[..=index];
            }
            // Store the index of the comment before the operator, if no line comment or no new line is found, then return all comments before operator
            else if source_text.bytes_contain(start, comment.span.start, operator) {
                index_before_operator = Some(index);
            }

            // Update the start position for the next iteration
            start = comment.span.end;
        }

        &comments[..index_before_operator.unwrap_or(comments.len())]
    };

    let comments = get_comments(f);
    FormatTrailingComments::Comments(comments).fmt(f);
}

/// Prints comments that occur before a conditional operator. Unlike the legacy ternary printer,
/// curious ternaries keep own-line comments on the branch preceding `?` or `:`.
fn format_comments_before_operator<'a>(
    mut start: u32,
    end: u32,
    operator: u8,
    f: &mut JsFormatter<'_, 'a>,
) {
    let mut get_comments = |f: &mut JsFormatter<'_, 'a>| -> &'a [Comment] {
        let source = f.source_text();
        let comments = f.context().comments().unprinted_comments();
        let mut count = 0;

        for comment in comments {
            if comment.span.end > end || comment.preceded_by_newline() {
                break;
            }

            let follows_operator = source.bytes_contain(start, comment.span.start, operator);
            // Match Prettier's attachment for `test ? consequent :/* multiline */ alternate`:
            // the comment is printed as a trailing comment of the consequent, before `:`.
            let is_attached_multiline_comment = operator == b':'
                && comment.is_multiline_block()
                && comment.span.start.checked_sub(1).is_some_and(|offset| {
                    source.as_bytes().get(offset as usize) == Some(&operator)
                });
            if follows_operator && !is_attached_multiline_comment {
                break;
            }

            count += 1;
            start = comment.span.end;

            if follows_operator {
                break;
            }
        }

        &comments[..count]
    };

    let comments = get_comments(f);
    FormatTrailingComments::Comments(comments).fmt(f);
}

impl<'a> FormatConditionalLike<'a, '_> {
    /// Returns comments to print before `:` and whether an end-of-line directive must remain after
    /// the colon. Preserved directives also disable alternate parenthesization so their line-based
    /// semantics stay intact.
    fn alternate_comments(&self, f: &JsFormatter<'_, 'a>) -> (&'a [Comment], bool) {
        if matches!(self.conditional, ConditionalLike::TSConditionalType(_)) {
            return (&[], false);
        }

        let (mut start, end) = match self.conditional {
            ConditionalLike::ConditionalExpression(conditional) => {
                (conditional.consequent.span().end, conditional.alternate.span().start)
            }
            ConditionalLike::TSConditionalType(conditional) => {
                (conditional.true_type.span().end, conditional.false_type.span().start)
            }
        };
        let source = f.source_text();
        let comments = f.context().comments().unprinted_comments();
        let mut count = 0;
        let mut passed_colon = false;

        for comment in comments {
            if comment.span.end > end {
                break;
            }

            passed_colon |= source.bytes_contain(start, comment.span.start, b':');
            if passed_colon && comment.is_line() && !comment.preceded_by_newline() {
                let content = source.text_for(&comment.content_span()).trim_start();
                let marker = content.split_ascii_whitespace().next().unwrap_or_default();
                let is_line_directive = comment.is_annotation()
                    || f.comments().is_suppression_comment(comment)
                    || marker.starts_with('@')
                    || marker.contains(':')
                    || marker.contains("-disable")
                    || marker.contains("-ignore");
                if is_line_directive {
                    return (&comments[..count], true);
                }
            }

            let is_alternate_comment = if passed_colon {
                comment.is_line() || comment.is_multiline_block()
            } else {
                comment.preceded_by_newline()
            };
            if !is_alternate_comment {
                break;
            }

            count += 1;
            start = comment.span.end;
        }

        (&comments[..count], false)
    }

    /// Determines the layout of this conditional based on its parent
    fn layout(&self, f: &JsFormatter<'_, 'a>) -> ConditionalLayout {
        let self_span = self.span();

        match self.parent() {
            AstNodes::ConditionalExpression(parent) => {
                let parent_expr = parent.as_ref();
                if parent_expr.test.span() == self_span {
                    ConditionalLayout::NestedTest
                } else if parent_expr.consequent.span() == self_span {
                    ConditionalLayout::NestedConsequent
                } else {
                    ConditionalLayout::NestedAlternate
                }
            }
            AstNodes::TSConditionalType(parent) => {
                let parent_type = parent.as_ref();
                // For TS conditional types, both check_type and extends_type are part of the test
                let is_test = parent_type.check_type.span() == self_span
                    || parent_type.extends_type.span() == self_span;
                if is_test {
                    ConditionalLayout::NestedTest
                } else if parent_type.true_type.span() == self_span {
                    ConditionalLayout::NestedConsequent
                } else {
                    ConditionalLayout::NestedAlternate
                }
            }
            _ => {
                let jsx_chain = !f.options().experimental_ternaries
                    && f.context().source_type().is_jsx()
                    && self.is_jsx_conditional_chain();
                ConditionalLayout::Root { jsx_chain }
            }
        }
    }

    /// Checks if this conditional expression contains JSX elements
    #[inline]
    fn is_jsx_conditional_chain(&self) -> bool {
        #[inline]
        fn has_jsx_expression(expr: &Expression) -> bool {
            match expr {
                Expression::JSXElement(_) | Expression::JSXFragment(_) => true,
                Expression::ConditionalExpression(conditional) => recurse(conditional),
                _ => false,
            }
        }

        fn recurse(expr: &ConditionalExpression<'_>) -> bool {
            has_jsx_expression(&expr.test)
                || has_jsx_expression(&expr.consequent)
                || has_jsx_expression(&expr.alternate)
        }

        let ConditionalLike::ConditionalExpression(conditional) = self.conditional else {
            return false; // Types can't contain JSX
        };

        recurse(conditional)
    }

    /// It is desired to add an extra indent if this conditional is a ConditionalExpression and is directly inside
    /// of a member chain:
    ///
    /// ```javascript
    /// // Input
    /// return (a ? b : c).member
    ///
    /// // Default
    /// return (a
    ///     ? b
    ///     : c
    /// ).member
    ///
    /// // Preferred
    /// return (
    ///     a
    ///         ? b
    ///         : c
    /// ).member
    /// ```
    fn should_extra_indent(&self, layout: ConditionalLayout) -> bool {
        if !layout.is_root() {
            return false;
        }

        // Only check for ConditionalExpression, not TS types
        let ConditionalLike::ConditionalExpression(expr) = self.conditional else {
            return false;
        };

        let mut expression_span = expr.span;
        let mut parent = expr.parent();

        // This tries to find the start of a member chain by iterating over all ancestors of the conditional.
        // The iteration "breaks" as soon as a non-member-chain node is found.
        loop {
            match parent {
                AstNodes::ChainExpression(chain) => {
                    if chain.expression.span() == expression_span {
                        expression_span = chain.span();
                        parent = chain.parent();
                    } else {
                        break;
                    }
                }
                AstNodes::StaticMemberExpression(member) => {
                    if member.object.span() == expression_span {
                        expression_span = member.span();
                        parent = member.parent();
                    } else {
                        break;
                    }
                }
                AstNodes::ComputedMemberExpression(member) => {
                    if member.object.span() == expression_span {
                        expression_span = member.span();
                        parent = member.parent();
                    } else {
                        break;
                    }
                }
                AstNodes::CallExpression(call) => {
                    if call.callee.span() == expression_span {
                        expression_span = call.span();
                        parent = call.parent();
                    } else {
                        break;
                    }
                }
                AstNodes::TSNonNullExpression(assertion) => {
                    if assertion.expression.span() == expression_span {
                        expression_span = assertion.span();
                        parent = assertion.parent();
                    } else {
                        break;
                    }
                }
                AstNodes::NewExpression(new_expr) => {
                    parent = new_expr.parent();
                    if new_expr.callee.span() == expression_span {
                        expression_span = new_expr.span();
                    }
                    break;
                }
                AstNodes::TSAsExpression(as_expr) => {
                    parent = as_expr.parent();
                    if as_expr.expression.span() == expression_span {
                        expression_span = as_expr.span();
                    }
                    break;
                }
                AstNodes::TSSatisfiesExpression(satisfies) => {
                    parent = satisfies.parent();
                    if satisfies.expression.span() == expression_span {
                        expression_span = satisfies.span();
                    }
                    break;
                }
                _ => break,
            }
        }

        // If we didn't find a member chain, no extra indent
        if expression_span == self.span() {
            return false;
        }

        // Check if the parent context requires extra indentation
        match parent {
            AstNodes::VariableDeclarator(decl) => {
                decl.init.as_ref().is_some_and(|init| init.span() == expression_span)
            }
            AstNodes::ReturnStatement(ret) => {
                ret.argument.as_ref().is_some_and(|arg| arg.span() == expression_span)
            }
            AstNodes::ThrowStatement(throw) => throw.argument.span() == expression_span,
            AstNodes::UnaryExpression(unary) => unary.argument.span() == expression_span,
            AstNodes::YieldExpression(yield_expr) => {
                yield_expr.argument.as_ref().is_some_and(|arg| arg.span() == expression_span)
            }
            AstNodes::AwaitExpression(await_expr) => await_expr.argument.span() == expression_span,
            AstNodes::AssignmentExpression(assign) => assign.right.span() == expression_span,
            _ => false,
        }
    }

    /// Returns `true` if this is the root conditional expression and the parent is a [`StaticMemberExpression`].
    #[inline]
    fn is_parent_static_member_expression(&self, layout: ConditionalLayout) -> bool {
        layout.is_root()
            && self.is_conditional_expression()
            && matches!(self.parent(), AstNodes::StaticMemberExpression(_))
    }

    /// Formats the test part of the conditional
    fn format_test<'f>(&self, f: &mut JsFormatter<'f, 'a>, layout: ConditionalLayout) {
        let format_inner = format_with(|f| {
            let (start, end) = match self.conditional {
                ConditionalLike::ConditionalExpression(conditional) => {
                    write!(f, FormatNodeWithoutTrailingComments(conditional.test()));
                    (conditional.test.span().end, conditional.consequent.span().start)
                }
                ConditionalLike::TSConditionalType(conditional) => {
                    write!(
                        f,
                        [
                            conditional.check_type(),
                            space(),
                            "extends",
                            space(),
                            FormatNodeWithoutTrailingComments(conditional.extends_type())
                        ]
                    );
                    (conditional.extends_type.span().end, conditional.true_type.span().start)
                }
            };

            format_trailing_comments(start, end, b'?', f);
        });

        if layout.is_nested_alternate() {
            // The leading comment should not be printed in the the `align`
            let start = self.conditional.span().start;
            let comments = f.context().comments().comments_before(start);
            FormatLeadingComments::Comments(comments).fmt(f);

            write!(f, [align(2, &format_inner)]);
        } else {
            write!(f, format_inner);
        }
    }

    /// Formats the consequent and alternate with proper formatting
    fn format_consequent_and_alternate<'f>(&self, f: &mut JsFormatter<'f, 'a>) {
        write!(f, [soft_line_break_or_space(), "?", space()]);

        let format_consequent = format_with(|f| {
            let format_consequent_with_trailing_comments = format_with(|f| {
                let (start, end) = match self.conditional {
                    ConditionalLike::ConditionalExpression(conditional) => {
                        write!(f, FormatNodeWithoutTrailingComments(conditional.consequent()));
                        (conditional.consequent.span().end, conditional.alternate.span().start)
                    }
                    ConditionalLike::TSConditionalType(conditional) => {
                        write!(f, FormatNodeWithoutTrailingComments(conditional.true_type()));
                        (conditional.true_type.span().end, conditional.false_type.span().start)
                    }
                };
                format_trailing_comments(start, end, b':', f);
            });

            let format_consequent_with_proper_indentation = format_with(|f| {
                if f.options().indent_style.is_space() {
                    write!(f, [align(2, &format_consequent_with_trailing_comments)]);
                } else {
                    write!(f, [indent(&format_consequent_with_trailing_comments)]);
                }
            });

            let is_nested_consequent = match self.conditional {
                ConditionalLike::ConditionalExpression(conditional) => {
                    matches!(conditional.consequent, Expression::ConditionalExpression(_))
                }
                ConditionalLike::TSConditionalType(conditional) => {
                    matches!(conditional.true_type, TSType::TSConditionalType(_))
                }
            };

            if is_nested_consequent {
                // Add parentheses around the consequent if it is a conditional expression and fits on the same line
                // so that it's easier to identify the parts that belong to a conditional expression.
                // `a ? b ? c: d : e` -> `a ? (b ? c: d) : e`
                write!(
                    f,
                    [
                        if_group_fits_on_line(&token("(")),
                        format_consequent_with_proper_indentation,
                        if_group_fits_on_line(&token(")"))
                    ]
                );
            } else {
                write!(f, format_consequent_with_proper_indentation);
            }
        });

        let format_alternative = format_with(|f| match self.conditional {
            ConditionalLike::ConditionalExpression(conditional) => {
                write!(f, [FormatNodeWithoutTrailingComments(conditional.alternate())]);
            }
            ConditionalLike::TSConditionalType(conditional) => {
                write!(f, [FormatNodeWithoutTrailingComments(conditional.false_type())]);
            }
        });
        let format_alternative = format_with(|f| {
            if f.options().indent_style.is_space() {
                write!(f, [align(2, &format_alternative)]);
            } else {
                write!(f, [indent(&format_alternative)]);
            }
        });

        write!(
            f,
            [format_consequent, soft_line_break_or_space(), ":", space(), format_alternative]
        );
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for ConditionalLike<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        FormatConditionalLike {
            conditional: self,
            options: FormatConditionalLikeOptions { jsx_chain: false, assignment_layout: None },
        }
        .fmt(f);
    }
}

struct FormatConditionalLikeOptions {
    /// Whether the parent is a jsx conditional chain.
    /// Gets passed through from the root to the consequent and alternate of [`ConditionalExpression`]s.
    ///
    /// Doesn't apply for [`TSConditionalType`].
    jsx_chain: bool,
    /// Assignment layout selected by the parent assignment-like node.
    assignment_layout: Option<AssignmentLikeLayout>,
}

struct FormatConditionalLike<'a, 'b> {
    conditional: &'b ConditionalLike<'a, 'b>,
    options: FormatConditionalLikeOptions,
}

#[derive(Clone)]
struct ReusedFormatElement<'a>(Option<FormatElement<'a>>);

impl<'a> Format<'a, JsFormatContext<'a>> for ReusedFormatElement<'a> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if let Some(element) = &self.0 {
            f.write_element(element.clone());
        }
    }
}

/// Formats a conditional while carrying the assignment layout selected by its parent.
pub fn format_conditional_like_with_assignment_layout<'a>(
    conditional: ConditionalLike<'a, '_>,
    assignment_layout: Option<AssignmentLikeLayout>,
    f: &mut JsFormatter<'_, 'a>,
) {
    FormatConditionalLike {
        conditional: &conditional,
        options: FormatConditionalLikeOptions { jsx_chain: false, assignment_layout },
    }
    .fmt(f);
}

impl<'a, 'b> Deref for FormatConditionalLike<'a, 'b> {
    type Target = ConditionalLike<'a, 'b>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.conditional
    }
}

impl<'a> FormatConditionalLike<'a, '_> {
    fn first_non_conditional_parent(&self) -> &AstNodes<'a> {
        let mut previous_span = self.span();
        let mut parent = self.parent();

        loop {
            let continues_chain = match (self.conditional, parent) {
                (
                    ConditionalLike::ConditionalExpression(_),
                    AstNodes::ConditionalExpression(conditional),
                ) => conditional.test.span() != previous_span,
                (
                    ConditionalLike::TSConditionalType(_),
                    AstNodes::TSConditionalType(conditional),
                ) => {
                    conditional.check_type.span() != previous_span
                        && conditional.extends_type.span() != previous_span
                }
                _ => false,
            };

            if !continues_chain {
                return parent;
            }

            previous_span = parent.span();
            parent = parent.parent();
        }
    }

    fn has_multiline_block_comments(&self, f: &JsFormatter<'_, 'a>) -> bool {
        let source = f.source_text();
        let span = self.span();
        f.comments().comments_in_range(span.start, span.end).iter().any(|comment| {
            comment.is_block()
                && source.contains_newline_between(comment.span.start, comment.span.end)
        })
    }

    fn format_experimental_test(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.conditional {
            ConditionalLike::ConditionalExpression(conditional) => {
                let test = format_with(|f| {
                    write!(f, [FormatNodeWithoutTrailingComments(conditional.test())]);
                    format_trailing_comments(
                        conditional.test.span().end,
                        conditional.consequent.span().start,
                        b'?',
                        f,
                    );
                });

                write!(
                    f,
                    [
                        if_group_breaks(&token("(")),
                        indent(&format_args!(soft_line_break(), test)),
                        soft_line_break(),
                        if_group_breaks(&token(")")),
                        matches!(conditional.test, Expression::ConditionalExpression(_))
                            .then_some(expand_parent())
                    ]
                );
            }
            ConditionalLike::TSConditionalType(conditional) => {
                write!(f, [conditional.check_type(), space(), "extends", space()]);

                let extends_type = format_with(|f| {
                    write!(f, [FormatNodeWithoutTrailingComments(conditional.extends_type())]);
                    format_trailing_comments(
                        conditional.extends_type.span().end,
                        conditional.true_type.span().start,
                        b'?',
                        f,
                    );
                });

                if matches!(
                    conditional.extends_type,
                    TSType::TSConditionalType(_) | TSType::TSMappedType(_)
                ) {
                    write!(f, [extends_type]);
                } else {
                    write!(
                        f,
                        [group(&format_args!(
                            if_group_breaks(&token("(")),
                            indent(&format_args!(soft_line_break(), extends_type)),
                            soft_line_break(),
                            if_group_breaks(&token(")")),
                        ))]
                    );
                }
            }
        }
    }

    fn fmt_experimental(&self, f: &mut JsFormatter<'_, 'a>) {
        let layout = self.layout(f);
        let parent = self.parent();
        let first_non_conditional_parent = self.first_non_conditional_parent();

        let is_parent_ternary = matches!(
            (self.conditional, parent),
            (ConditionalLike::ConditionalExpression(_), AstNodes::ConditionalExpression(_))
                | (ConditionalLike::TSConditionalType(_), AstNodes::TSConditionalType(_))
        );
        let is_in_test = layout.is_nested_test();
        let is_in_alternate = layout.is_nested_alternate();
        let is_ts_conditional = matches!(self.conditional, ConditionalLike::TSConditionalType(_));

        let (is_consequent_ternary, is_alternate_ternary) = match self.conditional {
            ConditionalLike::ConditionalExpression(conditional) => (
                matches!(conditional.consequent, Expression::ConditionalExpression(_)),
                matches!(conditional.alternate, Expression::ConditionalExpression(_)),
            ),
            ConditionalLike::TSConditionalType(conditional) => (
                matches!(conditional.true_type, TSType::TSConditionalType(_)),
                matches!(conditional.false_type, TSType::TSConditionalType(_)),
            ),
        };
        let is_in_chain = is_alternate_ternary || is_in_alternate;

        let is_on_same_line_as_assignment = self
            .options
            .assignment_layout
            .is_some_and(|layout| layout != AssignmentLikeLayout::BreakAfterOperator)
            && matches!(
                parent,
                AstNodes::AssignmentExpression(_)
                    | AstNodes::VariableDeclarator(_)
                    | AstNodes::PropertyDefinition(_)
                    | AstNodes::AccessorProperty(_)
                    | AstNodes::ObjectProperty(_)
            );
        let is_on_same_line_as_return =
            matches!(parent, AstNodes::ReturnStatement(_) | AstNodes::ThrowStatement(_))
                && !(is_consequent_ternary || is_alternate_ternary);

        let is_in_jsx = self.is_conditional_expression()
            && matches!(first_non_conditional_parent, AstNodes::JSXExpressionContainer(_))
            && !matches!(first_non_conditional_parent.parent(), AstNodes::JSXAttribute(_));
        let should_extra_indent = self.should_extra_indent(layout);
        let break_closing_paren = self.is_conditional_expression()
            && matches!(
                parent,
                AstNodes::StaticMemberExpression(_) | AstNodes::PrivateFieldExpression(_)
            );
        let break_ts_closing_paren = match self.conditional {
            ConditionalLike::TSConditionalType(conditional) => conditional.needs_parentheses(f),
            ConditionalLike::ConditionalExpression(_) => false,
        };

        let has_multiline_block_comments = self.has_multiline_block_comments(f);
        let should_break =
            has_multiline_block_comments || is_consequent_ternary || is_alternate_ternary;

        let printed_test =
            ReusedFormatElement(f.intern(&format_once(|f| self.format_experimental_test(f))));
        let printed_consequent =
            ReusedFormatElement(f.intern(&format_once(|f| match self.conditional {
                ConditionalLike::ConditionalExpression(conditional) => {
                    write!(f, [FormatNodeWithoutTrailingComments(conditional.consequent())]);
                    format_comments_before_operator(
                        conditional.consequent.span().end,
                        conditional.alternate.span().start,
                        b':',
                        f,
                    );
                }
                ConditionalLike::TSConditionalType(conditional) => {
                    write!(f, [FormatNodeWithoutTrailingComments(conditional.true_type())]);
                    format_comments_before_operator(
                        conditional.true_type.span().end,
                        conditional.false_type.span().start,
                        b':',
                        f,
                    );
                }
            })));
        let (alternate_comments, has_alternate_eol_directive) = self.alternate_comments(f);
        let has_alternate_comments = !alternate_comments.is_empty();
        let try_to_parenthesize_alternate = !has_alternate_eol_directive
            && !is_in_chain
            && !is_parent_ternary
            && !is_ts_conditional
            && match self.conditional {
                ConditionalLike::ConditionalExpression(conditional) if is_in_jsx => {
                    matches!(conditional.consequent, Expression::NullLiteral(_))
                }
                ConditionalLike::ConditionalExpression(conditional) => {
                    is_lone_short_argument(
                        &conditional.consequent,
                        conditional.test.span().end,
                        conditional.alternate.span().start,
                        f,
                    ) && is_simple_expression_by_node_count(&conditional.test, 3)
                }
                ConditionalLike::TSConditionalType(_) => false,
            };
        let should_group_test_and_consequent = is_in_chain
            || is_in_alternate
            || (is_ts_conditional && !is_parent_ternary)
            || (is_parent_ternary
                && self.is_conditional_expression()
                && match self.conditional {
                    ConditionalLike::ConditionalExpression(conditional) => {
                        is_simple_expression_by_node_count(&conditional.test, 1)
                    }
                    ConditionalLike::TSConditionalType(_) => false,
                })
            || try_to_parenthesize_alternate;
        let printed_alternate_comments = ReusedFormatElement(f.intern(&format_once(|f| {
            FormatDanglingComments::Comments {
                comments: alternate_comments,
                indent: DanglingIndentMode::None,
            }
            .fmt(f);
        })));
        let printed_alternate =
            ReusedFormatElement(f.intern(&format_once(|f| match self.conditional {
                ConditionalLike::ConditionalExpression(conditional) => {
                    conditional.alternate().fmt(f);
                }
                ConditionalLike::TSConditionalType(conditional) => {
                    conditional.false_type().fmt(f);
                }
            })));

        let test_id = f.group_id("conditional-test");
        let consequent_id = f.group_id("conditional-consequent");
        let test_and_consequent_id = f.group_id("conditional-test-and-consequent");
        let is_big_tabs =
            f.options().indent_width.value() > 2 || !f.options().indent_style.is_space();
        let use_tabs = !f.options().indent_style.is_space();
        let fill_width = f.options().indent_width.value().saturating_sub(1);

        let parts = format_once(|f| {
            let printed_test_with_question = format_with(|f| {
                write!(f, [printed_test.clone(), space(), "?"]);
            });
            let printed_test_with_question =
                group(&printed_test_with_question).with_group_id(Some(test_id));

            let consequent = format_with(|f| {
                let separator = if is_consequent_ternary
                    || (is_in_jsx
                        && (consequent_is_jsx(self.conditional)
                            || is_parent_ternary
                            || is_in_chain))
                {
                    hard_line_break()
                } else {
                    soft_line_break_or_space()
                };
                write!(f, [indent(&format_args!(separator, printed_consequent.clone()))]);
            });

            let test_and_consequent = format_with(|f| {
                write!(f, [printed_test_with_question]);
                if !should_group_test_and_consequent || is_in_chain {
                    write!(f, [consequent]);
                } else {
                    write!(
                        f,
                        [
                            if_group_breaks(&consequent).with_group_id(Some(test_id)),
                            if_group_fits_on_line(
                                &group(&consequent).with_group_id(Some(consequent_id))
                            )
                            .with_group_id(Some(test_id))
                        ]
                    );
                }
            });

            if should_group_test_and_consequent {
                write!(
                    f,
                    [group(&test_and_consequent).with_group_id(Some(test_and_consequent_id))]
                );
            } else {
                write!(f, [test_and_consequent]);
            }

            if has_alternate_comments {
                write!(
                    f,
                    [
                        indent(&format_args!(
                            hard_line_break(),
                            printed_alternate_comments.clone()
                        )),
                        hard_line_break()
                    ]
                );
            } else if is_alternate_ternary {
                write!(f, [hard_line_break()]);
            } else if try_to_parenthesize_alternate {
                write!(
                    f,
                    [
                        if_group_breaks(&soft_line_break_or_space())
                            .with_group_id(Some(test_and_consequent_id)),
                        if_group_fits_on_line(&space()).with_group_id(Some(test_and_consequent_id))
                    ]
                );
            } else {
                write!(f, [soft_line_break_or_space()]);
            }

            write!(f, [":"]);

            let fill_tab = format_with(|f| {
                if use_tabs {
                    write!(f, [text("\t")]);
                } else {
                    const SPACES: &str = "                                ";
                    write!(f, [text(&SPACES[..usize::from(fill_width)])]);
                }
            });

            if is_alternate_ternary || !is_big_tabs {
                write!(f, [space()]);
            } else if should_group_test_and_consequent {
                let flat_fill = format_with(|f| {
                    if is_in_chain || try_to_parenthesize_alternate {
                        write!(f, [space()]);
                    } else {
                        write!(f, [fill_tab]);
                    }
                });
                write!(
                    f,
                    [
                        if_group_breaks(&fill_tab).with_group_id(Some(test_and_consequent_id)),
                        if_group_fits_on_line(&format_args!(
                            if_group_breaks(&flat_fill),
                            if_group_fits_on_line(&space()),
                        ))
                        .with_group_id(Some(test_and_consequent_id))
                    ]
                );
            } else {
                write!(f, [if_group_breaks(&fill_tab), if_group_fits_on_line(&space())]);
            }

            let alternate_with_parens = format_with(|f| {
                let wrapped = format_with(|f| {
                    write!(
                        f,
                        [
                            if_group_breaks(&token("(")),
                            indent(&format_args!(soft_line_break(), printed_alternate.clone())),
                            soft_line_break(),
                            if_group_breaks(&token(")")),
                        ]
                    );
                });
                write!(
                    f,
                    [
                        if_group_breaks(&printed_alternate.clone())
                            .with_group_id(Some(test_and_consequent_id)),
                        if_group_fits_on_line(&dedent(&wrapped))
                            .with_group_id(Some(test_and_consequent_id))
                    ]
                );
            });

            let alternate = format_with(|f| {
                if try_to_parenthesize_alternate {
                    write!(f, [alternate_with_parens]);
                } else {
                    write!(f, [printed_alternate.clone()]);
                }
            });

            if is_alternate_ternary {
                write!(f, [alternate]);
            } else {
                write!(
                    f,
                    [group(&format_args!(
                        indent(&alternate),
                        (is_in_jsx && !try_to_parenthesize_alternate).then_some(soft_line_break())
                    ))]
                );
            }

            if break_closing_paren && !should_extra_indent {
                write!(f, [soft_line_break()]);
            }
            if should_break {
                write!(f, [expand_parent()]);
            }
        });

        if is_on_same_line_as_assignment && !should_break {
            write!(f, [group(&indent(&format_args!(soft_line_break(), group(&parts))))]);
        } else if is_on_same_line_as_assignment || is_on_same_line_as_return {
            write!(f, [group(&indent(&parts))]);
        } else if should_extra_indent || (is_ts_conditional && is_in_test) {
            write!(
                f,
                [group(&format_args!(
                    indent(&format_args!(soft_line_break(), parts)),
                    break_ts_closing_paren.then_some(soft_line_break())
                ))]
            );
        } else if std::ptr::eq(parent, first_non_conditional_parent) {
            write!(f, [group(&parts)]);
        } else {
            write!(f, [parts]);
        }
    }
}

fn consequent_is_jsx(conditional: &ConditionalLike<'_, '_>) -> bool {
    matches!(
        conditional,
        ConditionalLike::ConditionalExpression(conditional)
            if matches!(conditional.consequent, Expression::JSXElement(_) | Expression::JSXFragment(_))
    )
}

/// A bounded implementation of Prettier's generic AST-node counter. Only object-valued AST fields
/// count as children; node arrays such as arguments, elements, properties, and parameters do not.
fn is_simple_expression_by_node_count(expression: &Expression<'_>, max_count: usize) -> bool {
    fn visit_leaf(count: &mut usize, max_count: usize) -> bool {
        *count += 1;
        *count <= max_count
    }

    fn visit_optional_leaf(present: bool, count: &mut usize, max_count: usize) -> bool {
        !present || visit_leaf(count, max_count)
    }

    fn visit_child(expression: &Expression<'_>, count: &mut usize, max_count: usize) -> bool {
        visit_leaf(count, max_count) && visit_children(expression, count, max_count)
    }

    fn visit_static_member_children(
        expression: &StaticMemberExpression<'_>,
        count: &mut usize,
        max_count: usize,
    ) -> bool {
        visit_child(&expression.object, count, max_count) && visit_leaf(count, max_count)
    }

    fn visit_private_member_children(
        expression: &PrivateFieldExpression<'_>,
        count: &mut usize,
        max_count: usize,
    ) -> bool {
        visit_child(&expression.object, count, max_count) && visit_leaf(count, max_count)
    }

    fn visit_computed_member_children(
        expression: &ComputedMemberExpression<'_>,
        count: &mut usize,
        max_count: usize,
    ) -> bool {
        visit_child(&expression.object, count, max_count)
            && visit_child(&expression.expression, count, max_count)
    }

    fn visit_simple_assignment_target_children(
        target: &SimpleAssignmentTarget<'_>,
        count: &mut usize,
        max_count: usize,
    ) -> bool {
        match target {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(_) => true,
            SimpleAssignmentTarget::TSAsExpression(expression) => {
                visit_child(&expression.expression, count, max_count)
                    && visit_leaf(count, max_count)
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(expression) => {
                visit_child(&expression.expression, count, max_count)
                    && visit_leaf(count, max_count)
            }
            SimpleAssignmentTarget::TSTypeAssertion(expression) => {
                visit_leaf(count, max_count)
                    && visit_child(&expression.expression, count, max_count)
            }
            SimpleAssignmentTarget::TSNonNullExpression(expression) => {
                visit_child(&expression.expression, count, max_count)
            }
            SimpleAssignmentTarget::StaticMemberExpression(expression) => {
                visit_static_member_children(expression, count, max_count)
            }
            SimpleAssignmentTarget::PrivateFieldExpression(expression) => {
                visit_private_member_children(expression, count, max_count)
            }
            SimpleAssignmentTarget::ComputedMemberExpression(expression) => {
                visit_computed_member_children(expression, count, max_count)
            }
        }
    }

    fn visit_assignment_target(
        target: &AssignmentTarget<'_>,
        count: &mut usize,
        max_count: usize,
    ) -> bool {
        visit_leaf(count, max_count)
            && target.as_simple_assignment_target().is_none_or(|target| {
                visit_simple_assignment_target_children(target, count, max_count)
            })
    }

    fn visit_chain_element(
        element: &ChainElement<'_>,
        count: &mut usize,
        max_count: usize,
    ) -> bool {
        if !visit_leaf(count, max_count) {
            return false;
        }

        match element {
            ChainElement::CallExpression(expression) => {
                visit_child(&expression.callee, count, max_count)
                    && visit_optional_leaf(expression.type_arguments.is_some(), count, max_count)
            }
            ChainElement::TSNonNullExpression(expression) => {
                visit_child(&expression.expression, count, max_count)
            }
            ChainElement::StaticMemberExpression(expression) => {
                visit_static_member_children(expression, count, max_count)
            }
            ChainElement::PrivateFieldExpression(expression) => {
                visit_private_member_children(expression, count, max_count)
            }
            ChainElement::ComputedMemberExpression(expression) => {
                visit_computed_member_children(expression, count, max_count)
            }
        }
    }

    fn visit_jsx_member_object(
        object: &JSXMemberExpressionObject<'_>,
        count: &mut usize,
        max_count: usize,
    ) -> bool {
        visit_leaf(count, max_count)
            && match object {
                JSXMemberExpressionObject::MemberExpression(expression) => {
                    visit_jsx_member_children(expression, count, max_count)
                }
                JSXMemberExpressionObject::IdentifierReference(_)
                | JSXMemberExpressionObject::ThisExpression(_) => true,
            }
    }

    fn visit_jsx_member_children(
        expression: &JSXMemberExpression<'_>,
        count: &mut usize,
        max_count: usize,
    ) -> bool {
        visit_jsx_member_object(&expression.object, count, max_count)
            && visit_leaf(count, max_count)
    }

    fn visit_jsx_name(name: &JSXElementName<'_>, count: &mut usize, max_count: usize) -> bool {
        if !visit_leaf(count, max_count) {
            return false;
        }

        match name {
            JSXElementName::NamespacedName(_) => {
                visit_leaf(count, max_count) && visit_leaf(count, max_count)
            }
            JSXElementName::MemberExpression(expression) => {
                visit_jsx_member_children(expression, count, max_count)
            }
            JSXElementName::Identifier(_)
            | JSXElementName::IdentifierReference(_)
            | JSXElementName::ThisExpression(_) => true,
        }
    }

    fn visit_children(expression: &Expression<'_>, count: &mut usize, max_count: usize) -> bool {
        match expression {
            Expression::ArrayExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::SequenceExpression(_)
            | Expression::TemplateLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::Identifier(_)
            | Expression::ThisExpression(_)
            | Expression::Super(_)
            | Expression::ImportMeta(_)
            | Expression::NewTarget(_) => true,
            Expression::ArrowFunctionExpression(expression) => {
                visit_optional_leaf(expression.type_parameters.is_some(), count, max_count)
                    && visit_optional_leaf(expression.return_type.is_some(), count, max_count)
                    && if let Some(body) = expression.body.as_expression() {
                        visit_child(body, count, max_count)
                    } else {
                        visit_leaf(count, max_count)
                    }
            }
            Expression::AssignmentExpression(expression) => {
                visit_assignment_target(&expression.left, count, max_count)
                    && visit_child(&expression.right, count, max_count)
            }
            Expression::AwaitExpression(expression) => {
                visit_child(&expression.argument, count, max_count)
            }
            Expression::BinaryExpression(expression) => {
                visit_child(&expression.left, count, max_count)
                    && visit_child(&expression.right, count, max_count)
            }
            Expression::CallExpression(expression) => {
                visit_child(&expression.callee, count, max_count)
                    && visit_optional_leaf(expression.type_arguments.is_some(), count, max_count)
            }
            Expression::ChainExpression(expression) => {
                visit_chain_element(&expression.expression, count, max_count)
            }
            Expression::ClassExpression(expression) => {
                if !visit_optional_leaf(expression.id.is_some(), count, max_count)
                    || !visit_optional_leaf(expression.type_parameters.is_some(), count, max_count)
                {
                    return false;
                }
                if let Some(heritage) = &expression.heritage
                    && (!visit_child(&heritage.expression, count, max_count)
                        || !visit_optional_leaf(
                            heritage.type_arguments.is_some(),
                            count,
                            max_count,
                        ))
                {
                    return false;
                }
                visit_leaf(count, max_count)
            }
            Expression::LogicalExpression(expression) => {
                visit_child(&expression.left, count, max_count)
                    && visit_child(&expression.right, count, max_count)
            }
            Expression::ConditionalExpression(expression) => {
                visit_child(&expression.test, count, max_count)
                    && visit_child(&expression.consequent, count, max_count)
                    && visit_child(&expression.alternate, count, max_count)
            }
            Expression::FunctionExpression(expression) => {
                visit_optional_leaf(expression.id.is_some(), count, max_count)
                    && visit_optional_leaf(expression.type_parameters.is_some(), count, max_count)
                    && visit_optional_leaf(expression.return_type.is_some(), count, max_count)
                    && visit_optional_leaf(expression.body.is_some(), count, max_count)
            }
            Expression::ImportExpression(expression) => {
                visit_child(&expression.source, count, max_count)
                    && expression
                        .options
                        .as_ref()
                        .is_none_or(|options| visit_child(options, count, max_count))
            }
            Expression::NewExpression(expression) => {
                visit_child(&expression.callee, count, max_count)
                    && visit_optional_leaf(expression.type_arguments.is_some(), count, max_count)
            }
            Expression::ParenthesizedExpression(expression) => {
                visit_child(&expression.expression, count, max_count)
            }
            Expression::TaggedTemplateExpression(expression) => {
                visit_child(&expression.tag, count, max_count)
                    && visit_optional_leaf(expression.type_arguments.is_some(), count, max_count)
                    && visit_leaf(count, max_count)
            }
            Expression::TSAsExpression(expression) => {
                visit_child(&expression.expression, count, max_count)
                    && visit_leaf(count, max_count)
            }
            Expression::TSSatisfiesExpression(expression) => {
                visit_child(&expression.expression, count, max_count)
                    && visit_leaf(count, max_count)
            }
            Expression::TSTypeAssertion(expression) => {
                visit_leaf(count, max_count)
                    && visit_child(&expression.expression, count, max_count)
            }
            Expression::TSNonNullExpression(expression) => {
                visit_child(&expression.expression, count, max_count)
            }
            Expression::TSInstantiationExpression(expression) => {
                visit_child(&expression.expression, count, max_count)
                    && visit_leaf(count, max_count)
            }
            Expression::UnaryExpression(expression) => {
                visit_child(&expression.argument, count, max_count)
            }
            Expression::UpdateExpression(expression) => {
                visit_leaf(count, max_count)
                    && visit_simple_assignment_target_children(
                        &expression.argument,
                        count,
                        max_count,
                    )
            }
            Expression::YieldExpression(expression) => expression
                .argument
                .as_ref()
                .is_none_or(|argument| visit_child(argument, count, max_count)),
            Expression::PrivateInExpression(expression) => {
                visit_leaf(count, max_count) && visit_child(&expression.right, count, max_count)
            }
            Expression::StaticMemberExpression(expression) => {
                visit_static_member_children(expression, count, max_count)
            }
            Expression::PrivateFieldExpression(expression) => {
                visit_private_member_children(expression, count, max_count)
            }
            Expression::ComputedMemberExpression(expression) => {
                visit_computed_member_children(expression, count, max_count)
            }
            Expression::JSXElement(expression) => {
                visit_leaf(count, max_count)
                    && visit_jsx_name(&expression.opening_element.name, count, max_count)
                    && visit_optional_leaf(
                        expression.opening_element.type_arguments.is_some(),
                        count,
                        max_count,
                    )
                    && expression.closing_element.as_ref().is_none_or(|closing| {
                        visit_leaf(count, max_count)
                            && visit_jsx_name(&closing.name, count, max_count)
                    })
            }
            Expression::JSXFragment(_) => {
                visit_leaf(count, max_count) && visit_leaf(count, max_count)
            }
            Expression::V8IntrinsicExpression(_) => visit_leaf(count, max_count),
        }
    }

    let mut count = 0;
    visit_children(expression, &mut count, max_count) && count <= max_count
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatConditionalLike<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.options().experimental_ternaries {
            self.fmt_experimental(f);
            return;
        }

        let layout = self.layout(f);
        let should_extra_indent = self.should_extra_indent(layout);
        let is_jsx_chain = self.options.jsx_chain || layout.is_jsx_chain();

        let format_inner = format_with(|f| {
            self.format_test(f, layout);

            let format_tail_with_indent = format_with(|f| {
                if is_jsx_chain
                    && let ConditionalLike::ConditionalExpression(conditional) = self.conditional
                {
                    write!(
                        f,
                        [
                            space(),
                            "?",
                            space(),
                            format_jsx_chain_consequent(conditional.consequent()),
                            space(),
                            ":",
                            space(),
                            format_jsx_chain_alternate(conditional.alternate())
                        ]
                    );
                } else {
                    match &layout {
                        ConditionalLayout::Root { .. } | ConditionalLayout::NestedTest => {
                            write!(
                                f,
                                [indent(&format_with(|f| {
                                    self.format_consequent_and_alternate(f);
                                }))]
                            );
                        }
                        // This may look silly but the `dedent` is to remove the outer `align` added by the parent's formatting of the consequent.
                        // The `indent` is necessary to indent the content by one level with a tab.
                        // Adding an `indent` without the `dedent` would result in the `outer` align being converted
                        // into a `indent` + the `indent` added here, ultimately resulting in a two-level indention.
                        ConditionalLayout::NestedConsequent => {
                            write!(
                                f,
                                [dedent(&indent(&format_with(|f| {
                                    self.format_consequent_and_alternate(f);
                                })))]
                            );
                        }
                        ConditionalLayout::NestedAlternate => {
                            self.format_consequent_and_alternate(f);
                        }
                    }
                }
            });

            format_tail_with_indent.fmt(f);

            // Add a soft line break in front of the closing `)` in case the parent is a static member expression
            // ```text
            // (veryLongCondition
            //      ? a
            //      : b // <- enforce line break here if the conditional breaks
            // ).more
            // ```
            if !should_extra_indent
                && !is_jsx_chain
                && self.is_parent_static_member_expression(layout)
            {
                write!(f, [soft_line_break()]);
            }
        });

        let grouped = format_with(|f| {
            if layout.is_root() || layout.is_nested_test() {
                write!(f, [group(&format_inner)]);
            } else {
                format_inner.fmt(f);
            }
        });

        if layout.is_nested_test() || should_extra_indent {
            write!(f, [group(&soft_block_indent(&grouped))]);
        } else {
            grouped.fmt(f);
        }
    }
}

/// Formats JSX consequent with conditional wrapping
fn format_jsx_chain_consequent<'a, 'b>(
    expression: &'b AstNode<'a, Expression<'a>>,
) -> impl Format<'a, JsFormatContext<'a>> + 'b {
    FormatJsxChainExpression { expression, alternate: false }
}

/// Formats JSX alternate with conditional wrapping
fn format_jsx_chain_alternate<'a, 'b>(
    expression: &'b AstNode<'a, Expression<'a>>,
) -> impl Format<'a, JsFormatContext<'a>> + 'b {
    FormatJsxChainExpression { expression, alternate: true }
}

/// A [ConditionalExpression] that itself or any of its parent's [ConditionalExpression] have a [JSXElement]
/// as its test, consequent or alternate.
///
/// Parenthesizes the `consequent` and `alternate` if the group breaks except if the expressions are
/// * `null`
/// * `undefined`
/// * or a nested ConditionalExpression in the alternate branch
///
/// ```javascript
/// abcdefgh? (
///   <Element>
///     <Sub />
///     <Sub />
///   </Element>
/// ) : (
///   <Element2>
///     <Sub />
///     <Sub />
///   </Element2>
/// );
/// ```
struct FormatJsxChainExpression<'a, 'b> {
    expression: &'b AstNode<'a, Expression<'a>>,
    alternate: bool,
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatJsxChainExpression<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let no_wrap = match self.expression.as_ref() {
            Expression::Identifier(ident) => ident.name == "undefined",
            Expression::NullLiteral(_) => true,
            Expression::ConditionalExpression(_) if self.alternate => true,
            _ => false,
        };

        let format_expression = format_with(|f| {
            if let AstNodes::ConditionalExpression(conditional) = self.expression.as_ast_nodes() {
                FormatConditionalLike {
                    conditional: &ConditionalLike::ConditionalExpression(conditional),
                    options: FormatConditionalLikeOptions {
                        jsx_chain: true,
                        assignment_layout: None,
                    },
                }
                .fmt(f);
            } else {
                self.expression.fmt(f);
            }
        });

        if no_wrap {
            write!(f, [format_expression]);
        } else {
            write!(
                f,
                [
                    if_group_breaks(&token("(")),
                    soft_block_indent(&format_expression),
                    if_group_breaks(&token(")"))
                ]
            );
        }
    }
}
