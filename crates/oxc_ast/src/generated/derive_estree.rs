// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`.

#![allow(unused_imports, clippy::match_same_arms, clippy::semicolon_if_nothing_returned)]

use oxc_estree::{
    Concat2, Concat3, ESTree, FlatStructSerializer, JsonSafeString, Serializer, StructSerializer,
};

use crate::ast::comment::*;
use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl ESTree for Program<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        crate::serialize::ProgramConverter(self).serialize(serializer)
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
            Self::V8IntrinsicExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for IdentifierName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Identifier"));
        state.serialize_ts_field("decorators", &crate::serialize::basic::TsEmptyArray(self));
        state.serialize_field("name", &JsonSafeString(self.name.as_str()));
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_ts_field("typeAnnotation", &crate::serialize::basic::TsNull(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for IdentifierReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Identifier"));
        state.serialize_ts_field("decorators", &crate::serialize::basic::TsEmptyArray(self));
        state.serialize_field("name", &JsonSafeString(self.name.as_str()));
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_ts_field("typeAnnotation", &crate::serialize::basic::TsNull(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for BindingIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Identifier"));
        state.serialize_ts_field("decorators", &crate::serialize::basic::TsEmptyArray(self));
        state.serialize_field("name", &JsonSafeString(self.name.as_str()));
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_ts_field("typeAnnotation", &crate::serialize::basic::TsNull(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for LabelIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Identifier"));
        state.serialize_ts_field("decorators", &crate::serialize::basic::TsEmptyArray(self));
        state.serialize_field("name", &JsonSafeString(self.name.as_str()));
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_ts_field("typeAnnotation", &crate::serialize::basic::TsNull(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ThisExpression {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ThisExpression"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ArrayExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ArrayExpression"));
        state.serialize_field("elements", &self.elements);
        state.serialize_span(self.span);
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
            Self::V8IntrinsicExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for Elision {
    fn serialize<S: Serializer>(&self, serializer: S) {
        crate::serialize::basic::Null(self).serialize(serializer)
    }
}

impl ESTree for ObjectExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ObjectExpression"));
        state.serialize_field("properties", &self.properties);
        state.serialize_span(self.span);
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

impl ESTree for ObjectProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Property"));
        state.serialize_field("kind", &self.kind);
        state.serialize_field("key", &self.key);
        state.serialize_field("value", &self.value);
        state.serialize_field("method", &self.method);
        state.serialize_field("shorthand", &self.shorthand);
        state.serialize_field("computed", &self.computed);
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_span(self.span);
        state.end();
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
            Self::V8IntrinsicExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for PropertyKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Init => JsonSafeString("init").serialize(serializer),
            Self::Get => JsonSafeString("get").serialize(serializer),
            Self::Set => JsonSafeString("set").serialize(serializer),
        }
    }
}

impl ESTree for TemplateLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TemplateLiteral"));
        state.serialize_field("quasis", &self.quasis);
        state.serialize_field("expressions", &self.expressions);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TaggedTemplateExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TaggedTemplateExpression"));
        state.serialize_field("tag", &self.tag);
        state.serialize_ts_field("typeArguments", &self.type_arguments);
        state.serialize_field("quasi", &self.quasi);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TemplateElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        crate::serialize::literal::TemplateElementConverter(self).serialize(serializer)
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
        state.serialize_field("type", &JsonSafeString("MemberExpression"));
        state.serialize_field("object", &self.object);
        state.serialize_field("property", &self.expression);
        state.serialize_field("optional", &self.optional);
        state.serialize_field("computed", &crate::serialize::basic::True(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for StaticMemberExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("MemberExpression"));
        state.serialize_field("object", &self.object);
        state.serialize_field("property", &self.property);
        state.serialize_field("optional", &self.optional);
        state.serialize_field("computed", &crate::serialize::basic::False(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for PrivateFieldExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("MemberExpression"));
        state.serialize_field("object", &self.object);
        state.serialize_field("property", &self.field);
        state.serialize_field("optional", &self.optional);
        state.serialize_field("computed", &crate::serialize::basic::False(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for CallExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("CallExpression"));
        state.serialize_field("callee", &self.callee);
        state.serialize_ts_field("typeArguments", &self.type_arguments);
        state.serialize_field("arguments", &self.arguments);
        state.serialize_field("optional", &self.optional);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for NewExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("NewExpression"));
        state.serialize_field("callee", &self.callee);
        state.serialize_ts_field("typeArguments", &self.type_arguments);
        state.serialize_field("arguments", &self.arguments);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for MetaProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("MetaProperty"));
        state.serialize_field("meta", &self.meta);
        state.serialize_field("property", &self.property);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for SpreadElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("SpreadElement"));
        state.serialize_field("argument", &self.argument);
        state.serialize_span(self.span);
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
            Self::V8IntrinsicExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for UpdateExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("UpdateExpression"));
        state.serialize_field("operator", &self.operator);
        state.serialize_field("prefix", &self.prefix);
        state.serialize_field("argument", &self.argument);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for UnaryExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("UnaryExpression"));
        state.serialize_field("operator", &self.operator);
        state.serialize_field("argument", &self.argument);
        state.serialize_field("prefix", &crate::serialize::basic::True(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for BinaryExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("BinaryExpression"));
        state.serialize_field("left", &self.left);
        state.serialize_field("operator", &self.operator);
        state.serialize_field("right", &self.right);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for PrivateInExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("BinaryExpression"));
        state.serialize_field("left", &self.left);
        state.serialize_field("operator", &crate::serialize::basic::In(self));
        state.serialize_field("right", &self.right);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for LogicalExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("LogicalExpression"));
        state.serialize_field("left", &self.left);
        state.serialize_field("operator", &self.operator);
        state.serialize_field("right", &self.right);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ConditionalExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ConditionalExpression"));
        state.serialize_field("test", &self.test);
        state.serialize_field("consequent", &self.consequent);
        state.serialize_field("alternate", &self.alternate);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for AssignmentExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("AssignmentExpression"));
        state.serialize_field("operator", &self.operator);
        state.serialize_field("left", &self.left);
        state.serialize_field("right", &self.right);
        state.serialize_span(self.span);
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
        state.serialize_field("type", &JsonSafeString("ArrayPattern"));
        state.serialize_ts_field("decorators", &crate::serialize::basic::TsEmptyArray(self));
        state.serialize_field("elements", &Concat2(&self.elements, &self.rest));
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_ts_field("typeAnnotation", &crate::serialize::basic::TsNull(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ObjectAssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ObjectPattern"));
        state.serialize_ts_field("decorators", &crate::serialize::basic::TsEmptyArray(self));
        state.serialize_field("properties", &Concat2(&self.properties, &self.rest));
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_ts_field("typeAnnotation", &crate::serialize::basic::TsNull(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for AssignmentTargetRest<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("RestElement"));
        state.serialize_ts_field("decorators", &crate::serialize::basic::TsEmptyArray(self));
        state.serialize_field("argument", &self.target);
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_ts_field("typeAnnotation", &crate::serialize::basic::TsNull(self));
        state.serialize_ts_field("value", &crate::serialize::basic::TsNull(self));
        state.serialize_span(self.span);
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
        state.serialize_field("type", &JsonSafeString("AssignmentPattern"));
        state.serialize_ts_field("decorators", &crate::serialize::basic::TsEmptyArray(self));
        state.serialize_field("left", &self.binding);
        state.serialize_field("right", &self.init);
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_ts_field("typeAnnotation", &crate::serialize::basic::TsNull(self));
        state.serialize_span(self.span);
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
        state.serialize_field("type", &JsonSafeString("Property"));
        state.serialize_field("kind", &crate::serialize::basic::Init(self));
        state.serialize_field("key", &self.binding);
        state.serialize_field(
            "value",
            &crate::serialize::js::AssignmentTargetPropertyIdentifierInit(self),
        );
        state.serialize_field("method", &crate::serialize::basic::False(self));
        state.serialize_field("shorthand", &crate::serialize::basic::True(self));
        state.serialize_field("computed", &crate::serialize::basic::False(self));
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for AssignmentTargetPropertyProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Property"));
        state.serialize_field("kind", &crate::serialize::basic::Init(self));
        state.serialize_field("key", &self.name);
        state.serialize_field("value", &self.binding);
        state.serialize_field("method", &crate::serialize::basic::False(self));
        state.serialize_field("shorthand", &crate::serialize::basic::False(self));
        state.serialize_field("computed", &self.computed);
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for SequenceExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("SequenceExpression"));
        state.serialize_field("expressions", &self.expressions);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for Super {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Super"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for AwaitExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("AwaitExpression"));
        state.serialize_field("argument", &self.argument);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ChainExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ChainExpression"));
        state.serialize_field("expression", &self.expression);
        state.serialize_span(self.span);
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
        crate::serialize::js::ParenthesizedExpressionConverter(self).serialize(serializer)
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
            Self::TSGlobalDeclaration(it) => it.serialize(serializer),
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
        state.serialize_field("type", &JsonSafeString("ExpressionStatement"));
        state.serialize_field("expression", &self.expression);
        state.serialize_field("directive", &self.directive);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for Hashbang<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Hashbang"));
        state.serialize_field("value", &self.value);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for BlockStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("BlockStatement"));
        state.serialize_field("body", &self.body);
        state.serialize_span(self.span);
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
            Self::TSGlobalDeclaration(it) => it.serialize(serializer),
            Self::TSImportEqualsDeclaration(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for VariableDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("VariableDeclaration"));
        state.serialize_field("kind", &self.kind);
        state.serialize_field("declarations", &self.declarations);
        state.serialize_ts_field("declare", &self.declare);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for VariableDeclarationKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Var => JsonSafeString("var").serialize(serializer),
            Self::Let => JsonSafeString("let").serialize(serializer),
            Self::Const => JsonSafeString("const").serialize(serializer),
            Self::Using => JsonSafeString("using").serialize(serializer),
            Self::AwaitUsing => JsonSafeString("await using").serialize(serializer),
        }
    }
}

impl ESTree for VariableDeclarator<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("VariableDeclarator"));
        state.serialize_field("id", &self.id);
        state.serialize_field("init", &self.init);
        state.serialize_ts_field("definite", &self.definite);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for EmptyStatement {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("EmptyStatement"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ExpressionStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ExpressionStatement"));
        state.serialize_field("expression", &self.expression);
        state.serialize_ts_field(
            "directive",
            &crate::serialize::ts::ExpressionStatementDirective(self),
        );
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for IfStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("IfStatement"));
        state.serialize_field("test", &self.test);
        state.serialize_field("consequent", &self.consequent);
        state.serialize_field("alternate", &self.alternate);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for DoWhileStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("DoWhileStatement"));
        state.serialize_field("body", &self.body);
        state.serialize_field("test", &self.test);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for WhileStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("WhileStatement"));
        state.serialize_field("test", &self.test);
        state.serialize_field("body", &self.body);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ForStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ForStatement"));
        state.serialize_field("init", &self.init);
        state.serialize_field("test", &self.test);
        state.serialize_field("update", &self.update);
        state.serialize_field("body", &self.body);
        state.serialize_span(self.span);
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
            Self::V8IntrinsicExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for ForInStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ForInStatement"));
        state.serialize_field("left", &self.left);
        state.serialize_field("right", &self.right);
        state.serialize_field("body", &self.body);
        state.serialize_span(self.span);
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
        state.serialize_field("type", &JsonSafeString("ForOfStatement"));
        state.serialize_field("await", &self.r#await);
        state.serialize_field("left", &self.left);
        state.serialize_field("right", &self.right);
        state.serialize_field("body", &self.body);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ContinueStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ContinueStatement"));
        state.serialize_field("label", &self.label);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for BreakStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("BreakStatement"));
        state.serialize_field("label", &self.label);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ReturnStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ReturnStatement"));
        state.serialize_field("argument", &self.argument);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for WithStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("WithStatement"));
        state.serialize_field("object", &self.object);
        state.serialize_field("body", &self.body);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for SwitchStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("SwitchStatement"));
        state.serialize_field("discriminant", &self.discriminant);
        state.serialize_field("cases", &self.cases);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for SwitchCase<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("SwitchCase"));
        state.serialize_field("test", &self.test);
        state.serialize_field("consequent", &self.consequent);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for LabeledStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("LabeledStatement"));
        state.serialize_field("label", &self.label);
        state.serialize_field("body", &self.body);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ThrowStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ThrowStatement"));
        state.serialize_field("argument", &self.argument);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TryStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TryStatement"));
        state.serialize_field("block", &self.block);
        state.serialize_field("handler", &self.handler);
        state.serialize_field("finalizer", &self.finalizer);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for CatchClause<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("CatchClause"));
        state.serialize_field("param", &self.param);
        state.serialize_field("body", &self.body);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for CatchParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        crate::serialize::js::CatchParameterConverter(self).serialize(serializer)
    }
}

impl ESTree for DebuggerStatement {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("DebuggerStatement"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for BindingPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        crate::serialize::js::BindingPatternConverter(self).serialize(serializer)
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
        state.serialize_field("type", &JsonSafeString("AssignmentPattern"));
        state.serialize_ts_field("decorators", &crate::serialize::basic::TsEmptyArray(self));
        state.serialize_field("left", &self.left);
        state.serialize_field("right", &self.right);
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_ts_field("typeAnnotation", &crate::serialize::basic::TsNull(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ObjectPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ObjectPattern"));
        state.serialize_ts_field("decorators", &crate::serialize::basic::TsEmptyArray(self));
        state.serialize_field("properties", &Concat2(&self.properties, &self.rest));
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_ts_field("typeAnnotation", &crate::serialize::basic::TsNull(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for BindingProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Property"));
        state.serialize_field("kind", &crate::serialize::basic::Init(self));
        state.serialize_field("key", &self.key);
        state.serialize_field("value", &self.value);
        state.serialize_field("method", &crate::serialize::basic::False(self));
        state.serialize_field("shorthand", &self.shorthand);
        state.serialize_field("computed", &self.computed);
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ArrayPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ArrayPattern"));
        state.serialize_ts_field("decorators", &crate::serialize::basic::TsEmptyArray(self));
        state.serialize_field("elements", &Concat2(&self.elements, &self.rest));
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_ts_field("typeAnnotation", &crate::serialize::basic::TsNull(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for BindingRestElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("RestElement"));
        state.serialize_ts_field("decorators", &crate::serialize::basic::TsEmptyArray(self));
        state.serialize_field("argument", &self.argument);
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_ts_field("typeAnnotation", &crate::serialize::basic::TsNull(self));
        state.serialize_ts_field("value", &crate::serialize::basic::TsNull(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for Function<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &self.r#type);
        state.serialize_field("id", &self.id);
        state.serialize_field("generator", &self.generator);
        state.serialize_field("async", &self.r#async);
        state.serialize_ts_field("declare", &self.declare);
        state.serialize_ts_field("typeParameters", &self.type_parameters);
        state.serialize_field("params", &crate::serialize::js::FunctionParams(self));
        state.serialize_ts_field("returnType", &self.return_type);
        state.serialize_field("body", &self.body);
        state.serialize_field("expression", &crate::serialize::basic::False(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for FunctionType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::FunctionDeclaration => {
                JsonSafeString("FunctionDeclaration").serialize(serializer)
            }
            Self::FunctionExpression => JsonSafeString("FunctionExpression").serialize(serializer),
            Self::TSDeclareFunction => JsonSafeString("TSDeclareFunction").serialize(serializer),
            Self::TSEmptyBodyFunctionExpression => {
                JsonSafeString("TSEmptyBodyFunctionExpression").serialize(serializer)
            }
        }
    }
}

impl ESTree for FormalParameters<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        crate::serialize::js::FormalParametersConverter(self).serialize(serializer)
    }
}

impl ESTree for FormalParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        crate::serialize::js::FormalParameterConverter(self).serialize(serializer)
    }
}

impl ESTree for FormalParameterKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::FormalParameter => JsonSafeString("FormalParameter").serialize(serializer),
            Self::UniqueFormalParameters => {
                JsonSafeString("UniqueFormalParameters").serialize(serializer)
            }
            Self::ArrowFormalParameters => {
                JsonSafeString("ArrowFormalParameters").serialize(serializer)
            }
            Self::Signature => JsonSafeString("Signature").serialize(serializer),
        }
    }
}

impl ESTree for FunctionBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("BlockStatement"));
        state.serialize_field("body", &Concat2(&self.directives, &self.statements));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ArrowFunctionExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ArrowFunctionExpression"));
        state.serialize_field("expression", &self.expression);
        state.serialize_field("async", &self.r#async);
        state.serialize_ts_field("typeParameters", &self.type_parameters);
        state.serialize_field("params", &self.params);
        state.serialize_ts_field("returnType", &self.return_type);
        state.serialize_field("body", &crate::serialize::js::ArrowFunctionExpressionBody(self));
        state.serialize_field("id", &crate::serialize::basic::Null(self));
        state.serialize_field("generator", &crate::serialize::basic::False(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for YieldExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("YieldExpression"));
        state.serialize_field("delegate", &self.delegate);
        state.serialize_field("argument", &self.argument);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for Class<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &self.r#type);
        state.serialize_field("decorators", &self.decorators);
        state.serialize_field("id", &self.id);
        state.serialize_ts_field("typeParameters", &self.type_parameters);
        state.serialize_field("superClass", &self.super_class);
        state.serialize_ts_field("superTypeArguments", &self.super_type_arguments);
        state.serialize_ts_field("implements", &self.implements);
        state.serialize_field("body", &self.body);
        state.serialize_ts_field("abstract", &self.r#abstract);
        state.serialize_ts_field("declare", &self.declare);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ClassType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::ClassDeclaration => JsonSafeString("ClassDeclaration").serialize(serializer),
            Self::ClassExpression => JsonSafeString("ClassExpression").serialize(serializer),
        }
    }
}

impl ESTree for ClassBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ClassBody"));
        state.serialize_field("body", &self.body);
        state.serialize_span(self.span);
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
        state.serialize_field("decorators", &self.decorators);
        state.serialize_field("key", &self.key);
        state.serialize_field("value", &self.value);
        state.serialize_field("kind", &self.kind);
        state.serialize_field("computed", &self.computed);
        state.serialize_field("static", &self.r#static);
        state.serialize_ts_field("override", &self.r#override);
        state.serialize_ts_field("optional", &self.optional);
        state.serialize_ts_field("accessibility", &self.accessibility);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for MethodDefinitionType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::MethodDefinition => JsonSafeString("MethodDefinition").serialize(serializer),
            Self::TSAbstractMethodDefinition => {
                JsonSafeString("TSAbstractMethodDefinition").serialize(serializer)
            }
        }
    }
}

impl ESTree for PropertyDefinition<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &self.r#type);
        state.serialize_field("decorators", &self.decorators);
        state.serialize_field("key", &self.key);
        state.serialize_ts_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("value", &self.value);
        state.serialize_field("computed", &self.computed);
        state.serialize_field("static", &self.r#static);
        state.serialize_ts_field("declare", &self.declare);
        state.serialize_ts_field("override", &self.r#override);
        state.serialize_ts_field("optional", &self.optional);
        state.serialize_ts_field("definite", &self.definite);
        state.serialize_ts_field("readonly", &self.readonly);
        state.serialize_ts_field("accessibility", &self.accessibility);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for PropertyDefinitionType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::PropertyDefinition => JsonSafeString("PropertyDefinition").serialize(serializer),
            Self::TSAbstractPropertyDefinition => {
                JsonSafeString("TSAbstractPropertyDefinition").serialize(serializer)
            }
        }
    }
}

impl ESTree for MethodDefinitionKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Constructor => JsonSafeString("constructor").serialize(serializer),
            Self::Method => JsonSafeString("method").serialize(serializer),
            Self::Get => JsonSafeString("get").serialize(serializer),
            Self::Set => JsonSafeString("set").serialize(serializer),
        }
    }
}

impl ESTree for PrivateIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("PrivateIdentifier"));
        state.serialize_field("name", &self.name);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for StaticBlock<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("StaticBlock"));
        state.serialize_field("body", &self.body);
        state.serialize_span(self.span);
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
            Self::AccessorProperty => JsonSafeString("AccessorProperty").serialize(serializer),
            Self::TSAbstractAccessorProperty => {
                JsonSafeString("TSAbstractAccessorProperty").serialize(serializer)
            }
        }
    }
}

impl ESTree for AccessorProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &self.r#type);
        state.serialize_field("decorators", &self.decorators);
        state.serialize_field("key", &self.key);
        state.serialize_ts_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("value", &self.value);
        state.serialize_field("computed", &self.computed);
        state.serialize_field("static", &self.r#static);
        state.serialize_ts_field("override", &self.r#override);
        state.serialize_ts_field("definite", &self.definite);
        state.serialize_ts_field("accessibility", &self.accessibility);
        state.serialize_ts_field("declare", &crate::serialize::basic::TsFalse(self));
        state.serialize_ts_field("optional", &crate::serialize::basic::TsFalse(self));
        state.serialize_ts_field("readonly", &crate::serialize::basic::TsFalse(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ImportExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ImportExpression"));
        state.serialize_field("source", &self.source);
        state.serialize_field("options", &self.options);
        state.serialize_field("phase", &self.phase);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ImportDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ImportDeclaration"));
        state.serialize_field(
            "specifiers",
            &crate::serialize::js::ImportDeclarationSpecifiers(self),
        );
        state.serialize_field("source", &self.source);
        state.serialize_field("phase", &self.phase);
        state.serialize_field(
            "attributes",
            &crate::serialize::js::ImportDeclarationWithClause(self),
        );
        state.serialize_ts_field("importKind", &self.import_kind);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ImportPhase {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Source => JsonSafeString("source").serialize(serializer),
            Self::Defer => JsonSafeString("defer").serialize(serializer),
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
        state.serialize_field("type", &JsonSafeString("ImportSpecifier"));
        state.serialize_field("imported", &self.imported);
        state.serialize_field("local", &self.local);
        state.serialize_ts_field("importKind", &self.import_kind);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ImportDefaultSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ImportDefaultSpecifier"));
        state.serialize_field("local", &self.local);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ImportNamespaceSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ImportNamespaceSpecifier"));
        state.serialize_field("local", &self.local);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for WithClause<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("attributes", &self.with_entries);
        state.end();
    }
}

impl ESTree for ImportAttribute<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ImportAttribute"));
        state.serialize_field("key", &self.key);
        state.serialize_field("value", &self.value);
        state.serialize_span(self.span);
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

impl ESTree for ExportNamedDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ExportNamedDeclaration"));
        state.serialize_field("declaration", &self.declaration);
        state.serialize_field("specifiers", &self.specifiers);
        state.serialize_field("source", &self.source);
        state.serialize_ts_field("exportKind", &self.export_kind);
        state.serialize_field(
            "attributes",
            &crate::serialize::js::ExportNamedDeclarationWithClause(self),
        );
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ExportDefaultDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ExportDefaultDeclaration"));
        state.serialize_field("declaration", &self.declaration);
        state.serialize_ts_field("exportKind", &crate::serialize::basic::TsValue(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ExportAllDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ExportAllDeclaration"));
        state.serialize_field("exported", &self.exported);
        state.serialize_field("source", &self.source);
        state.serialize_field(
            "attributes",
            &crate::serialize::js::ExportAllDeclarationWithClause(self),
        );
        state.serialize_ts_field("exportKind", &self.export_kind);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ExportSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ExportSpecifier"));
        state.serialize_field("local", &self.local);
        state.serialize_field("exported", &self.exported);
        state.serialize_ts_field("exportKind", &self.export_kind);
        state.serialize_span(self.span);
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
            Self::V8IntrinsicExpression(it) => it.serialize(serializer),
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

impl ESTree for V8IntrinsicExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("V8IntrinsicExpression"));
        state.serialize_field("name", &self.name);
        state.serialize_field("arguments", &self.arguments);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for BooleanLiteral {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Literal"));
        state.serialize_field("value", &self.value);
        state.serialize_field("raw", &crate::serialize::literal::BooleanLiteralRaw(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for NullLiteral {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Literal"));
        state.serialize_field("value", &crate::serialize::basic::Null(self));
        state.serialize_field("raw", &crate::serialize::literal::NullLiteralRaw(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for NumericLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Literal"));
        state.serialize_field("value", &self.value);
        state.serialize_field("raw", &self.raw.map(|s| JsonSafeString(s.as_str())));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for StringLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Literal"));
        state.serialize_field("value", &crate::serialize::literal::StringLiteralValue(self));
        state.serialize_field("raw", &self.raw);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for BigIntLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Literal"));
        state.serialize_field("value", &crate::serialize::literal::BigIntLiteralValue(self));
        state.serialize_field("raw", &self.raw.map(|s| JsonSafeString(s.as_str())));
        state.serialize_field("bigint", &crate::serialize::literal::BigIntLiteralBigint(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for RegExpLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Literal"));
        state.serialize_field("value", &crate::serialize::literal::RegExpLiteralValue(self));
        state.serialize_field("raw", &self.raw);
        state.serialize_field("regex", &self.regex);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for RegExp<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("pattern", &self.pattern.text);
        state.serialize_field("flags", &self.flags);
        state.end();
    }
}

impl ESTree for RegExpPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("pattern", &self.text);
        state.end();
    }
}

impl ESTree for RegExpFlags {
    fn serialize<S: Serializer>(&self, serializer: S) {
        crate::serialize::literal::RegExpFlagsConverter(self).serialize(serializer)
    }
}

impl ESTree for JSXElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXElement"));
        state.serialize_field(
            "openingElement",
            &crate::serialize::jsx::JSXElementOpeningElement(self),
        );
        state.serialize_field("children", &self.children);
        state.serialize_field("closingElement", &self.closing_element);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for JSXOpeningElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXOpeningElement"));
        state.serialize_field("name", &self.name);
        state.serialize_ts_field("typeArguments", &self.type_arguments);
        state.serialize_field("attributes", &self.attributes);
        state.serialize_field(
            "selfClosing",
            &crate::serialize::jsx::JSXOpeningElementSelfClosing(self),
        );
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for JSXClosingElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXClosingElement"));
        state.serialize_field("name", &self.name);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for JSXFragment<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXFragment"));
        state.serialize_field("openingFragment", &self.opening_fragment);
        state.serialize_field("children", &self.children);
        state.serialize_field("closingFragment", &self.closing_fragment);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for JSXOpeningFragment {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXOpeningFragment"));
        state.serialize_js_field("attributes", &crate::serialize::basic::JsEmptyArray(self));
        state.serialize_js_field("selfClosing", &crate::serialize::basic::JsFalse(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for JSXClosingFragment {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXClosingFragment"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for JSXElementName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Identifier(it) => it.serialize(serializer),
            Self::IdentifierReference(it) => {
                crate::serialize::jsx::JSXElementIdentifierReference(it).serialize(serializer)
            }
            Self::NamespacedName(it) => it.serialize(serializer),
            Self::MemberExpression(it) => it.serialize(serializer),
            Self::ThisExpression(it) => {
                crate::serialize::jsx::JSXElementThisExpression(it).serialize(serializer)
            }
        }
    }
}

impl ESTree for JSXNamespacedName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXNamespacedName"));
        state.serialize_field("namespace", &self.namespace);
        state.serialize_field("name", &self.name);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for JSXMemberExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXMemberExpression"));
        state.serialize_field("object", &self.object);
        state.serialize_field("property", &self.property);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for JSXMemberExpressionObject<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::IdentifierReference(it) => {
                crate::serialize::jsx::JSXElementIdentifierReference(it).serialize(serializer)
            }
            Self::MemberExpression(it) => it.serialize(serializer),
            Self::ThisExpression(it) => {
                crate::serialize::jsx::JSXElementThisExpression(it).serialize(serializer)
            }
        }
    }
}

impl ESTree for JSXExpressionContainer<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXExpressionContainer"));
        state.serialize_field("expression", &self.expression);
        state.serialize_span(self.span);
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
            Self::V8IntrinsicExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for JSXEmptyExpression {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXEmptyExpression"));
        state.serialize_span(self.span);
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
        state.serialize_field("type", &JsonSafeString("JSXAttribute"));
        state.serialize_field("name", &self.name);
        state.serialize_field("value", &self.value);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for JSXSpreadAttribute<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXSpreadAttribute"));
        state.serialize_field("argument", &self.argument);
        state.serialize_span(self.span);
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
        state.serialize_field("type", &JsonSafeString("JSXIdentifier"));
        state.serialize_field("name", &JsonSafeString(self.name.as_str()));
        state.serialize_span(self.span);
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
        state.serialize_field("type", &JsonSafeString("JSXSpreadChild"));
        state.serialize_field("expression", &self.expression);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for JSXText<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXText"));
        state.serialize_field("value", &self.value);
        state.serialize_field("raw", &self.raw);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSThisParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Identifier"));
        state.serialize_field("decorators", &crate::serialize::basic::EmptyArray(self));
        state.serialize_field("name", &crate::serialize::basic::This(self));
        state.serialize_field("optional", &crate::serialize::basic::False(self));
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSEnumDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSEnumDeclaration"));
        state.serialize_field("id", &self.id);
        state.serialize_field("body", &self.body);
        state.serialize_field("const", &self.r#const);
        state.serialize_field("declare", &self.declare);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSEnumBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSEnumBody"));
        state.serialize_field("members", &self.members);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSEnumMember<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSEnumMember"));
        state.serialize_field("id", &self.id);
        state.serialize_field("initializer", &self.initializer);
        state.serialize_field("computed", &crate::serialize::ts::TSEnumMemberComputed(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSEnumMemberName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Identifier(it) => it.serialize(serializer),
            Self::String(it) => it.serialize(serializer),
            Self::ComputedString(it) => it.serialize(serializer),
            Self::ComputedTemplateString(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSTypeAnnotation<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSTypeAnnotation"));
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSLiteralType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSLiteralType"));
        state.serialize_field("literal", &self.literal);
        state.serialize_span(self.span);
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
        state.serialize_field("type", &JsonSafeString("TSConditionalType"));
        state.serialize_field("checkType", &self.check_type);
        state.serialize_field("extendsType", &self.extends_type);
        state.serialize_field("trueType", &self.true_type);
        state.serialize_field("falseType", &self.false_type);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSUnionType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSUnionType"));
        state.serialize_field("types", &self.types);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSIntersectionType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSIntersectionType"));
        state.serialize_field("types", &self.types);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSParenthesizedType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        crate::serialize::ts::TSParenthesizedTypeConverter(self).serialize(serializer)
    }
}

impl ESTree for TSTypeOperator<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSTypeOperator"));
        state.serialize_field("operator", &self.operator);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSTypeOperatorOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Keyof => JsonSafeString("keyof").serialize(serializer),
            Self::Unique => JsonSafeString("unique").serialize(serializer),
            Self::Readonly => JsonSafeString("readonly").serialize(serializer),
        }
    }
}

impl ESTree for TSArrayType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSArrayType"));
        state.serialize_field("elementType", &self.element_type);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSIndexedAccessType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSIndexedAccessType"));
        state.serialize_field("objectType", &self.object_type);
        state.serialize_field("indexType", &self.index_type);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSTupleType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSTupleType"));
        state.serialize_field("elementTypes", &self.element_types);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSNamedTupleMember<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSNamedTupleMember"));
        state.serialize_field("label", &self.label);
        state.serialize_field("elementType", &self.element_type);
        state.serialize_field("optional", &self.optional);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSOptionalType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSOptionalType"));
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSRestType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSRestType"));
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_span(self.span);
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
        state.serialize_field("type", &JsonSafeString("TSAnyKeyword"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSStringKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSStringKeyword"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSBooleanKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSBooleanKeyword"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSNumberKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSNumberKeyword"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSNeverKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSNeverKeyword"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSIntrinsicKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSIntrinsicKeyword"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSUnknownKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSUnknownKeyword"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSNullKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSNullKeyword"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSUndefinedKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSUndefinedKeyword"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSVoidKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSVoidKeyword"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSSymbolKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSSymbolKeyword"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSThisType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSThisType"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSObjectKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSObjectKeyword"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSBigIntKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSBigIntKeyword"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSTypeReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSTypeReference"));
        state.serialize_field("typeName", &self.type_name);
        state.serialize_field("typeArguments", &self.type_arguments);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSTypeName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::IdentifierReference(it) => it.serialize(serializer),
            Self::QualifiedName(it) => it.serialize(serializer),
            Self::ThisExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSQualifiedName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSQualifiedName"));
        state.serialize_field("left", &self.left);
        state.serialize_field("right", &self.right);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSTypeParameterInstantiation<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSTypeParameterInstantiation"));
        state.serialize_field("params", &self.params);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSTypeParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSTypeParameter"));
        state.serialize_field("name", &self.name);
        state.serialize_field("constraint", &self.constraint);
        state.serialize_field("default", &self.default);
        state.serialize_field("in", &self.r#in);
        state.serialize_field("out", &self.out);
        state.serialize_field("const", &self.r#const);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSTypeParameterDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSTypeParameterDeclaration"));
        state.serialize_field("params", &self.params);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSTypeAliasDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSTypeAliasDeclaration"));
        state.serialize_field("id", &self.id);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("declare", &self.declare);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSAccessibility {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Private => JsonSafeString("private").serialize(serializer),
            Self::Protected => JsonSafeString("protected").serialize(serializer),
            Self::Public => JsonSafeString("public").serialize(serializer),
        }
    }
}

impl ESTree for TSClassImplements<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSClassImplements"));
        state.serialize_field(
            "expression",
            &crate::serialize::ts::TSClassImplementsExpression(self),
        );
        state.serialize_field("typeArguments", &self.type_arguments);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSInterfaceDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSInterfaceDeclaration"));
        state.serialize_field("id", &self.id);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("extends", &self.extends);
        state.serialize_field("body", &self.body);
        state.serialize_field("declare", &self.declare);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSInterfaceBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSInterfaceBody"));
        state.serialize_field("body", &self.body);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSPropertySignature<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSPropertySignature"));
        state.serialize_field("computed", &self.computed);
        state.serialize_field("optional", &self.optional);
        state.serialize_field("readonly", &self.readonly);
        state.serialize_field("key", &self.key);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("accessibility", &crate::serialize::basic::Null(self));
        state.serialize_field("static", &crate::serialize::basic::False(self));
        state.serialize_span(self.span);
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
        state.serialize_field("type", &JsonSafeString("TSIndexSignature"));
        state.serialize_field("parameters", &self.parameters);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("readonly", &self.readonly);
        state.serialize_field("static", &self.r#static);
        state.serialize_field("accessibility", &crate::serialize::basic::Null(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSCallSignatureDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSCallSignatureDeclaration"));
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field(
            "params",
            &crate::serialize::ts::TSCallSignatureDeclarationParams(self),
        );
        state.serialize_field("returnType", &self.return_type);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSMethodSignatureKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Method => JsonSafeString("method").serialize(serializer),
            Self::Get => JsonSafeString("get").serialize(serializer),
            Self::Set => JsonSafeString("set").serialize(serializer),
        }
    }
}

impl ESTree for TSMethodSignature<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSMethodSignature"));
        state.serialize_field("key", &self.key);
        state.serialize_field("computed", &self.computed);
        state.serialize_field("optional", &self.optional);
        state.serialize_field("kind", &self.kind);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("params", &crate::serialize::ts::TSMethodSignatureParams(self));
        state.serialize_field("returnType", &self.return_type);
        state.serialize_field("accessibility", &crate::serialize::basic::Null(self));
        state.serialize_field("readonly", &crate::serialize::basic::False(self));
        state.serialize_field("static", &crate::serialize::basic::False(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSConstructSignatureDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSConstructSignatureDeclaration"));
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("params", &self.params);
        state.serialize_field("returnType", &self.return_type);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSIndexSignatureName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Identifier"));
        state.serialize_field("decorators", &crate::serialize::basic::EmptyArray(self));
        state.serialize_field("name", &JsonSafeString(self.name.as_str()));
        state.serialize_field("optional", &crate::serialize::basic::False(self));
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSInterfaceHeritage<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSInterfaceHeritage"));
        state.serialize_field("expression", &self.expression);
        state.serialize_field("typeArguments", &self.type_arguments);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSTypePredicate<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSTypePredicate"));
        state.serialize_field("parameterName", &self.parameter_name);
        state.serialize_field("asserts", &self.asserts);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_span(self.span);
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
        crate::serialize::ts::TSModuleDeclarationConverter(self).serialize(serializer)
    }
}

impl ESTree for TSModuleDeclarationKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Module => JsonSafeString("module").serialize(serializer),
            Self::Namespace => JsonSafeString("namespace").serialize(serializer),
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

impl ESTree for TSGlobalDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSModuleDeclaration"));
        state.serialize_field("id", &crate::serialize::ts::TSGlobalDeclarationId(self));
        state.serialize_field("body", &self.body);
        state.serialize_field("kind", &crate::serialize::basic::Global(self));
        state.serialize_field("declare", &self.declare);
        state.serialize_field("global", &crate::serialize::basic::True(self));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSModuleBlock<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSModuleBlock"));
        state.serialize_field("body", &Concat2(&self.directives, &self.body));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSTypeLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSTypeLiteral"));
        state.serialize_field("members", &self.members);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSInferType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSInferType"));
        state.serialize_field("typeParameter", &self.type_parameter);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSTypeQuery<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSTypeQuery"));
        state.serialize_field("exprName", &self.expr_name);
        state.serialize_field("typeArguments", &self.type_arguments);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSTypeQueryExprName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::TSImportType(it) => it.serialize(serializer),
            Self::IdentifierReference(it) => it.serialize(serializer),
            Self::QualifiedName(it) => it.serialize(serializer),
            Self::ThisExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSImportType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSImportType"));
        state.serialize_field("source", &self.source);
        state.serialize_field("options", &self.options);
        state.serialize_field("qualifier", &self.qualifier);
        state.serialize_field("typeArguments", &self.type_arguments);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSImportTypeQualifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Identifier(it) => it.serialize(serializer),
            Self::QualifiedName(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSImportTypeQualifiedName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSQualifiedName"));
        state.serialize_field("left", &self.left);
        state.serialize_field("right", &self.right);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSFunctionType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSFunctionType"));
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("params", &crate::serialize::ts::TSFunctionTypeParams(self));
        state.serialize_field("returnType", &self.return_type);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSConstructorType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSConstructorType"));
        state.serialize_field("abstract", &self.r#abstract);
        state.serialize_field("typeParameters", &self.type_parameters);
        state.serialize_field("params", &self.params);
        state.serialize_field("returnType", &self.return_type);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSMappedType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSMappedType"));
        state.serialize_field("key", &crate::serialize::ts::TSMappedTypeKey(self));
        state.serialize_field("constraint", &crate::serialize::ts::TSMappedTypeConstraint(self));
        state.serialize_field("nameType", &self.name_type);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("optional", &crate::serialize::ts::TSMappedTypeOptional(self));
        state.serialize_field("readonly", &self.readonly);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSMappedTypeModifierOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::True => crate::serialize::basic::True(()).serialize(serializer),
            Self::Plus => JsonSafeString("+").serialize(serializer),
            Self::Minus => JsonSafeString("-").serialize(serializer),
        }
    }
}

impl ESTree for TSTemplateLiteralType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSTemplateLiteralType"));
        state.serialize_field("quasis", &self.quasis);
        state.serialize_field("types", &self.types);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSAsExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSAsExpression"));
        state.serialize_field("expression", &self.expression);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSSatisfiesExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSSatisfiesExpression"));
        state.serialize_field("expression", &self.expression);
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSTypeAssertion<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSTypeAssertion"));
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("expression", &self.expression);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSImportEqualsDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSImportEqualsDeclaration"));
        state.serialize_field("id", &self.id);
        state.serialize_field("moduleReference", &self.module_reference);
        state.serialize_field("importKind", &self.import_kind);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSModuleReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::ExternalModuleReference(it) => it.serialize(serializer),
            Self::IdentifierReference(it) => it.serialize(serializer),
            Self::QualifiedName(it) => it.serialize(serializer),
            Self::ThisExpression(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for TSExternalModuleReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSExternalModuleReference"));
        state.serialize_field("expression", &self.expression);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSNonNullExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSNonNullExpression"));
        state.serialize_field("expression", &self.expression);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for Decorator<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Decorator"));
        state.serialize_field("expression", &self.expression);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSExportAssignment<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSExportAssignment"));
        state.serialize_field("expression", &self.expression);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSNamespaceExportDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSNamespaceExportDeclaration"));
        state.serialize_field("id", &self.id);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for TSInstantiationExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSInstantiationExpression"));
        state.serialize_field("expression", &self.expression);
        state.serialize_field("typeArguments", &self.type_arguments);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for ImportOrExportKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Value => JsonSafeString("value").serialize(serializer),
            Self::Type => JsonSafeString("type").serialize(serializer),
        }
    }
}

impl ESTree for JSDocNullableType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSJSDocNullableType"));
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("postfix", &self.postfix);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for JSDocNonNullableType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSJSDocNonNullableType"));
        state.serialize_field("typeAnnotation", &self.type_annotation);
        state.serialize_field("postfix", &self.postfix);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for JSDocUnknownType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSJSDocUnknownType"));
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for CommentKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Line => JsonSafeString("Line").serialize(serializer),
            Self::SinglelineBlock => JsonSafeString("Block").serialize(serializer),
            Self::MultilineBlock => JsonSafeString("Block").serialize(serializer),
        }
    }
}

impl ESTree for Comment {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &self.kind);
        state.serialize_field("value", &crate::serialize::CommentValue(self));
        state.serialize_span(self.span);
        state.end();
    }
}
