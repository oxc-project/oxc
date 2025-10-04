use std::ops::Deref;

use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};

use crate::{
    Format, FormatResult, FormatWrite,
    formatter::{Formatter, prelude::*, trivia::FormatTrailingComments},
    generated::ast_nodes::{AstNode, AstNodes},
    utils::format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
    write,
};

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
            ConditionalLike::ConditionalExpression(expr) => expr.parent,
            ConditionalLike::TSConditionalType(ty) => ty.parent,
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
    fn is_nested_consequent(self) -> bool {
        matches!(self, Self::NestedConsequent)
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
    f: &mut Formatter<'_, 'a>,
) -> FormatResult<()> {
    let mut get_comments = |f: &mut Formatter<'_, 'a>| -> &'a [Comment] {
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
            else if comment.is_line() || source_text.is_end_of_line_comment(comment) {
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
    FormatTrailingComments::Comments(comments).fmt(f)
}

impl<'a> FormatConditionalLike<'a, '_> {
    /// Determines the layout of this conditional based on its parent
    fn layout(&self, f: &mut Formatter<'_, 'a>) -> ConditionalLayout {
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
                let jsx_chain =
                    f.context().source_type().is_jsx() && self.is_jsx_conditional_chain();
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
        let mut parent = expr.parent;

        // This tries to find the start of a member chain by iterating over all ancestors of the conditional.
        // The iteration "breaks" as soon as a non-member-chain node is found.
        loop {
            match parent {
                AstNodes::ChainExpression(chain) => {
                    if chain.expression.span() == expression_span {
                        expression_span = chain.span();
                        parent = chain.parent;
                    } else {
                        break;
                    }
                }
                AstNodes::StaticMemberExpression(member) => {
                    if member.object.span() == expression_span {
                        expression_span = member.span();
                        parent = member.parent;
                    } else {
                        break;
                    }
                }
                AstNodes::ComputedMemberExpression(member) => {
                    if member.object.span() == expression_span {
                        expression_span = member.span();
                        parent = member.parent;
                    } else {
                        break;
                    }
                }
                AstNodes::CallExpression(call) => {
                    if call.callee.span() == expression_span {
                        expression_span = call.span();
                        parent = call.parent;
                    } else {
                        break;
                    }
                }
                AstNodes::TSNonNullExpression(assertion) => {
                    if assertion.expression.span() == expression_span {
                        expression_span = assertion.span();
                        parent = assertion.parent;
                    } else {
                        break;
                    }
                }
                AstNodes::NewExpression(new_expr) => {
                    parent = new_expr.parent;
                    if new_expr.callee.span() == expression_span {
                        expression_span = new_expr.span();
                    }
                    break;
                }
                AstNodes::TSAsExpression(as_expr) => {
                    parent = as_expr.parent;
                    if as_expr.expression.span() == expression_span {
                        expression_span = as_expr.span();
                    }
                    break;
                }
                AstNodes::TSSatisfiesExpression(satisfies) => {
                    parent = satisfies.parent;
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

    /// Checks if any part of the conditional has multiline comments
    #[inline]
    fn has_multiline_comment(&self, _f: &Formatter<'_, 'a>) -> bool {
        // TODO: Implement multiline comment detection
        false
    }

    /// Returns `true` if this is the root conditional expression and the parent is a [`StaticMemberExpression`].
    #[inline]
    fn is_parent_static_member_expression(&self, layout: ConditionalLayout) -> bool {
        layout.is_root()
            && self.is_conditional_expression()
            && matches!(self.parent(), AstNodes::StaticMemberExpression(_))
    }

    /// Formats the test part of the conditional
    fn format_test<'f>(
        &self,
        f: &mut Formatter<'f, 'a>,
        layout: ConditionalLayout,
    ) -> FormatResult<()> {
        let format_inner = format_with(|f| match self.conditional {
            ConditionalLike::ConditionalExpression(conditional) => {
                write!(f, FormatNodeWithoutTrailingComments(conditional.test()))?;
                format_trailing_comments(
                    conditional.test.span().end,
                    conditional.consequent.span().start,
                    b'?',
                    f,
                )
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
                )?;

                format_trailing_comments(
                    conditional.extends_type.span().end,
                    conditional.true_type.span().start,
                    b'?',
                    f,
                )
            }
        });

        if layout.is_nested_alternate() {
            write!(f, [align(2, &format_inner)])
        } else {
            format_inner.fmt(f)
        }
    }

    /// Formats the consequent and alternate with proper formatting
    fn format_consequent_and_alternate<'f>(
        &self,
        f: &mut Formatter<'f, 'a>,
        layout: ConditionalLayout,
    ) -> FormatResult<()> {
        write!(f, [soft_line_break_or_space(), "?", space()])?;

        let format_consequent = format_with(|f| {
            let format_consequent_with_trailing_comments =
                format_once(|f| match self.conditional {
                    ConditionalLike::ConditionalExpression(conditional) => {
                        write!(f, FormatNodeWithoutTrailingComments(conditional.consequent()))?;
                        format_trailing_comments(
                            conditional.consequent.span().end,
                            conditional.alternate.span().start,
                            b':',
                            f,
                        )
                    }
                    ConditionalLike::TSConditionalType(conditional) => {
                        write!(f, FormatNodeWithoutTrailingComments(conditional.true_type()))?;
                        format_trailing_comments(
                            conditional.true_type.span().end,
                            conditional.false_type.span().start,
                            b':',
                            f,
                        )
                    }
                });

            let format_consequent_with_proper_indentation = format_with(|f| {
                if f.options().indent_style.is_space() {
                    write!(f, [align(2, &format_consequent_with_trailing_comments)])
                } else {
                    write!(f, [indent(&format_consequent_with_trailing_comments)])
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
                        if_group_fits_on_line(&text("(")),
                        format_consequent_with_proper_indentation,
                        if_group_fits_on_line(&text(")"))
                    ]
                )
            } else {
                write!(f, format_consequent_with_proper_indentation)
            }
        });

        let format_alternative = format_with(|f| match self.conditional {
            ConditionalLike::ConditionalExpression(conditional) => {
                write!(f, [conditional.alternate()])
            }
            ConditionalLike::TSConditionalType(conditional) => {
                write!(f, [conditional.false_type()])
            }
        });
        let format_alternative = format_with(|f| {
            if f.options().indent_style.is_space() {
                write!(f, [align(2, &format_alternative)])
            } else {
                write!(f, [indent(&format_alternative)])
            }
        });

        write!(f, [format_consequent, soft_line_break_or_space(), ":", space(), format_alternative])
    }
}

impl<'a> Format<'a> for ConditionalLike<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        FormatConditionalLike {
            conditional: self,
            options: FormatConditionalLikeOptions { jsx_chain: false },
        }
        .fmt(f)
    }
}

struct FormatConditionalLikeOptions {
    /// Whether the parent is a jsx conditional chain.
    /// Gets passed through from the root to the consequent and alternate of [`ConditionalExpression`]s.
    ///
    /// Doesn't apply for [`TSConditionalType`].
    jsx_chain: bool,
}

struct FormatConditionalLike<'a, 'b> {
    conditional: &'b ConditionalLike<'a, 'b>,
    options: FormatConditionalLikeOptions,
}

impl<'a, 'b> Deref for FormatConditionalLike<'a, 'b> {
    type Target = ConditionalLike<'a, 'b>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.conditional
    }
}

impl<'a> Format<'a> for FormatConditionalLike<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let layout = self.layout(f);
        let should_extra_indent = self.should_extra_indent(layout);
        let has_multiline_comment = self.has_multiline_comment(f);
        let is_jsx_chain = self.options.jsx_chain || layout.is_jsx_chain();

        let format_inner = format_with(|f| {
            self.format_test(f, layout)?;

            let format_tail_with_indent = format_once(|f| {
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
                    )?;
                } else {
                    match &layout {
                        ConditionalLayout::Root { .. } | ConditionalLayout::NestedTest => {
                            write!(
                                f,
                                [indent(&format_with(|f| {
                                    self.format_consequent_and_alternate(f, layout)
                                }))]
                            )
                        }
                        // This may look silly but the `dedent` is to remove the outer `align` added by the parent's formatting of the consequent.
                        // The `indent` is necessary to indent the content by one level with a tab.
                        // Adding an `indent` without the `dedent` would result in the `outer` align being converted
                        // into a `indent` + the `indent` added here, ultimately resulting in a two-level indention.
                        ConditionalLayout::NestedConsequent => {
                            write!(
                                f,
                                [dedent(&indent(&format_with(|f| {
                                    self.format_consequent_and_alternate(f, layout)
                                })))]
                            )
                        }
                        ConditionalLayout::NestedAlternate => {
                            self.format_consequent_and_alternate(f, layout)
                        }
                    }?;
                }

                Ok(())
            });

            format_tail_with_indent.fmt(f)?;

            // Add a soft line break in front of the closing `)` in case the parent is a static member expression
            // ```
            // (veryLongCondition
            //      ? a
            //      : b // <- enforce line break here if the conditional breaks
            // ).more
            // ```
            if !should_extra_indent
                && !is_jsx_chain
                && self.is_parent_static_member_expression(layout)
            {
                write!(f, [soft_line_break()])?;
            }

            Ok(())
        });

        let grouped = format_with(|f| {
            if layout.is_root() { write!(f, [group(&format_inner)]) } else { format_inner.fmt(f) }
        });

        if layout.is_nested_test() || should_extra_indent {
            write!(f, [group(&soft_block_indent(&grouped)).should_expand(has_multiline_comment)])
        } else {
            if has_multiline_comment {
                write!(f, [expand_parent()])?;
            }
            grouped.fmt(f)
        }
    }
}

/// Formats JSX consequent with conditional wrapping
fn format_jsx_chain_consequent<'a, 'b>(
    expression: &'b AstNode<'a, Expression<'a>>,
) -> impl Format<'a> + 'b {
    FormatJsxChainExpression { expression, alternate: false }
}

/// Formats JSX alternate with conditional wrapping
fn format_jsx_chain_alternate<'a, 'b>(
    expression: &'b AstNode<'a, Expression<'a>>,
) -> impl Format<'a> + 'b {
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

impl<'a> Format<'a> for FormatJsxChainExpression<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
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
                    options: FormatConditionalLikeOptions { jsx_chain: true },
                }
                .fmt(f)
            } else {
                FormatNodeWithoutTrailingComments(self.expression).fmt(f)
            }
        });

        if no_wrap {
            write!(f, [format_expression])
        } else {
            write!(
                f,
                [
                    if_group_breaks(&text("(")),
                    soft_block_indent(&format_expression),
                    if_group_breaks(&text(")"))
                ]
            )
        }
    }
}
