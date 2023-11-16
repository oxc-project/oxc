#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, ss, Format, Prettier};

impl<'a> Prettier<'a> {
    pub(super) fn print_arrow_function(&mut self, expr: &ArrowExpression<'a>) -> Doc<'a> {
        let mut parts = self.vec();

        parts.push(ss!("() => "));
        parts.push(expr.body.format(self));

        Doc::Array(parts)
    }
}
