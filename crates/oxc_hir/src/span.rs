use oxc_span::{GetSpan, Span};

use crate::hir::{Expression, MemberExpression};

impl<'a> GetSpan for Expression<'a> {
    fn span(&self) -> Span {
        match self {
            Self::BooleanLiteral(e) => e.span,
            Self::NullLiteral(e) => e.span,
            Self::NumberLiteral(e) => e.span,
            Self::BigintLiteral(e) => e.span,
            Self::RegExpLiteral(e) => e.span,
            Self::StringLiteral(e) => e.span,
            Self::TemplateLiteral(e) => e.span,
            Self::Identifier(e) => e.span,
            Self::MetaProperty(e) => e.span,
            Self::Super(e) => e.span,
            Self::ArrayExpression(e) => e.span,
            Self::ArrowExpression(e) => e.span,
            Self::AssignmentExpression(e) => e.span,
            Self::AwaitExpression(e) => e.span,
            Self::BinaryExpression(e) => e.span,
            Self::PrivateInExpression(e) => e.span,
            Self::CallExpression(e) => e.span,
            Self::ChainExpression(e) => e.span,
            Self::ClassExpression(e) => e.span,
            Self::ConditionalExpression(e) => e.span,
            Self::FunctionExpression(e) => e.span,
            Self::ImportExpression(e) => e.span,
            Self::LogicalExpression(e) => e.span,
            Self::MemberExpression(e) => e.span(),
            Self::NewExpression(e) => e.span,
            Self::ObjectExpression(e) => e.span,
            Self::SequenceExpression(e) => e.span,
            Self::TaggedTemplateExpression(e) => e.span,
            Self::ThisExpression(e) => e.span,
            Self::UnaryExpression(e) => e.span,
            Self::UpdateExpression(e) => e.span,
            Self::YieldExpression(e) => e.span,
            Self::JSXElement(e) => e.span,
            Self::JSXFragment(e) => e.span,
        }
    }
}

impl<'a> GetSpan for MemberExpression<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ComputedMemberExpression(expr) => expr.span,
            Self::StaticMemberExpression(expr) => expr.span,
            Self::PrivateFieldExpression(expr) => expr.span,
        }
    }
}
