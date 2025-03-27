use crate::formatter::prelude::{dynamic_text, if_group_breaks, text, write};
use crate::formatter::{
    CommentKind, CommentPlacement, CommentStyle, DecoratedComment, FormatResult, FormatRule,
    Formatter, SourceComment, SyntaxTriviaPieceComments,
};

#[derive(Default)]
pub struct FormatJsLeadingComment;

impl FormatRule<SourceComment> for FormatJsLeadingComment {
    fn fmt(&self, comment: &SourceComment, f: &mut Formatter) -> FormatResult<()> {
        let text = comment.span.source_text(f.context().source_text());
        write!(f, [dynamic_text(text, comment.span.start)])
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Default)]
pub struct JsCommentStyle;

impl CommentStyle for JsCommentStyle {
    fn is_suppression(text: &str) -> bool {
        todo!()
    }

    fn get_comment_kind(comment: &SyntaxTriviaPieceComments) -> CommentKind {
        todo!()
    }

    fn place_comment(&self, comment: DecoratedComment) -> CommentPlacement {
        todo!()
    }
}
