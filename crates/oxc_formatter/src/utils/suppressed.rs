use oxc_formatter_core::{LINE_TERMINATORS, arena_cow_str, normalize_newlines};
use oxc_span::Span;

use crate::{
    Buffer, Format,
    formatter::{
        prelude::*,
        trivia::{FormatTrailingComments, format_leading_comments},
    },
    utils::typecast::{format_leading_comments_and_open_paren, write_suppressed_cast_target},
    write,
};

/// Prints a suppressed expression (`oxfmt-ignore` / `prettier-ignore`):
/// the single owner of the whole suppressed sequence for expression-shaped nodes
/// (leading comments, formatter-added parens, the verbatim range),
/// called by the generated `fmt` before anything of the node is printed,
/// so the cast decision is made once, with every comment still unprinted.
///
/// A cast target keeps its source cast parentheses (see `write_suppressed_cast_target`).
///
/// `needs_parentheses` promises a PARENTHESIZED output, not a formatter pair:
/// on the cast path the kept source parens satisfy it (a formatter pair on top would print `((x))`),
/// so callers must not add parens of their own around this call.
pub fn write_suppressed_expression(
    span: Span,
    leading_comments_start: u32,
    needs_parentheses: bool,
    f: &mut JsFormatter<'_, '_>,
) {
    if write_suppressed_cast_target(span, f) {
        return;
    }

    format_leading_comments_and_open_paren(span, leading_comments_start, needs_parentheses, f);
    FormatSuppressedNode(span).fmt(f);
    if needs_parentheses {
        write!(f, ")");
    }
}

/// Prints the given range of source text and marks its comments printed.
/// Nodes go through [`write_suppressed_node`] / [`write_suppressed_expression`]; this is the bytes primitive.
pub struct FormatSuppressedNode(pub Span);

impl<'a> Format<'a, JsFormatContext<'a>> for FormatSuppressedNode {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        // The IR only supports `\n` as a line break. Normalize CRLF / CR / LS / PS to LF;
        // the printer will re-emit the configured `LineEnding` at the final stage.
        let raw = f.source_text().text_for(&self.0);
        let normalized = normalize_newlines(raw, LINE_TERMINATORS);
        write!(f, [text(arena_cow_str(&normalized, f))]);

        // The suppressed node contains comments that should be marked as printed.
        f.context_mut().comments_mut().skip_comments_before(self.0.end);
    }
}

/// Prints a non-expression node's source text (`Comments::suppressed_range`)
/// and records it for the next statement's ASI guard (`previous_statement_terminated`).
/// Expression-shaped nodes go through [`write_suppressed_expression`] and never record:
/// a statement ending in a verbatim expression still prints its own `;`.
pub fn write_suppressed_node(span: Span, f: &mut JsFormatter<'_, '_>) {
    let range = f.comments().suppressed_range(span);
    FormatSuppressedNode(range).fmt(f);
    let terminated = f.source_text().text_for(&range).ends_with(';');
    f.context_mut().set_last_verbatim_node(span.end, terminated);
    // The `;` left out sits on a later line; the same-line comments before it
    // (the suppression comment itself) stay on the content's line
    if range.end < span.end {
        let comments = f.context().comments().end_of_line_comments_after(range.end);
        FormatTrailingComments::Comments(comments).fmt(f);
    }
}

/// Prints a member suppressed by a trailing comment (`p = 1; // prettier-ignore`)
/// from a container whose generated `fmt` only sees a leading one:
/// its leading comments, the source text, and the comments on its line.
pub fn write_trailing_suppressed_node(span: Span, f: &mut JsFormatter<'_, '_>) {
    format_leading_comments(span).fmt(f);
    write_suppressed_node(span, f);
    let comments = f.context().comments().end_of_line_comments_after(span.end);
    FormatTrailingComments::Comments(comments).fmt(f);
}
