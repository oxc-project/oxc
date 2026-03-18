//! [Expression precedence trait](GetPrecedence) implementations.
use oxc_syntax::precedence::{GetPrecedence, Precedence};

use crate::ast::{
    AssignmentExpression, AwaitExpression, BinaryExpression, CallExpression, ChainExpression,
    ComputedMemberExpression, ConditionalExpression, Expression, ImportExpression,
    LogicalExpression, MemberExpression, NewExpression, PrivateFieldExpression, SequenceExpression,
    StaticMemberExpression, TSTypeAssertion, UnaryExpression, UpdateExpression, YieldExpression,
    match_member_expression,
};
use crate::ast::js::ExpressionKind;

impl GetPrecedence for Expression<'_> {
    fn precedence(&self) -> Precedence {
        match self.kind() {
            ExpressionKind::SequenceExpression(expr) => expr.precedence(),
            ExpressionKind::AssignmentExpression(expr) => expr.precedence(),
            ExpressionKind::YieldExpression(expr) => expr.precedence(),
            ExpressionKind::ConditionalExpression(expr) => expr.precedence(),
            ExpressionKind::LogicalExpression(expr) => expr.precedence(),
            ExpressionKind::BinaryExpression(expr) => expr.precedence(),
            ExpressionKind::UnaryExpression(expr) => expr.precedence(),
            ExpressionKind::UpdateExpression(expr) => expr.precedence(),
            ExpressionKind::AwaitExpression(expr) => expr.precedence(),
            ExpressionKind::NewExpression(expr) => expr.precedence(),
            ExpressionKind::CallExpression(expr) => expr.precedence(),
            match_member_expression!(ExpressionKind) => self.to_member_expression().precedence(),
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
        if self.prefix { Precedence::Prefix } else { Precedence::Postfix }
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
