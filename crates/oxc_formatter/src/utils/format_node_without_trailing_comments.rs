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
        FormatNodeWithoutCommentsAfterPosition::new(self.0, self.0.span().end).fmt(f);
    }
}

/// Generic wrapper for formatting a node while hiding comments at or after `position`.
pub struct FormatNodeWithoutCommentsAfterPosition<'b, T> {
    node: &'b T,
    position: u32,
}

impl<'b, T> FormatNodeWithoutCommentsAfterPosition<'b, T> {
    pub const fn new(node: &'b T, position: u32) -> Self {
        Self { node, position }
    }
}

impl<'a, T> Format<'a, JsFormatContext<'a>> for FormatNodeWithoutCommentsAfterPosition<'_, T>
where
    T: Format<'a, JsFormatContext<'a>> + GetSpan,
{
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let node_span = self.node.span();

        if f.comments().has_trailing_suppression_comment(node_span.end) {
            format_leading_comments(node_span).fmt(f);
            FormatSuppressedNode(node_span).fmt(f);
            return;
        }

        // Save the current comment view limit and temporarily restrict it
        // to hide comments that start at or after the boundary position.
        let previous_limit = f.context_mut().comments_mut().limit_comments_up_to(self.position);

        // Format the node with the restricted comment view
        // This allows comments before the boundary to be formatted normally,
        // but hides any trailing comments that come after it.
        self.node.fmt(f);

        // Restore the previous comment view limit
        f.context_mut().comments_mut().restore_view_limit(previous_limit);
    }
}
