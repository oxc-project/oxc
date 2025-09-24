use std::{borrow::Cow, iter::FusedIterator};

use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::{Comment, CommentKind, ast::Program};
use oxc_syntax::identifier::is_line_terminator;

use crate::{
    Codegen, LegalComment,
    options::CommentOptions,
    str::{LS_LAST_2_BYTES, LS_OR_PS_FIRST_BYTE, PS_LAST_2_BYTES},
};

pub type CommentsMap = FxHashMap</* attached_to */ u32, Vec<Comment>>;

/// Custom iterator that splits text on line terminators while handling CRLF as a single unit.
/// This avoids creating empty strings between CR and LF characters.
///
/// Also splits on irregular line breaks (LS and PS).
///
/// # Example
/// Standard split would turn `"line1\r\nline2"` into `["line1", "", "line2"]` because
/// it treats `\r` and `\n` as separate terminators. This iterator correctly produces
/// `["line1", "line2"]` by treating `\r\n` as a single terminator.
struct LineTerminatorSplitter<'a> {
    text: &'a str,
}

impl<'a> LineTerminatorSplitter<'a> {
    fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl<'a> Iterator for LineTerminatorSplitter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.text.is_empty() {
            return None;
        }

        for (index, &byte) in self.text.as_bytes().iter().enumerate() {
            match byte {
                b'\n' => {
                    // SAFETY: Byte at `index` is `\n`, so `index` and `index + 1` are both UTF-8 char boundaries.
                    // Therefore, slices up to `index` and from `index + 1` are both valid `&str`s.
                    unsafe {
                        let line = self.text.get_unchecked(..index);
                        self.text = self.text.get_unchecked(index + 1..);
                        return Some(line);
                    }
                }
                b'\r' => {
                    // SAFETY: Byte at `index` is `\r`, so `index` is on a UTF-8 char boundary
                    let line = unsafe { self.text.get_unchecked(..index) };
                    // If the next byte is `\n`, consume it as well
                    let skip_bytes =
                        if self.text.as_bytes().get(index + 1) == Some(&b'\n') { 2 } else { 1 };
                    // SAFETY: `index + skip_bytes` is after `\r` or `\n`, so on a UTF-8 char boundary.
                    // Therefore slice from `index + skip_bytes` is a valid `&str`.
                    self.text = unsafe { self.text.get_unchecked(index + skip_bytes..) };
                    return Some(line);
                }
                LS_OR_PS_FIRST_BYTE => {
                    let next2: [u8; 2] = {
                        // SAFETY: 0xE2 is always the start of a 3-byte Unicode character,
                        // so there must be 2 more bytes available to consume
                        let next2 =
                            unsafe { self.text.as_bytes().get_unchecked(index + 1..index + 3) };
                        next2.try_into().unwrap()
                    };
                    // If this is LS or PS, treat it as a line terminator
                    if matches!(next2, LS_LAST_2_BYTES | PS_LAST_2_BYTES) {
                        // SAFETY: `index` is the start of a 3-byte Unicode character,
                        // so `index` and `index + 3` are both UTF-8 char boundaries.
                        // Therefore, slices up to `index` and from `index + 3` are both valid `&str`s.
                        unsafe {
                            let line = self.text.get_unchecked(..index);
                            self.text = self.text.get_unchecked(index + 3..);
                            return Some(line);
                        }
                    }
                }
                _ => {}
            }
        }

        // No line break found - return the remaining text. Next call will return `None`.
        let line = self.text;
        self.text = "";
        Some(line)
    }
}

impl FusedIterator for LineTerminatorSplitter<'_> {}

impl Codegen<'_> {
    pub(crate) fn build_comments(&mut self, comments: &[Comment]) {
        if self.options.comments == CommentOptions::disabled() {
            return;
        }
        for comment in comments {
            // Omit pure comments because they are handled separately.
            if comment.is_pure() || comment.is_no_side_effects() {
                continue;
            }
            let mut add = false;
            if comment.is_leading() {
                if comment.is_legal() && self.options.print_legal_comment() {
                    add = true;
                }
                if comment.is_jsdoc() && self.options.print_jsdoc_comment() {
                    add = true;
                }
                if comment.is_annotation() && self.options.print_annotation_comment() {
                    add = true;
                }
                if comment.is_normal() && self.options.print_normal_comment() {
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
                if comment.preceded_by_newline() {
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
                if comment.preceded_by_newline() {
                    self.print_hard_newline();
                    self.print_indent();
                } else if comment.is_legal() {
                    self.print_hard_newline();
                }
            }
            self.print_comment(comment);
            if i == comments.len() - 1 {
                if comment.is_line() || comment.followed_by_newline() {
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
                self.print_str_escaping_script_close_tag(comment_source);
            }
            CommentKind::Block => {
                for line in LineTerminatorSplitter::new(comment_source) {
                    if !line.starts_with("/*") {
                        self.print_indent();
                    }
                    self.print_str_escaping_script_close_tag(line.trim_start());
                    if !line.ends_with("*/") {
                        self.print_hard_newline();
                    }
                }
            }
        }
    }

    /// Handle Eof / Linked / External Comments.
    /// Return a list of comments of linked or external.
    pub(crate) fn handle_eof_linked_or_external_comments(
        &mut self,
        program: &Program<'_>,
    ) -> Vec<Comment> {
        let legal_comments = &self.options.comments.legal;
        if matches!(legal_comments, LegalComment::None | LegalComment::Inline) {
            return vec![];
        }

        // Dedupe legal comments for smaller output size.
        let mut set = FxHashSet::default();
        let mut comments = vec![];

        let source_text = program.source_text;
        for comment in program.comments.iter().filter(|c| c.is_legal()) {
            let mut text = Cow::Borrowed(comment.span.source_text(source_text));
            if comment.is_block() && text.contains(is_line_terminator) {
                let mut buffer = String::with_capacity(text.len());
                // Print block comments with our own indentation.
                for line in LineTerminatorSplitter::new(&text) {
                    if !line.starts_with("/*") {
                        buffer.push('\t');
                    }
                    buffer.push_str(line.trim_start());
                    if !line.ends_with("*/") {
                        buffer.push('\n');
                    }
                }
                text = Cow::Owned(buffer);
            }
            if set.insert(text) {
                comments.push(*comment);
            }
        }

        if comments.is_empty() {
            return vec![];
        }

        match legal_comments {
            LegalComment::Eof => {
                self.print_hard_newline();
                for c in comments {
                    self.print_comment(&c);
                    self.print_hard_newline();
                }
                vec![]
            }
            LegalComment::Linked(path) => {
                let path = path.clone();
                self.print_hard_newline();
                self.print_str("/*! For license information please see ");
                self.print_str(&path);
                self.print_str(" */");
                comments
            }
            LegalComment::External => comments,
            LegalComment::None | LegalComment::Inline => unreachable!(),
        }
    }
}
