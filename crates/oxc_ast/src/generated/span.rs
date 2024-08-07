// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_codegen/src/generators/impl_get_span.rs`

#![allow(clippy::match_same_arms)]

use crate::ast::*;
use oxc_span::{GetSpan, GetSpanMut, Span};

impl GetSpan for BooleanLiteral {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for BooleanLiteral {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for NullLiteral {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for NullLiteral {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for NumericLiteral<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for NumericLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for BigIntLiteral<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for BigIntLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for RegExpLiteral<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for RegExpLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for StringLiteral<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for StringLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for Program<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for Program<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for Expression<'a> {
    fn span(&self) -> Span {
        match self {
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for Expression<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for IdentifierName<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for IdentifierName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for IdentifierReference<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for IdentifierReference<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for BindingIdentifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for BindingIdentifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for LabelIdentifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for LabelIdentifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for ThisExpression {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for ThisExpression {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ArrayExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ArrayExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ArrayExpressionElement<'a> {
    fn span(&self) -> Span {
        match self {
            Self::SpreadElement(it) => it.span(),
            Self::Elision(it) => it.span(),
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for ArrayExpressionElement<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::SpreadElement(it) => it.span_mut(),
            Self::Elision(it) => it.span_mut(),
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpan for Elision {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for Elision {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ObjectExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ObjectExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ObjectPropertyKind<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ObjectProperty(it) => it.span(),
            Self::SpreadProperty(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for ObjectPropertyKind<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ObjectProperty(it) => it.span_mut(),
            Self::SpreadProperty(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for ObjectProperty<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ObjectProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for PropertyKey<'a> {
    fn span(&self) -> Span {
        match self {
            Self::StaticIdentifier(it) => it.span(),
            Self::PrivateIdentifier(it) => it.span(),
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for PropertyKey<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::StaticIdentifier(it) => it.span_mut(),
            Self::PrivateIdentifier(it) => it.span_mut(),
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for TemplateLiteral<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TemplateLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TaggedTemplateExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TaggedTemplateExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TemplateElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TemplateElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for MemberExpression<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for MemberExpression<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for ComputedMemberExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ComputedMemberExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for StaticMemberExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for StaticMemberExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for PrivateFieldExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for PrivateFieldExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for CallExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for CallExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for NewExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for NewExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for MetaProperty<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for MetaProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for SpreadElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for SpreadElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for Argument<'a> {
    fn span(&self) -> Span {
        match self {
            Self::SpreadElement(it) => it.span(),
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for Argument<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::SpreadElement(it) => it.span_mut(),
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for UpdateExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for UpdateExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for UnaryExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for UnaryExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for BinaryExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for BinaryExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for PrivateInExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for PrivateInExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for LogicalExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for LogicalExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ConditionalExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ConditionalExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for AssignmentExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for AssignmentExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for AssignmentTarget<'a> {
    fn span(&self) -> Span {
        match self {
            Self::AssignmentTargetIdentifier(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
            Self::ArrayAssignmentTarget(it) => it.span(),
            Self::ObjectAssignmentTarget(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for AssignmentTarget<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::AssignmentTargetIdentifier(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
            Self::ArrayAssignmentTarget(it) => it.span_mut(),
            Self::ObjectAssignmentTarget(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for SimpleAssignmentTarget<'a> {
    fn span(&self) -> Span {
        match self {
            Self::AssignmentTargetIdentifier(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for SimpleAssignmentTarget<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::AssignmentTargetIdentifier(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for AssignmentTargetPattern<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ArrayAssignmentTarget(it) => it.span(),
            Self::ObjectAssignmentTarget(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for AssignmentTargetPattern<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ArrayAssignmentTarget(it) => it.span_mut(),
            Self::ObjectAssignmentTarget(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for ArrayAssignmentTarget<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ArrayAssignmentTarget<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ObjectAssignmentTarget<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ObjectAssignmentTarget<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for AssignmentTargetRest<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for AssignmentTargetRest<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for AssignmentTargetMaybeDefault<'a> {
    fn span(&self) -> Span {
        match self {
            Self::AssignmentTargetWithDefault(it) => it.span(),
            Self::AssignmentTargetIdentifier(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
            Self::ArrayAssignmentTarget(it) => it.span(),
            Self::ObjectAssignmentTarget(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for AssignmentTargetMaybeDefault<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::AssignmentTargetWithDefault(it) => it.span_mut(),
            Self::AssignmentTargetIdentifier(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
            Self::ArrayAssignmentTarget(it) => it.span_mut(),
            Self::ObjectAssignmentTarget(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for AssignmentTargetWithDefault<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for AssignmentTargetWithDefault<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for AssignmentTargetProperty<'a> {
    fn span(&self) -> Span {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => it.span(),
            Self::AssignmentTargetPropertyProperty(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for AssignmentTargetProperty<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => it.span_mut(),
            Self::AssignmentTargetPropertyProperty(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for AssignmentTargetPropertyIdentifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for AssignmentTargetPropertyIdentifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for AssignmentTargetPropertyProperty<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for AssignmentTargetPropertyProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for SequenceExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for SequenceExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for Super {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for Super {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for AwaitExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for AwaitExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ChainExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ChainExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ChainElement<'a> {
    fn span(&self) -> Span {
        match self {
            Self::CallExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for ChainElement<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::CallExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for ParenthesizedExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ParenthesizedExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for Statement<'a> {
    fn span(&self) -> Span {
        match self {
            Self::BlockStatement(it) => it.span(),
            Self::BreakStatement(it) => it.span(),
            Self::ContinueStatement(it) => it.span(),
            Self::DebuggerStatement(it) => it.span(),
            Self::DoWhileStatement(it) => it.span(),
            Self::EmptyStatement(it) => it.span(),
            Self::ExpressionStatement(it) => it.span(),
            Self::ForInStatement(it) => it.span(),
            Self::ForOfStatement(it) => it.span(),
            Self::ForStatement(it) => it.span(),
            Self::IfStatement(it) => it.span(),
            Self::LabeledStatement(it) => it.span(),
            Self::ReturnStatement(it) => it.span(),
            Self::SwitchStatement(it) => it.span(),
            Self::ThrowStatement(it) => it.span(),
            Self::TryStatement(it) => it.span(),
            Self::WhileStatement(it) => it.span(),
            Self::WithStatement(it) => it.span(),
            Self::VariableDeclaration(it) => it.span(),
            Self::FunctionDeclaration(it) => it.span(),
            Self::ClassDeclaration(it) => it.span(),
            Self::UsingDeclaration(it) => it.span(),
            Self::TSTypeAliasDeclaration(it) => it.span(),
            Self::TSInterfaceDeclaration(it) => it.span(),
            Self::TSEnumDeclaration(it) => it.span(),
            Self::TSModuleDeclaration(it) => it.span(),
            Self::TSImportEqualsDeclaration(it) => it.span(),
            Self::ImportDeclaration(it) => it.span(),
            Self::ExportAllDeclaration(it) => it.span(),
            Self::ExportDefaultDeclaration(it) => it.span(),
            Self::ExportNamedDeclaration(it) => it.span(),
            Self::TSExportAssignment(it) => it.span(),
            Self::TSNamespaceExportDeclaration(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for Statement<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::BlockStatement(it) => it.span_mut(),
            Self::BreakStatement(it) => it.span_mut(),
            Self::ContinueStatement(it) => it.span_mut(),
            Self::DebuggerStatement(it) => it.span_mut(),
            Self::DoWhileStatement(it) => it.span_mut(),
            Self::EmptyStatement(it) => it.span_mut(),
            Self::ExpressionStatement(it) => it.span_mut(),
            Self::ForInStatement(it) => it.span_mut(),
            Self::ForOfStatement(it) => it.span_mut(),
            Self::ForStatement(it) => it.span_mut(),
            Self::IfStatement(it) => it.span_mut(),
            Self::LabeledStatement(it) => it.span_mut(),
            Self::ReturnStatement(it) => it.span_mut(),
            Self::SwitchStatement(it) => it.span_mut(),
            Self::ThrowStatement(it) => it.span_mut(),
            Self::TryStatement(it) => it.span_mut(),
            Self::WhileStatement(it) => it.span_mut(),
            Self::WithStatement(it) => it.span_mut(),
            Self::VariableDeclaration(it) => it.span_mut(),
            Self::FunctionDeclaration(it) => it.span_mut(),
            Self::ClassDeclaration(it) => it.span_mut(),
            Self::UsingDeclaration(it) => it.span_mut(),
            Self::TSTypeAliasDeclaration(it) => it.span_mut(),
            Self::TSInterfaceDeclaration(it) => it.span_mut(),
            Self::TSEnumDeclaration(it) => it.span_mut(),
            Self::TSModuleDeclaration(it) => it.span_mut(),
            Self::TSImportEqualsDeclaration(it) => it.span_mut(),
            Self::ImportDeclaration(it) => it.span_mut(),
            Self::ExportAllDeclaration(it) => it.span_mut(),
            Self::ExportDefaultDeclaration(it) => it.span_mut(),
            Self::ExportNamedDeclaration(it) => it.span_mut(),
            Self::TSExportAssignment(it) => it.span_mut(),
            Self::TSNamespaceExportDeclaration(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for Directive<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for Directive<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for Hashbang<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for Hashbang<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for BlockStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for BlockStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for Declaration<'a> {
    fn span(&self) -> Span {
        match self {
            Self::VariableDeclaration(it) => it.span(),
            Self::FunctionDeclaration(it) => it.span(),
            Self::ClassDeclaration(it) => it.span(),
            Self::UsingDeclaration(it) => it.span(),
            Self::TSTypeAliasDeclaration(it) => it.span(),
            Self::TSInterfaceDeclaration(it) => it.span(),
            Self::TSEnumDeclaration(it) => it.span(),
            Self::TSModuleDeclaration(it) => it.span(),
            Self::TSImportEqualsDeclaration(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for Declaration<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::VariableDeclaration(it) => it.span_mut(),
            Self::FunctionDeclaration(it) => it.span_mut(),
            Self::ClassDeclaration(it) => it.span_mut(),
            Self::UsingDeclaration(it) => it.span_mut(),
            Self::TSTypeAliasDeclaration(it) => it.span_mut(),
            Self::TSInterfaceDeclaration(it) => it.span_mut(),
            Self::TSEnumDeclaration(it) => it.span_mut(),
            Self::TSModuleDeclaration(it) => it.span_mut(),
            Self::TSImportEqualsDeclaration(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for VariableDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for VariableDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for VariableDeclarator<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for VariableDeclarator<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for UsingDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for UsingDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for EmptyStatement {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for EmptyStatement {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ExpressionStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ExpressionStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for IfStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for IfStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for DoWhileStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for DoWhileStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for WhileStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for WhileStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ForStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ForStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ForStatementInit<'a> {
    fn span(&self) -> Span {
        match self {
            Self::VariableDeclaration(it) => it.span(),
            Self::UsingDeclaration(it) => it.span(),
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for ForStatementInit<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::VariableDeclaration(it) => it.span_mut(),
            Self::UsingDeclaration(it) => it.span_mut(),
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for ForInStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ForInStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ForStatementLeft<'a> {
    fn span(&self) -> Span {
        match self {
            Self::VariableDeclaration(it) => it.span(),
            Self::UsingDeclaration(it) => it.span(),
            Self::AssignmentTargetIdentifier(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
            Self::ArrayAssignmentTarget(it) => it.span(),
            Self::ObjectAssignmentTarget(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for ForStatementLeft<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::VariableDeclaration(it) => it.span_mut(),
            Self::UsingDeclaration(it) => it.span_mut(),
            Self::AssignmentTargetIdentifier(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
            Self::ArrayAssignmentTarget(it) => it.span_mut(),
            Self::ObjectAssignmentTarget(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for ForOfStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ForOfStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ContinueStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ContinueStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for BreakStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for BreakStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ReturnStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ReturnStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for WithStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for WithStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for SwitchStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for SwitchStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for SwitchCase<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for SwitchCase<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for LabeledStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for LabeledStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ThrowStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ThrowStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TryStatement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TryStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for CatchClause<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for CatchClause<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for CatchParameter<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for CatchParameter<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for DebuggerStatement {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for DebuggerStatement {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for BindingPattern<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.kind.span()
    }
}

impl<'a> GetSpanMut for BindingPattern<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        self.kind.span_mut()
    }
}

impl<'a> GetSpan for BindingPatternKind<'a> {
    fn span(&self) -> Span {
        match self {
            Self::BindingIdentifier(it) => it.span(),
            Self::ObjectPattern(it) => it.span(),
            Self::ArrayPattern(it) => it.span(),
            Self::AssignmentPattern(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for BindingPatternKind<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::BindingIdentifier(it) => it.span_mut(),
            Self::ObjectPattern(it) => it.span_mut(),
            Self::ArrayPattern(it) => it.span_mut(),
            Self::AssignmentPattern(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for AssignmentPattern<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for AssignmentPattern<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ObjectPattern<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ObjectPattern<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for BindingProperty<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for BindingProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ArrayPattern<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ArrayPattern<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for BindingRestElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for BindingRestElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for Function<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for Function<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for FormalParameters<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for FormalParameters<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for FormalParameter<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for FormalParameter<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for FunctionBody<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for FunctionBody<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ArrowFunctionExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ArrowFunctionExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for YieldExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for YieldExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for Class<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for Class<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ClassBody<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ClassBody<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ClassElement<'a> {
    fn span(&self) -> Span {
        match self {
            Self::StaticBlock(it) => it.span(),
            Self::MethodDefinition(it) => it.span(),
            Self::PropertyDefinition(it) => it.span(),
            Self::AccessorProperty(it) => it.span(),
            Self::TSIndexSignature(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for ClassElement<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::StaticBlock(it) => it.span_mut(),
            Self::MethodDefinition(it) => it.span_mut(),
            Self::PropertyDefinition(it) => it.span_mut(),
            Self::AccessorProperty(it) => it.span_mut(),
            Self::TSIndexSignature(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for MethodDefinition<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for MethodDefinition<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for PropertyDefinition<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for PropertyDefinition<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for PrivateIdentifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for PrivateIdentifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for StaticBlock<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for StaticBlock<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ModuleDeclaration<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ImportDeclaration(it) => it.span(),
            Self::ExportAllDeclaration(it) => it.span(),
            Self::ExportDefaultDeclaration(it) => it.span(),
            Self::ExportNamedDeclaration(it) => it.span(),
            Self::TSExportAssignment(it) => it.span(),
            Self::TSNamespaceExportDeclaration(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for ModuleDeclaration<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ImportDeclaration(it) => it.span_mut(),
            Self::ExportAllDeclaration(it) => it.span_mut(),
            Self::ExportDefaultDeclaration(it) => it.span_mut(),
            Self::ExportNamedDeclaration(it) => it.span_mut(),
            Self::TSExportAssignment(it) => it.span_mut(),
            Self::TSNamespaceExportDeclaration(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for AccessorProperty<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for AccessorProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ImportExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ImportExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ImportDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ImportDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ImportDeclarationSpecifier<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ImportSpecifier(it) => it.span(),
            Self::ImportDefaultSpecifier(it) => it.span(),
            Self::ImportNamespaceSpecifier(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for ImportDeclarationSpecifier<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ImportSpecifier(it) => it.span_mut(),
            Self::ImportDefaultSpecifier(it) => it.span_mut(),
            Self::ImportNamespaceSpecifier(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for ImportSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ImportSpecifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ImportDefaultSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ImportDefaultSpecifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ImportNamespaceSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ImportNamespaceSpecifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for WithClause<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for WithClause<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ImportAttribute<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ImportAttribute<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ImportAttributeKey<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for ImportAttributeKey<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for ExportNamedDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ExportNamedDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ExportDefaultDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ExportDefaultDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ExportAllDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ExportAllDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ExportSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for ExportSpecifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for ExportDefaultDeclarationKind<'a> {
    fn span(&self) -> Span {
        match self {
            Self::FunctionDeclaration(it) => it.span(),
            Self::ClassDeclaration(it) => it.span(),
            Self::TSInterfaceDeclaration(it) => it.span(),
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for ExportDefaultDeclarationKind<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::FunctionDeclaration(it) => it.span_mut(),
            Self::ClassDeclaration(it) => it.span_mut(),
            Self::TSInterfaceDeclaration(it) => it.span_mut(),
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for ModuleExportName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::IdentifierName(it) => it.span(),
            Self::IdentifierReference(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for ModuleExportName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::IdentifierName(it) => it.span_mut(),
            Self::IdentifierReference(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for TSThisParameter<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSThisParameter<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSEnumDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSEnumDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSEnumMember<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSEnumMember<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSEnumMemberName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::StaticIdentifier(it) => it.span(),
            Self::StaticStringLiteral(it) => it.span(),
            Self::StaticTemplateLiteral(it) => it.span(),
            Self::StaticNumericLiteral(it) => it.span(),
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for TSEnumMemberName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::StaticIdentifier(it) => it.span_mut(),
            Self::StaticStringLiteral(it) => it.span_mut(),
            Self::StaticTemplateLiteral(it) => it.span_mut(),
            Self::StaticNumericLiteral(it) => it.span_mut(),
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for TSTypeAnnotation<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSTypeAnnotation<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSLiteralType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSLiteralType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSLiteral<'a> {
    fn span(&self) -> Span {
        match self {
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for TSLiteral<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for TSType<'a> {
    fn span(&self) -> Span {
        match self {
            Self::TSAnyKeyword(it) => it.span(),
            Self::TSBigIntKeyword(it) => it.span(),
            Self::TSBooleanKeyword(it) => it.span(),
            Self::TSIntrinsicKeyword(it) => it.span(),
            Self::TSNeverKeyword(it) => it.span(),
            Self::TSNullKeyword(it) => it.span(),
            Self::TSNumberKeyword(it) => it.span(),
            Self::TSObjectKeyword(it) => it.span(),
            Self::TSStringKeyword(it) => it.span(),
            Self::TSSymbolKeyword(it) => it.span(),
            Self::TSUndefinedKeyword(it) => it.span(),
            Self::TSUnknownKeyword(it) => it.span(),
            Self::TSVoidKeyword(it) => it.span(),
            Self::TSArrayType(it) => it.span(),
            Self::TSConditionalType(it) => it.span(),
            Self::TSConstructorType(it) => it.span(),
            Self::TSFunctionType(it) => it.span(),
            Self::TSImportType(it) => it.span(),
            Self::TSIndexedAccessType(it) => it.span(),
            Self::TSInferType(it) => it.span(),
            Self::TSIntersectionType(it) => it.span(),
            Self::TSLiteralType(it) => it.span(),
            Self::TSMappedType(it) => it.span(),
            Self::TSNamedTupleMember(it) => it.span(),
            Self::TSQualifiedName(it) => it.span(),
            Self::TSTemplateLiteralType(it) => it.span(),
            Self::TSThisType(it) => it.span(),
            Self::TSTupleType(it) => it.span(),
            Self::TSTypeLiteral(it) => it.span(),
            Self::TSTypeOperatorType(it) => it.span(),
            Self::TSTypePredicate(it) => it.span(),
            Self::TSTypeQuery(it) => it.span(),
            Self::TSTypeReference(it) => it.span(),
            Self::TSUnionType(it) => it.span(),
            Self::TSParenthesizedType(it) => it.span(),
            Self::JSDocNullableType(it) => it.span(),
            Self::JSDocNonNullableType(it) => it.span(),
            Self::JSDocUnknownType(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for TSType<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSAnyKeyword(it) => it.span_mut(),
            Self::TSBigIntKeyword(it) => it.span_mut(),
            Self::TSBooleanKeyword(it) => it.span_mut(),
            Self::TSIntrinsicKeyword(it) => it.span_mut(),
            Self::TSNeverKeyword(it) => it.span_mut(),
            Self::TSNullKeyword(it) => it.span_mut(),
            Self::TSNumberKeyword(it) => it.span_mut(),
            Self::TSObjectKeyword(it) => it.span_mut(),
            Self::TSStringKeyword(it) => it.span_mut(),
            Self::TSSymbolKeyword(it) => it.span_mut(),
            Self::TSUndefinedKeyword(it) => it.span_mut(),
            Self::TSUnknownKeyword(it) => it.span_mut(),
            Self::TSVoidKeyword(it) => it.span_mut(),
            Self::TSArrayType(it) => it.span_mut(),
            Self::TSConditionalType(it) => it.span_mut(),
            Self::TSConstructorType(it) => it.span_mut(),
            Self::TSFunctionType(it) => it.span_mut(),
            Self::TSImportType(it) => it.span_mut(),
            Self::TSIndexedAccessType(it) => it.span_mut(),
            Self::TSInferType(it) => it.span_mut(),
            Self::TSIntersectionType(it) => it.span_mut(),
            Self::TSLiteralType(it) => it.span_mut(),
            Self::TSMappedType(it) => it.span_mut(),
            Self::TSNamedTupleMember(it) => it.span_mut(),
            Self::TSQualifiedName(it) => it.span_mut(),
            Self::TSTemplateLiteralType(it) => it.span_mut(),
            Self::TSThisType(it) => it.span_mut(),
            Self::TSTupleType(it) => it.span_mut(),
            Self::TSTypeLiteral(it) => it.span_mut(),
            Self::TSTypeOperatorType(it) => it.span_mut(),
            Self::TSTypePredicate(it) => it.span_mut(),
            Self::TSTypeQuery(it) => it.span_mut(),
            Self::TSTypeReference(it) => it.span_mut(),
            Self::TSUnionType(it) => it.span_mut(),
            Self::TSParenthesizedType(it) => it.span_mut(),
            Self::JSDocNullableType(it) => it.span_mut(),
            Self::JSDocNonNullableType(it) => it.span_mut(),
            Self::JSDocUnknownType(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for TSConditionalType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSConditionalType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSUnionType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSUnionType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSIntersectionType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSIntersectionType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSParenthesizedType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSParenthesizedType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTypeOperator<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSTypeOperator<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSArrayType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSArrayType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSIndexedAccessType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSIndexedAccessType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTupleType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSTupleType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSNamedTupleMember<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSNamedTupleMember<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSOptionalType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSOptionalType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSRestType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSRestType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTupleElement<'a> {
    fn span(&self) -> Span {
        match self {
            Self::TSOptionalType(it) => it.span(),
            Self::TSRestType(it) => it.span(),
            Self::TSAnyKeyword(it) => it.span(),
            Self::TSBigIntKeyword(it) => it.span(),
            Self::TSBooleanKeyword(it) => it.span(),
            Self::TSIntrinsicKeyword(it) => it.span(),
            Self::TSNeverKeyword(it) => it.span(),
            Self::TSNullKeyword(it) => it.span(),
            Self::TSNumberKeyword(it) => it.span(),
            Self::TSObjectKeyword(it) => it.span(),
            Self::TSStringKeyword(it) => it.span(),
            Self::TSSymbolKeyword(it) => it.span(),
            Self::TSUndefinedKeyword(it) => it.span(),
            Self::TSUnknownKeyword(it) => it.span(),
            Self::TSVoidKeyword(it) => it.span(),
            Self::TSArrayType(it) => it.span(),
            Self::TSConditionalType(it) => it.span(),
            Self::TSConstructorType(it) => it.span(),
            Self::TSFunctionType(it) => it.span(),
            Self::TSImportType(it) => it.span(),
            Self::TSIndexedAccessType(it) => it.span(),
            Self::TSInferType(it) => it.span(),
            Self::TSIntersectionType(it) => it.span(),
            Self::TSLiteralType(it) => it.span(),
            Self::TSMappedType(it) => it.span(),
            Self::TSNamedTupleMember(it) => it.span(),
            Self::TSQualifiedName(it) => it.span(),
            Self::TSTemplateLiteralType(it) => it.span(),
            Self::TSThisType(it) => it.span(),
            Self::TSTupleType(it) => it.span(),
            Self::TSTypeLiteral(it) => it.span(),
            Self::TSTypeOperatorType(it) => it.span(),
            Self::TSTypePredicate(it) => it.span(),
            Self::TSTypeQuery(it) => it.span(),
            Self::TSTypeReference(it) => it.span(),
            Self::TSUnionType(it) => it.span(),
            Self::TSParenthesizedType(it) => it.span(),
            Self::JSDocNullableType(it) => it.span(),
            Self::JSDocNonNullableType(it) => it.span(),
            Self::JSDocUnknownType(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for TSTupleElement<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSOptionalType(it) => it.span_mut(),
            Self::TSRestType(it) => it.span_mut(),
            Self::TSAnyKeyword(it) => it.span_mut(),
            Self::TSBigIntKeyword(it) => it.span_mut(),
            Self::TSBooleanKeyword(it) => it.span_mut(),
            Self::TSIntrinsicKeyword(it) => it.span_mut(),
            Self::TSNeverKeyword(it) => it.span_mut(),
            Self::TSNullKeyword(it) => it.span_mut(),
            Self::TSNumberKeyword(it) => it.span_mut(),
            Self::TSObjectKeyword(it) => it.span_mut(),
            Self::TSStringKeyword(it) => it.span_mut(),
            Self::TSSymbolKeyword(it) => it.span_mut(),
            Self::TSUndefinedKeyword(it) => it.span_mut(),
            Self::TSUnknownKeyword(it) => it.span_mut(),
            Self::TSVoidKeyword(it) => it.span_mut(),
            Self::TSArrayType(it) => it.span_mut(),
            Self::TSConditionalType(it) => it.span_mut(),
            Self::TSConstructorType(it) => it.span_mut(),
            Self::TSFunctionType(it) => it.span_mut(),
            Self::TSImportType(it) => it.span_mut(),
            Self::TSIndexedAccessType(it) => it.span_mut(),
            Self::TSInferType(it) => it.span_mut(),
            Self::TSIntersectionType(it) => it.span_mut(),
            Self::TSLiteralType(it) => it.span_mut(),
            Self::TSMappedType(it) => it.span_mut(),
            Self::TSNamedTupleMember(it) => it.span_mut(),
            Self::TSQualifiedName(it) => it.span_mut(),
            Self::TSTemplateLiteralType(it) => it.span_mut(),
            Self::TSThisType(it) => it.span_mut(),
            Self::TSTupleType(it) => it.span_mut(),
            Self::TSTypeLiteral(it) => it.span_mut(),
            Self::TSTypeOperatorType(it) => it.span_mut(),
            Self::TSTypePredicate(it) => it.span_mut(),
            Self::TSTypeQuery(it) => it.span_mut(),
            Self::TSTypeReference(it) => it.span_mut(),
            Self::TSUnionType(it) => it.span_mut(),
            Self::TSParenthesizedType(it) => it.span_mut(),
            Self::JSDocNullableType(it) => it.span_mut(),
            Self::JSDocNonNullableType(it) => it.span_mut(),
            Self::JSDocUnknownType(it) => it.span_mut(),
        }
    }
}

impl GetSpan for TSAnyKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for TSAnyKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for TSStringKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for TSStringKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for TSBooleanKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for TSBooleanKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for TSNumberKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for TSNumberKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for TSNeverKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for TSNeverKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for TSIntrinsicKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for TSIntrinsicKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for TSUnknownKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for TSUnknownKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for TSNullKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for TSNullKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for TSUndefinedKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for TSUndefinedKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for TSVoidKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for TSVoidKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for TSSymbolKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for TSSymbolKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for TSThisType {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for TSThisType {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for TSObjectKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for TSObjectKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for TSBigIntKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for TSBigIntKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTypeReference<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSTypeReference<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTypeName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::IdentifierReference(it) => it.span(),
            Self::QualifiedName(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for TSTypeName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::IdentifierReference(it) => it.span_mut(),
            Self::QualifiedName(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for TSQualifiedName<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSQualifiedName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTypeParameterInstantiation<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSTypeParameterInstantiation<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTypeParameter<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSTypeParameter<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTypeParameterDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSTypeParameterDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTypeAliasDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSTypeAliasDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSClassImplements<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSClassImplements<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSInterfaceDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSInterfaceDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSInterfaceBody<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSInterfaceBody<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSPropertySignature<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSPropertySignature<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSSignature<'a> {
    fn span(&self) -> Span {
        match self {
            Self::TSIndexSignature(it) => it.span(),
            Self::TSPropertySignature(it) => it.span(),
            Self::TSCallSignatureDeclaration(it) => it.span(),
            Self::TSConstructSignatureDeclaration(it) => it.span(),
            Self::TSMethodSignature(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for TSSignature<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSIndexSignature(it) => it.span_mut(),
            Self::TSPropertySignature(it) => it.span_mut(),
            Self::TSCallSignatureDeclaration(it) => it.span_mut(),
            Self::TSConstructSignatureDeclaration(it) => it.span_mut(),
            Self::TSMethodSignature(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for TSIndexSignature<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSIndexSignature<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSCallSignatureDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSCallSignatureDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSMethodSignature<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSMethodSignature<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSConstructSignatureDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSConstructSignatureDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSIndexSignatureName<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSIndexSignatureName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSInterfaceHeritage<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSInterfaceHeritage<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTypePredicate<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSTypePredicate<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTypePredicateName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => it.span(),
            Self::This(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for TSTypePredicateName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::This(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for TSModuleDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSModuleDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSModuleDeclarationName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for TSModuleDeclarationName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for TSModuleDeclarationBody<'a> {
    fn span(&self) -> Span {
        match self {
            Self::TSModuleDeclaration(it) => it.span(),
            Self::TSModuleBlock(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for TSModuleDeclarationBody<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSModuleDeclaration(it) => it.span_mut(),
            Self::TSModuleBlock(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for TSModuleBlock<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSModuleBlock<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTypeLiteral<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSTypeLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSInferType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSInferType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTypeQuery<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSTypeQuery<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTypeQueryExprName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::TSImportType(it) => it.span(),
            Self::IdentifierReference(it) => it.span(),
            Self::QualifiedName(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for TSTypeQueryExprName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSImportType(it) => it.span_mut(),
            Self::IdentifierReference(it) => it.span_mut(),
            Self::QualifiedName(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for TSImportType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSImportType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSImportAttributes<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSImportAttributes<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSImportAttribute<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSImportAttribute<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSImportAttributeName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for TSImportAttributeName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for TSFunctionType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSFunctionType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSConstructorType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSConstructorType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSMappedType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSMappedType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTemplateLiteralType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSTemplateLiteralType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSAsExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSAsExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSSatisfiesExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSSatisfiesExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSTypeAssertion<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSTypeAssertion<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSImportEqualsDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSImportEqualsDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSModuleReference<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ExternalModuleReference(it) => it.span(),
            Self::IdentifierReference(it) => it.span(),
            Self::QualifiedName(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for TSModuleReference<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ExternalModuleReference(it) => it.span_mut(),
            Self::IdentifierReference(it) => it.span_mut(),
            Self::QualifiedName(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for TSExternalModuleReference<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSExternalModuleReference<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSNonNullExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSNonNullExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for Decorator<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for Decorator<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSExportAssignment<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSExportAssignment<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSNamespaceExportDeclaration<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSNamespaceExportDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for TSInstantiationExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for TSInstantiationExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSDocNullableType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for JSDocNullableType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSDocNonNullableType<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for JSDocNonNullableType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpan for JSDocUnknownType {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for JSDocUnknownType {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSXElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for JSXElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSXOpeningElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for JSXOpeningElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSXClosingElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for JSXClosingElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSXFragment<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for JSXFragment<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSXElementName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => it.span(),
            Self::NamespacedName(it) => it.span(),
            Self::MemberExpression(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for JSXElementName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::NamespacedName(it) => it.span_mut(),
            Self::MemberExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for JSXNamespacedName<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for JSXNamespacedName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSXMemberExpression<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for JSXMemberExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSXMemberExpressionObject<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => it.span(),
            Self::MemberExpression(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for JSXMemberExpressionObject<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::MemberExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for JSXExpressionContainer<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for JSXExpressionContainer<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSXExpression<'a> {
    fn span(&self) -> Span {
        match self {
            Self::EmptyExpression(it) => it.span(),
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for JSXExpression<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::EmptyExpression(it) => it.span_mut(),
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpan for JSXEmptyExpression {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpanMut for JSXEmptyExpression {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSXAttributeItem<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Attribute(it) => it.span(),
            Self::SpreadAttribute(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for JSXAttributeItem<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Attribute(it) => it.span_mut(),
            Self::SpreadAttribute(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for JSXAttribute<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for JSXAttribute<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSXSpreadAttribute<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for JSXSpreadAttribute<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSXAttributeName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => it.span(),
            Self::NamespacedName(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for JSXAttributeName<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::NamespacedName(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for JSXAttributeValue<'a> {
    fn span(&self) -> Span {
        match self {
            Self::StringLiteral(it) => it.span(),
            Self::ExpressionContainer(it) => it.span(),
            Self::Element(it) => it.span(),
            Self::Fragment(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for JSXAttributeValue<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::StringLiteral(it) => it.span_mut(),
            Self::ExpressionContainer(it) => it.span_mut(),
            Self::Element(it) => it.span_mut(),
            Self::Fragment(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for JSXIdentifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for JSXIdentifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSXChild<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Text(it) => it.span(),
            Self::Element(it) => it.span(),
            Self::Fragment(it) => it.span(),
            Self::ExpressionContainer(it) => it.span(),
            Self::Spread(it) => it.span(),
        }
    }
}

impl<'a> GetSpanMut for JSXChild<'a> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Text(it) => it.span_mut(),
            Self::Element(it) => it.span_mut(),
            Self::Fragment(it) => it.span_mut(),
            Self::ExpressionContainer(it) => it.span_mut(),
            Self::Spread(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpan for JSXSpreadChild<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for JSXSpreadChild<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpan for JSXText<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> GetSpanMut for JSXText<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}
