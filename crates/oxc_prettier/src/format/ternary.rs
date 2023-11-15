#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{array, doc::Doc, ss, Format, Prettier};

impl<'a> Prettier<'a> {
    pub(super) fn print_ternary(&mut self, expr: &ConditionalExpression<'a>) -> Doc<'a> {
        array![
            self,
            expr.test.format(self),
            ss!(" ? "),
            expr.consequent.format(self),
            ss!(" : "),
            expr.alternate.format(self)
        ]
    }
}
