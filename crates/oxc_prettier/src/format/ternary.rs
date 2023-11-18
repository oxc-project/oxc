use oxc_ast::ast::*;

use crate::{doc::Doc, group, indent, ss, Format, Prettier};

pub(super) fn print_ternary<'a>(p: &mut Prettier<'a>, expr: &ConditionalExpression<'a>) -> Doc<'a> {
    group![
        p,
        expr.test.format(p),
        indent!(
            p,
            Doc::Line,
            ss!("? "),
            expr.consequent.format(p),
            Doc::Line,
            ss!(": "),
            expr.alternate.format(p)
        )
    ]
}
