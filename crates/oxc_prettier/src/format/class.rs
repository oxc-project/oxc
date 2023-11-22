use oxc_ast::ast::*;

use crate::{array, doc::Doc, hardline, indent, ss, Format, Prettier};

pub(super) fn print_class<'a>(p: &mut Prettier<'a>, class: &Class<'a>) -> Doc<'a> {
    let mut parts = p.vec();
    parts.push(ss!("class "));
    if let Some(id) = &class.id {
        parts.push(id.format(p));
        parts.push(ss!(" "));
    }

    if let Some(super_class) = &class.super_class {
        parts.push(ss!("extends "));
        parts.push(super_class.format(p));
        parts.push(ss!(" "));
    }

    parts.push(class.body.format(p));
    Doc::Array(parts)
}

pub(super) fn print_class_body<'a>(p: &mut Prettier<'a>, class_body: &ClassBody<'a>) -> Doc<'a> {
    let mut inner_parts = p.vec();

    for class_element in &class_body.body {
        inner_parts.push(class_element.format(p));
    }

    let mut parts = p.vec();
    parts.push(ss!("{"));
    if !inner_parts.is_empty() {
        parts.push(array![p, indent!(p, hardline!(), Doc::Array(inner_parts)), hardline!()]);
    }
    parts.push(ss!("}"));

    Doc::Array(parts)
}
