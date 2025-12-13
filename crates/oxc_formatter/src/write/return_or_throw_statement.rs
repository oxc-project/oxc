use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Format,
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{Formatter, prelude::*},
    write,
    write::{ExpressionLeftSide, semicolon::OptionalSemicolon},
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, ReturnStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        ReturnAndThrowStatement::ReturnStatement(self).fmt(f);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ThrowStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        ReturnAndThrowStatement::ThrowStatement(self).fmt(f);
    }
}

/// Unified enum for statements that have an optional argument (return/throw)
pub enum ReturnAndThrowStatement<'a, 'b> {
    ReturnStatement(&'b AstNode<'a, ReturnStatement<'a>>),
    ThrowStatement(&'b AstNode<'a, ThrowStatement<'a>>),
}

impl<'a, 'b> ReturnAndThrowStatement<'a, 'b> {
    /// Get the keyword token for this statement
    fn keyword(&self) -> &'static str {
        match self {
            Self::ReturnStatement(_) => "return",
            Self::ThrowStatement(_) => "throw",
        }
    }

    /// Get the argument expression if present
    fn argument(&self) -> Option<&'b AstNode<'a, Expression<'a>>> {
        match self {
            Self::ReturnStatement(node) => node.argument(),
            Self::ThrowStatement(node) => Some(node.argument()),
        }
    }

    fn span(&self) -> Span {
        match self {
            Self::ReturnStatement(node) => node.span,
            Self::ThrowStatement(node) => node.span,
        }
    }
}

impl<'a> Format<'a> for ReturnAndThrowStatement<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, self.keyword());

        if let Some(argument) = self.argument() {
            write!(f, [space(), FormatAdjacentArgument(argument)]);
        }

        let dangling_comments = f.context().comments().comments_before(self.span().end);

        let is_last_comment_line =
            dangling_comments.last().is_some_and(|comment| comment.is_line());

        if is_last_comment_line {
            write!(f, OptionalSemicolon);
        }

        if !dangling_comments.is_empty() {
            write!(f, [space(), format_dangling_comments(self.span())]);
        }

        if !is_last_comment_line {
            write!(f, OptionalSemicolon);
        }
    }
}

pub struct FormatAdjacentArgument<'a, 'b>(pub &'b AstNode<'a, Expression<'a>>);

impl<'a> Format<'a> for FormatAdjacentArgument<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let argument = self.0;

        if !argument.is_jsx() && has_argument_leading_comments(argument, f) {
            write!(f, [token("("), &block_indent(&argument), token(")")]);
        } else if argument.is_binaryish() {
            write!(
                f,
                [group(&format_args!(
                    if_group_breaks(&token("(")),
                    soft_block_indent(&argument),
                    if_group_breaks(&token(")"))
                ))]
            );
        } else if matches!(argument.as_ref(), Expression::SequenceExpression(_)) {
            write!(f, [group(&format_args!(token("("), soft_block_indent(&argument), token(")")))]);
        } else {
            write!(f, argument);
        }
    }
}

/// Tests if the passed in argument has any leading comments. This is the case if
/// * the argument has any leading comment
/// * the argument's left side has any leading comment.
///
/// Traversing the left nodes is necessary in case the first node is parenthesized because
/// parentheses will be removed (and be re-added by the return statement, but only if the argument breaks)
fn has_argument_leading_comments(argument: &AstNode<Expression>, f: &Formatter<'_, '_>) -> bool {
    for left_side in ExpressionLeftSide::from(argument).iter() {
        let start = left_side.span().start;
        let comments = f.context().comments();
        let leading_comments = comments.comments_before(start);

        if leading_comments
            .iter()
            .any(|comment| comment.is_multiline_block() || comment.followed_by_newline())
        {
            return true;
        }

        let is_own_line_comment_or_multi_line_comment = |leading_comments: &[Comment]| {
            leading_comments
                .iter()
                .any(|comment| comment.is_multiline_block() || comment.preceded_by_newline())
        };

        // Yield expressions only need to check the leading comments on the left side.
        if matches!(argument.parent, AstNodes::YieldExpression(_)) {
            continue;
        }

        // This check is based on
        // <https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/comments/handle-comments.js#L335-L349>
        if let ExpressionLeftSide::Expression(left_side) = left_side {
            let has_leading_own_line_comment = match left_side.as_ref() {
                Expression::ChainExpression(chain) => {
                    if let ChainElement::StaticMemberExpression(member) = &chain.expression {
                        let comments = f
                            .comments()
                            .comments_in_range(member.object.span().end, member.property.span.end);
                        is_own_line_comment_or_multi_line_comment(comments)
                    } else {
                        false
                    }
                }
                Expression::StaticMemberExpression(member) => {
                    let comments = f
                        .comments()
                        .comments_in_range(member.object.span().end, member.property.span.end);
                    is_own_line_comment_or_multi_line_comment(comments)
                }
                _ => false,
            };

            if has_leading_own_line_comment {
                return true;
            }
        }
    }

    false
}
