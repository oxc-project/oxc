use oxc_formatter_core::{LINE_TERMINATORS, arena_cow_str, normalize_newlines};
use oxc_span::Span;

use crate::{
    Buffer, Format,
    formatter::prelude::*,
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

pub struct FormatSuppressedNode(pub Span);

impl<'a> Format<'a, JsFormatContext<'a>> for FormatSuppressedNode {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        // The IR only supports `\n` as a line break. Normalize CRLF / CR / LS / PS to LF;
        // the printer will re-emit the configured `LineEnding` at the final stage.
        let raw = f.source_text().text_for(&self.0);
        let normalized = normalize_newlines(raw, LINE_TERMINATORS);
        write!(f, [text(arena_cow_str(&normalized, f))]);

        // The suppressed node contains comments that should be marked as printed.
        mark_comments_as_printed_before(self.0.end, f);
    }
}

fn mark_comments_as_printed_before(end: u32, f: &mut JsFormatter<'_, '_>) {
    let count = f.comments().unprinted_comments().iter().take_while(|c| c.span.end <= end).count();
    f.context_mut().comments_mut().increase_printed_count_by(count);
}
