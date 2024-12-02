use oxc_ast::ast::*;

use crate::{group, ir::Doc, p_vec, DocBuilder, Format, Prettier};

pub(super) fn print_ternary<'a>(p: &mut Prettier<'a>, expr: &ConditionalExpression<'a>) -> Doc<'a> {
    let consequent = expr.consequent.format(p);
    let alternate = expr.alternate.format(p);

    group![
        p,
        expr.test.format(p),
        p._p_indent(p_vec!(
            p,
            p._p_line(),
            p._p_text("? "),
            consequent,
            p._p_line(),
            p._p_text(": "),
            alternate
        ))
    ]
}
