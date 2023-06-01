use oxc_syntax::{
    operator::{BinaryOperator, LogicalOperator},
    precedence::{GetPrecedence, Precedence},
};

use crate::hir::{
    ArrowExpression, AssignmentExpression, AwaitExpression, BinaryExpression, CallExpression,
    ConditionalExpression, Expression, LogicalExpression, MemberExpression, NewExpression,
    SequenceExpression, UnaryExpression, UpdateExpression, YieldExpression,
};

impl<'a> GetPrecedence for Expression<'a> {
    fn precedence(&self) -> Precedence {
        match self {
            Self::SequenceExpression(expr) => expr.precedence(),
            Self::AssignmentExpression(expr) => expr.precedence(),
            Self::YieldExpression(expr) => expr.precedence(),
            Self::ArrowExpression(expr) => expr.precedence(),
            Self::ConditionalExpression(expr) => expr.precedence(),
            Self::LogicalExpression(expr) => expr.precedence(),
            Self::BinaryExpression(expr) => expr.precedence(),
            Self::UnaryExpression(expr) => expr.precedence(),
            Self::UpdateExpression(expr) => expr.precedence(),
            Self::AwaitExpression(expr) => expr.precedence(),
            Self::NewExpression(expr) => expr.precedence(),
            Self::CallExpression(expr) => expr.precedence(),
            Self::MemberExpression(expr) => expr.precedence(),
            _ => Precedence::highest(),
        }
    }
}

impl<'a> GetPrecedence for SequenceExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Comma
    }
}

impl<'a> GetPrecedence for YieldExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Yield
    }
}

impl<'a> GetPrecedence for ArrowExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Arrow
    }
}

impl<'a> GetPrecedence for ConditionalExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Conditional
    }
}

impl<'a> GetPrecedence for AssignmentExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Assign
    }
}

impl<'a> GetPrecedence for LogicalExpression<'a> {
    fn precedence(&self) -> Precedence {
        match self.operator {
            LogicalOperator::Or => Precedence::LogicalOr,
            LogicalOperator::And => Precedence::LogicalAnd,
            LogicalOperator::Coalesce => Precedence::Coalesce,
        }
    }
}

impl<'a> GetPrecedence for BinaryExpression<'a> {
    fn precedence(&self) -> Precedence {
        match self.operator {
            BinaryOperator::BitwiseOR => Precedence::BitwiseOr,
            BinaryOperator::BitwiseXOR => Precedence::BitwiseXor,
            BinaryOperator::BitwiseAnd => Precedence::BitwiseAnd,
            BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality => Precedence::Equality,
            BinaryOperator::LessThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan
            | BinaryOperator::Instanceof
            | BinaryOperator::In => Precedence::Relational,
            BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill => Precedence::Shift,
            BinaryOperator::Subtraction | BinaryOperator::Addition => Precedence::Add,
            BinaryOperator::Multiplication
            | BinaryOperator::Remainder
            | BinaryOperator::Division => Precedence::Multiply,
            BinaryOperator::Exponential => Precedence::Exponential,
        }
    }
}

impl<'a> GetPrecedence for UnaryExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Prefix
    }
}

impl<'a> GetPrecedence for AwaitExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Prefix
    }
}

impl<'a> GetPrecedence for UpdateExpression<'a> {
    fn precedence(&self) -> Precedence {
        if self.prefix { Precedence::Prefix } else { Precedence::Postfix }
    }
}

impl<'a> GetPrecedence for CallExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Call
    }
}

impl<'a> GetPrecedence for NewExpression<'a> {
    fn precedence(&self) -> Precedence {
        if self.arguments.is_empty() { Precedence::NewWithoutArgs } else { Precedence::Call }
    }
}

impl<'a> GetPrecedence for MemberExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Member
    }
}
