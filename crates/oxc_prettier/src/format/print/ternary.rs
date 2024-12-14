use oxc_ast::ast::*;

use crate::{group, indent, ir::Doc, line, text, Format, Prettier};

pub fn print_ternary<'a>(p: &mut Prettier<'a>, expr: &ConditionalExpression<'a>) -> Doc<'a> {
    let test_doc = expr.test.format(p);
    let consequent_doc = expr.consequent.format(p);
    let alternate_doc = expr.alternate.format(p);

    group!(
        p,
        [
            test_doc,
            indent!(p, [line!(), text!("? "), consequent_doc, line!(), text!(": "), alternate_doc])
        ]
    )
}
