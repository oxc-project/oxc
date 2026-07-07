use oxc_ast::ast::Comment;

use crate::{
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
