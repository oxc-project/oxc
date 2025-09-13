//! On-demand comment processing architecture for the formatter.
//!
//! This module implements a fundamentally different approach to comment handling compared to
//! traditional formatters like Prettier. Instead of pre-processing and attaching comments to
//! AST nodes, we process comments on-demand during formatting using a cursor-based system.
//!
//! ## Core Architecture: On-Demand vs Pre-processes
//!
//! ### Why Not Pre-processes (like Prettier)?
//! Prettier's approach pre-processes all comments and attaches them to AST nodes before formatting:
//!
//! **Performance Issues:**
//! - **HashMap overhead**: Inserting comments into hash maps significantly hurts performance
//! - **AST visitor cost**: Requires an additional full AST traversal to process comments
//! - **Memory allocation**: Every node potentially stores comment arrays
//!
//! **Association Complexity:**
//! - **Hard to associate comments with nodes**: Determining which node "owns" a comment requires intricate heuristics
//! - **Edge case handling**: Many special cases require complex pre-processing logic
//!
//! ### Our On-Demand Approach
//! We process comments lazily during formatting using position-based queries:
//!
//! **Performance Benefits:**
//! - **No hash map overhead**: We directly use the parser's comment array, no additional data structures
//! - **Single AST pass**: No additional traversal needed for comment processing
//! - **Zero memory allocation**: No comment storage on AST nodes, just reference the parser's data
//! - **Lazy evaluation**: Only process comments that actually need formatting
//!
//! **Simpler Association:**
//! - **Position-based logic**: Comments found by source position, not ownership rules
//! - **Flexible categorization**: Comment roles determined dynamically during formatting
//!
//! ### Trade-offs of On-Demand
//! **Complexity Cost:**
//! - **More utility methods needed**: Requires many comment-checking utilities (as seen in this module)
//! - **Distributed logic**: Comment handling spread across formatter instead of centralized
//! - **Extra checking work**: Each node must query and check for relevant comments
//!
//! ## How We Print Comments
//!
//! ### Leading Comments
//! **When**: Before formatting an AST node
//! **How**: Query `comments_before(node.span.start)` to find all unprinted comments that end at or before the node
//! **Logic**: Print each comment with appropriate spacing, then mark as printed
//! ```javascript
//! // Leading comment 1
//! /* Leading comment 2 */
//! function example() {}
//! ```
//!
//! ### Trailing Comments
//! **When**: After formatting an AST node
//! **How**: Use complex logic in `get_trailing_comments()` to determine which comments belong to this node vs following nodes
//! **Logic**: Consider comment position, type, and context to avoid "stealing" comments from following nodes
//! ```javascript
//! const a = 1, // Trailing comment for 'a'
//!       b = 2; // This could be trailing for 'a' or 'b' - complex ownership logic needed
//! ```
//!
//! ### Dangling Comments
//! **When**: Inside container nodes (objects, arrays, blocks) that are empty or have no children to attach comments to
//! **How**: Query `comments_between()` to find comments within the container span when the container is empty
//! **Logic**: Format with appropriate indentation within the empty container
//! ```javascript
//! {
//!   // Dangling comment in empty object
//! }
//!
//! [
//!   // Dangling comment in empty array
//! ]
//! ```
//!
//! ## Avoiding Comment Re-printing
//!
//! ### The `printed_count` Cursor System
//! The key to avoiding duplicate comments is our cursor-based tracking:
//!
//! 1. **All comments are sorted by position** in the original array
//! 2. **`printed_count` acts as a cursor** dividing processed from unprocessed comments
//! 3. **Query methods only search unprinted comments** (`comments[printed_count..]`)
//! 4. **After printing, we advance the cursor** to mark comments as processed
//!
//! ```md
//! Comments: [A, B, C, D, E, F]
//!           └─processed─┘ └─unprocessed─┘
//!           printed_count = 3
//! ```
//!
//! ### Why This Works
//! - **Sequential processing**: Comments are always processed in source order
//! - **No double-processing**: Once printed, comments are never considered again
//! - **Efficient queries**: We only search the remaining unprocessed comments
//! - **Simple state management**: Single counter tracks entire system state
//!
//! ## References
//! - [Prettier handles special comments](https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/comments/handle-comments.js)
//! - [Prettier pre-processes comments](https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/main/comments/attach.js)
use std::ops::{ControlFlow, Deref};

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
    comments: &'a [Comment],
    /// **Critical state field**: Tracks how many comments have been processed.
    ///
    /// This acts as a cursor dividing the comments array into two sections:
    /// - `comments[0..printed_count]`: Already formatted (must not be processed again)
    /// - `comments[printed_count..]`: Available for processing
    ///
    /// This field MUST be incremented each time a comment is formatted to maintain
    /// system integrity. All comment query methods rely on this for correctness.
    printed_count: usize,
    /// The index of the type cast comment that has been printed already.
    /// Used to prevent duplicate processing of special TypeScript type cast comments.
    handled_type_cast_comment: usize,
}

impl<'a> Comments<'a> {
    pub fn new(source_text: SourceText<'a>, comments: &'a [Comment]) -> Self {
        Comments { source_text, comments, printed_count: 0, handled_type_cast_comment: 0 }
    }

    /// Returns comments that have not been printed yet.
    #[inline]
    pub fn unprinted_comments(&self) -> &'a [Comment] {
        &self.comments[self.printed_count..]
    }

    /// Returns comments that have already been printed.
    #[inline]
    pub fn printed_comments(&self) -> &'a [Comment] {
        &self.comments[..self.printed_count]
    }

    /// Returns an iterator over comments that end before or at the given position.
    pub fn comments_before_iter(&self, pos: u32) -> impl Iterator<Item = &Comment> {
        self.unprinted_comments().iter().take_while(move |c| c.span.end <= pos)
    }

    /// Returns all comments that end before or at the given position.
    pub fn comments_before(&self, pos: u32) -> &'a [Comment] {
        let index = self.comments_before_iter(pos).count();
        &self.unprinted_comments()[..index]
    }

    /// Returns all block comments that end before or at the given position.
    pub fn block_comments_before(&self, pos: u32) -> &'a [Comment] {
        let index = self.comments_before_iter(pos).take_while(|c| c.is_block()).count();
        &self.unprinted_comments()[..index]
    }

    /// Returns comments that are on their own line and end before or at the given position.
    pub fn own_line_comments_before(&self, pos: u32) -> &'a [Comment] {
        let index = self
            .comments_before_iter(pos)
            .take_while(|c| self.source_text.is_own_line_comment(c))
            .count();
        &self.unprinted_comments()[..index]
    }

    /// Returns comments that start after the given position (excluding printed ones).
    pub fn comments_after(&self, pos: u32) -> &'a [Comment] {
        let comments = self.unprinted_comments();
        let start_index = comments.iter().take_while(|c| c.span.end < pos).count();
        &comments[start_index..]
    }

    /// Returns comments that fall between the given start and end positions.
    pub fn comments_in_range(&self, start: u32, end: u32) -> &'a [Comment] {
        let comments = self.comments_after(start);
        let end_index = comments.iter().take_while(|c| c.span.end < end).count();
        &comments[..end_index]
    }

    /// Returns comments that occur before the first instance of a specific character.
    pub(crate) fn comments_before_character(&self, mut start: u32, character: u8) -> &'a [Comment] {
        let comments = self.comments_after(start);

        for (index, comment) in comments.iter().enumerate() {
            if self.source_text.bytes_contain(start, comment.span.start, character) {
                return &comments[..index];
            }
            start = comment.span.end;
        }

        comments
    }

    /// Checks if there are any comments between the given positions.
    pub fn has_comment_in_range(&self, start: u32, end: u32) -> bool {
        self.comments_before_iter(end).any(|comment| comment.span.end >= start)
    }

    /// Checks if there are any comments within the given span.
    #[inline]
    pub fn has_comment_in_span(&self, span: Span) -> bool {
        self.has_comment_in_range(span.start, span.end)
    }

    /// Checks if there are any comments before the given position.
    #[inline]
    pub fn has_comment_before(&self, start: u32) -> bool {
        self.comments_before_iter(start).next().is_some()
    }

    /// Checks if there are any leading own-line comments before the given position.
    pub fn has_leading_own_line_comment(&self, start: u32) -> bool {
        self.comments_before_iter(start).any(|comment| {
            self.source_text.is_own_line_comment(comment)
                || self.source_text.lines_after(comment.span.end) > 0
        })
    }

    /// Checks if there are leading or trailing comments around `current_span`.
    /// Leading comments are between `previous_end` and `current_span.start`.
    /// Trailing comments are between `current_span.end` and `following_start`.
    #[inline]
    pub fn has_comment(&self, previous_end: u32, current_span: Span, following_start: u32) -> bool {
        self.has_comment_in_range(previous_end, current_span.start)
            || self.has_comment_in_range(current_span.end, following_start)
    }

    /// **Critical method**: Advances the printed cursor by one.
    ///
    /// This MUST be called after formatting each comment to maintain system integrity.
    /// Failure to call this method will result in:
    /// - Comments being processed multiple times
    /// - Incorrect comment categorization in subsequent queries
    /// - Malformed formatter output
    ///
    /// This is automatically called by the trivia formatting functions, but must be
    /// called manually if comments are formatted through other means.
    #[inline]
    pub fn increment_printed_count(&mut self) {
        self.printed_count += 1;
    }

    /// **Critical method**: Advances the printed cursor by the specified amount.
    ///
    /// Used when multiple comments are processed in batch. Each unit of `count`
    /// represents one comment that has been formatted and should be marked as processed.
    ///
    /// Like [`increment_printed_count`], this is essential for maintaining the
    /// integrity of the comment tracking system.
    #[inline]
    pub fn increase_printed_count_by(&mut self, count: usize) {
        self.printed_count += count;
    }

    /// Gets trailing comments for a node based on its context.
    /// Returns comments that should be printed as trailing comments for `preceding_node`.
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

        // If preceding_node is a callee, let the following node handle its comments
        // Based on Prettier's comment handling logic
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

            if source_text.is_own_line_comment(comment) {
                // Own line comments are typically leading comments for the next node

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
            } else if self.source_text.is_end_of_line_comment(comment) {
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

                // End-of-line comments in specific contexts should be leading comments
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

        // Find the first comment (from the end) that has non-whitespace/non-paren content after it
        let mut gap_end = following_span.start;
        for (idx, comment) in comments[..comment_index].iter().enumerate().rev() {
            if source_text.all_bytes_match(comment.span.end, gap_end, |b| {
                b.is_ascii_whitespace() || b == b'('
            }) {
                gap_end = comment.span.start;
            } else {
                // If there is a non-whitespace character, we stop here
                return &comments[..=idx];
            }
        }

        &[]
    }

    /// Checks if the node has a suppression comment (prettier-ignore).
    pub fn is_suppressed(&self, start: u32) -> bool {
        self.comments_before(start).iter().any(|comment| {
            // TODO: Consider using `oxc-formatter-ignore` instead of `prettier-ignore`
            self.source_text.text_for(&comment.content_span()).trim() == "prettier-ignore"
        })
    }

    /// Checks if a comment is a type cast comment containing `@type` or `@satisfies`.
    pub fn is_type_cast_comment(&self, comment: &Comment) -> bool {
        const TYPE_PATTERN: &[u8] = b"@type";
        const SATISFIES_PATTERN: &[u8] = b"@satisfies";

        if !matches!(comment.content, CommentContent::Jsdoc) {
            return false;
        }

        let bytes = self.source_text.text_for(&comment.span).as_bytes();
        for (i, &byte) in bytes.iter().enumerate() {
            if byte == b'@'
                && (matches_pattern_at(bytes, i, TYPE_PATTERN)
                    || matches_pattern_at(bytes, i, SATISFIES_PATTERN))
            {
                return true;
            }
        }
        false
    }

    /// Finds the index of a type cast comment before the given span.
    ///
    /// Searches for a JSDoc comment containing @type or @satisfies that is followed
    /// by an opening parenthesis, which indicates a type cast pattern.
    pub fn get_type_cast_comment_index(&self, span: Span) -> Option<usize> {
        self.unprinted_comments().iter().take_while(|c| c.span.end <= span.start).position(
            |comment| {
                self.source_text.next_non_whitespace_byte_is(comment.span.end, b'(')
                    && self.is_type_cast_comment(comment)
            },
        )
    }

    /// Marks the most recently printed type cast comment as handled.
    pub fn mark_as_handled_type_cast_comment(&mut self) {
        self.handled_type_cast_comment = self.printed_count;
    }

    /// Checks if the most recently printed type cast comment has been handled.
    pub fn is_already_handled_type_cast_comment(&self) -> bool {
        self.printed_count == self.handled_type_cast_comment
    }
}

/// Checks if a pattern matches at the given position.
fn matches_pattern_at(bytes: &[u8], pos: usize, pattern: &[u8]) -> bool {
    bytes[pos..].starts_with(pattern)
        && matches!(bytes.get(pos + pattern.len()), Some(b' ' | b'\t' | b'\n' | b'\r' | b'{'))
}

/// Handles comment placement logic for if and while statements.
fn handle_if_and_while_statement_comments<'a>(
    mut end: u32,
    comment_index: usize,
    comments: &'a [Comment],
    source_text: SourceText,
) -> &'a [Comment] {
    // Handle pattern: `if (a /* comment */) // trailing comment`
    // Find the last comment that contains ')' between its end and the current end
    for (idx, comment) in comments[..=comment_index].iter().enumerate().rev() {
        if source_text.bytes_contain(comment.span.end, end, b')') {
            return &comments[..=idx];
        }
        end = comment.span.start;
    }

    &[]
}
