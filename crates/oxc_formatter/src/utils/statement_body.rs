use oxc_ast::{Comment, ast::Statement};
use oxc_formatter_core::{Buffer, Format};
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{
        JsFormatContext, JsFormatter, JsFormatterExt as _,
        prelude::{empty_line, format_once, hard_line_break, soft_line_indent_or_space, space},
        trivia::{FormatLeadingComments, FormatTrailingComments},
    },
    utils::format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
    write,
};

/// The separator between a construct's head and its `{`-delimited body,
/// followed by the pending head-side comments, which stay OUTSIDE the braces (head-body comment policy):
/// a same-line comment follows the space,
/// a line comment then forces the `{` onto the next line via its own break,
/// and an own-line comment keeps its own line:
///
/// ```js
/// while (x) /* c */ {}
///
/// while (x) // c
/// {}
///
/// if (x)
/// // c
/// {
/// }
/// ```
///
/// The caller prints the body (or its `{`) right after;
/// when the head ends with a node, print it via `FormatNodeWithoutTrailingComments`,
/// so its generic trailing pass cannot claim a line comment first.
/// A pending `space()` a caller has already emitted is safe:
/// the printer coalesces consecutive spaces and drops them at a line start.
pub fn write_head_body_separator(body_start: u32, f: &mut JsFormatter<'_, '_>) {
    let comments = f.context().comments().comments_before(body_start);
    if comments.first().is_some_and(|comment| comment.preceded_by_newline()) {
        write!(f, hard_line_break());
    } else {
        write!(f, space());
    }
    FormatLeadingComments::Comments(comments).fmt(f);
}

/// Prints `node` with its generic trailing pass suppressed, then its trailing
/// comments bounded at the next `character` (a `:` clause colon, a head's `)`).
/// The caller prints that token next; comments past it are left pending
/// (they lead whatever follows). Without the bound, the generic pass would
/// claim an end-of-line comment and flush it past the following body's `{`.
pub fn write_node_with_trailing_comments_before<'a, T>(
    node: &T,
    character: u8,
    f: &mut JsFormatter<'_, 'a>,
) where
    T: Format<'a, JsFormatContext<'a>> + GetSpan,
{
    FormatNodeWithoutTrailingComments(node).fmt(f);
    write_trailing_comments_before(node.span().end, character, f);
}

/// The node-less half of [`write_node_with_trailing_comments_before`]:
/// prints the pending comments bounded at the next `character` as trailing comments
/// (e.g. an empty for-head slot's comments before its `;`).
pub fn write_trailing_comments_before(start: u32, character: u8, f: &mut JsFormatter<'_, '_>) {
    let comments = f.context().comments().comments_before_character(start, character);
    let same_line_count =
        comments.iter().take_while(|comment| !comment.preceded_by_newline()).count();
    let (same_line, own_line) = comments.split_at(same_line_count);

    FormatTrailingComments::Comments(same_line).fmt(f);
    // Own-line comments print in place, never via `line_suffix`:
    // deferred, they would escape past the caller's token (and whatever follows before the flush).
    // The hard break both flushes a pending same-line line comment and expands the enclosing group,
    // as the own-line invariant requires;
    // the caller's token follows a block comment directly (`/* c */;`),
    // the stable form its own broken output re-parses to.
    for comment in own_line {
        if f.lines_before(comment.span) > 1 {
            write!(f, empty_line());
        } else {
            write!(f, hard_line_break());
        }
        f.context_mut().comments_mut().increment_printed_count();
        write!(f, comment);
        if comment.is_line() {
            write!(f, hard_line_break());
        }
    }
}

/// Comments between a block's `}` and a following keyword (`else`/`catch`/`finally`)
/// keep their positions: the same-line prefix trails the previous block,
/// own-line comments keep their own lines (ending with their own break or space).
///
/// A same-line line comment rides a `line_suffix` and flushes at the emitted
/// hard break — NEVER at a `line_suffix_boundary`, whose presence would
/// poison the preceding group's fits measurement and expand content that fits.
///
/// Returns whether the caller still has to write its default separator
/// before the keyword (`false` when the comments ended with their own).
pub fn write_comments_between_blocks<'a>(
    comments: &'a [Comment],
    f: &mut JsFormatter<'_, 'a>,
) -> bool {
    let same_line_count =
        comments.iter().take_while(|comment| !comment.preceded_by_newline()).count();
    let (same_line, own_line) = comments.split_at(same_line_count);

    if !same_line.is_empty() {
        FormatTrailingComments::Comments(same_line).fmt(f);
    }
    if let Some(first) = own_line.first() {
        // Flushes a pending same-line line comment, then starts the own-line run
        write!(f, hard_line_break());
        if f.lines_before(first.span) > 1 {
            write!(f, empty_line());
        }
        FormatLeadingComments::Comments(own_line).fmt(f);
        false
    } else if same_line.last().is_some_and(|comment| comment.is_line()) {
        // Only the last same-line comment can be a line comment
        // (anything after one starts a new line, landing in `own_line`)
        write!(f, hard_line_break());
        false
    } else {
        true
    }
}

pub struct FormatStatementBody<'a, 'b> {
    body: &'b AstNode<'a, Statement<'a>>,
    force_space: bool,
}

impl<'a, 'b> FormatStatementBody<'a, 'b> {
    pub fn new(body: &'b AstNode<'a, Statement<'a>>) -> Self {
        Self { body, force_space: false }
    }

    /// Prevents that the consequent is formatted on its own line and indented by one level and
    /// instead gets separated by a space.
    pub fn with_forced_space(mut self, forced: bool) -> Self {
        self.force_space = forced;
        self
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatStatementBody<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if let AstNodes::EmptyStatement(empty) = self.body.as_ast_nodes() {
            // The `;` IS the body (content), so the head-body separator applies before it too;
            // without comments no separator is written (`while (x);`).
            if f.context().comments().has_comment_before(empty.span.start) {
                write_head_body_separator(empty.span.start, f);
            }
            write!(f, empty);
        } else if let AstNodes::BlockStatement(block) = self.body.as_ast_nodes() {
            write_head_body_separator(block.span().start, f);
            write!(f, [block]);
        } else if self.force_space {
            write!(f, [space(), self.body]);
        } else {
            write!(
                f,
                [soft_line_indent_or_space(&format_once(|f| {
                    // Only live for a suppressed `if` consequent (`stuff() // oxfmt-ignore` before `else`):
                    // print it verbatim, then flush its end-of-line comments.
                    // Otherwise the `IfStatement` wrapper has hidden everything past the consequent already,
                    // and this reduces to the `else` arm.
                    let body_span = self.body.span();
                    let is_consequent_of_if_statement_parent = matches!(
                        self.body.parent(),
                        AstNodes::IfStatement(if_stmt)
                        if if_stmt.consequent.span() == body_span && if_stmt.alternate.is_some()
                    );
                    if is_consequent_of_if_statement_parent {
                        write!(f, FormatNodeWithoutTrailingComments(self.body));
                        let comments =
                            f.context().comments().end_of_line_comments_after(body_span.end);
                        FormatTrailingComments::Comments(comments).fmt(f);
                    } else {
                        write!(f, self.body);
                    }
                }))]
            );
        }
    }
}
