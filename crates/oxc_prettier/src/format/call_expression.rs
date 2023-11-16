use oxc_allocator::{Box, Vec};
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, ss, Format, Prettier};

impl<'a> Prettier<'a> {
    pub(super) fn print_call_expression(
        &mut self,
        callee: &Expression<'a>,
        arguments: &Vec<'a, Argument<'a>>,
        optional: bool, // for optional chaining
        type_parameters: &Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Doc<'a> {
        let mut parts = self.vec();
        parts.push(callee.format(self));
        parts.push(ss!("("));
        parts.extend(arguments.iter().map(|arg| arg.format(self)));
        parts.push(ss!(")"));
        Doc::Array(parts)
    }
}
