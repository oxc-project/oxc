// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, clippy::match_same_arms)]

use serde::{__private::ser::FlatMapSerializer, ser::SerializeMap, Serialize, Serializer};

use oxc_estree::ser::AppendTo;

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl Serialize for Program<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Program")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        self.source_type.serialize(FlatMapSerializer(&mut map))?;
        map.serialize_entry("hashbang", &self.hashbang)?;
        map.serialize_entry("directives", &self.directives)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Expression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for IdentifierName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Identifier")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

impl Serialize for IdentifierReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Identifier")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

impl Serialize for BindingIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Identifier")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

impl Serialize for LabelIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Identifier")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

impl Serialize for ThisExpression {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ThisExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for ArrayExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ArrayExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("elements", &self.elements)?;
        map.end()
    }
}

impl Serialize for ArrayExpressionElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for ObjectExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ObjectExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("properties", &self.properties)?;
        map.end()
    }
}

impl Serialize for ObjectPropertyKind<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ObjectPropertyKind::ObjectProperty(it) => it.serialize(serializer),
            ObjectPropertyKind::SpreadProperty(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for ObjectProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ObjectProperty")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("method", &self.method)?;
        map.serialize_entry("shorthand", &self.shorthand)?;
        map.serialize_entry("computed", &self.computed)?;
        map.end()
    }
}

impl Serialize for PropertyKey<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for PropertyKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            PropertyKind::Init => serializer.serialize_unit_variant("PropertyKind", 0, "init"),
            PropertyKind::Get => serializer.serialize_unit_variant("PropertyKind", 1, "get"),
            PropertyKind::Set => serializer.serialize_unit_variant("PropertyKind", 2, "set"),
        }
    }
}

impl Serialize for TemplateLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TemplateLiteral")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("quasis", &self.quasis)?;
        map.serialize_entry("expressions", &self.expressions)?;
        map.end()
    }
}

impl Serialize for TaggedTemplateExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TaggedTemplateExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("tag", &self.tag)?;
        map.serialize_entry("quasi", &self.quasi)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

impl Serialize for TemplateElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TemplateElement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("tail", &self.tail)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for TemplateElementValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("raw", &self.raw)?;
        map.serialize_entry("cooked", &self.cooked)?;
        map.end()
    }
}

impl Serialize for MemberExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            MemberExpression::ComputedMemberExpression(it) => it.serialize(serializer),
            MemberExpression::StaticMemberExpression(it) => it.serialize(serializer),
            MemberExpression::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for ComputedMemberExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "MemberExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("object", &self.object)?;
        map.serialize_entry("property", &self.expression)?;
        map.serialize_entry("optional", &self.optional)?;
        map.serialize_entry("computed", &true)?;
        map.end()
    }
}

impl Serialize for StaticMemberExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "MemberExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("object", &self.object)?;
        map.serialize_entry("property", &self.property)?;
        map.serialize_entry("optional", &self.optional)?;
        map.serialize_entry("computed", &false)?;
        map.end()
    }
}

impl Serialize for PrivateFieldExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "MemberExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("object", &self.object)?;
        map.serialize_entry("field", &self.field)?;
        map.serialize_entry("optional", &self.optional)?;
        map.serialize_entry("computed", &false)?;
        map.end()
    }
}

impl Serialize for CallExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CallExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("callee", &self.callee)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("arguments", &self.arguments)?;
        map.serialize_entry("optional", &self.optional)?;
        map.end()
    }
}

impl Serialize for NewExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "NewExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("callee", &self.callee)?;
        map.serialize_entry("arguments", &self.arguments)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

impl Serialize for MetaProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "MetaProperty")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("meta", &self.meta)?;
        map.serialize_entry("property", &self.property)?;
        map.end()
    }
}

impl Serialize for SpreadElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "SpreadElement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for Argument<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for UpdateExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "UpdateExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("operator", &self.operator)?;
        map.serialize_entry("prefix", &self.prefix)?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for UnaryExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "UnaryExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("operator", &self.operator)?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for BinaryExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BinaryExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("operator", &self.operator)?;
        map.serialize_entry("right", &self.right)?;
        map.end()
    }
}

impl Serialize for PrivateInExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "PrivateInExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("operator", &self.operator)?;
        map.serialize_entry("right", &self.right)?;
        map.end()
    }
}

impl Serialize for LogicalExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "LogicalExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("operator", &self.operator)?;
        map.serialize_entry("right", &self.right)?;
        map.end()
    }
}

impl Serialize for ConditionalExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ConditionalExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("test", &self.test)?;
        map.serialize_entry("consequent", &self.consequent)?;
        map.serialize_entry("alternate", &self.alternate)?;
        map.end()
    }
}

impl Serialize for AssignmentExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AssignmentExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("operator", &self.operator)?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("right", &self.right)?;
        map.end()
    }
}

impl Serialize for AssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for SimpleAssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for AssignmentTargetPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            AssignmentTargetPattern::ArrayAssignmentTarget(it) => it.serialize(serializer),
            AssignmentTargetPattern::ObjectAssignmentTarget(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for ArrayAssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ArrayAssignmentTarget")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("elements", &AppendTo { array: &self.elements, after: &self.rest })?;
        map.end()
    }
}

impl Serialize for ObjectAssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ObjectAssignmentTarget")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry(
            "properties",
            &AppendTo { array: &self.properties, after: &self.rest },
        )?;
        map.end()
    }
}

impl Serialize for AssignmentTargetRest<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "RestElement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("argument", &self.target)?;
        map.end()
    }
}

impl Serialize for AssignmentTargetMaybeDefault<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for AssignmentTargetWithDefault<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AssignmentTargetWithDefault")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("binding", &self.binding)?;
        map.serialize_entry("init", &self.init)?;
        map.end()
    }
}

impl Serialize for AssignmentTargetProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for AssignmentTargetPropertyIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AssignmentTargetPropertyIdentifier")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("binding", &self.binding)?;
        map.serialize_entry("init", &self.init)?;
        map.end()
    }
}

impl Serialize for AssignmentTargetPropertyProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AssignmentTargetPropertyProperty")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("binding", &self.binding)?;
        map.serialize_entry("computed", &self.computed)?;
        map.end()
    }
}

impl Serialize for SequenceExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "SequenceExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expressions", &self.expressions)?;
        map.end()
    }
}

impl Serialize for Super {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Super")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for AwaitExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AwaitExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for ChainExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ChainExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for ChainElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ChainElement::CallExpression(it) => it.serialize(serializer),
            ChainElement::TSNonNullExpression(it) => it.serialize(serializer),
            ChainElement::ComputedMemberExpression(it) => it.serialize(serializer),
            ChainElement::StaticMemberExpression(it) => it.serialize(serializer),
            ChainElement::PrivateFieldExpression(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for ParenthesizedExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ParenthesizedExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for Statement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for Directive<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Directive")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("directive", &self.directive)?;
        map.end()
    }
}

impl Serialize for Hashbang<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Hashbang")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for BlockStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BlockStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Declaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for VariableDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "VariableDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("declarations", &self.declarations)?;
        map.serialize_entry("declare", &self.declare)?;
        map.end()
    }
}

impl Serialize for VariableDeclarationKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            VariableDeclarationKind::Var => {
                serializer.serialize_unit_variant("VariableDeclarationKind", 0, "var")
            }
            VariableDeclarationKind::Const => {
                serializer.serialize_unit_variant("VariableDeclarationKind", 1, "const")
            }
            VariableDeclarationKind::Let => {
                serializer.serialize_unit_variant("VariableDeclarationKind", 2, "let")
            }
            VariableDeclarationKind::Using => {
                serializer.serialize_unit_variant("VariableDeclarationKind", 3, "using")
            }
            VariableDeclarationKind::AwaitUsing => {
                serializer.serialize_unit_variant("VariableDeclarationKind", 4, "await using")
            }
        }
    }
}

impl Serialize for VariableDeclarator<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "VariableDeclarator")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("init", &self.init)?;
        map.serialize_entry("definite", &self.definite)?;
        map.end()
    }
}

impl Serialize for EmptyStatement {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "EmptyStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for ExpressionStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ExpressionStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for IfStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "IfStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("test", &self.test)?;
        map.serialize_entry("consequent", &self.consequent)?;
        map.serialize_entry("alternate", &self.alternate)?;
        map.end()
    }
}

impl Serialize for DoWhileStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "DoWhileStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("body", &self.body)?;
        map.serialize_entry("test", &self.test)?;
        map.end()
    }
}

impl Serialize for WhileStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "WhileStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("test", &self.test)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ForStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ForStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("init", &self.init)?;
        map.serialize_entry("test", &self.test)?;
        map.serialize_entry("update", &self.update)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ForStatementInit<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for ForInStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ForInStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("right", &self.right)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ForStatementLeft<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for ForOfStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ForOfStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("await", &self.r#await)?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("right", &self.right)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ContinueStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ContinueStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("label", &self.label)?;
        map.end()
    }
}

impl Serialize for BreakStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BreakStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("label", &self.label)?;
        map.end()
    }
}

impl Serialize for ReturnStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ReturnStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for WithStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "WithStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("object", &self.object)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for SwitchStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "SwitchStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("discriminant", &self.discriminant)?;
        map.serialize_entry("cases", &self.cases)?;
        map.end()
    }
}

impl Serialize for SwitchCase<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "SwitchCase")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("test", &self.test)?;
        map.serialize_entry("consequent", &self.consequent)?;
        map.end()
    }
}

impl Serialize for LabeledStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "LabeledStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("label", &self.label)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ThrowStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ThrowStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for TryStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TryStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("block", &self.block)?;
        map.serialize_entry("handler", &self.handler)?;
        map.serialize_entry("finalizer", &self.finalizer)?;
        map.end()
    }
}

impl Serialize for CatchClause<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CatchClause")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("param", &self.param)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for CatchParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CatchParameter")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("pattern", &self.pattern)?;
        map.end()
    }
}

impl Serialize for DebuggerStatement {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "DebuggerStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for BindingPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        self.kind.serialize(FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("optional", &self.optional)?;
        map.end()
    }
}

impl Serialize for BindingPatternKind<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            BindingPatternKind::BindingIdentifier(it) => it.serialize(serializer),
            BindingPatternKind::ObjectPattern(it) => it.serialize(serializer),
            BindingPatternKind::ArrayPattern(it) => it.serialize(serializer),
            BindingPatternKind::AssignmentPattern(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for AssignmentPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AssignmentPattern")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("right", &self.right)?;
        map.end()
    }
}

impl Serialize for ObjectPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ObjectPattern")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry(
            "properties",
            &AppendTo { array: &self.properties, after: &self.rest },
        )?;
        map.end()
    }
}

impl Serialize for BindingProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BindingProperty")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("shorthand", &self.shorthand)?;
        map.serialize_entry("computed", &self.computed)?;
        map.end()
    }
}

impl Serialize for ArrayPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ArrayPattern")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("elements", &AppendTo { array: &self.elements, after: &self.rest })?;
        map.end()
    }
}

impl Serialize for BindingRestElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "RestElement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for Function<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("type", &self.r#type)?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("generator", &self.generator)?;
        map.serialize_entry("async", &self.r#async)?;
        map.serialize_entry("declare", &self.declare)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("thisParam", &self.this_param)?;
        map.serialize_entry("params", &self.params)?;
        map.serialize_entry("returnType", &self.return_type)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for FunctionType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            FunctionType::FunctionDeclaration => {
                serializer.serialize_unit_variant("FunctionType", 0, "FunctionDeclaration")
            }
            FunctionType::FunctionExpression => {
                serializer.serialize_unit_variant("FunctionType", 1, "FunctionExpression")
            }
            FunctionType::TSDeclareFunction => {
                serializer.serialize_unit_variant("FunctionType", 2, "TSDeclareFunction")
            }
            FunctionType::TSEmptyBodyFunctionExpression => serializer.serialize_unit_variant(
                "FunctionType",
                3,
                "TSEmptyBodyFunctionExpression",
            ),
        }
    }
}

impl Serialize for FormalParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("decorators", &self.decorators)?;
        self.pattern.kind.serialize(FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.pattern.type_annotation)?;
        map.serialize_entry("optional", &self.pattern.optional)?;
        map.serialize_entry("accessibility", &self.accessibility)?;
        map.serialize_entry("readonly", &self.readonly)?;
        map.serialize_entry("override", &self.r#override)?;
        map.end()
    }
}

impl Serialize for FormalParameterKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            FormalParameterKind::FormalParameter => {
                serializer.serialize_unit_variant("FormalParameterKind", 0, "FormalParameter")
            }
            FormalParameterKind::UniqueFormalParameters => serializer.serialize_unit_variant(
                "FormalParameterKind",
                1,
                "UniqueFormalParameters",
            ),
            FormalParameterKind::ArrowFormalParameters => {
                serializer.serialize_unit_variant("FormalParameterKind", 2, "ArrowFormalParameters")
            }
            FormalParameterKind::Signature => {
                serializer.serialize_unit_variant("FormalParameterKind", 3, "Signature")
            }
        }
    }
}

impl Serialize for FunctionBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "FunctionBody")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("directives", &self.directives)?;
        map.serialize_entry("statements", &self.statements)?;
        map.end()
    }
}

impl Serialize for ArrowFunctionExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ArrowFunctionExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("async", &self.r#async)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("params", &self.params)?;
        map.serialize_entry("returnType", &self.return_type)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for YieldExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "YieldExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("delegate", &self.delegate)?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for Class<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("type", &self.r#type)?;
        map.serialize_entry("decorators", &self.decorators)?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("superClass", &self.super_class)?;
        map.serialize_entry("superTypeParameters", &self.super_type_parameters)?;
        map.serialize_entry("implements", &self.implements)?;
        map.serialize_entry("body", &self.body)?;
        map.serialize_entry("abstract", &self.r#abstract)?;
        map.serialize_entry("declare", &self.declare)?;
        map.end()
    }
}

impl Serialize for ClassType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ClassType::ClassDeclaration => {
                serializer.serialize_unit_variant("ClassType", 0, "ClassDeclaration")
            }
            ClassType::ClassExpression => {
                serializer.serialize_unit_variant("ClassType", 1, "ClassExpression")
            }
        }
    }
}

impl Serialize for ClassBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ClassBody")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ClassElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ClassElement::StaticBlock(it) => it.serialize(serializer),
            ClassElement::MethodDefinition(it) => it.serialize(serializer),
            ClassElement::PropertyDefinition(it) => it.serialize(serializer),
            ClassElement::AccessorProperty(it) => it.serialize(serializer),
            ClassElement::TSIndexSignature(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for MethodDefinition<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("type", &self.r#type)?;
        map.serialize_entry("decorators", &self.decorators)?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("computed", &self.computed)?;
        map.serialize_entry("static", &self.r#static)?;
        map.serialize_entry("override", &self.r#override)?;
        map.serialize_entry("optional", &self.optional)?;
        map.serialize_entry("accessibility", &self.accessibility)?;
        map.end()
    }
}

impl Serialize for MethodDefinitionType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            MethodDefinitionType::MethodDefinition => {
                serializer.serialize_unit_variant("MethodDefinitionType", 0, "MethodDefinition")
            }
            MethodDefinitionType::TSAbstractMethodDefinition => serializer.serialize_unit_variant(
                "MethodDefinitionType",
                1,
                "TSAbstractMethodDefinition",
            ),
        }
    }
}

impl Serialize for PropertyDefinition<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("type", &self.r#type)?;
        map.serialize_entry("decorators", &self.decorators)?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("computed", &self.computed)?;
        map.serialize_entry("static", &self.r#static)?;
        map.serialize_entry("declare", &self.declare)?;
        map.serialize_entry("override", &self.r#override)?;
        map.serialize_entry("optional", &self.optional)?;
        map.serialize_entry("definite", &self.definite)?;
        map.serialize_entry("readonly", &self.readonly)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("accessibility", &self.accessibility)?;
        map.end()
    }
}

impl Serialize for PropertyDefinitionType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            PropertyDefinitionType::PropertyDefinition => {
                serializer.serialize_unit_variant("PropertyDefinitionType", 0, "PropertyDefinition")
            }
            PropertyDefinitionType::TSAbstractPropertyDefinition => serializer
                .serialize_unit_variant(
                    "PropertyDefinitionType",
                    1,
                    "TSAbstractPropertyDefinition",
                ),
        }
    }
}

impl Serialize for MethodDefinitionKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            MethodDefinitionKind::Constructor => {
                serializer.serialize_unit_variant("MethodDefinitionKind", 0, "constructor")
            }
            MethodDefinitionKind::Method => {
                serializer.serialize_unit_variant("MethodDefinitionKind", 1, "method")
            }
            MethodDefinitionKind::Get => {
                serializer.serialize_unit_variant("MethodDefinitionKind", 2, "get")
            }
            MethodDefinitionKind::Set => {
                serializer.serialize_unit_variant("MethodDefinitionKind", 3, "set")
            }
        }
    }
}

impl Serialize for PrivateIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "PrivateIdentifier")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

impl Serialize for StaticBlock<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "StaticBlock")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ModuleDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for AccessorPropertyType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            AccessorPropertyType::AccessorProperty => {
                serializer.serialize_unit_variant("AccessorPropertyType", 0, "AccessorProperty")
            }
            AccessorPropertyType::TSAbstractAccessorProperty => serializer.serialize_unit_variant(
                "AccessorPropertyType",
                1,
                "TSAbstractAccessorProperty",
            ),
        }
    }
}

impl Serialize for AccessorProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("type", &self.r#type)?;
        map.serialize_entry("decorators", &self.decorators)?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("computed", &self.computed)?;
        map.serialize_entry("static", &self.r#static)?;
        map.serialize_entry("definite", &self.definite)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("accessibility", &self.accessibility)?;
        map.end()
    }
}

impl Serialize for ImportExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("source", &self.source)?;
        map.serialize_entry("arguments", &self.arguments)?;
        map.serialize_entry("phase", &self.phase)?;
        map.end()
    }
}

impl Serialize for ImportDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry(
            "specifiers",
            &crate::serialize::OptionVecDefault::from(&self.specifiers),
        )?;
        map.serialize_entry("source", &self.source)?;
        map.serialize_entry("phase", &self.phase)?;
        map.serialize_entry("withClause", &self.with_clause)?;
        map.serialize_entry("importKind", &self.import_kind)?;
        map.end()
    }
}

impl Serialize for ImportPhase {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ImportPhase::Source => serializer.serialize_unit_variant("ImportPhase", 0, "source"),
            ImportPhase::Defer => serializer.serialize_unit_variant("ImportPhase", 1, "defer"),
        }
    }
}

impl Serialize for ImportDeclarationSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ImportDeclarationSpecifier::ImportSpecifier(it) => it.serialize(serializer),
            ImportDeclarationSpecifier::ImportDefaultSpecifier(it) => it.serialize(serializer),
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for ImportSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportSpecifier")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("imported", &self.imported)?;
        map.serialize_entry("local", &self.local)?;
        map.serialize_entry("importKind", &self.import_kind)?;
        map.end()
    }
}

impl Serialize for ImportDefaultSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportDefaultSpecifier")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("local", &self.local)?;
        map.end()
    }
}

impl Serialize for ImportNamespaceSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportNamespaceSpecifier")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("local", &self.local)?;
        map.end()
    }
}

impl Serialize for WithClause<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "WithClause")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("attributesKeyword", &self.attributes_keyword)?;
        map.serialize_entry("withEntries", &self.with_entries)?;
        map.end()
    }
}

impl Serialize for ImportAttribute<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportAttribute")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for ImportAttributeKey<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ImportAttributeKey::Identifier(it) => it.serialize(serializer),
            ImportAttributeKey::StringLiteral(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for ExportNamedDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ExportNamedDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("declaration", &self.declaration)?;
        map.serialize_entry("specifiers", &self.specifiers)?;
        map.serialize_entry("source", &self.source)?;
        map.serialize_entry("exportKind", &self.export_kind)?;
        map.serialize_entry("withClause", &self.with_clause)?;
        map.end()
    }
}

impl Serialize for ExportDefaultDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ExportDefaultDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("declaration", &self.declaration)?;
        map.serialize_entry("exported", &self.exported)?;
        map.end()
    }
}

impl Serialize for ExportAllDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ExportAllDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("exported", &self.exported)?;
        map.serialize_entry("source", &self.source)?;
        map.serialize_entry("withClause", &self.with_clause)?;
        map.serialize_entry("exportKind", &self.export_kind)?;
        map.end()
    }
}

impl Serialize for ExportSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ExportSpecifier")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("local", &self.local)?;
        map.serialize_entry("exported", &self.exported)?;
        map.serialize_entry("exportKind", &self.export_kind)?;
        map.end()
    }
}

impl Serialize for ExportDefaultDeclarationKind<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for ModuleExportName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ModuleExportName::IdentifierName(it) => it.serialize(serializer),
            ModuleExportName::IdentifierReference(it) => it.serialize(serializer),
            ModuleExportName::StringLiteral(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for BooleanLiteral {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        crate::serialize::ESTreeLiteral::from(self).serialize(serializer)
    }
}

impl Serialize for NullLiteral {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        crate::serialize::ESTreeLiteral::from(self).serialize(serializer)
    }
}

impl Serialize for NumericLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        crate::serialize::ESTreeLiteral::from(self).serialize(serializer)
    }
}

impl Serialize for StringLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        crate::serialize::ESTreeLiteral::from(self).serialize(serializer)
    }
}

impl Serialize for BigIntLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        crate::serialize::ESTreeLiteral::from(self).serialize(serializer)
    }
}

impl Serialize for RegExpLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        crate::serialize::ESTreeLiteral::from(self).serialize(serializer)
    }
}

impl Serialize for RegExp<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("pattern", &self.pattern)?;
        map.serialize_entry("flags", &self.flags)?;
        map.end()
    }
}

impl Serialize for RegExpPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            RegExpPattern::Raw(it) => it.serialize(serializer),
            RegExpPattern::Invalid(it) => it.serialize(serializer),
            RegExpPattern::Pattern(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for JSXElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXElement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("openingElement", &self.opening_element)?;
        map.serialize_entry("closingElement", &self.closing_element)?;
        map.serialize_entry("children", &self.children)?;
        map.end()
    }
}

impl Serialize for JSXOpeningElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXOpeningElement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("selfClosing", &self.self_closing)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("attributes", &self.attributes)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

impl Serialize for JSXClosingElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXClosingElement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

impl Serialize for JSXFragment<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXFragment")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("openingFragment", &self.opening_fragment)?;
        map.serialize_entry("closingFragment", &self.closing_fragment)?;
        map.serialize_entry("children", &self.children)?;
        map.end()
    }
}

impl Serialize for JSXOpeningFragment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXOpeningFragment")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for JSXClosingFragment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXClosingFragment")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for JSXNamespacedName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXNamespacedName")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("namespace", &self.namespace)?;
        map.serialize_entry("property", &self.property)?;
        map.end()
    }
}

impl Serialize for JSXMemberExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXMemberExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("object", &self.object)?;
        map.serialize_entry("property", &self.property)?;
        map.end()
    }
}

impl Serialize for JSXExpressionContainer<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXExpressionContainer")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for JSXExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for JSXEmptyExpression {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXEmptyExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for JSXAttributeItem<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JSXAttributeItem::Attribute(it) => it.serialize(serializer),
            JSXAttributeItem::SpreadAttribute(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for JSXAttribute<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXAttribute")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for JSXSpreadAttribute<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXSpreadAttribute")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for JSXAttributeName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JSXAttributeName::Identifier(it) => it.serialize(serializer),
            JSXAttributeName::NamespacedName(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for JSXAttributeValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JSXAttributeValue::StringLiteral(it) => it.serialize(serializer),
            JSXAttributeValue::ExpressionContainer(it) => it.serialize(serializer),
            JSXAttributeValue::Element(it) => it.serialize(serializer),
            JSXAttributeValue::Fragment(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for JSXIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXIdentifier")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

impl Serialize for JSXChild<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JSXChild::Text(it) => it.serialize(serializer),
            JSXChild::Element(it) => it.serialize(serializer),
            JSXChild::Fragment(it) => it.serialize(serializer),
            JSXChild::ExpressionContainer(it) => it.serialize(serializer),
            JSXChild::Spread(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for JSXSpreadChild<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXSpreadChild")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for JSXText<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXText")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for TSThisParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSThisParameter")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSEnumDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSEnumDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("members", &self.members)?;
        map.serialize_entry("const", &self.r#const)?;
        map.serialize_entry("declare", &self.declare)?;
        map.end()
    }
}

impl Serialize for TSEnumMember<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSEnumMember")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("initializer", &self.initializer)?;
        map.end()
    }
}

impl Serialize for TSEnumMemberName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSEnumMemberName::Identifier(it) => it.serialize(serializer),
            TSEnumMemberName::String(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for TSTypeAnnotation<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeAnnotation")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSLiteralType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSLiteralType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("literal", &self.literal)?;
        map.end()
    }
}

impl Serialize for TSLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSLiteral::BooleanLiteral(it) => it.serialize(serializer),
            TSLiteral::NullLiteral(it) => it.serialize(serializer),
            TSLiteral::NumericLiteral(it) => it.serialize(serializer),
            TSLiteral::BigIntLiteral(it) => it.serialize(serializer),
            TSLiteral::RegExpLiteral(it) => it.serialize(serializer),
            TSLiteral::StringLiteral(it) => it.serialize(serializer),
            TSLiteral::TemplateLiteral(it) => it.serialize(serializer),
            TSLiteral::UnaryExpression(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for TSType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
            TSType::TSQualifiedName(it) => it.serialize(serializer),
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

impl Serialize for TSConditionalType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSConditionalType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("checkType", &self.check_type)?;
        map.serialize_entry("extendsType", &self.extends_type)?;
        map.serialize_entry("trueType", &self.true_type)?;
        map.serialize_entry("falseType", &self.false_type)?;
        map.end()
    }
}

impl Serialize for TSUnionType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSUnionType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("types", &self.types)?;
        map.end()
    }
}

impl Serialize for TSIntersectionType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSIntersectionType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("types", &self.types)?;
        map.end()
    }
}

impl Serialize for TSParenthesizedType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSParenthesizedType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSTypeOperator<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeOperator")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("operator", &self.operator)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSTypeOperatorOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSTypeOperatorOperator::Keyof => {
                serializer.serialize_unit_variant("TSTypeOperatorOperator", 0, "keyof")
            }
            TSTypeOperatorOperator::Unique => {
                serializer.serialize_unit_variant("TSTypeOperatorOperator", 1, "unique")
            }
            TSTypeOperatorOperator::Readonly => {
                serializer.serialize_unit_variant("TSTypeOperatorOperator", 2, "readonly")
            }
        }
    }
}

impl Serialize for TSArrayType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSArrayType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("elementType", &self.element_type)?;
        map.end()
    }
}

impl Serialize for TSIndexedAccessType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSIndexedAccessType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("objectType", &self.object_type)?;
        map.serialize_entry("indexType", &self.index_type)?;
        map.end()
    }
}

impl Serialize for TSTupleType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTupleType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("elementTypes", &self.element_types)?;
        map.end()
    }
}

impl Serialize for TSNamedTupleMember<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNamedTupleMember")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("elementType", &self.element_type)?;
        map.serialize_entry("label", &self.label)?;
        map.serialize_entry("optional", &self.optional)?;
        map.end()
    }
}

impl Serialize for TSOptionalType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSOptionalType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSRestType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSRestType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSTupleElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
            TSTupleElement::TSQualifiedName(it) => it.serialize(serializer),
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

impl Serialize for TSAnyKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSAnyKeyword")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for TSStringKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSStringKeyword")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for TSBooleanKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSBooleanKeyword")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for TSNumberKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNumberKeyword")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for TSNeverKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNeverKeyword")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for TSIntrinsicKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSIntrinsicKeyword")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for TSUnknownKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSUnknownKeyword")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for TSNullKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNullKeyword")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for TSUndefinedKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSUndefinedKeyword")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for TSVoidKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSVoidKeyword")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for TSSymbolKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSSymbolKeyword")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for TSThisType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSThisType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for TSObjectKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSObjectKeyword")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for TSBigIntKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSBigIntKeyword")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for TSTypeReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeReference")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("typeName", &self.type_name)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

impl Serialize for TSTypeName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSTypeName::IdentifierReference(it) => it.serialize(serializer),
            TSTypeName::QualifiedName(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for TSQualifiedName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSQualifiedName")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("right", &self.right)?;
        map.end()
    }
}

impl Serialize for TSTypeParameterInstantiation<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeParameterInstantiation")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("params", &self.params)?;
        map.end()
    }
}

impl Serialize for TSTypeParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeParameter")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("constraint", &self.constraint)?;
        map.serialize_entry("default", &self.default)?;
        map.serialize_entry("in", &self.r#in)?;
        map.serialize_entry("out", &self.out)?;
        map.serialize_entry("const", &self.r#const)?;
        map.end()
    }
}

impl Serialize for TSTypeParameterDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeParameterDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("params", &self.params)?;
        map.end()
    }
}

impl Serialize for TSTypeAliasDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeAliasDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("declare", &self.declare)?;
        map.end()
    }
}

impl Serialize for TSAccessibility {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSAccessibility::Private => {
                serializer.serialize_unit_variant("TSAccessibility", 0, "private")
            }
            TSAccessibility::Protected => {
                serializer.serialize_unit_variant("TSAccessibility", 1, "protected")
            }
            TSAccessibility::Public => {
                serializer.serialize_unit_variant("TSAccessibility", 2, "public")
            }
        }
    }
}

impl Serialize for TSClassImplements<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSClassImplements")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

impl Serialize for TSInterfaceDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSInterfaceDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("extends", &self.extends)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("body", &self.body)?;
        map.serialize_entry("declare", &self.declare)?;
        map.end()
    }
}

impl Serialize for TSInterfaceBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSInterfaceBody")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for TSPropertySignature<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSPropertySignature")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("computed", &self.computed)?;
        map.serialize_entry("optional", &self.optional)?;
        map.serialize_entry("readonly", &self.readonly)?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSSignature<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSSignature::TSIndexSignature(it) => it.serialize(serializer),
            TSSignature::TSPropertySignature(it) => it.serialize(serializer),
            TSSignature::TSCallSignatureDeclaration(it) => it.serialize(serializer),
            TSSignature::TSConstructSignatureDeclaration(it) => it.serialize(serializer),
            TSSignature::TSMethodSignature(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for TSIndexSignature<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSIndexSignature")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("parameters", &self.parameters)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("readonly", &self.readonly)?;
        map.serialize_entry("static", &self.r#static)?;
        map.end()
    }
}

impl Serialize for TSCallSignatureDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSCallSignatureDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("thisParam", &self.this_param)?;
        map.serialize_entry("params", &self.params)?;
        map.serialize_entry("returnType", &self.return_type)?;
        map.end()
    }
}

impl Serialize for TSMethodSignatureKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSMethodSignatureKind::Method => {
                serializer.serialize_unit_variant("TSMethodSignatureKind", 0, "method")
            }
            TSMethodSignatureKind::Get => {
                serializer.serialize_unit_variant("TSMethodSignatureKind", 1, "get")
            }
            TSMethodSignatureKind::Set => {
                serializer.serialize_unit_variant("TSMethodSignatureKind", 2, "set")
            }
        }
    }
}

impl Serialize for TSMethodSignature<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSMethodSignature")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("computed", &self.computed)?;
        map.serialize_entry("optional", &self.optional)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("thisParam", &self.this_param)?;
        map.serialize_entry("params", &self.params)?;
        map.serialize_entry("returnType", &self.return_type)?;
        map.end()
    }
}

impl Serialize for TSConstructSignatureDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSConstructSignatureDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("params", &self.params)?;
        map.serialize_entry("returnType", &self.return_type)?;
        map.end()
    }
}

impl Serialize for TSIndexSignatureName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Identifier")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSInterfaceHeritage<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSInterfaceHeritage")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

impl Serialize for TSTypePredicate<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypePredicate")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("parameterName", &self.parameter_name)?;
        map.serialize_entry("asserts", &self.asserts)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSTypePredicateName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSTypePredicateName::Identifier(it) => it.serialize(serializer),
            TSTypePredicateName::This(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for TSModuleDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSModuleDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("body", &self.body)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("declare", &self.declare)?;
        map.end()
    }
}

impl Serialize for TSModuleDeclarationKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSModuleDeclarationKind::Global => {
                serializer.serialize_unit_variant("TSModuleDeclarationKind", 0, "global")
            }
            TSModuleDeclarationKind::Module => {
                serializer.serialize_unit_variant("TSModuleDeclarationKind", 1, "module")
            }
            TSModuleDeclarationKind::Namespace => {
                serializer.serialize_unit_variant("TSModuleDeclarationKind", 2, "namespace")
            }
        }
    }
}

impl Serialize for TSModuleDeclarationName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSModuleDeclarationName::Identifier(it) => it.serialize(serializer),
            TSModuleDeclarationName::StringLiteral(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for TSModuleDeclarationBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSModuleDeclarationBody::TSModuleDeclaration(it) => it.serialize(serializer),
            TSModuleDeclarationBody::TSModuleBlock(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for TSTypeLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeLiteral")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("members", &self.members)?;
        map.end()
    }
}

impl Serialize for TSInferType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSInferType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("typeParameter", &self.type_parameter)?;
        map.end()
    }
}

impl Serialize for TSTypeQuery<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeQuery")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("exprName", &self.expr_name)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

impl Serialize for TSTypeQueryExprName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSTypeQueryExprName::TSImportType(it) => it.serialize(serializer),
            TSTypeQueryExprName::IdentifierReference(it) => it.serialize(serializer),
            TSTypeQueryExprName::QualifiedName(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for TSImportType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSImportType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("isTypeOf", &self.is_type_of)?;
        map.serialize_entry("parameter", &self.parameter)?;
        map.serialize_entry("qualifier", &self.qualifier)?;
        map.serialize_entry("attributes", &self.attributes)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

impl Serialize for TSImportAttributes<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSImportAttributes")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("attributesKeyword", &self.attributes_keyword)?;
        map.serialize_entry("elements", &self.elements)?;
        map.end()
    }
}

impl Serialize for TSImportAttribute<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSImportAttribute")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for TSImportAttributeName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSImportAttributeName::Identifier(it) => it.serialize(serializer),
            TSImportAttributeName::StringLiteral(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for TSFunctionType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSFunctionType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("thisParam", &self.this_param)?;
        map.serialize_entry("params", &self.params)?;
        map.serialize_entry("returnType", &self.return_type)?;
        map.end()
    }
}

impl Serialize for TSConstructorType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSConstructorType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("abstract", &self.r#abstract)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("params", &self.params)?;
        map.serialize_entry("returnType", &self.return_type)?;
        map.end()
    }
}

impl Serialize for TSMappedType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSMappedType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("typeParameter", &self.type_parameter)?;
        map.serialize_entry("nameType", &self.name_type)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("optional", &self.optional)?;
        map.serialize_entry("readonly", &self.readonly)?;
        map.end()
    }
}

impl Serialize for TSMappedTypeModifierOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSMappedTypeModifierOperator::True => {
                serializer.serialize_unit_variant("TSMappedTypeModifierOperator", 0, "true")
            }
            TSMappedTypeModifierOperator::Plus => {
                serializer.serialize_unit_variant("TSMappedTypeModifierOperator", 1, "+")
            }
            TSMappedTypeModifierOperator::Minus => {
                serializer.serialize_unit_variant("TSMappedTypeModifierOperator", 2, "-")
            }
            TSMappedTypeModifierOperator::None => {
                serializer.serialize_unit_variant("TSMappedTypeModifierOperator", 3, "none")
            }
        }
    }
}

impl Serialize for TSTemplateLiteralType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTemplateLiteralType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("quasis", &self.quasis)?;
        map.serialize_entry("types", &self.types)?;
        map.end()
    }
}

impl Serialize for TSAsExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSAsExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSSatisfiesExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSSatisfiesExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSTypeAssertion<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeAssertion")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSImportEqualsDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSImportEqualsDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("moduleReference", &self.module_reference)?;
        map.serialize_entry("importKind", &self.import_kind)?;
        map.end()
    }
}

impl Serialize for TSModuleReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSModuleReference::ExternalModuleReference(it) => it.serialize(serializer),
            TSModuleReference::IdentifierReference(it) => it.serialize(serializer),
            TSModuleReference::QualifiedName(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for TSExternalModuleReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSExternalModuleReference")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for TSNonNullExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNonNullExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for Decorator<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Decorator")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for TSExportAssignment<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSExportAssignment")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for TSNamespaceExportDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNamespaceExportDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("id", &self.id)?;
        map.end()
    }
}

impl Serialize for TSInstantiationExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSInstantiationExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

impl Serialize for ImportOrExportKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ImportOrExportKind::Value => {
                serializer.serialize_unit_variant("ImportOrExportKind", 0, "value")
            }
            ImportOrExportKind::Type => {
                serializer.serialize_unit_variant("ImportOrExportKind", 1, "type")
            }
        }
    }
}

impl Serialize for JSDocNullableType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSDocNullableType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("postfix", &self.postfix)?;
        map.end()
    }
}

impl Serialize for JSDocNonNullableType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSDocNonNullableType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("postfix", &self.postfix)?;
        map.end()
    }
}

impl Serialize for JSDocUnknownType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSDocUnknownType")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}
