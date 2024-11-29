//! [Expression precedence trait](GetPrecedence) implementations.
use oxc_syntax::precedence::{GetPrecedence, Precedence};

use crate::ast::{
    match_member_expression, AssignmentExpression, AwaitExpression, BinaryExpression,
    CallExpression, ChainExpression, ComputedMemberExpression, ConditionalExpression, Expression,
    ImportExpression, LogicalExpression, MemberExpression, NewExpression, PrivateFieldExpression,
    SequenceExpression, StaticMemberExpression, TSTypeAssertion, UnaryExpression, UpdateExpression,
    YieldExpression,
};

impl GetPrecedence for Expression<'_> {
    fn precedence(&self) -> Precedence {
        match self {
            Self::SequenceExpression(expr) => expr.precedence(),
            Self::AssignmentExpression(expr) => expr.precedence(),
            Self::YieldExpression(expr) => expr.precedence(),
            Self::ConditionalExpression(expr) => expr.precedence(),
            Self::LogicalExpression(expr) => expr.precedence(),
            Self::BinaryExpression(expr) => expr.precedence(),
            Self::UnaryExpression(expr) => expr.precedence(),
            Self::UpdateExpression(expr) => expr.precedence(),
            Self::AwaitExpression(expr) => expr.precedence(),
            Self::NewExpression(expr) => expr.precedence(),
            Self::CallExpression(expr) => expr.precedence(),
            match_member_expression!(Self) => self.to_member_expression().precedence(),
            _ => panic!("All cases should be covered"),
        }
    }
}

impl GetPrecedence for SequenceExpression<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Comma
    }
}

impl GetPrecedence for YieldExpression<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Yield
    }
}

impl GetPrecedence for ConditionalExpression<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Conditional
    }
}

impl GetPrecedence for AssignmentExpression<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Assign
    }
}

impl GetPrecedence for LogicalExpression<'_> {
    fn precedence(&self) -> Precedence {
        self.operator.precedence()
    }
}

impl GetPrecedence for BinaryExpression<'_> {
    fn precedence(&self) -> Precedence {
        self.operator.precedence()
    }
}

impl GetPrecedence for UnaryExpression<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Prefix
    }
}

impl GetPrecedence for AwaitExpression<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Prefix
    }
}

impl GetPrecedence for UpdateExpression<'_> {
    fn precedence(&self) -> Precedence {
        if self.prefix {
            Precedence::Prefix
        } else {
            Precedence::Postfix
        }
    }
}

impl GetPrecedence for CallExpression<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Call
    }
}

impl GetPrecedence for ImportExpression<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Call
    }
}

impl GetPrecedence for NewExpression<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Call
    }
}

impl GetPrecedence for ChainExpression<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Member
    }
}

impl GetPrecedence for MemberExpression<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Member
    }
}

impl GetPrecedence for ComputedMemberExpression<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Member
    }
}

impl GetPrecedence for StaticMemberExpression<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Member
    }
}

impl GetPrecedence for PrivateFieldExpression<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Member
    }
}

impl GetPrecedence for TSTypeAssertion<'_> {
    fn precedence(&self) -> Precedence {
        Precedence::Lowest
    }
}
