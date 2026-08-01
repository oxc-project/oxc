use oxc_span::GetSpan;

use crate::{
    Format,
    formatter::{
        JsFormatContext, JsFormatter, JsFormatterExt as _, trivia::format_leading_comments,
    },
    utils::suppressed::FormatSuppressedNode,
};

/// Generic wrapper for formatting a node without its trailing comments.
///
/// This wrapper temporarily hides comments that appear after the node's span end position,
/// effectively preventing trailing comments from being formatted while preserving all other
/// comment formatting behavior (leading comments, internal comments).
pub struct FormatNodeWithoutTrailingComments<'b, T>(pub &'b T);

impl<'a, T> Format<'a, JsFormatContext<'a>> for FormatNodeWithoutTrailingComments<'_, T>
where
    T: Format<'a, JsFormatContext<'a>> + GetSpan,
{
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let node_end = self.0.span().end;

        if f.comments().has_trailing_suppression_comment(node_end) {
            format_leading_comments(self.0.span()).fmt(f);
            FormatSuppressedNode(self.0.span()).fmt(f);
            return;
        }

        format_content_without_comments_after(self.0, node_end, f);
    }
}

/// Formats `content` while hiding comments that start at or after `end`,
/// for content that has no usable span (e.g. `format_with` closures) or
/// when the limit position differs from the content's span end.
///
/// Prefer [`FormatNodeWithoutTrailingComments`] when the content implements [`GetSpan`];
/// it also handles trailing suppression comments.
pub fn format_content_without_comments_after<'a, T>(
    content: &T,
    end: u32,
    f: &mut JsFormatter<'_, 'a>,
) where
    T: Format<'a, JsFormatContext<'a>>,
{
    // Save the current comment view limit and temporarily restrict it
    // to hide comments that start at or after the end position.
    let previous_limit = f.context_mut().comments_mut().limit_comments_up_to(end);

    // Format the content with the restricted comment view.
    // This allows all comments within the content's span to be formatted normally,
    // but hides any trailing comments that come after it.
    content.fmt(f);

    // Restore the previous comment view limit
    f.context_mut().comments_mut().restore_view_limit(previous_limit);
}
