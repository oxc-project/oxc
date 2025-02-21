// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, clippy::match_same_arms, clippy::semicolon_if_nothing_returned)]

use oxc_estree::{
    ser::{AppendTo, AppendToConcat},
    ESTree, FlatStructSerializer, Serializer, StructSerializer,
};

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl ESTree for Program<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Program");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field(
            "body",
            &AppendToConcat { array: &self.directives, after: &self.body },
        );
        self.source_type.serialize(FlatStructSerializer(&mut state));
        state.serialize_field("hashbang", &self.hashbang);
        state.end();
    }
}

impl ESTree for Expression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Expression::BooleanLiteral(it) => it.serialize(serializer),
            Expression::NullLiteral(it) => it.serialize(serializer),
            Expression::NumericLiteral(it) => it.serialize(serializer),
            Expression::BigIntLiteral(it) => it.serialize(serializer),
            Expression::RegExpLiteral(it) => it.serialize(serializer),
            Expression::StringLiteral(it) => it.serialize(serializer),
            Expression::TemplateLiteral(it) => it.serialize(serializer),
            Expression::Identifier(it) => it.serialize(serializer),
            Expression::MetaProperty(it) => it.serialize(serializer),
            Expression::Super(it) => it.serialize(serializer),
            Expression::ArrayExpression(it) => it.serialize(serializer),
            Expression::ArrowFunctionExpression(it) => it.serialize(serializer),
            Expression::AssignmentExpression(it) => it.serialize(serializer),
            Expression::AwaitExpression(it) => it.serialize(serializer),
            Expression::BinaryExpression(it) => it.serialize(serializer),
            Expression::CallExpression(it) => it.serialize(serializer),
            Expression::ChainExpression(it) => it.serialize(serializer),
            Expression::ClassExpression(it) => it.serialize(serializer),
            Expression::ConditionalExpression(it) => it.serialize(serializer),
            Expression::FunctionExpression(it) => it.serialize(serializer),
            Expression::ImportExpression(it) => it.serialize(serializer),
            Expression::LogicalExpression(it) => it.serialize(serializer),
            Expression::NewExpression(it) => it.serialize(serializer),
            Expression::ObjectExpression(it) => it.serialize(serializer),
            Expression::ParenthesizedExpression(it) => it.serialize(serializer),
            Expression::SequenceExpression(it) => it.serialize(serializer),
            Expression::TaggedTemplateExpression(it) => it.serialize(serializer),
            Expression::ThisExpression(it) => it.serialize(serializer),
            Expression::UnaryExpression(it) => it.serialize(serializer),
            Expression::UpdateExpression(it) => it.serialize(serializer),
            Expression::YieldExpression(it) => it.serialize(serializer),
            Expression::PrivateInExpression(it) => it.serialize(serializer),
            Expression::JSXElement(it) => it.serialize(serializer),
            Expression::JSXFragment(it) => it.serialize(serializer),
            Expression::TSAsExpression(it) => it.serialize(serializer),
            Expression::TSSatisfiesExpression(it) => it.serialize(serializer),
            Expression::TSTypeAssertion(it) => it.serialize(serializer),
            Expression::TSNonNullExpression(it) => it.serialize(serializer),
            Expression::TSInstantiationExpression(it) => it.serialize(serializer),
            Expression::ComputedMemberExpression(it) => it.serialize(serializer),
            Expression::StaticMemberExpression(it) => it.serialize(serializer),
            Expression::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for IdentifierName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Identifier");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("name", &self.name);
        state.end();
    }
}

impl ESTree for IdentifierReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Identifier");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("name", &self.name);
        state.end();
    }
}

impl ESTree for BindingIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Identifier");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("name", &self.name);
        state.end();
    }
}

impl ESTree for LabelIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Identifier");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("name", &self.name);
        state.end();
    }
}

impl ESTree for ThisExpression {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ThisExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for ArrayExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ArrayExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("elements", &self.elements);
        state.end();
    }
}

impl ESTree for ArrayExpressionElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            ArrayExpressionElement::SpreadElement(it) => it.serialize(serializer),
            ArrayExpressionElement::Elision(it) => it.serialize(serializer),
            ArrayExpressionElement::BooleanLiteral(it) => it.serialize(serializer),
            ArrayExpressionElement::NullLiteral(it) => it.serialize(serializer),
            ArrayExpressionElement::NumericLiteral(it) => it.serialize(serializer),
            ArrayExpressionElement::BigIntLiteral(it) => it.serialize(serializer),
            ArrayExpressionElement::RegExpLiteral(it) => it.serialize(serializer),
            ArrayExpressionElement::StringLiteral(it) => it.serialize(serializer),
            ArrayExpressionElement::TemplateLiteral(it) => it.serialize(serializer),
            ArrayExpressionElement::Identifier(it) => it.serialize(serializer),
            ArrayExpressionElement::MetaProperty(it) => it.serialize(serializer),
            ArrayExpressionElement::Super(it) => it.serialize(serializer),
            ArrayExpressionElement::ArrayExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::ArrowFunctionExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::AssignmentExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::AwaitExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::BinaryExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::CallExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::ChainExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::ClassExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::ConditionalExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::FunctionExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::ImportExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::LogicalExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::NewExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::ObjectExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::ParenthesizedExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::SequenceExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::TaggedTemplateExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::ThisExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::UnaryExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::UpdateExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::YieldExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::PrivateInExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::JSXElement(it) => it.serialize(serializer),
            ArrayExpressionElement::JSXFragment(it) => it.serialize(serializer),
            ArrayExpressionElement::TSAsExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::TSSatisfiesExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::TSTypeAssertion(it) => it.serialize(serializer),
            ArrayExpressionElement::TSNonNullExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::TSInstantiationExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::ComputedMemberExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::StaticMemberExpression(it) => it.serialize(serializer),
            ArrayExpressionElement::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for ObjectExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ObjectExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("properties", &self.properties);
        state.end();
    }
}

impl ESTree for ObjectPropertyKind<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            ObjectPropertyKind::ObjectProperty(it) => it.serialize(serializer),
            ObjectPropertyKind::SpreadProperty(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for PropertyKey<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            PropertyKey::StaticIdentifier(it) => it.serialize(serializer),
            PropertyKey::PrivateIdentifier(it) => it.serialize(serializer),
            PropertyKey::BooleanLiteral(it) => it.serialize(serializer),
            PropertyKey::NullLiteral(it) => it.serialize(serializer),
            PropertyKey::NumericLiteral(it) => it.serialize(serializer),
            PropertyKey::BigIntLiteral(it) => it.serialize(serializer),
            PropertyKey::RegExpLiteral(it) => it.serialize(serializer),
            PropertyKey::StringLiteral(it) => it.serialize(serializer),
            PropertyKey::TemplateLiteral(it) => it.serialize(serializer),
            PropertyKey::Identifier(it) => it.serialize(serializer),
            PropertyKey::MetaProperty(it) => it.serialize(serializer),
            PropertyKey::Super(it) => it.serialize(serializer),
            PropertyKey::ArrayExpression(it) => it.serialize(serializer),
            PropertyKey::ArrowFunctionExpression(it) => it.serialize(serializer),
            PropertyKey::AssignmentExpression(it) => it.serialize(serializer),
            PropertyKey::AwaitExpression(it) => it.serialize(serializer),
            PropertyKey::BinaryExpression(it) => it.serialize(serializer),
            PropertyKey::CallExpression(it) => it.serialize(serializer),
            PropertyKey::ChainExpression(it) => it.serialize(serializer),
            PropertyKey::ClassExpression(it) => it.serialize(serializer),
            PropertyKey::ConditionalExpression(it) => it.serialize(serializer),
            PropertyKey::FunctionExpression(it) => it.serialize(serializer),
            PropertyKey::ImportExpression(it) => it.serialize(serializer),
            PropertyKey::LogicalExpression(it) => it.serialize(serializer),
            PropertyKey::NewExpression(it) => it.serialize(serializer),
            PropertyKey::ObjectExpression(it) => it.serialize(serializer),
            PropertyKey::ParenthesizedExpression(it) => it.serialize(serializer),
            PropertyKey::SequenceExpression(it) => it.serialize(serializer),
            PropertyKey::TaggedTemplateExpression(it) => it.serialize(serializer),
            PropertyKey::ThisExpression(it) => it.serialize(serializer),
            PropertyKey::UnaryExpression(it) => it.serialize(serializer),
            PropertyKey::UpdateExpression(it) => it.serialize(serializer),
            PropertyKey::YieldExpression(it) => it.serialize(serializer),
            PropertyKey::PrivateInExpression(it) => it.serialize(serializer),
            PropertyKey::JSXElement(it) => it.serialize(serializer),
            PropertyKey::JSXFragment(it) => it.serialize(serializer),
            PropertyKey::TSAsExpression(it) => it.serialize(serializer),
            PropertyKey::TSSatisfiesExpression(it) => it.serialize(serializer),
            PropertyKey::TSTypeAssertion(it) => it.serialize(serializer),
            PropertyKey::TSNonNullExpression(it) => it.serialize(serializer),
            PropertyKey::TSInstantiationExpression(it) => it.serialize(serializer),
            PropertyKey::ComputedMemberExpression(it) => it.serialize(serializer),
            PropertyKey::StaticMemberExpression(it) => it.serialize(serializer),
            PropertyKey::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for PropertyKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            PropertyKind::Init => "init".serialize(serializer),
            PropertyKind::Get => "get".serialize(serializer),
            PropertyKind::Set => "set".serialize(serializer),
        }
    }
}

impl ESTree for TemplateLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TemplateLiteral");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expressions", &self.expressions);
        state.serialize_field("quasis", &self.quasis);
        state.end();
    }
}

impl ESTree for TaggedTemplateExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TaggedTemplateExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("tag", &self.tag);
        state.serialize_field("quasi", &self.quasi);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.end();
    }
}

impl ESTree for TemplateElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TemplateElement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("value", &self.value);
        state.serialize_field("tail", &self.tail);
        state.end();
    }
}

impl ESTree for TemplateElementValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("raw", &self.raw);
        state.serialize_field("cooked", &self.cooked);
        state.end();
    }
}

impl ESTree for MemberExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            MemberExpression::ComputedMemberExpression(it) => it.serialize(serializer),
            MemberExpression::StaticMemberExpression(it) => it.serialize(serializer),
            MemberExpression::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for ComputedMemberExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "MemberExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("object", &self.object);
        state.serialize_field("property", &self.expression);
        state.serialize_field("computed", &crate::serialize::True(self));
        state.serialize_field("optional", &self.optional);
        state.end();
    }
}

impl ESTree for StaticMemberExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "MemberExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("object", &self.object);
        state.serialize_field("property", &self.property);
        state.serialize_field("computed", &crate::serialize::False(self));
        state.serialize_field("optional", &self.optional);
        state.end();
    }
}

impl ESTree for PrivateFieldExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "MemberExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("object", &self.object);
        state.serialize_field("property", &self.field);
        state.serialize_field("computed", &crate::serialize::False(self));
        state.serialize_field("optional", &self.optional);
        state.end();
    }
}

impl ESTree for CallExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "CallExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("callee", &self.callee);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("arguments", &self.arguments);
        state.serialize_field("optional", &self.optional);
        state.end();
    }
}

impl ESTree for NewExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "NewExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("callee", &self.callee);
        state.serialize_field("arguments", &self.arguments);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.end();
    }
}

impl ESTree for MetaProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "MetaProperty");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("meta", &self.meta);
        state.serialize_field("property", &self.property);
        state.end();
    }
}

impl ESTree for SpreadElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "SpreadElement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("argument", &self.argument);
        state.end();
    }
}

impl ESTree for Argument<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Argument::SpreadElement(it) => it.serialize(serializer),
            Argument::BooleanLiteral(it) => it.serialize(serializer),
            Argument::NullLiteral(it) => it.serialize(serializer),
            Argument::NumericLiteral(it) => it.serialize(serializer),
            Argument::BigIntLiteral(it) => it.serialize(serializer),
            Argument::RegExpLiteral(it) => it.serialize(serializer),
            Argument::StringLiteral(it) => it.serialize(serializer),
            Argument::TemplateLiteral(it) => it.serialize(serializer),
            Argument::Identifier(it) => it.serialize(serializer),
            Argument::MetaProperty(it) => it.serialize(serializer),
            Argument::Super(it) => it.serialize(serializer),
            Argument::ArrayExpression(it) => it.serialize(serializer),
            Argument::ArrowFunctionExpression(it) => it.serialize(serializer),
            Argument::AssignmentExpression(it) => it.serialize(serializer),
            Argument::AwaitExpression(it) => it.serialize(serializer),
            Argument::BinaryExpression(it) => it.serialize(serializer),
            Argument::CallExpression(it) => it.serialize(serializer),
            Argument::ChainExpression(it) => it.serialize(serializer),
            Argument::ClassExpression(it) => it.serialize(serializer),
            Argument::ConditionalExpression(it) => it.serialize(serializer),
            Argument::FunctionExpression(it) => it.serialize(serializer),
            Argument::ImportExpression(it) => it.serialize(serializer),
            Argument::LogicalExpression(it) => it.serialize(serializer),
            Argument::NewExpression(it) => it.serialize(serializer),
            Argument::ObjectExpression(it) => it.serialize(serializer),
            Argument::ParenthesizedExpression(it) => it.serialize(serializer),
            Argument::SequenceExpression(it) => it.serialize(serializer),
            Argument::TaggedTemplateExpression(it) => it.serialize(serializer),
            Argument::ThisExpression(it) => it.serialize(serializer),
            Argument::UnaryExpression(it) => it.serialize(serializer),
            Argument::UpdateExpression(it) => it.serialize(serializer),
            Argument::YieldExpression(it) => it.serialize(serializer),
            Argument::PrivateInExpression(it) => it.serialize(serializer),
            Argument::JSXElement(it) => it.serialize(serializer),
            Argument::JSXFragment(it) => it.serialize(serializer),
            Argument::TSAsExpression(it) => it.serialize(serializer),
            Argument::TSSatisfiesExpression(it) => it.serialize(serializer),
            Argument::TSTypeAssertion(it) => it.serialize(serializer),
            Argument::TSNonNullExpression(it) => it.serialize(serializer),
            Argument::TSInstantiationExpression(it) => it.serialize(serializer),
            Argument::ComputedMemberExpression(it) => it.serialize(serializer),
            Argument::StaticMemberExpression(it) => it.serialize(serializer),
            Argument::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for UpdateExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "UpdateExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("operator", &self.operator);
        state.serialize_field("prefix", &self.prefix);
        state.serialize_field("argument", &self.argument);
        state.end();
    }
}

impl ESTree for UnaryExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "UnaryExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("operator", &self.operator);
        state.serialize_field("prefix", &crate::serialize::True(self));
        state.serialize_field("argument", &self.argument);
        state.end();
    }
}

impl ESTree for BinaryExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "BinaryExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("left", &self.left);
        state.serialize_field("operator", &self.operator);
        state.serialize_field("right", &self.right);
        state.end();
    }
}

impl ESTree for PrivateInExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "BinaryExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("left", &self.left);
        state.serialize_field("operator", &crate::serialize::In(self));
        state.serialize_field("right", &self.right);
        state.end();
    }
}

impl ESTree for LogicalExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "LogicalExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("left", &self.left);
        state.serialize_field("operator", &self.operator);
        state.serialize_field("right", &self.right);
        state.end();
    }
}

impl ESTree for ConditionalExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ConditionalExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("test", &self.test);
        state.serialize_field("consequent", &self.consequent);
        state.serialize_field("alternate", &self.alternate);
        state.end();
    }
}

impl ESTree for AssignmentExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "AssignmentExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("operator", &self.operator);
        state.serialize_field("left", &self.left);
        state.serialize_field("right", &self.right);
        state.end();
    }
}

impl ESTree for AssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            AssignmentTarget::AssignmentTargetIdentifier(it) => it.serialize(serializer),
            AssignmentTarget::TSAsExpression(it) => it.serialize(serializer),
            AssignmentTarget::TSSatisfiesExpression(it) => it.serialize(serializer),
            AssignmentTarget::TSNonNullExpression(it) => it.serialize(serializer),
            AssignmentTarget::TSTypeAssertion(it) => it.serialize(serializer),
            AssignmentTarget::TSInstantiationExpression(it) => it.serialize(serializer),
            AssignmentTarget::ComputedMemberExpression(it) => it.serialize(serializer),
            AssignmentTarget::StaticMemberExpression(it) => it.serialize(serializer),
            AssignmentTarget::PrivateFieldExpression(it) => it.serialize(serializer),
            AssignmentTarget::ArrayAssignmentTarget(it) => it.serialize(serializer),
            AssignmentTarget::ObjectAssignmentTarget(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for SimpleAssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(it) => it.serialize(serializer),
            SimpleAssignmentTarget::TSAsExpression(it) => it.serialize(serializer),
            SimpleAssignmentTarget::TSSatisfiesExpression(it) => it.serialize(serializer),
            SimpleAssignmentTarget::TSNonNullExpression(it) => it.serialize(serializer),
            SimpleAssignmentTarget::TSTypeAssertion(it) => it.serialize(serializer),
            SimpleAssignmentTarget::TSInstantiationExpression(it) => it.serialize(serializer),
            SimpleAssignmentTarget::ComputedMemberExpression(it) => it.serialize(serializer),
            SimpleAssignmentTarget::StaticMemberExpression(it) => it.serialize(serializer),
            SimpleAssignmentTarget::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for AssignmentTargetPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            AssignmentTargetPattern::ArrayAssignmentTarget(it) => it.serialize(serializer),
            AssignmentTargetPattern::ObjectAssignmentTarget(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for ArrayAssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ArrayPattern");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("elements", &AppendTo { array: &self.elements, after: &self.rest });
        state.end();
    }
}

impl ESTree for ObjectAssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ObjectPattern");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field(
            "properties",
            &AppendTo { array: &self.properties, after: &self.rest },
        );
        state.end();
    }
}

impl ESTree for AssignmentTargetRest<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "RestElement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("argument", &self.target);
        state.end();
    }
}

impl ESTree for AssignmentTargetMaybeDefault<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(it) => {
                it.serialize(serializer)
            }
            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(it) => {
                it.serialize(serializer)
            }
            AssignmentTargetMaybeDefault::TSAsExpression(it) => it.serialize(serializer),
            AssignmentTargetMaybeDefault::TSSatisfiesExpression(it) => it.serialize(serializer),
            AssignmentTargetMaybeDefault::TSNonNullExpression(it) => it.serialize(serializer),
            AssignmentTargetMaybeDefault::TSTypeAssertion(it) => it.serialize(serializer),
            AssignmentTargetMaybeDefault::TSInstantiationExpression(it) => it.serialize(serializer),
            AssignmentTargetMaybeDefault::ComputedMemberExpression(it) => it.serialize(serializer),
            AssignmentTargetMaybeDefault::StaticMemberExpression(it) => it.serialize(serializer),
            AssignmentTargetMaybeDefault::PrivateFieldExpression(it) => it.serialize(serializer),
            AssignmentTargetMaybeDefault::ArrayAssignmentTarget(it) => it.serialize(serializer),
            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for AssignmentTargetWithDefault<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "AssignmentPattern");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("left", &self.binding);
        state.serialize_field("right", &self.init);
        state.end();
    }
}

impl ESTree for AssignmentTargetProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(it) => {
                it.serialize(serializer)
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(it) => {
                it.serialize(serializer)
            }
        }
    }
}

impl ESTree for AssignmentTargetPropertyIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Property");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("method", &crate::serialize::False(self));
        state.serialize_field("shorthand", &crate::serialize::True(self));
        state.serialize_field("computed", &crate::serialize::False(self));
        state.serialize_field("key", &self.binding);
        state.serialize_field("kind", &crate::serialize::Init(self));
        state.serialize_field(
            "value",
            &crate::serialize::AssignmentTargetPropertyIdentifierValue(self),
        );
        state.end();
    }
}

impl ESTree for AssignmentTargetPropertyProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Property");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("method", &crate::serialize::False(self));
        state.serialize_field("shorthand", &crate::serialize::False(self));
        state.serialize_field("computed", &self.computed);
        state.serialize_field("key", &self.name);
        state.serialize_field("value", &self.binding);
        state.serialize_field("kind", &crate::serialize::Init(self));
        state.end();
    }
}

impl ESTree for SequenceExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "SequenceExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expressions", &self.expressions);
        state.end();
    }
}

impl ESTree for Super {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Super");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for AwaitExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "AwaitExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("argument", &self.argument);
        state.end();
    }
}

impl ESTree for ChainExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ChainExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.end();
    }
}

impl ESTree for ChainElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            ChainElement::CallExpression(it) => it.serialize(serializer),
            ChainElement::TSNonNullExpression(it) => it.serialize(serializer),
            ChainElement::ComputedMemberExpression(it) => it.serialize(serializer),
            ChainElement::StaticMemberExpression(it) => it.serialize(serializer),
            ChainElement::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for ParenthesizedExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ParenthesizedExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.end();
    }
}

impl ESTree for Statement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Statement::BlockStatement(it) => it.serialize(serializer),
            Statement::BreakStatement(it) => it.serialize(serializer),
            Statement::ContinueStatement(it) => it.serialize(serializer),
            Statement::DebuggerStatement(it) => it.serialize(serializer),
            Statement::DoWhileStatement(it) => it.serialize(serializer),
            Statement::EmptyStatement(it) => it.serialize(serializer),
            Statement::ExpressionStatement(it) => it.serialize(serializer),
            Statement::ForInStatement(it) => it.serialize(serializer),
            Statement::ForOfStatement(it) => it.serialize(serializer),
            Statement::ForStatement(it) => it.serialize(serializer),
            Statement::IfStatement(it) => it.serialize(serializer),
            Statement::LabeledStatement(it) => it.serialize(serializer),
            Statement::ReturnStatement(it) => it.serialize(serializer),
            Statement::SwitchStatement(it) => it.serialize(serializer),
            Statement::ThrowStatement(it) => it.serialize(serializer),
            Statement::TryStatement(it) => it.serialize(serializer),
            Statement::WhileStatement(it) => it.serialize(serializer),
            Statement::WithStatement(it) => it.serialize(serializer),
            Statement::VariableDeclaration(it) => it.serialize(serializer),
            Statement::FunctionDeclaration(it) => it.serialize(serializer),
            Statement::ClassDeclaration(it) => it.serialize(serializer),
            Statement::TSTypeAliasDeclaration(it) => it.serialize(serializer),
            Statement::TSInterfaceDeclaration(it) => it.serialize(serializer),
            Statement::TSEnumDeclaration(it) => it.serialize(serializer),
            Statement::TSModuleDeclaration(it) => it.serialize(serializer),
            Statement::TSImportEqualsDeclaration(it) => it.serialize(serializer),
            Statement::ImportDeclaration(it) => it.serialize(serializer),
            Statement::ExportAllDeclaration(it) => it.serialize(serializer),
            Statement::ExportDefaultDeclaration(it) => it.serialize(serializer),
            Statement::ExportNamedDeclaration(it) => it.serialize(serializer),
            Statement::TSExportAssignment(it) => it.serialize(serializer),
            Statement::TSNamespaceExportDeclaration(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for Directive<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ExpressionStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.serialize_field("directive", &self.directive);
        state.end();
    }
}

impl ESTree for Hashbang<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Hashbang");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("value", &self.value);
        state.end();
    }
}

impl ESTree for BlockStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "BlockStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for Declaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Declaration::VariableDeclaration(it) => it.serialize(serializer),
            Declaration::FunctionDeclaration(it) => it.serialize(serializer),
            Declaration::ClassDeclaration(it) => it.serialize(serializer),
            Declaration::TSTypeAliasDeclaration(it) => it.serialize(serializer),
            Declaration::TSInterfaceDeclaration(it) => it.serialize(serializer),
            Declaration::TSEnumDeclaration(it) => it.serialize(serializer),
            Declaration::TSModuleDeclaration(it) => it.serialize(serializer),
            Declaration::TSImportEqualsDeclaration(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for VariableDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "VariableDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("declarations", &self.declarations);
        state.serialize_field("kind", &self.kind);
        state.serialize_field("declare", &self.declare);
        state.end();
    }
}

impl ESTree for VariableDeclarationKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            VariableDeclarationKind::Var => "var".serialize(serializer),
            VariableDeclarationKind::Const => "const".serialize(serializer),
            VariableDeclarationKind::Let => "let".serialize(serializer),
            VariableDeclarationKind::Using => "using".serialize(serializer),
            VariableDeclarationKind::AwaitUsing => "await using".serialize(serializer),
        }
    }
}

impl ESTree for VariableDeclarator<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "VariableDeclarator");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("id", &self.id);
        state.serialize_field("init", &self.init);
        state.serialize_field("definite", &self.definite);
        state.end();
    }
}

impl ESTree for EmptyStatement {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "EmptyStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for ExpressionStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ExpressionStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.end();
    }
}

impl ESTree for IfStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "IfStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("test", &self.test);
        state.serialize_field("consequent", &self.consequent);
        state.serialize_field("alternate", &self.alternate);
        state.end();
    }
}

impl ESTree for DoWhileStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "DoWhileStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("body", &self.body);
        state.serialize_field("test", &self.test);
        state.end();
    }
}

impl ESTree for WhileStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "WhileStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("test", &self.test);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for ForStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ForStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("init", &self.init);
        state.serialize_field("test", &self.test);
        state.serialize_field("update", &self.update);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for ForStatementInit<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            ForStatementInit::VariableDeclaration(it) => it.serialize(serializer),
            ForStatementInit::BooleanLiteral(it) => it.serialize(serializer),
            ForStatementInit::NullLiteral(it) => it.serialize(serializer),
            ForStatementInit::NumericLiteral(it) => it.serialize(serializer),
            ForStatementInit::BigIntLiteral(it) => it.serialize(serializer),
            ForStatementInit::RegExpLiteral(it) => it.serialize(serializer),
            ForStatementInit::StringLiteral(it) => it.serialize(serializer),
            ForStatementInit::TemplateLiteral(it) => it.serialize(serializer),
            ForStatementInit::Identifier(it) => it.serialize(serializer),
            ForStatementInit::MetaProperty(it) => it.serialize(serializer),
            ForStatementInit::Super(it) => it.serialize(serializer),
            ForStatementInit::ArrayExpression(it) => it.serialize(serializer),
            ForStatementInit::ArrowFunctionExpression(it) => it.serialize(serializer),
            ForStatementInit::AssignmentExpression(it) => it.serialize(serializer),
            ForStatementInit::AwaitExpression(it) => it.serialize(serializer),
            ForStatementInit::BinaryExpression(it) => it.serialize(serializer),
            ForStatementInit::CallExpression(it) => it.serialize(serializer),
            ForStatementInit::ChainExpression(it) => it.serialize(serializer),
            ForStatementInit::ClassExpression(it) => it.serialize(serializer),
            ForStatementInit::ConditionalExpression(it) => it.serialize(serializer),
            ForStatementInit::FunctionExpression(it) => it.serialize(serializer),
            ForStatementInit::ImportExpression(it) => it.serialize(serializer),
            ForStatementInit::LogicalExpression(it) => it.serialize(serializer),
            ForStatementInit::NewExpression(it) => it.serialize(serializer),
            ForStatementInit::ObjectExpression(it) => it.serialize(serializer),
            ForStatementInit::ParenthesizedExpression(it) => it.serialize(serializer),
            ForStatementInit::SequenceExpression(it) => it.serialize(serializer),
            ForStatementInit::TaggedTemplateExpression(it) => it.serialize(serializer),
            ForStatementInit::ThisExpression(it) => it.serialize(serializer),
            ForStatementInit::UnaryExpression(it) => it.serialize(serializer),
            ForStatementInit::UpdateExpression(it) => it.serialize(serializer),
            ForStatementInit::YieldExpression(it) => it.serialize(serializer),
            ForStatementInit::PrivateInExpression(it) => it.serialize(serializer),
            ForStatementInit::JSXElement(it) => it.serialize(serializer),
            ForStatementInit::JSXFragment(it) => it.serialize(serializer),
            ForStatementInit::TSAsExpression(it) => it.serialize(serializer),
            ForStatementInit::TSSatisfiesExpression(it) => it.serialize(serializer),
            ForStatementInit::TSTypeAssertion(it) => it.serialize(serializer),
            ForStatementInit::TSNonNullExpression(it) => it.serialize(serializer),
            ForStatementInit::TSInstantiationExpression(it) => it.serialize(serializer),
            ForStatementInit::ComputedMemberExpression(it) => it.serialize(serializer),
            ForStatementInit::StaticMemberExpression(it) => it.serialize(serializer),
            ForStatementInit::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for ForInStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ForInStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("left", &self.left);
        state.serialize_field("right", &self.right);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for ForStatementLeft<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            ForStatementLeft::VariableDeclaration(it) => it.serialize(serializer),
            ForStatementLeft::AssignmentTargetIdentifier(it) => it.serialize(serializer),
            ForStatementLeft::TSAsExpression(it) => it.serialize(serializer),
            ForStatementLeft::TSSatisfiesExpression(it) => it.serialize(serializer),
            ForStatementLeft::TSNonNullExpression(it) => it.serialize(serializer),
            ForStatementLeft::TSTypeAssertion(it) => it.serialize(serializer),
            ForStatementLeft::TSInstantiationExpression(it) => it.serialize(serializer),
            ForStatementLeft::ComputedMemberExpression(it) => it.serialize(serializer),
            ForStatementLeft::StaticMemberExpression(it) => it.serialize(serializer),
            ForStatementLeft::PrivateFieldExpression(it) => it.serialize(serializer),
            ForStatementLeft::ArrayAssignmentTarget(it) => it.serialize(serializer),
            ForStatementLeft::ObjectAssignmentTarget(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for ForOfStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ForOfStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("await", &self.r#await);
        state.serialize_field("left", &self.left);
        state.serialize_field("right", &self.right);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for ContinueStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ContinueStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("label", &self.label);
        state.end();
    }
}

impl ESTree for BreakStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "BreakStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("label", &self.label);
        state.end();
    }
}

impl ESTree for ReturnStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ReturnStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("argument", &self.argument);
        state.end();
    }
}

impl ESTree for WithStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "WithStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("object", &self.object);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for SwitchStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "SwitchStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("discriminant", &self.discriminant);
        state.serialize_field("cases", &self.cases);
        state.end();
    }
}

impl ESTree for SwitchCase<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "SwitchCase");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("consequent", &self.consequent);
        state.serialize_field("test", &self.test);
        state.end();
    }
}

impl ESTree for LabeledStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "LabeledStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("body", &self.body);
        state.serialize_field("label", &self.label);
        state.end();
    }
}

impl ESTree for ThrowStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ThrowStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("argument", &self.argument);
        state.end();
    }
}

impl ESTree for TryStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TryStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("block", &self.block);
        state.serialize_field("handler", &self.handler);
        state.serialize_field("finalizer", &self.finalizer);
        state.end();
    }
}

impl ESTree for CatchClause<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "CatchClause");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("param", &self.param);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for CatchParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        self.pattern.kind.serialize(FlatStructSerializer(&mut state));
        state.serialize_field("typeAnnotation", &self.pattern.type_annotation);
        state.serialize_field("optional", &self.pattern.optional);
        state.end();
    }
}

impl ESTree for DebuggerStatement {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "DebuggerStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for BindingPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        self.kind.serialize(FlatStructSerializer(&mut state));
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("optional", &self.optional);
        state.end();
    }
}

impl ESTree for BindingPatternKind<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            BindingPatternKind::BindingIdentifier(it) => it.serialize(serializer),
            BindingPatternKind::ObjectPattern(it) => it.serialize(serializer),
            BindingPatternKind::ArrayPattern(it) => it.serialize(serializer),
            BindingPatternKind::AssignmentPattern(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for AssignmentPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "AssignmentPattern");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("left", &self.left);
        state.serialize_field("right", &self.right);
        state.end();
    }
}

impl ESTree for ObjectPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ObjectPattern");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field(
            "properties",
            &AppendTo { array: &self.properties, after: &self.rest },
        );
        state.end();
    }
}

impl ESTree for ArrayPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ArrayPattern");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("elements", &AppendTo { array: &self.elements, after: &self.rest });
        state.end();
    }
}

impl ESTree for BindingRestElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "RestElement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("argument", &self.argument);
        state.end();
    }
}

impl ESTree for Function<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &self.r#type);
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("id", &self.id);
        state.serialize_field("expression", &crate::serialize::False(self));
        state.serialize_field("generator", &self.generator);
        state.serialize_field("async", &self.r#async);
        state.serialize_field("params", &self.params);
        state.serialize_field("body", &self.body);
        state.serialize_field("declare", &self.declare);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("thisParam", &self.this_param);
        state.serialize_field("returnType", &self.return_type);
        state.end();
    }
}

impl ESTree for FunctionType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            FunctionType::FunctionDeclaration => "FunctionDeclaration".serialize(serializer),
            FunctionType::FunctionExpression => "FunctionExpression".serialize(serializer),
            FunctionType::TSDeclareFunction => "TSDeclareFunction".serialize(serializer),
            FunctionType::TSEmptyBodyFunctionExpression => {
                "TSEmptyBodyFunctionExpression".serialize(serializer)
            }
        }
    }
}

impl ESTree for FormalParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        self.pattern.kind.serialize(FlatStructSerializer(&mut state));
        state.serialize_field("typeAnnotation", &self.pattern.type_annotation);
        state.serialize_field("optional", &self.pattern.optional);
        state.serialize_field("decorators", &self.decorators);
        state.serialize_field("accessibility", &self.accessibility);
        state.serialize_field("readonly", &self.readonly);
        state.serialize_field("override", &self.r#override);
        state.end();
    }
}

impl ESTree for FormalParameterKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            FormalParameterKind::FormalParameter => "FormalParameter".serialize(serializer),
            FormalParameterKind::UniqueFormalParameters => {
                "UniqueFormalParameters".serialize(serializer)
            }
            FormalParameterKind::ArrowFormalParameters => {
                "ArrowFormalParameters".serialize(serializer)
            }
            FormalParameterKind::Signature => "Signature".serialize(serializer),
        }
    }
}

impl ESTree for FunctionBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "BlockStatement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field(
            "body",
            &AppendToConcat { array: &self.directives, after: &self.statements },
        );
        state.end();
    }
}

impl ESTree for ArrowFunctionExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ArrowFunctionExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("id", &crate::serialize::Null(self));
        state.serialize_field("expression", &self.expression);
        state.serialize_field("generator", &crate::serialize::False(self));
        state.serialize_field("async", &self.r#async);
        state.serialize_field("params", &self.params);
        state.serialize_field("body", &crate::serialize::ArrowFunctionExpressionBody(self));
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("returnType", &self.return_type);
        state.end();
    }
}

impl ESTree for YieldExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "YieldExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("delegate", &self.delegate);
        state.serialize_field("argument", &self.argument);
        state.end();
    }
}

impl ESTree for Class<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &self.r#type);
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("id", &self.id);
        state.serialize_field("superClass", &self.super_class);
        state.serialize_field("body", &self.body);
        state.serialize_field("decorators", &self.decorators);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("superTypeParameters", &self.super_type_parameters);
        state.serialize_field("implements", &self.implements);
        state.serialize_field("abstract", &self.r#abstract);
        state.serialize_field("declare", &self.declare);
        state.end();
    }
}

impl ESTree for ClassType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            ClassType::ClassDeclaration => "ClassDeclaration".serialize(serializer),
            ClassType::ClassExpression => "ClassExpression".serialize(serializer),
        }
    }
}

impl ESTree for ClassBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ClassBody");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for ClassElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            ClassElement::StaticBlock(it) => it.serialize(serializer),
            ClassElement::MethodDefinition(it) => it.serialize(serializer),
            ClassElement::PropertyDefinition(it) => it.serialize(serializer),
            ClassElement::AccessorProperty(it) => it.serialize(serializer),
            ClassElement::TSIndexSignature(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for MethodDefinition<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &self.r#type);
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("static", &self.r#static);
        state.serialize_field("computed", &self.computed);
        state.serialize_field("key", &self.key);
        state.serialize_field("kind", &self.kind);
        state.serialize_field("value", &self.value);
        state.serialize_field("decorators", &self.decorators);
        state.serialize_field("override", &self.r#override);
        state.serialize_field("optional", &self.optional);
        state.serialize_field("accessibility", &self.accessibility);
        state.end();
    }
}

impl ESTree for MethodDefinitionType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            MethodDefinitionType::MethodDefinition => "MethodDefinition".serialize(serializer),
            MethodDefinitionType::TSAbstractMethodDefinition => {
                "TSAbstractMethodDefinition".serialize(serializer)
            }
        }
    }
}

impl ESTree for PropertyDefinition<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &self.r#type);
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("static", &self.r#static);
        state.serialize_field("computed", &self.computed);
        state.serialize_field("key", &self.key);
        state.serialize_field("value", &self.value);
        state.serialize_field("decorators", &self.decorators);
        state.serialize_field("declare", &self.declare);
        state.serialize_field("override", &self.r#override);
        state.serialize_field("optional", &self.optional);
        state.serialize_field("definite", &self.definite);
        state.serialize_field("readonly", &self.readonly);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("accessibility", &self.accessibility);
        state.end();
    }
}

impl ESTree for PropertyDefinitionType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            PropertyDefinitionType::PropertyDefinition => {
                "PropertyDefinition".serialize(serializer)
            }
            PropertyDefinitionType::TSAbstractPropertyDefinition => {
                "TSAbstractPropertyDefinition".serialize(serializer)
            }
        }
    }
}

impl ESTree for MethodDefinitionKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            MethodDefinitionKind::Constructor => "constructor".serialize(serializer),
            MethodDefinitionKind::Method => "method".serialize(serializer),
            MethodDefinitionKind::Get => "get".serialize(serializer),
            MethodDefinitionKind::Set => "set".serialize(serializer),
        }
    }
}

impl ESTree for PrivateIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "PrivateIdentifier");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("name", &self.name);
        state.end();
    }
}

impl ESTree for StaticBlock<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "StaticBlock");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for ModuleDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            ModuleDeclaration::ImportDeclaration(it) => it.serialize(serializer),
            ModuleDeclaration::ExportAllDeclaration(it) => it.serialize(serializer),
            ModuleDeclaration::ExportDefaultDeclaration(it) => it.serialize(serializer),
            ModuleDeclaration::ExportNamedDeclaration(it) => it.serialize(serializer),
            ModuleDeclaration::TSExportAssignment(it) => it.serialize(serializer),
            ModuleDeclaration::TSNamespaceExportDeclaration(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for AccessorPropertyType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            AccessorPropertyType::AccessorProperty => "AccessorProperty".serialize(serializer),
            AccessorPropertyType::TSAbstractAccessorProperty => {
                "TSAbstractAccessorProperty".serialize(serializer)
            }
        }
    }
}

impl ESTree for AccessorProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &self.r#type);
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("key", &self.key);
        state.serialize_field("value", &self.value);
        state.serialize_field("computed", &self.computed);
        state.serialize_field("static", &self.r#static);
        state.serialize_field("decorators", &self.decorators);
        state.serialize_field("definite", &self.definite);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("accessibility", &self.accessibility);
        state.end();
    }
}

impl ESTree for ImportExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ImportExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("source", &self.source);
        state.serialize_field("options", &crate::serialize::ImportExpressionArguments(self));
        state.end();
    }
}

impl ESTree for ImportDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ImportDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("specifiers", &crate::serialize::ImportDeclarationSpecifiers(self));
        state.serialize_field("source", &self.source);
        state.serialize_field("phase", &self.phase);
        state.serialize_field("attributes", &crate::serialize::ImportDeclarationWithClause(self));
        state.serialize_field("importKind", &self.import_kind);
        state.end();
    }
}

impl ESTree for ImportPhase {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            ImportPhase::Source => "source".serialize(serializer),
            ImportPhase::Defer => "defer".serialize(serializer),
        }
    }
}

impl ESTree for ImportDeclarationSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            ImportDeclarationSpecifier::ImportSpecifier(it) => it.serialize(serializer),
            ImportDeclarationSpecifier::ImportDefaultSpecifier(it) => it.serialize(serializer),
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for ImportSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ImportSpecifier");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("imported", &self.imported);
        state.serialize_field("local", &self.local);
        state.serialize_field("importKind", &self.import_kind);
        state.end();
    }
}

impl ESTree for ImportDefaultSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ImportDefaultSpecifier");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("local", &self.local);
        state.end();
    }
}

impl ESTree for ImportNamespaceSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ImportNamespaceSpecifier");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("local", &self.local);
        state.end();
    }
}

impl ESTree for WithClause<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "WithClause");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("attributesKeyword", &self.attributes_keyword);
        state.serialize_field("withEntries", &self.with_entries);
        state.end();
    }
}

impl ESTree for ImportAttribute<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ImportAttribute");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("key", &self.key);
        state.serialize_field("value", &self.value);
        state.end();
    }
}

impl ESTree for ImportAttributeKey<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            ImportAttributeKey::Identifier(it) => it.serialize(serializer),
            ImportAttributeKey::StringLiteral(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for ExportNamedDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ExportNamedDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("declaration", &self.declaration);
        state.serialize_field("specifiers", &self.specifiers);
        state.serialize_field("source", &self.source);
        state.serialize_field("exportKind", &self.export_kind);
        state.serialize_field(
            "attributes",
            &crate::serialize::ExportNamedDeclarationWithClause(self),
        );
        state.end();
    }
}

impl ESTree for ExportDefaultDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ExportDefaultDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("declaration", &self.declaration);
        state.serialize_field("exported", &self.exported);
        state.end();
    }
}

impl ESTree for ExportAllDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ExportAllDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("exported", &self.exported);
        state.serialize_field("source", &self.source);
        state
            .serialize_field("attributes", &crate::serialize::ExportAllDeclarationWithClause(self));
        state.serialize_field("exportKind", &self.export_kind);
        state.end();
    }
}

impl ESTree for ExportSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ExportSpecifier");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("local", &self.local);
        state.serialize_field("exported", &self.exported);
        state.serialize_field("exportKind", &self.export_kind);
        state.end();
    }
}

impl ESTree for ExportDefaultDeclarationKind<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            ExportDefaultDeclarationKind::FunctionDeclaration(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::ClassDeclaration(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::BooleanLiteral(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::NullLiteral(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::NumericLiteral(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::BigIntLiteral(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::RegExpLiteral(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::StringLiteral(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::TemplateLiteral(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::Identifier(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::MetaProperty(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::Super(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::ArrayExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::ArrowFunctionExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::AssignmentExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::AwaitExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::BinaryExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::CallExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::ChainExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::ClassExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::ConditionalExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::FunctionExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::ImportExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::LogicalExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::NewExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::ObjectExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::ParenthesizedExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::SequenceExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::TaggedTemplateExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::ThisExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::UnaryExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::UpdateExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::YieldExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::PrivateInExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::JSXElement(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::JSXFragment(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::TSAsExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::TSSatisfiesExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::TSTypeAssertion(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::TSNonNullExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::TSInstantiationExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::ComputedMemberExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::StaticMemberExpression(it) => it.serialize(serializer),
            ExportDefaultDeclarationKind::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for ModuleExportName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            ModuleExportName::IdentifierName(it) => it.serialize(serializer),
            ModuleExportName::IdentifierReference(it) => it.serialize(serializer),
            ModuleExportName::StringLiteral(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for BooleanLiteral {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Literal");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("value", &self.value);
        state.serialize_field("raw", &crate::serialize::BooleanLiteralRaw(self));
        state.end();
    }
}

impl ESTree for NullLiteral {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Literal");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("value", &crate::serialize::Null(self));
        state.serialize_field("raw", &crate::serialize::NullLiteralRaw(self));
        state.end();
    }
}

impl ESTree for NumericLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Literal");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("value", &self.value);
        state.serialize_field("raw", &self.raw);
        state.end();
    }
}

impl ESTree for StringLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Literal");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("value", &self.value);
        state.serialize_field("raw", &self.raw);
        state.end();
    }
}

impl ESTree for BigIntLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Literal");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("value", &crate::serialize::BigIntLiteralValue(self));
        state.serialize_field("raw", &self.raw);
        state.serialize_field("bigint", &crate::serialize::BigIntLiteralBigint(self));
        state.end();
    }
}

impl ESTree for RegExpLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Literal");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("value", &crate::serialize::RegExpLiteralValue(self));
        state.serialize_field("raw", &self.raw);
        state.serialize_field("regex", &crate::serialize::RegExpLiteralRegex(self));
        state.end();
    }
}

impl ESTree for RegExp<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("pattern", &self.pattern);
        state.serialize_field("flags", &self.flags);
        state.end();
    }
}

impl ESTree for JSXElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXElement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("openingElement", &self.opening_element);
        state.serialize_field("closingElement", &self.closing_element);
        state.serialize_field("children", &self.children);
        state.end();
    }
}

impl ESTree for JSXOpeningElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXOpeningElement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("selfClosing", &self.self_closing);
        state.serialize_field("name", &self.name);
        state.serialize_field("attributes", &self.attributes);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.end();
    }
}

impl ESTree for JSXClosingElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXClosingElement");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("name", &self.name);
        state.end();
    }
}

impl ESTree for JSXFragment<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXFragment");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("openingFragment", &self.opening_fragment);
        state.serialize_field("closingFragment", &self.closing_fragment);
        state.serialize_field("children", &self.children);
        state.end();
    }
}

impl ESTree for JSXOpeningFragment {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXOpeningFragment");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for JSXClosingFragment {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXClosingFragment");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for JSXNamespacedName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXNamespacedName");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("namespace", &self.namespace);
        state.serialize_field("property", &self.property);
        state.end();
    }
}

impl ESTree for JSXMemberExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXMemberExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("object", &self.object);
        state.serialize_field("property", &self.property);
        state.end();
    }
}

impl ESTree for JSXExpressionContainer<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXExpressionContainer");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.end();
    }
}

impl ESTree for JSXExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            JSXExpression::EmptyExpression(it) => it.serialize(serializer),
            JSXExpression::BooleanLiteral(it) => it.serialize(serializer),
            JSXExpression::NullLiteral(it) => it.serialize(serializer),
            JSXExpression::NumericLiteral(it) => it.serialize(serializer),
            JSXExpression::BigIntLiteral(it) => it.serialize(serializer),
            JSXExpression::RegExpLiteral(it) => it.serialize(serializer),
            JSXExpression::StringLiteral(it) => it.serialize(serializer),
            JSXExpression::TemplateLiteral(it) => it.serialize(serializer),
            JSXExpression::Identifier(it) => it.serialize(serializer),
            JSXExpression::MetaProperty(it) => it.serialize(serializer),
            JSXExpression::Super(it) => it.serialize(serializer),
            JSXExpression::ArrayExpression(it) => it.serialize(serializer),
            JSXExpression::ArrowFunctionExpression(it) => it.serialize(serializer),
            JSXExpression::AssignmentExpression(it) => it.serialize(serializer),
            JSXExpression::AwaitExpression(it) => it.serialize(serializer),
            JSXExpression::BinaryExpression(it) => it.serialize(serializer),
            JSXExpression::CallExpression(it) => it.serialize(serializer),
            JSXExpression::ChainExpression(it) => it.serialize(serializer),
            JSXExpression::ClassExpression(it) => it.serialize(serializer),
            JSXExpression::ConditionalExpression(it) => it.serialize(serializer),
            JSXExpression::FunctionExpression(it) => it.serialize(serializer),
            JSXExpression::ImportExpression(it) => it.serialize(serializer),
            JSXExpression::LogicalExpression(it) => it.serialize(serializer),
            JSXExpression::NewExpression(it) => it.serialize(serializer),
            JSXExpression::ObjectExpression(it) => it.serialize(serializer),
            JSXExpression::ParenthesizedExpression(it) => it.serialize(serializer),
            JSXExpression::SequenceExpression(it) => it.serialize(serializer),
            JSXExpression::TaggedTemplateExpression(it) => it.serialize(serializer),
            JSXExpression::ThisExpression(it) => it.serialize(serializer),
            JSXExpression::UnaryExpression(it) => it.serialize(serializer),
            JSXExpression::UpdateExpression(it) => it.serialize(serializer),
            JSXExpression::YieldExpression(it) => it.serialize(serializer),
            JSXExpression::PrivateInExpression(it) => it.serialize(serializer),
            JSXExpression::JSXElement(it) => it.serialize(serializer),
            JSXExpression::JSXFragment(it) => it.serialize(serializer),
            JSXExpression::TSAsExpression(it) => it.serialize(serializer),
            JSXExpression::TSSatisfiesExpression(it) => it.serialize(serializer),
            JSXExpression::TSTypeAssertion(it) => it.serialize(serializer),
            JSXExpression::TSNonNullExpression(it) => it.serialize(serializer),
            JSXExpression::TSInstantiationExpression(it) => it.serialize(serializer),
            JSXExpression::ComputedMemberExpression(it) => it.serialize(serializer),
            JSXExpression::StaticMemberExpression(it) => it.serialize(serializer),
            JSXExpression::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for JSXEmptyExpression {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXEmptyExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for JSXAttributeItem<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            JSXAttributeItem::Attribute(it) => it.serialize(serializer),
            JSXAttributeItem::SpreadAttribute(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for JSXAttribute<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXAttribute");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("name", &self.name);
        state.serialize_field("value", &self.value);
        state.end();
    }
}

impl ESTree for JSXSpreadAttribute<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXSpreadAttribute");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("argument", &self.argument);
        state.end();
    }
}

impl ESTree for JSXAttributeName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            JSXAttributeName::Identifier(it) => it.serialize(serializer),
            JSXAttributeName::NamespacedName(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for JSXAttributeValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            JSXAttributeValue::StringLiteral(it) => it.serialize(serializer),
            JSXAttributeValue::ExpressionContainer(it) => it.serialize(serializer),
            JSXAttributeValue::Element(it) => it.serialize(serializer),
            JSXAttributeValue::Fragment(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for JSXIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXIdentifier");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("name", &self.name);
        state.end();
    }
}

impl ESTree for JSXChild<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            JSXChild::Text(it) => it.serialize(serializer),
            JSXChild::Element(it) => it.serialize(serializer),
            JSXChild::Fragment(it) => it.serialize(serializer),
            JSXChild::ExpressionContainer(it) => it.serialize(serializer),
            JSXChild::Spread(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for JSXSpreadChild<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXSpreadChild");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.end();
    }
}

impl ESTree for JSXText<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSXText");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("value", &self.value);
        state.end();
    }
}

impl ESTree for TSThisParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSThisParameter");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.end();
    }
}

impl ESTree for TSEnumDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSEnumDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("id", &self.id);
        state.serialize_field("members", &self.members);
        state.serialize_field("const", &self.r#const);
        state.serialize_field("declare", &self.declare);
        state.end();
    }
}

impl ESTree for TSEnumMember<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSEnumMember");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("id", &self.id);
        state.serialize_field("initializer", &self.initializer);
        state.end();
    }
}

impl ESTree for TSEnumMemberName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSEnumMemberName::Identifier(it) => it.serialize(serializer),
            TSEnumMemberName::String(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSTypeAnnotation<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSTypeAnnotation");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.end();
    }
}

impl ESTree for TSLiteralType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSLiteralType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("literal", &self.literal);
        state.end();
    }
}

impl ESTree for TSLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSLiteral::BooleanLiteral(it) => it.serialize(serializer),
            TSLiteral::NumericLiteral(it) => it.serialize(serializer),
            TSLiteral::BigIntLiteral(it) => it.serialize(serializer),
            TSLiteral::StringLiteral(it) => it.serialize(serializer),
            TSLiteral::TemplateLiteral(it) => it.serialize(serializer),
            TSLiteral::UnaryExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSType::TSAnyKeyword(it) => it.serialize(serializer),
            TSType::TSBigIntKeyword(it) => it.serialize(serializer),
            TSType::TSBooleanKeyword(it) => it.serialize(serializer),
            TSType::TSIntrinsicKeyword(it) => it.serialize(serializer),
            TSType::TSNeverKeyword(it) => it.serialize(serializer),
            TSType::TSNullKeyword(it) => it.serialize(serializer),
            TSType::TSNumberKeyword(it) => it.serialize(serializer),
            TSType::TSObjectKeyword(it) => it.serialize(serializer),
            TSType::TSStringKeyword(it) => it.serialize(serializer),
            TSType::TSSymbolKeyword(it) => it.serialize(serializer),
            TSType::TSUndefinedKeyword(it) => it.serialize(serializer),
            TSType::TSUnknownKeyword(it) => it.serialize(serializer),
            TSType::TSVoidKeyword(it) => it.serialize(serializer),
            TSType::TSArrayType(it) => it.serialize(serializer),
            TSType::TSConditionalType(it) => it.serialize(serializer),
            TSType::TSConstructorType(it) => it.serialize(serializer),
            TSType::TSFunctionType(it) => it.serialize(serializer),
            TSType::TSImportType(it) => it.serialize(serializer),
            TSType::TSIndexedAccessType(it) => it.serialize(serializer),
            TSType::TSInferType(it) => it.serialize(serializer),
            TSType::TSIntersectionType(it) => it.serialize(serializer),
            TSType::TSLiteralType(it) => it.serialize(serializer),
            TSType::TSMappedType(it) => it.serialize(serializer),
            TSType::TSNamedTupleMember(it) => it.serialize(serializer),
            TSType::TSTemplateLiteralType(it) => it.serialize(serializer),
            TSType::TSThisType(it) => it.serialize(serializer),
            TSType::TSTupleType(it) => it.serialize(serializer),
            TSType::TSTypeLiteral(it) => it.serialize(serializer),
            TSType::TSTypeOperatorType(it) => it.serialize(serializer),
            TSType::TSTypePredicate(it) => it.serialize(serializer),
            TSType::TSTypeQuery(it) => it.serialize(serializer),
            TSType::TSTypeReference(it) => it.serialize(serializer),
            TSType::TSUnionType(it) => it.serialize(serializer),
            TSType::TSParenthesizedType(it) => it.serialize(serializer),
            TSType::JSDocNullableType(it) => it.serialize(serializer),
            TSType::JSDocNonNullableType(it) => it.serialize(serializer),
            TSType::JSDocUnknownType(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSConditionalType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSConditionalType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("checkType", &self.check_type);
        state.serialize_field("extendsType", &self.extends_type);
        state.serialize_field("trueType", &self.true_type);
        state.serialize_field("falseType", &self.false_type);
        state.end();
    }
}

impl ESTree for TSUnionType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSUnionType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("types", &self.types);
        state.end();
    }
}

impl ESTree for TSIntersectionType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSIntersectionType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("types", &self.types);
        state.end();
    }
}

impl ESTree for TSParenthesizedType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSParenthesizedType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.end();
    }
}

impl ESTree for TSTypeOperator<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSTypeOperator");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("operator", &self.operator);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.end();
    }
}

impl ESTree for TSTypeOperatorOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSTypeOperatorOperator::Keyof => "keyof".serialize(serializer),
            TSTypeOperatorOperator::Unique => "unique".serialize(serializer),
            TSTypeOperatorOperator::Readonly => "readonly".serialize(serializer),
        }
    }
}

impl ESTree for TSArrayType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSArrayType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("elementType", &self.element_type);
        state.end();
    }
}

impl ESTree for TSIndexedAccessType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSIndexedAccessType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("objectType", &self.object_type);
        state.serialize_field("indexType", &self.index_type);
        state.end();
    }
}

impl ESTree for TSTupleType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSTupleType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("elementTypes", &self.element_types);
        state.end();
    }
}

impl ESTree for TSNamedTupleMember<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSNamedTupleMember");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("elementType", &self.element_type);
        state.serialize_field("label", &self.label);
        state.serialize_field("optional", &self.optional);
        state.end();
    }
}

impl ESTree for TSOptionalType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSOptionalType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.end();
    }
}

impl ESTree for TSRestType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSRestType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.end();
    }
}

impl ESTree for TSTupleElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSTupleElement::TSOptionalType(it) => it.serialize(serializer),
            TSTupleElement::TSRestType(it) => it.serialize(serializer),
            TSTupleElement::TSAnyKeyword(it) => it.serialize(serializer),
            TSTupleElement::TSBigIntKeyword(it) => it.serialize(serializer),
            TSTupleElement::TSBooleanKeyword(it) => it.serialize(serializer),
            TSTupleElement::TSIntrinsicKeyword(it) => it.serialize(serializer),
            TSTupleElement::TSNeverKeyword(it) => it.serialize(serializer),
            TSTupleElement::TSNullKeyword(it) => it.serialize(serializer),
            TSTupleElement::TSNumberKeyword(it) => it.serialize(serializer),
            TSTupleElement::TSObjectKeyword(it) => it.serialize(serializer),
            TSTupleElement::TSStringKeyword(it) => it.serialize(serializer),
            TSTupleElement::TSSymbolKeyword(it) => it.serialize(serializer),
            TSTupleElement::TSUndefinedKeyword(it) => it.serialize(serializer),
            TSTupleElement::TSUnknownKeyword(it) => it.serialize(serializer),
            TSTupleElement::TSVoidKeyword(it) => it.serialize(serializer),
            TSTupleElement::TSArrayType(it) => it.serialize(serializer),
            TSTupleElement::TSConditionalType(it) => it.serialize(serializer),
            TSTupleElement::TSConstructorType(it) => it.serialize(serializer),
            TSTupleElement::TSFunctionType(it) => it.serialize(serializer),
            TSTupleElement::TSImportType(it) => it.serialize(serializer),
            TSTupleElement::TSIndexedAccessType(it) => it.serialize(serializer),
            TSTupleElement::TSInferType(it) => it.serialize(serializer),
            TSTupleElement::TSIntersectionType(it) => it.serialize(serializer),
            TSTupleElement::TSLiteralType(it) => it.serialize(serializer),
            TSTupleElement::TSMappedType(it) => it.serialize(serializer),
            TSTupleElement::TSNamedTupleMember(it) => it.serialize(serializer),
            TSTupleElement::TSTemplateLiteralType(it) => it.serialize(serializer),
            TSTupleElement::TSThisType(it) => it.serialize(serializer),
            TSTupleElement::TSTupleType(it) => it.serialize(serializer),
            TSTupleElement::TSTypeLiteral(it) => it.serialize(serializer),
            TSTupleElement::TSTypeOperatorType(it) => it.serialize(serializer),
            TSTupleElement::TSTypePredicate(it) => it.serialize(serializer),
            TSTupleElement::TSTypeQuery(it) => it.serialize(serializer),
            TSTupleElement::TSTypeReference(it) => it.serialize(serializer),
            TSTupleElement::TSUnionType(it) => it.serialize(serializer),
            TSTupleElement::TSParenthesizedType(it) => it.serialize(serializer),
            TSTupleElement::JSDocNullableType(it) => it.serialize(serializer),
            TSTupleElement::JSDocNonNullableType(it) => it.serialize(serializer),
            TSTupleElement::JSDocUnknownType(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSAnyKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSAnyKeyword");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for TSStringKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSStringKeyword");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for TSBooleanKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSBooleanKeyword");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for TSNumberKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSNumberKeyword");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for TSNeverKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSNeverKeyword");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for TSIntrinsicKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSIntrinsicKeyword");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for TSUnknownKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSUnknownKeyword");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for TSNullKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSNullKeyword");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for TSUndefinedKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSUndefinedKeyword");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for TSVoidKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSVoidKeyword");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for TSSymbolKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSSymbolKeyword");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for TSThisType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSThisType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for TSObjectKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSObjectKeyword");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for TSBigIntKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSBigIntKeyword");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for TSTypeReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSTypeReference");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("typeName", &self.type_name);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.end();
    }
}

impl ESTree for TSTypeName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSTypeName::IdentifierReference(it) => it.serialize(serializer),
            TSTypeName::QualifiedName(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSQualifiedName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSQualifiedName");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("left", &self.left);
        state.serialize_field("right", &self.right);
        state.end();
    }
}

impl ESTree for TSTypeParameterInstantiation<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSTypeParameterInstantiation");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("params", &self.params);
        state.end();
    }
}

impl ESTree for TSTypeParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSTypeParameter");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("name", &self.name);
        state.serialize_field("constraint", &self.constraint);
        state.serialize_field("default", &self.default);
        state.serialize_field("in", &self.r#in);
        state.serialize_field("out", &self.out);
        state.serialize_field("const", &self.r#const);
        state.end();
    }
}

impl ESTree for TSTypeParameterDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSTypeParameterDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("params", &self.params);
        state.end();
    }
}

impl ESTree for TSTypeAliasDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSTypeAliasDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("id", &self.id);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("declare", &self.declare);
        state.end();
    }
}

impl ESTree for TSAccessibility {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSAccessibility::Private => "private".serialize(serializer),
            TSAccessibility::Protected => "protected".serialize(serializer),
            TSAccessibility::Public => "public".serialize(serializer),
        }
    }
}

impl ESTree for TSClassImplements<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSClassImplements");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.end();
    }
}

impl ESTree for TSInterfaceDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSInterfaceDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("id", &self.id);
        state.serialize_field("extends", &self.extends);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("body", &self.body);
        state.serialize_field("declare", &self.declare);
        state.end();
    }
}

impl ESTree for TSInterfaceBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSInterfaceBody");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for TSPropertySignature<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSPropertySignature");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("computed", &self.computed);
        state.serialize_field("optional", &self.optional);
        state.serialize_field("readonly", &self.readonly);
        state.serialize_field("key", &self.key);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.end();
    }
}

impl ESTree for TSSignature<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSSignature::TSIndexSignature(it) => it.serialize(serializer),
            TSSignature::TSPropertySignature(it) => it.serialize(serializer),
            TSSignature::TSCallSignatureDeclaration(it) => it.serialize(serializer),
            TSSignature::TSConstructSignatureDeclaration(it) => it.serialize(serializer),
            TSSignature::TSMethodSignature(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSIndexSignature<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSIndexSignature");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("parameters", &self.parameters);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("readonly", &self.readonly);
        state.serialize_field("static", &self.r#static);
        state.end();
    }
}

impl ESTree for TSCallSignatureDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSCallSignatureDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("thisParam", &self.this_param);
        state.serialize_field("params", &self.params);
        state.serialize_field("returnType", &self.return_type);
        state.end();
    }
}

impl ESTree for TSMethodSignatureKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSMethodSignatureKind::Method => "method".serialize(serializer),
            TSMethodSignatureKind::Get => "get".serialize(serializer),
            TSMethodSignatureKind::Set => "set".serialize(serializer),
        }
    }
}

impl ESTree for TSMethodSignature<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSMethodSignature");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("key", &self.key);
        state.serialize_field("computed", &self.computed);
        state.serialize_field("optional", &self.optional);
        state.serialize_field("kind", &self.kind);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("thisParam", &self.this_param);
        state.serialize_field("params", &self.params);
        state.serialize_field("returnType", &self.return_type);
        state.end();
    }
}

impl ESTree for TSConstructSignatureDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSConstructSignatureDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("params", &self.params);
        state.serialize_field("returnType", &self.return_type);
        state.end();
    }
}

impl ESTree for TSIndexSignatureName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Identifier");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("name", &self.name);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.end();
    }
}

impl ESTree for TSInterfaceHeritage<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSInterfaceHeritage");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.end();
    }
}

impl ESTree for TSTypePredicate<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSTypePredicate");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("parameterName", &self.parameter_name);
        state.serialize_field("asserts", &self.asserts);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.end();
    }
}

impl ESTree for TSTypePredicateName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSTypePredicateName::Identifier(it) => it.serialize(serializer),
            TSTypePredicateName::This(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSModuleDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSModuleDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("id", &self.id);
        state.serialize_field("body", &self.body);
        state.serialize_field("kind", &self.kind);
        state.serialize_field("declare", &self.declare);
        state.end();
    }
}

impl ESTree for TSModuleDeclarationKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSModuleDeclarationKind::Global => "global".serialize(serializer),
            TSModuleDeclarationKind::Module => "module".serialize(serializer),
            TSModuleDeclarationKind::Namespace => "namespace".serialize(serializer),
        }
    }
}

impl ESTree for TSModuleDeclarationName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSModuleDeclarationName::Identifier(it) => it.serialize(serializer),
            TSModuleDeclarationName::StringLiteral(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSModuleDeclarationBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSModuleDeclarationBody::TSModuleDeclaration(it) => it.serialize(serializer),
            TSModuleDeclarationBody::TSModuleBlock(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSModuleBlock<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSModuleBlock");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field(
            "body",
            &AppendToConcat { array: &self.directives, after: &self.body },
        );
        state.end();
    }
}

impl ESTree for TSTypeLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSTypeLiteral");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("members", &self.members);
        state.end();
    }
}

impl ESTree for TSInferType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSInferType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("typeParameter", &self.type_parameter);
        state.end();
    }
}

impl ESTree for TSTypeQuery<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSTypeQuery");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("exprName", &self.expr_name);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.end();
    }
}

impl ESTree for TSTypeQueryExprName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSTypeQueryExprName::TSImportType(it) => it.serialize(serializer),
            TSTypeQueryExprName::IdentifierReference(it) => it.serialize(serializer),
            TSTypeQueryExprName::QualifiedName(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSImportType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSImportType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("isTypeOf", &self.is_type_of);
        state.serialize_field("parameter", &self.parameter);
        state.serialize_field("qualifier", &self.qualifier);
        state.serialize_field("attributes", &self.attributes);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.end();
    }
}

impl ESTree for TSImportAttributes<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSImportAttributes");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("attributesKeyword", &self.attributes_keyword);
        state.serialize_field("elements", &self.elements);
        state.end();
    }
}

impl ESTree for TSImportAttribute<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSImportAttribute");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("name", &self.name);
        state.serialize_field("value", &self.value);
        state.end();
    }
}

impl ESTree for TSImportAttributeName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSImportAttributeName::Identifier(it) => it.serialize(serializer),
            TSImportAttributeName::StringLiteral(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSFunctionType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSFunctionType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("thisParam", &self.this_param);
        state.serialize_field("params", &self.params);
        state.serialize_field("returnType", &self.return_type);
        state.end();
    }
}

impl ESTree for TSConstructorType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSConstructorType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("abstract", &self.r#abstract);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("params", &self.params);
        state.serialize_field("returnType", &self.return_type);
        state.end();
    }
}

impl ESTree for TSMappedType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSMappedType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("typeParameter", &self.type_parameter);
        state.serialize_field("nameType", &self.name_type);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("optional", &self.optional);
        state.serialize_field("readonly", &self.readonly);
        state.end();
    }
}

impl ESTree for TSMappedTypeModifierOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSMappedTypeModifierOperator::True => "true".serialize(serializer),
            TSMappedTypeModifierOperator::Plus => "+".serialize(serializer),
            TSMappedTypeModifierOperator::Minus => "-".serialize(serializer),
            TSMappedTypeModifierOperator::None => "none".serialize(serializer),
        }
    }
}

impl ESTree for TSTemplateLiteralType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSTemplateLiteralType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("quasis", &self.quasis);
        state.serialize_field("types", &self.types);
        state.end();
    }
}

impl ESTree for TSAsExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSAsExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.end();
    }
}

impl ESTree for TSSatisfiesExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSSatisfiesExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.end();
    }
}

impl ESTree for TSTypeAssertion<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSTypeAssertion");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.end();
    }
}

impl ESTree for TSImportEqualsDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSImportEqualsDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("id", &self.id);
        state.serialize_field("moduleReference", &self.module_reference);
        state.serialize_field("importKind", &self.import_kind);
        state.end();
    }
}

impl ESTree for TSModuleReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            TSModuleReference::ExternalModuleReference(it) => it.serialize(serializer),
            TSModuleReference::IdentifierReference(it) => it.serialize(serializer),
            TSModuleReference::QualifiedName(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSExternalModuleReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSExternalModuleReference");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.end();
    }
}

impl ESTree for TSNonNullExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSNonNullExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.end();
    }
}

impl ESTree for Decorator<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Decorator");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.end();
    }
}

impl ESTree for TSExportAssignment<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSExportAssignment");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.end();
    }
}

impl ESTree for TSNamespaceExportDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSNamespaceExportDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("id", &self.id);
        state.end();
    }
}

impl ESTree for TSInstantiationExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "TSInstantiationExpression");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("expression", &self.expression);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.end();
    }
}

impl ESTree for ImportOrExportKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            ImportOrExportKind::Value => "value".serialize(serializer),
            ImportOrExportKind::Type => "type".serialize(serializer),
        }
    }
}

impl ESTree for JSDocNullableType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSDocNullableType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("postfix", &self.postfix);
        state.end();
    }
}

impl ESTree for JSDocNonNullableType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSDocNonNullableType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("postfix", &self.postfix);
        state.end();
    }
}

impl ESTree for JSDocUnknownType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "JSDocUnknownType");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}
