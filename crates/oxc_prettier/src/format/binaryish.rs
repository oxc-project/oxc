#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_syntax::operator::{BinaryOperator, LogicalOperator};

use crate::{doc::Doc, group, ss, Format, Prettier};

pub enum BinaryishLeft<'a, 'b> {
    Expression(&'b Expression<'a>),
    PrivateIdentifier(&'b PrivateIdentifier),
}

impl<'a, 'b> Format<'a> for BinaryishLeft<'a, 'b> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::Expression(expr) => expr.format(p),
            Self::PrivateIdentifier(ident) => ident.format(p),
        }
    }
}

#[derive(Clone, Copy)]
pub enum BinaryishOperator {
    BinaryOperator(BinaryOperator),
    LogicalOperator(LogicalOperator),
}

impl BinaryishOperator {
    fn as_str(self) -> &'static str {
        match self {
            Self::BinaryOperator(op) => op.as_str(),
            Self::LogicalOperator(op) => op.as_str(),
        }
    }
}

impl<'a> Prettier<'a> {
    pub(super) fn print_binaryish_expression<'b>(
        &mut self,
        left: &BinaryishLeft<'a, 'b>,
        operator: BinaryishOperator,
        right: &Expression<'a>,
    ) -> Doc<'a> {
        let mut parts = self.vec();
        parts.push(left.format(self));
        let mut parts_inner = self.vec();
        parts_inner.push(ss!(" "));
        let mut parts_inner_inner = self.vec();
        parts_inner_inner.push(ss!(operator.as_str()));
        parts_inner_inner.push(Doc::Line);
        parts_inner_inner.push(right.format(self));
        let indent = Doc::Indent(parts_inner_inner);
        parts_inner.push(group!(self, indent));
        parts.push(Doc::Indent(parts_inner));
        Doc::Group(parts)
    }
}
