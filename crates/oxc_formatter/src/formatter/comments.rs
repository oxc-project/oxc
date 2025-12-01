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
use oxc_ast::{Comment, CommentContent};
use oxc_span::{GetSpan, Span};

use crate::formatter::SourceText;

#[derive(Debug, Clone)]
pub struct Comments<'a> {
    source_text: SourceText<'a>,
    inner: &'a [Comment],
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
    last_handled_type_cast_comment: usize,
    type_cast_node_span: Span,
    /// Optional limit for the unprinted_comments view.
    ///
    /// When set, [`Self::unprinted_comments()`] will only return comments up to this index,
    /// effectively hiding comments beyond this point from the formatter.
    pub view_limit: Option<usize>,
}

impl<'a> Comments<'a> {
    pub fn new(source_text: SourceText<'a>, comments: &'a [Comment]) -> Self {
        Comments {
            source_text,
            inner: comments,
            printed_count: 0,
            last_handled_type_cast_comment: 0,
            type_cast_node_span: Span::default(),
            view_limit: None,
        }
    }

    /// Returns comments that have not been printed yet.
    #[inline]
    pub fn unprinted_comments(&self) -> &'a [Comment] {
        let end = self.view_limit.unwrap_or(self.inner.len());
        &self.inner[self.printed_count..end]
    }

    /// Returns comments that have already been printed.
    #[inline]
    pub fn printed_comments(&self) -> &'a [Comment] {
        &self.inner[..self.printed_count]
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

    /// Returns all block comments that end before or at the given position.
    pub fn line_comments_before(&self, pos: u32) -> &'a [Comment] {
        let index = self.comments_before_iter(pos).take_while(|c| c.is_line()).count();
        &self.unprinted_comments()[..index]
    }

    /// Returns comments that are on their own line and end before or at the given position.
    pub fn own_line_comments_before(&self, pos: u32) -> &'a [Comment] {
        let index =
            self.comments_before_iter(pos).take_while(|c| self.is_own_line_comment(c)).count();
        &self.unprinted_comments()[..index]
    }

    /// Returns end-of-line comments that are after the given position (excluding printed ones).
    pub fn end_of_line_comments_after(&self, mut pos: u32) -> &'a [Comment] {
        let comments = self.unprinted_comments();
        for (index, comment) in comments.iter().enumerate() {
            if self.source_text.all_bytes_match(pos, comment.span.start, |b| {
                matches!(b, b'\t' | b' ' | b'=' | b':')
            }) {
                if comment.is_line() || self.is_end_of_line_comment(comment) {
                    return &comments[..=index];
                }
                pos = comment.span.end;
            } else {
                break;
            }
        }
        &[]
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
        let end_index = comments.iter().take_while(|c| c.span.end <= end).count();
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
        self.comments_before_iter(end).any(|comment| comment.span.end > start)
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
        self.comments_before_iter(start)
            .any(|comment| self.source_text.lines_after(comment.span.end) > 0)
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
    /// Like [`Comments::increment_printed_count`], this is essential for maintaining the
    /// integrity of the comment tracking system.
    #[inline]
    pub fn increase_printed_count_by(&mut self, count: usize) {
        self.printed_count += count;
    }

    /// Gets trailing comments for a node based on its context.
    /// Returns comments that should be printed as trailing comments for `preceding_node`.
    pub fn get_trailing_comments(
        &self,
        enclosing_span: Span,
        preceding_span: Span,
        following_span: Option<Span>,
    ) -> &'a [Comment] {
        let comments = self.unprinted_comments();
        if comments.is_empty() {
            return &[];
        }

        let source_text = self.source_text;

        // All of the comments before this node are printed already.
        debug_assert!(
            comments.first().is_none_or(|comment| comment.span.end > preceding_span.start)
        );

        let Some(following_span) = following_span else {
            // Find dangling comments at the end of the enclosing node
            let comments = self.comments_before(enclosing_span.end);
            let mut start = preceding_span.end;
            for (idx, comment) in comments.iter().enumerate() {
                // Comments inside the preceding node, which should be printed without checking
                if start > comment.span.start {
                    continue;
                }

                if !source_text.all_bytes_match(start, comment.span.start, |b| {
                    b.is_ascii_whitespace() || matches!(b, b')' | b',' | b';')
                }) {
                    return &comments[..idx];
                }

                start = comment.span.end;
            }

            return comments;
        };

        let mut comment_index = 0;
        let mut type_cast_comment = None;

        while let Some(comment) = comments.get(comment_index) {
            // Stop if the comment:
            // 1. is over the following node
            // 2. is after the enclosing node, which means the comment should be printed in the parent node.
            if comment.span.end > following_span.start || comment.span.end > enclosing_span.end {
                break;
            }

            if following_span.start > enclosing_span.end && comment.span.end <= enclosing_span.end {
                // Do nothing; this comment is inside the enclosing node, and the following node is outside the enclosing node.
                // So it must be a trailing comment, continue checking the next comment.
            } else if self.is_type_cast_comment(comment) {
                // `A || /* @type {Number} */ (B)`:
                //      ^^^^^^^^^^^^^^^^^^^^^^^^
                // Type cast comments should always be treated as leading comment to the following node
                type_cast_comment = Some(comment);
                break;
            } else if self.is_own_line_comment(comment) {
                // Own-line comments should be treated as leading comments to the following node
                break;
            } else if self.is_end_of_line_comment(comment) {
                //End-of-line comments are always trailing comments to the preceding node.
                return &comments[..=comment_index];
            }

            comment_index += 1;
        }

        // Find the first comment (from the end) that has non-whitespace/non-paren content after it
        let mut gap_end = type_cast_comment.map_or(following_span.start, |c| c.span.start);

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
        self.comments_before(start).iter().any(|comment| self.is_suppression_comment(comment))
    }

    pub fn is_suppression_comment(&self, comment: &Comment) -> bool {
        // TODO: Consider using `oxfmt-ignore` instead of `prettier-ignore`
        self.source_text.text_for(&comment.content_span()).trim() == "prettier-ignore"
    }

    /// Checks if a comment is a type cast comment containing `@type` or `@satisfies`.
    pub fn is_type_cast_comment(&self, comment: &Comment) -> bool {
        const TYPE_PATTERN: &[u8] = b"@type";
        const SATISFIES_PATTERN: &[u8] = b"@satisfies";

        /// Checks if a pattern matches at the given position.
        fn matches_pattern_at(bytes: &[u8], pos: usize, pattern: &[u8]) -> bool {
            bytes[pos..].starts_with(pattern)
                && bytes
                    .get(pos + pattern.len())
                    .is_some_and(|&byte| byte.is_ascii_whitespace() || byte == b'{')
        }

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

    pub fn is_own_line_comment(&self, comment: &Comment) -> bool {
        self.source_text.has_newline_before(comment.span.start)
    }

    pub fn is_end_of_line_comment(&self, comment: &Comment) -> bool {
        self.source_text.has_newline_after(comment.span.end)
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

    /// Marks the given span as a type cast node.
    pub fn mark_as_type_cast_node(&mut self, node: &impl GetSpan) {
        self.type_cast_node_span = node.span();
        self.last_handled_type_cast_comment = self.printed_count;
    }

    /// Checks if the most recently printed type cast comment has been handled.
    pub fn is_handled_type_cast_comment(&self) -> bool {
        self.printed_count == self.last_handled_type_cast_comment
    }

    #[inline]
    pub fn is_type_cast_node(&self, node: &impl GetSpan) -> bool {
        self.type_cast_node_span == node.span()
    }

    /// Temporarily limits the unprinted comments view to only those before the given position.
    /// Returns the previous view limit to allow restoration.
    pub fn limit_comments_up_to(&mut self, end_pos: u32) -> Option<usize> {
        // Save the original limit for restoration
        let original_limit = self.view_limit;

        // Find the index of the first comment that starts at or after end_pos
        // Using binary search would be more efficient for large comment arrays
        let limit_index = self.inner[self.printed_count..]
            .iter()
            .position(|c| c.span.start >= end_pos)
            .map_or(self.inner.len(), |idx| self.printed_count + idx);

        // Only update if we're actually limiting the view
        if limit_index < self.inner.len() {
            self.view_limit = Some(limit_index);
        }

        original_limit
    }

    /// Restores the view limit to a previously saved value.
    /// This is typically used after temporarily limiting the view with `limit_comments_up_to`.
    #[inline]
    pub fn restore_view_limit(&mut self, limit: Option<usize>) {
        self.view_limit = limit;
    }
}
