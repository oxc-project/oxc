use oxc_ast::ast::*;

use crate::{group, indent, ir::Doc, line, text, DocBuilder, Format, Prettier};

pub(super) fn print_ternary<'a>(p: &mut Prettier<'a>, expr: &ConditionalExpression<'a>) -> Doc<'a> {
    group![
        p,
        expr.test.format(p),
        indent!(
            p,
            line!(),
            text!("? "),
            expr.consequent.format(p),
            line!(),
            text!(": "),
            expr.alternate.format(p)
        )
    ]
}
