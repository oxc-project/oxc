use oxc_span::Span;

use crate::{
    Buffer, Format,
    formatter::{Formatter, prelude::*},
    write,
};

pub struct FormatSuppressedNode(pub Span);

impl<'a> Format<'a> for FormatSuppressedNode {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, [text(f.source_text().text_for(&self.0))]);

        // The suppressed node contains comments that should be marked as printed.
        mark_comments_as_printed_before(self.0.end, f);
    }
}

fn mark_comments_as_printed_before(end: u32, f: &mut Formatter<'_, '_>) {
    let count = f.comments().unprinted_comments().iter().take_while(|c| c.span.end <= end).count();
    f.context_mut().comments_mut().increase_printed_count_by(count);
}
