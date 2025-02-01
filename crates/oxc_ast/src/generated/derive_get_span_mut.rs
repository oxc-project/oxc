// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/get_span.rs`

#![allow(clippy::match_same_arms)]

use oxc_span::{GetSpanMut, Span};

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
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

impl GetSpanMut for NumericLiteral<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for StringLiteral<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BigIntLiteral<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for RegExpLiteral<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Program<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Expression<'_> {
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

impl GetSpanMut for IdentifierName<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for IdentifierReference<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BindingIdentifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for LabelIdentifier<'_> {
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

impl GetSpanMut for ArrayExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ArrayExpressionElement<'_> {
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

impl GetSpanMut for ObjectExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ObjectPropertyKind<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ObjectProperty(it) => GetSpanMut::span_mut(&mut **it),
            Self::SpreadProperty(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for ObjectProperty<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for PropertyKey<'_> {
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

impl GetSpanMut for TemplateLiteral<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TaggedTemplateExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TemplateElement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for MemberExpression<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for ComputedMemberExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for StaticMemberExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for PrivateFieldExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for CallExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for NewExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for MetaProperty<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for SpreadElement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Argument<'_> {
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

impl GetSpanMut for UpdateExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for UnaryExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BinaryExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for PrivateInExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for LogicalExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ConditionalExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for AssignmentExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for AssignmentTarget<'_> {
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

impl GetSpanMut for SimpleAssignmentTarget<'_> {
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

impl GetSpanMut for AssignmentTargetPattern<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ArrayAssignmentTarget(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectAssignmentTarget(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for ArrayAssignmentTarget<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ObjectAssignmentTarget<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for AssignmentTargetRest<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for AssignmentTargetMaybeDefault<'_> {
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

impl GetSpanMut for AssignmentTargetWithDefault<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for AssignmentTargetProperty<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::AssignmentTargetPropertyProperty(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for AssignmentTargetPropertyIdentifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for AssignmentTargetPropertyProperty<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for SequenceExpression<'_> {
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

impl GetSpanMut for AwaitExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ChainExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ChainElement<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::CallExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for ParenthesizedExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Statement<'_> {
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

impl GetSpanMut for Directive<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Hashbang<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BlockStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Declaration<'_> {
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

impl GetSpanMut for VariableDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for VariableDeclarator<'_> {
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

impl GetSpanMut for ExpressionStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for IfStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for DoWhileStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for WhileStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ForStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ForStatementInit<'_> {
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

impl GetSpanMut for ForInStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ForStatementLeft<'_> {
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

impl GetSpanMut for ForOfStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ContinueStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BreakStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ReturnStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for WithStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for SwitchStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for SwitchCase<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for LabeledStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ThrowStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TryStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for CatchClause<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for CatchParameter<'_> {
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

impl GetSpanMut for BindingPattern<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        GetSpanMut::span_mut(&mut self.kind)
    }
}

impl GetSpanMut for BindingPatternKind<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::BindingIdentifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::ObjectPattern(it) => GetSpanMut::span_mut(&mut **it),
            Self::ArrayPattern(it) => GetSpanMut::span_mut(&mut **it),
            Self::AssignmentPattern(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for AssignmentPattern<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ObjectPattern<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BindingProperty<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ArrayPattern<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BindingRestElement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Function<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for FormalParameters<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for FormalParameter<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for FunctionBody<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ArrowFunctionExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for YieldExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Class<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ClassBody<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ClassElement<'_> {
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

impl GetSpanMut for MethodDefinition<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for PropertyDefinition<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for PrivateIdentifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for StaticBlock<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ModuleDeclaration<'_> {
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

impl GetSpanMut for AccessorProperty<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ImportExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ImportDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ImportDeclarationSpecifier<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ImportSpecifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::ImportDefaultSpecifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::ImportNamespaceSpecifier(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for ImportSpecifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ImportDefaultSpecifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ImportNamespaceSpecifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for WithClause<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ImportAttribute<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ImportAttributeKey<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => GetSpanMut::span_mut(it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(it),
        }
    }
}

impl GetSpanMut for ExportNamedDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ExportDefaultDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ExportAllDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ExportSpecifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ExportDefaultDeclarationKind<'_> {
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

impl GetSpanMut for ModuleExportName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::IdentifierName(it) => GetSpanMut::span_mut(it),
            Self::IdentifierReference(it) => GetSpanMut::span_mut(it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(it),
        }
    }
}

impl GetSpanMut for TSThisParameter<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSEnumDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSEnumMember<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSEnumMemberName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::String(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for TSTypeAnnotation<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSLiteralType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSLiteral<'_> {
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

impl GetSpanMut for TSType<'_> {
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

impl GetSpanMut for TSConditionalType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSUnionType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSIntersectionType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSParenthesizedType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeOperator<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSArrayType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSIndexedAccessType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTupleType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSNamedTupleMember<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSOptionalType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSRestType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTupleElement<'_> {
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

impl GetSpanMut for TSTypeReference<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::IdentifierReference(it) => GetSpanMut::span_mut(&mut **it),
            Self::QualifiedName(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for TSQualifiedName<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeParameterInstantiation<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeParameter<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeParameterDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeAliasDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSClassImplements<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSInterfaceDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSInterfaceBody<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSPropertySignature<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSSignature<'_> {
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

impl GetSpanMut for TSIndexSignature<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSCallSignatureDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSMethodSignature<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSConstructSignatureDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSIndexSignatureName<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSInterfaceHeritage<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypePredicate<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypePredicateName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::This(it) => GetSpanMut::span_mut(it),
        }
    }
}

impl GetSpanMut for TSModuleDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSModuleDeclarationName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => GetSpanMut::span_mut(it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(it),
        }
    }
}

impl GetSpanMut for TSModuleDeclarationBody<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSModuleDeclaration(it) => GetSpanMut::span_mut(&mut **it),
            Self::TSModuleBlock(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for TSModuleBlock<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeLiteral<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSInferType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeQuery<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeQueryExprName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSImportType(it) => GetSpanMut::span_mut(&mut **it),
            Self::IdentifierReference(it) => GetSpanMut::span_mut(&mut **it),
            Self::QualifiedName(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for TSImportType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSImportAttributes<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSImportAttribute<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSImportAttributeName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => GetSpanMut::span_mut(it),
            Self::StringLiteral(it) => GetSpanMut::span_mut(it),
        }
    }
}

impl GetSpanMut for TSFunctionType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSConstructorType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSMappedType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTemplateLiteralType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSAsExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSSatisfiesExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeAssertion<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSImportEqualsDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSModuleReference<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ExternalModuleReference(it) => GetSpanMut::span_mut(&mut **it),
            Self::IdentifierReference(it) => GetSpanMut::span_mut(&mut **it),
            Self::QualifiedName(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for TSExternalModuleReference<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSNonNullExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Decorator<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSExportAssignment<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSNamespaceExportDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSInstantiationExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSDocNullableType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSDocNonNullableType<'_> {
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

impl GetSpanMut for JSXElement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXOpeningElement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXClosingElement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXFragment<'_> {
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

impl GetSpanMut for JSXElementName<'_> {
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

impl GetSpanMut for JSXNamespacedName<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXMemberExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXMemberExpressionObject<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::IdentifierReference(it) => GetSpanMut::span_mut(&mut **it),
            Self::MemberExpression(it) => GetSpanMut::span_mut(&mut **it),
            Self::ThisExpression(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for JSXExpressionContainer<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXExpression<'_> {
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

impl GetSpanMut for JSXAttributeItem<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Attribute(it) => GetSpanMut::span_mut(&mut **it),
            Self::SpreadAttribute(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for JSXAttribute<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXSpreadAttribute<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXAttributeName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => GetSpanMut::span_mut(&mut **it),
            Self::NamespacedName(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for JSXAttributeValue<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::StringLiteral(it) => GetSpanMut::span_mut(&mut **it),
            Self::ExpressionContainer(it) => GetSpanMut::span_mut(&mut **it),
            Self::Element(it) => GetSpanMut::span_mut(&mut **it),
            Self::Fragment(it) => GetSpanMut::span_mut(&mut **it),
        }
    }
}

impl GetSpanMut for JSXIdentifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXChild<'_> {
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

impl GetSpanMut for JSXSpreadChild<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXText<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}
