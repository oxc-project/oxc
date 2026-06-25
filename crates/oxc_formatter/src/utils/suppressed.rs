use oxc_formatter_core::{LINE_TERMINATORS, arena_cow_str, normalize_newlines};
use oxc_span::Span;

use crate::{Buffer, Format, formatter::prelude::*, write};

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
