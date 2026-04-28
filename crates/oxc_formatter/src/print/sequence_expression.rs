use oxc_ast::ast::*;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{Format, Formatter, prelude::*},
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, SequenceExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        let is_arrow_body = matches!(
            self.parent(),
            AstNodes::ExpressionStatement(statement) if statement.is_arrow_function_body()
        );

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
                || (matches!(self.parent(), AstNodes::ExpressionStatement(statement)
                    if !statement.is_arrow_function_body()))
            {
                write!(f, [indent(&rest)]);
            } else {
                rest.fmt(f);
            }
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
