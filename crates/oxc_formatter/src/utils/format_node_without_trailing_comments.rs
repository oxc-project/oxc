use oxc_span::GetSpan;

use crate::{Format, formatter::Formatter};

/// Generic wrapper for formatting a node without its trailing comments.
///
/// This wrapper temporarily hides comments that appear after the node's span end position,
/// effectively preventing trailing comments from being formatted while preserving all other
/// comment formatting behavior (leading comments, internal comments).
pub struct FormatNodeWithoutTrailingComments<'b, T>(pub &'b T);

impl<'a, T> Format<'a> for FormatNodeWithoutTrailingComments<'_, T>
where
    T: Format<'a> + GetSpan,
{
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let node_end = self.0.span().end;

        // Save the current comment view limit and temporarily restrict it
        // to hide comments that start at or after the node's end position
        let previous_limit = f.context_mut().comments_mut().limit_comments_up_to(node_end);

        // Format the node with the restricted comment view
        // This allows all comments within the node's span to be formatted normally,
        // but hides any trailing comments that come after it
        self.0.fmt(f);

        // Restore the previous comment view limit
        f.context_mut().comments_mut().restore_view_limit(previous_limit);
    }
}
