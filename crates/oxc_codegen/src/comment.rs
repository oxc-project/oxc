use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashMap;

use oxc_ast::{Comment, CommentKind, ast::Argument};
use oxc_syntax::identifier::is_line_terminator;

use crate::{Codegen, LegalComment};

pub type CommentsMap = FxHashMap</* attached_to */ u32, Vec<Comment>>;

impl Codegen<'_> {
    pub(crate) fn build_comments(&mut self, comments: &[Comment]) {
        self.comments.reserve(comments.len());
        let move_legal_comments = {
            let legal_comments = &self.options.legal_comments;
            matches!(
                legal_comments,
                LegalComment::Eof | LegalComment::Linked(_) | LegalComment::External
            )
        };
        for comment in comments {
            // Omit pure comments because they are handled separately.
            if comment.is_pure() || comment.is_no_side_effects() {
                continue;
            }
            if comment.is_legal() && move_legal_comments {
                self.legal_comments.push(*comment);
            }
            self.comments.entry(comment.attached_to).or_default().push(*comment);
        }
    }

    pub(crate) fn has_comment(&self, start: u32) -> bool {
        self.comments.contains_key(&start)
    }

    pub(crate) fn contains_comment_in_call_like_expression(
        &self,
        span: Span,
        arguments: &[Argument<'_>],
    ) -> (bool, bool) {
        let has_comment_before_right_paren =
            self.print_annotation_comment && span.end > 0 && self.has_comment(span.end - 1);

        let has_comment = has_comment_before_right_paren
            || self.print_annotation_comment
                && arguments.iter().any(|item| self.has_comment(item.span().start));

        (has_comment, has_comment_before_right_paren)
    }

    /// Whether to keep leading comments.
    fn should_keep_leading_comment(comment: &Comment) -> bool {
        comment.preceded_by_newline && comment.is_annotation()
    }

    pub(crate) fn print_leading_comments(&mut self, start: u32) {
        if !self.print_any_comment {
            return;
        }
        let Some(comments) = self.comments.remove(&start) else {
            return;
        };
        let comments =
            comments.into_iter().filter(Self::should_keep_leading_comment).collect::<Vec<_>>();
        self.print_comments(&comments);
    }

    pub(crate) fn get_statement_comments(&mut self, start: u32) -> Option<Vec<Comment>> {
        let comments = self.comments.remove(&start)?;

        let mut leading_comments = vec![];

        for comment in comments {
            if comment.is_legal() {
                match &self.options.legal_comments {
                    LegalComment::None if self.options.comments => {
                        leading_comments.push(comment);
                        continue;
                    }
                    LegalComment::Inline => {
                        leading_comments.push(comment);
                        continue;
                    }
                    LegalComment::Eof | LegalComment::Linked(_) | LegalComment::External => {
                        /* noop, handled by `build_comments`. */
                        continue;
                    }
                    LegalComment::None => {}
                }
            }
            if Self::should_keep_leading_comment(&comment) {
                leading_comments.push(comment);
            }
        }

        Some(leading_comments)
    }

    /// A statement comment also includes legal comments
    #[inline]
    pub(crate) fn print_statement_comments(&mut self, start: u32) {
        if self.print_any_comment {
            if let Some(comments) = self.get_statement_comments(start) {
                self.print_comments(&comments);
            }
        }
    }

    pub(crate) fn print_expr_comments(&mut self, start: u32) -> bool {
        let Some(comments) = self.comments.remove(&start) else { return false };

        for comment in &comments {
            self.print_hard_newline();
            self.print_indent();
            self.print_comment(comment);
        }

        if comments.is_empty() {
            false
        } else {
            self.print_hard_newline();
            true
        }
    }

    pub(crate) fn print_comments(&mut self, comments: &[Comment]) {
        for (i, comment) in comments.iter().enumerate() {
            if i == 0 {
                if comment.preceded_by_newline {
                    // Skip printing newline if this comment is already on a newline.
                    if let Some(b) = self.last_byte() {
                        match b {
                            b'\n' => self.print_indent(),
                            b'\t' => { /* noop */ }
                            _ => {
                                self.print_hard_newline();
                                self.print_indent();
                            }
                        }
                    }
                } else {
                    self.print_indent();
                }
            }
            if i >= 1 {
                if comment.preceded_by_newline {
                    self.print_hard_newline();
                    self.print_indent();
                } else if comment.is_legal() {
                    self.print_hard_newline();
                }
            }
            self.print_comment(comment);
            if i == comments.len() - 1 {
                if comment.is_line() || comment.followed_by_newline {
                    self.print_hard_newline();
                } else {
                    self.print_next_indent_as_space = true;
                }
            }
        }
    }

    fn print_comment(&mut self, comment: &Comment) {
        let Some(source_text) = self.source_text else {
            return;
        };
        let comment_source = comment.span.source_text(source_text);
        match comment.kind {
            CommentKind::Line => {
                self.print_str(comment_source);
            }
            CommentKind::Block => {
                // Print block comments with our own indentation.
                let lines = comment_source.split(is_line_terminator);
                for line in lines {
                    if !line.starts_with("/*") {
                        self.print_indent();
                    }
                    self.print_str(line.trim_start());
                    if !line.ends_with("*/") {
                        self.print_hard_newline();
                    }
                }
            }
        }
    }

    pub(crate) fn try_print_eof_legal_comments(&mut self) {
        match self.options.legal_comments.clone() {
            LegalComment::Eof => {
                let comments = self.legal_comments.drain(..).collect::<Vec<_>>();
                if !comments.is_empty() {
                    self.print_hard_newline();
                }
                for c in comments {
                    self.print_comment(&c);
                    self.print_hard_newline();
                }
            }
            LegalComment::Linked(path) => {
                self.print_hard_newline();
                self.print_str("/*! For license information please see ");
                self.print_str(&path);
                self.print_str(" */");
            }
            _ => {}
        }
    }
}
