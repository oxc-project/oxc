// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/content_hash.rs`

#![allow(clippy::match_same_arms)]

use std::{hash::Hasher, mem::discriminant};

use oxc_span::hash::ContentHash;

#[allow(clippy::wildcard_imports)]
use crate::ast::js::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::jsx::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::literal::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::ts::*;

impl ContentHash for BooleanLiteral {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.value.content_hash(state);
    }
}

impl<'a> ContentHash for BigIntLiteral<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.raw.content_hash(state);
        self.base.content_hash(state);
    }
}

impl<'a> ContentHash for RegExpLiteral<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.value.content_hash(state);
        self.regex.content_hash(state);
    }
}

impl<'a> ContentHash for RegExp<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.pattern.content_hash(state);
        self.flags.content_hash(state);
    }
}

impl<'a> ContentHash for RegExpPattern<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Raw(it) => it.content_hash(state),
            Self::Invalid(it) => it.content_hash(state),
            Self::Pattern(it) => it.content_hash(state),
        }
    }
}

impl ContentHash for EmptyObject {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for StringLiteral<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.value.content_hash(state);
    }
}

impl<'a> ContentHash for Program<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.source_type.content_hash(state);
        self.hashbang.content_hash(state);
        self.directives.content_hash(state);
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for Expression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::BooleanLiteral(it) => it.content_hash(state),
            Self::NullLiteral(it) => it.content_hash(state),
            Self::NumericLiteral(it) => it.content_hash(state),
            Self::BigIntLiteral(it) => it.content_hash(state),
            Self::RegExpLiteral(it) => it.content_hash(state),
            Self::StringLiteral(it) => it.content_hash(state),
            Self::TemplateLiteral(it) => it.content_hash(state),
            Self::Identifier(it) => it.content_hash(state),
            Self::MetaProperty(it) => it.content_hash(state),
            Self::Super(it) => it.content_hash(state),
            Self::ArrayExpression(it) => it.content_hash(state),
            Self::ArrowFunctionExpression(it) => it.content_hash(state),
            Self::AssignmentExpression(it) => it.content_hash(state),
            Self::AwaitExpression(it) => it.content_hash(state),
            Self::BinaryExpression(it) => it.content_hash(state),
            Self::CallExpression(it) => it.content_hash(state),
            Self::ChainExpression(it) => it.content_hash(state),
            Self::ClassExpression(it) => it.content_hash(state),
            Self::ConditionalExpression(it) => it.content_hash(state),
            Self::FunctionExpression(it) => it.content_hash(state),
            Self::ImportExpression(it) => it.content_hash(state),
            Self::LogicalExpression(it) => it.content_hash(state),
            Self::NewExpression(it) => it.content_hash(state),
            Self::ObjectExpression(it) => it.content_hash(state),
            Self::ParenthesizedExpression(it) => it.content_hash(state),
            Self::SequenceExpression(it) => it.content_hash(state),
            Self::TaggedTemplateExpression(it) => it.content_hash(state),
            Self::ThisExpression(it) => it.content_hash(state),
            Self::UnaryExpression(it) => it.content_hash(state),
            Self::UpdateExpression(it) => it.content_hash(state),
            Self::YieldExpression(it) => it.content_hash(state),
            Self::PrivateInExpression(it) => it.content_hash(state),
            Self::JSXElement(it) => it.content_hash(state),
            Self::JSXFragment(it) => it.content_hash(state),
            Self::TSAsExpression(it) => it.content_hash(state),
            Self::TSSatisfiesExpression(it) => it.content_hash(state),
            Self::TSTypeAssertion(it) => it.content_hash(state),
            Self::TSNonNullExpression(it) => it.content_hash(state),
            Self::TSInstantiationExpression(it) => it.content_hash(state),
            Self::ComputedMemberExpression(it) => it.content_hash(state),
            Self::StaticMemberExpression(it) => it.content_hash(state),
            Self::PrivateFieldExpression(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for IdentifierName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.name.content_hash(state);
    }
}

impl<'a> ContentHash for IdentifierReference<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.name.content_hash(state);
    }
}

impl<'a> ContentHash for BindingIdentifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.name.content_hash(state);
    }
}

impl<'a> ContentHash for LabelIdentifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.name.content_hash(state);
    }
}

impl ContentHash for ThisExpression {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for ArrayExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.elements.content_hash(state);
    }
}

impl<'a> ContentHash for ArrayExpressionElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::SpreadElement(it) => it.content_hash(state),
            Self::Elision(it) => it.content_hash(state),
            Self::BooleanLiteral(it) => it.content_hash(state),
            Self::NullLiteral(it) => it.content_hash(state),
            Self::NumericLiteral(it) => it.content_hash(state),
            Self::BigIntLiteral(it) => it.content_hash(state),
            Self::RegExpLiteral(it) => it.content_hash(state),
            Self::StringLiteral(it) => it.content_hash(state),
            Self::TemplateLiteral(it) => it.content_hash(state),
            Self::Identifier(it) => it.content_hash(state),
            Self::MetaProperty(it) => it.content_hash(state),
            Self::Super(it) => it.content_hash(state),
            Self::ArrayExpression(it) => it.content_hash(state),
            Self::ArrowFunctionExpression(it) => it.content_hash(state),
            Self::AssignmentExpression(it) => it.content_hash(state),
            Self::AwaitExpression(it) => it.content_hash(state),
            Self::BinaryExpression(it) => it.content_hash(state),
            Self::CallExpression(it) => it.content_hash(state),
            Self::ChainExpression(it) => it.content_hash(state),
            Self::ClassExpression(it) => it.content_hash(state),
            Self::ConditionalExpression(it) => it.content_hash(state),
            Self::FunctionExpression(it) => it.content_hash(state),
            Self::ImportExpression(it) => it.content_hash(state),
            Self::LogicalExpression(it) => it.content_hash(state),
            Self::NewExpression(it) => it.content_hash(state),
            Self::ObjectExpression(it) => it.content_hash(state),
            Self::ParenthesizedExpression(it) => it.content_hash(state),
            Self::SequenceExpression(it) => it.content_hash(state),
            Self::TaggedTemplateExpression(it) => it.content_hash(state),
            Self::ThisExpression(it) => it.content_hash(state),
            Self::UnaryExpression(it) => it.content_hash(state),
            Self::UpdateExpression(it) => it.content_hash(state),
            Self::YieldExpression(it) => it.content_hash(state),
            Self::PrivateInExpression(it) => it.content_hash(state),
            Self::JSXElement(it) => it.content_hash(state),
            Self::JSXFragment(it) => it.content_hash(state),
            Self::TSAsExpression(it) => it.content_hash(state),
            Self::TSSatisfiesExpression(it) => it.content_hash(state),
            Self::TSTypeAssertion(it) => it.content_hash(state),
            Self::TSNonNullExpression(it) => it.content_hash(state),
            Self::TSInstantiationExpression(it) => it.content_hash(state),
            Self::ComputedMemberExpression(it) => it.content_hash(state),
            Self::StaticMemberExpression(it) => it.content_hash(state),
            Self::PrivateFieldExpression(it) => it.content_hash(state),
        }
    }
}

impl ContentHash for Elision {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for ObjectExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.properties.content_hash(state);
    }
}

impl<'a> ContentHash for ObjectPropertyKind<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ObjectProperty(it) => it.content_hash(state),
            Self::SpreadProperty(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for ObjectProperty<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.kind.content_hash(state);
        self.key.content_hash(state);
        self.value.content_hash(state);
        self.init.content_hash(state);
        self.method.content_hash(state);
        self.shorthand.content_hash(state);
        self.computed.content_hash(state);
    }
}

impl<'a> ContentHash for PropertyKey<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::StaticIdentifier(it) => it.content_hash(state),
            Self::PrivateIdentifier(it) => it.content_hash(state),
            Self::BooleanLiteral(it) => it.content_hash(state),
            Self::NullLiteral(it) => it.content_hash(state),
            Self::NumericLiteral(it) => it.content_hash(state),
            Self::BigIntLiteral(it) => it.content_hash(state),
            Self::RegExpLiteral(it) => it.content_hash(state),
            Self::StringLiteral(it) => it.content_hash(state),
            Self::TemplateLiteral(it) => it.content_hash(state),
            Self::Identifier(it) => it.content_hash(state),
            Self::MetaProperty(it) => it.content_hash(state),
            Self::Super(it) => it.content_hash(state),
            Self::ArrayExpression(it) => it.content_hash(state),
            Self::ArrowFunctionExpression(it) => it.content_hash(state),
            Self::AssignmentExpression(it) => it.content_hash(state),
            Self::AwaitExpression(it) => it.content_hash(state),
            Self::BinaryExpression(it) => it.content_hash(state),
            Self::CallExpression(it) => it.content_hash(state),
            Self::ChainExpression(it) => it.content_hash(state),
            Self::ClassExpression(it) => it.content_hash(state),
            Self::ConditionalExpression(it) => it.content_hash(state),
            Self::FunctionExpression(it) => it.content_hash(state),
            Self::ImportExpression(it) => it.content_hash(state),
            Self::LogicalExpression(it) => it.content_hash(state),
            Self::NewExpression(it) => it.content_hash(state),
            Self::ObjectExpression(it) => it.content_hash(state),
            Self::ParenthesizedExpression(it) => it.content_hash(state),
            Self::SequenceExpression(it) => it.content_hash(state),
            Self::TaggedTemplateExpression(it) => it.content_hash(state),
            Self::ThisExpression(it) => it.content_hash(state),
            Self::UnaryExpression(it) => it.content_hash(state),
            Self::UpdateExpression(it) => it.content_hash(state),
            Self::YieldExpression(it) => it.content_hash(state),
            Self::PrivateInExpression(it) => it.content_hash(state),
            Self::JSXElement(it) => it.content_hash(state),
            Self::JSXFragment(it) => it.content_hash(state),
            Self::TSAsExpression(it) => it.content_hash(state),
            Self::TSSatisfiesExpression(it) => it.content_hash(state),
            Self::TSTypeAssertion(it) => it.content_hash(state),
            Self::TSNonNullExpression(it) => it.content_hash(state),
            Self::TSInstantiationExpression(it) => it.content_hash(state),
            Self::ComputedMemberExpression(it) => it.content_hash(state),
            Self::StaticMemberExpression(it) => it.content_hash(state),
            Self::PrivateFieldExpression(it) => it.content_hash(state),
        }
    }
}

impl ContentHash for PropertyKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for TemplateLiteral<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.quasis.content_hash(state);
        self.expressions.content_hash(state);
    }
}

impl<'a> ContentHash for TaggedTemplateExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.tag.content_hash(state);
        self.quasi.content_hash(state);
        self.type_parameters.content_hash(state);
    }
}

impl<'a> ContentHash for TemplateElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.tail.content_hash(state);
        self.value.content_hash(state);
    }
}

impl<'a> ContentHash for TemplateElementValue<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.raw.content_hash(state);
        self.cooked.content_hash(state);
    }
}

impl<'a> ContentHash for MemberExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ComputedMemberExpression(it) => it.content_hash(state),
            Self::StaticMemberExpression(it) => it.content_hash(state),
            Self::PrivateFieldExpression(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for ComputedMemberExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.object.content_hash(state);
        self.expression.content_hash(state);
        self.optional.content_hash(state);
    }
}

impl<'a> ContentHash for StaticMemberExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.object.content_hash(state);
        self.property.content_hash(state);
        self.optional.content_hash(state);
    }
}

impl<'a> ContentHash for PrivateFieldExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.object.content_hash(state);
        self.field.content_hash(state);
        self.optional.content_hash(state);
    }
}

impl<'a> ContentHash for CallExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.callee.content_hash(state);
        self.type_parameters.content_hash(state);
        self.arguments.content_hash(state);
        self.optional.content_hash(state);
    }
}

impl<'a> ContentHash for NewExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.callee.content_hash(state);
        self.arguments.content_hash(state);
        self.type_parameters.content_hash(state);
    }
}

impl<'a> ContentHash for MetaProperty<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.meta.content_hash(state);
        self.property.content_hash(state);
    }
}

impl<'a> ContentHash for SpreadElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.argument.content_hash(state);
    }
}

impl<'a> ContentHash for Argument<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::SpreadElement(it) => it.content_hash(state),
            Self::BooleanLiteral(it) => it.content_hash(state),
            Self::NullLiteral(it) => it.content_hash(state),
            Self::NumericLiteral(it) => it.content_hash(state),
            Self::BigIntLiteral(it) => it.content_hash(state),
            Self::RegExpLiteral(it) => it.content_hash(state),
            Self::StringLiteral(it) => it.content_hash(state),
            Self::TemplateLiteral(it) => it.content_hash(state),
            Self::Identifier(it) => it.content_hash(state),
            Self::MetaProperty(it) => it.content_hash(state),
            Self::Super(it) => it.content_hash(state),
            Self::ArrayExpression(it) => it.content_hash(state),
            Self::ArrowFunctionExpression(it) => it.content_hash(state),
            Self::AssignmentExpression(it) => it.content_hash(state),
            Self::AwaitExpression(it) => it.content_hash(state),
            Self::BinaryExpression(it) => it.content_hash(state),
            Self::CallExpression(it) => it.content_hash(state),
            Self::ChainExpression(it) => it.content_hash(state),
            Self::ClassExpression(it) => it.content_hash(state),
            Self::ConditionalExpression(it) => it.content_hash(state),
            Self::FunctionExpression(it) => it.content_hash(state),
            Self::ImportExpression(it) => it.content_hash(state),
            Self::LogicalExpression(it) => it.content_hash(state),
            Self::NewExpression(it) => it.content_hash(state),
            Self::ObjectExpression(it) => it.content_hash(state),
            Self::ParenthesizedExpression(it) => it.content_hash(state),
            Self::SequenceExpression(it) => it.content_hash(state),
            Self::TaggedTemplateExpression(it) => it.content_hash(state),
            Self::ThisExpression(it) => it.content_hash(state),
            Self::UnaryExpression(it) => it.content_hash(state),
            Self::UpdateExpression(it) => it.content_hash(state),
            Self::YieldExpression(it) => it.content_hash(state),
            Self::PrivateInExpression(it) => it.content_hash(state),
            Self::JSXElement(it) => it.content_hash(state),
            Self::JSXFragment(it) => it.content_hash(state),
            Self::TSAsExpression(it) => it.content_hash(state),
            Self::TSSatisfiesExpression(it) => it.content_hash(state),
            Self::TSTypeAssertion(it) => it.content_hash(state),
            Self::TSNonNullExpression(it) => it.content_hash(state),
            Self::TSInstantiationExpression(it) => it.content_hash(state),
            Self::ComputedMemberExpression(it) => it.content_hash(state),
            Self::StaticMemberExpression(it) => it.content_hash(state),
            Self::PrivateFieldExpression(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for UpdateExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.operator.content_hash(state);
        self.prefix.content_hash(state);
        self.argument.content_hash(state);
    }
}

impl<'a> ContentHash for UnaryExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.operator.content_hash(state);
        self.argument.content_hash(state);
    }
}

impl<'a> ContentHash for BinaryExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.left.content_hash(state);
        self.operator.content_hash(state);
        self.right.content_hash(state);
    }
}

impl<'a> ContentHash for PrivateInExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.left.content_hash(state);
        self.operator.content_hash(state);
        self.right.content_hash(state);
    }
}

impl<'a> ContentHash for LogicalExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.left.content_hash(state);
        self.operator.content_hash(state);
        self.right.content_hash(state);
    }
}

impl<'a> ContentHash for ConditionalExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.test.content_hash(state);
        self.consequent.content_hash(state);
        self.alternate.content_hash(state);
    }
}

impl<'a> ContentHash for AssignmentExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.operator.content_hash(state);
        self.left.content_hash(state);
        self.right.content_hash(state);
    }
}

impl<'a> ContentHash for AssignmentTarget<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::AssignmentTargetIdentifier(it) => it.content_hash(state),
            Self::TSAsExpression(it) => it.content_hash(state),
            Self::TSSatisfiesExpression(it) => it.content_hash(state),
            Self::TSNonNullExpression(it) => it.content_hash(state),
            Self::TSTypeAssertion(it) => it.content_hash(state),
            Self::TSInstantiationExpression(it) => it.content_hash(state),
            Self::ComputedMemberExpression(it) => it.content_hash(state),
            Self::StaticMemberExpression(it) => it.content_hash(state),
            Self::PrivateFieldExpression(it) => it.content_hash(state),
            Self::ArrayAssignmentTarget(it) => it.content_hash(state),
            Self::ObjectAssignmentTarget(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for SimpleAssignmentTarget<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::AssignmentTargetIdentifier(it) => it.content_hash(state),
            Self::TSAsExpression(it) => it.content_hash(state),
            Self::TSSatisfiesExpression(it) => it.content_hash(state),
            Self::TSNonNullExpression(it) => it.content_hash(state),
            Self::TSTypeAssertion(it) => it.content_hash(state),
            Self::TSInstantiationExpression(it) => it.content_hash(state),
            Self::ComputedMemberExpression(it) => it.content_hash(state),
            Self::StaticMemberExpression(it) => it.content_hash(state),
            Self::PrivateFieldExpression(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for AssignmentTargetPattern<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ArrayAssignmentTarget(it) => it.content_hash(state),
            Self::ObjectAssignmentTarget(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for ArrayAssignmentTarget<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.elements.content_hash(state);
        self.rest.content_hash(state);
    }
}

impl<'a> ContentHash for ObjectAssignmentTarget<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.properties.content_hash(state);
        self.rest.content_hash(state);
    }
}

impl<'a> ContentHash for AssignmentTargetRest<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.target.content_hash(state);
    }
}

impl<'a> ContentHash for AssignmentTargetMaybeDefault<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::AssignmentTargetWithDefault(it) => it.content_hash(state),
            Self::AssignmentTargetIdentifier(it) => it.content_hash(state),
            Self::TSAsExpression(it) => it.content_hash(state),
            Self::TSSatisfiesExpression(it) => it.content_hash(state),
            Self::TSNonNullExpression(it) => it.content_hash(state),
            Self::TSTypeAssertion(it) => it.content_hash(state),
            Self::TSInstantiationExpression(it) => it.content_hash(state),
            Self::ComputedMemberExpression(it) => it.content_hash(state),
            Self::StaticMemberExpression(it) => it.content_hash(state),
            Self::PrivateFieldExpression(it) => it.content_hash(state),
            Self::ArrayAssignmentTarget(it) => it.content_hash(state),
            Self::ObjectAssignmentTarget(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for AssignmentTargetWithDefault<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.binding.content_hash(state);
        self.init.content_hash(state);
    }
}

impl<'a> ContentHash for AssignmentTargetProperty<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => it.content_hash(state),
            Self::AssignmentTargetPropertyProperty(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for AssignmentTargetPropertyIdentifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.binding.content_hash(state);
        self.init.content_hash(state);
    }
}

impl<'a> ContentHash for AssignmentTargetPropertyProperty<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.name.content_hash(state);
        self.binding.content_hash(state);
    }
}

impl<'a> ContentHash for SequenceExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expressions.content_hash(state);
    }
}

impl ContentHash for Super {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for AwaitExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.argument.content_hash(state);
    }
}

impl<'a> ContentHash for ChainExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
    }
}

impl<'a> ContentHash for ChainElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::CallExpression(it) => it.content_hash(state),
            Self::ComputedMemberExpression(it) => it.content_hash(state),
            Self::StaticMemberExpression(it) => it.content_hash(state),
            Self::PrivateFieldExpression(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for ParenthesizedExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
    }
}

impl<'a> ContentHash for Statement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::BlockStatement(it) => it.content_hash(state),
            Self::BreakStatement(it) => it.content_hash(state),
            Self::ContinueStatement(it) => it.content_hash(state),
            Self::DebuggerStatement(it) => it.content_hash(state),
            Self::DoWhileStatement(it) => it.content_hash(state),
            Self::EmptyStatement(it) => it.content_hash(state),
            Self::ExpressionStatement(it) => it.content_hash(state),
            Self::ForInStatement(it) => it.content_hash(state),
            Self::ForOfStatement(it) => it.content_hash(state),
            Self::ForStatement(it) => it.content_hash(state),
            Self::IfStatement(it) => it.content_hash(state),
            Self::LabeledStatement(it) => it.content_hash(state),
            Self::ReturnStatement(it) => it.content_hash(state),
            Self::SwitchStatement(it) => it.content_hash(state),
            Self::ThrowStatement(it) => it.content_hash(state),
            Self::TryStatement(it) => it.content_hash(state),
            Self::WhileStatement(it) => it.content_hash(state),
            Self::WithStatement(it) => it.content_hash(state),
            Self::VariableDeclaration(it) => it.content_hash(state),
            Self::FunctionDeclaration(it) => it.content_hash(state),
            Self::ClassDeclaration(it) => it.content_hash(state),
            Self::TSTypeAliasDeclaration(it) => it.content_hash(state),
            Self::TSInterfaceDeclaration(it) => it.content_hash(state),
            Self::TSEnumDeclaration(it) => it.content_hash(state),
            Self::TSModuleDeclaration(it) => it.content_hash(state),
            Self::TSImportEqualsDeclaration(it) => it.content_hash(state),
            Self::ImportDeclaration(it) => it.content_hash(state),
            Self::ExportAllDeclaration(it) => it.content_hash(state),
            Self::ExportDefaultDeclaration(it) => it.content_hash(state),
            Self::ExportNamedDeclaration(it) => it.content_hash(state),
            Self::TSExportAssignment(it) => it.content_hash(state),
            Self::TSNamespaceExportDeclaration(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for Directive<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
        self.directive.content_hash(state);
    }
}

impl<'a> ContentHash for Hashbang<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.value.content_hash(state);
    }
}

impl<'a> ContentHash for BlockStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for Declaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::VariableDeclaration(it) => it.content_hash(state),
            Self::FunctionDeclaration(it) => it.content_hash(state),
            Self::ClassDeclaration(it) => it.content_hash(state),
            Self::TSTypeAliasDeclaration(it) => it.content_hash(state),
            Self::TSInterfaceDeclaration(it) => it.content_hash(state),
            Self::TSEnumDeclaration(it) => it.content_hash(state),
            Self::TSModuleDeclaration(it) => it.content_hash(state),
            Self::TSImportEqualsDeclaration(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for VariableDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.kind.content_hash(state);
        self.declarations.content_hash(state);
        self.declare.content_hash(state);
    }
}

impl ContentHash for VariableDeclarationKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for VariableDeclarator<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.kind.content_hash(state);
        self.id.content_hash(state);
        self.init.content_hash(state);
        self.definite.content_hash(state);
    }
}

impl ContentHash for EmptyStatement {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for ExpressionStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
    }
}

impl<'a> ContentHash for IfStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.test.content_hash(state);
        self.consequent.content_hash(state);
        self.alternate.content_hash(state);
    }
}

impl<'a> ContentHash for DoWhileStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.body.content_hash(state);
        self.test.content_hash(state);
    }
}

impl<'a> ContentHash for WhileStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.test.content_hash(state);
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for ForStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.init.content_hash(state);
        self.test.content_hash(state);
        self.update.content_hash(state);
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for ForStatementInit<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::VariableDeclaration(it) => it.content_hash(state),
            Self::BooleanLiteral(it) => it.content_hash(state),
            Self::NullLiteral(it) => it.content_hash(state),
            Self::NumericLiteral(it) => it.content_hash(state),
            Self::BigIntLiteral(it) => it.content_hash(state),
            Self::RegExpLiteral(it) => it.content_hash(state),
            Self::StringLiteral(it) => it.content_hash(state),
            Self::TemplateLiteral(it) => it.content_hash(state),
            Self::Identifier(it) => it.content_hash(state),
            Self::MetaProperty(it) => it.content_hash(state),
            Self::Super(it) => it.content_hash(state),
            Self::ArrayExpression(it) => it.content_hash(state),
            Self::ArrowFunctionExpression(it) => it.content_hash(state),
            Self::AssignmentExpression(it) => it.content_hash(state),
            Self::AwaitExpression(it) => it.content_hash(state),
            Self::BinaryExpression(it) => it.content_hash(state),
            Self::CallExpression(it) => it.content_hash(state),
            Self::ChainExpression(it) => it.content_hash(state),
            Self::ClassExpression(it) => it.content_hash(state),
            Self::ConditionalExpression(it) => it.content_hash(state),
            Self::FunctionExpression(it) => it.content_hash(state),
            Self::ImportExpression(it) => it.content_hash(state),
            Self::LogicalExpression(it) => it.content_hash(state),
            Self::NewExpression(it) => it.content_hash(state),
            Self::ObjectExpression(it) => it.content_hash(state),
            Self::ParenthesizedExpression(it) => it.content_hash(state),
            Self::SequenceExpression(it) => it.content_hash(state),
            Self::TaggedTemplateExpression(it) => it.content_hash(state),
            Self::ThisExpression(it) => it.content_hash(state),
            Self::UnaryExpression(it) => it.content_hash(state),
            Self::UpdateExpression(it) => it.content_hash(state),
            Self::YieldExpression(it) => it.content_hash(state),
            Self::PrivateInExpression(it) => it.content_hash(state),
            Self::JSXElement(it) => it.content_hash(state),
            Self::JSXFragment(it) => it.content_hash(state),
            Self::TSAsExpression(it) => it.content_hash(state),
            Self::TSSatisfiesExpression(it) => it.content_hash(state),
            Self::TSTypeAssertion(it) => it.content_hash(state),
            Self::TSNonNullExpression(it) => it.content_hash(state),
            Self::TSInstantiationExpression(it) => it.content_hash(state),
            Self::ComputedMemberExpression(it) => it.content_hash(state),
            Self::StaticMemberExpression(it) => it.content_hash(state),
            Self::PrivateFieldExpression(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for ForInStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.left.content_hash(state);
        self.right.content_hash(state);
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for ForStatementLeft<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::VariableDeclaration(it) => it.content_hash(state),
            Self::AssignmentTargetIdentifier(it) => it.content_hash(state),
            Self::TSAsExpression(it) => it.content_hash(state),
            Self::TSSatisfiesExpression(it) => it.content_hash(state),
            Self::TSNonNullExpression(it) => it.content_hash(state),
            Self::TSTypeAssertion(it) => it.content_hash(state),
            Self::TSInstantiationExpression(it) => it.content_hash(state),
            Self::ComputedMemberExpression(it) => it.content_hash(state),
            Self::StaticMemberExpression(it) => it.content_hash(state),
            Self::PrivateFieldExpression(it) => it.content_hash(state),
            Self::ArrayAssignmentTarget(it) => it.content_hash(state),
            Self::ObjectAssignmentTarget(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for ForOfStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.r#await.content_hash(state);
        self.left.content_hash(state);
        self.right.content_hash(state);
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for ContinueStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.label.content_hash(state);
    }
}

impl<'a> ContentHash for BreakStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.label.content_hash(state);
    }
}

impl<'a> ContentHash for ReturnStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.argument.content_hash(state);
    }
}

impl<'a> ContentHash for WithStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.object.content_hash(state);
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for SwitchStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.discriminant.content_hash(state);
        self.cases.content_hash(state);
    }
}

impl<'a> ContentHash for SwitchCase<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.test.content_hash(state);
        self.consequent.content_hash(state);
    }
}

impl<'a> ContentHash for LabeledStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.label.content_hash(state);
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for ThrowStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.argument.content_hash(state);
    }
}

impl<'a> ContentHash for TryStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.block.content_hash(state);
        self.handler.content_hash(state);
        self.finalizer.content_hash(state);
    }
}

impl<'a> ContentHash for CatchClause<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.param.content_hash(state);
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for CatchParameter<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.pattern.content_hash(state);
    }
}

impl ContentHash for DebuggerStatement {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for BindingPattern<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.kind.content_hash(state);
        self.type_annotation.content_hash(state);
        self.optional.content_hash(state);
    }
}

impl<'a> ContentHash for BindingPatternKind<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::BindingIdentifier(it) => it.content_hash(state),
            Self::ObjectPattern(it) => it.content_hash(state),
            Self::ArrayPattern(it) => it.content_hash(state),
            Self::AssignmentPattern(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for AssignmentPattern<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.left.content_hash(state);
        self.right.content_hash(state);
    }
}

impl<'a> ContentHash for ObjectPattern<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.properties.content_hash(state);
        self.rest.content_hash(state);
    }
}

impl<'a> ContentHash for BindingProperty<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.key.content_hash(state);
        self.value.content_hash(state);
        self.shorthand.content_hash(state);
        self.computed.content_hash(state);
    }
}

impl<'a> ContentHash for ArrayPattern<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.elements.content_hash(state);
        self.rest.content_hash(state);
    }
}

impl<'a> ContentHash for BindingRestElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.argument.content_hash(state);
    }
}

impl<'a> ContentHash for Function<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.r#type.content_hash(state);
        self.id.content_hash(state);
        self.generator.content_hash(state);
        self.r#async.content_hash(state);
        self.declare.content_hash(state);
        self.type_parameters.content_hash(state);
        self.this_param.content_hash(state);
        self.params.content_hash(state);
        self.return_type.content_hash(state);
        self.body.content_hash(state);
    }
}

impl ContentHash for FunctionType {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for FormalParameters<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.kind.content_hash(state);
        self.items.content_hash(state);
        self.rest.content_hash(state);
    }
}

impl<'a> ContentHash for FormalParameter<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.decorators.content_hash(state);
        self.pattern.content_hash(state);
        self.accessibility.content_hash(state);
        self.readonly.content_hash(state);
        self.r#override.content_hash(state);
    }
}

impl ContentHash for FormalParameterKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for FunctionBody<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.directives.content_hash(state);
        self.statements.content_hash(state);
    }
}

impl<'a> ContentHash for ArrowFunctionExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
        self.r#async.content_hash(state);
        self.type_parameters.content_hash(state);
        self.params.content_hash(state);
        self.return_type.content_hash(state);
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for YieldExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.delegate.content_hash(state);
        self.argument.content_hash(state);
    }
}

impl<'a> ContentHash for Class<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.r#type.content_hash(state);
        self.decorators.content_hash(state);
        self.id.content_hash(state);
        self.type_parameters.content_hash(state);
        self.super_class.content_hash(state);
        self.super_type_parameters.content_hash(state);
        self.implements.content_hash(state);
        self.body.content_hash(state);
        self.r#abstract.content_hash(state);
        self.declare.content_hash(state);
    }
}

impl ContentHash for ClassType {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for ClassBody<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for ClassElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::StaticBlock(it) => it.content_hash(state),
            Self::MethodDefinition(it) => it.content_hash(state),
            Self::PropertyDefinition(it) => it.content_hash(state),
            Self::AccessorProperty(it) => it.content_hash(state),
            Self::TSIndexSignature(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for MethodDefinition<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.r#type.content_hash(state);
        self.decorators.content_hash(state);
        self.key.content_hash(state);
        self.value.content_hash(state);
        self.kind.content_hash(state);
        self.computed.content_hash(state);
        self.r#static.content_hash(state);
        self.r#override.content_hash(state);
        self.optional.content_hash(state);
        self.accessibility.content_hash(state);
    }
}

impl ContentHash for MethodDefinitionType {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for PropertyDefinition<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.r#type.content_hash(state);
        self.decorators.content_hash(state);
        self.key.content_hash(state);
        self.value.content_hash(state);
        self.computed.content_hash(state);
        self.r#static.content_hash(state);
        self.declare.content_hash(state);
        self.r#override.content_hash(state);
        self.optional.content_hash(state);
        self.definite.content_hash(state);
        self.readonly.content_hash(state);
        self.type_annotation.content_hash(state);
        self.accessibility.content_hash(state);
    }
}

impl ContentHash for PropertyDefinitionType {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl ContentHash for MethodDefinitionKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for PrivateIdentifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.name.content_hash(state);
    }
}

impl<'a> ContentHash for StaticBlock<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for ModuleDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ImportDeclaration(it) => it.content_hash(state),
            Self::ExportAllDeclaration(it) => it.content_hash(state),
            Self::ExportDefaultDeclaration(it) => it.content_hash(state),
            Self::ExportNamedDeclaration(it) => it.content_hash(state),
            Self::TSExportAssignment(it) => it.content_hash(state),
            Self::TSNamespaceExportDeclaration(it) => it.content_hash(state),
        }
    }
}

impl ContentHash for AccessorPropertyType {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for AccessorProperty<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.r#type.content_hash(state);
        self.decorators.content_hash(state);
        self.key.content_hash(state);
        self.value.content_hash(state);
        self.computed.content_hash(state);
        self.r#static.content_hash(state);
        self.definite.content_hash(state);
        self.type_annotation.content_hash(state);
        self.accessibility.content_hash(state);
    }
}

impl<'a> ContentHash for ImportExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.source.content_hash(state);
        self.arguments.content_hash(state);
    }
}

impl<'a> ContentHash for ImportDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.specifiers.content_hash(state);
        self.source.content_hash(state);
        self.with_clause.content_hash(state);
        self.import_kind.content_hash(state);
    }
}

impl<'a> ContentHash for ImportDeclarationSpecifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ImportSpecifier(it) => it.content_hash(state),
            Self::ImportDefaultSpecifier(it) => it.content_hash(state),
            Self::ImportNamespaceSpecifier(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for ImportSpecifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.imported.content_hash(state);
        self.local.content_hash(state);
        self.import_kind.content_hash(state);
    }
}

impl<'a> ContentHash for ImportDefaultSpecifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.local.content_hash(state);
    }
}

impl<'a> ContentHash for ImportNamespaceSpecifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.local.content_hash(state);
    }
}

impl<'a> ContentHash for WithClause<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.attributes_keyword.content_hash(state);
        self.with_entries.content_hash(state);
    }
}

impl<'a> ContentHash for ImportAttribute<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.key.content_hash(state);
        self.value.content_hash(state);
    }
}

impl<'a> ContentHash for ImportAttributeKey<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => it.content_hash(state),
            Self::StringLiteral(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for ExportNamedDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.declaration.content_hash(state);
        self.specifiers.content_hash(state);
        self.source.content_hash(state);
        self.export_kind.content_hash(state);
        self.with_clause.content_hash(state);
    }
}

impl<'a> ContentHash for ExportDefaultDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.declaration.content_hash(state);
        self.exported.content_hash(state);
    }
}

impl<'a> ContentHash for ExportAllDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.exported.content_hash(state);
        self.source.content_hash(state);
        self.with_clause.content_hash(state);
        self.export_kind.content_hash(state);
    }
}

impl<'a> ContentHash for ExportSpecifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.local.content_hash(state);
        self.exported.content_hash(state);
        self.export_kind.content_hash(state);
    }
}

impl<'a> ContentHash for ExportDefaultDeclarationKind<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::FunctionDeclaration(it) => it.content_hash(state),
            Self::ClassDeclaration(it) => it.content_hash(state),
            Self::TSInterfaceDeclaration(it) => it.content_hash(state),
            Self::BooleanLiteral(it) => it.content_hash(state),
            Self::NullLiteral(it) => it.content_hash(state),
            Self::NumericLiteral(it) => it.content_hash(state),
            Self::BigIntLiteral(it) => it.content_hash(state),
            Self::RegExpLiteral(it) => it.content_hash(state),
            Self::StringLiteral(it) => it.content_hash(state),
            Self::TemplateLiteral(it) => it.content_hash(state),
            Self::Identifier(it) => it.content_hash(state),
            Self::MetaProperty(it) => it.content_hash(state),
            Self::Super(it) => it.content_hash(state),
            Self::ArrayExpression(it) => it.content_hash(state),
            Self::ArrowFunctionExpression(it) => it.content_hash(state),
            Self::AssignmentExpression(it) => it.content_hash(state),
            Self::AwaitExpression(it) => it.content_hash(state),
            Self::BinaryExpression(it) => it.content_hash(state),
            Self::CallExpression(it) => it.content_hash(state),
            Self::ChainExpression(it) => it.content_hash(state),
            Self::ClassExpression(it) => it.content_hash(state),
            Self::ConditionalExpression(it) => it.content_hash(state),
            Self::FunctionExpression(it) => it.content_hash(state),
            Self::ImportExpression(it) => it.content_hash(state),
            Self::LogicalExpression(it) => it.content_hash(state),
            Self::NewExpression(it) => it.content_hash(state),
            Self::ObjectExpression(it) => it.content_hash(state),
            Self::ParenthesizedExpression(it) => it.content_hash(state),
            Self::SequenceExpression(it) => it.content_hash(state),
            Self::TaggedTemplateExpression(it) => it.content_hash(state),
            Self::ThisExpression(it) => it.content_hash(state),
            Self::UnaryExpression(it) => it.content_hash(state),
            Self::UpdateExpression(it) => it.content_hash(state),
            Self::YieldExpression(it) => it.content_hash(state),
            Self::PrivateInExpression(it) => it.content_hash(state),
            Self::JSXElement(it) => it.content_hash(state),
            Self::JSXFragment(it) => it.content_hash(state),
            Self::TSAsExpression(it) => it.content_hash(state),
            Self::TSSatisfiesExpression(it) => it.content_hash(state),
            Self::TSTypeAssertion(it) => it.content_hash(state),
            Self::TSNonNullExpression(it) => it.content_hash(state),
            Self::TSInstantiationExpression(it) => it.content_hash(state),
            Self::ComputedMemberExpression(it) => it.content_hash(state),
            Self::StaticMemberExpression(it) => it.content_hash(state),
            Self::PrivateFieldExpression(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for ModuleExportName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::IdentifierName(it) => it.content_hash(state),
            Self::IdentifierReference(it) => it.content_hash(state),
            Self::StringLiteral(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for TSThisParameter<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.type_annotation.content_hash(state);
    }
}

impl<'a> ContentHash for TSEnumDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.id.content_hash(state);
        self.members.content_hash(state);
        self.r#const.content_hash(state);
        self.declare.content_hash(state);
    }
}

impl<'a> ContentHash for TSEnumMember<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.id.content_hash(state);
        self.initializer.content_hash(state);
    }
}

impl<'a> ContentHash for TSEnumMemberName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::StaticIdentifier(it) => it.content_hash(state),
            Self::StaticStringLiteral(it) => it.content_hash(state),
            Self::StaticTemplateLiteral(it) => it.content_hash(state),
            Self::StaticNumericLiteral(it) => it.content_hash(state),
            Self::BooleanLiteral(it) => it.content_hash(state),
            Self::NullLiteral(it) => it.content_hash(state),
            Self::NumericLiteral(it) => it.content_hash(state),
            Self::BigIntLiteral(it) => it.content_hash(state),
            Self::RegExpLiteral(it) => it.content_hash(state),
            Self::StringLiteral(it) => it.content_hash(state),
            Self::TemplateLiteral(it) => it.content_hash(state),
            Self::Identifier(it) => it.content_hash(state),
            Self::MetaProperty(it) => it.content_hash(state),
            Self::Super(it) => it.content_hash(state),
            Self::ArrayExpression(it) => it.content_hash(state),
            Self::ArrowFunctionExpression(it) => it.content_hash(state),
            Self::AssignmentExpression(it) => it.content_hash(state),
            Self::AwaitExpression(it) => it.content_hash(state),
            Self::BinaryExpression(it) => it.content_hash(state),
            Self::CallExpression(it) => it.content_hash(state),
            Self::ChainExpression(it) => it.content_hash(state),
            Self::ClassExpression(it) => it.content_hash(state),
            Self::ConditionalExpression(it) => it.content_hash(state),
            Self::FunctionExpression(it) => it.content_hash(state),
            Self::ImportExpression(it) => it.content_hash(state),
            Self::LogicalExpression(it) => it.content_hash(state),
            Self::NewExpression(it) => it.content_hash(state),
            Self::ObjectExpression(it) => it.content_hash(state),
            Self::ParenthesizedExpression(it) => it.content_hash(state),
            Self::SequenceExpression(it) => it.content_hash(state),
            Self::TaggedTemplateExpression(it) => it.content_hash(state),
            Self::ThisExpression(it) => it.content_hash(state),
            Self::UnaryExpression(it) => it.content_hash(state),
            Self::UpdateExpression(it) => it.content_hash(state),
            Self::YieldExpression(it) => it.content_hash(state),
            Self::PrivateInExpression(it) => it.content_hash(state),
            Self::JSXElement(it) => it.content_hash(state),
            Self::JSXFragment(it) => it.content_hash(state),
            Self::TSAsExpression(it) => it.content_hash(state),
            Self::TSSatisfiesExpression(it) => it.content_hash(state),
            Self::TSTypeAssertion(it) => it.content_hash(state),
            Self::TSNonNullExpression(it) => it.content_hash(state),
            Self::TSInstantiationExpression(it) => it.content_hash(state),
            Self::ComputedMemberExpression(it) => it.content_hash(state),
            Self::StaticMemberExpression(it) => it.content_hash(state),
            Self::PrivateFieldExpression(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for TSTypeAnnotation<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.type_annotation.content_hash(state);
    }
}

impl<'a> ContentHash for TSLiteralType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.literal.content_hash(state);
    }
}

impl<'a> ContentHash for TSLiteral<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::BooleanLiteral(it) => it.content_hash(state),
            Self::NullLiteral(it) => it.content_hash(state),
            Self::NumericLiteral(it) => it.content_hash(state),
            Self::BigIntLiteral(it) => it.content_hash(state),
            Self::RegExpLiteral(it) => it.content_hash(state),
            Self::StringLiteral(it) => it.content_hash(state),
            Self::TemplateLiteral(it) => it.content_hash(state),
            Self::UnaryExpression(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for TSType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::TSAnyKeyword(it) => it.content_hash(state),
            Self::TSBigIntKeyword(it) => it.content_hash(state),
            Self::TSBooleanKeyword(it) => it.content_hash(state),
            Self::TSIntrinsicKeyword(it) => it.content_hash(state),
            Self::TSNeverKeyword(it) => it.content_hash(state),
            Self::TSNullKeyword(it) => it.content_hash(state),
            Self::TSNumberKeyword(it) => it.content_hash(state),
            Self::TSObjectKeyword(it) => it.content_hash(state),
            Self::TSStringKeyword(it) => it.content_hash(state),
            Self::TSSymbolKeyword(it) => it.content_hash(state),
            Self::TSUndefinedKeyword(it) => it.content_hash(state),
            Self::TSUnknownKeyword(it) => it.content_hash(state),
            Self::TSVoidKeyword(it) => it.content_hash(state),
            Self::TSArrayType(it) => it.content_hash(state),
            Self::TSConditionalType(it) => it.content_hash(state),
            Self::TSConstructorType(it) => it.content_hash(state),
            Self::TSFunctionType(it) => it.content_hash(state),
            Self::TSImportType(it) => it.content_hash(state),
            Self::TSIndexedAccessType(it) => it.content_hash(state),
            Self::TSInferType(it) => it.content_hash(state),
            Self::TSIntersectionType(it) => it.content_hash(state),
            Self::TSLiteralType(it) => it.content_hash(state),
            Self::TSMappedType(it) => it.content_hash(state),
            Self::TSNamedTupleMember(it) => it.content_hash(state),
            Self::TSQualifiedName(it) => it.content_hash(state),
            Self::TSTemplateLiteralType(it) => it.content_hash(state),
            Self::TSThisType(it) => it.content_hash(state),
            Self::TSTupleType(it) => it.content_hash(state),
            Self::TSTypeLiteral(it) => it.content_hash(state),
            Self::TSTypeOperatorType(it) => it.content_hash(state),
            Self::TSTypePredicate(it) => it.content_hash(state),
            Self::TSTypeQuery(it) => it.content_hash(state),
            Self::TSTypeReference(it) => it.content_hash(state),
            Self::TSUnionType(it) => it.content_hash(state),
            Self::TSParenthesizedType(it) => it.content_hash(state),
            Self::JSDocNullableType(it) => it.content_hash(state),
            Self::JSDocNonNullableType(it) => it.content_hash(state),
            Self::JSDocUnknownType(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for TSConditionalType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.check_type.content_hash(state);
        self.extends_type.content_hash(state);
        self.true_type.content_hash(state);
        self.false_type.content_hash(state);
    }
}

impl<'a> ContentHash for TSUnionType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.types.content_hash(state);
    }
}

impl<'a> ContentHash for TSIntersectionType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.types.content_hash(state);
    }
}

impl<'a> ContentHash for TSParenthesizedType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.type_annotation.content_hash(state);
    }
}

impl<'a> ContentHash for TSTypeOperator<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.operator.content_hash(state);
        self.type_annotation.content_hash(state);
    }
}

impl ContentHash for TSTypeOperatorOperator {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for TSArrayType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.element_type.content_hash(state);
    }
}

impl<'a> ContentHash for TSIndexedAccessType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.object_type.content_hash(state);
        self.index_type.content_hash(state);
    }
}

impl<'a> ContentHash for TSTupleType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.element_types.content_hash(state);
    }
}

impl<'a> ContentHash for TSNamedTupleMember<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.element_type.content_hash(state);
        self.label.content_hash(state);
        self.optional.content_hash(state);
    }
}

impl<'a> ContentHash for TSOptionalType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.type_annotation.content_hash(state);
    }
}

impl<'a> ContentHash for TSRestType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.type_annotation.content_hash(state);
    }
}

impl<'a> ContentHash for TSTupleElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::TSOptionalType(it) => it.content_hash(state),
            Self::TSRestType(it) => it.content_hash(state),
            Self::TSAnyKeyword(it) => it.content_hash(state),
            Self::TSBigIntKeyword(it) => it.content_hash(state),
            Self::TSBooleanKeyword(it) => it.content_hash(state),
            Self::TSIntrinsicKeyword(it) => it.content_hash(state),
            Self::TSNeverKeyword(it) => it.content_hash(state),
            Self::TSNullKeyword(it) => it.content_hash(state),
            Self::TSNumberKeyword(it) => it.content_hash(state),
            Self::TSObjectKeyword(it) => it.content_hash(state),
            Self::TSStringKeyword(it) => it.content_hash(state),
            Self::TSSymbolKeyword(it) => it.content_hash(state),
            Self::TSUndefinedKeyword(it) => it.content_hash(state),
            Self::TSUnknownKeyword(it) => it.content_hash(state),
            Self::TSVoidKeyword(it) => it.content_hash(state),
            Self::TSArrayType(it) => it.content_hash(state),
            Self::TSConditionalType(it) => it.content_hash(state),
            Self::TSConstructorType(it) => it.content_hash(state),
            Self::TSFunctionType(it) => it.content_hash(state),
            Self::TSImportType(it) => it.content_hash(state),
            Self::TSIndexedAccessType(it) => it.content_hash(state),
            Self::TSInferType(it) => it.content_hash(state),
            Self::TSIntersectionType(it) => it.content_hash(state),
            Self::TSLiteralType(it) => it.content_hash(state),
            Self::TSMappedType(it) => it.content_hash(state),
            Self::TSNamedTupleMember(it) => it.content_hash(state),
            Self::TSQualifiedName(it) => it.content_hash(state),
            Self::TSTemplateLiteralType(it) => it.content_hash(state),
            Self::TSThisType(it) => it.content_hash(state),
            Self::TSTupleType(it) => it.content_hash(state),
            Self::TSTypeLiteral(it) => it.content_hash(state),
            Self::TSTypeOperatorType(it) => it.content_hash(state),
            Self::TSTypePredicate(it) => it.content_hash(state),
            Self::TSTypeQuery(it) => it.content_hash(state),
            Self::TSTypeReference(it) => it.content_hash(state),
            Self::TSUnionType(it) => it.content_hash(state),
            Self::TSParenthesizedType(it) => it.content_hash(state),
            Self::JSDocNullableType(it) => it.content_hash(state),
            Self::JSDocNonNullableType(it) => it.content_hash(state),
            Self::JSDocUnknownType(it) => it.content_hash(state),
        }
    }
}

impl ContentHash for TSAnyKeyword {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for TSStringKeyword {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for TSBooleanKeyword {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for TSNumberKeyword {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for TSNeverKeyword {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for TSIntrinsicKeyword {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for TSUnknownKeyword {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for TSNullKeyword {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for TSUndefinedKeyword {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for TSVoidKeyword {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for TSSymbolKeyword {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for TSThisType {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for TSObjectKeyword {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for TSBigIntKeyword {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for TSTypeReference<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.type_name.content_hash(state);
        self.type_parameters.content_hash(state);
    }
}

impl<'a> ContentHash for TSTypeName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::IdentifierReference(it) => it.content_hash(state),
            Self::QualifiedName(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for TSQualifiedName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.left.content_hash(state);
        self.right.content_hash(state);
    }
}

impl<'a> ContentHash for TSTypeParameterInstantiation<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.params.content_hash(state);
    }
}

impl<'a> ContentHash for TSTypeParameter<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.name.content_hash(state);
        self.constraint.content_hash(state);
        self.default.content_hash(state);
        self.r#in.content_hash(state);
        self.out.content_hash(state);
        self.r#const.content_hash(state);
    }
}

impl<'a> ContentHash for TSTypeParameterDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.params.content_hash(state);
    }
}

impl<'a> ContentHash for TSTypeAliasDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.id.content_hash(state);
        self.type_parameters.content_hash(state);
        self.type_annotation.content_hash(state);
        self.declare.content_hash(state);
    }
}

impl ContentHash for TSAccessibility {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for TSClassImplements<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
        self.type_parameters.content_hash(state);
    }
}

impl<'a> ContentHash for TSInterfaceDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.id.content_hash(state);
        self.extends.content_hash(state);
        self.type_parameters.content_hash(state);
        self.body.content_hash(state);
        self.declare.content_hash(state);
    }
}

impl<'a> ContentHash for TSInterfaceBody<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for TSPropertySignature<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.computed.content_hash(state);
        self.optional.content_hash(state);
        self.readonly.content_hash(state);
        self.key.content_hash(state);
        self.type_annotation.content_hash(state);
    }
}

impl<'a> ContentHash for TSSignature<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::TSIndexSignature(it) => it.content_hash(state),
            Self::TSPropertySignature(it) => it.content_hash(state),
            Self::TSCallSignatureDeclaration(it) => it.content_hash(state),
            Self::TSConstructSignatureDeclaration(it) => it.content_hash(state),
            Self::TSMethodSignature(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for TSIndexSignature<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.parameters.content_hash(state);
        self.type_annotation.content_hash(state);
        self.readonly.content_hash(state);
    }
}

impl<'a> ContentHash for TSCallSignatureDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.this_param.content_hash(state);
        self.params.content_hash(state);
        self.return_type.content_hash(state);
        self.type_parameters.content_hash(state);
    }
}

impl ContentHash for TSMethodSignatureKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for TSMethodSignature<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.key.content_hash(state);
        self.computed.content_hash(state);
        self.optional.content_hash(state);
        self.kind.content_hash(state);
        self.this_param.content_hash(state);
        self.params.content_hash(state);
        self.return_type.content_hash(state);
        self.type_parameters.content_hash(state);
    }
}

impl<'a> ContentHash for TSConstructSignatureDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.params.content_hash(state);
        self.return_type.content_hash(state);
        self.type_parameters.content_hash(state);
    }
}

impl<'a> ContentHash for TSIndexSignatureName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.name.content_hash(state);
        self.type_annotation.content_hash(state);
    }
}

impl<'a> ContentHash for TSInterfaceHeritage<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
        self.type_parameters.content_hash(state);
    }
}

impl<'a> ContentHash for TSTypePredicate<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.parameter_name.content_hash(state);
        self.asserts.content_hash(state);
        self.type_annotation.content_hash(state);
    }
}

impl<'a> ContentHash for TSTypePredicateName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => it.content_hash(state),
            Self::This(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for TSModuleDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.id.content_hash(state);
        self.body.content_hash(state);
        self.kind.content_hash(state);
        self.declare.content_hash(state);
    }
}

impl ContentHash for TSModuleDeclarationKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for TSModuleDeclarationName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => it.content_hash(state),
            Self::StringLiteral(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for TSModuleDeclarationBody<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::TSModuleDeclaration(it) => it.content_hash(state),
            Self::TSModuleBlock(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for TSModuleBlock<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.directives.content_hash(state);
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for TSTypeLiteral<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.members.content_hash(state);
    }
}

impl<'a> ContentHash for TSInferType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.type_parameter.content_hash(state);
    }
}

impl<'a> ContentHash for TSTypeQuery<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expr_name.content_hash(state);
        self.type_parameters.content_hash(state);
    }
}

impl<'a> ContentHash for TSTypeQueryExprName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::TSImportType(it) => it.content_hash(state),
            Self::IdentifierReference(it) => it.content_hash(state),
            Self::QualifiedName(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for TSImportType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.is_type_of.content_hash(state);
        self.parameter.content_hash(state);
        self.qualifier.content_hash(state);
        self.attributes.content_hash(state);
        self.type_parameters.content_hash(state);
    }
}

impl<'a> ContentHash for TSImportAttributes<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.attributes_keyword.content_hash(state);
        self.elements.content_hash(state);
    }
}

impl<'a> ContentHash for TSImportAttribute<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.name.content_hash(state);
        self.value.content_hash(state);
    }
}

impl<'a> ContentHash for TSImportAttributeName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => it.content_hash(state),
            Self::StringLiteral(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for TSFunctionType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.this_param.content_hash(state);
        self.params.content_hash(state);
        self.return_type.content_hash(state);
        self.type_parameters.content_hash(state);
    }
}

impl<'a> ContentHash for TSConstructorType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.r#abstract.content_hash(state);
        self.params.content_hash(state);
        self.return_type.content_hash(state);
        self.type_parameters.content_hash(state);
    }
}

impl<'a> ContentHash for TSMappedType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.type_parameter.content_hash(state);
        self.name_type.content_hash(state);
        self.type_annotation.content_hash(state);
        self.optional.content_hash(state);
        self.readonly.content_hash(state);
    }
}

impl ContentHash for TSMappedTypeModifierOperator {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for TSTemplateLiteralType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.quasis.content_hash(state);
        self.types.content_hash(state);
    }
}

impl<'a> ContentHash for TSAsExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
        self.type_annotation.content_hash(state);
    }
}

impl<'a> ContentHash for TSSatisfiesExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
        self.type_annotation.content_hash(state);
    }
}

impl<'a> ContentHash for TSTypeAssertion<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
        self.type_annotation.content_hash(state);
    }
}

impl<'a> ContentHash for TSImportEqualsDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.id.content_hash(state);
        self.module_reference.content_hash(state);
        self.import_kind.content_hash(state);
    }
}

impl<'a> ContentHash for TSModuleReference<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ExternalModuleReference(it) => it.content_hash(state),
            Self::IdentifierReference(it) => it.content_hash(state),
            Self::QualifiedName(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for TSExternalModuleReference<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
    }
}

impl<'a> ContentHash for TSNonNullExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
    }
}

impl<'a> ContentHash for Decorator<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
    }
}

impl<'a> ContentHash for TSExportAssignment<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
    }
}

impl<'a> ContentHash for TSNamespaceExportDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.id.content_hash(state);
    }
}

impl<'a> ContentHash for TSInstantiationExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
        self.type_parameters.content_hash(state);
    }
}

impl ContentHash for ImportOrExportKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for JSDocNullableType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.type_annotation.content_hash(state);
        self.postfix.content_hash(state);
    }
}

impl<'a> ContentHash for JSDocNonNullableType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.type_annotation.content_hash(state);
        self.postfix.content_hash(state);
    }
}

impl ContentHash for JSDocUnknownType {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for JSXElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.opening_element.content_hash(state);
        self.closing_element.content_hash(state);
        self.children.content_hash(state);
    }
}

impl<'a> ContentHash for JSXOpeningElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.self_closing.content_hash(state);
        self.name.content_hash(state);
        self.attributes.content_hash(state);
        self.type_parameters.content_hash(state);
    }
}

impl<'a> ContentHash for JSXClosingElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.name.content_hash(state);
    }
}

impl<'a> ContentHash for JSXFragment<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.opening_fragment.content_hash(state);
        self.closing_fragment.content_hash(state);
        self.children.content_hash(state);
    }
}

impl ContentHash for JSXOpeningFragment {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for JSXClosingFragment {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for JSXElementName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => it.content_hash(state),
            Self::IdentifierReference(it) => it.content_hash(state),
            Self::NamespacedName(it) => it.content_hash(state),
            Self::MemberExpression(it) => it.content_hash(state),
            Self::ThisExpression(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for JSXNamespacedName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.namespace.content_hash(state);
        self.property.content_hash(state);
    }
}

impl<'a> ContentHash for JSXMemberExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.object.content_hash(state);
        self.property.content_hash(state);
    }
}

impl<'a> ContentHash for JSXMemberExpressionObject<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::IdentifierReference(it) => it.content_hash(state),
            Self::MemberExpression(it) => it.content_hash(state),
            Self::ThisExpression(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for JSXExpressionContainer<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
    }
}

impl<'a> ContentHash for JSXExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::EmptyExpression(it) => it.content_hash(state),
            Self::BooleanLiteral(it) => it.content_hash(state),
            Self::NullLiteral(it) => it.content_hash(state),
            Self::NumericLiteral(it) => it.content_hash(state),
            Self::BigIntLiteral(it) => it.content_hash(state),
            Self::RegExpLiteral(it) => it.content_hash(state),
            Self::StringLiteral(it) => it.content_hash(state),
            Self::TemplateLiteral(it) => it.content_hash(state),
            Self::Identifier(it) => it.content_hash(state),
            Self::MetaProperty(it) => it.content_hash(state),
            Self::Super(it) => it.content_hash(state),
            Self::ArrayExpression(it) => it.content_hash(state),
            Self::ArrowFunctionExpression(it) => it.content_hash(state),
            Self::AssignmentExpression(it) => it.content_hash(state),
            Self::AwaitExpression(it) => it.content_hash(state),
            Self::BinaryExpression(it) => it.content_hash(state),
            Self::CallExpression(it) => it.content_hash(state),
            Self::ChainExpression(it) => it.content_hash(state),
            Self::ClassExpression(it) => it.content_hash(state),
            Self::ConditionalExpression(it) => it.content_hash(state),
            Self::FunctionExpression(it) => it.content_hash(state),
            Self::ImportExpression(it) => it.content_hash(state),
            Self::LogicalExpression(it) => it.content_hash(state),
            Self::NewExpression(it) => it.content_hash(state),
            Self::ObjectExpression(it) => it.content_hash(state),
            Self::ParenthesizedExpression(it) => it.content_hash(state),
            Self::SequenceExpression(it) => it.content_hash(state),
            Self::TaggedTemplateExpression(it) => it.content_hash(state),
            Self::ThisExpression(it) => it.content_hash(state),
            Self::UnaryExpression(it) => it.content_hash(state),
            Self::UpdateExpression(it) => it.content_hash(state),
            Self::YieldExpression(it) => it.content_hash(state),
            Self::PrivateInExpression(it) => it.content_hash(state),
            Self::JSXElement(it) => it.content_hash(state),
            Self::JSXFragment(it) => it.content_hash(state),
            Self::TSAsExpression(it) => it.content_hash(state),
            Self::TSSatisfiesExpression(it) => it.content_hash(state),
            Self::TSTypeAssertion(it) => it.content_hash(state),
            Self::TSNonNullExpression(it) => it.content_hash(state),
            Self::TSInstantiationExpression(it) => it.content_hash(state),
            Self::ComputedMemberExpression(it) => it.content_hash(state),
            Self::StaticMemberExpression(it) => it.content_hash(state),
            Self::PrivateFieldExpression(it) => it.content_hash(state),
        }
    }
}

impl ContentHash for JSXEmptyExpression {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for JSXAttributeItem<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Attribute(it) => it.content_hash(state),
            Self::SpreadAttribute(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for JSXAttribute<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.name.content_hash(state);
        self.value.content_hash(state);
    }
}

impl<'a> ContentHash for JSXSpreadAttribute<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.argument.content_hash(state);
    }
}

impl<'a> ContentHash for JSXAttributeName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => it.content_hash(state),
            Self::NamespacedName(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for JSXAttributeValue<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::StringLiteral(it) => it.content_hash(state),
            Self::ExpressionContainer(it) => it.content_hash(state),
            Self::Element(it) => it.content_hash(state),
            Self::Fragment(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for JSXIdentifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.name.content_hash(state);
    }
}

impl<'a> ContentHash for JSXChild<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Text(it) => it.content_hash(state),
            Self::Element(it) => it.content_hash(state),
            Self::Fragment(it) => it.content_hash(state),
            Self::ExpressionContainer(it) => it.content_hash(state),
            Self::Spread(it) => it.content_hash(state),
        }
    }
}

impl<'a> ContentHash for JSXSpreadChild<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.expression.content_hash(state);
    }
}

impl<'a> ContentHash for JSXText<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.value.content_hash(state);
    }
}
