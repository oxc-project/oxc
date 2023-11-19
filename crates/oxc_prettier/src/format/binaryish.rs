use oxc_ast::ast::*;
use oxc_syntax::operator::{BinaryOperator, LogicalOperator};

use crate::{doc::Doc, ss, Format, Prettier};

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

pub(super) fn print_binaryish_expression<'a>(
    p: &mut Prettier<'a>,
    left: &BinaryishLeft<'a, '_>,
    operator: BinaryishOperator,
    right: &Expression<'a>,
) -> Doc<'a> {
    let mut parts = p.vec();
    match &left {
        BinaryishLeft::Expression(expr) => {
            if let Expression::LogicalExpression(logical_expr) = expr {
                parts.push(print_binaryish_expression(
                    p,
                    &BinaryishLeft::Expression(&logical_expr.left),
                    BinaryishOperator::LogicalOperator(logical_expr.operator),
                    &logical_expr.right,
                ));
            } else {
                parts.push(left.format(p));
            }
        }
        BinaryishLeft::PrivateIdentifier(ident) => {
            parts.push(left.format(p));
        }
    }
    parts.push(ss!(" "));
    parts.push(ss!(operator.as_str()));
    parts.push(Doc::Line);
    parts.push(right.format(p));
    Doc::Array(parts)
}
