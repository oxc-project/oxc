// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/get_span.rs`

#![allow(clippy::match_same_arms)]

use oxc_span::{GetSpanMut, Span};

#[allow(clippy::wildcard_imports)]
use crate::ast::js::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::jsx::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::literal::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::ts::*;

impl GetSpanMut for BooleanLiteral {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for NullLiteral {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for NumericLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BigIntLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for RegExpLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for StringLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Program<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Expression<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::BooleanLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NullLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NumericLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::BigIntLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::RegExpLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::TemplateLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::Identifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::MetaProperty(it) => GetSpanMut::span_mut(&mut **it),
            Self::Super(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrayExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AssignmentExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AwaitExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::BinaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::CallExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ChainExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ClassExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ConditionalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::FunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ImportExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::LogicalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::NewExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::SequenceExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ThisExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UnaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UpdateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::YieldExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateInExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXElement(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXFragment(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSAsExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for IdentifierName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for IdentifierReference<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BindingIdentifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for LabelIdentifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ThisExpression {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ArrayExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ArrayExpressionElement<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::SpreadElement(it) => GetSpanMut::span_mut(&mut **it),
            Self::Elision(it) => GetSpanMut::span_mut(it),
            Self::BooleanLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NullLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NumericLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::BigIntLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::RegExpLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::TemplateLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::Identifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::MetaProperty(it) => GetSpanMut::span_mut(&mut **it),
            Self::Super(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrayExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AssignmentExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AwaitExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::BinaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::CallExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ChainExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ClassExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ConditionalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::FunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ImportExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::LogicalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::NewExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::SequenceExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ThisExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UnaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UpdateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::YieldExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateInExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXElement(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXFragment(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSAsExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for Elision {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ObjectExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ObjectPropertyKind<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ObjectProperty(it) => GetSpanMut::span_mut(&mut **it),
            Self::SpreadProperty(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for ObjectProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for PropertyKey<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::StaticIdentifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateIdentifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::BooleanLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NullLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NumericLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::BigIntLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::RegExpLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::TemplateLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::Identifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::MetaProperty(it) => GetSpanMut::span_mut(&mut **it),
            Self::Super(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrayExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AssignmentExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AwaitExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::BinaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::CallExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ChainExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ClassExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ConditionalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::FunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ImportExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::LogicalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::NewExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::SequenceExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ThisExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UnaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UpdateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::YieldExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateInExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXElement(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXFragment(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSAsExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for TemplateLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TaggedTemplateExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TemplateElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for MemberExpression<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for ComputedMemberExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for StaticMemberExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for PrivateFieldExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for CallExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for NewExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for MetaProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for SpreadElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Argument<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::SpreadElement(it) => GetSpanMut::span_mut(&mut **it),
            Self::BooleanLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NullLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NumericLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::BigIntLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::RegExpLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::TemplateLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::Identifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::MetaProperty(it) => GetSpanMut::span_mut(&mut **it),
            Self::Super(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrayExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AssignmentExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AwaitExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::BinaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::CallExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ChainExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ClassExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ConditionalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::FunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ImportExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::LogicalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::NewExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::SequenceExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ThisExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UnaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UpdateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::YieldExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateInExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXElement(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXFragment(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSAsExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for UpdateExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for UnaryExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BinaryExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for PrivateInExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for LogicalExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ConditionalExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for AssignmentExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for AssignmentTarget<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::AssignmentTargetIdentifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSAsExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrayAssignmentTarget(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectAssignmentTarget(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for SimpleAssignmentTarget<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::AssignmentTargetIdentifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSAsExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for AssignmentTargetPattern<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ArrayAssignmentTarget(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectAssignmentTarget(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for ArrayAssignmentTarget<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ObjectAssignmentTarget<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for AssignmentTargetRest<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for AssignmentTargetMaybeDefault<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::AssignmentTargetWithDefault(it) => GetSpanMut::span_mut(&mut **it),
            Self::AssignmentTargetIdentifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSAsExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrayAssignmentTarget(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectAssignmentTarget(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for AssignmentTargetWithDefault<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for AssignmentTargetProperty<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::AssignmentTargetPropertyProperty(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for AssignmentTargetPropertyIdentifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for AssignmentTargetPropertyProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for SequenceExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Super {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for AwaitExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ChainExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ChainElement<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::CallExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for ParenthesizedExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Statement<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::BlockStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::BreakStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::ContinueStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::DebuggerStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::DoWhileStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::EmptyStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::ExpressionStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::ForInStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::ForOfStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::ForStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::IfStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::LabeledStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::ReturnStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::SwitchStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::ThrowStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::TryStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::WhileStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::WithStatement(it) => GetSpanMut::span_mut(&mut **it),
            Self::VariableDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::FunctionDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::ClassDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeAliasDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInterfaceDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSEnumDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSModuleDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSImportEqualsDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::ImportDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::ExportAllDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::ExportDefaultDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::ExportNamedDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSExportAssignment(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNamespaceExportDeclaration(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for Directive<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Hashbang<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BlockStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Declaration<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::VariableDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::FunctionDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::ClassDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeAliasDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInterfaceDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSEnumDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSModuleDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSImportEqualsDeclaration(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for VariableDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for VariableDeclarator<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for EmptyStatement {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ExpressionStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for IfStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for DoWhileStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for WhileStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ForStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ForStatementInit<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::VariableDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::BooleanLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NullLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NumericLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::BigIntLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::RegExpLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::TemplateLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::Identifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::MetaProperty(it) => GetSpanMut::span_mut(&mut **it),
            Self::Super(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrayExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AssignmentExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AwaitExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::BinaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::CallExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ChainExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ClassExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ConditionalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::FunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ImportExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::LogicalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::NewExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::SequenceExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ThisExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UnaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UpdateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::YieldExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateInExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXElement(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXFragment(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSAsExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for ForInStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ForStatementLeft<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::VariableDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::AssignmentTargetIdentifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSAsExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrayAssignmentTarget(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectAssignmentTarget(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for ForOfStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ContinueStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BreakStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ReturnStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for WithStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for SwitchStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for SwitchCase<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for LabeledStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ThrowStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TryStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for CatchClause<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for CatchParameter<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for DebuggerStatement {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BindingPattern<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        GetSpanMut::span_mut(&mut self.kind)
    }
}

impl<'a> GetSpanMut for BindingPatternKind<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::BindingIdentifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectPattern(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrayPattern(it) => GetSpanMut::span_mut(&mut **it),
            Self::AssignmentPattern(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for AssignmentPattern<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ObjectPattern<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BindingProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ArrayPattern<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BindingRestElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Function<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for FormalParameters<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for FormalParameter<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for FunctionBody<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ArrowFunctionExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for YieldExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Class<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ClassBody<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ClassElement<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::StaticBlock(it) => GetSpanMut::span_mut(&mut **it),
            Self::MethodDefinition(it) => GetSpanMut::span_mut(&mut **it),
            Self::PropertyDefinition(it) => GetSpanMut::span_mut(&mut **it),
            Self::AccessorProperty(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSIndexSignature(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for MethodDefinition<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for PropertyDefinition<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for PrivateIdentifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for StaticBlock<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ModuleDeclaration<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ImportDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::ExportAllDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::ExportDefaultDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::ExportNamedDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSExportAssignment(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNamespaceExportDeclaration(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for AccessorProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ImportExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ImportDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ImportDeclarationSpecifier<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ImportSpecifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::ImportDefaultSpecifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::ImportNamespaceSpecifier(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for ImportSpecifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ImportDefaultSpecifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ImportNamespaceSpecifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for WithClause<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ImportAttribute<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ImportAttributeKey<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => GetSpanMut::span_mut(it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(it),
        }
    }
}

impl<'a> GetSpanMut for ExportNamedDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ExportDefaultDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ExportAllDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ExportSpecifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ExportDefaultDeclarationKind<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::FunctionDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::ClassDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInterfaceDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::BooleanLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NullLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NumericLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::BigIntLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::RegExpLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::TemplateLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::Identifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::MetaProperty(it) => GetSpanMut::span_mut(&mut **it),
            Self::Super(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrayExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AssignmentExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AwaitExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::BinaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::CallExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ChainExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ClassExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ConditionalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::FunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ImportExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::LogicalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::NewExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::SequenceExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ThisExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UnaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UpdateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::YieldExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateInExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXElement(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXFragment(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSAsExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for ModuleExportName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::IdentifierName(it) => GetSpanMut::span_mut(it),
            Self::IdentifierReference(it) => GetSpanMut::span_mut(it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(it),
        }
    }
}

impl<'a> GetSpanMut for TSThisParameter<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSEnumDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSEnumMember<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSEnumMemberName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::StaticIdentifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticStringLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticTemplateLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticNumericLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::BooleanLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NullLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NumericLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::BigIntLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::RegExpLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::TemplateLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::Identifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::MetaProperty(it) => GetSpanMut::span_mut(&mut **it),
            Self::Super(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrayExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AssignmentExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AwaitExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::BinaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::CallExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ChainExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ClassExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ConditionalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::FunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ImportExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::LogicalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::NewExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::SequenceExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ThisExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UnaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UpdateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::YieldExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateInExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXElement(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXFragment(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSAsExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for TSTypeAnnotation<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSLiteralType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSLiteral<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::BooleanLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NullLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NumericLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::BigIntLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::RegExpLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::TemplateLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::UnaryExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for TSType<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSAnyKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSBigIntKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSBooleanKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSIntrinsicKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNeverKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNullKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNumberKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSObjectKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSStringKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSSymbolKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSUndefinedKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSUnknownKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSVoidKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSArrayType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSConditionalType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSConstructorType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSFunctionType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSImportType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSIndexedAccessType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInferType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSIntersectionType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSLiteralType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSMappedType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNamedTupleMember(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSQualifiedName(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTemplateLiteralType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSThisType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTupleType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeOperatorType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypePredicate(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeQuery(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeReference(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSUnionType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSParenthesizedType(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSDocNullableType(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSDocNonNullableType(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSDocUnknownType(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for TSConditionalType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSUnionType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSIntersectionType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSParenthesizedType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeOperator<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSArrayType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSIndexedAccessType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTupleType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSNamedTupleMember<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSOptionalType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSRestType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTupleElement<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSOptionalType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSRestType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSAnyKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSBigIntKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSBooleanKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSIntrinsicKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNeverKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNullKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNumberKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSObjectKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSStringKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSSymbolKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSUndefinedKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSUnknownKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSVoidKeyword(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSArrayType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSConditionalType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSConstructorType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSFunctionType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSImportType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSIndexedAccessType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInferType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSIntersectionType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSLiteralType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSMappedType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNamedTupleMember(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSQualifiedName(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTemplateLiteralType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSThisType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTupleType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeOperatorType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypePredicate(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeQuery(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeReference(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSUnionType(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSParenthesizedType(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSDocNullableType(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSDocNonNullableType(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSDocUnknownType(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for TSAnyKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSStringKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSBooleanKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSNumberKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSNeverKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSIntrinsicKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSUnknownKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSNullKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSUndefinedKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSVoidKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSSymbolKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSThisType {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSObjectKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSBigIntKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeReference<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::IdentifierReference(it) => GetSpanMut::span_mut(&mut **it),
            Self::QualifiedName(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for TSQualifiedName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeParameterInstantiation<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeParameter<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeParameterDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeAliasDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSClassImplements<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSInterfaceDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSInterfaceBody<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSPropertySignature<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSSignature<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSIndexSignature(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSPropertySignature(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSCallSignatureDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSConstructSignatureDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSMethodSignature(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for TSIndexSignature<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSCallSignatureDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSMethodSignature<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSConstructSignatureDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSIndexSignatureName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSInterfaceHeritage<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypePredicate<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypePredicateName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::This(it) => GetSpanMut::span_mut(it),
        }
    }
}

impl<'a> GetSpanMut for TSModuleDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSModuleDeclarationName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => GetSpanMut::span_mut(it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(it),
        }
    }
}

impl<'a> GetSpanMut for TSModuleDeclarationBody<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSModuleDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSModuleBlock(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for TSModuleBlock<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSInferType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeQuery<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeQueryExprName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSImportType(it) => GetSpanMut::span_mut(&mut **it),
            Self::IdentifierReference(it) => GetSpanMut::span_mut(&mut **it),
            Self::QualifiedName(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for TSImportType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSImportAttributes<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSImportAttribute<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSImportAttributeName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => GetSpanMut::span_mut(it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(it),
        }
    }
}

impl<'a> GetSpanMut for TSFunctionType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSConstructorType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSMappedType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTemplateLiteralType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSAsExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSSatisfiesExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeAssertion<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSImportEqualsDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSModuleReference<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ExternalModuleReference(it) => GetSpanMut::span_mut(&mut **it),
            Self::IdentifierReference(it) => GetSpanMut::span_mut(&mut **it),
            Self::QualifiedName(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for TSExternalModuleReference<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSNonNullExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Decorator<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSExportAssignment<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSNamespaceExportDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSInstantiationExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSDocNullableType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSDocNonNullableType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSDocUnknownType {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXOpeningElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXClosingElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXFragment<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXOpeningFragment {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXClosingFragment {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXElementName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::IdentifierReference(it) => GetSpanMut::span_mut(&mut **it),
            Self::NamespacedName(it) => GetSpanMut::span_mut(&mut **it),
            Self::MemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ThisExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for JSXNamespacedName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXMemberExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXMemberExpressionObject<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::IdentifierReference(it) => GetSpanMut::span_mut(&mut **it),
            Self::MemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ThisExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for JSXExpressionContainer<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXExpression<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::EmptyExpression(it) => GetSpanMut::span_mut(it),
            Self::BooleanLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NullLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::NumericLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::BigIntLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::RegExpLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::TemplateLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::Identifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::MetaProperty(it) => GetSpanMut::span_mut(&mut **it),
            Self::Super(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrayExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AssignmentExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::AwaitExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::BinaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::CallExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ChainExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ClassExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ConditionalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::FunctionExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ImportExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::LogicalExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::NewExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::SequenceExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ThisExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UnaryExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::UpdateExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::YieldExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateInExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXElement(it) => GetSpanMut::span_mut(&mut **it),
            Self::JSXFragment(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSAsExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for JSXEmptyExpression {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXAttributeItem<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Attribute(it) => GetSpanMut::span_mut(&mut **it),
            Self::SpreadAttribute(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for JSXAttribute<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXSpreadAttribute<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXAttributeName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::NamespacedName(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for JSXAttributeValue<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::StringLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::ExpressionContainer(it) => GetSpanMut::span_mut(&mut **it),
            Self::Element(it) => GetSpanMut::span_mut(&mut **it),
            Self::Fragment(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for JSXIdentifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXChild<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Text(it) => GetSpanMut::span_mut(&mut **it),
            Self::Element(it) => GetSpanMut::span_mut(&mut **it),
            Self::Fragment(it) => GetSpanMut::span_mut(&mut **it),
            Self::ExpressionContainer(it) => GetSpanMut::span_mut(&mut **it),
            Self::Spread(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl<'a> GetSpanMut for JSXSpreadChild<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXText<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}
