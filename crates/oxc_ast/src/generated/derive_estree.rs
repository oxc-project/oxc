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
            Self::BooleanLiteral(it) => it.serialize(serializer),
            Self::NullLiteral(it) => it.serialize(serializer),
            Self::NumericLiteral(it) => it.serialize(serializer),
            Self::BigIntLiteral(it) => it.serialize(serializer),
            Self::RegExpLiteral(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
            Self::TemplateLiteral(it) => it.serialize(serializer),
            Self::Identifier(it) => it.serialize(serializer),
            Self::MetaProperty(it) => it.serialize(serializer),
            Self::Super(it) => it.serialize(serializer),
            Self::ArrayExpression(it) => it.serialize(serializer),
            Self::ArrowFunctionExpression(it) => it.serialize(serializer),
            Self::AssignmentExpression(it) => it.serialize(serializer),
            Self::AwaitExpression(it) => it.serialize(serializer),
            Self::BinaryExpression(it) => it.serialize(serializer),
            Self::CallExpression(it) => it.serialize(serializer),
            Self::ChainExpression(it) => it.serialize(serializer),
            Self::ClassExpression(it) => it.serialize(serializer),
            Self::ConditionalExpression(it) => it.serialize(serializer),
            Self::FunctionExpression(it) => it.serialize(serializer),
            Self::ImportExpression(it) => it.serialize(serializer),
            Self::LogicalExpression(it) => it.serialize(serializer),
            Self::NewExpression(it) => it.serialize(serializer),
            Self::ObjectExpression(it) => it.serialize(serializer),
            Self::ParenthesizedExpression(it) => it.serialize(serializer),
            Self::SequenceExpression(it) => it.serialize(serializer),
            Self::TaggedTemplateExpression(it) => it.serialize(serializer),
            Self::ThisExpression(it) => it.serialize(serializer),
            Self::UnaryExpression(it) => it.serialize(serializer),
            Self::UpdateExpression(it) => it.serialize(serializer),
            Self::YieldExpression(it) => it.serialize(serializer),
            Self::PrivateInExpression(it) => it.serialize(serializer),
            Self::JSXElement(it) => it.serialize(serializer),
            Self::JSXFragment(it) => it.serialize(serializer),
            Self::TSAsExpression(it) => it.serialize(serializer),
            Self::TSSatisfiesExpression(it) => it.serialize(serializer),
            Self::TSTypeAssertion(it) => it.serialize(serializer),
            Self::TSNonNullExpression(it) => it.serialize(serializer),
            Self::TSInstantiationExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
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
            Self::SpreadElement(it) => it.serialize(serializer),
            Self::Elision(it) => it.serialize(serializer),
            Self::BooleanLiteral(it) => it.serialize(serializer),
            Self::NullLiteral(it) => it.serialize(serializer),
            Self::NumericLiteral(it) => it.serialize(serializer),
            Self::BigIntLiteral(it) => it.serialize(serializer),
            Self::RegExpLiteral(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
            Self::TemplateLiteral(it) => it.serialize(serializer),
            Self::Identifier(it) => it.serialize(serializer),
            Self::MetaProperty(it) => it.serialize(serializer),
            Self::Super(it) => it.serialize(serializer),
            Self::ArrayExpression(it) => it.serialize(serializer),
            Self::ArrowFunctionExpression(it) => it.serialize(serializer),
            Self::AssignmentExpression(it) => it.serialize(serializer),
            Self::AwaitExpression(it) => it.serialize(serializer),
            Self::BinaryExpression(it) => it.serialize(serializer),
            Self::CallExpression(it) => it.serialize(serializer),
            Self::ChainExpression(it) => it.serialize(serializer),
            Self::ClassExpression(it) => it.serialize(serializer),
            Self::ConditionalExpression(it) => it.serialize(serializer),
            Self::FunctionExpression(it) => it.serialize(serializer),
            Self::ImportExpression(it) => it.serialize(serializer),
            Self::LogicalExpression(it) => it.serialize(serializer),
            Self::NewExpression(it) => it.serialize(serializer),
            Self::ObjectExpression(it) => it.serialize(serializer),
            Self::ParenthesizedExpression(it) => it.serialize(serializer),
            Self::SequenceExpression(it) => it.serialize(serializer),
            Self::TaggedTemplateExpression(it) => it.serialize(serializer),
            Self::ThisExpression(it) => it.serialize(serializer),
            Self::UnaryExpression(it) => it.serialize(serializer),
            Self::UpdateExpression(it) => it.serialize(serializer),
            Self::YieldExpression(it) => it.serialize(serializer),
            Self::PrivateInExpression(it) => it.serialize(serializer),
            Self::JSXElement(it) => it.serialize(serializer),
            Self::JSXFragment(it) => it.serialize(serializer),
            Self::TSAsExpression(it) => it.serialize(serializer),
            Self::TSSatisfiesExpression(it) => it.serialize(serializer),
            Self::TSTypeAssertion(it) => it.serialize(serializer),
            Self::TSNonNullExpression(it) => it.serialize(serializer),
            Self::TSInstantiationExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
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
            Self::ObjectProperty(it) => it.serialize(serializer),
            Self::SpreadProperty(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for PropertyKey<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::StaticIdentifier(it) => it.serialize(serializer),
            Self::PrivateIdentifier(it) => it.serialize(serializer),
            Self::BooleanLiteral(it) => it.serialize(serializer),
            Self::NullLiteral(it) => it.serialize(serializer),
            Self::NumericLiteral(it) => it.serialize(serializer),
            Self::BigIntLiteral(it) => it.serialize(serializer),
            Self::RegExpLiteral(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
            Self::TemplateLiteral(it) => it.serialize(serializer),
            Self::Identifier(it) => it.serialize(serializer),
            Self::MetaProperty(it) => it.serialize(serializer),
            Self::Super(it) => it.serialize(serializer),
            Self::ArrayExpression(it) => it.serialize(serializer),
            Self::ArrowFunctionExpression(it) => it.serialize(serializer),
            Self::AssignmentExpression(it) => it.serialize(serializer),
            Self::AwaitExpression(it) => it.serialize(serializer),
            Self::BinaryExpression(it) => it.serialize(serializer),
            Self::CallExpression(it) => it.serialize(serializer),
            Self::ChainExpression(it) => it.serialize(serializer),
            Self::ClassExpression(it) => it.serialize(serializer),
            Self::ConditionalExpression(it) => it.serialize(serializer),
            Self::FunctionExpression(it) => it.serialize(serializer),
            Self::ImportExpression(it) => it.serialize(serializer),
            Self::LogicalExpression(it) => it.serialize(serializer),
            Self::NewExpression(it) => it.serialize(serializer),
            Self::ObjectExpression(it) => it.serialize(serializer),
            Self::ParenthesizedExpression(it) => it.serialize(serializer),
            Self::SequenceExpression(it) => it.serialize(serializer),
            Self::TaggedTemplateExpression(it) => it.serialize(serializer),
            Self::ThisExpression(it) => it.serialize(serializer),
            Self::UnaryExpression(it) => it.serialize(serializer),
            Self::UpdateExpression(it) => it.serialize(serializer),
            Self::YieldExpression(it) => it.serialize(serializer),
            Self::PrivateInExpression(it) => it.serialize(serializer),
            Self::JSXElement(it) => it.serialize(serializer),
            Self::JSXFragment(it) => it.serialize(serializer),
            Self::TSAsExpression(it) => it.serialize(serializer),
            Self::TSSatisfiesExpression(it) => it.serialize(serializer),
            Self::TSTypeAssertion(it) => it.serialize(serializer),
            Self::TSNonNullExpression(it) => it.serialize(serializer),
            Self::TSInstantiationExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for PropertyKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Init => "init".serialize(serializer),
            Self::Get => "get".serialize(serializer),
            Self::Set => "set".serialize(serializer),
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
        state.serialize_ts_field("typeParameters", &self.type_parameters);
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
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
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
        state.serialize_ts_field("typeParameters", &self.type_parameters);
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
        state.serialize_ts_field("typeParameters", &self.type_parameters);
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
            Self::SpreadElement(it) => it.serialize(serializer),
            Self::BooleanLiteral(it) => it.serialize(serializer),
            Self::NullLiteral(it) => it.serialize(serializer),
            Self::NumericLiteral(it) => it.serialize(serializer),
            Self::BigIntLiteral(it) => it.serialize(serializer),
            Self::RegExpLiteral(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
            Self::TemplateLiteral(it) => it.serialize(serializer),
            Self::Identifier(it) => it.serialize(serializer),
            Self::MetaProperty(it) => it.serialize(serializer),
            Self::Super(it) => it.serialize(serializer),
            Self::ArrayExpression(it) => it.serialize(serializer),
            Self::ArrowFunctionExpression(it) => it.serialize(serializer),
            Self::AssignmentExpression(it) => it.serialize(serializer),
            Self::AwaitExpression(it) => it.serialize(serializer),
            Self::BinaryExpression(it) => it.serialize(serializer),
            Self::CallExpression(it) => it.serialize(serializer),
            Self::ChainExpression(it) => it.serialize(serializer),
            Self::ClassExpression(it) => it.serialize(serializer),
            Self::ConditionalExpression(it) => it.serialize(serializer),
            Self::FunctionExpression(it) => it.serialize(serializer),
            Self::ImportExpression(it) => it.serialize(serializer),
            Self::LogicalExpression(it) => it.serialize(serializer),
            Self::NewExpression(it) => it.serialize(serializer),
            Self::ObjectExpression(it) => it.serialize(serializer),
            Self::ParenthesizedExpression(it) => it.serialize(serializer),
            Self::SequenceExpression(it) => it.serialize(serializer),
            Self::TaggedTemplateExpression(it) => it.serialize(serializer),
            Self::ThisExpression(it) => it.serialize(serializer),
            Self::UnaryExpression(it) => it.serialize(serializer),
            Self::UpdateExpression(it) => it.serialize(serializer),
            Self::YieldExpression(it) => it.serialize(serializer),
            Self::PrivateInExpression(it) => it.serialize(serializer),
            Self::JSXElement(it) => it.serialize(serializer),
            Self::JSXFragment(it) => it.serialize(serializer),
            Self::TSAsExpression(it) => it.serialize(serializer),
            Self::TSSatisfiesExpression(it) => it.serialize(serializer),
            Self::TSTypeAssertion(it) => it.serialize(serializer),
            Self::TSNonNullExpression(it) => it.serialize(serializer),
            Self::TSInstantiationExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
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
            Self::AssignmentTargetIdentifier(it) => it.serialize(serializer),
            Self::TSAsExpression(it) => it.serialize(serializer),
            Self::TSSatisfiesExpression(it) => it.serialize(serializer),
            Self::TSNonNullExpression(it) => it.serialize(serializer),
            Self::TSTypeAssertion(it) => it.serialize(serializer),
            Self::TSInstantiationExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
            Self::ArrayAssignmentTarget(it) => it.serialize(serializer),
            Self::ObjectAssignmentTarget(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for SimpleAssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::AssignmentTargetIdentifier(it) => it.serialize(serializer),
            Self::TSAsExpression(it) => it.serialize(serializer),
            Self::TSSatisfiesExpression(it) => it.serialize(serializer),
            Self::TSNonNullExpression(it) => it.serialize(serializer),
            Self::TSTypeAssertion(it) => it.serialize(serializer),
            Self::TSInstantiationExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for AssignmentTargetPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::ArrayAssignmentTarget(it) => it.serialize(serializer),
            Self::ObjectAssignmentTarget(it) => it.serialize(serializer),
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
            Self::AssignmentTargetWithDefault(it) => it.serialize(serializer),
            Self::AssignmentTargetIdentifier(it) => it.serialize(serializer),
            Self::TSAsExpression(it) => it.serialize(serializer),
            Self::TSSatisfiesExpression(it) => it.serialize(serializer),
            Self::TSNonNullExpression(it) => it.serialize(serializer),
            Self::TSTypeAssertion(it) => it.serialize(serializer),
            Self::TSInstantiationExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
            Self::ArrayAssignmentTarget(it) => it.serialize(serializer),
            Self::ObjectAssignmentTarget(it) => it.serialize(serializer),
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
            Self::AssignmentTargetPropertyIdentifier(it) => it.serialize(serializer),
            Self::AssignmentTargetPropertyProperty(it) => it.serialize(serializer),
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
            Self::CallExpression(it) => it.serialize(serializer),
            Self::TSNonNullExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
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
            Self::BlockStatement(it) => it.serialize(serializer),
            Self::BreakStatement(it) => it.serialize(serializer),
            Self::ContinueStatement(it) => it.serialize(serializer),
            Self::DebuggerStatement(it) => it.serialize(serializer),
            Self::DoWhileStatement(it) => it.serialize(serializer),
            Self::EmptyStatement(it) => it.serialize(serializer),
            Self::ExpressionStatement(it) => it.serialize(serializer),
            Self::ForInStatement(it) => it.serialize(serializer),
            Self::ForOfStatement(it) => it.serialize(serializer),
            Self::ForStatement(it) => it.serialize(serializer),
            Self::IfStatement(it) => it.serialize(serializer),
            Self::LabeledStatement(it) => it.serialize(serializer),
            Self::ReturnStatement(it) => it.serialize(serializer),
            Self::SwitchStatement(it) => it.serialize(serializer),
            Self::ThrowStatement(it) => it.serialize(serializer),
            Self::TryStatement(it) => it.serialize(serializer),
            Self::WhileStatement(it) => it.serialize(serializer),
            Self::WithStatement(it) => it.serialize(serializer),
            Self::VariableDeclaration(it) => it.serialize(serializer),
            Self::FunctionDeclaration(it) => it.serialize(serializer),
            Self::ClassDeclaration(it) => it.serialize(serializer),
            Self::TSTypeAliasDeclaration(it) => it.serialize(serializer),
            Self::TSInterfaceDeclaration(it) => it.serialize(serializer),
            Self::TSEnumDeclaration(it) => it.serialize(serializer),
            Self::TSModuleDeclaration(it) => it.serialize(serializer),
            Self::TSImportEqualsDeclaration(it) => it.serialize(serializer),
            Self::ImportDeclaration(it) => it.serialize(serializer),
            Self::ExportAllDeclaration(it) => it.serialize(serializer),
            Self::ExportDefaultDeclaration(it) => it.serialize(serializer),
            Self::ExportNamedDeclaration(it) => it.serialize(serializer),
            Self::TSExportAssignment(it) => it.serialize(serializer),
            Self::TSNamespaceExportDeclaration(it) => it.serialize(serializer),
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
            Self::VariableDeclaration(it) => it.serialize(serializer),
            Self::FunctionDeclaration(it) => it.serialize(serializer),
            Self::ClassDeclaration(it) => it.serialize(serializer),
            Self::TSTypeAliasDeclaration(it) => it.serialize(serializer),
            Self::TSInterfaceDeclaration(it) => it.serialize(serializer),
            Self::TSEnumDeclaration(it) => it.serialize(serializer),
            Self::TSModuleDeclaration(it) => it.serialize(serializer),
            Self::TSImportEqualsDeclaration(it) => it.serialize(serializer),
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
        state.serialize_ts_field("declare", &self.declare);
        state.end();
    }
}

impl ESTree for VariableDeclarationKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Var => "var".serialize(serializer),
            Self::Const => "const".serialize(serializer),
            Self::Let => "let".serialize(serializer),
            Self::Using => "using".serialize(serializer),
            Self::AwaitUsing => "await using".serialize(serializer),
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
        state.serialize_ts_field("definite", &self.definite);
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
            Self::VariableDeclaration(it) => it.serialize(serializer),
            Self::BooleanLiteral(it) => it.serialize(serializer),
            Self::NullLiteral(it) => it.serialize(serializer),
            Self::NumericLiteral(it) => it.serialize(serializer),
            Self::BigIntLiteral(it) => it.serialize(serializer),
            Self::RegExpLiteral(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
            Self::TemplateLiteral(it) => it.serialize(serializer),
            Self::Identifier(it) => it.serialize(serializer),
            Self::MetaProperty(it) => it.serialize(serializer),
            Self::Super(it) => it.serialize(serializer),
            Self::ArrayExpression(it) => it.serialize(serializer),
            Self::ArrowFunctionExpression(it) => it.serialize(serializer),
            Self::AssignmentExpression(it) => it.serialize(serializer),
            Self::AwaitExpression(it) => it.serialize(serializer),
            Self::BinaryExpression(it) => it.serialize(serializer),
            Self::CallExpression(it) => it.serialize(serializer),
            Self::ChainExpression(it) => it.serialize(serializer),
            Self::ClassExpression(it) => it.serialize(serializer),
            Self::ConditionalExpression(it) => it.serialize(serializer),
            Self::FunctionExpression(it) => it.serialize(serializer),
            Self::ImportExpression(it) => it.serialize(serializer),
            Self::LogicalExpression(it) => it.serialize(serializer),
            Self::NewExpression(it) => it.serialize(serializer),
            Self::ObjectExpression(it) => it.serialize(serializer),
            Self::ParenthesizedExpression(it) => it.serialize(serializer),
            Self::SequenceExpression(it) => it.serialize(serializer),
            Self::TaggedTemplateExpression(it) => it.serialize(serializer),
            Self::ThisExpression(it) => it.serialize(serializer),
            Self::UnaryExpression(it) => it.serialize(serializer),
            Self::UpdateExpression(it) => it.serialize(serializer),
            Self::YieldExpression(it) => it.serialize(serializer),
            Self::PrivateInExpression(it) => it.serialize(serializer),
            Self::JSXElement(it) => it.serialize(serializer),
            Self::JSXFragment(it) => it.serialize(serializer),
            Self::TSAsExpression(it) => it.serialize(serializer),
            Self::TSSatisfiesExpression(it) => it.serialize(serializer),
            Self::TSTypeAssertion(it) => it.serialize(serializer),
            Self::TSNonNullExpression(it) => it.serialize(serializer),
            Self::TSInstantiationExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
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
            Self::VariableDeclaration(it) => it.serialize(serializer),
            Self::AssignmentTargetIdentifier(it) => it.serialize(serializer),
            Self::TSAsExpression(it) => it.serialize(serializer),
            Self::TSSatisfiesExpression(it) => it.serialize(serializer),
            Self::TSNonNullExpression(it) => it.serialize(serializer),
            Self::TSTypeAssertion(it) => it.serialize(serializer),
            Self::TSInstantiationExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
            Self::ArrayAssignmentTarget(it) => it.serialize(serializer),
            Self::ObjectAssignmentTarget(it) => it.serialize(serializer),
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
        state.serialize_ts_field("typeAnnotation", &self.pattern.type_annotation);
        state.serialize_ts_field("optional", &self.pattern.optional);
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
        state.serialize_ts_field("typeAnnotation", &self.type_annotation);
        state.serialize_ts_field("optional", &self.optional);
        state.end();
    }
}

impl ESTree for BindingPatternKind<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::BindingIdentifier(it) => it.serialize(serializer),
            Self::ObjectPattern(it) => it.serialize(serializer),
            Self::ArrayPattern(it) => it.serialize(serializer),
            Self::AssignmentPattern(it) => it.serialize(serializer),
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
        state.serialize_ts_field("declare", &self.declare);
        state.serialize_ts_field("typeParameters", &self.type_parameters);
        state.serialize_ts_field("thisParam", &self.this_param);
        state.serialize_ts_field("returnType", &self.return_type);
        state.end();
    }
}

impl ESTree for FunctionType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::FunctionDeclaration => "FunctionDeclaration".serialize(serializer),
            Self::FunctionExpression => "FunctionExpression".serialize(serializer),
            Self::TSDeclareFunction => "TSDeclareFunction".serialize(serializer),
            Self::TSEmptyBodyFunctionExpression => {
                "TSEmptyBodyFunctionExpression".serialize(serializer)
            }
        }
    }
}

impl ESTree for FormalParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        self.pattern.kind.serialize(FlatStructSerializer(&mut state));
        state.serialize_ts_field("typeAnnotation", &self.pattern.type_annotation);
        state.serialize_ts_field("optional", &self.pattern.optional);
        state.serialize_ts_field("decorators", &self.decorators);
        state.serialize_ts_field("accessibility", &self.accessibility);
        state.serialize_ts_field("readonly", &self.readonly);
        state.serialize_ts_field("override", &self.r#override);
        state.end();
    }
}

impl ESTree for FormalParameterKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::FormalParameter => "FormalParameter".serialize(serializer),
            Self::UniqueFormalParameters => "UniqueFormalParameters".serialize(serializer),
            Self::ArrowFormalParameters => "ArrowFormalParameters".serialize(serializer),
            Self::Signature => "Signature".serialize(serializer),
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
        state.serialize_ts_field("typeParameters", &self.type_parameters);
        state.serialize_ts_field("returnType", &self.return_type);
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
        state.serialize_ts_field("decorators", &self.decorators);
        state.serialize_ts_field("typeParameters", &self.type_parameters);
        state.serialize_ts_field("superTypeParameters", &self.super_type_parameters);
        state.serialize_ts_field("implements", &self.implements);
        state.serialize_ts_field("abstract", &self.r#abstract);
        state.serialize_ts_field("declare", &self.declare);
        state.end();
    }
}

impl ESTree for ClassType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::ClassDeclaration => "ClassDeclaration".serialize(serializer),
            Self::ClassExpression => "ClassExpression".serialize(serializer),
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
            Self::StaticBlock(it) => it.serialize(serializer),
            Self::MethodDefinition(it) => it.serialize(serializer),
            Self::PropertyDefinition(it) => it.serialize(serializer),
            Self::AccessorProperty(it) => it.serialize(serializer),
            Self::TSIndexSignature(it) => it.serialize(serializer),
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
        state.serialize_ts_field("decorators", &self.decorators);
        state.serialize_ts_field("override", &self.r#override);
        state.serialize_ts_field("optional", &self.optional);
        state.serialize_ts_field("accessibility", &self.accessibility);
        state.end();
    }
}

impl ESTree for MethodDefinitionType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::MethodDefinition => "MethodDefinition".serialize(serializer),
            Self::TSAbstractMethodDefinition => "TSAbstractMethodDefinition".serialize(serializer),
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
        state.serialize_ts_field("decorators", &self.decorators);
        state.serialize_ts_field("declare", &self.declare);
        state.serialize_ts_field("override", &self.r#override);
        state.serialize_ts_field("optional", &self.optional);
        state.serialize_ts_field("definite", &self.definite);
        state.serialize_ts_field("readonly", &self.readonly);
        state.serialize_ts_field("typeAnnotation", &self.type_annotation);
        state.serialize_ts_field("accessibility", &self.accessibility);
        state.end();
    }
}

impl ESTree for PropertyDefinitionType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::PropertyDefinition => "PropertyDefinition".serialize(serializer),
            Self::TSAbstractPropertyDefinition => {
                "TSAbstractPropertyDefinition".serialize(serializer)
            }
        }
    }
}

impl ESTree for MethodDefinitionKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Constructor => "constructor".serialize(serializer),
            Self::Method => "method".serialize(serializer),
            Self::Get => "get".serialize(serializer),
            Self::Set => "set".serialize(serializer),
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
            Self::ImportDeclaration(it) => it.serialize(serializer),
            Self::ExportAllDeclaration(it) => it.serialize(serializer),
            Self::ExportDefaultDeclaration(it) => it.serialize(serializer),
            Self::ExportNamedDeclaration(it) => it.serialize(serializer),
            Self::TSExportAssignment(it) => it.serialize(serializer),
            Self::TSNamespaceExportDeclaration(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for AccessorPropertyType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::AccessorProperty => "AccessorProperty".serialize(serializer),
            Self::TSAbstractAccessorProperty => "TSAbstractAccessorProperty".serialize(serializer),
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
        state.serialize_ts_field("decorators", &self.decorators);
        state.serialize_ts_field("definite", &self.definite);
        state.serialize_ts_field("typeAnnotation", &self.type_annotation);
        state.serialize_ts_field("accessibility", &self.accessibility);
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
        state.serialize_field("attributes", &crate::serialize::ImportDeclarationWithClause(self));
        state.serialize_ts_field("importKind", &self.import_kind);
        state.end();
    }
}

impl ESTree for ImportPhase {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Source => "source".serialize(serializer),
            Self::Defer => "defer".serialize(serializer),
        }
    }
}

impl ESTree for ImportDeclarationSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::ImportSpecifier(it) => it.serialize(serializer),
            Self::ImportDefaultSpecifier(it) => it.serialize(serializer),
            Self::ImportNamespaceSpecifier(it) => it.serialize(serializer),
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
        state.serialize_ts_field("importKind", &self.import_kind);
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
            Self::Identifier(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for ExportDefaultDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ExportDefaultDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("declaration", &self.declaration);
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
        state.serialize_ts_field("exportKind", &self.export_kind);
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
        state.serialize_ts_field("exportKind", &self.export_kind);
        state.end();
    }
}

impl ESTree for ExportDefaultDeclarationKind<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::FunctionDeclaration(it) => it.serialize(serializer),
            Self::ClassDeclaration(it) => it.serialize(serializer),
            Self::TSInterfaceDeclaration(it) => it.serialize(serializer),
            Self::BooleanLiteral(it) => it.serialize(serializer),
            Self::NullLiteral(it) => it.serialize(serializer),
            Self::NumericLiteral(it) => it.serialize(serializer),
            Self::BigIntLiteral(it) => it.serialize(serializer),
            Self::RegExpLiteral(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
            Self::TemplateLiteral(it) => it.serialize(serializer),
            Self::Identifier(it) => it.serialize(serializer),
            Self::MetaProperty(it) => it.serialize(serializer),
            Self::Super(it) => it.serialize(serializer),
            Self::ArrayExpression(it) => it.serialize(serializer),
            Self::ArrowFunctionExpression(it) => it.serialize(serializer),
            Self::AssignmentExpression(it) => it.serialize(serializer),
            Self::AwaitExpression(it) => it.serialize(serializer),
            Self::BinaryExpression(it) => it.serialize(serializer),
            Self::CallExpression(it) => it.serialize(serializer),
            Self::ChainExpression(it) => it.serialize(serializer),
            Self::ClassExpression(it) => it.serialize(serializer),
            Self::ConditionalExpression(it) => it.serialize(serializer),
            Self::FunctionExpression(it) => it.serialize(serializer),
            Self::ImportExpression(it) => it.serialize(serializer),
            Self::LogicalExpression(it) => it.serialize(serializer),
            Self::NewExpression(it) => it.serialize(serializer),
            Self::ObjectExpression(it) => it.serialize(serializer),
            Self::ParenthesizedExpression(it) => it.serialize(serializer),
            Self::SequenceExpression(it) => it.serialize(serializer),
            Self::TaggedTemplateExpression(it) => it.serialize(serializer),
            Self::ThisExpression(it) => it.serialize(serializer),
            Self::UnaryExpression(it) => it.serialize(serializer),
            Self::UpdateExpression(it) => it.serialize(serializer),
            Self::YieldExpression(it) => it.serialize(serializer),
            Self::PrivateInExpression(it) => it.serialize(serializer),
            Self::JSXElement(it) => it.serialize(serializer),
            Self::JSXFragment(it) => it.serialize(serializer),
            Self::TSAsExpression(it) => it.serialize(serializer),
            Self::TSSatisfiesExpression(it) => it.serialize(serializer),
            Self::TSTypeAssertion(it) => it.serialize(serializer),
            Self::TSNonNullExpression(it) => it.serialize(serializer),
            Self::TSInstantiationExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for ModuleExportName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::IdentifierName(it) => it.serialize(serializer),
            Self::IdentifierReference(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
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
        state.serialize_ts_field("typeParameters", &self.type_parameters);
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
            Self::EmptyExpression(it) => it.serialize(serializer),
            Self::BooleanLiteral(it) => it.serialize(serializer),
            Self::NullLiteral(it) => it.serialize(serializer),
            Self::NumericLiteral(it) => it.serialize(serializer),
            Self::BigIntLiteral(it) => it.serialize(serializer),
            Self::RegExpLiteral(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
            Self::TemplateLiteral(it) => it.serialize(serializer),
            Self::Identifier(it) => it.serialize(serializer),
            Self::MetaProperty(it) => it.serialize(serializer),
            Self::Super(it) => it.serialize(serializer),
            Self::ArrayExpression(it) => it.serialize(serializer),
            Self::ArrowFunctionExpression(it) => it.serialize(serializer),
            Self::AssignmentExpression(it) => it.serialize(serializer),
            Self::AwaitExpression(it) => it.serialize(serializer),
            Self::BinaryExpression(it) => it.serialize(serializer),
            Self::CallExpression(it) => it.serialize(serializer),
            Self::ChainExpression(it) => it.serialize(serializer),
            Self::ClassExpression(it) => it.serialize(serializer),
            Self::ConditionalExpression(it) => it.serialize(serializer),
            Self::FunctionExpression(it) => it.serialize(serializer),
            Self::ImportExpression(it) => it.serialize(serializer),
            Self::LogicalExpression(it) => it.serialize(serializer),
            Self::NewExpression(it) => it.serialize(serializer),
            Self::ObjectExpression(it) => it.serialize(serializer),
            Self::ParenthesizedExpression(it) => it.serialize(serializer),
            Self::SequenceExpression(it) => it.serialize(serializer),
            Self::TaggedTemplateExpression(it) => it.serialize(serializer),
            Self::ThisExpression(it) => it.serialize(serializer),
            Self::UnaryExpression(it) => it.serialize(serializer),
            Self::UpdateExpression(it) => it.serialize(serializer),
            Self::YieldExpression(it) => it.serialize(serializer),
            Self::PrivateInExpression(it) => it.serialize(serializer),
            Self::JSXElement(it) => it.serialize(serializer),
            Self::JSXFragment(it) => it.serialize(serializer),
            Self::TSAsExpression(it) => it.serialize(serializer),
            Self::TSSatisfiesExpression(it) => it.serialize(serializer),
            Self::TSTypeAssertion(it) => it.serialize(serializer),
            Self::TSNonNullExpression(it) => it.serialize(serializer),
            Self::TSInstantiationExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
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
            Self::Attribute(it) => it.serialize(serializer),
            Self::SpreadAttribute(it) => it.serialize(serializer),
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
            Self::Identifier(it) => it.serialize(serializer),
            Self::NamespacedName(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for JSXAttributeValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::StringLiteral(it) => it.serialize(serializer),
            Self::ExpressionContainer(it) => it.serialize(serializer),
            Self::Element(it) => it.serialize(serializer),
            Self::Fragment(it) => it.serialize(serializer),
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
            Self::Text(it) => it.serialize(serializer),
            Self::Element(it) => it.serialize(serializer),
            Self::Fragment(it) => it.serialize(serializer),
            Self::ExpressionContainer(it) => it.serialize(serializer),
            Self::Spread(it) => it.serialize(serializer),
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
            Self::Identifier(it) => it.serialize(serializer),
            Self::String(it) => it.serialize(serializer),
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
            Self::BooleanLiteral(it) => it.serialize(serializer),
            Self::NumericLiteral(it) => it.serialize(serializer),
            Self::BigIntLiteral(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
            Self::TemplateLiteral(it) => it.serialize(serializer),
            Self::UnaryExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::TSAnyKeyword(it) => it.serialize(serializer),
            Self::TSBigIntKeyword(it) => it.serialize(serializer),
            Self::TSBooleanKeyword(it) => it.serialize(serializer),
            Self::TSIntrinsicKeyword(it) => it.serialize(serializer),
            Self::TSNeverKeyword(it) => it.serialize(serializer),
            Self::TSNullKeyword(it) => it.serialize(serializer),
            Self::TSNumberKeyword(it) => it.serialize(serializer),
            Self::TSObjectKeyword(it) => it.serialize(serializer),
            Self::TSStringKeyword(it) => it.serialize(serializer),
            Self::TSSymbolKeyword(it) => it.serialize(serializer),
            Self::TSUndefinedKeyword(it) => it.serialize(serializer),
            Self::TSUnknownKeyword(it) => it.serialize(serializer),
            Self::TSVoidKeyword(it) => it.serialize(serializer),
            Self::TSArrayType(it) => it.serialize(serializer),
            Self::TSConditionalType(it) => it.serialize(serializer),
            Self::TSConstructorType(it) => it.serialize(serializer),
            Self::TSFunctionType(it) => it.serialize(serializer),
            Self::TSImportType(it) => it.serialize(serializer),
            Self::TSIndexedAccessType(it) => it.serialize(serializer),
            Self::TSInferType(it) => it.serialize(serializer),
            Self::TSIntersectionType(it) => it.serialize(serializer),
            Self::TSLiteralType(it) => it.serialize(serializer),
            Self::TSMappedType(it) => it.serialize(serializer),
            Self::TSNamedTupleMember(it) => it.serialize(serializer),
            Self::TSTemplateLiteralType(it) => it.serialize(serializer),
            Self::TSThisType(it) => it.serialize(serializer),
            Self::TSTupleType(it) => it.serialize(serializer),
            Self::TSTypeLiteral(it) => it.serialize(serializer),
            Self::TSTypeOperatorType(it) => it.serialize(serializer),
            Self::TSTypePredicate(it) => it.serialize(serializer),
            Self::TSTypeQuery(it) => it.serialize(serializer),
            Self::TSTypeReference(it) => it.serialize(serializer),
            Self::TSUnionType(it) => it.serialize(serializer),
            Self::TSParenthesizedType(it) => it.serialize(serializer),
            Self::JSDocNullableType(it) => it.serialize(serializer),
            Self::JSDocNonNullableType(it) => it.serialize(serializer),
            Self::JSDocUnknownType(it) => it.serialize(serializer),
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
            Self::Keyof => "keyof".serialize(serializer),
            Self::Unique => "unique".serialize(serializer),
            Self::Readonly => "readonly".serialize(serializer),
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
            Self::TSOptionalType(it) => it.serialize(serializer),
            Self::TSRestType(it) => it.serialize(serializer),
            Self::TSAnyKeyword(it) => it.serialize(serializer),
            Self::TSBigIntKeyword(it) => it.serialize(serializer),
            Self::TSBooleanKeyword(it) => it.serialize(serializer),
            Self::TSIntrinsicKeyword(it) => it.serialize(serializer),
            Self::TSNeverKeyword(it) => it.serialize(serializer),
            Self::TSNullKeyword(it) => it.serialize(serializer),
            Self::TSNumberKeyword(it) => it.serialize(serializer),
            Self::TSObjectKeyword(it) => it.serialize(serializer),
            Self::TSStringKeyword(it) => it.serialize(serializer),
            Self::TSSymbolKeyword(it) => it.serialize(serializer),
            Self::TSUndefinedKeyword(it) => it.serialize(serializer),
            Self::TSUnknownKeyword(it) => it.serialize(serializer),
            Self::TSVoidKeyword(it) => it.serialize(serializer),
            Self::TSArrayType(it) => it.serialize(serializer),
            Self::TSConditionalType(it) => it.serialize(serializer),
            Self::TSConstructorType(it) => it.serialize(serializer),
            Self::TSFunctionType(it) => it.serialize(serializer),
            Self::TSImportType(it) => it.serialize(serializer),
            Self::TSIndexedAccessType(it) => it.serialize(serializer),
            Self::TSInferType(it) => it.serialize(serializer),
            Self::TSIntersectionType(it) => it.serialize(serializer),
            Self::TSLiteralType(it) => it.serialize(serializer),
            Self::TSMappedType(it) => it.serialize(serializer),
            Self::TSNamedTupleMember(it) => it.serialize(serializer),
            Self::TSTemplateLiteralType(it) => it.serialize(serializer),
            Self::TSThisType(it) => it.serialize(serializer),
            Self::TSTupleType(it) => it.serialize(serializer),
            Self::TSTypeLiteral(it) => it.serialize(serializer),
            Self::TSTypeOperatorType(it) => it.serialize(serializer),
            Self::TSTypePredicate(it) => it.serialize(serializer),
            Self::TSTypeQuery(it) => it.serialize(serializer),
            Self::TSTypeReference(it) => it.serialize(serializer),
            Self::TSUnionType(it) => it.serialize(serializer),
            Self::TSParenthesizedType(it) => it.serialize(serializer),
            Self::JSDocNullableType(it) => it.serialize(serializer),
            Self::JSDocNonNullableType(it) => it.serialize(serializer),
            Self::JSDocUnknownType(it) => it.serialize(serializer),
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
            Self::IdentifierReference(it) => it.serialize(serializer),
            Self::QualifiedName(it) => it.serialize(serializer),
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
            Self::Private => "private".serialize(serializer),
            Self::Protected => "protected".serialize(serializer),
            Self::Public => "public".serialize(serializer),
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
            Self::TSIndexSignature(it) => it.serialize(serializer),
            Self::TSPropertySignature(it) => it.serialize(serializer),
            Self::TSCallSignatureDeclaration(it) => it.serialize(serializer),
            Self::TSConstructSignatureDeclaration(it) => it.serialize(serializer),
            Self::TSMethodSignature(it) => it.serialize(serializer),
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
            Self::Method => "method".serialize(serializer),
            Self::Get => "get".serialize(serializer),
            Self::Set => "set".serialize(serializer),
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
            Self::Identifier(it) => it.serialize(serializer),
            Self::This(it) => it.serialize(serializer),
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
            Self::Global => "global".serialize(serializer),
            Self::Module => "module".serialize(serializer),
            Self::Namespace => "namespace".serialize(serializer),
        }
    }
}

impl ESTree for TSModuleDeclarationName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Identifier(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSModuleDeclarationBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::TSModuleDeclaration(it) => it.serialize(serializer),
            Self::TSModuleBlock(it) => it.serialize(serializer),
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
            Self::TSImportType(it) => it.serialize(serializer),
            Self::IdentifierReference(it) => it.serialize(serializer),
            Self::QualifiedName(it) => it.serialize(serializer),
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
            Self::Identifier(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
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
            Self::True => "true".serialize(serializer),
            Self::Plus => "+".serialize(serializer),
            Self::Minus => "-".serialize(serializer),
            Self::None => "none".serialize(serializer),
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
            Self::ExternalModuleReference(it) => it.serialize(serializer),
            Self::IdentifierReference(it) => it.serialize(serializer),
            Self::QualifiedName(it) => it.serialize(serializer),
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
            Self::Value => "value".serialize(serializer),
            Self::Type => "type".serialize(serializer),
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
