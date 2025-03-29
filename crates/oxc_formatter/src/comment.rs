use crate::formatter::prelude::{dynamic_text, if_group_breaks, text, write};
use crate::formatter::{
    Buffer, CommentKind, CommentPlacement, CommentStyle, DecoratedComment, FormatResult,
    FormatRule, Formatter, SourceComment, SyntaxTriviaPieceComments,
};

#[derive(Default)]
pub struct FormatComment;

impl FormatRule<SourceComment> for FormatComment {
    fn fmt(&self, comment: &SourceComment, f: &mut Formatter) -> FormatResult<()> {
        // let text = comment.span.source_text(f.context().source_text());
        // write!(f, [dynamic_text(text, comment.span.start)])
        Ok(())
    }
}
