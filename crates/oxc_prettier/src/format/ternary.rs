#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, group, indent, ss, Format, Prettier};

impl<'a> Prettier<'a> {
    pub(super) fn print_ternary(&mut self, expr: &ConditionalExpression<'a>) -> Doc<'a> {
        group![
            self,
            expr.test.format(self),
            indent!(
                self,
                Doc::Line,
                ss!("? "),
                expr.consequent.format(self),
                Doc::Line,
                ss!(": "),
                expr.alternate.format(self)
            )
        ]
    }
}
