//! Provides builders for comments and skipped token trivia.

use oxc_ast::{
    Comment, CommentKind,
    ast::{CallExpression, NewExpression},
};
use oxc_span::{GetSpan, Span};
use oxc_syntax::comment_node;

use crate::{
    formatter::comments::{is_alignable_comment, is_end_of_line_comment, is_own_line_comment},
    generated::{ast_nodes::SiblingNode, format},
    write,
};

use super::{Argument, Arguments, GroupId, SyntaxToken, prelude::*};

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
fn should_nestle_adjacent_doc_comments(first_comment: &Comment, second_comment: &Comment) -> bool {
    false
    // let first = first_comment.piece();
    // let second = second_comment.piece();

    // first.has_newline()
    // && second.has_newline()
    // && (second.text_range().start()).sub(first.text_range().end()) == TextSize::from(0)
    // && is_doc_comment(first)
    // && is_doc_comment(second)
}

/// Formats the leading comments of `node`
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
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        fn format_leading_comments_impl<'a>(
            comments: impl IntoIterator<Item = &'a Comment>,
            f: &mut Formatter<'_, 'a>,
        ) -> FormatResult<()> {
            let mut leading_comments_iter = comments.into_iter().peekable();
            while let Some(comment) = leading_comments_iter.next() {
                f.context_mut().increment_printed_count();
                write!(f, comment)?;

                match comment.kind {
                    CommentKind::Block => {
                        match get_lines_after(comment.span.end, f.source_text()) {
                            0 => {
                                let should_nestle =
                                    leading_comments_iter.peek().is_some_and(|next_comment| {
                                        // should_nestle_adjacent_doc_comments(comment, next_comment)
                                        false
                                    });

                                write!(f, [maybe_space(!should_nestle)])?;
                            }
                            1 => {
                                if get_lines_before(comment.span, f) == 0 {
                                    write!(f, [soft_line_break_or_space()])?;
                                } else {
                                    write!(f, [hard_line_break()])?;
                                }
                            }
                            _ => write!(f, [empty_line()])?,
                        }
                    }
                    CommentKind::Line => match get_lines_after(comment.span.end, f.source_text()) {
                        0 | 1 => write!(f, [hard_line_break()])?,
                        _ => write!(f, [empty_line()])?,
                    },
                }
            }

            Ok(())
        }

        match self {
            Self::Node(span) => {
                let leading_comments = f.context().comments().comments_before(span.start);
                format_leading_comments_impl(leading_comments, f)
            }
            Self::Comments(comments) => format_leading_comments_impl(*comments, f),
        }
    }
}

/// Formats the trailing comments of `node`.
pub const fn format_trailing_comments<'a, 'b>(
    enclosing_node: &'b SiblingNode<'a>,
    preceding_node: &'b SiblingNode<'a>,
    following_node: Option<&'b SiblingNode<'a>>,
) -> FormatTrailingComments<'a, 'b> {
    FormatTrailingComments::Node((enclosing_node, preceding_node, following_node))
}

/// Formats the trailing comments of `node`
#[derive(Debug, Clone, Copy)]
pub enum FormatTrailingComments<'a, 'b> {
    // (enclosing_node, preceding_node, following_node)
    Node((&'b SiblingNode<'a>, &'b SiblingNode<'a>, Option<&'b SiblingNode<'a>>)),
    Comments(&'a [Comment]),
}

impl<'a> Format<'a> for FormatTrailingComments<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        fn format_trailing_comments_impl<'a>(
            comments: impl IntoIterator<Item = &'a Comment>,
            f: &mut Formatter<'_, 'a>,
        ) -> FormatResult<()> {
            let mut total_lines_before = 0;
            let mut previous_comment: Option<&Comment> = None;

            for comment in comments {
                f.context_mut().increment_printed_count();

                let lines_before = get_lines_before(comment.span, f);
                total_lines_before += lines_before;

                let should_nestle = previous_comment.is_some_and(|previous_comment| {
                    // should_nestle_adjacent_doc_comments(previous_comment, comment)
                    false
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
                if total_lines_before > 0 {
                    write!(
                        f,
                        [
                            line_suffix(&format_with(|f| {
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
                                            write!(f, [hard_line_break()])?;
                                        } else {
                                            write!(f, [space()])?;
                                        }
                                    }
                                    1 => write!(f, [hard_line_break()])?,
                                    _ => write!(f, [empty_line()])?,
                                }

                                write!(f, [comment])
                            })),
                            expand_parent()
                        ]
                    )?;
                } else {
                    let content =
                        format_with(|f| write!(f, [maybe_space(!should_nestle), comment]));

                    if comment.is_line() {
                        write!(f, [line_suffix(&content), expand_parent()])?;
                    } else {
                        write!(f, [content])?;
                    }
                }

                previous_comment = Some(comment);
            }

            Ok(())
        }

        match self {
            Self::Node((enclosing_node, preceding_node, following_node)) => {
                let comments = f.context().comments().get_trailing_comments(
                    enclosing_node,
                    preceding_node,
                    *following_node,
                );

                format_trailing_comments_impl(comments, f)
            }
            Self::Comments(comments) => format_trailing_comments_impl(*comments, f),
        }
    }
}

/// Formats the dangling comments of `node`.
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

    const fn indent(&self) -> DanglingIndentMode {
        match self {
            FormatDanglingComments::Node { indent, .. }
            | FormatDanglingComments::Comments { indent, .. } => *indent,
        }
    }
}

impl<'a> Format<'a> for FormatDanglingComments<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        fn format_dangling_comments_impl<'a>(
            comments: impl IntoIterator<Item = &'a Comment>,
            indent: DanglingIndentMode,
            f: &mut Formatter<'_, 'a>,
        ) -> FormatResult<()> {
            // Write all comments up to the first skipped token trivia or the token
            let format_dangling_comments = format_once(|f| {
                let mut previous_comment: Option<&Comment> = None;

                for comment in comments {
                    f.context_mut().increment_printed_count();

                    let should_nestle = previous_comment.is_some_and(|previous_comment| {
                        // should_nestle_adjacent_doc_comments(previous_comment, comment)
                        false
                    });

                    write!(
                        f,
                        [
                            (previous_comment.is_some() && !should_nestle)
                                .then_some(hard_line_break()),
                            comment
                        ]
                    )?;

                    previous_comment = Some(comment);
                }

                if matches!(indent, DanglingIndentMode::Soft)
                    && previous_comment.copied().is_some_and(Comment::is_line)
                {
                    write!(f, [hard_line_break()])?;
                }

                Ok(())
            });

            match indent {
                DanglingIndentMode::Block => {
                    write!(f, [block_indent(&format_dangling_comments)])
                }
                DanglingIndentMode::Soft => {
                    write!(f, [group(&soft_block_indent(&format_dangling_comments))])
                }
                DanglingIndentMode::None => {
                    write!(f, [format_dangling_comments])
                }
            }
        };

        match self {
            FormatDanglingComments::Node { span, indent } => {
                let source_text = f.context().source_text();
                format_dangling_comments_impl(
                    f.context().comments().comments_before(span.end),
                    *indent,
                    f,
                )
            }
            FormatDanglingComments::Comments { comments, indent } => {
                format_dangling_comments_impl(*comments, *indent, f)
            }
        }
    }
}

/// Formats a token without its skipped token trivia
///
/// ## Warning
/// It's your responsibility to format any skipped trivia.
pub const fn format_trimmed_token(token: &SyntaxToken) -> FormatTrimmedToken<'_> {
    FormatTrimmedToken { token }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct FormatTrimmedToken<'a> {
    token: &'a SyntaxToken,
}

// impl<C> Format<C> for FormatTrimmedToken<'_>
// where
// C: CstFormatContext<Language>,
// {
// fn fmt(&self, f: &mut Formatter<C>) -> FormatResult<()> {
// let trimmed_range = self.token.text_trimmed_range();
// located_token_text(self.token, trimmed_range).fmt(f)
// }
// }

/// Formats the skipped token trivia of a removed token and marks the token as tracked.
pub const fn format_removed(span: Span) -> FormatRemoved {
    FormatRemoved { span }
}

/// Formats the trivia of a token that is present in the source text but should be omitted in the
/// formatted output.
pub struct FormatRemoved {
    span: Span,
}

impl Format<'_> for FormatRemoved {
    fn fmt(&self, f: &mut Formatter) -> FormatResult<()> {
        f.state_mut().track_token(self.span);
        write!(f, format_skipped_token_trivia(self.span))
    }
}

/// Print out a `token` from the original source with a different `content`.
///
/// This will print the skipped token trivia that belong to `token` to `content`;
/// `token` is then marked as consumed by the formatter.
pub fn format_replaced<'content, 'ast>(
    span: Span,
    content: &'content impl Format<'ast>,
) -> FormatReplaced<'content, 'ast> {
    FormatReplaced { span, content: Argument::new(content) }
}

/// Formats a token's skipped token trivia but uses the provided content instead
/// of the token in the formatted output.
#[derive(Copy, Clone)]
pub struct FormatReplaced<'content, 'ast> {
    span: Span,
    content: Argument<'content, 'ast>,
}

impl<'ast> Format<'ast> for FormatReplaced<'_, 'ast> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        f.state_mut().track_token(self.span);
        write!(f, format_skipped_token_trivia(self.span))?;
        f.write_fmt(Arguments::from(&self.content))
    }
}

/// Formats the given token only if the group does break and otherwise retains the token's skipped token trivia.
pub fn format_only_if_breaks<'content, 'ast, Content>(
    span: Span,
    content: &'content Content,
) -> FormatOnlyIfBreaks<'content, 'ast>
where
    Content: Format<'ast>,
{
    FormatOnlyIfBreaks { span, content: Argument::new(content), group_id: None }
}

/// Formats a token with its skipped token trivia that only gets printed if its enclosing
/// group does break but otherwise gets omitted from the formatted output.
pub struct FormatOnlyIfBreaks<'content, 'ast> {
    span: Span,
    content: Argument<'content, 'ast>,
    group_id: Option<GroupId>,
}

impl FormatOnlyIfBreaks<'_, '_> {
    pub fn with_group_id(mut self, group_id: Option<GroupId>) -> Self {
        self.group_id = group_id;
        self
    }
}

impl<'ast> Format<'ast> for FormatOnlyIfBreaks<'_, 'ast> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        write!(f, if_group_breaks(&self.content).with_group_id(self.group_id))?;
        // TODO: unsupported yet
        // if f.comments().has_skipped(self.span) {
        //     // Print the trivia otherwise
        //     write!(
        //         f,
        //         if_group_fits_on_line(&format_skipped_token_trivia(self.span))
        //             .with_group_id(self.group_id)
        //     )?;
        // }
        Ok(())
    }
}

/// Formats the skipped token trivia of `token`.
pub const fn format_skipped_token_trivia(span: Span) -> FormatSkippedTokenTrivia {
    FormatSkippedTokenTrivia { span }
}

/// Formats the skipped token trivia of `token`.
pub struct FormatSkippedTokenTrivia {
    span: Span,
}

impl FormatSkippedTokenTrivia {
    #[cold]
    fn fmt_skipped(&self, f: &mut Formatter) -> FormatResult<()> {
        todo!()
        // Lines/spaces before the next token/comment
        // let (mut lines, mut spaces) = match self.token.prev_token() {
        // Some(token) => {
        // let mut lines = 0u32;
        // let mut spaces = 0u32;
        // for piece in token.trailing_trivia().pieces().rev() {
        // if piece.is_whitespace() {
        // spaces += 1;
        // } else if piece.is_newline() {
        // spaces = 0;
        // lines += 1;
        // } else {
        // break;
        // }
        // }

        // (lines, spaces)
        // }
        // None => (0, 0),
        // };

        // // The comments between the last skipped token trivia and the token
        // let mut dangling_comments = Vec::new();
        // let mut skipped_range: Option<TextRange> = None;

        // // Iterate over the remaining pieces to find the full range from the first to the last skipped token trivia.
        // // Extract the comments between the last skipped token trivia and the token.
        // for piece in self.token.leading_trivia().pieces() {
        // if piece.is_whitespace() {
        // spaces += 1;
        // continue;
        // }

        // if piece.is_newline() {
        // lines += 1;
        // spaces = 0;
        // } else if let Some(comment) = piece.as_comments() {
        // let source_comment = SourceComment {
        // kind: Context::Style::get_comment_kind(&comment),
        // lines_before: lines,
        // lines_after: 0,
        // // piece: comment,
        // #[cfg(debug_assertions)]
        // formatted: Cell::new(true),
        // };

        // dangling_comments.push(source_comment);

        // lines = 0;
        // spaces = 0;
        // } else if piece.is_skipped() {
        // skipped_range = Some(match skipped_range {
        // Some(range) => range.cover(piece.text_range()),
        // None => {
        // if dangling_comments.is_empty() {
        // match lines {
        // 0 if spaces == 0 => {
        // // Token had no space to previous token nor any preceding comment. Keep it that way
        // }
        // 0 => write!(f, [space()])?,
        // _ => write!(f, [hard_line_break()])?,
        // };
        // } else {
        // match lines {
        // 0 => write!(f, [space()])?,
        // 1 => write!(f, [hard_line_break()])?,
        // _ => write!(f, [empty_line()])?,
        // };
        // }

        // piece.text_range()
        // }
        // });

        // lines = 0;
        // spaces = 0;
        // dangling_comments.clear();
        // }
        // }

        // let skipped_range =
        // skipped_range.unwrap_or_else(|| TextRange::empty(self.token.text_range().start()));

        // f.write_element(FormatElement::Tag(Tag::StartVerbatim(VerbatimKind::Verbatim {
        // length: skipped_range.len(),
        // })))?;
        // write!(f, [located_token_text(self.token, skipped_range)])?;
        // f.write_element(FormatElement::Tag(Tag::EndVerbatim))?;

        // // Write whitespace separator between skipped/last comment and token
        // if dangling_comments.is_empty() {
        // match lines {
        // 0 if spaces == 0 => {
        // // Don't write a space if there was non in the source document
        // Ok(())
        // }
        // 0 => write!(f, [space()]),
        // _ => write!(f, [hard_line_break()]),
        // }
        // } else {
        // match dangling_comments.first().unwrap().lines_before {
        // 0 => write!(f, [space()])?,
        // 1 => write!(f, [hard_line_break()])?,
        // _ => write!(f, [empty_line()])?,
        // }

        // write!(
        // f,
        // [FormatDanglingComments::Comments {
        // comments: &dangling_comments,
        // indent: DanglingIndentMode::None
        // }]
        // )?;

        // match lines {
        // 0 => write!(f, [space()]),
        // _ => write!(f, [hard_line_break()]),
        // }
        // }
    }
}

impl Format<'_> for FormatSkippedTokenTrivia {
    fn fmt(&self, f: &mut Formatter) -> FormatResult<()> {
        // TODO: Unsupported yet
        // if f.comments().has_skipped(self.span) { self.fmt_skipped(f) } else { Ok(()) }
        Ok(())
    }
}

impl<'a> Format<'a> for Comment {
    #[expect(clippy::cast_possible_truncation)]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let source_text = self.span.source_text(f.source_text());
        if is_alignable_comment(source_text) {
            let mut source_offset = self.span.start;

            let mut lines = source_text.lines();

            // `is_alignable_comment` only returns `true` for multiline comments
            let first_line = lines.next().unwrap();
            write!(f, [dynamic_text(first_line.trim_end())])?;

            source_offset += first_line.len() as u32;

            // Indent the remaining lines by one space so that all `*` are aligned.
            write!(
                f,
                [&format_once(|f| {
                    for line in lines {
                        write!(f, [hard_line_break(), " ", dynamic_text(line.trim())])?;
                        source_offset += line.len() as u32;
                    }
                    Ok(())
                })]
            )
        } else {
            write!(f, [dynamic_text(source_text)])
        }
    }
}
