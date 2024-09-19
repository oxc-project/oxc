use oxc_ast::ast::*;

use crate::{doc::Doc, group, indent, line, ss, DocBuilder, Format, Prettier};

pub(super) fn print_ternary<'a>(p: &mut Prettier<'a>, expr: &ConditionalExpression<'a>) -> Doc<'a> {
    group![
        p,
        expr.test.format(p),
        indent!(
            p,
            line!(),
            ss!("? "),
            expr.consequent.format(p),
            line!(),
            ss!(": "),
            expr.alternate.format(p)
        )
    ]
}
