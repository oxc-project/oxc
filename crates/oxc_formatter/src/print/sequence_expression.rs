use oxc_ast::ast::*;
use oxc_formatter_core::Format;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{JsFormatter, prelude::*},
    print::semicolon::write_trailing_comments_inside_parens,
    write,
};

use super::FormatWrite;

/// The sequence spans from the first element's source `(` when it is parenthesized,
/// so a comment inside those dropped parentheses sits within the sequence's span but leads the sequence,
/// not the element: it prints OUTSIDE the formatter-added parentheses
/// (`((/* c */ a), b);` -> `/* c */ (a, b);`, prettier#19894's fixpoint).
pub fn sequence_leading_comments_start(sequence: &SequenceExpression<'_>) -> u32 {
    sequence.expressions.first().map_or(sequence.span.start, |e| e.span().start)
}

impl<'a> FormatWrite<'a> for AstNode<'a, SequenceExpression<'a>> {
    fn leading_comments_start(&self) -> u32 {
        sequence_leading_comments_start(self)
    }

    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_arrow_body = matches!(self.parent(), AstNodes::ArrowFunctionExpression(_));

        let format_inner = format_with(|f| {
            let mut expressions = self.expressions().iter();
            let separator = format_with(|f| {
                write!(f, [",", line_suffix_boundary(), soft_line_break_or_space()]);
            })
            .memoized();

            write!(f, [expressions.next()]);

            if self.expressions.len() > 1 {
                write!(f, [",", line_suffix_boundary()]);
            }

            let rest = format_once(|f| {
                write!(f, soft_line_break_or_space());
                let mut joiner = f.join_with(separator);
                joiner.entries(expressions);
            });

            if matches!(self.parent(), AstNodes::ForStatement(_))
                || matches!(self.parent(), AstNodes::ExpressionStatement(_))
            {
                write!(f, [indent(&rest)]);
            } else {
                rest.fmt(f);
            }

            // Print the comments before the closing paren inside the group,
            // so they stay on the last expression's line.
            write_trailing_comments_inside_parens(f, self.parent(), self.span.end, true);
        });

        // For arrow bodies, own the `soft_block_indent` so the break decision is made
        // at the opening `(`, not at the already-indented column inside it. The arrow
        // body handler skips its own indent to defer to this group.
        if is_arrow_body {
            write!(f, group(&soft_block_indent(&format_inner)));
        } else {
            write!(f, group(&format_inner));
        }
    }
}
