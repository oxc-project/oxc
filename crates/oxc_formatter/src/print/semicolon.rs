use oxc_ast::ast::{Comment, Expression};

use crate::{
    ast_nodes::AstNodes,
    formatter::{Buffer, Format, JsFormatContext, JsFormatter, trivia::FormatTrailingComments},
    options::Semicolons,
    utils::format_node_without_trailing_comments::format_content_without_comments_after,
    write,
};

pub struct OptionalSemicolon;

impl<'a> Format<'a, JsFormatContext<'a>> for OptionalSemicolon {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match f.options().semicolons {
            Semicolons::Always => write!(f, ";"),
            Semicolons::AsNeeded => (),
        }
    }
}

/// The Prettier >= 3.9 rule shared by every semicolon-terminated context:
/// same-line trailing comments between the content end and the source `;`
/// are printed behind the semicolon (`foo = 1 /* c */;` -> `foo = 1; /* c */`).
///
/// Returns the comments to print after the semicolon,
/// or `None` when the rule does not apply and the content prints as-is:
/// - no comment between the content and the end of the node (the common case),
/// - no actual `;` in the source (ASI) — there is nothing to move the comments across,
/// - a trailing suppression comment (`content /* oxfmt-ignore */;`) must
///   stay visible to the content so it preserves its original text.
///
/// `Some` may hold an empty slice (e.g. own-line comments only); the caller
/// still hides the comments from the content and a later pass prints them.
pub fn trailing_comments_to_move_behind_semicolon<'a>(
    f: &JsFormatter<'_, 'a>,
    content_end: u32,
    node_end: u32,
) -> Option<&'a [Comment]> {
    let comments = f.context().comments();
    if !comments.has_comment_in_range(content_end, node_end)
        || !comments.has_semicolon_in_range(content_end, node_end)
    {
        return None;
    }
    let trailing_comments = comments.end_of_line_comments_after(content_end);
    if trailing_comments.iter().any(|comment| comments.is_suppression_comment(comment)) {
        return None;
    }
    Some(trailing_comments)
}

/// Whether a trailing comment sitting right before the closing source paren of
/// the rightmost sub-expression stays inside the re-printed parentheses,
/// instead of moving behind the semicolon.
///
/// Mirrors Prettier's `handleParenthesizedExpressionTrailingComment` (prettier#19263):
/// a parenthesized sequence/assignment keeps the comment when it is a variable declarator initializer,
/// a `return` argument, an arrow expression body, or an assignment's right-hand side.
/// Positions where the parentheses survive in the output. (Notably not `throw` and not a plain expression statement.)
/// The sequence/assignment itself prints the comment before its closing paren.
///
/// `gated` is `true` when `expr` itself sits in one of those positions.
pub fn keeps_trailing_comment_inside_parens(expr: &Expression<'_>, gated: bool) -> bool {
    match expr {
        Expression::SequenceExpression(_) => gated,
        Expression::AssignmentExpression(assignment) => {
            gated
                || match &assignment.right {
                    Expression::SequenceExpression(_) => true,
                    right => keeps_trailing_comment_inside_parens(right, false),
                }
        }
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => arrow
            .get_expression()
            .is_some_and(|body| keeps_trailing_comment_inside_parens(body, true)),
        _ => false,
    }
}

/// The printing half of [`keeps_trailing_comment_inside_parens`]:
/// sequence/assignment call this at the end of their `write` to print the comments
/// sitting right before their closing source paren inside the parentheses.
///
/// `is_sequence` distinguishes the one asymmetric position:
/// only a sequence keeps its parens as an assignment's right-hand side.
pub fn write_trailing_comments_inside_parens<'a>(
    f: &mut JsFormatter<'_, 'a>,
    parent: &AstNodes<'a>,
    node_end: u32,
    is_sequence: bool,
) {
    let parens_survive = match parent {
        AstNodes::VariableDeclarator(_) | AstNodes::ReturnStatement(_) => true,
        AstNodes::AssignmentExpression(_) => is_sequence,
        AstNodes::ExpressionStatement(statement) => statement.is_arrow_function_body(),
        _ => false,
    };
    if parens_survive
        && let Some(comments) = f.context().comments().comments_before_closing_paren(node_end)
    {
        write!(f, FormatTrailingComments::Comments(comments));
    }
}

/// Formats `content` followed by an `OptionalSemicolon`,
/// printing the content's trailing comments after the semicolon like Prettier:
/// `foo = 1 /* c */;` -> `foo = 1; /* c */`
///
/// In these statement-terminator contexts the semicolon directly follows the content in the output
/// (source parentheses like `(a = c /* c */);` are not re-printed around the end),
/// so the comments move even from inside them.
/// Return/throw arguments differ: their parentheses survive in the output,
/// so comments inside them stay there (see `ReturnAndThrowStatement`).
///
/// Own-line comments before the semicolon are left for the next node's leading-comments pass,
/// also like Prettier.
pub struct FormatContentWithSemicolon<'b, T> {
    content: &'b T,
    /// End position of the content; comments between it and the semicolon
    /// move behind the semicolon.
    content_end: u32,
    /// End position of the whole node, after the semicolon if it exists.
    node_end: u32,
}

impl<'b, T> FormatContentWithSemicolon<'b, T> {
    pub fn new(content: &'b T, content_end: u32, node_end: u32) -> Self {
        Self { content, content_end, node_end }
    }
}

impl<'a, T> Format<'a, JsFormatContext<'a>> for FormatContentWithSemicolon<'_, T>
where
    T: Format<'a, JsFormatContext<'a>>,
{
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let Some(trailing_comments) =
            trailing_comments_to_move_behind_semicolon(f, self.content_end, self.node_end)
        else {
            write!(f, [self.content, OptionalSemicolon]);
            return;
        };

        // Hide the trailing comments while formatting the content.
        // So they are not printed before the semicolon.
        format_content_without_comments_after(self.content, self.content_end, f);

        write!(f, [OptionalSemicolon, FormatTrailingComments::Comments(trailing_comments)]);
    }
}
