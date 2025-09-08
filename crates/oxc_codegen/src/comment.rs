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
pub(crate) struct LineTerminatorSplitter<'a> {
    text: &'a str,
}

impl<'a> LineTerminatorSplitter<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl<'a> Iterator for LineTerminatorSplitter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.text.is_empty() {
            return None;
        }

        // Line terminators will be very rare in most text. So we try to make the search as quick as possible by:
        // 1. Searching for line terminator bytes (`\r`, `\n`, `0xE2`) first, and only checking details once found.
        // 2. Searching longer strings in chunks of 16 bytes using SIMD, and only doing the
        //    more expensive byte-by-byte search once a line terminator is found.

        let bytes = self.text.as_bytes();
        let mut consumed = 0;

        // Search range of bytes for line terminators, byte by byte.
        //
        // Bytes between `ptr` and `last_ptr` (inclusive) are searched for `\r`, `\n`, or `0xE2`.
        // If found, process the line terminator and return the line.
        //
        // SAFETY:
        // * `ptr` and `last_ptr` must be within bounds of `bytes`.
        // * `last_ptr` must be greater or equal to `ptr`.
        // * For `0xE2` (LS/PS), `last_ptr` must be no later than 3 bytes before end of string.
        //   i.e. safe to read 3 bytes at `last_ptr`.
        let mut search_bytes = |mut ptr: *const u8, last_ptr| -> Option<&'a str> {
            loop {
                // SAFETY: `ptr` is always less than or equal to `last_ptr`.
                // `last_ptr` is within bounds of `bytes`, so safe to read a byte at `ptr`.
                let byte = unsafe { *ptr };
                match byte {
                    b'\n' => {
                        // SAFETY: `consumed` is initially 0, and only updated to valid UTF-8 boundaries.
                        // `index` is on `\n`, so `index` and `index + 1` are UTF-8 char boundaries.
                        unsafe {
                            let index = ptr.offset_from(bytes.as_ptr()) as usize;
                            let line = self.text.get_unchecked(consumed..index);
                            // Set `consumed` to after `\n`
                            consumed = index + 1;
                            self.text = self.text.get_unchecked(consumed..);
                            return Some(line);
                        }
                    }
                    b'\r' => {
                        // SAFETY: `consumed` is initially 0, and only updated to valid UTF-8 boundaries.
                        // `index` is on `\r`, so `index` is a UTF-8 char boundary.
                        unsafe {
                            let index = ptr.offset_from(bytes.as_ptr()) as usize;
                            let line = self.text.get_unchecked(consumed..index);
                            // Check if next byte is `\n` and consume it as well
                            let skip_bytes = if bytes.get(index + 1) == Some(&b'\n') { 2 } else { 1 };
                            // Set `consumed` to after `\r` or `\r\n`
                            consumed = index + skip_bytes;
                            self.text = self.text.get_unchecked(consumed..);
                            return Some(line);
                        }
                    }
                    LS_OR_PS_FIRST_BYTE => {
                        let next2: [u8; 2] = {
                            // SAFETY: We ensure `last_ptr` is at least 3 bytes before end,
                            // so safe to read 2 more bytes after `0xE2`
                            let next2 = unsafe {
                                let slice_ptr = ptr.add(1);
                                std::slice::from_raw_parts(slice_ptr, 2)
                            };
                            [next2[0], next2[1]]
                        };
                        // If this is LS or PS, treat it as a line terminator
                        if matches!(next2, LS_LAST_2_BYTES | PS_LAST_2_BYTES) {
                            // SAFETY: `consumed` is initially 0, and only updated to valid UTF-8 boundaries.
                            // `index` is start of 3-byte Unicode char, so `index` and `index + 3` are UTF-8 boundaries.
                            unsafe {
                                let index = ptr.offset_from(bytes.as_ptr()) as usize;
                                let line = self.text.get_unchecked(consumed..index);
                                // Set `consumed` to after the 3-byte LS/PS character
                                consumed = index + 3;
                                self.text = self.text.get_unchecked(consumed..);
                                return Some(line);
                            }
                        }
                    }
                    _ => {}
                }

                if ptr == last_ptr {
                    break;
                }
                // SAFETY: `ptr` is less than `last_ptr`, which is in bounds, so safe to increment `ptr`
                ptr = unsafe { ptr.add(1) };
            }
            None
        };

        // Search string in chunks of 16 bytes
        let mut chunks = bytes.chunks_exact(16);
        for (chunk_index, chunk) in chunks.by_ref().enumerate() {
            #[expect(clippy::missing_panics_doc, reason = "infallible")]
            let chunk: &[u8; 16] = chunk.try_into().unwrap();

            // Compiler vectorizes this loop to a few SIMD ops
            let mut contains_line_terminator = false;
            for &byte in chunk {
                if matches!(byte, b'\r' | b'\n' | LS_OR_PS_FIRST_BYTE) {
                    contains_line_terminator = true;
                    break;
                }
            }

            if contains_line_terminator {
                // Chunk contains at least one line terminator.
                // Find them and process.
                //
                // SAFETY: `index` is byte index of start of chunk.
                // We search bytes starting with first byte of chunk, and ending with last byte of chunk.
                // i.e. `index` to `index + 15` (inclusive).
                // If this chunk is towards the end of the string, reduce the range of bytes searched
                // so the last byte searched has at least 2 further bytes after it for LS/PS detection.
                // i.e. safe to read 3 bytes at `last_ptr`.
                return crate::str::cold_branch(|| unsafe {
                    let index = chunk_index * 16;
                    let remaining_bytes = bytes.len() - index;
                    let last_offset = if remaining_bytes >= 3 {
                        std::cmp::min(remaining_bytes - 3, 15)
                    } else {
                        // Not enough bytes for LS/PS, but still check for \r and \n
                        if remaining_bytes > 0 { remaining_bytes - 1 } else { 0 }
                    };
                    let ptr = bytes.as_ptr().add(index);
                    let last_ptr = ptr.add(last_offset);
                    search_bytes(ptr, last_ptr)
                });
            }
        }

        // Search last chunk byte-by-byte.
        // Skip LS/PS checks if less than 3 bytes remaining.
        let last_chunk = chunks.remainder();
        if !last_chunk.is_empty() {
            let ptr = last_chunk.as_ptr();
            let last_offset = if last_chunk.len() >= 3 {
                last_chunk.len() - 3
            } else {
                // Not enough bytes for LS/PS, but still check for \r and \n
                if last_chunk.len() > 0 { last_chunk.len() - 1 } else { 0 }
            };
            // SAFETY: `last_offset` is calculated to be in bounds of `last_chunk`.
            let last_ptr = unsafe { ptr.add(last_offset) };
            if let Some(line) = search_bytes(ptr, last_ptr) {
                return Some(line);
            }
        }

        // No line break found - return the remaining text. Next call will return `None`.
        // SAFETY: `consumed` is either 0 or set to valid UTF-8 boundaries from previous processing.
        let line = unsafe { self.text.get_unchecked(consumed..) };
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
