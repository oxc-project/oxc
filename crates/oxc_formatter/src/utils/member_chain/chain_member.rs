use std::{iter, ops::Deref};

use crate::{
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{JsFormatter, prelude::*, trivia::FormatLeadingComments},
    utils::{call_expression::callee_opener, statement_body::write_comments_before_opener},
    write,
};
use oxc_ast::ast::*;
use oxc_formatter_core::Format;
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
pub enum ChainMember<'a> {
    /// Holds onto a [oxc_ast::ast::StaticMemberExpression]
    StaticMember(AstNode<'a, StaticMemberExpression<'a>>),

    /// Holds onto a [oxc_ast::ast::CallExpression]
    CallExpression {
        expression: AstNode<'a, CallExpression<'a>>,
        position: CallExpressionPosition,
    },

    /// Holds onto a [oxc_ast::ast::ComputedMemberExpression]
    ComputedMember(AstNode<'a, ComputedMemberExpression<'a>>),

    TSNonNullExpression(AstNode<'a, TSNonNullExpression<'a>>),

    /// Any other node that are not [oxc_ast::ast::CallExpression] or [oxc_ast::ast::StaticMemberExpression]
    /// Are tracked using this variant
    Node(AstNode<'a, Expression<'a>>),
}

impl ChainMember<'_> {
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

impl<'a> ChainMember<'a> {
    /// The node of this member.
    fn node(&self) -> AstNodes<'a> {
        match self {
            Self::StaticMember(member) => AstNodes::StaticMemberExpression(*member),
            Self::TSNonNullExpression(e) => AstNodes::TSNonNullExpression(*e),
            Self::CallExpression { expression, .. } => AstNodes::CallExpression(*expression),
            Self::ComputedMember(member) => AstNodes::ComputedMemberExpression(*member),
            Self::Node(node) => node.as_ast_nodes(),
        }
    }

    /// Whether [`Self::fmt_member`] prints the node's own `fmt`, instead of printing its children.
    fn runs_node_fmt(&self) -> bool {
        matches!(
            self,
            Self::Node(_) | Self::CallExpression { position: CallExpressionPosition::Start, .. }
        )
    }
}

/// A member is printed in the frame of the chain's root call, the current node:
/// the chain's nodes are pushed down to the member's node, or to its parent when it runs its own `fmt`.
impl<'a> Format<'a, JsFormatContext<'a>> for ChainMember<'a> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let root = f.context().ancestors().current();
        let node = self.node();
        if root.is(&node) {
            return self.fmt_member(f);
        }
        let ancestors = iter::successors(chain_object(root), |node| chain_object(*node))
            .take_while(|ancestor| !ancestor.is(&node))
            .chain((!self.runs_node_fmt()).then_some(node));
        f.with_nodes(ancestors, |f| self.fmt_member(f));
    }
}

/// The next node down a member chain: the callee of a call, the object of a member,
/// the expression of a non-null assertion or of a (transparent) `ChainExpression`.
fn chain_object(node: AstNodes<'_>) -> Option<AstNodes<'_>> {
    match node {
        AstNodes::CallExpression(call) => Some(call.callee().as_ast_nodes()),
        AstNodes::StaticMemberExpression(member) => Some(member.object().as_ast_nodes()),
        AstNodes::ComputedMemberExpression(member) => Some(member.object().as_ast_nodes()),
        AstNodes::TSNonNullExpression(e) => Some(e.expression().as_ast_nodes()),
        AstNodes::ChainExpression(chain) => Some(chain.expression().as_ast_nodes()),
        _ => None,
    }
}

impl<'a> ChainMember<'a> {
    fn fmt_member(&self, f: &mut JsFormatter<'_, 'a>) {
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

                // The callee of `A.b(c)` / `A.b<T>(c)`: its comments keep their side of the opener
                if let AstNodes::CallExpression(call) = member.parent(f)
                    && !call.optional
                {
                    write_comments_before_opener(member.span().end, callee_opener(&call), f);
                } else {
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
                FormatComputedMemberExpressionWithoutObject(*member).fmt(f);
                member.format_trailing_comments(f);
            }
            Self::Node(node) => write!(f, node),
        }
    }
}

pub struct FormatComputedMemberExpressionWithoutObject<'a>(
    pub AstNode<'a, ComputedMemberExpression<'a>>,
);

impl<'a> Deref for FormatComputedMemberExpressionWithoutObject<'a> {
    type Target = AstNode<'a, ComputedMemberExpression<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatComputedMemberExpressionWithoutObject<'a> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
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
                    soft_block_indent(&self.expression()),
                    "]"
                ))
            );
        }
    }
}
