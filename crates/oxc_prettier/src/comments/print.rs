use oxc_allocator::Vec;
use oxc_span::Span;

use crate::{Prettier, ir::Doc};

use super::{CommentFlags, DanglingCommentsPrintOptions};

impl<'a> Prettier<'a> {
    #[must_use]
    pub(crate) fn print_comments(
        &self,
        before: Option<Doc<'a>>,
        doc: Doc<'a>,
        after: Option<Doc<'a>>,
    ) -> Doc<'a> {
        if before.is_some() || after.is_some() {
            let mut parts = Vec::new_in(self.allocator);
            if let Some(doc) = before {
                parts.push(doc);
            }
            parts.push(doc);
            if let Some(doc) = after {
                parts.push(doc);
            }
            return Doc::Array(parts);
        }
        doc
    }

    pub(crate) fn has_comment(&self, _span: Span, _flags: CommentFlags) -> bool {
        false
    }

    #[must_use]
    pub(crate) fn print_leading_comments(&self, _span: Span) -> Option<Doc<'a>> {
        None
    }

    #[must_use]
    pub(crate) fn print_trailing_comments(&self, _span: Span) -> Option<Doc<'a>> {
        None
    }

    #[must_use]
    pub(crate) fn print_inner_comment(&self, _span: Span) -> Vec<'a, Doc<'a>> {
        Vec::new_in(self.allocator)
    }

    #[must_use]
    pub(crate) fn print_dangling_comments(
        &self,
        _span: Span,
        _dangling_options: Option<&DanglingCommentsPrintOptions>,
    ) -> Option<Doc<'a>> {
        None
    }
}
