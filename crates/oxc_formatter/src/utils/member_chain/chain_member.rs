use std::ops::Deref;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        JsFormatter,
        prelude::*,
        trivia::{FormatLeadingComments, format_trailing_comments},
    },
    utils::{call_expression::callee_opener, statement_body::write_comments_before_opener},
    write,
};
use oxc_ast::ast::*;
use oxc_formatter_core::Format;
use oxc_span::{GetSpan, Span};

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
    StaticMember(&'b AstNode<'b, 'a, StaticMemberExpression<'a>>),
    /// Holds onto a [oxc_ast::ast::PrivateFieldExpression], treated the same as `StaticMember`
    PrivateField(&'b AstNode<'b, 'a, PrivateFieldExpression<'a>>),
    /// Holds onto a [oxc_ast::ast::CallExpression]
    CallExpression {
        expression: &'b AstNode<'b, 'a, CallExpression<'a>>,
        position: CallExpressionPosition,
    },
    /// Holds onto a [oxc_ast::ast::ComputedMemberExpression]
    ComputedMember(&'b AstNode<'b, 'a, ComputedMemberExpression<'a>>),
    TSNonNullExpression(&'b AstNode<'b, 'a, TSNonNullExpression<'a>>),
    /// Any other node that is not one of the above
    /// Are tracked using this variant
    Node(&'b AstNode<'b, 'a, Expression<'a>>),
}

impl<'a> ChainMember<'a, '_> {
    pub(crate) const fn is_call_expression(&self) -> bool {
        matches!(self, Self::CallExpression { .. })
    }

    pub const fn is_computed_expression(&self) -> bool {
        matches!(self, Self::ComputedMember { .. })
    }

    /// The object, the property span and the property name of `a.b` or `a.#b`
    pub fn as_non_computed_member(&self) -> Option<(&'a Expression<'a>, Span, &'a str)> {
        match self {
            Self::StaticMember(member) => {
                let member = member.inner();
                Some((&member.object, member.property.span, member.property.name.as_str()))
            }
            Self::PrivateField(member) => {
                let member = member.inner();
                Some((&member.object, member.field.span, member.field.name.as_str()))
            }
            _ => None,
        }
    }

    pub fn span(&self) -> oxc_span::Span {
        match self {
            Self::StaticMember(member) => member.span(),
            Self::PrivateField(member) => member.span(),
            Self::TSNonNullExpression(e) => e.span(),
            Self::CallExpression { expression, .. } => expression.span(),
            Self::ComputedMember(member) => member.span(),
            Self::Node(node) => node.span(),
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for ChainMember<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self {
            Self::StaticMember(member) => {
                write_member_lookup(member.optional(), member.property(), f);
            }
            Self::PrivateField(member) => {
                write_member_lookup(member.optional(), member.field(), f);
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

/// Writes `.b` of `a.b` or `.#b` of `a.#b` with its comments.
fn write_member_lookup<'b, 'a, P: GetSpan>(
    optional: bool,
    property: AstNode<'b, 'a, P>,
    f: &mut JsFormatter<'_, 'a>,
) where
    AstNode<'b, 'a, P>: Format<'a, JsFormatContext<'a>>,
{
    write!(
        f,
        [
            FormatLeadingComments::Comments(
                f.context().comments().comments_before(property.span().start)
            ),
            optional.then_some("?"),
            ".",
            property
        ]
    );

    // The member expression itself
    let member = property.parent();
    // The callee of `A.b(c)` / `A.b<T>(c)`: its comments keep their side of the opener
    if let AstNodes::CallExpression(call) = member.parent()
        && !call.optional
    {
        write_comments_before_opener(member.span().end, callee_opener(call), f);
    } else {
        format_trailing_comments(
            member.parent().span(),
            member.span(),
            property.following_span_start(),
        )
        .fmt(f);
    }
}

pub struct FormatComputedMemberExpressionWithoutObject<'a, 'b>(
    pub &'b AstNode<'b, 'a, ComputedMemberExpression<'a>>,
);

impl<'a, 'b> Deref for FormatComputedMemberExpressionWithoutObject<'a, 'b> {
    type Target = AstNode<'b, 'a, ComputedMemberExpression<'a>>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatComputedMemberExpressionWithoutObject<'a, '_> {
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
