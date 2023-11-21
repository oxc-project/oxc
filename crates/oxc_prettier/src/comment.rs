//! Comment helpers

#![allow(non_upper_case_globals)]

use bitflags::bitflags;

use oxc_ast::CommentKind;
use oxc_span::Span;

use crate::{
    doc::{Doc, Separator},
    hardline, indent, line, ss, Prettier,
};

use oxc_allocator::Vec;

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
    pub(crate) fn print_inner_comment(&mut self, range: Span) -> Vec<'a, Doc<'a>> {
        let mut parts = self.vec();
        while let Some((start, end, kind)) = self.trivias.peek().copied() {
            // Comment within the span
            if start >= range.start && end <= range.end {
                self.trivias.next();
                parts.push(self.print_comment(start, end, kind));
            } else {
                break;
            }
        }
        parts
    }

    #[must_use]
    pub(crate) fn print_leading_comments(&mut self, range: Span) -> Option<Doc<'a>> {
        let mut parts = self.vec();
        while let Some((start, end, kind)) = self.trivias.peek().copied() {
            // Comment before the span
            if end <= range.start {
                self.trivias.next();

                parts.push(self.print_comment(start, end, kind));
                if kind.is_multi_line() {
                    let line_break = if self.has_newline(end) {
                        if self.has_newline(start) {
                            hardline!()
                        } else {
                            line!()
                        }
                    } else {
                        ss!(" ")
                    };
                    parts.push(line_break);
                } else {
                    parts.push(hardline!());
                }

                if self
                    .get_comment_end(kind, end)
                    .map(|end| self.skip_spaces(end))
                    .and_then(|idx| self.skip_newline(idx))
                    .is_some_and(|i| self.has_newline(i))
                {
                    parts.push(hardline!());
                }
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

    #[allow(clippy::cast_possible_truncation)]
    fn get_comment_end(&self, kind: CommentKind, end: u32) -> Option<u32> {
        if kind.is_single_line() {
            self.source_text[end as usize..].chars().next().map(|c| end + c.len_utf8() as u32)
        } else {
            Some(end)
        }
    }
}
