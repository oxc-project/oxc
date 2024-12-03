use oxc_ast::ast::*;

use crate::{ir::Doc, p_vec, DocBuilder, Format, Prettier};

pub(super) fn print_ternary<'a>(p: &mut Prettier<'a>, expr: &ConditionalExpression<'a>) -> Doc<'a> {
    let test_doc = expr.test.format(p);
    let consequent_doc = expr.consequent.format(p);
    let alternate_doc = expr.alternate.format(p);

    p.group(p.array(p_vec!(
        p,
        test_doc,
        p.indent(p_vec!(
            p,
            p.line(),
            p.text("? "),
            consequent_doc,
            p.line(),
            p.text(": "),
            alternate_doc
        ))
    )))
}
