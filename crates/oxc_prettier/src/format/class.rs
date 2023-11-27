use oxc_ast::ast::*;

use crate::{
    array,
    doc::{Doc, DocBuilder},
    hardline, ss, Format, Prettier,
};

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
        let indent = {
            let mut parts = p.vec();
            parts.extend(hardline!());
            parts.push(Doc::Array(inner_parts));
            Doc::Indent(parts)
        };
        parts.push(array![p, indent]);
        parts.extend(hardline!());
    }
    parts.push(ss!("}"));

    Doc::Array(parts)
}
