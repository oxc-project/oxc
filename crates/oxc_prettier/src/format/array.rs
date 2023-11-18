#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_span::Span;

use crate::{
    comment::DanglingCommentsPrintOptions, doc::Doc, group, if_break, softline, ss, Prettier,
};
use oxc_allocator::Vec;

use super::Format;

#[allow(clippy::enum_variant_names)]
pub enum Array<'a, 'b> {
    ArrayExpression(&'b ArrayExpression<'a>),
    #[allow(unused)]
    TSTupleType(&'b TSTupleType<'a>),
    ArrayPattern(&'b ArrayPattern<'a>),
    ArrayAssignmentTarget(&'b ArrayAssignmentTarget<'a>),
}

impl<'a, 'b> Array<'a, 'b> {
    fn len(&self) -> usize {
        match self {
            Self::ArrayExpression(array) => array.elements.len(),
            Self::TSTupleType(tuple) => tuple.element_types.len(),
            Self::ArrayPattern(array) => array.elements.len(),
            Self::ArrayAssignmentTarget(array) => array.elements.len(),
        }
    }
    fn span(&self) -> Span {
        match self {
            Self::ArrayExpression(array) => array.span,
            Self::TSTupleType(tuple) => tuple.span,
            Self::ArrayPattern(array) => array.span,
            Self::ArrayAssignmentTarget(array) => array.span,
        }
    }
}

pub(super) fn print_array<'a>(p: &mut Prettier<'a>, array: &Array<'a, '_>) -> Doc<'a> {
    if array.len() == 0 {
        return print_empty_array_elements(p, array);
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

fn print_empty_array_elements<'a>(p: &mut Prettier<'a>, array: &Array<'a, '_>) -> Doc<'a> {
    let dangling_options = DanglingCommentsPrintOptions::default().with_ident(true);
    p.print_dangling_comments(array.span(), Some(dangling_options)).map_or_else(
        || ss!("[]"),
        |dangling_comments| group![p, ss!("["), dangling_comments, softline!(), ss!("]")],
    )
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
        Array::ArrayPattern(array_pat) => {
            for (i, element) in array_pat.elements.iter().enumerate() {
                if i > 0 && i < array_pat.elements.len() {
                    parts.push(ss!(","));
                    parts.push(Doc::Line);
                }

                if let Some(binding_pat) = element {
                    parts.push(binding_pat.format(p));
                }
            }

            if let Some(rest) = &array_pat.rest {
                parts.push(ss!(","));
                parts.push(Doc::Line);
                parts.push(rest.format(p));
            }
        }
        Array::ArrayAssignmentTarget(array_pat) => {
            for (i, element) in array_pat.elements.iter().enumerate() {
                if i > 0 && i < array_pat.elements.len() {
                    parts.push(ss!(","));
                    parts.push(Doc::Line);
                }

                if let Some(binding_pat) = element {
                    parts.push(binding_pat.format(p));
                }
            }

            if let Some(rest) = &array_pat.rest {
                parts.push(ss!(","));
                parts.push(Doc::Line);
                parts.push(rest.format(p));
            }
        }
    }

    parts
}
