use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_syntax::identifier::is_line_terminator;

use crate::{
    Format, FormatResult, format_args,
    formatter::{Formatter, comments::Comments, prelude::*},
    generated::ast_nodes::AstNode,
    write,
    write::{ExpressionLeftSide, semicolon::OptionalSemicolon},
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, ReturnStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        ReturnAndThrowStatement::ReturnStatement(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ThrowStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        ReturnAndThrowStatement::ThrowStatement(self).fmt(f)
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
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.keyword())?;

        if let Some(argument) = self.argument() {
            write!(f, [space(), FormatReturnOrThrowArgument(argument)])?;
        }

        let dangling_comments = f.context().comments().comments_before(self.span().end);

        let is_last_comment_line =
            dangling_comments.last().is_some_and(|comment| comment.is_line());

        if is_last_comment_line {
            write!(f, OptionalSemicolon)?;
        }

        if !dangling_comments.is_empty() {
            write!(f, [space(), format_dangling_comments(self.span())])?;
        }

        if !is_last_comment_line {
            write!(f, OptionalSemicolon)?;
        }

        Ok(())
    }
}

pub struct FormatReturnOrThrowArgument<'a, 'b>(&'b AstNode<'a, Expression<'a>>);

impl<'a> Format<'a> for FormatReturnOrThrowArgument<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let argument = self.0;

        if !matches!(argument.as_ref(), Expression::JSXElement(_) | Expression::JSXFragment(_))
            && has_argument_leading_comments(argument, f)
        {
            write!(f, [text("("), &block_indent(&argument), text(")")])
        } else if is_binary_or_sequence_argument(argument) {
            write!(
                f,
                [group(&format_args!(
                    if_group_breaks(&text("(")),
                    soft_block_indent(&argument),
                    if_group_breaks(&text(")"))
                ))]
            )
        } else {
            write!(f, argument)
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
    let source_text = f.source_text();
    let mut current = Some(ExpressionLeftSide::from(argument));

    while let Some(left_side) = current {
        let start = left_side.span().start;
        let comments = f.comments().comments_before(start);

        let is_line_comment_or_multi_line_comment = |comments: &[Comment]| {
            comments.iter().any(|comment| {
                comment.is_line()
                    || source_text.contains_newline(comment.span)
                    || source_text.is_end_of_line_comment(comment)
            })
        };

        if is_line_comment_or_multi_line_comment(comments) {
            return true;
        }

        // This check is based on
        // <https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/comments/handle-comments.js#L335-L349>
        if let ExpressionLeftSide::Expression(left_side) = left_side {
            let has_leading_own_line_comment = match left_side.as_ref() {
                Expression::ChainExpression(chain) => {
                    if let ChainElement::StaticMemberExpression(member) = &chain.expression {
                        is_line_comment_or_multi_line_comment(
                            f.comments().comments_in_range(
                                member.object.span().end,
                                member.property.span.end,
                            ),
                        )
                    } else {
                        false
                    }
                }
                Expression::StaticMemberExpression(member) => {
                    is_line_comment_or_multi_line_comment(
                        f.comments()
                            .comments_in_range(member.object.span().end, member.property.span.end),
                    )
                }
                _ => false,
            };

            if has_leading_own_line_comment {
                return true;
            }
        }

        current = left_side.left_expression();
    }

    false
}

fn is_binary_or_sequence_argument(argument: &Expression) -> bool {
    matches!(
        argument,
        Expression::BinaryExpression(_)
            | Expression::LogicalExpression(_)
            | Expression::SequenceExpression(_)
    )
}
