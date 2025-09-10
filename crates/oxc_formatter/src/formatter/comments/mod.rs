use std::{
    backtrace,
    cell::Cell,
    marker::PhantomData,
    ops::{ControlFlow, Deref},
};

use oxc_allocator::Vec;
use oxc_ast::{
    Comment, CommentContent, CommentKind,
    ast::{self, CallExpression, NewExpression},
};
use oxc_span::{GetSpan, Span};

use crate::{
    Format, FormatResult, SyntaxTriviaPieceComments,
    formatter::{Formatter, SourceText},
    generated::ast_nodes::SiblingNode,
};

#[derive(Debug, Clone)]
pub struct Comments<'a> {
    source_text: SourceText<'a>,
    comments: &'a Vec<'a, Comment>,
    printed_count: usize,
    // The index of the type cast comment that has been printed already.
    handled_type_cast_comment: usize,
}

impl<'a> Comments<'a> {
    pub fn new(source_text: SourceText<'a>, comments: &'a Vec<'a, Comment>) -> Self {
        Comments { source_text, comments, printed_count: 0, handled_type_cast_comment: 0 }
    }

    /// Returns comments that not printed yet.
    #[inline]
    pub fn unprinted_comments(&self) -> &'a [Comment] {
        &self.comments[self.printed_count..]
    }

    /// Returns comments that were printed already.
    #[inline]
    pub fn printed_comments(&self) -> &'a [Comment] {
        &self.comments[..self.printed_count]
    }

    /// Returns comments that are before the given `pos`.
    pub fn comments_before(&self, pos: u32) -> &'a [Comment] {
        let mut index = 0;

        let comments = self.unprinted_comments();
        for comment in comments {
            if comment.span.end > pos {
                break;
            }
            index += 1;
        }

        &comments[..index]
    }

    pub fn block_comments_before(&self, pos: u32) -> &'a [Comment] {
        let mut index = 0;

        let comments = self.unprinted_comments();
        for comment in comments {
            if comment.span.end > pos || !comment.is_block() {
                break;
            }
            index += 1;
        }

        &comments[..index]
    }

    /// Returns own line comments that are before the given `pos`.
    pub fn own_line_comments_before(&self, pos: u32) -> &'a [Comment] {
        let mut index = 0;

        let comments = self.unprinted_comments();
        for comment in comments {
            if !is_own_line_comment(comment, self.source_text) {
                break;
            }

            if comment.span.end > pos {
                break;
            }
            index += 1;
        }

        &comments[..index]
    }

    pub(crate) fn comments_before_character(&self, mut start: u32, character: u8) -> &'a [Comment] {
        let mut index = 0;
        let comments = self.comments_after(start);
        while index < comments.len() {
            let comment = &comments[index];
            if self.source_text.bytes_range(start, comment.span.start).contains(&character) {
                return &comments[..index];
            }

            start = comment.span.end;
            index += 1;
        }

        comments
    }

    /// Returns the comments that after the given `start` position, even if they were already printed.
    pub fn comments_after(&self, pos: u32) -> &'a [Comment] {
        let mut index = self.printed_count;

        // No comments or all are printed already
        if index == self.comments.len() {
            return &[];
        }

        // You may want to check comments after your are printing the node,
        // so it may have a lot of comments that don't print yet.
        //
        // Skip comments that before pos
        while index < self.comments.len() - 1 && self.comments[index].span.end < pos {
            index += 1;
        }

        if self.comments[index].span.end < pos {
            &self.comments[index + 1..]
        } else {
            &self.comments[index..]
        }
    }

    pub fn comments_between(&self, start: u32, end: u32) -> &'a [Comment] {
        let comments = self.comments_after(start);

        if comments.is_empty() {
            return &[];
        }

        let mut index = 0;
        while index < comments.len() - 1 && comments[index].span.end < end {
            index += 1;
        }

        if comments[index].span.end < end { &comments[..=index] } else { &comments[..index] }
    }

    #[inline]
    pub fn filter_comments_in_span(&self, span: Span) -> impl Iterator<Item = &Comment> {
        self.comments
            .iter()
            .skip_while(move |comment| comment.span.end < span.start)
            .take_while(move |comment| comment.span.start <= span.end)
    }

    #[inline]
    pub fn has_comments_in_span(&self, span: Span) -> bool {
        self.has_comments_between(span.start, span.end)
    }

    pub fn has_comments_between(&self, start: u32, end: u32) -> bool {
        for comment in self.unprinted_comments() {
            // Check if the comment before the span
            if start > comment.span.end {
                continue;
            }

            // Check if the comment after the span
            if comment.span.start > end {
                return false;
            }

            // Then it is a dangling comment
            return true;
        }

        false
    }

    #[inline]
    pub fn has_comments_before(&self, start: u32) -> bool {
        self.unprinted_comments()
            .iter()
            .take_while(|comment| comment.span.end <= start)
            .next()
            .is_some()
    }

    #[inline]
    pub fn has_dangling_comments(&self, span: Span) -> bool {
        self.has_comments_in_span(span)
    }

    pub fn has_leading_comments(&self, previous_end: u32, current_start: u32) -> bool {
        let comments = self.unprinted_comments();
        let mut comment_index = 0;
        while let Some(comment) = comments.get(comment_index) {
            // Check if the comment is after the previous node's span
            if comment.span.start < previous_end {
                comment_index += 1;
                continue;
            }

            // Check if the comment is before the following node's span
            if comment.span.end > current_start {
                break;
            }

            if is_own_line_comment(comment, self.source_text) {
                return true;
            } else if is_end_of_line_comment(comment, self.source_text) {
                return false;
            }

            comment_index += 1;
        }

        if comment_index == 0 {
            return false;
        }

        let last_remaining_comment = &comments[comment_index - 1];
        self.source_text.all_bytes_match(last_remaining_comment.span.end, current_start, |b| {
            b.is_ascii_whitespace() || b == b'('
        })
    }

    pub fn has_leading_own_line_comments(&self, start: u32) -> bool {
        for comment in self.unprinted_comments() {
            // Check if the comment is before the following node's span
            if comment.span.end > start {
                return false;
            }

            if is_own_line_comment(comment, self.source_text)
                || self.source_text.lines_after(comment.span.end) > 0
            {
                return true;
            }
        }

        false
    }

    pub fn has_trailing_comments(&self, current_end: u32, following_start: u32) -> bool {
        let comments = &self.comments_after(current_end);

        let mut comment_index = 0;
        while let Some(comment) = comments.get(comment_index) {
            // Check if the comment is before the following node's span
            if comment.span.end > following_start {
                break;
            }

            if is_own_line_comment(comment, self.source_text) {
                return false;
            } else if is_end_of_line_comment(comment, self.source_text) {
                return true;
            }

            comment_index += 1;
        }

        if comment_index == 0 {
            return false;
        }

        let mut gap_end = following_start;
        for cur_index in (0..comment_index).rev() {
            let comment = &comments[cur_index];
            if self.source_text.all_bytes_match(comment.span.end, gap_end, |b| {
                b.is_ascii_whitespace() || b == b'('
            }) {
                gap_end = comment.span.start;
            } else {
                return true;
            }
        }

        false
    }

    pub fn has_trailing_line_comments(&self, current_end: u32, following_start: u32) -> bool {
        for comment in self.comments_after(current_end) {
            if comment.span.start > following_start {
                return false;
            }

            if is_own_line_comment(comment, self.source_text) {
                return false;
            } else if is_end_of_line_comment(comment, self.source_text) {
                return true;
            }
        }

        false
    }

    /// Leading comments are between the `previous_span` and the `current_span`.
    /// Trailing comments are between the `current_span` and the `following_span`.
    #[inline]
    pub fn has_comments(
        &self,
        previous_end: u32,
        current_span: Span,
        following_start: u32,
    ) -> bool {
        self.has_leading_comments(previous_end, current_span.start)
            || self.has_trailing_comments(current_span.end, following_start)
    }

    #[inline]
    pub fn is_trailing_line_comment(&self, comment: &Comment) -> bool {
        comment.is_line()
            && !is_own_line_comment(comment, self.source_text)
            && is_end_of_line_comment(comment, self.source_text)
    }

    #[inline]
    pub fn increment_printed_count(&mut self) {
        self.printed_count += 1;
    }

    /// Increases the printed count by the given amount.
    #[inline]
    pub fn increase_printed_count_by(&mut self, count: usize) {
        self.printed_count += count;
    }

    pub fn get_trailing_comments(
        &self,
        enclosing_node: &SiblingNode<'a>,
        preceding_node: &SiblingNode<'a>,
        mut following_node: Option<&SiblingNode<'a>>,
    ) -> &'a [Comment] {
        if !matches!(
            enclosing_node,
            SiblingNode::Program(_)
                | SiblingNode::BlockStatement(_)
                | SiblingNode::FunctionBody(_)
                | SiblingNode::TSModuleBlock(_)
                | SiblingNode::SwitchStatement(_)
                | SiblingNode::StaticBlock(_)
        ) && matches!(following_node, Some(SiblingNode::EmptyStatement(_)))
        {
            let enclosing_span = enclosing_node.span();
            return self.comments_before(enclosing_span.end);
        }

        // The preceding_node is the callee of the call expression or new expression, let following node to print it.
        // Based on https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/comments/handle-comments.js#L726-L741
        if matches!(enclosing_node, SiblingNode::CallExpression(CallExpression { callee, ..}) | SiblingNode::NewExpression(NewExpression { callee, ..}) if callee.span().contains_inclusive(preceding_node.span()))
        {
            return &[];
        }

        let comments = self.unprinted_comments();
        if comments.is_empty() {
            return &[];
        }

        let source_text = self.source_text;
        let preceding_span = preceding_node.span();

        // All of the comments before this node are printed already.
        debug_assert!(
            comments.first().is_none_or(|comment| comment.span.end > preceding_span.start)
        );

        let Some(following_node) = following_node else {
            if let SiblingNode::ImportDeclaration(import) = enclosing_node
                && import.source.span.start > preceding_span.end
            {
                return self.comments_before(import.source.span.start);
            }

            let enclosing_span = enclosing_node.span();
            return self.comments_before(enclosing_span.end);
        };

        let following_span = following_node.span();

        let mut comment_index = 0;
        while let Some(comment) = comments.get(comment_index) {
            // Check if the comment is before the following node's span
            if comment.span.end > following_span.start {
                break;
            }

            if matches!(comment.content, CommentContent::Jsdoc)
                && self.is_type_cast_comment(comment)
            {
                break;
            }

            if is_own_line_comment(comment, source_text) {
                // TODO: describe the logic here
                // Reached an own line comment, which means it is the leading comment for the next node.

                if matches!(enclosing_node, SiblingNode::IfStatement(stmt) if stmt.test.span() == preceding_span)
                    || matches!(enclosing_node, SiblingNode::WhileStatement(stmt) if stmt.test.span() == preceding_span)
                {
                    return handle_if_and_while_statement_comments(
                        following_span.start,
                        comment_index,
                        comments,
                        source_text,
                    );
                }

                break;
            } else if is_end_of_line_comment(comment, source_text) {
                if let SiblingNode::IfStatement(if_stmt) = enclosing_node {
                    if if_stmt.consequent.span() == preceding_span {
                        // If comment is after the `else` keyword, it is not a trailing comment of consequent.
                        if source_text[preceding_span.end as usize..comment.span.start as usize]
                            .contains("else")
                        {
                            return &[];
                        }
                    }
                }

                if matches!(enclosing_node, SiblingNode::IfStatement(stmt) if stmt.test.span() == preceding_span)
                    || matches!(enclosing_node, SiblingNode::WhileStatement(stmt) if stmt.test.span() == preceding_span)
                {
                    return handle_if_and_while_statement_comments(
                        following_span.start,
                        comment_index,
                        comments,
                        source_text,
                    );
                }

                // Should be a leading comment of following node.
                // Based on https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/comments/handle-comments.js#L852-L883
                if matches!(
                    enclosing_node,
                    SiblingNode::VariableDeclarator(_)
                        | SiblingNode::AssignmentExpression(_)
                        | SiblingNode::TSTypeAliasDeclaration(_)
                ) && (comment.is_block()
                    || matches!(
                        following_node,
                        SiblingNode::ObjectExpression(_)
                            | SiblingNode::ArrayExpression(_)
                            | SiblingNode::TSTypeLiteral(_)
                            | SiblingNode::TemplateLiteral(_)
                            | SiblingNode::TaggedTemplateExpression(_)
                    ))
                {
                    return &[];
                }
                return &comments[..=comment_index];
            }

            comment_index += 1;
        }

        if comment_index == 0 {
            // No comments to print
            return &[];
        }

        if matches!(
            enclosing_node,
            SiblingNode::ImportDeclaration(_) | SiblingNode::ExportAllDeclaration(_)
        ) {
            return &comments[..comment_index];
        }

        let mut gap_end = following_span.start;
        for cur_index in (0..comment_index).rev() {
            let comment = &comments[cur_index];
            if source_text.all_bytes_match(comment.span.end, gap_end, |b| {
                b.is_ascii_whitespace() || b == b'('
            }) {
                gap_end = comment.span.start;
            } else {
                // If there is a non-whitespace character, we stop here
                return &comments[..=cur_index];
            }
        }

        &[]
    }

    /// Check whether the node has an ignore comment.
    pub fn is_suppressed(&self, start: u32) -> bool {
        self.comments_before(start).iter().any(|comment| self.is_suppressed_comment(comment))
    }

    fn is_suppressed_comment(&self, comment: &Comment) -> bool {
        // TODO: should replace `prettier-ignore` with `oxc-formatter-ignore` or something else later.
        self.source_text.text_for(&comment.content_span()).trim() == "prettier-ignore"
    }

    pub fn is_type_cast_comment(&self, comment: &Comment) -> bool {
        const TYPE_PATTERN: &[u8] = b"@type";
        const SATISFIES_PATTERN: &[u8] = b"@satisfies";

        if !matches!(comment.content, CommentContent::Jsdoc) {
            return false;
        }

        let bytes = self.source_text.text_for(&comment.span).as_bytes();
        let mut bytes_iter = bytes.iter().enumerate();
        for (i, &byte) in bytes_iter {
            if byte == b'@'
                && (matches_pattern_at(bytes, i, TYPE_PATTERN)
                    || matches_pattern_at(bytes, i, SATISFIES_PATTERN))
            {
                return true;
            }
        }
        false
    }

    /// Find the index of a type cast comment that precedes the given span.
    ///
    /// A type cast comment is a JSDoc comment (like `/** @type {string} */`) that
    /// appears immediately before an opening parenthesis `(`.
    ///
    /// This method searches through unprinted comments that end before the span starts,
    /// looking for a type cast comment followed by an opening parenthesis.
    ///
    /// Returns the index of the type cast comment in the unprinted comments array,
    /// or None if no matching type cast comment is found.
    pub fn get_type_cast_comment_index(&self, span: Span) -> Option<usize> {
        let start = span.start;

        let comments = self.unprinted_comments().iter().take_while(|c| c.span.end <= start);
        for (index, comment) in comments.enumerate() {
            if comment.span.end > start {
                return None;
            }

            if self.source_text.next_non_whitespace_byte_is(comment.span.end, b'(')
                && self.is_type_cast_comment(comment)
            {
                return Some(index);
            }
        }

        None
    }

    /// Marks the last printed type cast comment as handled.
    pub fn mark_as_handled_type_cast_comment(&mut self) {
        self.handled_type_cast_comment = self.printed_count;
    }

    /// Returns `true` if the last printed type cast comment that has been handled.
    pub fn is_already_handled_type_cast_comment(&self) -> bool {
        self.printed_count == self.handled_type_cast_comment
    }
}

fn is_word_boundary_byte(byte: Option<&u8>) -> bool {
    matches!(byte, Some(b' ' | b'\t' | b'\n' | b'\r' | b'{'))
}

fn matches_pattern_at(bytes: &[u8], pos: usize, pattern: &[u8]) -> bool {
    bytes[pos..].starts_with(pattern) && is_word_boundary_byte(bytes.get(pos + pattern.len()))
}

fn handle_if_and_while_statement_comments<'a>(
    mut end: u32,
    mut comment_index: usize,
    comments: &'a [Comment],
    source_text: SourceText,
) -> &'a [Comment] {
    // `if (a /* comment before paren */) // comment after paren`
    loop {
        let cur_comment_span = comments[comment_index].span;
        if source_text.bytes_contain(cur_comment_span.end, end, b')') {
            return &comments[..=comment_index];
        }

        end = cur_comment_span.start;

        if comment_index == 0 {
            return &[];
        }

        comment_index -= 1;
    }

    unreachable!()
}

pub fn is_own_line_comment(comment: &Comment, source_text: SourceText) -> bool {
    source_text.has_newline_before(comment.span.start)
}

pub fn is_end_of_line_comment(comment: &Comment, source_text: SourceText) -> bool {
    source_text.has_newline_after(comment.span.end)
}

/// Returns `true` if `comment` is a multi line block comment where each line
/// starts with a star (`*`). These comments can be formatted to always have
/// the leading stars line up in a column.
///
/// # Examples
///
/// ```rs,ignore
/// assert!(is_alignable_comment(&parse_comment(r#"
///     /**
///      * Multiline doc comment
///      */
/// "#)));
///
/// assert!(is_alignable_comment(&parse_comment(r#"
///     /*
///      * Single star
///      */
/// "#)));
///
///
/// // Non indentable-comments
/// assert!(!is_alignable_comment(&parse_comment(r#"/** has no line break */"#)));
///
/// assert!(!is_alignable_comment(&parse_comment(r#"
/// /*
///  *
///  this line doesn't start with a star
///  */
/// "#)));
/// ```
pub fn is_alignable_comment(source_text: &str) -> bool {
    if !source_text.contains('\n') {
        return false;
    }
    source_text.lines().enumerate().all(|(index, line)| {
        if index == 0 { line.starts_with("/*") } else { line.trim_start().starts_with('*') }
    })
}
