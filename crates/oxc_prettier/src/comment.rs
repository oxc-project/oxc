#![allow(non_upper_case_globals)]

//! Comment helpers

use bitflags::bitflags;

use oxc_ast::CommentKind;
use oxc_span::Span;

use crate::{
    doc::{Doc, Separator},
    hardline, indent, Prettier,
};

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct CommentFlags: u8 {
        const Leading        = 1 << 0; // Check comment is a leading comment
        const Trailing       = 1 << 1; // Check comment is a trailing comment
        const Dangling       = 1 << 2; // Check comment is a dangling comment
        const Block          = 1 << 3; // Check comment is a block comment
        const Line           = 1 << 4; // Check comment is a line comment
        const PrettierIgnore = 1 << 5; // Check comment is a `prettier-ignore` comment
        const First          = 1 << 6; // Check comment is the first attached comment
        const Last           = 1 << 7; // Check comment is the last attached comment
    }
}

#[derive(Default)]
pub struct DanglingCommentsPrintOptions {
    ident: bool,
}

impl DanglingCommentsPrintOptions {
    pub(crate) fn with_ident(mut self, ident: bool) -> Self {
        self.ident = ident;
        self
    }
}

impl<'a> Prettier<'a> {
    #[allow(unused)]
    pub(crate) fn has_comment(_span: Span, _flags: CommentFlags) -> bool {
        false
    }

    #[must_use]
    pub(crate) fn print_leading_comments(&mut self, range: Span) -> Option<Doc<'a>> {
        let mut parts = self.vec();
        while let Some((start, end, kind)) = self.trivias.peek().copied() {
            // Comment before the span
            if end <= range.start {
                parts.push(self.print_comment(start, end, kind));
                if kind.is_multi_line() {
                    parts.push(hardline!());
                }
                self.trivias.next();
            } else {
                break;
            }
        }
        if parts.is_empty() {
            return None;
        }
        Some(Doc::Array(parts))
    }

    #[must_use]
    pub(crate) fn print_dangling_comments(
        &mut self,
        range: Span,
        dangling_options: Option<DanglingCommentsPrintOptions>,
    ) -> Option<Doc<'a>> {
        let mut parts = vec![];
        while let Some((start, end, kind)) = self.trivias.peek().copied() {
            // Comment within the span
            if end <= range.end {
                parts.push(self.print_comment(start, end, kind));
                self.trivias.next();
            } else {
                break;
            }
        }
        (!parts.is_empty()).then(|| Doc::Array(self.join(Separator::Hardline, parts))).map(|doc| {
            if dangling_options.is_some_and(|options| options.ident) {
                indent!(self, hardline!(), doc)
            } else {
                doc
            }
        })
    }

    #[must_use]
    fn print_comment(&self, start: u32, end: u32, kind: CommentKind) -> Doc<'a> {
        let end_offset = if kind.is_multi_line() { 2 } else { 0 };
        let comment = Span::new(start - 2, end + end_offset).source_text(self.source_text);
        Doc::Str(comment)
    }
}
