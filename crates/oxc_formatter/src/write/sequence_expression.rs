use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    formatter::{Format, FormatResult, Formatter, prelude::*, separated::FormatSeparatedIter},
    generated::ast_nodes::{AstNode, AstNodeIterator, AstNodes},
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, SequenceExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let format_inner = format_with(|f| {
            let mut expressions = self.expressions().iter();
            let separator = format_once(|f| {
                write!(f, [",", line_suffix_boundary(), soft_line_break_or_space()])
            })
            .memoized();

            write!(f, [expressions.next()])?;

            if self.expressions.len() > 1 {
                write!(f, [",", line_suffix_boundary()])?;
            }

            let rest = format_once(|f| {
                write!(f, soft_line_break_or_space())?;
                let mut joiner = f.join_with(separator);
                joiner.entries(expressions);
                joiner.finish()
            });

            if matches!(self.parent, AstNodes::ForStatement(_))
                || (matches!(self.parent, AstNodes::ExpressionStatement(statement) if
                    !matches!(statement.parent, AstNodes::FunctionBody(body) if matches!(body.parent, AstNodes::ArrowFunctionExpression(arrow) if arrow.expression))))
            {
                write!(f, [indent(&rest)])
            } else {
                rest.fmt(f)
            }
        });

        write!(f, group(&format_inner))
    }
}
