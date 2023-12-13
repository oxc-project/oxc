use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    operator::{BinaryOperator, LogicalOperator},
    precedence::{GetPrecedence, Precedence},
};

use crate::{Doc, Format, Prettier};

#[derive(Clone, Copy)]
pub enum BinaryishLeft<'a, 'b> {
    Expression(&'b Expression<'a>),
    PrivateIdentifier(&'b PrivateIdentifier),
}

impl<'a, 'b> From<&'b Expression<'a>> for BinaryishLeft<'a, 'b> {
    fn from(e: &'b Expression<'a>) -> Self {
        Self::Expression(e)
    }
}

impl<'a, 'b> From<&'b PrivateIdentifier> for BinaryishLeft<'a, 'b> {
    fn from(e: &'b PrivateIdentifier) -> Self {
        Self::PrivateIdentifier(e)
    }
}

impl<'a, 'b> BinaryishLeft<'a, 'b> {
    pub fn operator(&self) -> Option<BinaryishOperator> {
        match self {
            Self::Expression(Expression::BinaryExpression(e)) => {
                Some(BinaryishOperator::BinaryOperator(e.operator))
            }
            Self::Expression(Expression::LogicalExpression(e)) => {
                Some(BinaryishOperator::LogicalOperator(e.operator))
            }
            _ => None,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            Self::Expression(e) => e.span(),
            Self::PrivateIdentifier(e) => e.span,
        }
    }
}

impl<'a, 'b> Format<'a> for BinaryishLeft<'a, 'b> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::Expression(expr) => expr.format(p),
            Self::PrivateIdentifier(ident) => ident.format(p),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BinaryishOperator {
    BinaryOperator(BinaryOperator),
    LogicalOperator(LogicalOperator),
}

impl From<BinaryOperator> for BinaryishOperator {
    fn from(op: BinaryOperator) -> Self {
        Self::BinaryOperator(op)
    }
}

impl From<LogicalOperator> for BinaryishOperator {
    fn from(op: LogicalOperator) -> Self {
        Self::LogicalOperator(op)
    }
}

impl GetPrecedence for BinaryishOperator {
    fn precedence(&self) -> Precedence {
        match self {
            Self::BinaryOperator(op) => op.precedence(),
            Self::LogicalOperator(op) => op.precedence(),
        }
    }
}

impl BinaryishOperator {
    pub fn is_binary(self) -> bool {
        matches!(self, Self::BinaryOperator(_))
    }

    pub fn should_flatten(self, parent_op: Self) -> bool {
        if self.precedence() != parent_op.precedence() {
            return false;
        }

        let Self::BinaryOperator(op) = self else { return true };

        let Self::BinaryOperator(parent_op) = parent_op else { return true };

        // ** is right-associative
        // x ** y ** z --> x ** (y ** z)
        if parent_op == BinaryOperator::Exponential {
            return false;
        }

        // x == y == z --> (x == y) == z
        if parent_op.is_equality() && op.is_equality() {
            return false;
        }

        // x * y % z --> (x * y) % z
        if (op == BinaryOperator::Remainder && parent_op.is_multiplicative())
            || (parent_op == BinaryOperator::Remainder && op.is_multiplicative())
        {
            return false;
        }

        // x * y / z --> (x * y) / z
        // x / y * z --> (x / y) * z
        if op != parent_op && parent_op.is_multiplicative() && op.is_multiplicative() {
            return false;
        }

        // x << y << z --> (x << y) << z
        if parent_op.is_bitshift() && op.is_bitshift() {
            return false;
        }

        true
    }
}

impl BinaryishOperator {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BinaryOperator(op) => op.as_str(),
            Self::LogicalOperator(op) => op.as_str(),
        }
    }
}
