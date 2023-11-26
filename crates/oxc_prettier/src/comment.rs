//! Comment helpers

#![allow(non_upper_case_globals)]

use bitflags::bitflags;

use oxc_ast::CommentKind;
use oxc_span::Span;

use crate::{
    array,
    doc::{Doc, DocBuilder, Separator},
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

#[derive(Debug, Clone, Copy)]
struct Comment {
    start: u32,
    end: u32,
    is_block: bool,
    has_line_suffix: bool,
}

impl Comment {
    fn new(start: u32, end: u32, kind: CommentKind) -> Self {
        Self { start: start - 2, end, is_block: kind.is_multi_line(), has_line_suffix: false }
    }

    fn with_line_suffix(mut self, yes: bool) -> Self {
        self.has_line_suffix = yes;
        self
    }
}

impl<'a> Prettier<'a> {
    #[must_use]
    pub(crate) fn print_comments(
        &mut self,
        before: Option<Doc<'a>>,
        doc: Doc<'a>,
        after: Option<Doc<'a>>,
    ) -> Doc<'a> {
        if before.is_some() || after.is_some() {
            let mut parts = self.vec();
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

    #[allow(unused)]
    pub(crate) fn has_comment(_span: Span, _flags: CommentFlags) -> bool {
        false
    }

    #[must_use]
    pub(crate) fn print_leading_comments(&mut self, range: Span) -> Option<Doc<'a>> {
        let mut parts = self.vec();
        while let Some((start, end, kind)) = self.trivias.peek().copied() {
            let comment = Comment::new(start, end, kind);
            // Comment before the span
            if end <= range.start {
                self.trivias.next();

                parts.push(self.print_comment(comment));
                if kind.is_multi_line() {
                    let line_break = if self.has_newline(end, /* backwards */ false) {
                        if self.has_newline(start, /* backwards */ false) {
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
                    .map(|end| self.skip_spaces(end, false))
                    .and_then(|idx| self.skip_newline(idx, false))
                    .is_some_and(|i| self.has_newline(i, /* backwards */ false))
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
    pub(crate) fn print_trailing_comments(&mut self, range: Span) -> Option<Doc<'a>> {
        let mut parts = self.vec();
        let mut previous_comment: Option<Comment> = None;
        while let Some((start, end, kind)) = self.trivias.peek().copied() {
            let comment = Comment::new(start, end, kind);
            // Trailing comment if there is nothing in between.
            if range.end < comment.start
                && self.source_text[range.end as usize..comment.start as usize]
                    .chars()
                    .all(|c| c == ' ')
            {
                self.trivias.next();
                let (doc, previous) = self.print_trailing_comment(comment, previous_comment);
                previous_comment = Some(previous);
                parts.push(doc);
            } else {
                break;
            }
        }
        if parts.is_empty() {
            return None;
        }
        Some(Doc::Array(parts))
    }

    fn print_trailing_comment(
        &mut self,
        comment: Comment,
        previous: Option<Comment>,
    ) -> (Doc<'a>, Comment) {
        let printed = self.print_comment(comment);

        dbg!(self.has_newline(comment.start, /* backwards */ true));
        if previous.is_some_and(|c| c.has_line_suffix && !c.is_block)
            || self.has_newline(comment.start, /* backwards */ true)
        {
            let doc = {
                let is_line_before_empty = self.is_previous_line_empty(comment.start);
                let mut parts = self.vec();
                parts.push(hardline!());
                if is_line_before_empty {
                    parts.push(hardline!());
                }
                parts.push(printed);
                Doc::LineSuffix(parts)
            };
            return (doc, comment.with_line_suffix(true));
        }

        if !comment.is_block || previous.is_some_and(|c| c.has_line_suffix) {
            let mut parts = self.vec();
            parts.push(ss!(" "));
            parts.push(printed);
            let doc = array![self, Doc::LineSuffix(parts), Doc::BreakParent];
            return (doc, comment.with_line_suffix(true));
        }

        (array![self, ss!(" "), printed], comment.with_line_suffix(false))
    }

    #[must_use]
    pub(crate) fn print_inner_comment(&mut self, range: Span) -> Vec<'a, Doc<'a>> {
        let mut parts = self.vec();
        while let Some((start, end, kind)) = self.trivias.peek().copied() {
            let comment = Comment::new(start, end, kind);
            // Comment within the span
            if start >= range.start && end <= range.end {
                self.trivias.next();
                parts.push(self.print_comment(comment));
            } else {
                break;
            }
        }

        parts
    }

    #[must_use]
    pub(crate) fn print_dangling_comments(
        &mut self,
        range: Span,
        dangling_options: Option<DanglingCommentsPrintOptions>,
    ) -> Option<Doc<'a>> {
        let mut parts = vec![];
        while let Some((start, end, kind)) = self.trivias.peek().copied() {
            let comment = Comment::new(start, end, kind);
            // Comment within the span
            if end <= range.end {
                parts.push(self.print_comment(comment));
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
    fn print_comment(&self, comment: Comment) -> Doc<'a> {
        let end_offset = if comment.is_block { 2 } else { 0 };
        let comment =
            Span::new(comment.start, comment.end + end_offset).source_text(self.source_text);
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
