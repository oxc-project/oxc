//! Comment helpers

use oxc_ast::CommentKind;
use oxc_span::Span;

use crate::{
    doc::{Doc, Separator},
    Prettier,
};

#[derive(Clone, Copy)]
#[allow(unused)]
pub enum CommentFlags {
    /// Check comment is a leading comment
    Leading,
    /// Check comment is a trailing comment
    Trailing,
    /// Check comment is a dangling comment
    Dangling,
    /// Check comment is a block comment
    Block,
    /// Check comment is a line comment
    Line,
    /// Check comment is a `prettier-ignore` comment
    PrettierIgnore,
    /// Check comment is the first attached comment
    First,
    /// Check comment is the last attached comment
    Last,
}

impl<'a> Prettier<'a> {
    #[allow(unused)]
    pub(crate) fn has_comment(_span: Span, _flags: CommentFlags) -> bool {
        false
    }

    pub(crate) fn print_dangling_comments(&mut self, range: Span) -> Option<Doc<'a>> {
        let mut parts = vec![];
        while let Some((start, end, kind)) = self.trivias.peek().copied() {
            if end <= range.end {
                parts.push(self.print_comment(start, end, kind));
                self.trivias.next();
            } else {
                break;
            }
        }
        (!parts.is_empty()).then(|| self.join(Separator::Hardline, parts))
    }

    fn print_comment(&self, start: u32, end: u32, kind: CommentKind) -> Doc<'a> {
        let end_offset = if kind.is_multi_line() { 2 } else { 0 };
        let comment = Span::new(start - 2, end + end_offset).source_text(self.source_text);
        Doc::Str(comment)
    }
}
