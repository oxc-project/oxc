use oxc_allocator::{Box, Vec};
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    doc::{Doc, Group},
    if_break, ss, Format, Prettier,
};

use super::misc;

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

    if optional {
        parts.push(ss!("?."));
    }

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
    let should_break = arguments
        .iter()
        .any(|arg| misc::has_new_line_in_range(p.source_text, arg.span().start, arg.span().end));
    Doc::Group(Group::new(parts, should_break))
}
