use oxc_syntax::precedence::{GetPrecedence, Precedence};

use crate::ast::{
    match_member_expression, AssignmentExpression, AwaitExpression, BinaryExpression,
    CallExpression, ChainExpression, ComputedMemberExpression, ConditionalExpression, Expression,
    ImportExpression, LogicalExpression, MemberExpression, NewExpression, PrivateFieldExpression,
    SequenceExpression, StaticMemberExpression, TSTypeAssertion, UnaryExpression, UpdateExpression,
    YieldExpression,
};

impl<'a> GetPrecedence for Expression<'a> {
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
        self.operator.precedence()
    }
}

impl<'a> GetPrecedence for BinaryExpression<'a> {
    fn precedence(&self) -> Precedence {
        self.operator.precedence()
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
        if self.prefix {
            Precedence::Prefix
        } else {
            Precedence::Postfix
        }
    }
}

impl<'a> GetPrecedence for CallExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Call
    }
}

impl<'a> GetPrecedence for ImportExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Call
    }
}

impl<'a> GetPrecedence for NewExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Call
    }
}

impl<'a> GetPrecedence for ChainExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Member
    }
}

impl<'a> GetPrecedence for MemberExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Member
    }
}

impl<'a> GetPrecedence for ComputedMemberExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Member
    }
}

impl<'a> GetPrecedence for StaticMemberExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Member
    }
}

impl<'a> GetPrecedence for PrivateFieldExpression<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Member
    }
}

impl<'a> GetPrecedence for TSTypeAssertion<'a> {
    fn precedence(&self) -> Precedence {
        Precedence::Lowest
    }
}
