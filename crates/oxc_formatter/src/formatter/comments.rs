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
//! We process comments lazily during formatting, associating them by source position:
//!
//! **Performance Benefits:**
//! - **No hash map overhead**: We directly use the parser's comment array, no additional data structures
//! - **Single AST pass**: No additional traversal needed for comment processing
//! - **Zero memory allocation**: No comment storage on AST nodes, just reference the parser's data
//! - **Lazy evaluation**: Only process comments that actually need formatting
//!
//! **Simpler Association:**
//! - **Source-position logic**: Comments found by source position, not ownership rules
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
//! ### When the Cursor Is the Wrong Oracle
//! The cursor answers "what is left to print", so its answer changes as printing proceeds.
//! A layout decision evaluated more than once for the same node
//! (grouped call arguments re-format the grouped function with its body reused from cache,
//! a `write` asked after the node's leading comments are printed, a re-entered `fmt`)
//! must instead ask the position-based queries (`all_comments_in_range` and friends),
//! which read ALL comments and depend only on the source.
//!
//! ## Lexical Byte Scans
//! Some queries scan the source bytes between comments for a token (`)`, `;`, a keyword's first byte),
//! skipping comment interiors by span (see [`gap_segments`]). They are lexical, not syntactic:
//! the scanned range must hold only trivia and tokens that cannot contain the byte
//! (punctuation, or a keyword scanned for its first byte, e.g. `e` for `else`),
//! such as an expression end up to the statement end.
//! A range covering code would false-match the byte inside a string literal or nested syntax.
//!
//! ## References
//! - [Prettier handles special comments](https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/comments/handle-comments.js)
//! - [Prettier pre-processes comments](https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/main/comments/attach.js)
use oxc_ast::{Comment, CommentContent};
use oxc_formatter_core::SourceText;
use oxc_span::{GetSpan, Span};

/// Saved comment cursor state for [`Comments::snapshot`] / [`Comments::restore`].
#[derive(Clone, Copy)]
pub struct CommentSnapshot {
    printed_count: usize,
    last_handled_type_cast_comment: usize,
    type_cast_node_span: Span,
    view_limit: Option<usize>,
}

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
    view_limit: Option<usize>,
}

// Construction and the printed cursor / view state
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

    /// Returns the span of the first not-yet-printed comment, if any.
    ///
    /// Used by [`SourceText::get_lines_before`], which only needs that comment's
    /// span (not the `Comment` itself) to skip leading trivia.
    #[inline]
    pub fn first_unprinted_span(&self) -> Option<Span> {
        self.unprinted_comments().first().map(|c| c.span)
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

    /// Advances the cursor past all comments ending before `pos`:
    /// comments a verbatim (suppressed) range already contains, or, before speculative formatting,
    /// comments outside the span of interest that would otherwise lead the speculatively formatted node.
    pub fn skip_comments_before(&mut self, pos: u32) {
        self.printed_count += self.comments_before_iter(pos).count();
    }

    /// Temporarily limits the unprinted comments view to only those before the given position.
    /// Returns the previous view limit to allow restoration.
    pub fn limit_comments_up_to(&mut self, end_pos: u32) -> Option<usize> {
        let original_limit = self.view_limit;
        let limit_index = self.printed_count
            + self.inner[self.printed_count..].partition_point(|c| c.span.start < end_pos);
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

    /// Saves the current comment processing state for later restoration.
    ///
    /// Use with [`Comments::restore`] to safely perform speculative formatting
    /// without permanently advancing the comment cursor. This prevents comment
    /// deletion bugs when using `Formatter::intern` for `will_break` checks.
    ///
    /// NOTE: with [`Comments::restore`], only used by `speculate_will_break` for `is_complex_type_arguments` (`assignment_like.rs`).
    pub fn snapshot(&self) -> CommentSnapshot {
        CommentSnapshot {
            printed_count: self.printed_count,
            last_handled_type_cast_comment: self.last_handled_type_cast_comment,
            type_cast_node_span: self.type_cast_node_span,
            view_limit: self.view_limit,
        }
    }

    /// Restores comment processing state from a previous snapshot.
    ///
    /// This rolls back the comment cursor so that any comments consumed
    /// during speculative formatting are available again for real formatting.
    pub fn restore(&mut self, snapshot: CommentSnapshot) {
        self.printed_count = snapshot.printed_count;
        self.last_handled_type_cast_comment = snapshot.last_handled_type_cast_comment;
        self.type_cast_node_span = snapshot.type_cast_node_span;
        self.view_limit = snapshot.view_limit;
    }
}

// Cursor-based queries: the unprinted view, for PRINTING.
// "Unprinted and before `pos`" means "this node's own leading comments" because printing follows source order.
impl<'a> Comments<'a> {
    /// Returns an iterator over comments that end before or at the given position.
    pub fn comments_before_iter(&self, pos: u32) -> impl Iterator<Item = &Comment> {
        self.unprinted_comments().iter().take_while(move |c| c.span.end <= pos)
    }

    /// Returns comments that end before or at the given position.
    pub fn comments_before(&self, pos: u32) -> &'a [Comment] {
        &self.unprinted_comments()[..self.comments_before_iter(pos).count()]
    }

    /// Returns the line comments that end before or at the given position.
    pub fn line_comments_before(&self, pos: u32) -> &'a [Comment] {
        self.comments_before_while(pos, |c| c.is_line())
    }

    /// Returns comments that are on their own line and end before or at the given position.
    pub fn own_line_comments_before(&self, pos: u32) -> &'a [Comment] {
        self.comments_before_while(pos, |c| c.preceded_by_newline())
    }

    /// The leading run of [`Self::comments_before_iter`] satisfying `predicate`, as a slice.
    fn comments_before_while(
        &self,
        pos: u32,
        predicate: impl Fn(&Comment) -> bool,
    ) -> &'a [Comment] {
        let index = self.comments_before_iter(pos).take_while(|c| predicate(c)).count();
        &self.unprinted_comments()[..index]
    }

    /// Returns comments that end at or after the given position.
    pub fn comments_after(&self, pos: u32) -> &'a [Comment] {
        let comments = self.unprinted_comments();
        &comments[comments.partition_point(|c| c.span.end < pos)..]
    }

    /// Returns comments between the given positions.
    /// Unlike [`Self::all_comments_in_range`], a comment ending exactly at `start` is included.
    pub fn comments_in_range(&self, start: u32, end: u32) -> &'a [Comment] {
        let comments = self.comments_after(start);
        let end_index = comments.iter().take_while(|c| c.span.end <= end).count();
        &comments[..end_index]
    }

    /// Returns end-of-line comments that are after the given position.
    pub fn end_of_line_comments_after(&self, mut pos: u32) -> &'a [Comment] {
        let comments = self.comments_after(pos);
        for (index, comment) in comments.iter().enumerate() {
            if self.source_text.all_bytes_match(pos, comment.span.start, |b| {
                matches!(b, b'\t' | b' ' | b'=' | b':' | b',')
            }) {
                if comment.is_line() || comment.followed_by_newline() {
                    return &comments[..=index];
                }
                pos = comment.span.end;
            } else {
                break;
            }
        }
        &[]
    }

    /// Returns comments that occur before the first instance of a specific character (a lexical byte scan, see the module doc).
    /// Not on [`gap_segments`]: the result is a comment count, and the gap after the last comment is not scanned.
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

    /// Comments sitting between `pos` and a closing source paren (a lexical byte scan, see the module doc).
    /// `None` when there is no such comment or another token intervenes.
    pub(crate) fn comments_before_closing_paren(&self, pos: u32) -> Option<&'a [Comment]> {
        self.comment_run_before_closing_paren(self.comments_after(pos), pos)
    }

    /// Gets trailing comments for a node based on its context.
    /// Returns comments that should be printed as trailing comments for `preceding_node`.
    ///
    /// `following_span_start` is the start position of the following sibling node, or 0 if none.
    pub fn get_trailing_comments(
        &self,
        enclosing_span: Span,
        preceding_span: Span,
        following_span_start: u32,
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

        if following_span_start == 0 {
            // Find dangling comments at the end of the enclosing node
            let comments = self.comments_before(enclosing_span.end);

            // When the enclosing statement ends at the very position the preceding node does
            // (a single-statement body sharing its distant `;`),
            // ```js
            // if (1) foo
            // // c
            // ;
            // ```
            // every comment here sits inside the preceding node,
            // and own-line comments the preceding statement deferred must escape the enclosing statement
            // to stay own-line for a later pass (the next statement's leading comments),
            // like Prettier whose statement `locEnd` excludes the trailing `;`.
            // When the enclosing node continues past the preceding one (e.g. a block's last statement),
            // the loop below keeps them inside as trailing comments instead.
            if enclosing_span.end == preceding_span.end {
                for (idx, comment) in comments.iter().enumerate() {
                    if !comment.preceded_by_newline() {
                        continue;
                    }
                    // Everything from here to the terminator must be trivia:
                    // a run of own-line comments separated by whitespace, then the `;`.
                    let mut pos = comment.span.end;
                    let mut deferrable = true;
                    for next in &comments[idx + 1..] {
                        if !next.preceded_by_newline()
                            || !source_text
                                .all_bytes_match(pos, next.span.start, |b| b.is_ascii_whitespace())
                        {
                            deferrable = false;
                            break;
                        }
                        pos = next.span.end;
                    }
                    if deferrable
                        && source_text.all_bytes_match(pos, preceding_span.end, |b| {
                            b.is_ascii_whitespace() || b == b';'
                        })
                        && source_text.bytes_contain(pos, preceding_span.end, b';')
                    {
                        return &comments[..idx];
                    }
                }
                return comments;
            }

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
        }

        let mut comment_index = 0;
        let mut type_cast_comment = None;

        while let Some(comment) = comments.get(comment_index) {
            // Stop if the comment:
            // 1. is over the following node
            // 2. is after the enclosing node, which means the comment should be printed in the parent node.
            if comment.span.end > following_span_start || comment.span.end > enclosing_span.end {
                break;
            }

            if following_span_start > enclosing_span.end && comment.span.end <= enclosing_span.end {
                // Do nothing; this comment is inside the enclosing node, and the following node is outside the enclosing node.
                // So it must be a trailing comment, continue checking the next comment.
            } else if self.is_type_cast_comment(comment) {
                // `A || /* @type {Number} */ (B)`:
                //      ^^^^^^^^^^^^^^^^^^^^^^^^
                // Type cast comments should always be treated as leading comment to the following node
                type_cast_comment = Some(comment);
                break;
            } else if comment.preceded_by_newline() {
                // Own-line comments should be treated as leading comments to the following node
                break;
            } else if comment.followed_by_newline() {
                // End-of-line comments are always trailing comments to the preceding node.
                return &comments[..=comment_index];
            }

            comment_index += 1;
        }

        // Find the first comment (from the end) that has non-whitespace/non-paren content after it
        let mut gap_end = type_cast_comment.map_or(following_span_start, |c| c.span.start);

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

    /// End of the content including its source parentheses:
    /// the position after the last `)` between `content_end` and `node_end` (`)` bytes inside comments don't count),
    /// or `content_end` itself when the content is not parenthesized.
    ///
    /// Comments before this position sit inside the parentheses and belong to the content;
    /// only comments after it may move behind the semicolon (a lexical byte scan, see the module doc).
    pub(crate) fn end_including_source_parens(&self, content_end: u32, node_end: u32) -> u32 {
        #[expect(clippy::cast_possible_truncation)] // Offsets fit in `u32`, source length does
        let position_after_last_close_paren = |from: u32, to: u32| {
            let offset = self.source_text.bytes_range(from, to).iter().rposition(|&b| b == b')')?;
            Some(from + offset as u32 + 1)
        };

        gap_segments(self.comments_in_range(content_end, node_end), content_end, node_end)
            .filter_map(|(from, to)| position_after_last_close_paren(from, to))
            .last()
            .unwrap_or(content_end)
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
        self.comments_before_iter(start).any(|comment| comment.followed_by_newline())
    }

    /// Index into [`Self::unprinted_comments`] of the first cast comment
    /// ([`Self::is_type_cast_comment_followed_by_paren`]) before the given span.
    /// Cursor-based on purpose: printing peels nested casts one per pass (see `utils/typecast.rs`).
    pub fn get_type_cast_comment_index(&self, span: Span) -> Option<usize> {
        self.comments_before_iter(span.start)
            .position(|comment| self.is_type_cast_comment_followed_by_paren(comment))
    }

    pub fn has_end_of_line_comment_after(&self, pos: u32) -> bool {
        !self.end_of_line_comments_after(pos).is_empty()
    }

    /// Checks if the node has a suppression comment.
    pub fn is_suppressed(&self, start: u32) -> bool {
        self.comments_before_iter(start).any(|comment| self.is_suppression_comment(comment))
    }

    /// Checks if there is a trailing suppression comment on the same line.
    ///
    /// This supports patterns like:
    /// `statement(); // prettier-ignore`
    /// `statement(); /* prettier-ignore */`
    /// `value, // prettier-ignore`
    pub fn has_trailing_suppression_comment(&self, pos: u32) -> bool {
        self.end_of_line_comments_after(pos)
            .iter()
            .any(|comment| self.is_suppression_comment(comment))
    }

    /// Whether a node whose terminator the formatter owns is suppressed by a leading comment or a trailing one:
    /// on its line after the `;` (`foo(); // prettier-ignore`),
    /// or after the content when the source `;` sits on a later line (`foo() // prettier-ignore` + `;[].sort()`, the `semi: false` style).
    /// `content_end` is asked only for that last shape (the span's last comment is a suppression comment).
    pub fn is_node_suppressed(
        &self,
        span: Span,
        content_end: impl FnOnce() -> Option<u32>,
    ) -> bool {
        // The common case, every statement pays this check
        if self.unprinted_comments().is_empty() {
            return false;
        }
        if self.is_suppressed(span.start) || self.has_trailing_suppression_comment(span.end) {
            return true;
        }
        let Some(last) = self.all_comments_before(span.end).last() else { return false };
        last.span.start >= span.start
            && self.is_suppression_comment(last)
            && content_end().is_some_and(|end| self.has_trailing_suppression_comment(end))
    }

    /// Whether the range holds a `;` or a `)` outside comments (`foo /* ; */` doesn't count).
    /// Why a `)` counts as a statement terminator: see `trailing_comments_to_move_behind_semicolon`
    /// (a lexical byte scan, see the module doc).
    pub(crate) fn has_semicolon_or_closing_paren_in_range(&self, start: u32, end: u32) -> bool {
        gap_segments(self.comments_in_range(start, end), start, end).any(|(from, to)| {
            self.source_text.bytes_range(from, to).iter().any(|&b| matches!(b, b';' | b')'))
        })
    }
}

// Position-based queries: ALL comments (printed and hidden ones included), for LAYOUT DECISIONS.
impl<'a> Comments<'a> {
    /// Position-based variant of [`Self::comments_in_range`] (contained in `[start, end]`).
    pub fn all_comments_in_range(&self, start: u32, end: u32) -> impl Iterator<Item = &'a Comment> {
        let first = self.inner.partition_point(|comment| comment.span.start < start);
        self.inner[first..].iter().take_while(move |comment| comment.span.end <= end)
    }

    /// Position-based variant of [`Self::comments_before`].
    pub fn all_comments_before(&self, pos: u32) -> &'a [Comment] {
        &self.inner[..self.inner.partition_point(|comment| comment.span.end <= pos)]
    }

    /// Position-based variant of [`Self::comments_after`].
    fn all_comments_after(&self, pos: u32) -> &'a [Comment] {
        &self.inner[self.inner.partition_point(|comment| comment.span.end <= pos)..]
    }

    /// Position-based variant of [`Self::has_comment_in_range`].
    pub fn has_any_comment_in_range(&self, start: u32, end: u32) -> bool {
        self.all_comments_in_range(start, end).next().is_some()
    }

    /// Position-based analog of [`Self::has_leading_own_line_comment`], over a range.
    pub fn has_own_line_comment_in_range(&self, start: u32, end: u32) -> bool {
        self.all_comments_in_range(start, end).any(|comment| comment.followed_by_newline())
    }

    /// Whether the first non-whitespace byte after `pos` outside comments is `)`.
    pub(crate) fn is_followed_by_closing_paren(&self, pos: u32) -> bool {
        // Only a comment can hide the `)`, so anything but `/` answers without touching the comments;
        // a `/` that is an operator instead is settled by the comment spans below (no adjacent comment: `false`)
        match self.source_text.next_non_whitespace_byte(pos) {
            Some(b')') => return true,
            Some(b'/') => {}
            _ => return false,
        }
        self.comment_run_before_closing_paren(self.all_comments_after(pos), pos).is_some()
    }

    /// Position just past the first instance of `character` at or after `start`
    /// (a lexical byte scan, see the module doc);
    /// `character` must occur in the scanned range (guaranteed by grammar at the call sites).
    pub(crate) fn position_after_character(&self, start: u32, character: u8) -> u32 {
        let source_end = u32::try_from(self.source_text.as_str().len()).unwrap();
        for (from, to) in gap_segments(self.all_comments_after(start), start, source_end) {
            if let Some(offset) =
                self.source_text.bytes_range(from, to).iter().position(|&byte| byte == character)
            {
                return from + u32::try_from(offset).unwrap() + 1;
            }
        }
        unreachable!("the caller guarantees the character occurs outside a comment")
    }
}

// Classification of a single comment (no cursor involved)
impl Comments<'_> {
    /// Checks if a comment is a suppression comment (`oxfmt-ignore`).
    ///
    /// `prettier-ignore` is also supported for compatibility.
    pub fn is_suppression_comment(&self, comment: &Comment) -> bool {
        oxc_formatter_core::spec::is_suppression_marker(
            self.source_text.text_for(&comment.content_span()),
        )
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

    /// Checks if a comment forms the type cast pattern:
    /// a type cast comment immediately followed by an opening parenthesis.
    pub fn is_type_cast_comment_followed_by_paren(&self, comment: &Comment) -> bool {
        self.source_text.next_non_whitespace_byte_is(comment.span.end, b'(')
            && self.is_type_cast_comment(comment)
    }
}

// Type cast printing state (cursor-based), see `utils/typecast.rs`
impl Comments<'_> {
    /// Marks the given span as a type cast node.
    pub fn mark_as_type_cast_node(&mut self, node: &impl GetSpan) {
        self.type_cast_node_span = node.span();
        self.last_handled_type_cast_comment = self.printed_count;
    }

    /// Checks if the most recently printed type cast comment has been handled.
    pub fn is_handled_type_cast_comment(&self) -> bool {
        self.printed_count == self.last_handled_type_cast_comment
    }

    /// Checks if the node is the one currently being formatted as a cast target
    /// (the read side of [`Comments::mark_as_type_cast_node`];
    /// not the source-based classification, see `classify_type_cast`).
    #[inline]
    pub fn is_marked_as_type_cast_node(&self, node: &impl GetSpan) -> bool {
        self.type_cast_node_span == node.span()
    }
}

// View-neutral scans over a comment slice the caller picked (cursor-based or position-based)
impl<'a> Comments<'a> {
    /// The non-empty run of comments after `pos` ([`Self::comment_run_after`])
    /// when the next non-whitespace byte after it is `)`.
    fn comment_run_before_closing_paren(
        &self,
        comments: &'a [Comment],
        pos: u32,
    ) -> Option<&'a [Comment]> {
        let run = self.comment_run_after(comments, pos);
        let end = run.last()?.span.end;
        self.source_text.next_non_whitespace_byte_is(end, b')').then_some(run)
    }

    /// The run of comments after `pos` separated only by whitespace (a prefix of `comments`).
    fn comment_run_after(&self, comments: &'a [Comment], pos: u32) -> &'a [Comment] {
        let mut cursor = pos;
        let mut count = 0;
        for comment in comments {
            if comment.span.start < cursor
                || !self
                    .source_text
                    .all_bytes_match(cursor, comment.span.start, |b| b.is_ascii_whitespace())
            {
                break;
            }
            count += 1;
            cursor = comment.span.end;
        }
        &comments[..count]
    }
}

/// Byte segments between `start` and `bound` lying outside the given comment spans:
/// one `(gap_start, gap_end)` pair per gap, ending with the tail segment up to `bound`.
/// The caller picks the view (cursor-based or position-based).
pub fn gap_segments<'c>(
    comments: impl IntoIterator<Item = &'c Comment> + 'c,
    start: u32,
    bound: u32,
) -> impl Iterator<Item = (u32, u32)> + 'c {
    let mut pos = start;
    comments
        .into_iter()
        .map(|comment| comment.span)
        .chain(std::iter::once(Span::empty(bound)))
        .filter_map(move |span| {
            // A comment ending exactly at `pos` lies before the range
            if span.start < pos {
                return None;
            }
            let segment = (pos, span.start);
            pos = span.end;
            Some(segment)
        })
}
