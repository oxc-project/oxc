use std::{
    backtrace,
    cell::Cell,
    marker::PhantomData,
    ops::{ControlFlow, Deref},
};

use oxc_allocator::Vec;
use oxc_ast::{
    Comment, CommentKind,
    ast::{self, CallExpression, NewExpression},
};
use oxc_span::{GetSpan, Span};

use crate::{
    Format, FormatResult, SyntaxTriviaPieceComments,
    formatter::{
        Formatter,
        prelude::{get_lines_after, get_lines_before},
    },
    generated::ast_nodes::SiblingNode,
};

#[derive(Debug, Clone)]
pub struct Comments<'a> {
    pub source_text: &'a str,
    comments: &'a Vec<'a, Comment>,
    printed_count: usize,
}

impl<'a> Comments<'a> {
    pub fn new(source_text: &'a str, comments: &'a Vec<'a, Comment>) -> Self {
        Comments { source_text, comments, printed_count: 0 }
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
        let comments = self.unprinted_comments();
        while index < comments.len() {
            let comment = &comments[index];

            if self.source_text[start as usize..comment.span.end as usize]
                .contains(character as char)
            {
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
        let gap_str =
            Span::new(last_remaining_comment.span.end, current_start).source_text(self.source_text);

        gap_str.as_bytes().iter().all(|&b| matches!(b, b' ' | b'('))
    }

    pub fn has_leading_own_line_comments(&self, start: u32) -> bool {
        for comment in self.unprinted_comments() {
            // Check if the comment is before the following node's span
            if comment.span.end > start {
                return false;
            }

            if is_own_line_comment(comment, self.source_text)
                || get_lines_after(comment.span.end, self.source_text) > 0
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
            let gap_str = Span::new(comment.span.end, gap_end).source_text(self.source_text);
            if gap_str.as_bytes().iter().all(|&b| matches!(b, b' ' | b'(')) {
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

        // No need to print trailing comments for the most right side for `BinaryExpression` and `LogicalExpression`,
        // instead, print trailing comments for expression itself.
        if matches!(
            enclosing_node,
            SiblingNode::BinaryExpression(_) | SiblingNode::LogicalExpression(_)
        ) && matches!(following_node, Some(SiblingNode::ExpressionStatement(_)))
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
            let gap_str = Span::new(comment.span.end, gap_end).source_text(source_text);
            if gap_str.as_bytes().iter().all(|&b| matches!(b, b' ' | b'(')) {
                gap_end = comment.span.start;
            } else {
                // If there is a non-whitespace character, we stop here
                return &comments[..=cur_index];
            }
        }

        &[]
    }
}

fn handle_if_and_while_statement_comments<'a>(
    mut end: u32,
    mut comment_index: usize,
    comments: &'a [Comment],
    source_text: &'a str,
) -> &'a [Comment] {
    // `if (a /* comment before paren */) // comment after paren`
    loop {
        let cur_comment_span = comments[comment_index].span;
        if source_text.as_bytes()[cur_comment_span.end as usize..end as usize].contains(&b')') {
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

#[inline]
pub fn is_new_line(char: char) -> ControlFlow<bool> {
    if char == ' ' || char == '\t' {
        ControlFlow::Continue(())
    } else if char == '\n' || char == '\r' || char == '\u{2028}' || char == '\u{2029}' {
        ControlFlow::Break(true)
    } else {
        ControlFlow::Break(false)
    }
}

pub fn has_new_line_backward(text: &str) -> bool {
    let mut chars = text.chars().rev();

    for char in chars {
        match is_new_line(char) {
            ControlFlow::Continue(()) => {}
            ControlFlow::Break(true) => return true,
            ControlFlow::Break(false) => return false,
        }
    }

    false
}

pub fn has_new_line_forward(text: &str) -> bool {
    let mut chars = text.chars();

    for char in chars {
        match is_new_line(char) {
            ControlFlow::Continue(()) => {}
            ControlFlow::Break(true) => return true,
            ControlFlow::Break(false) => return false,
        }
    }

    false
}

pub fn is_own_line_comment(comment: &Comment, source_text: &str) -> bool {
    let start = comment.span.start;
    if start == 0 {
        return false;
    }

    has_new_line_backward(Span::sized(0, comment.span.start).source_text(source_text))
}

pub fn is_end_of_line_comment(comment: &Comment, source_text: &str) -> bool {
    let end = comment.span.end;
    has_new_line_forward(&source_text[(end as usize)..])
}

/// Formats a comment as it was in the source document
pub struct FormatPlainComment<C> {
    context: PhantomData<C>,
}

impl<C> Default for FormatPlainComment<C> {
    fn default() -> Self {
        FormatPlainComment { context: PhantomData }
    }
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

/// **TODO:** This is really JS-specific logic, both in syntax and semantics.
/// It should probably be moved to `biome_js_formatter` when possible, but is
/// currently tied to other behavior about formatting sets of comments (which
/// might also be best to move as well, since it relates to the same specific
/// behavior).
///
/// Returns `true` if `comment` is a documentation-style comment, specifically
/// matching the JSDoc format where the comment:
/// - spans over multiple lines
/// - starts with two stars (like `/**`)
///
/// This is a special case of [self::is_alignable_comment].
///
/// # Examples
///
/// ```rs,ignore
/// assert!(is_doc_comment(&parse_comment(r#"
///     /**
///      * Multiline doc comment
///      */
/// "#)));
///
/// // Non doc-comments
/// assert!(!is_doc_comment(&parse_comment(r#"
///     /*
///      * Single star
///      */
/// "#)));
///
/// assert!(!is_doc_comment(&parse_comment(r#"/** has no line break */"#)));
///
/// assert!(!is_doc_comment(&parse_comment(r#"
///     /**
///      *
///     this line doesn't start with a star
///     */
/// "#)));
/// ```
pub fn is_doc_comment(comment: &SyntaxTriviaPieceComments) -> bool {
    todo!()
    // if !comment.has_newline() {
    // return false;
    // }

    // let text = comment.text();

    // text.lines().enumerate().all(|(index, line)| {
    // if index == 0 { line.starts_with("/**") } else { line.trim_start().starts_with('*') }
    // })
}
