use crate::{
    formatter::{
        Buffer, Format, JsFormatContext, JsFormatter, JsFormatterExt as _,
        trivia::FormatTrailingComments,
    },
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

/// Formats `content` followed by an `OptionalSemicolon`,
/// printing the content's trailing comments after the semicolon like Prettier:
/// `foo = 1 /* c */;` -> `foo = 1; /* c */`
///
/// In these statement-terminator contexts the semicolon directly follows the
/// content in the output (source parentheses like `(a = c /* c */);` are not /// re-printed around the end),
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
        // Without an actual semicolon in the source (ASI),
        // there is nothing to move the comments across;
        // the usual trailing-comments pass handles them.
        // A trailing suppression comment (`content /* oxfmt-ignore */;`) must also
        // stay visible to the content so it preserves its original text.
        if !f.comments().has_semicolon_in_range(self.content_end, self.node_end)
            || f.comments().has_trailing_suppression_comment(self.content_end)
        {
            write!(f, [self.content, OptionalSemicolon]);
            return;
        }

        let trailing_comments = f.context().comments().end_of_line_comments_after(self.content_end);

        // Hide the trailing comments while formatting the content.
        // So they are not printed before the semicolon.
        format_content_without_comments_after(self.content, self.content_end, f);

        write!(f, [OptionalSemicolon, FormatTrailingComments::Comments(trailing_comments)]);
    }
}
