#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, group, if_break, ss, Prettier};
use oxc_allocator::Vec;

use super::Format;

pub enum Array<'a, 'b> {
    ArrayExpression(&'b ArrayExpression<'a>),
    #[allow(unused)]
    TSTupleType(&'b TSTupleType<'a>),
}

impl<'a, 'b> Array<'a, 'b> {
    fn len(&self) -> usize {
        match self {
            Self::ArrayExpression(array) => array.elements.len(),
            Self::TSTupleType(tuple) => tuple.element_types.len(),
        }
    }
}

pub(super) fn print_array<'a>(p: &mut Prettier<'a>, array: &Array<'a, '_>) -> Doc<'a> {
    if array.len() == 0 {
        return ss!("[]");
    }

    let mut parts = p.vec();
    parts.push(ss!("["));

    let mut parts_inner = p.vec();
    parts_inner.push(Doc::Softline);
    parts_inner.extend(print_elements(p, array));
    parts_inner.push(if_break!(p, ","));

    parts.push(group!(p, Doc::Indent(parts_inner)));
    parts.push(Doc::Softline);
    parts.push(ss!("]"));

    Doc::Group(parts)
}

fn print_elements<'a>(p: &mut Prettier<'a>, array: &Array<'a, '_>) -> Vec<'a, Doc<'a>> {
    let mut parts = p.vec();
    match array {
        Array::ArrayExpression(array) => {
            for (i, element) in array.elements.iter().enumerate() {
                if i > 0 && i < array.elements.len() {
                    parts.push(ss!(","));
                    parts.push(Doc::Line);
                }

                parts.push(element.format(p));
            }
        }
        Array::TSTupleType(tuple) => {
            for (i, element) in tuple.element_types.iter().enumerate() {
                if i > 0 && i < tuple.element_types.len() {
                    parts.push(ss!(","));
                    parts.push(Doc::Line);
                }

                parts.push(element.format(p));
            }
        }
    }

    parts
}
