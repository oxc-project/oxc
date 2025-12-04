//! High-level comment formatting interface for the on-demand comment system.
//!
//! This module provides the formatting implementations that work with the cursor-based
//! comment tracking system in [`crate::formatter::comments`]. It handles the actual
//! rendering of comments with proper spacing, line breaks, and indentation.
//!
//! ## Integration with Comment Architecture
//!
//! This module is the "formatting layer" of our on-demand comment system:
//!
//! 1. **AST formatting code calls** the format functions in this module
//! 2. **These functions query** the comment system for relevant comments
//! 3. **Comments are formatted** with appropriate spacing and breaks
//! 4. **The cursor is advanced** to mark comments as processed
//!
//! ## Comment Formatting Implementation
//!
//! ### Leading Comment Formatting ([`FormatLeadingComments`])
//! ```rust,ignore
//! // In AST node formatting:
//! write!(f, [format_leading_comments(node.span)]);
//! write!(f, [node]);
//! ```
//!
//! **Implementation**:
//! 1. Calls `comments_before(node.span.start)` to get unprinted leading comments
//! 2. Formats each comment with spacing based on line breaks in original source
//! 3. Advances cursor by calling `increment_printed_count()` for each comment
//! 4. Handles special cases like JSDoc comment "nestling"
//!
//! ### Trailing Comment Formatting ([`FormatTrailingComments`])
//! ```rust,ignore
//! // In AST node formatting:
//! write!(f, [node]);
//! write!(f, [format_trailing_comments(enclosing, preceding, following)]);
//! ```
//!
//! **Implementation**:
//! 1. Calls `get_trailing_comments()` with node context to determine ownership
//! 2. Uses line suffixes to prevent comments from interfering with code layout
//! 3. Handles complex spacing rules for different comment types
//! 4. Advances cursor after processing each comment
//!
//! ### Dangling Comment Formatting ([`FormatDanglingComments`])
//! ```rust,ignore
//! // In container node formatting:
//! write!(f, [
//!     "{",
//!     format_dangling_comments(container.span).with_block_indent(),
//!     "}"
//! ]);
//! ```
//!
//! **Implementation**:
//! 1. Calls `comments_between()` to find internal comments not owned by children
//! 2. Applies indentation based on container type (block, soft, none)
//! 3. Preserves comment relationships and spacing
//! 4. Advances cursor for processed comments
use oxc_ast::{Comment, CommentContent, CommentKind};
use oxc_span::Span;

use crate::write;

use super::prelude::*;

/// Returns true if:
/// - `next_comment` is Some, and
/// - both comments are documentation comments, and
/// - both comments are multiline, and
/// - the two comments are immediately adjacent to each other, with no characters between them.
///
/// In this case, the comments are considered "nestled" - a pattern that JSDoc uses to represent
/// overloaded types, which get merged together to create the final type for the subject. The
/// comments must be kept immediately adjacent after formatting to preserve this behavior.
///
/// There isn't much documentation about this behavior, but it is mentioned on the JSDoc repo
/// for documentation: <https://github.com/jsdoc/jsdoc.github.io/issues/40>. Prettier also
/// implements the same behavior: <https://github.com/prettier/prettier/pull/13445/files#diff-3d5eaa2a1593372823589e6e55e7ca905f7c64203ecada0aa4b3b0cdddd5c3ddR160-R178>
fn should_nestle_adjacent_doc_comments(current: &Comment, next: &Comment) -> bool {
    matches!(current.content, CommentContent::Jsdoc)
        && matches!(next.content, CommentContent::Jsdoc)
        && current.is_multiline_block()
        && next.is_multiline_block()
        && current.span.end == next.span.start
}

/// Formats the leading comments of `node`
#[inline]
pub const fn format_leading_comments<'a>(span: Span) -> FormatLeadingComments<'a> {
    FormatLeadingComments::Node(span)
}

/// Formats the leading comments of a node.
#[derive(Debug, Copy, Clone)]
pub enum FormatLeadingComments<'a> {
    Node(Span),
    Comments(&'a [Comment]),
}

impl<'a> Format<'a> for FormatLeadingComments<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        fn format_leading_comments_impl<'a>(
            comments: impl IntoIterator<Item = &'a Comment>,
            f: &mut Formatter<'_, 'a>,
        ) {
            let mut leading_comments_iter = comments.into_iter().peekable();
            while let Some(comment) = leading_comments_iter.next() {
                f.context_mut().comments_mut().increment_printed_count();
                write!(f, comment);

                match comment.kind {
                    CommentKind::SinglelineBlock | CommentKind::MultilineBlock => {
                        match f.source_text().lines_after(comment.span.end) {
                            0 => {
                                let should_nestle =
                                    leading_comments_iter.peek().is_some_and(|next_comment| {
                                        should_nestle_adjacent_doc_comments(comment, next_comment)
                                    });

                                write!(f, [maybe_space(!should_nestle)]);
                            }
                            1 => {
                                if f.source_text().get_lines_before(comment.span, f.comments()) == 0
                                {
                                    write!(f, [soft_line_break_or_space()]);
                                } else {
                                    write!(f, [hard_line_break()]);
                                }
                            }
                            _ => write!(f, [empty_line()]),
                        }
                    }
                    CommentKind::Line => match f.source_text().lines_after(comment.span.end) {
                        0 | 1 => write!(f, [hard_line_break()]),
                        _ => write!(f, [empty_line()]),
                    },
                }
            }
        }

        match self {
            Self::Node(span) => {
                let leading_comments = f.context().comments().comments_before(span.start);
                if leading_comments.is_empty() {
                    return;
                }
                format_leading_comments_impl(leading_comments, f);
            }
            Self::Comments(comments) => {
                if comments.is_empty() {
                    return;
                }
                format_leading_comments_impl(*comments, f);
            }
        }
    }
}

/// Formats the trailing comments of `node`.
#[inline]
pub const fn format_trailing_comments<'a>(
    enclosing_span: Span,
    preceding_span: Span,
    following_span: Option<Span>,
) -> FormatTrailingComments<'a> {
    FormatTrailingComments::Node((enclosing_span, preceding_span, following_span))
}

/// Formats the trailing comments of `node`
#[derive(Debug, Clone, Copy)]
pub enum FormatTrailingComments<'a> {
    // (enclosing_span, preceding_span, following_span)
    Node((Span, Span, Option<Span>)),
    Comments(&'a [Comment]),
}

impl<'a> Format<'a> for FormatTrailingComments<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        fn format_trailing_comments_impl<'a>(
            comments: impl IntoIterator<Item = &'a Comment>,
            f: &mut Formatter<'_, 'a>,
        ) {
            let mut total_lines_before = 0;
            let mut previous_comment: Option<&Comment> = None;

            for comment in comments {
                f.context_mut().comments_mut().increment_printed_count();

                let lines_before = f.source_text().get_lines_before(comment.span, f.comments());
                total_lines_before += lines_before;

                let should_nestle = previous_comment.is_some_and(|previous_comment| {
                    should_nestle_adjacent_doc_comments(previous_comment, comment)
                });

                // This allows comments at the end of nested structures:
                // {
                //   x: 1,
                //   y: 2
                //   // A comment
                // }
                // Those kinds of comments are almost always leading comments, but
                // here it doesn't go "outside" the block and turns it into a
                // trailing comment for `2`. We can simulate the above by checking
                // if this a comment on its own line; normal trailing comments are
                // always at the end of another expression.
                if total_lines_before > 0
                    || previous_comment.is_some_and(|comment| comment.is_line())
                {
                    write!(
                        f,
                        [line_suffix(&format_with(|f| {
                            match lines_before {
                                _ if should_nestle => {}
                                0 => {
                                    // If the comment is immediately following a block-like comment,
                                    // then it can stay on the same line with just a space between.
                                    // Otherwise, it gets a hard break.
                                    //
                                    //   [>* hello <] // hi
                                    //   [>*
                                    //    * docs
                                    //   */ [> still on the same line <]
                                    if previous_comment.copied().is_some_and(Comment::is_line) {
                                        write!(f, [hard_line_break()]);
                                    } else {
                                        write!(f, [space()]);
                                    }
                                }
                                1 => write!(f, [hard_line_break()]),
                                _ => write!(f, [empty_line()]),
                            }

                            write!(f, [comment]);
                        }))]
                    );
                } else {
                    let content =
                        format_with(|f| write!(f, [maybe_space(!should_nestle), comment]));

                    if comment.is_line() {
                        write!(f, [line_suffix(&content), expand_parent()]);
                    } else {
                        write!(f, [content]);
                    }
                }

                previous_comment = Some(comment);
            }
        }

        match self {
            Self::Node((enclosing_span, preceding_span, following_span)) => {
                let comments = f.context().comments().get_trailing_comments(
                    *enclosing_span,
                    *preceding_span,
                    *following_span,
                );

                if comments.is_empty() {
                    return;
                }

                format_trailing_comments_impl(comments, f);
            }
            Self::Comments(comments) => {
                if comments.is_empty() {
                    return;
                }

                format_trailing_comments_impl(*comments, f);
            }
        }
    }
}

/// Formats the dangling comments of `node`.
#[inline]
pub const fn format_dangling_comments<'a>(span: Span) -> FormatDanglingComments<'a> {
    FormatDanglingComments::Node { span, indent: DanglingIndentMode::None }
}

/// Formats the dangling trivia of `token`.
pub enum FormatDanglingComments<'a> {
    Node { span: Span, indent: DanglingIndentMode },
    Comments { comments: &'a [Comment], indent: DanglingIndentMode },
}

#[derive(Copy, Clone, Debug)]
pub enum DanglingIndentMode {
    /// Writes every comment on its own line and indents them with a block indent.
    ///
    /// # Examples
    /// ```ignore
    /// [
    ///     /* comment */
    /// ]
    ///
    /// [
    ///     /* comment */
    ///     /* multiple */
    /// ]
    /// ```
    Block,

    /// Writes every comment on its own line and indents them with a soft line indent.
    /// Guarantees to write a line break if the last formatted comment is a [line](CommentKind::Line) comment.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// [/* comment */]
    ///
    /// [
    ///     /* comment */
    ///     /* other */
    /// ]
    ///
    /// [
    ///     // line
    /// ]
    /// ```
    Soft,

    /// Writes every comment on its own line.
    None,
}

impl FormatDanglingComments<'_> {
    /// Indents the comments with a [block](DanglingIndentMode::Block) indent.
    pub fn with_block_indent(self) -> Self {
        self.with_indent_mode(DanglingIndentMode::Block)
    }

    /// Indents the comments with a [soft block](DanglingIndentMode::Soft) indent.
    pub fn with_soft_block_indent(self) -> Self {
        self.with_indent_mode(DanglingIndentMode::Soft)
    }

    fn with_indent_mode(mut self, mode: DanglingIndentMode) -> Self {
        match &mut self {
            FormatDanglingComments::Node { indent, .. }
            | FormatDanglingComments::Comments { indent, .. } => *indent = mode,
        }
        self
    }
}

impl<'a> Format<'a> for FormatDanglingComments<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        fn format_dangling_comments_impl<'a>(
            comments: impl IntoIterator<Item = &'a Comment>,
            indent: DanglingIndentMode,
            f: &mut Formatter<'_, 'a>,
        ) {
            // Write all comments up to the first skipped token trivia or the token
            let format_dangling_comments = format_once(|f| {
                let mut previous_comment: Option<&Comment> = None;

                for comment in comments {
                    f.context_mut().comments_mut().increment_printed_count();

                    let should_nestle = previous_comment.is_some_and(|previous_comment| {
                        should_nestle_adjacent_doc_comments(previous_comment, comment)
                    });

                    write!(
                        f,
                        [
                            (previous_comment.is_some() && !should_nestle)
                                .then_some(hard_line_break()),
                            comment
                        ]
                    );

                    previous_comment = Some(comment);
                }

                if matches!(indent, DanglingIndentMode::Soft)
                    && previous_comment.copied().is_some_and(Comment::is_line)
                {
                    write!(f, [hard_line_break()]);
                }
            });

            match indent {
                DanglingIndentMode::Block => {
                    write!(f, [block_indent(&format_dangling_comments)]);
                }
                DanglingIndentMode::Soft => {
                    write!(f, [group(&soft_block_indent(&format_dangling_comments))]);
                }
                DanglingIndentMode::None => {
                    write!(f, [format_dangling_comments]);
                }
            }
        }

        match self {
            FormatDanglingComments::Node { span, indent } => {
                let dangling_comments = f.context().comments().comments_before(span.end);
                if dangling_comments.is_empty() {
                    return;
                }
                format_dangling_comments_impl(dangling_comments, *indent, f);
            }
            FormatDanglingComments::Comments { comments, indent } => {
                if comments.is_empty() {
                    return;
                }
                format_dangling_comments_impl(*comments, *indent, f);
            }
        }
    }
}

impl<'a> Format<'a> for Comment {
    #[expect(clippy::cast_possible_truncation)]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let source_text = f.source_text().text_for(&self.span).trim_end();
        if is_alignable_comment(self, source_text) {
            let mut source_offset = self.span.start;

            let mut lines = source_text.lines();

            // `is_alignable_comment` only returns `true` for multiline comments
            let first_line = lines.next().unwrap();
            write!(f, [text(first_line.trim_end())]);

            source_offset += first_line.len() as u32;

            // Indent the remaining lines by one space so that all `*` are aligned.
            write!(
                f,
                [&format_once(|f| {
                    for line in lines {
                        write!(f, [hard_line_break(), " ", text(line.trim())]);
                        source_offset += line.len() as u32;
                    }
                })]
            );
        } else {
            write!(f, [text(source_text)]);
        }
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
pub fn is_alignable_comment(comment: &Comment, source_text: &str) -> bool {
    if !comment.is_multiline_block() {
        return false;
    }
    source_text.lines().enumerate().all(|(index, line)| {
        if index == 0 { line.starts_with("/*") } else { line.trim_start().starts_with('*') }
    })
}
