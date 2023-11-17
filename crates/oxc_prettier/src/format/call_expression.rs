use oxc_allocator::{Box, Vec};
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, ss, Format, Prettier};

pub(super) fn print_call_expression<'a>(
    p: &mut Prettier<'a>,
    callee: &Expression<'a>,
    arguments: &Vec<'a, Argument<'a>>,
    optional: bool, // for optional chaining
    type_parameters: &Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
) -> Doc<'a> {
    let mut parts = p.vec();
    parts.push(callee.format(p));
    parts.push(ss!("("));
    parts.extend(arguments.iter().map(|arg| arg.format(p)));
    parts.push(ss!(")"));
    Doc::Array(parts)
}
