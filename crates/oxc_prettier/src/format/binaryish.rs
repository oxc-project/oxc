use oxc_ast::{ast::*, AstKind};
use oxc_syntax::operator::{BinaryOperator, LogicalOperator};

use crate::{
    doc::{Doc, DocBuilder, Group},
    enter, group, ss, Format, Prettier,
};

pub enum BinaryishLeft<'a, 'b> {
    Expression(&'b Expression<'a>),
    PrivateIdentifier(&'b PrivateIdentifier),
}

impl<'a, 'b> Format<'a> for BinaryishLeft<'a, 'b> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::Expression(expr) => expr.format(p),
            Self::PrivateIdentifier(ident) => enter!(p, PrivateIdentifier, ident),
        }
    }
}

#[derive(Clone, Copy)]
pub enum BinaryishOperator {
    BinaryOperator(BinaryOperator),
    LogicalOperator(LogicalOperator),
}

impl BinaryishOperator {
    fn is_binary(self) -> bool {
        matches!(self, Self::BinaryOperator(_))
    }
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
        BinaryishLeft::Expression(expr) => match expr {
            Expression::LogicalExpression(logical) => {
                parts.push(print_binaryish_expression(
                    p,
                    &BinaryishLeft::Expression(&logical.left),
                    BinaryishOperator::LogicalOperator(logical.operator),
                    &logical.right,
                ));
            }
            Expression::BinaryExpression(binary) => {
                parts.push(print_binaryish_expression(
                    p,
                    &BinaryishLeft::Expression(&binary.left),
                    BinaryishOperator::BinaryOperator(binary.operator),
                    &binary.right,
                ));
            }
            _ => {
                parts.push(left.format(p));
            }
        },
        BinaryishLeft::PrivateIdentifier(ident) => {
            parts.push(left.format(p));
        }
    }
    parts.push(ss!(" "));

    if operator.is_binary() {
        parts.push(group!(p, ss!(operator.as_str()), Doc::Line, right.format(p)));
        Doc::Group(Group { contents: parts, should_break: false })
    } else {
        parts.push(ss!(operator.as_str()));
        parts.push(Doc::Line);
        parts.push(right.format(p));
        Doc::Array(parts)
    }
}
