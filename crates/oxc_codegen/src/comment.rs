use rustc_hash::FxHashMap;

use oxc_ast::{Comment, CommentKind};
use oxc_syntax::identifier::is_line_terminator;

use crate::{Codegen, LegalComment};

pub type CommentsMap = FxHashMap</* attached_to */ u32, Vec<Comment>>;

impl Codegen<'_> {
    pub(crate) fn build_comments(&mut self, comments: &[Comment]) {
        if !self.options.comments
            && self.options.legal_comments.is_none()
            && !self.options.annotation_comments
        {
            return;
        }
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
            let mut add = false;
            if comment.is_legal() {
                if move_legal_comments {
                    self.legal_comments.push(*comment);
                } else if self.options.print_legal_comment() {
                    add = true;
                }
            } else if comment.is_leading() {
                if comment.is_annotation() {
                    if self.options.print_annotation_comment() {
                        add = true;
                    }
                } else if self.options.print_normal_comment() {
                    add = true;
                }
            }
            if add {
                self.comments.entry(comment.attached_to).or_default().push(*comment);
            }
        }
    }

    pub(crate) fn has_comment(&self, start: u32) -> bool {
        self.comments.contains_key(&start)
    }

    pub(crate) fn print_leading_comments(&mut self, start: u32) {
        if let Some(comments) = self.comments.remove(&start) {
            self.print_comments(&comments);
        }
    }

    pub(crate) fn get_comments(&mut self, start: u32) -> Option<Vec<Comment>> {
        if self.comments.is_empty() {
            return None;
        }
        self.comments.remove(&start)
    }

    #[inline]
    pub(crate) fn print_comments_at(&mut self, start: u32) {
        if let Some(comments) = self.get_comments(start) {
            self.print_comments(&comments);
        }
    }

    pub(crate) fn print_expr_comments(&mut self, start: u32) -> bool {
        if self.comments.is_empty() {
            return false;
        }
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
