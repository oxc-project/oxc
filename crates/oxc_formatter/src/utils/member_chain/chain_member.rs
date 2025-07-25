use crate::{
    formatter::{Format, FormatResult, Formatter, prelude::*, trivia::FormatLeadingComments},
    generated::ast_nodes::AstNode,
    write,
};
use oxc_ast::{AstKind, ast::*};
use oxc_span::GetSpan;

#[derive(Copy, Clone, Debug)]
pub(crate) enum CallExpressionPosition {
    /// At the start of a call chain.
    /// `of` in `of().test`
    Start,

    /// Somewhere in the middle.
    ///
    /// `b` in `a.b().c()`
    Middle,

    /// At the end of a call chain (root)
    /// `c` in `a.b.c()`
    End,
}

/// Data structure that holds the node with its formatted version
#[derive(Clone, Debug)]
pub(crate) enum ChainMember<'a, 'b> {
    /// Holds onto a [oxc_ast::ast::StaticMemberExpression]
    StaticMember(&'b AstNode<'a, StaticMemberExpression<'a>>),

    /// Holds onto a [oxc_ast::ast::CallExpression]
    CallExpression {
        expression: &'b AstNode<'a, CallExpression<'a>>,
        position: CallExpressionPosition,
    },

    /// Holds onto a [oxc_ast::ast::ComputedMemberExpression]
    ComputedMember(&'b AstNode<'a, ComputedMemberExpression<'a>>),

    TSNonNullExpression(&'b AstNode<'a, TSNonNullExpression<'a>>),

    /// Any other node that are not [oxc_ast::ast::CallExpression] or [oxc_ast::ast::StaticMemberExpression]
    /// Are tracked using this variant
    Node(&'b AstNode<'a, Expression<'a>>),
}

impl<'a, 'b> ChainMember<'a, 'b> {
    /// checks if the current node is a [oxc_ast::ast::CallExpression], or a [oxc_ast::ast::ImportExpression]
    pub fn is_call_like_expression(&self) -> bool {
        match self {
            Self::CallExpression { .. } => true,
            Self::Node(node) => {
                matches!(
                    node.as_ref(),
                    Expression::ImportExpression(_) | Expression::CallExpression(_)
                )
            }
            _ => false,
        }
    }

    pub(crate) const fn is_call_expression(&self) -> bool {
        matches!(self, Self::CallExpression { .. })
    }

    pub const fn is_computed_expression(&self) -> bool {
        matches!(self, Self::ComputedMember { .. })
    }
}

impl<'a, 'b> Format<'a> for ChainMember<'a, 'b> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::StaticMember(e) => {
                write!(
                    f,
                    [
                        FormatLeadingComments::Comments(
                            f.context().comments().comments_before(e.property().span().start)
                        ),
                        e.optional().then_some("?"),
                        ".",
                        e.property()
                    ]
                )?;
                e.format_trailing_comments(f)
            }

            Self::TSNonNullExpression(e) => {
                e.format_leading_comments(f)?;
                write!(f, ["!"])?;
                e.format_trailing_comments(f)
            }

            Self::CallExpression { expression, position } => match *position {
                CallExpressionPosition::Start => write!(f, expression),
                CallExpressionPosition::Middle => {
                    expression.format_leading_comments(f);
                    write!(
                        f,
                        [
                            expression.optional().then_some("?."),
                            expression.type_arguments(),
                            expression.arguments()
                        ]
                    );
                    expression.format_trailing_comments(f)
                }
                CallExpressionPosition::End => {
                    write!(
                        f,
                        [
                            expression.optional().then_some("?."),
                            expression.type_arguments(),
                            expression.arguments(),
                        ]
                    )
                }
            },
            Self::ComputedMember(e) => {
                e.format_leading_comments(f)?;
                write!(f, [e.optional().then_some("?"), "[", e.expression(), "]"])?;
                e.format_trailing_comments(f)
            }
            Self::Node(node) => write!(f, node),
        }
    }
}
