// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/content_hash.rs`

#![allow(clippy::match_same_arms)]

use std::{hash::Hasher, mem::discriminant};

use oxc_span::hash::ContentHash;

#[allow(clippy::wildcard_imports)]
use crate::ast::comment::*;

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
        ContentHash::content_hash(&self.value, state);
    }
}

impl<'a> ContentHash for BigIntLiteral<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.raw, state);
        ContentHash::content_hash(&self.base, state);
    }
}

impl<'a> ContentHash for RegExpLiteral<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.value, state);
        ContentHash::content_hash(&self.regex, state);
    }
}

impl<'a> ContentHash for RegExp<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.pattern, state);
        ContentHash::content_hash(&self.flags, state);
    }
}

impl<'a> ContentHash for RegExpPattern<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Raw(it) => ContentHash::content_hash(it, state),
            Self::Invalid(it) => ContentHash::content_hash(it, state),
            Self::Pattern(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for EmptyObject {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for StringLiteral<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.value, state);
    }
}

impl<'a> ContentHash for Program<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.source_type, state);
        ContentHash::content_hash(&self.source_text, state);
        ContentHash::content_hash(&self.comments, state);
        ContentHash::content_hash(&self.hashbang, state);
        ContentHash::content_hash(&self.directives, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for Expression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::BooleanLiteral(it) => ContentHash::content_hash(it, state),
            Self::NullLiteral(it) => ContentHash::content_hash(it, state),
            Self::NumericLiteral(it) => ContentHash::content_hash(it, state),
            Self::BigIntLiteral(it) => ContentHash::content_hash(it, state),
            Self::RegExpLiteral(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
            Self::TemplateLiteral(it) => ContentHash::content_hash(it, state),
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::MetaProperty(it) => ContentHash::content_hash(it, state),
            Self::Super(it) => ContentHash::content_hash(it, state),
            Self::ArrayExpression(it) => ContentHash::content_hash(it, state),
            Self::ArrowFunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::AssignmentExpression(it) => ContentHash::content_hash(it, state),
            Self::AwaitExpression(it) => ContentHash::content_hash(it, state),
            Self::BinaryExpression(it) => ContentHash::content_hash(it, state),
            Self::CallExpression(it) => ContentHash::content_hash(it, state),
            Self::ChainExpression(it) => ContentHash::content_hash(it, state),
            Self::ClassExpression(it) => ContentHash::content_hash(it, state),
            Self::ConditionalExpression(it) => ContentHash::content_hash(it, state),
            Self::FunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::ImportExpression(it) => ContentHash::content_hash(it, state),
            Self::LogicalExpression(it) => ContentHash::content_hash(it, state),
            Self::NewExpression(it) => ContentHash::content_hash(it, state),
            Self::ObjectExpression(it) => ContentHash::content_hash(it, state),
            Self::ParenthesizedExpression(it) => ContentHash::content_hash(it, state),
            Self::SequenceExpression(it) => ContentHash::content_hash(it, state),
            Self::TaggedTemplateExpression(it) => ContentHash::content_hash(it, state),
            Self::ThisExpression(it) => ContentHash::content_hash(it, state),
            Self::UnaryExpression(it) => ContentHash::content_hash(it, state),
            Self::UpdateExpression(it) => ContentHash::content_hash(it, state),
            Self::YieldExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateInExpression(it) => ContentHash::content_hash(it, state),
            Self::JSXElement(it) => ContentHash::content_hash(it, state),
            Self::JSXFragment(it) => ContentHash::content_hash(it, state),
            Self::TSAsExpression(it) => ContentHash::content_hash(it, state),
            Self::TSSatisfiesExpression(it) => ContentHash::content_hash(it, state),
            Self::TSTypeAssertion(it) => ContentHash::content_hash(it, state),
            Self::TSNonNullExpression(it) => ContentHash::content_hash(it, state),
            Self::TSInstantiationExpression(it) => ContentHash::content_hash(it, state),
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for IdentifierName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
    }
}

impl<'a> ContentHash for IdentifierReference<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
    }
}

impl<'a> ContentHash for BindingIdentifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
    }
}

impl<'a> ContentHash for LabelIdentifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
    }
}

impl ContentHash for ThisExpression {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for ArrayExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.elements, state);
    }
}

impl<'a> ContentHash for ArrayExpressionElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::SpreadElement(it) => ContentHash::content_hash(it, state),
            Self::Elision(it) => ContentHash::content_hash(it, state),
            Self::BooleanLiteral(it) => ContentHash::content_hash(it, state),
            Self::NullLiteral(it) => ContentHash::content_hash(it, state),
            Self::NumericLiteral(it) => ContentHash::content_hash(it, state),
            Self::BigIntLiteral(it) => ContentHash::content_hash(it, state),
            Self::RegExpLiteral(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
            Self::TemplateLiteral(it) => ContentHash::content_hash(it, state),
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::MetaProperty(it) => ContentHash::content_hash(it, state),
            Self::Super(it) => ContentHash::content_hash(it, state),
            Self::ArrayExpression(it) => ContentHash::content_hash(it, state),
            Self::ArrowFunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::AssignmentExpression(it) => ContentHash::content_hash(it, state),
            Self::AwaitExpression(it) => ContentHash::content_hash(it, state),
            Self::BinaryExpression(it) => ContentHash::content_hash(it, state),
            Self::CallExpression(it) => ContentHash::content_hash(it, state),
            Self::ChainExpression(it) => ContentHash::content_hash(it, state),
            Self::ClassExpression(it) => ContentHash::content_hash(it, state),
            Self::ConditionalExpression(it) => ContentHash::content_hash(it, state),
            Self::FunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::ImportExpression(it) => ContentHash::content_hash(it, state),
            Self::LogicalExpression(it) => ContentHash::content_hash(it, state),
            Self::NewExpression(it) => ContentHash::content_hash(it, state),
            Self::ObjectExpression(it) => ContentHash::content_hash(it, state),
            Self::ParenthesizedExpression(it) => ContentHash::content_hash(it, state),
            Self::SequenceExpression(it) => ContentHash::content_hash(it, state),
            Self::TaggedTemplateExpression(it) => ContentHash::content_hash(it, state),
            Self::ThisExpression(it) => ContentHash::content_hash(it, state),
            Self::UnaryExpression(it) => ContentHash::content_hash(it, state),
            Self::UpdateExpression(it) => ContentHash::content_hash(it, state),
            Self::YieldExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateInExpression(it) => ContentHash::content_hash(it, state),
            Self::JSXElement(it) => ContentHash::content_hash(it, state),
            Self::JSXFragment(it) => ContentHash::content_hash(it, state),
            Self::TSAsExpression(it) => ContentHash::content_hash(it, state),
            Self::TSSatisfiesExpression(it) => ContentHash::content_hash(it, state),
            Self::TSTypeAssertion(it) => ContentHash::content_hash(it, state),
            Self::TSNonNullExpression(it) => ContentHash::content_hash(it, state),
            Self::TSInstantiationExpression(it) => ContentHash::content_hash(it, state),
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for Elision {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for ObjectExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.properties, state);
    }
}

impl<'a> ContentHash for ObjectPropertyKind<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ObjectProperty(it) => ContentHash::content_hash(it, state),
            Self::SpreadProperty(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for ObjectProperty<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.key, state);
        ContentHash::content_hash(&self.value, state);
        ContentHash::content_hash(&self.init, state);
        ContentHash::content_hash(&self.method, state);
        ContentHash::content_hash(&self.shorthand, state);
        ContentHash::content_hash(&self.computed, state);
    }
}

impl<'a> ContentHash for PropertyKey<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::StaticIdentifier(it) => ContentHash::content_hash(it, state),
            Self::PrivateIdentifier(it) => ContentHash::content_hash(it, state),
            Self::BooleanLiteral(it) => ContentHash::content_hash(it, state),
            Self::NullLiteral(it) => ContentHash::content_hash(it, state),
            Self::NumericLiteral(it) => ContentHash::content_hash(it, state),
            Self::BigIntLiteral(it) => ContentHash::content_hash(it, state),
            Self::RegExpLiteral(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
            Self::TemplateLiteral(it) => ContentHash::content_hash(it, state),
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::MetaProperty(it) => ContentHash::content_hash(it, state),
            Self::Super(it) => ContentHash::content_hash(it, state),
            Self::ArrayExpression(it) => ContentHash::content_hash(it, state),
            Self::ArrowFunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::AssignmentExpression(it) => ContentHash::content_hash(it, state),
            Self::AwaitExpression(it) => ContentHash::content_hash(it, state),
            Self::BinaryExpression(it) => ContentHash::content_hash(it, state),
            Self::CallExpression(it) => ContentHash::content_hash(it, state),
            Self::ChainExpression(it) => ContentHash::content_hash(it, state),
            Self::ClassExpression(it) => ContentHash::content_hash(it, state),
            Self::ConditionalExpression(it) => ContentHash::content_hash(it, state),
            Self::FunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::ImportExpression(it) => ContentHash::content_hash(it, state),
            Self::LogicalExpression(it) => ContentHash::content_hash(it, state),
            Self::NewExpression(it) => ContentHash::content_hash(it, state),
            Self::ObjectExpression(it) => ContentHash::content_hash(it, state),
            Self::ParenthesizedExpression(it) => ContentHash::content_hash(it, state),
            Self::SequenceExpression(it) => ContentHash::content_hash(it, state),
            Self::TaggedTemplateExpression(it) => ContentHash::content_hash(it, state),
            Self::ThisExpression(it) => ContentHash::content_hash(it, state),
            Self::UnaryExpression(it) => ContentHash::content_hash(it, state),
            Self::UpdateExpression(it) => ContentHash::content_hash(it, state),
            Self::YieldExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateInExpression(it) => ContentHash::content_hash(it, state),
            Self::JSXElement(it) => ContentHash::content_hash(it, state),
            Self::JSXFragment(it) => ContentHash::content_hash(it, state),
            Self::TSAsExpression(it) => ContentHash::content_hash(it, state),
            Self::TSSatisfiesExpression(it) => ContentHash::content_hash(it, state),
            Self::TSTypeAssertion(it) => ContentHash::content_hash(it, state),
            Self::TSNonNullExpression(it) => ContentHash::content_hash(it, state),
            Self::TSInstantiationExpression(it) => ContentHash::content_hash(it, state),
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
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
        ContentHash::content_hash(&self.quasis, state);
        ContentHash::content_hash(&self.expressions, state);
    }
}

impl<'a> ContentHash for TaggedTemplateExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.tag, state);
        ContentHash::content_hash(&self.quasi, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl<'a> ContentHash for TemplateElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.tail, state);
        ContentHash::content_hash(&self.value, state);
    }
}

impl<'a> ContentHash for TemplateElementValue<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.raw, state);
        ContentHash::content_hash(&self.cooked, state);
    }
}

impl<'a> ContentHash for MemberExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for ComputedMemberExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.object, state);
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.optional, state);
    }
}

impl<'a> ContentHash for StaticMemberExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.object, state);
        ContentHash::content_hash(&self.property, state);
        ContentHash::content_hash(&self.optional, state);
    }
}

impl<'a> ContentHash for PrivateFieldExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.object, state);
        ContentHash::content_hash(&self.field, state);
        ContentHash::content_hash(&self.optional, state);
    }
}

impl<'a> ContentHash for CallExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.callee, state);
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.arguments, state);
        ContentHash::content_hash(&self.optional, state);
    }
}

impl<'a> ContentHash for NewExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.callee, state);
        ContentHash::content_hash(&self.arguments, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl<'a> ContentHash for MetaProperty<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.meta, state);
        ContentHash::content_hash(&self.property, state);
    }
}

impl<'a> ContentHash for SpreadElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.argument, state);
    }
}

impl<'a> ContentHash for Argument<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::SpreadElement(it) => ContentHash::content_hash(it, state),
            Self::BooleanLiteral(it) => ContentHash::content_hash(it, state),
            Self::NullLiteral(it) => ContentHash::content_hash(it, state),
            Self::NumericLiteral(it) => ContentHash::content_hash(it, state),
            Self::BigIntLiteral(it) => ContentHash::content_hash(it, state),
            Self::RegExpLiteral(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
            Self::TemplateLiteral(it) => ContentHash::content_hash(it, state),
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::MetaProperty(it) => ContentHash::content_hash(it, state),
            Self::Super(it) => ContentHash::content_hash(it, state),
            Self::ArrayExpression(it) => ContentHash::content_hash(it, state),
            Self::ArrowFunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::AssignmentExpression(it) => ContentHash::content_hash(it, state),
            Self::AwaitExpression(it) => ContentHash::content_hash(it, state),
            Self::BinaryExpression(it) => ContentHash::content_hash(it, state),
            Self::CallExpression(it) => ContentHash::content_hash(it, state),
            Self::ChainExpression(it) => ContentHash::content_hash(it, state),
            Self::ClassExpression(it) => ContentHash::content_hash(it, state),
            Self::ConditionalExpression(it) => ContentHash::content_hash(it, state),
            Self::FunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::ImportExpression(it) => ContentHash::content_hash(it, state),
            Self::LogicalExpression(it) => ContentHash::content_hash(it, state),
            Self::NewExpression(it) => ContentHash::content_hash(it, state),
            Self::ObjectExpression(it) => ContentHash::content_hash(it, state),
            Self::ParenthesizedExpression(it) => ContentHash::content_hash(it, state),
            Self::SequenceExpression(it) => ContentHash::content_hash(it, state),
            Self::TaggedTemplateExpression(it) => ContentHash::content_hash(it, state),
            Self::ThisExpression(it) => ContentHash::content_hash(it, state),
            Self::UnaryExpression(it) => ContentHash::content_hash(it, state),
            Self::UpdateExpression(it) => ContentHash::content_hash(it, state),
            Self::YieldExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateInExpression(it) => ContentHash::content_hash(it, state),
            Self::JSXElement(it) => ContentHash::content_hash(it, state),
            Self::JSXFragment(it) => ContentHash::content_hash(it, state),
            Self::TSAsExpression(it) => ContentHash::content_hash(it, state),
            Self::TSSatisfiesExpression(it) => ContentHash::content_hash(it, state),
            Self::TSTypeAssertion(it) => ContentHash::content_hash(it, state),
            Self::TSNonNullExpression(it) => ContentHash::content_hash(it, state),
            Self::TSInstantiationExpression(it) => ContentHash::content_hash(it, state),
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for UpdateExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.operator, state);
        ContentHash::content_hash(&self.prefix, state);
        ContentHash::content_hash(&self.argument, state);
    }
}

impl<'a> ContentHash for UnaryExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.operator, state);
        ContentHash::content_hash(&self.argument, state);
    }
}

impl<'a> ContentHash for BinaryExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.operator, state);
        ContentHash::content_hash(&self.right, state);
    }
}

impl<'a> ContentHash for PrivateInExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.operator, state);
        ContentHash::content_hash(&self.right, state);
    }
}

impl<'a> ContentHash for LogicalExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.operator, state);
        ContentHash::content_hash(&self.right, state);
    }
}

impl<'a> ContentHash for ConditionalExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.test, state);
        ContentHash::content_hash(&self.consequent, state);
        ContentHash::content_hash(&self.alternate, state);
    }
}

impl<'a> ContentHash for AssignmentExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.operator, state);
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.right, state);
    }
}

impl<'a> ContentHash for AssignmentTarget<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::AssignmentTargetIdentifier(it) => ContentHash::content_hash(it, state),
            Self::TSAsExpression(it) => ContentHash::content_hash(it, state),
            Self::TSSatisfiesExpression(it) => ContentHash::content_hash(it, state),
            Self::TSNonNullExpression(it) => ContentHash::content_hash(it, state),
            Self::TSTypeAssertion(it) => ContentHash::content_hash(it, state),
            Self::TSInstantiationExpression(it) => ContentHash::content_hash(it, state),
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
            Self::ArrayAssignmentTarget(it) => ContentHash::content_hash(it, state),
            Self::ObjectAssignmentTarget(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for SimpleAssignmentTarget<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::AssignmentTargetIdentifier(it) => ContentHash::content_hash(it, state),
            Self::TSAsExpression(it) => ContentHash::content_hash(it, state),
            Self::TSSatisfiesExpression(it) => ContentHash::content_hash(it, state),
            Self::TSNonNullExpression(it) => ContentHash::content_hash(it, state),
            Self::TSTypeAssertion(it) => ContentHash::content_hash(it, state),
            Self::TSInstantiationExpression(it) => ContentHash::content_hash(it, state),
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for AssignmentTargetPattern<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ArrayAssignmentTarget(it) => ContentHash::content_hash(it, state),
            Self::ObjectAssignmentTarget(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for ArrayAssignmentTarget<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.elements, state);
        ContentHash::content_hash(&self.rest, state);
    }
}

impl<'a> ContentHash for ObjectAssignmentTarget<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.properties, state);
        ContentHash::content_hash(&self.rest, state);
    }
}

impl<'a> ContentHash for AssignmentTargetRest<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.target, state);
    }
}

impl<'a> ContentHash for AssignmentTargetMaybeDefault<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::AssignmentTargetWithDefault(it) => ContentHash::content_hash(it, state),
            Self::AssignmentTargetIdentifier(it) => ContentHash::content_hash(it, state),
            Self::TSAsExpression(it) => ContentHash::content_hash(it, state),
            Self::TSSatisfiesExpression(it) => ContentHash::content_hash(it, state),
            Self::TSNonNullExpression(it) => ContentHash::content_hash(it, state),
            Self::TSTypeAssertion(it) => ContentHash::content_hash(it, state),
            Self::TSInstantiationExpression(it) => ContentHash::content_hash(it, state),
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
            Self::ArrayAssignmentTarget(it) => ContentHash::content_hash(it, state),
            Self::ObjectAssignmentTarget(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for AssignmentTargetWithDefault<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.binding, state);
        ContentHash::content_hash(&self.init, state);
    }
}

impl<'a> ContentHash for AssignmentTargetProperty<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => ContentHash::content_hash(it, state),
            Self::AssignmentTargetPropertyProperty(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for AssignmentTargetPropertyIdentifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.binding, state);
        ContentHash::content_hash(&self.init, state);
    }
}

impl<'a> ContentHash for AssignmentTargetPropertyProperty<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
        ContentHash::content_hash(&self.binding, state);
    }
}

impl<'a> ContentHash for SequenceExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expressions, state);
    }
}

impl ContentHash for Super {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for AwaitExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.argument, state);
    }
}

impl<'a> ContentHash for ChainExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl<'a> ContentHash for ChainElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::CallExpression(it) => ContentHash::content_hash(it, state),
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for ParenthesizedExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl<'a> ContentHash for Statement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::BlockStatement(it) => ContentHash::content_hash(it, state),
            Self::BreakStatement(it) => ContentHash::content_hash(it, state),
            Self::ContinueStatement(it) => ContentHash::content_hash(it, state),
            Self::DebuggerStatement(it) => ContentHash::content_hash(it, state),
            Self::DoWhileStatement(it) => ContentHash::content_hash(it, state),
            Self::EmptyStatement(it) => ContentHash::content_hash(it, state),
            Self::ExpressionStatement(it) => ContentHash::content_hash(it, state),
            Self::ForInStatement(it) => ContentHash::content_hash(it, state),
            Self::ForOfStatement(it) => ContentHash::content_hash(it, state),
            Self::ForStatement(it) => ContentHash::content_hash(it, state),
            Self::IfStatement(it) => ContentHash::content_hash(it, state),
            Self::LabeledStatement(it) => ContentHash::content_hash(it, state),
            Self::ReturnStatement(it) => ContentHash::content_hash(it, state),
            Self::SwitchStatement(it) => ContentHash::content_hash(it, state),
            Self::ThrowStatement(it) => ContentHash::content_hash(it, state),
            Self::TryStatement(it) => ContentHash::content_hash(it, state),
            Self::WhileStatement(it) => ContentHash::content_hash(it, state),
            Self::WithStatement(it) => ContentHash::content_hash(it, state),
            Self::VariableDeclaration(it) => ContentHash::content_hash(it, state),
            Self::FunctionDeclaration(it) => ContentHash::content_hash(it, state),
            Self::ClassDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSTypeAliasDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSInterfaceDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSEnumDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSModuleDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSImportEqualsDeclaration(it) => ContentHash::content_hash(it, state),
            Self::ImportDeclaration(it) => ContentHash::content_hash(it, state),
            Self::ExportAllDeclaration(it) => ContentHash::content_hash(it, state),
            Self::ExportDefaultDeclaration(it) => ContentHash::content_hash(it, state),
            Self::ExportNamedDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSExportAssignment(it) => ContentHash::content_hash(it, state),
            Self::TSNamespaceExportDeclaration(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for Directive<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.directive, state);
    }
}

impl<'a> ContentHash for Hashbang<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.value, state);
    }
}

impl<'a> ContentHash for BlockStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for Declaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::VariableDeclaration(it) => ContentHash::content_hash(it, state),
            Self::FunctionDeclaration(it) => ContentHash::content_hash(it, state),
            Self::ClassDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSTypeAliasDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSInterfaceDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSEnumDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSModuleDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSImportEqualsDeclaration(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for VariableDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.declarations, state);
        ContentHash::content_hash(&self.declare, state);
    }
}

impl ContentHash for VariableDeclarationKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for VariableDeclarator<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.id, state);
        ContentHash::content_hash(&self.init, state);
        ContentHash::content_hash(&self.definite, state);
    }
}

impl ContentHash for EmptyStatement {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for ExpressionStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl<'a> ContentHash for IfStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.test, state);
        ContentHash::content_hash(&self.consequent, state);
        ContentHash::content_hash(&self.alternate, state);
    }
}

impl<'a> ContentHash for DoWhileStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.body, state);
        ContentHash::content_hash(&self.test, state);
    }
}

impl<'a> ContentHash for WhileStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.test, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for ForStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.init, state);
        ContentHash::content_hash(&self.test, state);
        ContentHash::content_hash(&self.update, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for ForStatementInit<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::VariableDeclaration(it) => ContentHash::content_hash(it, state),
            Self::BooleanLiteral(it) => ContentHash::content_hash(it, state),
            Self::NullLiteral(it) => ContentHash::content_hash(it, state),
            Self::NumericLiteral(it) => ContentHash::content_hash(it, state),
            Self::BigIntLiteral(it) => ContentHash::content_hash(it, state),
            Self::RegExpLiteral(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
            Self::TemplateLiteral(it) => ContentHash::content_hash(it, state),
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::MetaProperty(it) => ContentHash::content_hash(it, state),
            Self::Super(it) => ContentHash::content_hash(it, state),
            Self::ArrayExpression(it) => ContentHash::content_hash(it, state),
            Self::ArrowFunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::AssignmentExpression(it) => ContentHash::content_hash(it, state),
            Self::AwaitExpression(it) => ContentHash::content_hash(it, state),
            Self::BinaryExpression(it) => ContentHash::content_hash(it, state),
            Self::CallExpression(it) => ContentHash::content_hash(it, state),
            Self::ChainExpression(it) => ContentHash::content_hash(it, state),
            Self::ClassExpression(it) => ContentHash::content_hash(it, state),
            Self::ConditionalExpression(it) => ContentHash::content_hash(it, state),
            Self::FunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::ImportExpression(it) => ContentHash::content_hash(it, state),
            Self::LogicalExpression(it) => ContentHash::content_hash(it, state),
            Self::NewExpression(it) => ContentHash::content_hash(it, state),
            Self::ObjectExpression(it) => ContentHash::content_hash(it, state),
            Self::ParenthesizedExpression(it) => ContentHash::content_hash(it, state),
            Self::SequenceExpression(it) => ContentHash::content_hash(it, state),
            Self::TaggedTemplateExpression(it) => ContentHash::content_hash(it, state),
            Self::ThisExpression(it) => ContentHash::content_hash(it, state),
            Self::UnaryExpression(it) => ContentHash::content_hash(it, state),
            Self::UpdateExpression(it) => ContentHash::content_hash(it, state),
            Self::YieldExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateInExpression(it) => ContentHash::content_hash(it, state),
            Self::JSXElement(it) => ContentHash::content_hash(it, state),
            Self::JSXFragment(it) => ContentHash::content_hash(it, state),
            Self::TSAsExpression(it) => ContentHash::content_hash(it, state),
            Self::TSSatisfiesExpression(it) => ContentHash::content_hash(it, state),
            Self::TSTypeAssertion(it) => ContentHash::content_hash(it, state),
            Self::TSNonNullExpression(it) => ContentHash::content_hash(it, state),
            Self::TSInstantiationExpression(it) => ContentHash::content_hash(it, state),
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for ForInStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.right, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for ForStatementLeft<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::VariableDeclaration(it) => ContentHash::content_hash(it, state),
            Self::AssignmentTargetIdentifier(it) => ContentHash::content_hash(it, state),
            Self::TSAsExpression(it) => ContentHash::content_hash(it, state),
            Self::TSSatisfiesExpression(it) => ContentHash::content_hash(it, state),
            Self::TSNonNullExpression(it) => ContentHash::content_hash(it, state),
            Self::TSTypeAssertion(it) => ContentHash::content_hash(it, state),
            Self::TSInstantiationExpression(it) => ContentHash::content_hash(it, state),
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
            Self::ArrayAssignmentTarget(it) => ContentHash::content_hash(it, state),
            Self::ObjectAssignmentTarget(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for ForOfStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.r#await, state);
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.right, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for ContinueStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.label, state);
    }
}

impl<'a> ContentHash for BreakStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.label, state);
    }
}

impl<'a> ContentHash for ReturnStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.argument, state);
    }
}

impl<'a> ContentHash for WithStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.object, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for SwitchStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.discriminant, state);
        ContentHash::content_hash(&self.cases, state);
    }
}

impl<'a> ContentHash for SwitchCase<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.test, state);
        ContentHash::content_hash(&self.consequent, state);
    }
}

impl<'a> ContentHash for LabeledStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.label, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for ThrowStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.argument, state);
    }
}

impl<'a> ContentHash for TryStatement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.block, state);
        ContentHash::content_hash(&self.handler, state);
        ContentHash::content_hash(&self.finalizer, state);
    }
}

impl<'a> ContentHash for CatchClause<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.param, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for CatchParameter<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.pattern, state);
    }
}

impl ContentHash for DebuggerStatement {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for BindingPattern<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.type_annotation, state);
        ContentHash::content_hash(&self.optional, state);
    }
}

impl<'a> ContentHash for BindingPatternKind<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::BindingIdentifier(it) => ContentHash::content_hash(it, state),
            Self::ObjectPattern(it) => ContentHash::content_hash(it, state),
            Self::ArrayPattern(it) => ContentHash::content_hash(it, state),
            Self::AssignmentPattern(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for AssignmentPattern<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.right, state);
    }
}

impl<'a> ContentHash for ObjectPattern<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.properties, state);
        ContentHash::content_hash(&self.rest, state);
    }
}

impl<'a> ContentHash for BindingProperty<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.key, state);
        ContentHash::content_hash(&self.value, state);
        ContentHash::content_hash(&self.shorthand, state);
        ContentHash::content_hash(&self.computed, state);
    }
}

impl<'a> ContentHash for ArrayPattern<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.elements, state);
        ContentHash::content_hash(&self.rest, state);
    }
}

impl<'a> ContentHash for BindingRestElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.argument, state);
    }
}

impl<'a> ContentHash for Function<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.r#type, state);
        ContentHash::content_hash(&self.id, state);
        ContentHash::content_hash(&self.generator, state);
        ContentHash::content_hash(&self.r#async, state);
        ContentHash::content_hash(&self.declare, state);
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.this_param, state);
        ContentHash::content_hash(&self.params, state);
        ContentHash::content_hash(&self.return_type, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for FunctionType {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for FormalParameters<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.items, state);
        ContentHash::content_hash(&self.rest, state);
    }
}

impl<'a> ContentHash for FormalParameter<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.decorators, state);
        ContentHash::content_hash(&self.pattern, state);
        ContentHash::content_hash(&self.accessibility, state);
        ContentHash::content_hash(&self.readonly, state);
        ContentHash::content_hash(&self.r#override, state);
    }
}

impl ContentHash for FormalParameterKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for FunctionBody<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.directives, state);
        ContentHash::content_hash(&self.statements, state);
    }
}

impl<'a> ContentHash for ArrowFunctionExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.r#async, state);
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.params, state);
        ContentHash::content_hash(&self.return_type, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for YieldExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.delegate, state);
        ContentHash::content_hash(&self.argument, state);
    }
}

impl<'a> ContentHash for Class<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.r#type, state);
        ContentHash::content_hash(&self.decorators, state);
        ContentHash::content_hash(&self.id, state);
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.super_class, state);
        ContentHash::content_hash(&self.super_type_parameters, state);
        ContentHash::content_hash(&self.implements, state);
        ContentHash::content_hash(&self.body, state);
        ContentHash::content_hash(&self.r#abstract, state);
        ContentHash::content_hash(&self.declare, state);
    }
}

impl ContentHash for ClassType {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for ClassBody<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for ClassElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::StaticBlock(it) => ContentHash::content_hash(it, state),
            Self::MethodDefinition(it) => ContentHash::content_hash(it, state),
            Self::PropertyDefinition(it) => ContentHash::content_hash(it, state),
            Self::AccessorProperty(it) => ContentHash::content_hash(it, state),
            Self::TSIndexSignature(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for MethodDefinition<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.r#type, state);
        ContentHash::content_hash(&self.decorators, state);
        ContentHash::content_hash(&self.key, state);
        ContentHash::content_hash(&self.value, state);
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.computed, state);
        ContentHash::content_hash(&self.r#static, state);
        ContentHash::content_hash(&self.r#override, state);
        ContentHash::content_hash(&self.optional, state);
        ContentHash::content_hash(&self.accessibility, state);
    }
}

impl ContentHash for MethodDefinitionType {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for PropertyDefinition<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.r#type, state);
        ContentHash::content_hash(&self.decorators, state);
        ContentHash::content_hash(&self.key, state);
        ContentHash::content_hash(&self.value, state);
        ContentHash::content_hash(&self.computed, state);
        ContentHash::content_hash(&self.r#static, state);
        ContentHash::content_hash(&self.declare, state);
        ContentHash::content_hash(&self.r#override, state);
        ContentHash::content_hash(&self.optional, state);
        ContentHash::content_hash(&self.definite, state);
        ContentHash::content_hash(&self.readonly, state);
        ContentHash::content_hash(&self.type_annotation, state);
        ContentHash::content_hash(&self.accessibility, state);
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
        ContentHash::content_hash(&self.name, state);
    }
}

impl<'a> ContentHash for StaticBlock<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for ModuleDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ImportDeclaration(it) => ContentHash::content_hash(it, state),
            Self::ExportAllDeclaration(it) => ContentHash::content_hash(it, state),
            Self::ExportDefaultDeclaration(it) => ContentHash::content_hash(it, state),
            Self::ExportNamedDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSExportAssignment(it) => ContentHash::content_hash(it, state),
            Self::TSNamespaceExportDeclaration(it) => ContentHash::content_hash(it, state),
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
        ContentHash::content_hash(&self.r#type, state);
        ContentHash::content_hash(&self.decorators, state);
        ContentHash::content_hash(&self.key, state);
        ContentHash::content_hash(&self.value, state);
        ContentHash::content_hash(&self.computed, state);
        ContentHash::content_hash(&self.r#static, state);
        ContentHash::content_hash(&self.definite, state);
        ContentHash::content_hash(&self.type_annotation, state);
        ContentHash::content_hash(&self.accessibility, state);
    }
}

impl<'a> ContentHash for ImportExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.source, state);
        ContentHash::content_hash(&self.arguments, state);
    }
}

impl<'a> ContentHash for ImportDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.specifiers, state);
        ContentHash::content_hash(&self.source, state);
        ContentHash::content_hash(&self.with_clause, state);
        ContentHash::content_hash(&self.import_kind, state);
    }
}

impl<'a> ContentHash for ImportDeclarationSpecifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ImportSpecifier(it) => ContentHash::content_hash(it, state),
            Self::ImportDefaultSpecifier(it) => ContentHash::content_hash(it, state),
            Self::ImportNamespaceSpecifier(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for ImportSpecifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.imported, state);
        ContentHash::content_hash(&self.local, state);
        ContentHash::content_hash(&self.import_kind, state);
    }
}

impl<'a> ContentHash for ImportDefaultSpecifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.local, state);
    }
}

impl<'a> ContentHash for ImportNamespaceSpecifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.local, state);
    }
}

impl<'a> ContentHash for WithClause<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.attributes_keyword, state);
        ContentHash::content_hash(&self.with_entries, state);
    }
}

impl<'a> ContentHash for ImportAttribute<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.key, state);
        ContentHash::content_hash(&self.value, state);
    }
}

impl<'a> ContentHash for ImportAttributeKey<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for ExportNamedDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.declaration, state);
        ContentHash::content_hash(&self.specifiers, state);
        ContentHash::content_hash(&self.source, state);
        ContentHash::content_hash(&self.export_kind, state);
        ContentHash::content_hash(&self.with_clause, state);
    }
}

impl<'a> ContentHash for ExportDefaultDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.declaration, state);
        ContentHash::content_hash(&self.exported, state);
    }
}

impl<'a> ContentHash for ExportAllDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.exported, state);
        ContentHash::content_hash(&self.source, state);
        ContentHash::content_hash(&self.with_clause, state);
        ContentHash::content_hash(&self.export_kind, state);
    }
}

impl<'a> ContentHash for ExportSpecifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.local, state);
        ContentHash::content_hash(&self.exported, state);
        ContentHash::content_hash(&self.export_kind, state);
    }
}

impl<'a> ContentHash for ExportDefaultDeclarationKind<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::FunctionDeclaration(it) => ContentHash::content_hash(it, state),
            Self::ClassDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSInterfaceDeclaration(it) => ContentHash::content_hash(it, state),
            Self::BooleanLiteral(it) => ContentHash::content_hash(it, state),
            Self::NullLiteral(it) => ContentHash::content_hash(it, state),
            Self::NumericLiteral(it) => ContentHash::content_hash(it, state),
            Self::BigIntLiteral(it) => ContentHash::content_hash(it, state),
            Self::RegExpLiteral(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
            Self::TemplateLiteral(it) => ContentHash::content_hash(it, state),
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::MetaProperty(it) => ContentHash::content_hash(it, state),
            Self::Super(it) => ContentHash::content_hash(it, state),
            Self::ArrayExpression(it) => ContentHash::content_hash(it, state),
            Self::ArrowFunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::AssignmentExpression(it) => ContentHash::content_hash(it, state),
            Self::AwaitExpression(it) => ContentHash::content_hash(it, state),
            Self::BinaryExpression(it) => ContentHash::content_hash(it, state),
            Self::CallExpression(it) => ContentHash::content_hash(it, state),
            Self::ChainExpression(it) => ContentHash::content_hash(it, state),
            Self::ClassExpression(it) => ContentHash::content_hash(it, state),
            Self::ConditionalExpression(it) => ContentHash::content_hash(it, state),
            Self::FunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::ImportExpression(it) => ContentHash::content_hash(it, state),
            Self::LogicalExpression(it) => ContentHash::content_hash(it, state),
            Self::NewExpression(it) => ContentHash::content_hash(it, state),
            Self::ObjectExpression(it) => ContentHash::content_hash(it, state),
            Self::ParenthesizedExpression(it) => ContentHash::content_hash(it, state),
            Self::SequenceExpression(it) => ContentHash::content_hash(it, state),
            Self::TaggedTemplateExpression(it) => ContentHash::content_hash(it, state),
            Self::ThisExpression(it) => ContentHash::content_hash(it, state),
            Self::UnaryExpression(it) => ContentHash::content_hash(it, state),
            Self::UpdateExpression(it) => ContentHash::content_hash(it, state),
            Self::YieldExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateInExpression(it) => ContentHash::content_hash(it, state),
            Self::JSXElement(it) => ContentHash::content_hash(it, state),
            Self::JSXFragment(it) => ContentHash::content_hash(it, state),
            Self::TSAsExpression(it) => ContentHash::content_hash(it, state),
            Self::TSSatisfiesExpression(it) => ContentHash::content_hash(it, state),
            Self::TSTypeAssertion(it) => ContentHash::content_hash(it, state),
            Self::TSNonNullExpression(it) => ContentHash::content_hash(it, state),
            Self::TSInstantiationExpression(it) => ContentHash::content_hash(it, state),
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for ModuleExportName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::IdentifierName(it) => ContentHash::content_hash(it, state),
            Self::IdentifierReference(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for TSThisParameter<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl<'a> ContentHash for TSEnumDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.id, state);
        ContentHash::content_hash(&self.members, state);
        ContentHash::content_hash(&self.r#const, state);
        ContentHash::content_hash(&self.declare, state);
    }
}

impl<'a> ContentHash for TSEnumMember<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.id, state);
        ContentHash::content_hash(&self.initializer, state);
    }
}

impl<'a> ContentHash for TSEnumMemberName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::StaticIdentifier(it) => ContentHash::content_hash(it, state),
            Self::StaticStringLiteral(it) => ContentHash::content_hash(it, state),
            Self::StaticTemplateLiteral(it) => ContentHash::content_hash(it, state),
            Self::StaticNumericLiteral(it) => ContentHash::content_hash(it, state),
            Self::BooleanLiteral(it) => ContentHash::content_hash(it, state),
            Self::NullLiteral(it) => ContentHash::content_hash(it, state),
            Self::NumericLiteral(it) => ContentHash::content_hash(it, state),
            Self::BigIntLiteral(it) => ContentHash::content_hash(it, state),
            Self::RegExpLiteral(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
            Self::TemplateLiteral(it) => ContentHash::content_hash(it, state),
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::MetaProperty(it) => ContentHash::content_hash(it, state),
            Self::Super(it) => ContentHash::content_hash(it, state),
            Self::ArrayExpression(it) => ContentHash::content_hash(it, state),
            Self::ArrowFunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::AssignmentExpression(it) => ContentHash::content_hash(it, state),
            Self::AwaitExpression(it) => ContentHash::content_hash(it, state),
            Self::BinaryExpression(it) => ContentHash::content_hash(it, state),
            Self::CallExpression(it) => ContentHash::content_hash(it, state),
            Self::ChainExpression(it) => ContentHash::content_hash(it, state),
            Self::ClassExpression(it) => ContentHash::content_hash(it, state),
            Self::ConditionalExpression(it) => ContentHash::content_hash(it, state),
            Self::FunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::ImportExpression(it) => ContentHash::content_hash(it, state),
            Self::LogicalExpression(it) => ContentHash::content_hash(it, state),
            Self::NewExpression(it) => ContentHash::content_hash(it, state),
            Self::ObjectExpression(it) => ContentHash::content_hash(it, state),
            Self::ParenthesizedExpression(it) => ContentHash::content_hash(it, state),
            Self::SequenceExpression(it) => ContentHash::content_hash(it, state),
            Self::TaggedTemplateExpression(it) => ContentHash::content_hash(it, state),
            Self::ThisExpression(it) => ContentHash::content_hash(it, state),
            Self::UnaryExpression(it) => ContentHash::content_hash(it, state),
            Self::UpdateExpression(it) => ContentHash::content_hash(it, state),
            Self::YieldExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateInExpression(it) => ContentHash::content_hash(it, state),
            Self::JSXElement(it) => ContentHash::content_hash(it, state),
            Self::JSXFragment(it) => ContentHash::content_hash(it, state),
            Self::TSAsExpression(it) => ContentHash::content_hash(it, state),
            Self::TSSatisfiesExpression(it) => ContentHash::content_hash(it, state),
            Self::TSTypeAssertion(it) => ContentHash::content_hash(it, state),
            Self::TSNonNullExpression(it) => ContentHash::content_hash(it, state),
            Self::TSInstantiationExpression(it) => ContentHash::content_hash(it, state),
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for TSTypeAnnotation<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl<'a> ContentHash for TSLiteralType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.literal, state);
    }
}

impl<'a> ContentHash for TSLiteral<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::BooleanLiteral(it) => ContentHash::content_hash(it, state),
            Self::NullLiteral(it) => ContentHash::content_hash(it, state),
            Self::NumericLiteral(it) => ContentHash::content_hash(it, state),
            Self::BigIntLiteral(it) => ContentHash::content_hash(it, state),
            Self::RegExpLiteral(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
            Self::TemplateLiteral(it) => ContentHash::content_hash(it, state),
            Self::UnaryExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for TSType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::TSAnyKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSBigIntKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSBooleanKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSIntrinsicKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSNeverKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSNullKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSNumberKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSObjectKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSStringKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSSymbolKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSUndefinedKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSUnknownKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSVoidKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSArrayType(it) => ContentHash::content_hash(it, state),
            Self::TSConditionalType(it) => ContentHash::content_hash(it, state),
            Self::TSConstructorType(it) => ContentHash::content_hash(it, state),
            Self::TSFunctionType(it) => ContentHash::content_hash(it, state),
            Self::TSImportType(it) => ContentHash::content_hash(it, state),
            Self::TSIndexedAccessType(it) => ContentHash::content_hash(it, state),
            Self::TSInferType(it) => ContentHash::content_hash(it, state),
            Self::TSIntersectionType(it) => ContentHash::content_hash(it, state),
            Self::TSLiteralType(it) => ContentHash::content_hash(it, state),
            Self::TSMappedType(it) => ContentHash::content_hash(it, state),
            Self::TSNamedTupleMember(it) => ContentHash::content_hash(it, state),
            Self::TSQualifiedName(it) => ContentHash::content_hash(it, state),
            Self::TSTemplateLiteralType(it) => ContentHash::content_hash(it, state),
            Self::TSThisType(it) => ContentHash::content_hash(it, state),
            Self::TSTupleType(it) => ContentHash::content_hash(it, state),
            Self::TSTypeLiteral(it) => ContentHash::content_hash(it, state),
            Self::TSTypeOperatorType(it) => ContentHash::content_hash(it, state),
            Self::TSTypePredicate(it) => ContentHash::content_hash(it, state),
            Self::TSTypeQuery(it) => ContentHash::content_hash(it, state),
            Self::TSTypeReference(it) => ContentHash::content_hash(it, state),
            Self::TSUnionType(it) => ContentHash::content_hash(it, state),
            Self::TSParenthesizedType(it) => ContentHash::content_hash(it, state),
            Self::JSDocNullableType(it) => ContentHash::content_hash(it, state),
            Self::JSDocNonNullableType(it) => ContentHash::content_hash(it, state),
            Self::JSDocUnknownType(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for TSConditionalType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.check_type, state);
        ContentHash::content_hash(&self.extends_type, state);
        ContentHash::content_hash(&self.true_type, state);
        ContentHash::content_hash(&self.false_type, state);
    }
}

impl<'a> ContentHash for TSUnionType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.types, state);
    }
}

impl<'a> ContentHash for TSIntersectionType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.types, state);
    }
}

impl<'a> ContentHash for TSParenthesizedType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl<'a> ContentHash for TSTypeOperator<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.operator, state);
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl ContentHash for TSTypeOperatorOperator {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for TSArrayType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.element_type, state);
    }
}

impl<'a> ContentHash for TSIndexedAccessType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.object_type, state);
        ContentHash::content_hash(&self.index_type, state);
    }
}

impl<'a> ContentHash for TSTupleType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.element_types, state);
    }
}

impl<'a> ContentHash for TSNamedTupleMember<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.element_type, state);
        ContentHash::content_hash(&self.label, state);
        ContentHash::content_hash(&self.optional, state);
    }
}

impl<'a> ContentHash for TSOptionalType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl<'a> ContentHash for TSRestType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl<'a> ContentHash for TSTupleElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::TSOptionalType(it) => ContentHash::content_hash(it, state),
            Self::TSRestType(it) => ContentHash::content_hash(it, state),
            Self::TSAnyKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSBigIntKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSBooleanKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSIntrinsicKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSNeverKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSNullKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSNumberKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSObjectKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSStringKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSSymbolKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSUndefinedKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSUnknownKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSVoidKeyword(it) => ContentHash::content_hash(it, state),
            Self::TSArrayType(it) => ContentHash::content_hash(it, state),
            Self::TSConditionalType(it) => ContentHash::content_hash(it, state),
            Self::TSConstructorType(it) => ContentHash::content_hash(it, state),
            Self::TSFunctionType(it) => ContentHash::content_hash(it, state),
            Self::TSImportType(it) => ContentHash::content_hash(it, state),
            Self::TSIndexedAccessType(it) => ContentHash::content_hash(it, state),
            Self::TSInferType(it) => ContentHash::content_hash(it, state),
            Self::TSIntersectionType(it) => ContentHash::content_hash(it, state),
            Self::TSLiteralType(it) => ContentHash::content_hash(it, state),
            Self::TSMappedType(it) => ContentHash::content_hash(it, state),
            Self::TSNamedTupleMember(it) => ContentHash::content_hash(it, state),
            Self::TSQualifiedName(it) => ContentHash::content_hash(it, state),
            Self::TSTemplateLiteralType(it) => ContentHash::content_hash(it, state),
            Self::TSThisType(it) => ContentHash::content_hash(it, state),
            Self::TSTupleType(it) => ContentHash::content_hash(it, state),
            Self::TSTypeLiteral(it) => ContentHash::content_hash(it, state),
            Self::TSTypeOperatorType(it) => ContentHash::content_hash(it, state),
            Self::TSTypePredicate(it) => ContentHash::content_hash(it, state),
            Self::TSTypeQuery(it) => ContentHash::content_hash(it, state),
            Self::TSTypeReference(it) => ContentHash::content_hash(it, state),
            Self::TSUnionType(it) => ContentHash::content_hash(it, state),
            Self::TSParenthesizedType(it) => ContentHash::content_hash(it, state),
            Self::JSDocNullableType(it) => ContentHash::content_hash(it, state),
            Self::JSDocNonNullableType(it) => ContentHash::content_hash(it, state),
            Self::JSDocUnknownType(it) => ContentHash::content_hash(it, state),
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
        ContentHash::content_hash(&self.type_name, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl<'a> ContentHash for TSTypeName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::IdentifierReference(it) => ContentHash::content_hash(it, state),
            Self::QualifiedName(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for TSQualifiedName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.right, state);
    }
}

impl<'a> ContentHash for TSTypeParameterInstantiation<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.params, state);
    }
}

impl<'a> ContentHash for TSTypeParameter<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
        ContentHash::content_hash(&self.constraint, state);
        ContentHash::content_hash(&self.default, state);
        ContentHash::content_hash(&self.r#in, state);
        ContentHash::content_hash(&self.out, state);
        ContentHash::content_hash(&self.r#const, state);
    }
}

impl<'a> ContentHash for TSTypeParameterDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.params, state);
    }
}

impl<'a> ContentHash for TSTypeAliasDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.id, state);
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.type_annotation, state);
        ContentHash::content_hash(&self.declare, state);
    }
}

impl ContentHash for TSAccessibility {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for TSClassImplements<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl<'a> ContentHash for TSInterfaceDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.id, state);
        ContentHash::content_hash(&self.extends, state);
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.body, state);
        ContentHash::content_hash(&self.declare, state);
    }
}

impl<'a> ContentHash for TSInterfaceBody<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for TSPropertySignature<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.computed, state);
        ContentHash::content_hash(&self.optional, state);
        ContentHash::content_hash(&self.readonly, state);
        ContentHash::content_hash(&self.key, state);
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl<'a> ContentHash for TSSignature<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::TSIndexSignature(it) => ContentHash::content_hash(it, state),
            Self::TSPropertySignature(it) => ContentHash::content_hash(it, state),
            Self::TSCallSignatureDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSConstructSignatureDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSMethodSignature(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for TSIndexSignature<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.parameters, state);
        ContentHash::content_hash(&self.type_annotation, state);
        ContentHash::content_hash(&self.readonly, state);
    }
}

impl<'a> ContentHash for TSCallSignatureDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.this_param, state);
        ContentHash::content_hash(&self.params, state);
        ContentHash::content_hash(&self.return_type, state);
    }
}

impl ContentHash for TSMethodSignatureKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for TSMethodSignature<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.key, state);
        ContentHash::content_hash(&self.computed, state);
        ContentHash::content_hash(&self.optional, state);
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.this_param, state);
        ContentHash::content_hash(&self.params, state);
        ContentHash::content_hash(&self.return_type, state);
    }
}

impl<'a> ContentHash for TSConstructSignatureDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.params, state);
        ContentHash::content_hash(&self.return_type, state);
    }
}

impl<'a> ContentHash for TSIndexSignatureName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl<'a> ContentHash for TSInterfaceHeritage<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl<'a> ContentHash for TSTypePredicate<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.parameter_name, state);
        ContentHash::content_hash(&self.asserts, state);
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl<'a> ContentHash for TSTypePredicateName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::This(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for TSModuleDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.id, state);
        ContentHash::content_hash(&self.body, state);
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.declare, state);
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
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for TSModuleDeclarationBody<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::TSModuleDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSModuleBlock(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for TSModuleBlock<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.directives, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for TSTypeLiteral<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.members, state);
    }
}

impl<'a> ContentHash for TSInferType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_parameter, state);
    }
}

impl<'a> ContentHash for TSTypeQuery<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expr_name, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl<'a> ContentHash for TSTypeQueryExprName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::TSImportType(it) => ContentHash::content_hash(it, state),
            Self::IdentifierReference(it) => ContentHash::content_hash(it, state),
            Self::QualifiedName(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for TSImportType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.is_type_of, state);
        ContentHash::content_hash(&self.parameter, state);
        ContentHash::content_hash(&self.qualifier, state);
        ContentHash::content_hash(&self.attributes, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl<'a> ContentHash for TSImportAttributes<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.attributes_keyword, state);
        ContentHash::content_hash(&self.elements, state);
    }
}

impl<'a> ContentHash for TSImportAttribute<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
        ContentHash::content_hash(&self.value, state);
    }
}

impl<'a> ContentHash for TSImportAttributeName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for TSFunctionType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.this_param, state);
        ContentHash::content_hash(&self.params, state);
        ContentHash::content_hash(&self.return_type, state);
    }
}

impl<'a> ContentHash for TSConstructorType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.r#abstract, state);
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.params, state);
        ContentHash::content_hash(&self.return_type, state);
    }
}

impl<'a> ContentHash for TSMappedType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_parameter, state);
        ContentHash::content_hash(&self.name_type, state);
        ContentHash::content_hash(&self.type_annotation, state);
        ContentHash::content_hash(&self.optional, state);
        ContentHash::content_hash(&self.readonly, state);
    }
}

impl ContentHash for TSMappedTypeModifierOperator {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for TSTemplateLiteralType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.quasis, state);
        ContentHash::content_hash(&self.types, state);
    }
}

impl<'a> ContentHash for TSAsExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl<'a> ContentHash for TSSatisfiesExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl<'a> ContentHash for TSTypeAssertion<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl<'a> ContentHash for TSImportEqualsDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.id, state);
        ContentHash::content_hash(&self.module_reference, state);
        ContentHash::content_hash(&self.import_kind, state);
    }
}

impl<'a> ContentHash for TSModuleReference<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ExternalModuleReference(it) => ContentHash::content_hash(it, state),
            Self::IdentifierReference(it) => ContentHash::content_hash(it, state),
            Self::QualifiedName(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for TSExternalModuleReference<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl<'a> ContentHash for TSNonNullExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl<'a> ContentHash for Decorator<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl<'a> ContentHash for TSExportAssignment<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl<'a> ContentHash for TSNamespaceExportDeclaration<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.id, state);
    }
}

impl<'a> ContentHash for TSInstantiationExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl ContentHash for ImportOrExportKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for JSDocNullableType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_annotation, state);
        ContentHash::content_hash(&self.postfix, state);
    }
}

impl<'a> ContentHash for JSDocNonNullableType<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_annotation, state);
        ContentHash::content_hash(&self.postfix, state);
    }
}

impl ContentHash for JSDocUnknownType {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for JSXElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.opening_element, state);
        ContentHash::content_hash(&self.closing_element, state);
        ContentHash::content_hash(&self.children, state);
    }
}

impl<'a> ContentHash for JSXOpeningElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.self_closing, state);
        ContentHash::content_hash(&self.name, state);
        ContentHash::content_hash(&self.attributes, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl<'a> ContentHash for JSXClosingElement<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
    }
}

impl<'a> ContentHash for JSXFragment<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.opening_fragment, state);
        ContentHash::content_hash(&self.closing_fragment, state);
        ContentHash::content_hash(&self.children, state);
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
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::IdentifierReference(it) => ContentHash::content_hash(it, state),
            Self::NamespacedName(it) => ContentHash::content_hash(it, state),
            Self::MemberExpression(it) => ContentHash::content_hash(it, state),
            Self::ThisExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for JSXNamespacedName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.namespace, state);
        ContentHash::content_hash(&self.property, state);
    }
}

impl<'a> ContentHash for JSXMemberExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.object, state);
        ContentHash::content_hash(&self.property, state);
    }
}

impl<'a> ContentHash for JSXMemberExpressionObject<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::IdentifierReference(it) => ContentHash::content_hash(it, state),
            Self::MemberExpression(it) => ContentHash::content_hash(it, state),
            Self::ThisExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for JSXExpressionContainer<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl<'a> ContentHash for JSXExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::EmptyExpression(it) => ContentHash::content_hash(it, state),
            Self::BooleanLiteral(it) => ContentHash::content_hash(it, state),
            Self::NullLiteral(it) => ContentHash::content_hash(it, state),
            Self::NumericLiteral(it) => ContentHash::content_hash(it, state),
            Self::BigIntLiteral(it) => ContentHash::content_hash(it, state),
            Self::RegExpLiteral(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
            Self::TemplateLiteral(it) => ContentHash::content_hash(it, state),
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::MetaProperty(it) => ContentHash::content_hash(it, state),
            Self::Super(it) => ContentHash::content_hash(it, state),
            Self::ArrayExpression(it) => ContentHash::content_hash(it, state),
            Self::ArrowFunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::AssignmentExpression(it) => ContentHash::content_hash(it, state),
            Self::AwaitExpression(it) => ContentHash::content_hash(it, state),
            Self::BinaryExpression(it) => ContentHash::content_hash(it, state),
            Self::CallExpression(it) => ContentHash::content_hash(it, state),
            Self::ChainExpression(it) => ContentHash::content_hash(it, state),
            Self::ClassExpression(it) => ContentHash::content_hash(it, state),
            Self::ConditionalExpression(it) => ContentHash::content_hash(it, state),
            Self::FunctionExpression(it) => ContentHash::content_hash(it, state),
            Self::ImportExpression(it) => ContentHash::content_hash(it, state),
            Self::LogicalExpression(it) => ContentHash::content_hash(it, state),
            Self::NewExpression(it) => ContentHash::content_hash(it, state),
            Self::ObjectExpression(it) => ContentHash::content_hash(it, state),
            Self::ParenthesizedExpression(it) => ContentHash::content_hash(it, state),
            Self::SequenceExpression(it) => ContentHash::content_hash(it, state),
            Self::TaggedTemplateExpression(it) => ContentHash::content_hash(it, state),
            Self::ThisExpression(it) => ContentHash::content_hash(it, state),
            Self::UnaryExpression(it) => ContentHash::content_hash(it, state),
            Self::UpdateExpression(it) => ContentHash::content_hash(it, state),
            Self::YieldExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateInExpression(it) => ContentHash::content_hash(it, state),
            Self::JSXElement(it) => ContentHash::content_hash(it, state),
            Self::JSXFragment(it) => ContentHash::content_hash(it, state),
            Self::TSAsExpression(it) => ContentHash::content_hash(it, state),
            Self::TSSatisfiesExpression(it) => ContentHash::content_hash(it, state),
            Self::TSTypeAssertion(it) => ContentHash::content_hash(it, state),
            Self::TSNonNullExpression(it) => ContentHash::content_hash(it, state),
            Self::TSInstantiationExpression(it) => ContentHash::content_hash(it, state),
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
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
            Self::Attribute(it) => ContentHash::content_hash(it, state),
            Self::SpreadAttribute(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for JSXAttribute<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
        ContentHash::content_hash(&self.value, state);
    }
}

impl<'a> ContentHash for JSXSpreadAttribute<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.argument, state);
    }
}

impl<'a> ContentHash for JSXAttributeName<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::NamespacedName(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for JSXAttributeValue<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
            Self::ExpressionContainer(it) => ContentHash::content_hash(it, state),
            Self::Element(it) => ContentHash::content_hash(it, state),
            Self::Fragment(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for JSXIdentifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
    }
}

impl<'a> ContentHash for JSXChild<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Text(it) => ContentHash::content_hash(it, state),
            Self::Element(it) => ContentHash::content_hash(it, state),
            Self::Fragment(it) => ContentHash::content_hash(it, state),
            Self::ExpressionContainer(it) => ContentHash::content_hash(it, state),
            Self::Spread(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl<'a> ContentHash for JSXSpreadChild<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl<'a> ContentHash for JSXText<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.value, state);
    }
}

impl ContentHash for CommentKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl ContentHash for CommentPosition {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl ContentHash for Comment {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.position, state);
        ContentHash::content_hash(&self.attached_to, state);
        ContentHash::content_hash(&self.preceded_by_newline, state);
        ContentHash::content_hash(&self.followed_by_newline, state);
    }
}
