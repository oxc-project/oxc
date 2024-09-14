use oxc_allocator::Vec;
use oxc_span::Span;

use super::{Comment, CommentFlags, DanglingCommentsPrintOptions};
use crate::{
    array,
    doc::{Doc, DocBuilder, Separator},
    hardline, line, space, Prettier,
};

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

    pub(crate) fn has_comment(&mut self, range: Span, flags: CommentFlags) -> bool {
        let mut peekable_trivias = self.trivias.clone();

        while let Some(comment) = peekable_trivias.peek().copied() {
            let mut should_break = true;
            let comment = Comment::new(comment);

            if range.end < comment.start
                && self.source_text[range.end as usize..comment.start as usize]
                    .chars()
                    .all(|c| c == ' ')
            {
                if flags.contains(CommentFlags::Trailing) && comment.matches_flags(flags) {
                    return true;
                }

                should_break = false;
            }

            if comment.end <= range.end {
                if flags.contains(CommentFlags::Dangling) && comment.matches_flags(flags) {
                    return true;
                }
                should_break = false;
            }

            if should_break {
                break;
            }
            peekable_trivias.next();
        }

        false
    }

    #[must_use]
    pub(crate) fn print_leading_comments(&mut self, range: Span) -> Option<Doc<'a>> {
        let mut parts = self.vec();
        while let Some(comment) = self.trivias.peek().copied() {
            let comment = Comment::new(comment);
            // Comment before the span
            if comment.end <= range.start {
                self.trivias.next();
                self.print_leading_comment(&mut parts, comment);
            } else {
                break;
            }
        }
        if parts.is_empty() {
            return None;
        }
        Some(Doc::Array(parts))
    }

    fn print_leading_comment(&mut self, parts: &mut Vec<'a, Doc<'a>>, comment: Comment) {
        let printed = self.print_comment(comment);
        parts.push(printed);

        if comment.is_block {
            if self.has_newline(comment.end, /* backwards */ false) {
                if self.has_newline(comment.start, /* backwards */ true) {
                    parts.extend(hardline!());
                } else {
                    parts.push(line!());
                }
            } else {
                parts.push(space!());
            };
        } else {
            parts.extend(hardline!());
        }

        if self
            .skip_spaces(Some(comment.end), false)
            .and_then(|idx| self.skip_newline(Some(idx), false))
            .is_some_and(|i| self.has_newline(i, /* backwards */ false))
        {
            parts.extend(hardline!());
        }
    }

    #[must_use]
    pub(crate) fn print_trailing_comments(&mut self, range: Span) -> Option<Doc<'a>> {
        let mut parts = self.vec();
        let mut previous_comment: Option<Comment> = None;
        while let Some(comment) = self.trivias.peek().copied() {
            let comment = Comment::new(comment);
            // Trailing comment if there is nothing in between.
            if range.end < comment.start
                && self.source_text[range.end as usize..comment.start as usize]
                    .chars()
                    .all(|c| c == ' ')
            {
                self.trivias.next();
                let previous = self.print_trailing_comment(&mut parts, comment, previous_comment);
                previous_comment = Some(previous);
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
        parts: &mut Vec<'a, Doc<'a>>,
        comment: Comment,
        previous: Option<Comment>,
    ) -> Comment {
        let printed = self.print_comment(comment);

        if previous.is_some_and(|c| c.has_line_suffix && !c.is_block)
            || self.has_newline(comment.start, /* backwards */ true)
        {
            parts.push(printed);
            let suffix = {
                let mut parts = self.vec();
                parts.extend(hardline!());
                if self.is_previous_line_empty(comment.start) {
                    parts.extend(hardline!());
                }
                parts
            };
            parts.push(Doc::LineSuffix(suffix));
            return comment.with_line_suffix(true);
        }

        if !comment.is_block || previous.is_some_and(|c| c.has_line_suffix) {
            let suffix = {
                let mut parts = self.vec();
                parts.push(space!());
                parts.push(printed);
                parts
            };
            let doc = array![self, Doc::LineSuffix(suffix), Doc::BreakParent];
            parts.push(doc);
            return comment.with_line_suffix(true);
        }

        let doc = array![self, space!(), printed];
        parts.push(doc);
        comment.with_line_suffix(false)
    }

    #[must_use]
    pub(crate) fn print_inner_comment(&mut self, range: Span) -> Vec<'a, Doc<'a>> {
        let mut parts = self.vec();
        while let Some(comment) = self.trivias.peek().copied() {
            let comment = Comment::new(comment);
            // Comment within the span
            if comment.start >= range.start && comment.end <= range.end {
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
        while let Some(comment) = self.trivias.peek().copied() {
            let comment = Comment::new(comment);
            // Comment within the span
            if comment.end <= range.end {
                parts.push(self.print_comment(comment));
                self.trivias.next();
            } else {
                break;
            }
        }
        (!parts.is_empty()).then(|| Doc::Array(self.join(Separator::Hardline, parts))).map(|doc| {
            if dangling_options.is_some_and(|options| options.ident) {
                Doc::Indent({
                    let mut parts = self.vec();
                    parts.extend(hardline!());
                    parts.push(doc);
                    parts
                })
            } else {
                doc
            }
        })
    }

    #[must_use]
    fn print_comment(&self, comment: Comment) -> Doc<'a> {
        Doc::Str(Span::new(comment.start, comment.end).source_text(self.source_text))
    }
}
