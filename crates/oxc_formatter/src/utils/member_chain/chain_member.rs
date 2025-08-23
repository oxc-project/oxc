use crate::{
    formatter::{Format, FormatResult, Formatter, prelude::*, trivia::FormatLeadingComments},
    generated::ast_nodes::AstNode,
    write,
};
use oxc_ast::{AstKind, ast::*};
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

    /// Check if this member has a trailing line comment on the same line
    pub fn has_same_line_trailing_comment(&self, f: &Formatter<'_, '_>) -> bool {
        // Only check for trailing comments on static members for now
        // to avoid breaking up other constructs
        match self {
            Self::StaticMember(e) => {
                let member_end = e.property().span.end;
                let source = f.source_text();

                // Look for the rest of the line after this member
                let rest_of_line = &source[member_end as usize..];

                // Find the end of the current line
                let line_end_pos = rest_of_line.find('\n').unwrap_or(rest_of_line.len());
                let line_content = &rest_of_line[..line_end_pos];

                // Check if there's a line comment on this line
                // Also check that it's not just whitespace before the comment
                if let Some(comment_pos) = line_content.find("//") {
                    let before_comment = &line_content[..comment_pos];

                    before_comment.chars().all(char::is_whitespace) && !before_comment.is_empty()
                } else {
                    false
                }
            }
            // Don't break groups for other member types for now
            _ => false,
        }
    }
}

impl<'a> Format<'a> for ChainMember<'a, '_> {
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
