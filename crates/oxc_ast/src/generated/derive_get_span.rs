// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/get_span.rs`

#![allow(clippy::match_same_arms)]

use oxc_span::{GetSpan, Span};

#[allow(clippy::wildcard_imports)]
use crate::ast::js::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::jsx::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::literal::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::ts::*;

impl GetSpan for BooleanLiteral {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for NullLiteral {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for NumericLiteral<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for BigIntLiteral<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for RegExpLiteral<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for StringLiteral<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for Program<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for Expression<'a> {
    fn span(&self) -> Span {
        match self {
            Self::BooleanLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NullLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NumericLiteral(it) => GetSpan::span(it.as_ref()),
            Self::BigIntLiteral(it) => GetSpan::span(it.as_ref()),
            Self::RegExpLiteral(it) => GetSpan::span(it.as_ref()),
            Self::StringLiteral(it) => GetSpan::span(it.as_ref()),
            Self::TemplateLiteral(it) => GetSpan::span(it.as_ref()),
            Self::Identifier(it) => GetSpan::span(it.as_ref()),
            Self::MetaProperty(it) => GetSpan::span(it.as_ref()),
            Self::Super(it) => GetSpan::span(it.as_ref()),
            Self::ArrayExpression(it) => GetSpan::span(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::AssignmentExpression(it) => GetSpan::span(it.as_ref()),
            Self::AwaitExpression(it) => GetSpan::span(it.as_ref()),
            Self::BinaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::CallExpression(it) => GetSpan::span(it.as_ref()),
            Self::ChainExpression(it) => GetSpan::span(it.as_ref()),
            Self::ClassExpression(it) => GetSpan::span(it.as_ref()),
            Self::ConditionalExpression(it) => GetSpan::span(it.as_ref()),
            Self::FunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::ImportExpression(it) => GetSpan::span(it.as_ref()),
            Self::LogicalExpression(it) => GetSpan::span(it.as_ref()),
            Self::NewExpression(it) => GetSpan::span(it.as_ref()),
            Self::ObjectExpression(it) => GetSpan::span(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetSpan::span(it.as_ref()),
            Self::SequenceExpression(it) => GetSpan::span(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetSpan::span(it.as_ref()),
            Self::ThisExpression(it) => GetSpan::span(it.as_ref()),
            Self::UnaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::UpdateExpression(it) => GetSpan::span(it.as_ref()),
            Self::YieldExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateInExpression(it) => GetSpan::span(it.as_ref()),
            Self::JSXElement(it) => GetSpan::span(it.as_ref()),
            Self::JSXFragment(it) => GetSpan::span(it.as_ref()),
            Self::TSAsExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeAssertion(it) => GetSpan::span(it.as_ref()),
            Self::TSNonNullExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetSpan::span(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for IdentifierName<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for IdentifierReference<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for BindingIdentifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for LabelIdentifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ThisExpression {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ArrayExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ArrayExpressionElement<'a> {
    fn span(&self) -> Span {
        match self {
            Self::SpreadElement(it) => GetSpan::span(it.as_ref()),
            Self::Elision(it) => GetSpan::span(it),
            Self::BooleanLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NullLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NumericLiteral(it) => GetSpan::span(it.as_ref()),
            Self::BigIntLiteral(it) => GetSpan::span(it.as_ref()),
            Self::RegExpLiteral(it) => GetSpan::span(it.as_ref()),
            Self::StringLiteral(it) => GetSpan::span(it.as_ref()),
            Self::TemplateLiteral(it) => GetSpan::span(it.as_ref()),
            Self::Identifier(it) => GetSpan::span(it.as_ref()),
            Self::MetaProperty(it) => GetSpan::span(it.as_ref()),
            Self::Super(it) => GetSpan::span(it.as_ref()),
            Self::ArrayExpression(it) => GetSpan::span(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::AssignmentExpression(it) => GetSpan::span(it.as_ref()),
            Self::AwaitExpression(it) => GetSpan::span(it.as_ref()),
            Self::BinaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::CallExpression(it) => GetSpan::span(it.as_ref()),
            Self::ChainExpression(it) => GetSpan::span(it.as_ref()),
            Self::ClassExpression(it) => GetSpan::span(it.as_ref()),
            Self::ConditionalExpression(it) => GetSpan::span(it.as_ref()),
            Self::FunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::ImportExpression(it) => GetSpan::span(it.as_ref()),
            Self::LogicalExpression(it) => GetSpan::span(it.as_ref()),
            Self::NewExpression(it) => GetSpan::span(it.as_ref()),
            Self::ObjectExpression(it) => GetSpan::span(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetSpan::span(it.as_ref()),
            Self::SequenceExpression(it) => GetSpan::span(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetSpan::span(it.as_ref()),
            Self::ThisExpression(it) => GetSpan::span(it.as_ref()),
            Self::UnaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::UpdateExpression(it) => GetSpan::span(it.as_ref()),
            Self::YieldExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateInExpression(it) => GetSpan::span(it.as_ref()),
            Self::JSXElement(it) => GetSpan::span(it.as_ref()),
            Self::JSXFragment(it) => GetSpan::span(it.as_ref()),
            Self::TSAsExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeAssertion(it) => GetSpan::span(it.as_ref()),
            Self::TSNonNullExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetSpan::span(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for Elision {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ObjectExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ObjectPropertyKind<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ObjectProperty(it) => GetSpan::span(it.as_ref()),
            Self::SpreadProperty(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for ObjectProperty<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for PropertyKey<'a> {
    fn span(&self) -> Span {
        match self {
            Self::StaticIdentifier(it) => GetSpan::span(it.as_ref()),
            Self::PrivateIdentifier(it) => GetSpan::span(it.as_ref()),
            Self::BooleanLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NullLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NumericLiteral(it) => GetSpan::span(it.as_ref()),
            Self::BigIntLiteral(it) => GetSpan::span(it.as_ref()),
            Self::RegExpLiteral(it) => GetSpan::span(it.as_ref()),
            Self::StringLiteral(it) => GetSpan::span(it.as_ref()),
            Self::TemplateLiteral(it) => GetSpan::span(it.as_ref()),
            Self::Identifier(it) => GetSpan::span(it.as_ref()),
            Self::MetaProperty(it) => GetSpan::span(it.as_ref()),
            Self::Super(it) => GetSpan::span(it.as_ref()),
            Self::ArrayExpression(it) => GetSpan::span(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::AssignmentExpression(it) => GetSpan::span(it.as_ref()),
            Self::AwaitExpression(it) => GetSpan::span(it.as_ref()),
            Self::BinaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::CallExpression(it) => GetSpan::span(it.as_ref()),
            Self::ChainExpression(it) => GetSpan::span(it.as_ref()),
            Self::ClassExpression(it) => GetSpan::span(it.as_ref()),
            Self::ConditionalExpression(it) => GetSpan::span(it.as_ref()),
            Self::FunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::ImportExpression(it) => GetSpan::span(it.as_ref()),
            Self::LogicalExpression(it) => GetSpan::span(it.as_ref()),
            Self::NewExpression(it) => GetSpan::span(it.as_ref()),
            Self::ObjectExpression(it) => GetSpan::span(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetSpan::span(it.as_ref()),
            Self::SequenceExpression(it) => GetSpan::span(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetSpan::span(it.as_ref()),
            Self::ThisExpression(it) => GetSpan::span(it.as_ref()),
            Self::UnaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::UpdateExpression(it) => GetSpan::span(it.as_ref()),
            Self::YieldExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateInExpression(it) => GetSpan::span(it.as_ref()),
            Self::JSXElement(it) => GetSpan::span(it.as_ref()),
            Self::JSXFragment(it) => GetSpan::span(it.as_ref()),
            Self::TSAsExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeAssertion(it) => GetSpan::span(it.as_ref()),
            Self::TSNonNullExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetSpan::span(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for TemplateLiteral<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TaggedTemplateExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TemplateElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for MemberExpression<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for ComputedMemberExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for StaticMemberExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for PrivateFieldExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for CallExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for NewExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for MetaProperty<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for SpreadElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for Argument<'a> {
    fn span(&self) -> Span {
        match self {
            Self::SpreadElement(it) => GetSpan::span(it.as_ref()),
            Self::BooleanLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NullLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NumericLiteral(it) => GetSpan::span(it.as_ref()),
            Self::BigIntLiteral(it) => GetSpan::span(it.as_ref()),
            Self::RegExpLiteral(it) => GetSpan::span(it.as_ref()),
            Self::StringLiteral(it) => GetSpan::span(it.as_ref()),
            Self::TemplateLiteral(it) => GetSpan::span(it.as_ref()),
            Self::Identifier(it) => GetSpan::span(it.as_ref()),
            Self::MetaProperty(it) => GetSpan::span(it.as_ref()),
            Self::Super(it) => GetSpan::span(it.as_ref()),
            Self::ArrayExpression(it) => GetSpan::span(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::AssignmentExpression(it) => GetSpan::span(it.as_ref()),
            Self::AwaitExpression(it) => GetSpan::span(it.as_ref()),
            Self::BinaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::CallExpression(it) => GetSpan::span(it.as_ref()),
            Self::ChainExpression(it) => GetSpan::span(it.as_ref()),
            Self::ClassExpression(it) => GetSpan::span(it.as_ref()),
            Self::ConditionalExpression(it) => GetSpan::span(it.as_ref()),
            Self::FunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::ImportExpression(it) => GetSpan::span(it.as_ref()),
            Self::LogicalExpression(it) => GetSpan::span(it.as_ref()),
            Self::NewExpression(it) => GetSpan::span(it.as_ref()),
            Self::ObjectExpression(it) => GetSpan::span(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetSpan::span(it.as_ref()),
            Self::SequenceExpression(it) => GetSpan::span(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetSpan::span(it.as_ref()),
            Self::ThisExpression(it) => GetSpan::span(it.as_ref()),
            Self::UnaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::UpdateExpression(it) => GetSpan::span(it.as_ref()),
            Self::YieldExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateInExpression(it) => GetSpan::span(it.as_ref()),
            Self::JSXElement(it) => GetSpan::span(it.as_ref()),
            Self::JSXFragment(it) => GetSpan::span(it.as_ref()),
            Self::TSAsExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeAssertion(it) => GetSpan::span(it.as_ref()),
            Self::TSNonNullExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetSpan::span(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for UpdateExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for UnaryExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for BinaryExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for PrivateInExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for LogicalExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ConditionalExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for AssignmentExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for AssignmentTarget<'a> {
    fn span(&self) -> Span {
        match self {
            Self::AssignmentTargetIdentifier(it) => GetSpan::span(it.as_ref()),
            Self::TSAsExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSNonNullExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeAssertion(it) => GetSpan::span(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetSpan::span(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
            Self::ArrayAssignmentTarget(it) => GetSpan::span(it.as_ref()),
            Self::ObjectAssignmentTarget(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for SimpleAssignmentTarget<'a> {
    fn span(&self) -> Span {
        match self {
            Self::AssignmentTargetIdentifier(it) => GetSpan::span(it.as_ref()),
            Self::TSAsExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSNonNullExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeAssertion(it) => GetSpan::span(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetSpan::span(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for AssignmentTargetPattern<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ArrayAssignmentTarget(it) => GetSpan::span(it.as_ref()),
            Self::ObjectAssignmentTarget(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for ArrayAssignmentTarget<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ObjectAssignmentTarget<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for AssignmentTargetRest<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for AssignmentTargetMaybeDefault<'a> {
    fn span(&self) -> Span {
        match self {
            Self::AssignmentTargetWithDefault(it) => GetSpan::span(it.as_ref()),
            Self::AssignmentTargetIdentifier(it) => GetSpan::span(it.as_ref()),
            Self::TSAsExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSNonNullExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeAssertion(it) => GetSpan::span(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetSpan::span(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
            Self::ArrayAssignmentTarget(it) => GetSpan::span(it.as_ref()),
            Self::ObjectAssignmentTarget(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for AssignmentTargetWithDefault<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for AssignmentTargetProperty<'a> {
    fn span(&self) -> Span {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => GetSpan::span(it.as_ref()),
            Self::AssignmentTargetPropertyProperty(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for AssignmentTargetPropertyIdentifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for AssignmentTargetPropertyProperty<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for SequenceExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Super {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for AwaitExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ChainExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ChainElement<'a> {
    fn span(&self) -> Span {
        match self {
            Self::CallExpression(it) => GetSpan::span(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for ParenthesizedExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for Statement<'a> {
    fn span(&self) -> Span {
        match self {
            Self::BlockStatement(it) => GetSpan::span(it.as_ref()),
            Self::BreakStatement(it) => GetSpan::span(it.as_ref()),
            Self::ContinueStatement(it) => GetSpan::span(it.as_ref()),
            Self::DebuggerStatement(it) => GetSpan::span(it.as_ref()),
            Self::DoWhileStatement(it) => GetSpan::span(it.as_ref()),
            Self::EmptyStatement(it) => GetSpan::span(it.as_ref()),
            Self::ExpressionStatement(it) => GetSpan::span(it.as_ref()),
            Self::ForInStatement(it) => GetSpan::span(it.as_ref()),
            Self::ForOfStatement(it) => GetSpan::span(it.as_ref()),
            Self::ForStatement(it) => GetSpan::span(it.as_ref()),
            Self::IfStatement(it) => GetSpan::span(it.as_ref()),
            Self::LabeledStatement(it) => GetSpan::span(it.as_ref()),
            Self::ReturnStatement(it) => GetSpan::span(it.as_ref()),
            Self::SwitchStatement(it) => GetSpan::span(it.as_ref()),
            Self::ThrowStatement(it) => GetSpan::span(it.as_ref()),
            Self::TryStatement(it) => GetSpan::span(it.as_ref()),
            Self::WhileStatement(it) => GetSpan::span(it.as_ref()),
            Self::WithStatement(it) => GetSpan::span(it.as_ref()),
            Self::VariableDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::FunctionDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::ClassDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeAliasDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSInterfaceDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSEnumDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSModuleDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSImportEqualsDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::ImportDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::ExportAllDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::ExportDefaultDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::ExportNamedDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSExportAssignment(it) => GetSpan::span(it.as_ref()),
            Self::TSNamespaceExportDeclaration(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for Directive<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for Hashbang<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for BlockStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for Declaration<'a> {
    fn span(&self) -> Span {
        match self {
            Self::VariableDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::FunctionDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::ClassDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeAliasDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSInterfaceDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSEnumDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSModuleDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSImportEqualsDeclaration(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for VariableDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for VariableDeclarator<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for EmptyStatement {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ExpressionStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for IfStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for DoWhileStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for WhileStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ForStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ForStatementInit<'a> {
    fn span(&self) -> Span {
        match self {
            Self::VariableDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::BooleanLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NullLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NumericLiteral(it) => GetSpan::span(it.as_ref()),
            Self::BigIntLiteral(it) => GetSpan::span(it.as_ref()),
            Self::RegExpLiteral(it) => GetSpan::span(it.as_ref()),
            Self::StringLiteral(it) => GetSpan::span(it.as_ref()),
            Self::TemplateLiteral(it) => GetSpan::span(it.as_ref()),
            Self::Identifier(it) => GetSpan::span(it.as_ref()),
            Self::MetaProperty(it) => GetSpan::span(it.as_ref()),
            Self::Super(it) => GetSpan::span(it.as_ref()),
            Self::ArrayExpression(it) => GetSpan::span(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::AssignmentExpression(it) => GetSpan::span(it.as_ref()),
            Self::AwaitExpression(it) => GetSpan::span(it.as_ref()),
            Self::BinaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::CallExpression(it) => GetSpan::span(it.as_ref()),
            Self::ChainExpression(it) => GetSpan::span(it.as_ref()),
            Self::ClassExpression(it) => GetSpan::span(it.as_ref()),
            Self::ConditionalExpression(it) => GetSpan::span(it.as_ref()),
            Self::FunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::ImportExpression(it) => GetSpan::span(it.as_ref()),
            Self::LogicalExpression(it) => GetSpan::span(it.as_ref()),
            Self::NewExpression(it) => GetSpan::span(it.as_ref()),
            Self::ObjectExpression(it) => GetSpan::span(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetSpan::span(it.as_ref()),
            Self::SequenceExpression(it) => GetSpan::span(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetSpan::span(it.as_ref()),
            Self::ThisExpression(it) => GetSpan::span(it.as_ref()),
            Self::UnaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::UpdateExpression(it) => GetSpan::span(it.as_ref()),
            Self::YieldExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateInExpression(it) => GetSpan::span(it.as_ref()),
            Self::JSXElement(it) => GetSpan::span(it.as_ref()),
            Self::JSXFragment(it) => GetSpan::span(it.as_ref()),
            Self::TSAsExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeAssertion(it) => GetSpan::span(it.as_ref()),
            Self::TSNonNullExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetSpan::span(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for ForInStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ForStatementLeft<'a> {
    fn span(&self) -> Span {
        match self {
            Self::VariableDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::AssignmentTargetIdentifier(it) => GetSpan::span(it.as_ref()),
            Self::TSAsExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSNonNullExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeAssertion(it) => GetSpan::span(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetSpan::span(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
            Self::ArrayAssignmentTarget(it) => GetSpan::span(it.as_ref()),
            Self::ObjectAssignmentTarget(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for ForOfStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ContinueStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for BreakStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ReturnStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for WithStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for SwitchStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for SwitchCase<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for LabeledStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ThrowStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TryStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for CatchClause<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for CatchParameter<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for DebuggerStatement {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for BindingPattern<'a> {
    #[inline]
    fn span(&self) -> Span {
        GetSpan::span(&self.kind)
    }
}

impl<'a> GetSpan for BindingPatternKind<'a> {
    fn span(&self) -> Span {
        match self {
            Self::BindingIdentifier(it) => GetSpan::span(it.as_ref()),
            Self::ObjectPattern(it) => GetSpan::span(it.as_ref()),
            Self::ArrayPattern(it) => GetSpan::span(it.as_ref()),
            Self::AssignmentPattern(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for AssignmentPattern<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ObjectPattern<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for BindingProperty<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ArrayPattern<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for BindingRestElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for Function<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for FormalParameters<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for FormalParameter<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for FunctionBody<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ArrowFunctionExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for YieldExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for Class<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ClassBody<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ClassElement<'a> {
    fn span(&self) -> Span {
        match self {
            Self::StaticBlock(it) => GetSpan::span(it.as_ref()),
            Self::MethodDefinition(it) => GetSpan::span(it.as_ref()),
            Self::PropertyDefinition(it) => GetSpan::span(it.as_ref()),
            Self::AccessorProperty(it) => GetSpan::span(it.as_ref()),
            Self::TSIndexSignature(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for MethodDefinition<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for PropertyDefinition<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for PrivateIdentifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for StaticBlock<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ModuleDeclaration<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ImportDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::ExportAllDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::ExportDefaultDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::ExportNamedDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSExportAssignment(it) => GetSpan::span(it.as_ref()),
            Self::TSNamespaceExportDeclaration(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for AccessorProperty<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ImportExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ImportDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ImportDeclarationSpecifier<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ImportSpecifier(it) => GetSpan::span(it.as_ref()),
            Self::ImportDefaultSpecifier(it) => GetSpan::span(it.as_ref()),
            Self::ImportNamespaceSpecifier(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for ImportSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ImportDefaultSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ImportNamespaceSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for WithClause<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ImportAttribute<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ImportAttributeKey<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => GetSpan::span(it),
            Self::StringLiteral(it) => GetSpan::span(it),
        }
    }
}

impl<'a> GetSpan for ExportNamedDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ExportDefaultDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ExportAllDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ExportSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for ExportDefaultDeclarationKind<'a> {
    fn span(&self) -> Span {
        match self {
            Self::FunctionDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::ClassDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSInterfaceDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::BooleanLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NullLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NumericLiteral(it) => GetSpan::span(it.as_ref()),
            Self::BigIntLiteral(it) => GetSpan::span(it.as_ref()),
            Self::RegExpLiteral(it) => GetSpan::span(it.as_ref()),
            Self::StringLiteral(it) => GetSpan::span(it.as_ref()),
            Self::TemplateLiteral(it) => GetSpan::span(it.as_ref()),
            Self::Identifier(it) => GetSpan::span(it.as_ref()),
            Self::MetaProperty(it) => GetSpan::span(it.as_ref()),
            Self::Super(it) => GetSpan::span(it.as_ref()),
            Self::ArrayExpression(it) => GetSpan::span(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::AssignmentExpression(it) => GetSpan::span(it.as_ref()),
            Self::AwaitExpression(it) => GetSpan::span(it.as_ref()),
            Self::BinaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::CallExpression(it) => GetSpan::span(it.as_ref()),
            Self::ChainExpression(it) => GetSpan::span(it.as_ref()),
            Self::ClassExpression(it) => GetSpan::span(it.as_ref()),
            Self::ConditionalExpression(it) => GetSpan::span(it.as_ref()),
            Self::FunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::ImportExpression(it) => GetSpan::span(it.as_ref()),
            Self::LogicalExpression(it) => GetSpan::span(it.as_ref()),
            Self::NewExpression(it) => GetSpan::span(it.as_ref()),
            Self::ObjectExpression(it) => GetSpan::span(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetSpan::span(it.as_ref()),
            Self::SequenceExpression(it) => GetSpan::span(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetSpan::span(it.as_ref()),
            Self::ThisExpression(it) => GetSpan::span(it.as_ref()),
            Self::UnaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::UpdateExpression(it) => GetSpan::span(it.as_ref()),
            Self::YieldExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateInExpression(it) => GetSpan::span(it.as_ref()),
            Self::JSXElement(it) => GetSpan::span(it.as_ref()),
            Self::JSXFragment(it) => GetSpan::span(it.as_ref()),
            Self::TSAsExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeAssertion(it) => GetSpan::span(it.as_ref()),
            Self::TSNonNullExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetSpan::span(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for ModuleExportName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::IdentifierName(it) => GetSpan::span(it),
            Self::IdentifierReference(it) => GetSpan::span(it),
            Self::StringLiteral(it) => GetSpan::span(it),
        }
    }
}

impl<'a> GetSpan for TSThisParameter<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSEnumDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSEnumMember<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSEnumMemberName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::StaticIdentifier(it) => GetSpan::span(it.as_ref()),
            Self::StaticStringLiteral(it) => GetSpan::span(it.as_ref()),
            Self::StaticTemplateLiteral(it) => GetSpan::span(it.as_ref()),
            Self::StaticNumericLiteral(it) => GetSpan::span(it.as_ref()),
            Self::BooleanLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NullLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NumericLiteral(it) => GetSpan::span(it.as_ref()),
            Self::BigIntLiteral(it) => GetSpan::span(it.as_ref()),
            Self::RegExpLiteral(it) => GetSpan::span(it.as_ref()),
            Self::StringLiteral(it) => GetSpan::span(it.as_ref()),
            Self::TemplateLiteral(it) => GetSpan::span(it.as_ref()),
            Self::Identifier(it) => GetSpan::span(it.as_ref()),
            Self::MetaProperty(it) => GetSpan::span(it.as_ref()),
            Self::Super(it) => GetSpan::span(it.as_ref()),
            Self::ArrayExpression(it) => GetSpan::span(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::AssignmentExpression(it) => GetSpan::span(it.as_ref()),
            Self::AwaitExpression(it) => GetSpan::span(it.as_ref()),
            Self::BinaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::CallExpression(it) => GetSpan::span(it.as_ref()),
            Self::ChainExpression(it) => GetSpan::span(it.as_ref()),
            Self::ClassExpression(it) => GetSpan::span(it.as_ref()),
            Self::ConditionalExpression(it) => GetSpan::span(it.as_ref()),
            Self::FunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::ImportExpression(it) => GetSpan::span(it.as_ref()),
            Self::LogicalExpression(it) => GetSpan::span(it.as_ref()),
            Self::NewExpression(it) => GetSpan::span(it.as_ref()),
            Self::ObjectExpression(it) => GetSpan::span(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetSpan::span(it.as_ref()),
            Self::SequenceExpression(it) => GetSpan::span(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetSpan::span(it.as_ref()),
            Self::ThisExpression(it) => GetSpan::span(it.as_ref()),
            Self::UnaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::UpdateExpression(it) => GetSpan::span(it.as_ref()),
            Self::YieldExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateInExpression(it) => GetSpan::span(it.as_ref()),
            Self::JSXElement(it) => GetSpan::span(it.as_ref()),
            Self::JSXFragment(it) => GetSpan::span(it.as_ref()),
            Self::TSAsExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeAssertion(it) => GetSpan::span(it.as_ref()),
            Self::TSNonNullExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetSpan::span(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for TSTypeAnnotation<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSLiteralType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSLiteral<'a> {
    fn span(&self) -> Span {
        match self {
            Self::BooleanLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NullLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NumericLiteral(it) => GetSpan::span(it.as_ref()),
            Self::BigIntLiteral(it) => GetSpan::span(it.as_ref()),
            Self::RegExpLiteral(it) => GetSpan::span(it.as_ref()),
            Self::StringLiteral(it) => GetSpan::span(it.as_ref()),
            Self::TemplateLiteral(it) => GetSpan::span(it.as_ref()),
            Self::UnaryExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for TSType<'a> {
    fn span(&self) -> Span {
        match self {
            Self::TSAnyKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSBigIntKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSBooleanKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSIntrinsicKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSNeverKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSNullKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSNumberKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSObjectKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSStringKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSSymbolKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSUndefinedKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSUnknownKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSVoidKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSArrayType(it) => GetSpan::span(it.as_ref()),
            Self::TSConditionalType(it) => GetSpan::span(it.as_ref()),
            Self::TSConstructorType(it) => GetSpan::span(it.as_ref()),
            Self::TSFunctionType(it) => GetSpan::span(it.as_ref()),
            Self::TSImportType(it) => GetSpan::span(it.as_ref()),
            Self::TSIndexedAccessType(it) => GetSpan::span(it.as_ref()),
            Self::TSInferType(it) => GetSpan::span(it.as_ref()),
            Self::TSIntersectionType(it) => GetSpan::span(it.as_ref()),
            Self::TSLiteralType(it) => GetSpan::span(it.as_ref()),
            Self::TSMappedType(it) => GetSpan::span(it.as_ref()),
            Self::TSNamedTupleMember(it) => GetSpan::span(it.as_ref()),
            Self::TSQualifiedName(it) => GetSpan::span(it.as_ref()),
            Self::TSTemplateLiteralType(it) => GetSpan::span(it.as_ref()),
            Self::TSThisType(it) => GetSpan::span(it.as_ref()),
            Self::TSTupleType(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeLiteral(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeOperatorType(it) => GetSpan::span(it.as_ref()),
            Self::TSTypePredicate(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeQuery(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeReference(it) => GetSpan::span(it.as_ref()),
            Self::TSUnionType(it) => GetSpan::span(it.as_ref()),
            Self::TSParenthesizedType(it) => GetSpan::span(it.as_ref()),
            Self::JSDocNullableType(it) => GetSpan::span(it.as_ref()),
            Self::JSDocNonNullableType(it) => GetSpan::span(it.as_ref()),
            Self::JSDocUnknownType(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for TSConditionalType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSUnionType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSIntersectionType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSParenthesizedType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTypeOperator<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSArrayType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSIndexedAccessType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTupleType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSNamedTupleMember<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSOptionalType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSRestType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTupleElement<'a> {
    fn span(&self) -> Span {
        match self {
            Self::TSOptionalType(it) => GetSpan::span(it.as_ref()),
            Self::TSRestType(it) => GetSpan::span(it.as_ref()),
            Self::TSAnyKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSBigIntKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSBooleanKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSIntrinsicKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSNeverKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSNullKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSNumberKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSObjectKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSStringKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSSymbolKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSUndefinedKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSUnknownKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSVoidKeyword(it) => GetSpan::span(it.as_ref()),
            Self::TSArrayType(it) => GetSpan::span(it.as_ref()),
            Self::TSConditionalType(it) => GetSpan::span(it.as_ref()),
            Self::TSConstructorType(it) => GetSpan::span(it.as_ref()),
            Self::TSFunctionType(it) => GetSpan::span(it.as_ref()),
            Self::TSImportType(it) => GetSpan::span(it.as_ref()),
            Self::TSIndexedAccessType(it) => GetSpan::span(it.as_ref()),
            Self::TSInferType(it) => GetSpan::span(it.as_ref()),
            Self::TSIntersectionType(it) => GetSpan::span(it.as_ref()),
            Self::TSLiteralType(it) => GetSpan::span(it.as_ref()),
            Self::TSMappedType(it) => GetSpan::span(it.as_ref()),
            Self::TSNamedTupleMember(it) => GetSpan::span(it.as_ref()),
            Self::TSQualifiedName(it) => GetSpan::span(it.as_ref()),
            Self::TSTemplateLiteralType(it) => GetSpan::span(it.as_ref()),
            Self::TSThisType(it) => GetSpan::span(it.as_ref()),
            Self::TSTupleType(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeLiteral(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeOperatorType(it) => GetSpan::span(it.as_ref()),
            Self::TSTypePredicate(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeQuery(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeReference(it) => GetSpan::span(it.as_ref()),
            Self::TSUnionType(it) => GetSpan::span(it.as_ref()),
            Self::TSParenthesizedType(it) => GetSpan::span(it.as_ref()),
            Self::JSDocNullableType(it) => GetSpan::span(it.as_ref()),
            Self::JSDocNonNullableType(it) => GetSpan::span(it.as_ref()),
            Self::JSDocUnknownType(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for TSAnyKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSStringKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSBooleanKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSNumberKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSNeverKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSIntrinsicKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSUnknownKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSNullKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSUndefinedKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSVoidKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSSymbolKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSThisType {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSObjectKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSBigIntKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTypeReference<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTypeName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::IdentifierReference(it) => GetSpan::span(it.as_ref()),
            Self::QualifiedName(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for TSQualifiedName<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTypeParameterInstantiation<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTypeParameter<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTypeParameterDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTypeAliasDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSClassImplements<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSInterfaceDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSInterfaceBody<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSPropertySignature<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSSignature<'a> {
    fn span(&self) -> Span {
        match self {
            Self::TSIndexSignature(it) => GetSpan::span(it.as_ref()),
            Self::TSPropertySignature(it) => GetSpan::span(it.as_ref()),
            Self::TSCallSignatureDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSConstructSignatureDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSMethodSignature(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for TSIndexSignature<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSCallSignatureDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSMethodSignature<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSConstructSignatureDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSIndexSignatureName<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSInterfaceHeritage<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTypePredicate<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTypePredicateName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => GetSpan::span(it.as_ref()),
            Self::This(it) => GetSpan::span(it),
        }
    }
}

impl<'a> GetSpan for TSModuleDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSModuleDeclarationName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => GetSpan::span(it),
            Self::StringLiteral(it) => GetSpan::span(it),
        }
    }
}

impl<'a> GetSpan for TSModuleDeclarationBody<'a> {
    fn span(&self) -> Span {
        match self {
            Self::TSModuleDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSModuleBlock(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for TSModuleBlock<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTypeLiteral<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSInferType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTypeQuery<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTypeQueryExprName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::TSImportType(it) => GetSpan::span(it.as_ref()),
            Self::IdentifierReference(it) => GetSpan::span(it.as_ref()),
            Self::QualifiedName(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for TSImportType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSImportAttributes<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSImportAttribute<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSImportAttributeName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => GetSpan::span(it),
            Self::StringLiteral(it) => GetSpan::span(it),
        }
    }
}

impl<'a> GetSpan for TSFunctionType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSConstructorType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSMappedType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTemplateLiteralType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSAsExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSSatisfiesExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSTypeAssertion<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSImportEqualsDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSModuleReference<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ExternalModuleReference(it) => GetSpan::span(it.as_ref()),
            Self::IdentifierReference(it) => GetSpan::span(it.as_ref()),
            Self::QualifiedName(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for TSExternalModuleReference<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSNonNullExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for Decorator<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSExportAssignment<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSNamespaceExportDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for TSInstantiationExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSDocNullableType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSDocNonNullableType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSDocUnknownType {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSXElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSXOpeningElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSXClosingElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSXFragment<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXOpeningFragment {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXClosingFragment {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSXElementName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => GetSpan::span(it.as_ref()),
            Self::IdentifierReference(it) => GetSpan::span(it.as_ref()),
            Self::NamespacedName(it) => GetSpan::span(it.as_ref()),
            Self::MemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::ThisExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for JSXNamespacedName<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSXMemberExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSXMemberExpressionObject<'a> {
    fn span(&self) -> Span {
        match self {
            Self::IdentifierReference(it) => GetSpan::span(it.as_ref()),
            Self::MemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::ThisExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for JSXExpressionContainer<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSXExpression<'a> {
    fn span(&self) -> Span {
        match self {
            Self::EmptyExpression(it) => GetSpan::span(it),
            Self::BooleanLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NullLiteral(it) => GetSpan::span(it.as_ref()),
            Self::NumericLiteral(it) => GetSpan::span(it.as_ref()),
            Self::BigIntLiteral(it) => GetSpan::span(it.as_ref()),
            Self::RegExpLiteral(it) => GetSpan::span(it.as_ref()),
            Self::StringLiteral(it) => GetSpan::span(it.as_ref()),
            Self::TemplateLiteral(it) => GetSpan::span(it.as_ref()),
            Self::Identifier(it) => GetSpan::span(it.as_ref()),
            Self::MetaProperty(it) => GetSpan::span(it.as_ref()),
            Self::Super(it) => GetSpan::span(it.as_ref()),
            Self::ArrayExpression(it) => GetSpan::span(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::AssignmentExpression(it) => GetSpan::span(it.as_ref()),
            Self::AwaitExpression(it) => GetSpan::span(it.as_ref()),
            Self::BinaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::CallExpression(it) => GetSpan::span(it.as_ref()),
            Self::ChainExpression(it) => GetSpan::span(it.as_ref()),
            Self::ClassExpression(it) => GetSpan::span(it.as_ref()),
            Self::ConditionalExpression(it) => GetSpan::span(it.as_ref()),
            Self::FunctionExpression(it) => GetSpan::span(it.as_ref()),
            Self::ImportExpression(it) => GetSpan::span(it.as_ref()),
            Self::LogicalExpression(it) => GetSpan::span(it.as_ref()),
            Self::NewExpression(it) => GetSpan::span(it.as_ref()),
            Self::ObjectExpression(it) => GetSpan::span(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetSpan::span(it.as_ref()),
            Self::SequenceExpression(it) => GetSpan::span(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetSpan::span(it.as_ref()),
            Self::ThisExpression(it) => GetSpan::span(it.as_ref()),
            Self::UnaryExpression(it) => GetSpan::span(it.as_ref()),
            Self::UpdateExpression(it) => GetSpan::span(it.as_ref()),
            Self::YieldExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateInExpression(it) => GetSpan::span(it.as_ref()),
            Self::JSXElement(it) => GetSpan::span(it.as_ref()),
            Self::JSXFragment(it) => GetSpan::span(it.as_ref()),
            Self::TSAsExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSTypeAssertion(it) => GetSpan::span(it.as_ref()),
            Self::TSNonNullExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetSpan::span(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for JSXEmptyExpression {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSXAttributeItem<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Attribute(it) => GetSpan::span(it.as_ref()),
            Self::SpreadAttribute(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for JSXAttribute<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSXSpreadAttribute<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSXAttributeName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => GetSpan::span(it.as_ref()),
            Self::NamespacedName(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for JSXAttributeValue<'a> {
    fn span(&self) -> Span {
        match self {
            Self::StringLiteral(it) => GetSpan::span(it.as_ref()),
            Self::ExpressionContainer(it) => GetSpan::span(it.as_ref()),
            Self::Element(it) => GetSpan::span(it.as_ref()),
            Self::Fragment(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for JSXIdentifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSXChild<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Text(it) => GetSpan::span(it.as_ref()),
            Self::Element(it) => GetSpan::span(it.as_ref()),
            Self::Fragment(it) => GetSpan::span(it.as_ref()),
            Self::ExpressionContainer(it) => GetSpan::span(it.as_ref()),
            Self::Spread(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl<'a> GetSpan for JSXSpreadChild<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpan for JSXText<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
