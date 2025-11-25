use std::ops::Deref;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{Format, Formatter, prelude::*, trivia::FormatLeadingComments},
    write,
};
use oxc_ast::ast::*;
use oxc_span::GetSpan;

#[derive(Copy, Clone, Debug)]
pub enum CallExpressionPosition {
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
pub enum ChainMember<'a, 'b> {
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

impl ChainMember<'_, '_> {
    pub(crate) const fn is_call_expression(&self) -> bool {
        matches!(self, Self::CallExpression { .. })
    }

    pub const fn is_computed_expression(&self) -> bool {
        matches!(self, Self::ComputedMember { .. })
    }

    pub fn span(&self) -> oxc_span::Span {
        match self {
            Self::StaticMember(member) => member.span(),
            Self::TSNonNullExpression(e) => e.span(),
            Self::CallExpression { expression, .. } => expression.span(),
            Self::ComputedMember(member) => member.span(),
            Self::Node(node) => node.span(),
        }
    }
}

impl<'a> Format<'a> for ChainMember<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        match self {
            Self::StaticMember(member) => {
                write!(
                    f,
                    [
                        FormatLeadingComments::Comments(
                            f.context().comments().comments_before(member.property().span().start)
                        ),
                        member.optional().then_some("?"),
                        ".",
                        member.property()
                    ]
                );

                // `A.b /* comment */ (c)` -> `A.b(/* comment */ c)`
                if !matches!(member.parent, AstNodes::CallExpression(call) if call.type_arguments.is_none())
                {
                    member.format_trailing_comments(f);
                }
            }
            Self::TSNonNullExpression(e) => {
                e.format_leading_comments(f);
                write!(f, ["!"]);
                e.format_trailing_comments(f);
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
                    expression.format_trailing_comments(f);
                }
                CallExpressionPosition::End => {
                    write!(
                        f,
                        [
                            expression.optional().then_some("?."),
                            expression.type_arguments(),
                            expression.arguments(),
                        ]
                    );
                }
            },
            Self::ComputedMember(member) => {
                write!(f, line_suffix_boundary());
                member.format_leading_comments(f);
                FormatComputedMemberExpressionWithoutObject(member).fmt(f);
                member.format_trailing_comments(f);
            }
            Self::Node(node) => write!(f, node),
        }
    }
}

pub struct FormatComputedMemberExpressionWithoutObject<'a, 'b>(
    pub &'b AstNode<'a, ComputedMemberExpression<'a>>,
);

impl<'a> Deref for FormatComputedMemberExpressionWithoutObject<'a, '_> {
    type Target = AstNode<'a, ComputedMemberExpression<'a>>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Format<'a> for FormatComputedMemberExpressionWithoutObject<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let comments = f.context().comments().comments_before_character(self.span.start, b'[');
        if !comments.is_empty() {
            write!(f, [soft_line_break(), FormatLeadingComments::Comments(comments)]);
        }

        if matches!(self.expression, Expression::NumericLiteral(_))
            && !f.comments().has_comment_before(self.span.end)
        {
            write!(f, [self.optional().then_some("?."), "[", self.expression(), "]"]);
        } else {
            write!(
                f,
                group(&format_args!(
                    self.optional().then_some("?."),
                    "[",
                    soft_block_indent(self.expression()),
                    "]"
                ))
            );
        }
    }
}
