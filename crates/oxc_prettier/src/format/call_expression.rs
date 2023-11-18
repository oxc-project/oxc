use oxc_allocator::{Box, Vec};
use oxc_ast::ast::*;

use crate::{
    doc::{Doc, Group},
    if_break, ss, Format, Prettier,
};

pub(super) fn print_call_expression<'a>(
    p: &mut Prettier<'a>,
    callee: &Expression<'a>,
    arguments: &Vec<'a, Argument<'a>>,
    optional: bool, // for optional chaining
    type_parameters: &Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    is_new: bool,
) -> Doc<'a> {
    let mut parts = p.vec();
    if is_new {
        parts.push(ss!("new "));
    }
    parts.push(callee.format(p));

    parts.push(print_call_expression_arguments(p, arguments));

    Doc::Array(parts)
}

fn print_call_expression_arguments<'a>(
    p: &mut Prettier<'a>,
    arguments: &Vec<'a, Argument<'a>>,
) -> Doc<'a> {
    let mut parts = p.vec();
    parts.push(ss!("("));

    let mut parts_inner = p.vec();
    parts_inner.push(Doc::Softline);
    for (i, element) in arguments.iter().enumerate() {
        parts_inner.push(element.format(p));
        if i < arguments.len() - 1 {
            parts_inner.push(ss!(","));
            parts_inner.push(Doc::Line);
        }
    }
    parts.push(Doc::Indent(parts_inner));
    parts.push(if_break!(p, ","));
    parts.push(Doc::Softline);
    parts.push(ss!(")"));
    Doc::Group(Group { docs: parts, group_id: None })
}
