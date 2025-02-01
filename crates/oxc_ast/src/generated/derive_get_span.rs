// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/get_span.rs`

#![allow(clippy::match_same_arms)]

use oxc_span::{GetSpan, Span};

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
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

impl GetSpan for NumericLiteral<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for StringLiteral<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BigIntLiteral<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for RegExpLiteral<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Program<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Expression<'_> {
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

impl GetSpan for IdentifierName<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for IdentifierReference<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BindingIdentifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for LabelIdentifier<'_> {
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

impl GetSpan for ArrayExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ArrayExpressionElement<'_> {
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

impl GetSpan for ObjectExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ObjectPropertyKind<'_> {
    fn span(&self) -> Span {
        match self {
            Self::ObjectProperty(it) => GetSpan::span(it.as_ref()),
            Self::SpreadProperty(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for ObjectProperty<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for PropertyKey<'_> {
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

impl GetSpan for TemplateLiteral<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TaggedTemplateExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TemplateElement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for MemberExpression<'_> {
    fn span(&self) -> Span {
        match self {
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for ComputedMemberExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for StaticMemberExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for PrivateFieldExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for CallExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for NewExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for MetaProperty<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for SpreadElement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Argument<'_> {
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

impl GetSpan for UpdateExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for UnaryExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BinaryExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for PrivateInExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for LogicalExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ConditionalExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for AssignmentExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for AssignmentTarget<'_> {
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

impl GetSpan for SimpleAssignmentTarget<'_> {
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

impl GetSpan for AssignmentTargetPattern<'_> {
    fn span(&self) -> Span {
        match self {
            Self::ArrayAssignmentTarget(it) => GetSpan::span(it.as_ref()),
            Self::ObjectAssignmentTarget(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for ArrayAssignmentTarget<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ObjectAssignmentTarget<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for AssignmentTargetRest<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for AssignmentTargetMaybeDefault<'_> {
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

impl GetSpan for AssignmentTargetWithDefault<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for AssignmentTargetProperty<'_> {
    fn span(&self) -> Span {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => GetSpan::span(it.as_ref()),
            Self::AssignmentTargetPropertyProperty(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for AssignmentTargetPropertyIdentifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for AssignmentTargetPropertyProperty<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for SequenceExpression<'_> {
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

impl GetSpan for AwaitExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ChainExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ChainElement<'_> {
    fn span(&self) -> Span {
        match self {
            Self::CallExpression(it) => GetSpan::span(it.as_ref()),
            Self::TSNonNullExpression(it) => GetSpan::span(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::StaticMemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for ParenthesizedExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Statement<'_> {
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

impl GetSpan for Directive<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Hashbang<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BlockStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Declaration<'_> {
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

impl GetSpan for VariableDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for VariableDeclarator<'_> {
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

impl GetSpan for ExpressionStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for IfStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for DoWhileStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for WhileStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ForStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ForStatementInit<'_> {
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

impl GetSpan for ForInStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ForStatementLeft<'_> {
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

impl GetSpan for ForOfStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ContinueStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BreakStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ReturnStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for WithStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for SwitchStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for SwitchCase<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for LabeledStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ThrowStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TryStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for CatchClause<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for CatchParameter<'_> {
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

impl GetSpan for BindingPattern<'_> {
    #[inline]
    fn span(&self) -> Span {
        GetSpan::span(&self.kind)
    }
}

impl GetSpan for BindingPatternKind<'_> {
    fn span(&self) -> Span {
        match self {
            Self::BindingIdentifier(it) => GetSpan::span(it.as_ref()),
            Self::ObjectPattern(it) => GetSpan::span(it.as_ref()),
            Self::ArrayPattern(it) => GetSpan::span(it.as_ref()),
            Self::AssignmentPattern(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for AssignmentPattern<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ObjectPattern<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BindingProperty<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ArrayPattern<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BindingRestElement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Function<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for FormalParameters<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for FormalParameter<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for FunctionBody<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ArrowFunctionExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for YieldExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Class<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ClassBody<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ClassElement<'_> {
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

impl GetSpan for MethodDefinition<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for PropertyDefinition<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for PrivateIdentifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for StaticBlock<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ModuleDeclaration<'_> {
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

impl GetSpan for AccessorProperty<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ImportExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ImportDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ImportDeclarationSpecifier<'_> {
    fn span(&self) -> Span {
        match self {
            Self::ImportSpecifier(it) => GetSpan::span(it.as_ref()),
            Self::ImportDefaultSpecifier(it) => GetSpan::span(it.as_ref()),
            Self::ImportNamespaceSpecifier(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for ImportSpecifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ImportDefaultSpecifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ImportNamespaceSpecifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for WithClause<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ImportAttribute<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ImportAttributeKey<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => GetSpan::span(it),
            Self::StringLiteral(it) => GetSpan::span(it),
        }
    }
}

impl GetSpan for ExportNamedDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ExportDefaultDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ExportAllDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ExportSpecifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ExportDefaultDeclarationKind<'_> {
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

impl GetSpan for ModuleExportName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::IdentifierName(it) => GetSpan::span(it),
            Self::IdentifierReference(it) => GetSpan::span(it),
            Self::StringLiteral(it) => GetSpan::span(it),
        }
    }
}

impl GetSpan for TSThisParameter<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSEnumDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSEnumMember<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSEnumMemberName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => GetSpan::span(it.as_ref()),
            Self::String(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for TSTypeAnnotation<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSLiteralType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSLiteral<'_> {
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

impl GetSpan for TSType<'_> {
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

impl GetSpan for TSConditionalType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSUnionType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSIntersectionType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSParenthesizedType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeOperator<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSArrayType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSIndexedAccessType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTupleType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSNamedTupleMember<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSOptionalType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSRestType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTupleElement<'_> {
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

impl GetSpan for TSTypeReference<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::IdentifierReference(it) => GetSpan::span(it.as_ref()),
            Self::QualifiedName(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for TSQualifiedName<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeParameterInstantiation<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeParameter<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeParameterDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeAliasDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSClassImplements<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSInterfaceDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSInterfaceBody<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSPropertySignature<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSSignature<'_> {
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

impl GetSpan for TSIndexSignature<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSCallSignatureDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSMethodSignature<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSConstructSignatureDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSIndexSignatureName<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSInterfaceHeritage<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypePredicate<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypePredicateName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => GetSpan::span(it.as_ref()),
            Self::This(it) => GetSpan::span(it),
        }
    }
}

impl GetSpan for TSModuleDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSModuleDeclarationName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => GetSpan::span(it),
            Self::StringLiteral(it) => GetSpan::span(it),
        }
    }
}

impl GetSpan for TSModuleDeclarationBody<'_> {
    fn span(&self) -> Span {
        match self {
            Self::TSModuleDeclaration(it) => GetSpan::span(it.as_ref()),
            Self::TSModuleBlock(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for TSModuleBlock<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeLiteral<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSInferType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeQuery<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeQueryExprName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::TSImportType(it) => GetSpan::span(it.as_ref()),
            Self::IdentifierReference(it) => GetSpan::span(it.as_ref()),
            Self::QualifiedName(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for TSImportType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSImportAttributes<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSImportAttribute<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSImportAttributeName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => GetSpan::span(it),
            Self::StringLiteral(it) => GetSpan::span(it),
        }
    }
}

impl GetSpan for TSFunctionType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSConstructorType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSMappedType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTemplateLiteralType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSAsExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSSatisfiesExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeAssertion<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSImportEqualsDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSModuleReference<'_> {
    fn span(&self) -> Span {
        match self {
            Self::ExternalModuleReference(it) => GetSpan::span(it.as_ref()),
            Self::IdentifierReference(it) => GetSpan::span(it.as_ref()),
            Self::QualifiedName(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for TSExternalModuleReference<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSNonNullExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Decorator<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSExportAssignment<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSNamespaceExportDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSInstantiationExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSDocNullableType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSDocNonNullableType<'_> {
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

impl GetSpan for JSXElement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXOpeningElement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXClosingElement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXFragment<'_> {
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

impl GetSpan for JSXElementName<'_> {
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

impl GetSpan for JSXNamespacedName<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXMemberExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXMemberExpressionObject<'_> {
    fn span(&self) -> Span {
        match self {
            Self::IdentifierReference(it) => GetSpan::span(it.as_ref()),
            Self::MemberExpression(it) => GetSpan::span(it.as_ref()),
            Self::ThisExpression(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for JSXExpressionContainer<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXExpression<'_> {
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

impl GetSpan for JSXAttributeItem<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Attribute(it) => GetSpan::span(it.as_ref()),
            Self::SpreadAttribute(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for JSXAttribute<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXSpreadAttribute<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXAttributeName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => GetSpan::span(it.as_ref()),
            Self::NamespacedName(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for JSXAttributeValue<'_> {
    fn span(&self) -> Span {
        match self {
            Self::StringLiteral(it) => GetSpan::span(it.as_ref()),
            Self::ExpressionContainer(it) => GetSpan::span(it.as_ref()),
            Self::Element(it) => GetSpan::span(it.as_ref()),
            Self::Fragment(it) => GetSpan::span(it.as_ref()),
        }
    }
}

impl GetSpan for JSXIdentifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXChild<'_> {
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

impl GetSpan for JSXSpreadChild<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXText<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
