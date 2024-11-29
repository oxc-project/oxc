// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/content_hash.rs`

#![allow(clippy::match_same_arms)]

use std::{hash::Hasher, mem::discriminant};

use oxc_span::hash::ContentHash;

use crate::ast::comment::*;

use crate::ast::js::*;

use crate::ast::jsx::*;

use crate::ast::literal::*;

use crate::ast::ts::*;

impl ContentHash for BooleanLiteral {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.value, state);
    }
}

impl ContentHash for StringLiteral<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.value, state);
        ContentHash::content_hash(&self.raw, state);
    }
}

impl ContentHash for BigIntLiteral<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.raw, state);
        ContentHash::content_hash(&self.base, state);
    }
}

impl ContentHash for RegExpLiteral<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.regex, state);
        ContentHash::content_hash(&self.raw, state);
    }
}

impl ContentHash for RegExp<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.pattern, state);
        ContentHash::content_hash(&self.flags, state);
    }
}

impl ContentHash for RegExpPattern<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Raw(it) => ContentHash::content_hash(it, state),
            Self::Invalid(it) => ContentHash::content_hash(it, state),
            Self::Pattern(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for Program<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.source_type, state);
        ContentHash::content_hash(&self.source_text, state);
        ContentHash::content_hash(&self.comments, state);
        ContentHash::content_hash(&self.hashbang, state);
        ContentHash::content_hash(&self.directives, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for Expression<'_> {
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

impl ContentHash for IdentifierName<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
    }
}

impl ContentHash for IdentifierReference<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
    }
}

impl ContentHash for BindingIdentifier<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
    }
}

impl ContentHash for LabelIdentifier<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
    }
}

impl ContentHash for ThisExpression {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for ArrayExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.elements, state);
    }
}

impl ContentHash for ArrayExpressionElement<'_> {
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

impl ContentHash for ObjectExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.properties, state);
    }
}

impl ContentHash for ObjectPropertyKind<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ObjectProperty(it) => ContentHash::content_hash(it, state),
            Self::SpreadProperty(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for ObjectProperty<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.key, state);
        ContentHash::content_hash(&self.value, state);
        ContentHash::content_hash(&self.method, state);
        ContentHash::content_hash(&self.shorthand, state);
        ContentHash::content_hash(&self.computed, state);
    }
}

impl ContentHash for PropertyKey<'_> {
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

impl ContentHash for TemplateLiteral<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.quasis, state);
        ContentHash::content_hash(&self.expressions, state);
    }
}

impl ContentHash for TaggedTemplateExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.tag, state);
        ContentHash::content_hash(&self.quasi, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl ContentHash for TemplateElement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.tail, state);
        ContentHash::content_hash(&self.value, state);
    }
}

impl ContentHash for TemplateElementValue<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.raw, state);
        ContentHash::content_hash(&self.cooked, state);
    }
}

impl ContentHash for MemberExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for ComputedMemberExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.object, state);
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.optional, state);
    }
}

impl ContentHash for StaticMemberExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.object, state);
        ContentHash::content_hash(&self.property, state);
        ContentHash::content_hash(&self.optional, state);
    }
}

impl ContentHash for PrivateFieldExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.object, state);
        ContentHash::content_hash(&self.field, state);
        ContentHash::content_hash(&self.optional, state);
    }
}

impl ContentHash for CallExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.callee, state);
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.arguments, state);
        ContentHash::content_hash(&self.optional, state);
    }
}

impl ContentHash for NewExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.callee, state);
        ContentHash::content_hash(&self.arguments, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl ContentHash for MetaProperty<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.meta, state);
        ContentHash::content_hash(&self.property, state);
    }
}

impl ContentHash for SpreadElement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.argument, state);
    }
}

impl ContentHash for Argument<'_> {
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

impl ContentHash for UpdateExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.operator, state);
        ContentHash::content_hash(&self.prefix, state);
        ContentHash::content_hash(&self.argument, state);
    }
}

impl ContentHash for UnaryExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.operator, state);
        ContentHash::content_hash(&self.argument, state);
    }
}

impl ContentHash for BinaryExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.operator, state);
        ContentHash::content_hash(&self.right, state);
    }
}

impl ContentHash for PrivateInExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.operator, state);
        ContentHash::content_hash(&self.right, state);
    }
}

impl ContentHash for LogicalExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.operator, state);
        ContentHash::content_hash(&self.right, state);
    }
}

impl ContentHash for ConditionalExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.test, state);
        ContentHash::content_hash(&self.consequent, state);
        ContentHash::content_hash(&self.alternate, state);
    }
}

impl ContentHash for AssignmentExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.operator, state);
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.right, state);
    }
}

impl ContentHash for AssignmentTarget<'_> {
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

impl ContentHash for SimpleAssignmentTarget<'_> {
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

impl ContentHash for AssignmentTargetPattern<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ArrayAssignmentTarget(it) => ContentHash::content_hash(it, state),
            Self::ObjectAssignmentTarget(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for ArrayAssignmentTarget<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.elements, state);
        ContentHash::content_hash(&self.rest, state);
    }
}

impl ContentHash for ObjectAssignmentTarget<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.properties, state);
        ContentHash::content_hash(&self.rest, state);
    }
}

impl ContentHash for AssignmentTargetRest<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.target, state);
    }
}

impl ContentHash for AssignmentTargetMaybeDefault<'_> {
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

impl ContentHash for AssignmentTargetWithDefault<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.binding, state);
        ContentHash::content_hash(&self.init, state);
    }
}

impl ContentHash for AssignmentTargetProperty<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => ContentHash::content_hash(it, state),
            Self::AssignmentTargetPropertyProperty(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for AssignmentTargetPropertyIdentifier<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.binding, state);
        ContentHash::content_hash(&self.init, state);
    }
}

impl ContentHash for AssignmentTargetPropertyProperty<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
        ContentHash::content_hash(&self.binding, state);
    }
}

impl ContentHash for SequenceExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expressions, state);
    }
}

impl ContentHash for Super {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for AwaitExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.argument, state);
    }
}

impl ContentHash for ChainExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl ContentHash for ChainElement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::CallExpression(it) => ContentHash::content_hash(it, state),
            Self::TSNonNullExpression(it) => ContentHash::content_hash(it, state),
            Self::ComputedMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::StaticMemberExpression(it) => ContentHash::content_hash(it, state),
            Self::PrivateFieldExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for ParenthesizedExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl ContentHash for Statement<'_> {
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

impl ContentHash for Directive<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.directive, state);
    }
}

impl ContentHash for Hashbang<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.value, state);
    }
}

impl ContentHash for BlockStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for Declaration<'_> {
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

impl ContentHash for VariableDeclaration<'_> {
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

impl ContentHash for VariableDeclarator<'_> {
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

impl ContentHash for ExpressionStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl ContentHash for IfStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.test, state);
        ContentHash::content_hash(&self.consequent, state);
        ContentHash::content_hash(&self.alternate, state);
    }
}

impl ContentHash for DoWhileStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.body, state);
        ContentHash::content_hash(&self.test, state);
    }
}

impl ContentHash for WhileStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.test, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for ForStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.init, state);
        ContentHash::content_hash(&self.test, state);
        ContentHash::content_hash(&self.update, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for ForStatementInit<'_> {
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

impl ContentHash for ForInStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.right, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for ForStatementLeft<'_> {
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

impl ContentHash for ForOfStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.r#await, state);
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.right, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for ContinueStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.label, state);
    }
}

impl ContentHash for BreakStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.label, state);
    }
}

impl ContentHash for ReturnStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.argument, state);
    }
}

impl ContentHash for WithStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.object, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for SwitchStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.discriminant, state);
        ContentHash::content_hash(&self.cases, state);
    }
}

impl ContentHash for SwitchCase<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.test, state);
        ContentHash::content_hash(&self.consequent, state);
    }
}

impl ContentHash for LabeledStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.label, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for ThrowStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.argument, state);
    }
}

impl ContentHash for TryStatement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.block, state);
        ContentHash::content_hash(&self.handler, state);
        ContentHash::content_hash(&self.finalizer, state);
    }
}

impl ContentHash for CatchClause<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.param, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for CatchParameter<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.pattern, state);
    }
}

impl ContentHash for DebuggerStatement {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for BindingPattern<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.type_annotation, state);
        ContentHash::content_hash(&self.optional, state);
    }
}

impl ContentHash for BindingPatternKind<'_> {
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

impl ContentHash for AssignmentPattern<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.right, state);
    }
}

impl ContentHash for ObjectPattern<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.properties, state);
        ContentHash::content_hash(&self.rest, state);
    }
}

impl ContentHash for BindingProperty<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.key, state);
        ContentHash::content_hash(&self.value, state);
        ContentHash::content_hash(&self.shorthand, state);
        ContentHash::content_hash(&self.computed, state);
    }
}

impl ContentHash for ArrayPattern<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.elements, state);
        ContentHash::content_hash(&self.rest, state);
    }
}

impl ContentHash for BindingRestElement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.argument, state);
    }
}

impl ContentHash for Function<'_> {
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

impl ContentHash for FormalParameters<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.items, state);
        ContentHash::content_hash(&self.rest, state);
    }
}

impl ContentHash for FormalParameter<'_> {
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

impl ContentHash for FunctionBody<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.directives, state);
        ContentHash::content_hash(&self.statements, state);
    }
}

impl ContentHash for ArrowFunctionExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.r#async, state);
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.params, state);
        ContentHash::content_hash(&self.return_type, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for YieldExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.delegate, state);
        ContentHash::content_hash(&self.argument, state);
    }
}

impl ContentHash for Class<'_> {
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

impl ContentHash for ClassBody<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for ClassElement<'_> {
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

impl ContentHash for MethodDefinition<'_> {
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

impl ContentHash for PropertyDefinition<'_> {
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

impl ContentHash for PrivateIdentifier<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
    }
}

impl ContentHash for StaticBlock<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for ModuleDeclaration<'_> {
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

impl ContentHash for AccessorProperty<'_> {
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

impl ContentHash for ImportExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.source, state);
        ContentHash::content_hash(&self.arguments, state);
    }
}

impl ContentHash for ImportDeclaration<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.specifiers, state);
        ContentHash::content_hash(&self.source, state);
        ContentHash::content_hash(&self.with_clause, state);
        ContentHash::content_hash(&self.import_kind, state);
    }
}

impl ContentHash for ImportDeclarationSpecifier<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ImportSpecifier(it) => ContentHash::content_hash(it, state),
            Self::ImportDefaultSpecifier(it) => ContentHash::content_hash(it, state),
            Self::ImportNamespaceSpecifier(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for ImportSpecifier<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.imported, state);
        ContentHash::content_hash(&self.local, state);
        ContentHash::content_hash(&self.import_kind, state);
    }
}

impl ContentHash for ImportDefaultSpecifier<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.local, state);
    }
}

impl ContentHash for ImportNamespaceSpecifier<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.local, state);
    }
}

impl ContentHash for WithClause<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.attributes_keyword, state);
        ContentHash::content_hash(&self.with_entries, state);
    }
}

impl ContentHash for ImportAttribute<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.key, state);
        ContentHash::content_hash(&self.value, state);
    }
}

impl ContentHash for ImportAttributeKey<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for ExportNamedDeclaration<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.declaration, state);
        ContentHash::content_hash(&self.specifiers, state);
        ContentHash::content_hash(&self.source, state);
        ContentHash::content_hash(&self.export_kind, state);
        ContentHash::content_hash(&self.with_clause, state);
    }
}

impl ContentHash for ExportDefaultDeclaration<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.declaration, state);
        ContentHash::content_hash(&self.exported, state);
    }
}

impl ContentHash for ExportAllDeclaration<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.exported, state);
        ContentHash::content_hash(&self.source, state);
        ContentHash::content_hash(&self.with_clause, state);
        ContentHash::content_hash(&self.export_kind, state);
    }
}

impl ContentHash for ExportSpecifier<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.local, state);
        ContentHash::content_hash(&self.exported, state);
        ContentHash::content_hash(&self.export_kind, state);
    }
}

impl ContentHash for ExportDefaultDeclarationKind<'_> {
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

impl ContentHash for ModuleExportName<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::IdentifierName(it) => ContentHash::content_hash(it, state),
            Self::IdentifierReference(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for TSThisParameter<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl ContentHash for TSEnumDeclaration<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.id, state);
        ContentHash::content_hash(&self.members, state);
        ContentHash::content_hash(&self.r#const, state);
        ContentHash::content_hash(&self.declare, state);
    }
}

impl ContentHash for TSEnumMember<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.id, state);
        ContentHash::content_hash(&self.initializer, state);
    }
}

impl ContentHash for TSEnumMemberName<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::String(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for TSTypeAnnotation<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl ContentHash for TSLiteralType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.literal, state);
    }
}

impl ContentHash for TSLiteral<'_> {
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

impl ContentHash for TSType<'_> {
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

impl ContentHash for TSConditionalType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.check_type, state);
        ContentHash::content_hash(&self.extends_type, state);
        ContentHash::content_hash(&self.true_type, state);
        ContentHash::content_hash(&self.false_type, state);
    }
}

impl ContentHash for TSUnionType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.types, state);
    }
}

impl ContentHash for TSIntersectionType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.types, state);
    }
}

impl ContentHash for TSParenthesizedType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl ContentHash for TSTypeOperator<'_> {
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

impl ContentHash for TSArrayType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.element_type, state);
    }
}

impl ContentHash for TSIndexedAccessType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.object_type, state);
        ContentHash::content_hash(&self.index_type, state);
    }
}

impl ContentHash for TSTupleType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.element_types, state);
    }
}

impl ContentHash for TSNamedTupleMember<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.element_type, state);
        ContentHash::content_hash(&self.label, state);
        ContentHash::content_hash(&self.optional, state);
    }
}

impl ContentHash for TSOptionalType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl ContentHash for TSRestType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl ContentHash for TSTupleElement<'_> {
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

impl ContentHash for TSTypeReference<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_name, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl ContentHash for TSTypeName<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::IdentifierReference(it) => ContentHash::content_hash(it, state),
            Self::QualifiedName(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for TSQualifiedName<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.left, state);
        ContentHash::content_hash(&self.right, state);
    }
}

impl ContentHash for TSTypeParameterInstantiation<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.params, state);
    }
}

impl ContentHash for TSTypeParameter<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
        ContentHash::content_hash(&self.constraint, state);
        ContentHash::content_hash(&self.default, state);
        ContentHash::content_hash(&self.r#in, state);
        ContentHash::content_hash(&self.out, state);
        ContentHash::content_hash(&self.r#const, state);
    }
}

impl ContentHash for TSTypeParameterDeclaration<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.params, state);
    }
}

impl ContentHash for TSTypeAliasDeclaration<'_> {
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

impl ContentHash for TSClassImplements<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl ContentHash for TSInterfaceDeclaration<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.id, state);
        ContentHash::content_hash(&self.extends, state);
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.body, state);
        ContentHash::content_hash(&self.declare, state);
    }
}

impl ContentHash for TSInterfaceBody<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for TSPropertySignature<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.computed, state);
        ContentHash::content_hash(&self.optional, state);
        ContentHash::content_hash(&self.readonly, state);
        ContentHash::content_hash(&self.key, state);
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl ContentHash for TSSignature<'_> {
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

impl ContentHash for TSIndexSignature<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.parameters, state);
        ContentHash::content_hash(&self.type_annotation, state);
        ContentHash::content_hash(&self.readonly, state);
        ContentHash::content_hash(&self.r#static, state);
    }
}

impl ContentHash for TSCallSignatureDeclaration<'_> {
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

impl ContentHash for TSMethodSignature<'_> {
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

impl ContentHash for TSConstructSignatureDeclaration<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.params, state);
        ContentHash::content_hash(&self.return_type, state);
    }
}

impl ContentHash for TSIndexSignatureName<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl ContentHash for TSInterfaceHeritage<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl ContentHash for TSTypePredicate<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.parameter_name, state);
        ContentHash::content_hash(&self.asserts, state);
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl ContentHash for TSTypePredicateName<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::This(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for TSModuleDeclaration<'_> {
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

impl ContentHash for TSModuleDeclarationName<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for TSModuleDeclarationBody<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::TSModuleDeclaration(it) => ContentHash::content_hash(it, state),
            Self::TSModuleBlock(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for TSModuleBlock<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.directives, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for TSTypeLiteral<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.members, state);
    }
}

impl ContentHash for TSInferType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_parameter, state);
    }
}

impl ContentHash for TSTypeQuery<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expr_name, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl ContentHash for TSTypeQueryExprName<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::TSImportType(it) => ContentHash::content_hash(it, state),
            Self::IdentifierReference(it) => ContentHash::content_hash(it, state),
            Self::QualifiedName(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for TSImportType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.is_type_of, state);
        ContentHash::content_hash(&self.parameter, state);
        ContentHash::content_hash(&self.qualifier, state);
        ContentHash::content_hash(&self.attributes, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl ContentHash for TSImportAttributes<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.attributes_keyword, state);
        ContentHash::content_hash(&self.elements, state);
    }
}

impl ContentHash for TSImportAttribute<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
        ContentHash::content_hash(&self.value, state);
    }
}

impl ContentHash for TSImportAttributeName<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::StringLiteral(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for TSFunctionType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.this_param, state);
        ContentHash::content_hash(&self.params, state);
        ContentHash::content_hash(&self.return_type, state);
    }
}

impl ContentHash for TSConstructorType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.r#abstract, state);
        ContentHash::content_hash(&self.type_parameters, state);
        ContentHash::content_hash(&self.params, state);
        ContentHash::content_hash(&self.return_type, state);
    }
}

impl ContentHash for TSMappedType<'_> {
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

impl ContentHash for TSTemplateLiteralType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.quasis, state);
        ContentHash::content_hash(&self.types, state);
    }
}

impl ContentHash for TSAsExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl ContentHash for TSSatisfiesExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl ContentHash for TSTypeAssertion<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
        ContentHash::content_hash(&self.type_annotation, state);
    }
}

impl ContentHash for TSImportEqualsDeclaration<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.id, state);
        ContentHash::content_hash(&self.module_reference, state);
        ContentHash::content_hash(&self.import_kind, state);
    }
}

impl ContentHash for TSModuleReference<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::ExternalModuleReference(it) => ContentHash::content_hash(it, state),
            Self::IdentifierReference(it) => ContentHash::content_hash(it, state),
            Self::QualifiedName(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for TSExternalModuleReference<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl ContentHash for TSNonNullExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl ContentHash for Decorator<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl ContentHash for TSExportAssignment<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl ContentHash for TSNamespaceExportDeclaration<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.id, state);
    }
}

impl ContentHash for TSInstantiationExpression<'_> {
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

impl ContentHash for JSDocNullableType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_annotation, state);
        ContentHash::content_hash(&self.postfix, state);
    }
}

impl ContentHash for JSDocNonNullableType<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.type_annotation, state);
        ContentHash::content_hash(&self.postfix, state);
    }
}

impl ContentHash for JSDocUnknownType {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl ContentHash for JSXElement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.opening_element, state);
        ContentHash::content_hash(&self.closing_element, state);
        ContentHash::content_hash(&self.children, state);
    }
}

impl ContentHash for JSXOpeningElement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.self_closing, state);
        ContentHash::content_hash(&self.name, state);
        ContentHash::content_hash(&self.attributes, state);
        ContentHash::content_hash(&self.type_parameters, state);
    }
}

impl ContentHash for JSXClosingElement<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
    }
}

impl ContentHash for JSXFragment<'_> {
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

impl ContentHash for JSXElementName<'_> {
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

impl ContentHash for JSXNamespacedName<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.namespace, state);
        ContentHash::content_hash(&self.property, state);
    }
}

impl ContentHash for JSXMemberExpression<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.object, state);
        ContentHash::content_hash(&self.property, state);
    }
}

impl ContentHash for JSXMemberExpressionObject<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::IdentifierReference(it) => ContentHash::content_hash(it, state),
            Self::MemberExpression(it) => ContentHash::content_hash(it, state),
            Self::ThisExpression(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for JSXExpressionContainer<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl ContentHash for JSXExpression<'_> {
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

impl ContentHash for JSXAttributeItem<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Attribute(it) => ContentHash::content_hash(it, state),
            Self::SpreadAttribute(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for JSXAttribute<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
        ContentHash::content_hash(&self.value, state);
    }
}

impl ContentHash for JSXSpreadAttribute<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.argument, state);
    }
}

impl ContentHash for JSXAttributeName<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::Identifier(it) => ContentHash::content_hash(it, state),
            Self::NamespacedName(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for JSXAttributeValue<'_> {
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

impl ContentHash for JSXIdentifier<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
    }
}

impl ContentHash for JSXChild<'_> {
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

impl ContentHash for JSXSpreadChild<'_> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.expression, state);
    }
}

impl ContentHash for JSXText<'_> {
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
        ContentHash::content_hash(&self.attached_to, state);
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.position, state);
        ContentHash::content_hash(&self.preceded_by_newline, state);
        ContentHash::content_hash(&self.followed_by_newline, state);
    }
}
